/// socket/mod.rs
/// Serveur WebSocket — gère les sessions, reçoit les EVENT,
/// dispatche vers PHP, renvoie les PATCH.

use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tracing::{info, warn, error};
use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::{
    extract::{Query, State, Path as AxPath},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum::http::{StatusCode, header};
use std::collections::HashMap;

use crate::compiler::{NhtmlCompiler, CompileResult};
use crate::compiler::handler_table::{HandlerTable, build_from_tree};
use crate::proto;
use crate::core::SessionState;
use crate::session::SessionManager;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn verify_hmac(secret: &[u8], data: &[u8], signature: &[u8]) -> bool {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size");
    mac.update(data);
    mac.verify_slice(signature).is_ok()
}

// ─── État de session ────────────────────────────────────────────────────────

pub struct Session {
    pub state: SessionState,
    pub php_script: String,
    pub handler_table: HandlerTable,
    pub sm: Arc<SessionManager>,
    pub fpm_addr: Option<String>,
}

impl Session {
    fn new(id: String, result: &CompileResult, php_script: String, sm: Arc<SessionManager>, fpm_addr: Option<String>) -> Self {
        let handler_table = build_from_tree(&result.root);
        Self { 
            state: SessionState::new(id), 
            php_script, 
            handler_table,
            sm,
            fpm_addr
        }
    }
}

// ─── Point d'entrée ─────────────────────────────────────────────────────────

pub async fn serve(
    port: u16, 
    root: String, 
    _entry: String, 
    php: String, 
    fpm_addr: Option<String>,
    sm: Arc<SessionManager>, 
    tx_monitor: tokio::sync::broadcast::Sender<crate::MonitoringEvent>, 
    tx_app_broadcast: tokio::sync::broadcast::Sender<Vec<u8>>
) {
    let shared_state = Arc::new(GatewayState {
        root: root.clone(),
        sm: sm.clone(),
        fpm_addr,
        tx_monitor,
        tx_app_broadcast,
    });

    let app = Router::new()
        .route("/ws", get(handle_ws_route))
        .route("/", get(handle_http_root))
        .route("/*path", get(handle_http))
        .with_state(shared_state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Impossible de lier le port");
    info!("Gateway Industrielle en écoute sur http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn handle_ws_route(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<GatewayState>>,
    ws: WebSocketUpgrade,
) -> Response {
    let sid = params.get("sid").cloned().unwrap_or_else(|| "AUTO".to_string());
    let path = params.get("path").cloned().unwrap_or_else(|| "".to_string());
    info!("[WS] Upgrade request via /ws: {} (sid: {})", path, sid);
    ws.on_upgrade(move |socket| handle_ws_wrapper(socket, path, sid, state))
}

struct GatewayState {
    root: String,
    sm: Arc<SessionManager>,
    fpm_addr: Option<String>,
    tx_monitor: tokio::sync::broadcast::Sender<crate::MonitoringEvent>,
    tx_app_broadcast: tokio::sync::broadcast::Sender<Vec<u8>>,
}

async fn handle_http_root(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<GatewayState>>,
    ws: Option<WebSocketUpgrade>,
) -> Response {
    handle_http_inner("".to_string(), params, state, ws).await
}

async fn handle_http(
    AxPath(path): AxPath<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<GatewayState>>,
    ws: Option<WebSocketUpgrade>,
) -> Response {
    handle_http_inner(path, params, state, ws).await
}

async fn handle_http_inner(
    path: String,
    params: HashMap<String, String>,
    state: Arc<GatewayState>,
    ws: Option<WebSocketUpgrade>,
) -> Response {
    // 1. Si c'est une requête WebSocket, on upgrade
    if let Some(ws) = ws {
        let sid = params.get("sid").cloned().unwrap_or_else(|| "AUTO".to_string());
        let path_param = params.get("path").cloned().unwrap_or_else(|| path.clone());
        let state_clone = state.clone();
        info!("[WS] Upgrade request for path: {} (sid: {})", path_param, sid);
        return ws.on_upgrade(move |socket| handle_ws_wrapper(socket, path_param, sid, state_clone));
    }

    // 2. Résolution intelligente du chemin
    let mut full_path = format!("./{}", path);
    if !std::path::Path::new(&full_path).exists() {
        full_path = format!("{}/{}", state.root, path);
    }
    
    let path_obj = std::path::Path::new(&full_path);

    // Si c'est un dossier, on cherche index.nhtml
    let mut final_path = full_path.clone();
    if path_obj.is_dir() {
        let index = if final_path.ends_with('/') { "index.nhtml" } else { "/index.nhtml" };
        final_path.push_str(index);
    }
    
    let final_path_obj = std::path::Path::new(&final_path);
    info!("[HTTP] Request: {} -> Resolved: {}", path, final_path);

    if !final_path_obj.exists() {
        warn!("[HTTP] 404 - Not Found: {}", final_path);
        return (StatusCode::NOT_FOUND, format!("Fichier non trouvé: {}", path)).into_response();
    }

    let content = match std::fs::read(&final_path_obj) {
        Ok(c) => c,
        Err(_) => { return (StatusCode::INTERNAL_SERVER_ERROR, "Erreur de lecture").into_response(); }
    };

    let mime_str = if final_path.ends_with(".nhtml") {
        "text/html; charset=utf-8".to_string()
    } else {
        let mime = mime_guess::from_path(final_path_obj).first_or_octet_stream();
        let mut m = mime.to_string();
        if m.contains("text/") || m.contains("javascript") {
            if !m.contains("charset") {
                m.push_str("; charset=utf-8");
            }
        }
        m
    };

    if final_path.ends_with(".nhtml") {
        let mut html = if let Ok(source) = std::fs::read_to_string(&final_path_obj) {
            let result = crate::compiler::NhtmlCompiler::compile(&source);
            result.html
        } else {
            String::from_utf8_lossy(&content).to_string()
        };

        // INJECTION DU BRIDGE (Automatique)
        let bridge_script = r#"
    <script src="/assets/js/fzstd.min.js" defer></script>
    <script src="/assets/js/bridge.js" charset="UTF-8" defer></script>
"#;
        if let Some(pos) = html.find("</head>") {
            html.insert_str(pos, bridge_script);
        } else {
            html.push_str(bridge_script);
        }
        
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(axum::body::Body::from(html))
            .unwrap();
    }

    Response::builder()
        .header(header::CONTENT_TYPE, mime_str)
        .body(axum::body::Body::from(content))
        .unwrap()
}

async fn handle_ws_wrapper(socket: WebSocket, path: String, sid: String, state: Arc<GatewayState>) {
    // Conversion Axum WebSocket -> Tungstenite-like Stream si besoin, 
    // ou adaptation de la logique handle_connection existante.
    // Pour l'instant on va adapter handle_connection pour accepter le socket Axum.
    handle_connection_axum(socket, path, sid, state).await;
}

async fn handle_connection_axum(
    socket: WebSocket,
    requested_path: String,
    requested_sid: String,
    state: Arc<GatewayState>,
) {
    let root = &state.root;
    let sm = &state.sm;

    // Résolution du fichier .nhtml
    let mut nhtml_rel_path = if requested_path.is_empty() { "index.nhtml".to_string() } else { requested_path.clone() };
    if nhtml_rel_path.ends_with('/') {
        nhtml_rel_path.push_str("index.nhtml");
    }
    // Résolution intelligente du chemin .nhtml
    let mut nhtml_abs_path = format!("./{}", nhtml_rel_path);
    if !std::path::Path::new(&nhtml_abs_path).exists() {
        nhtml_abs_path = format!("{}/{}", root, nhtml_rel_path);
    }
    
    // Si c'est un dossier, on cherche index.nhtml
    if std::path::Path::new(&nhtml_abs_path).is_dir() {
        if !nhtml_abs_path.ends_with('/') { nhtml_abs_path.push('/'); }
        nhtml_abs_path.push_str("index.nhtml");
    }

    info!("[WS] Handshake Resolution: {} -> {}", nhtml_rel_path, nhtml_abs_path);

    let source = match std::fs::read_to_string(&nhtml_abs_path) {
        Ok(s) => s,
        Err(e) => {
            error!("Impossible de lire {} : {}", nhtml_abs_path, e);
            return;
        }
    };

    // Compilation à la volée
    let mut result = NhtmlCompiler::compile(&source);

    // ─── Restauration de session (SID) ───────────────────────────────────
    let session_id = if requested_sid == "AUTO" || requested_sid.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        requested_sid
    };

    let secret = match sm.register_session(session_id.clone(), requested_path.clone()).await {
        Ok(s) => s,
        Err(e) => {
            error!("[{}] Impossible d'enregistrer la session en DB : {}", session_id, e);
            return;
        }
    };

    // Charger les états depuis SQLite si existants
    let mut append_patches = Vec::new();
    let mut last_seq = 0;
    if let Ok(Some((_, seq))) = sm.get_session_security(session_id.clone()).await {
        last_seq = seq;
    }
    if let Ok(db_nodes) = sm.get_all_nodes(session_id.clone()).await {
        if !db_nodes.is_empty() {
            info!("[{}] Restauration de {} nœuds depuis SQLite", session_id, db_nodes.len());
            for (db_id, db_ver, db_tag, db_val, is_append) in db_nodes {
                if is_append {
                    // Si c'est un append, on ne l'inclut PAS dans le B-TREE
                    // On le prépare comme un patch séparé à envoyer après le B-TREE
                    append_patches.push(proto::PatchOp::append_html(db_id, db_ver, &db_val));
                } else {
                    // On cherche le nœud par son tag métier (NID) car les IDs binaires peuvent varier
                    if let Some(state_node) = result.states.iter_mut().find(|s| s.2 == db_tag) {
                        state_node.1 = db_ver;
                        state_node.3 = db_val;
                    }
                }
            }
            // Re-générer le B-TREE binaire avec les valeurs restaurées
            result.btree_bytes = proto::serialize_nodes(&result.states);
        }
    }
    
    // Résolution du script PHP (app.php dans le même dossier que le .nhtml)
    let php_script = if let Some(parent) = std::path::Path::new(&nhtml_abs_path).parent() {
        parent.join("app.php").to_string_lossy().to_string()
    } else {
        format!("{}/app.php", root)
    };

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let mut session = Session::new(session_id.clone(), &result, php_script, sm.clone(), state.fpm_addr.clone());

    // ── Séquence d'initialisation ──────────────────────────────────────────

    // 1. HELLO
    let hello = proto::hello(&session_id, &secret, last_seq);
    ws_sender.send(WsMessage::Binary(hello)).await.ok();

    // 2. B-TREE
    let (btree_pkt, comp_ratio) = proto::wrap_btree(&result.btree_bytes);
    ws_sender.send(WsMessage::Binary(btree_pkt.clone())).await.ok();
    info!("[{}] B-TREE envoyé ({} bytes, ratio={:.2})", session_id, result.btree_bytes.len(), comp_ratio);

    monitor_pkt(&state.tx_monitor, "OUT", proto::PKT_BTREE, btree_pkt.len(), &session_id, Some("BTREE".to_string()), Some("Full DOM Initial State".to_string()), None).await;

    // 2.5 Patches d'append restaurés
    if !append_patches.is_empty() {
        ws_sender.send(WsMessage::Binary(proto::patch(&append_patches))).await.ok();
        info!("[{}] {} patches d'append restaurés envoyés", session_id, append_patches.len());
    }

    // ─── Hydratation initiale (Appel PHP Init) ───────────────────────────
    let init_patches_res = call_php(
        &session.php_script,
        &session.handler_table,
        0, 0, "init", &[], 
        &session_id, 0,
        sm.clone(),
        &session.fpm_addr
    ).await;
    
    let init_patches = match init_patches_res {
        Ok(p) => p,
        Err(e) => {
            error!("[{}] PHP Init Error: {}", session_id, e);
            Vec::new()
        }
    };
    
    if !init_patches.is_empty() {
        let mut final_patches = Vec::new();
        for (mut op, _) in init_patches {
            let nid = session.handler_table.by_id.get(&(op.target_id))
                .and_then(|e| e.n_id.clone())
                .unwrap_or_else(|| "".to_string());
            
            if !nid.is_empty() {
                let val = match op.op_type {
                    0x01 | 0x0A | 0x0B | 0x04 | 0x05 => {
                        if op.data.len() >= 2 { String::from_utf8_lossy(&op.data[2..]).to_string() } else { "".to_string() }
                    }
                    0x02 | 0x09 => {
                        if op.data.len() > 0 {
                            let k_len = op.data[0] as usize;
                            if op.data.len() >= 1 + k_len + 2 {
                                String::from_utf8_lossy(&op.data[1 + k_len + 2..]).to_string()
                            } else { "".to_string() }
                        } else { "".to_string() }
                    }
                    _ => "".to_string()
                };
                let is_append = op.op_type == 0x0B;
                if let Ok(new_ver) = session.sm.update_node(session_id.clone(), op.target_id as u32, nid, val, is_append).await {
                    op.version = new_ver;
                }
            }
            final_patches.push(op);
        }
        info!("[{}] Envoi de {} patches d'hydratation initiale", session_id, final_patches.len());
        ws_sender.send(WsMessage::Binary(proto::patch(&final_patches))).await.ok();
    }

    // 3. BIND × N
    for bind_pkt in &result.bind_packets {
        ws_sender.send(WsMessage::Binary(bind_pkt.clone())).await.ok();
    }
    info!("[{}] {} paquets BIND envoyés", session_id, result.bind_packets.len());

    // ── Boucle de messages ─────────────────────────────────────────────────
    let mut app_rx = state.tx_app_broadcast.subscribe();
    loop {
            // ─── Écouteur de Broadcast Applicatif (Multi-utilisateur) ───────
            Ok(msg) = app_rx.recv() => {
                if msg.len() > 2 {
                    let scope_type = msg[0];
                    let sid_len = msg[1] as usize;
                    if msg.len() >= 2 + sid_len {
                        let sender_sid = String::from_utf8_lossy(&msg[2..2+sid_len]);
                        let pkt = &msg[2+sid_len..];
                        
                        let should_send = match scope_type {
                            0x01 => sender_sid != session_id, // others
                            0x02 => true, // all
                            _ => false
                        };
                        
                        if should_send {
                            let _ = ws_sender.send(WsMessage::Binary(pkt.to_vec())).await;
                        }
                    }
                }
            }
            msg_opt = ws_receiver.next() => {
                let msg = match msg_opt {
                    Some(m) => m,
                    None => {
                        info!("[{}] Connexion terminée", session_id);
                        break;
                    }
                };

                match msg {
                    Ok(WsMessage::Binary(data)) => {
                        if data.is_empty() { continue; }

                        let type_byte = data[0];

                        if type_byte == 0x02 { // EVENT
                            monitor_pkt(&state.tx_monitor, "IN", 0x02, data.len(), &session_id, None, Some(format!("Raw Event Data ({} bytes)", data.len())), None).await;
                            if let Some(patch_pkt) = handle_event(&data, &mut session, &state.tx_app_broadcast, &state.tx_monitor).await {
                                ws_sender.send(WsMessage::Binary(patch_pkt)).await.ok();
                            }
                        } else if type_byte == 0x01 { // HELLO (Client → Server)
                            info!("[{}] HELLO reçu du client", session_id);
                            monitor_pkt(&state.tx_monitor, "IN", 0x01, data.len(), &session_id, None, Some("Session Handshake".to_string()), None).await;
                        } 
                        
                        // ─── PKT_PUSH_PATCH (0x08) ───
                        // Client -> Server (Zero-Server / Multi-user Sync)
                        else if type_byte == proto::PKT_PUSH_PATCH {
                            // 1. Validation du format minimal
                            if data.len() < 12 { continue; }
                            
                            // 2. Décoder le nombre d'opérations
                            let op_count = u16::from_be_bytes([data[5], data[6]]) as usize;
                            let mut offset = 7;
                            let mut validated_ops = Vec::new();
                            
                            for _ in 0..op_count {
                                if data.len() < offset + 9 { break; }
                                let target_id = u16::from_be_bytes([data[offset], data[offset+1]]);
                                let op_type = data[offset + 2];
                                let version = u32::from_be_bytes([data[offset+3], data[offset+4], data[offset+5], data[offset+6]]);
                                let data_len = u16::from_be_bytes([data[offset+7], data[offset+8]]) as usize;
                                offset += 9;
                                
                                if data.len() < offset + data_len { break; }
                                let op_data = &data[offset .. offset + data_len];
                                offset += data_len;

                                // --- VALIDATION SÉCURITÉ ---
                                // A. Whitelist des Opcodes autorisés (Zéro-Server Multi)
                                let is_safe = match op_type {
                                    proto::OP_SET_TEXT  | // 0x01
                                    proto::OP_SET_ATTR  | // 0x02
                                    proto::OP_DEL_ATTR  | // 0x03
                                    proto::OP_ADD_CLASS | // 0x04
                                    proto::OP_DEL_CLASS | // 0x05
                                    proto::OP_SET_STYLE | // 0x09
                                    0x0C | // SCROLL_TO
                                    0x0D   // FOCUS
                                    => true,
                                    _ => {
                                        warn!("[{}] PUSH_PATCH rejeté : Opcode non autorisé ({:#02x})", session_id, op_type);
                                        false
                                    }
                                };

                                if is_safe {
                                    validated_ops.push(proto::PatchOp {
                                        op_type,
                                        target_id,
                                        version,
                                        data: op_data.to_vec(),
                                    });
                                }
                            }

                            if !validated_ops.is_empty() {
                                info!("[{}] PUSH_PATCH : {} opérations validées", session_id, validated_ops.len());
                                
                                // 3. Persister en SQLite
                                for op in &validated_ops {
                                    // On récupère le tag pour persister correctement
                                    let db_nodes = sm.get_all_nodes(session_id.clone()).await.unwrap_or_default();
                                    if let Some((_, _, tag, _, _)) = db_nodes.iter().find(|n| n.0 == op.target_id) {
                                        // On ne décode pas le payload ici, on le stocke tel quel (en mode append si nécessaire, mais ici on est sur du SET_TEXT par ex)
                                        let val_str = String::from_utf8_lossy(&op.data).to_string();
                                        let _ = sm.update_node(session_id.clone(), op.target_id as u32, tag.clone(), val_str, false).await;
                                    }
                                }

                                // 4. Broadcaster aux autres clients de la session
                                let broadcast_pkt = proto::patch(&validated_ops);
                                let _ = state.tx_app_broadcast.send(broadcast_pkt);
                            }
                        }

                        else if type_byte == proto::PKT_PING { // PING
                            // Répondre PONG (Type 0x09, Payload = Sequence)
                            let seq = data.get(5).copied().unwrap_or(0);
                            ws_sender.send(WsMessage::Binary(proto::ping(seq))).await.ok();
                            monitor_pkt(&state.tx_monitor, "IN", 0x09, data.len(), &session_id, None, Some("Keep-alive".to_string()), None).await;
                        }
                        else {
                            warn!("[{}] Paquet inattendu type=0x{:02X}", session_id, type_byte);
                        }
                    }
                    Ok(WsMessage::Close(_)) => {
                        info!("[{}] Connexion fermée proprement", session_id);
                        break;
                    }
                    Err(e) => {
                        error!("[{}] Erreur WebSocket: {}", session_id, e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn handle_event(
    data       : &[u8],
    session    : &mut Session,
    tx_app_broadcast: &tokio::sync::broadcast::Sender<Vec<u8>>,
    tx_monitor: &tokio::sync::broadcast::Sender<crate::MonitoringEvent>
) -> Option<Vec<u8>>
{
    // 1. Décodage v0.5.0 (Sécurité Industrielle)
    let decoded = crate::decoder::decode(data);
    let (seq_id, signature, node_id, handler_name, payload_str) = match decoded {
        crate::decoder::DecodedMessage::Event { seq_id, signature, node_id, handler, payload } => {
            (seq_id, signature, node_id, handler, payload)
        }
        _ => {
            warn!("[{}] Paquet EVENT invalide ou mal formé", session.state.session_id);
            return None;
        }
    };

    // 2. Vérification de la Séquence (Anti-Replay)
    if let Ok(Some((secret, last_seq))) = session.sm.get_session_security(session.state.session_id.clone()).await {
        if seq_id <= last_seq {
            warn!("[{}] REJET: Attaque par rejeu détectée (SeqID {} <= {})", session.state.session_id, seq_id, last_seq);
            return None;
        }

        // 3. Vérification HMAC (Authenticité)
        let mut sign_data = Vec::new();
        sign_data.push(0x02); // Type
        let total_len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
        sign_data.extend_from_slice(&total_len.to_be_bytes());
        sign_data.extend_from_slice(&seq_id.to_be_bytes());
        if data.len() >= 41 { sign_data.extend_from_slice(&data[41..]); }

        if !verify_hmac(&secret, &sign_data, &signature) {
            error!("[{}] REJET: Signature HMAC invalide !", session.state.session_id);
            return None;
        }
        let _ = session.sm.update_seq_id(session.state.session_id.clone(), seq_id).await;
    }

    let handler = if !handler_name.is_empty() {
        handler_name
    } else {
        session.handler_table.by_id.get(&(node_id as u16))
            .and_then(|e| e.handler.clone())
            .unwrap_or_else(|| "".to_string())
    };

    let payload = payload_str.as_bytes();
    info!("[{}] EVENT validé (Seq:{}) node={} handler='{}'", 
        session.state.session_id, seq_id, node_id, handler);

    monitor_pkt(tx_monitor, "IN", proto::PKT_EVENT, data.len(), &session.state.session_id, Some(handler.clone()), Some(format!("Payload: {} bytes", payload.len())), None).await;

    // --- Mise à jour AUTO de l'état local depuis le payload ---
    if let Ok(json_payload) = serde_json::from_slice::<serde_json::Value>(payload) {
        if let Some(obj) = json_payload.as_object() {
            for (nid, val) in obj {
                if let Some(val_str) = val.as_str() {
                    if let Some(target_id) = session.handler_table.nid_map.get(nid) {
                        session.sm.update_node(session.state.session_id.clone(), *target_id as u32, nid.clone(), val_str.to_string(), false).await.ok();
                    }
                }
            }
        }
    }

    // Appeler PHP
    let start = std::time::Instant::now();
    let res = call_php(
        &session.php_script,
        &session.handler_table,
        node_id as u16,
        0, &handler, payload,
        &session.state.session_id,
        session.state.last_version,
        session.sm.clone(),
        &session.fpm_addr
    ).await;

    let (patches, broadcast_instr) = match res {
        Ok(p) => p,
        Err(e) => {
            error!("[{}] PHP Error: {}", session.state.session_id, e);
            // Envoyer un message d'erreur au client
            let err_msg = format!("Internal Server Error: {}", e);
            let err_pkt = proto::log_msg(3, &err_msg); // 3 = ERROR level
            return Some(err_pkt);
        }
    };

    if !patches.is_empty() {
        let mut final_patches = Vec::new();
        let mut broadcast_patches = Vec::new();
        for (mut op, is_broadcast) in patches {
            let nid = session.handler_table.by_id.get(&(op.target_id)).and_then(|e| e.n_id.clone()).unwrap_or_default();
            if !nid.is_empty() {
                let val = match op.op_type {
                    0x01 | 0x0A | 0x0B => { if op.data.len() >= 2 { Some(String::from_utf8_lossy(&op.data[2..]).to_string()) } else { None } }
                    _ => None
                };
                if let Some(v) = val {
                    let is_append = op.op_type == 0x0B;
                    if let Ok(new_ver) = session.sm.update_node(session.state.session_id.clone(), op.target_id as u32, nid, v, is_append).await {
                        op.version = new_ver;
                    }
                }
            }
            if is_broadcast { broadcast_patches.push(op.clone()); } else { final_patches.push(op); }
        }

        if !final_patches.is_empty() {
            if let Some(max_ver) = final_patches.iter().map(|op| op.version).max() {
                if max_ver > 0 { session.state.last_version = max_ver; }
            }
            let elapsed = start.elapsed().as_millis() as u64;
            let patch_pkt = proto::patch(&final_patches);
            monitor_pkt(tx_monitor, "OUT", proto::PKT_PATCH, patch_pkt.len(), &session.state.session_id, Some(handler.clone()), Some(format!("Applied {} patches", final_patches.len())), Some(elapsed)).await;
            
            // --- Traitement du Broadcast v0.6.0 (SDK-driven) ---
            if let Some(bc) = broadcast_instr {
                info!("[{}] BROADCAST via PHP (scope: {})", session.state.session_id, bc.scope);
                let bc_pkt = proto::patch(&bc.patches);
                
                let mut msg = Vec::new();
                msg.push(if bc.scope == "all" { 0x02 } else { 0x01 });
                msg.push(session.state.session_id.len() as u8);
                msg.extend_from_slice(session.state.session_id.as_bytes());
                msg.extend_from_slice(&bc_pkt);
                
                let _ = tx_app_broadcast.send(msg);
            }

            return Some(patch_pkt);
        }
    }
    None
}

async fn monitor_pkt(
    tx: &tokio::sync::broadcast::Sender<crate::MonitoringEvent>,
    direction: &str,
    pkt_type: u8,
    size: usize,
    sid: &str,
    handler: Option<String>,
    details: Option<String>,
    latency: Option<u64>
) {
    use chrono::Local;
    let ev = crate::MonitoringEvent {
        direction: direction.to_string(),
        pkt_type,
        size,
        session_id: sid.to_string(),
        timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
        latency_ms: latency,
        compression_ratio: None,
        handler,
        details,
    };
    let _ = tx.send(ev);
}

// ─── Bridge PHP ─────────────────────────────────────────────────────────────

async fn call_php(
    php_script    : &str,
    handler_table : &HandlerTable,
    source_id     : u16,
    _event_type   : u8,
    handler       : &str,
    payload       : &[u8],
    session_id    : &str,
    last_version  : u32,
    sm            : Arc<SessionManager>,
    fpm_addr      : &Option<String>,
) -> crate::core::Result<(Vec<(proto::PatchOp, bool)>, Option<proto::BroadcastInstruction>)>
{
    use serde_json::json;

    // Récupérer tous les nœuds actuels pour cette session
    let db_nodes = sm.get_all_nodes(session_id.to_string()).await
        .map_err(|e| crate::core::GatewayError::DatabaseError(e.to_string()))?;
        
    let mut nodes_map = serde_json::Map::new();
    for (_id, ver, tag, val, is_append) in db_nodes {
        nodes_map.insert(tag.clone(), json!({
            "ver": ver,
            "val": val,
            "append": is_append
        }));
    }

    // Résoudre le n-id textuel pour le source_id numérique
    let n_id = handler_table.by_id.get(&source_id)
        .and_then(|e| e.n_id.clone())
        .unwrap_or_else(|| source_id.to_string());

    // Construire le contexte JSON
    let context = json!({
        "handler": handler,
        "source_id": n_id,
        "session_id": session_id,
        "payload": String::from_utf8_lossy(payload),
        "last_version": last_version,
        "handler_table": handler_table.to_json(),
        "nid_map": handler_table.nid_map,
        "nodes": nodes_map, 
    });

    if let Some(ref addr) = fpm_addr {
        call_php_fpm(addr, php_script, &context, handler_table).await
    } else {
        call_php_process(php_script, &context, handler_table).await
    }
}

async fn call_php_process(
    php_script: &str,
    context: &serde_json::Value,
    handler_table: &HandlerTable,
) -> crate::core::Result<(Vec<(proto::PatchOp, bool)>, Option<proto::BroadcastInstruction>)> {
    use std::process::Stdio;
    use tokio::process::Command;
    use tokio::io::AsyncWriteExt;

    let input = context.to_string();
    
    let mut child = Command::new("php")
        .arg("-f")
        .arg(php_script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| crate::core::GatewayError::PhpNotFound(e.to_string()))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes()).await;
    }

    let output = child.wait_with_output().await
        .map_err(|e| crate::core::GatewayError::PhpExecutionError(e.to_string()))?;

    if output.status.success() {
        Ok(parse_php_response(&output.stdout, handler_table))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(crate::core::GatewayError::PhpExecutionError(stderr.to_string()))
    }
}

async fn call_php_fpm(
    addr: &str,
    php_script: &str,
    context: &serde_json::Value,
    handler_table: &HandlerTable,
) -> crate::core::Result<(Vec<(proto::PatchOp, bool)>, Option<proto::BroadcastInstruction>)> {
    use fastcgi_client::{Client, Params};
    use std::io::{Cursor, Read};
    use std::net::TcpStream;

    let addr_own = addr.to_string();
    let script_own = php_script.to_string();
    let input = context.to_string();

    let result = tokio::task::spawn_blocking(move || {
        // Support simple TCP pour l'instant (127.0.0.1:9000)
        let stream = TcpStream::connect(&addr_own).map_err(|e| e.to_string())?;
        let mut client = Client::new(stream, false);

        let mut params = Params::default();
        params.insert("SCRIPT_FILENAME".into(), script_own.into());
        params.insert("REQUEST_METHOD".into(), "POST".into());
        params.insert("CONTENT_TYPE".into(), "application/json".into());
        params.insert("CONTENT_LENGTH".into(), input.len().to_string().into());

        let mut body = Cursor::new(input);
        let output = client.execute(params, &mut body).map_err(|e| e.to_string())?;

        let mut stdout = Vec::new();
        output.get_stdout().read_to_end(&mut stdout).map_err(|e| e.to_string())?;
        
        Ok::<Vec<u8>, String>(stdout)
    }).await.map_err(|e| crate::core::GatewayError::FastCgiError(e.to_string()))?;

    match result {
        Ok(stdout) => Ok(parse_php_response(&stdout, handler_table)),
        Err(e) => Err(crate::core::GatewayError::FastCgiError(e))
    }
}

// ─── Parser la réponse PHP ──────────────────────────────────────────────────

fn parse_php_response(
    stdout        : &[u8],
    handler_table : &HandlerTable,
) -> (Vec<(proto::PatchOp, bool)>, Option<proto::BroadcastInstruction>)
{
    // Nettoyage éventuel du stdout (PHP peut afficher des warnings avant le JSON)
    let json_str = std::str::from_utf8(stdout).unwrap_or("");
    // Cherche le premier [ ou { — supporte les deux formats de réponse PHP
    let pos_array  = json_str.find('[');
    let pos_object = json_str.find('{');
    let json_start = match (pos_array, pos_object) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None)    => a,
        (None, Some(b))    => b,
        (None, None)       => 0,
    };
    let json_clean = &json_str[json_start..];

    let json: serde_json::Value = match serde_json::from_str(json_clean) {
        Ok(v) => v,
        Err(e) => {
            error!("Réponse PHP invalide: {} — Content: {}", e, json_clean);
            return vec![];
        }
    };
    
    // Le SDK PHP renvoie soit un objet {"patch": [...]} soit directement le tableau [...]
    let ops_array = if json.is_array() {
        json.as_array()
    } else {
        json.get("patch").and_then(|v| v.as_array())
    };

    let mut broadcast = None;
    if !json.is_array() {
        if let Some(bc) = json.get("broadcast") {
            let scope = bc["scope"].as_str().unwrap_or("others").to_string();
            let mut bc_ops = Vec::new();
            if let Some(patches) = bc["patch"].as_array() {
                for op in patches {
                    if let Some(p) = parse_single_op(op, handler_table) {
                        bc_ops.push(p);
                    }
                }
            }
            if !bc_ops.is_empty() {
                broadcast = Some(proto::BroadcastInstruction { scope, patches: bc_ops });
            }
        }
    }

    let mut patch_ops = Vec::new();
    let Some(ops) = ops_array else { return vec![]; };

    for op in ops {
        let is_broadcast = op["broadcast"].as_bool().unwrap_or(false);
        if let Some(patch) = parse_single_op(op, handler_table) {
            patch_ops.push((patch, is_broadcast));
        }
    }

    (patch_ops, broadcast)
}

fn parse_single_op(op: &serde_json::Value, handler_table: &HandlerTable) -> Option<proto::PatchOp> {
    let op_type = op["op"].as_str().unwrap_or("");
    
    // Résoudre le n-id métier en node_id binaire
    let node_id = op["nid"].as_str()
        .and_then(|nid| handler_table.resolve_nid(nid))
        .or_else(|| op["node_id"].as_u64().map(|n| n as u16))
        .unwrap_or(0);

    if node_id == 0 {
        warn!("PatchOp sans node_id résolvable: {:?}", op);
        return None;
    }

    let patch = match op_type {
        "set_text" => {
            let val = op["val"].as_str().unwrap_or("");
            proto::PatchOp::set_text(node_id, 0, val)
        }
        "add_class" => {
            let val = op["val"].as_str().unwrap_or("");
            proto::PatchOp::add_class(node_id, 0, val)
        }
        "del_class" => {
            let val = op["val"].as_str().unwrap_or("");
            proto::PatchOp::del_class(node_id, 0, val)
        }
        "set_attr" => {
            let key = op["key"].as_str().unwrap_or("");
            let val = op["val"].as_str().unwrap_or("");
            proto::PatchOp::set_attr(node_id, 0, key, val)
        }
        "del_attr" => {
            let key = op["key"].as_str().unwrap_or("");
            proto::PatchOp::del_attr(node_id, 0, key)
        }
        "set_style" => {
            let prop = op["prop"].as_str().unwrap_or("");
            let val  = op["val"].as_str().unwrap_or("");
            proto::PatchOp::set_style(node_id, 0, prop, val)
        }
        "set_html" | "replace_inner" => {
            let val = op["val"].as_str().unwrap_or("");
            proto::PatchOp::replace_inner(node_id, 0, val)
        }
        "append_html" => {
            let val = op["val"].as_str().unwrap_or("");
            proto::PatchOp::append_html(node_id, 0, val)
        }
        "insert_before" => {
            let val = op["val"].as_str().unwrap_or("");
            proto::PatchOp::insert_before(node_id, 0, val)
        }
        "insert_after" => {
            let val = op["val"].as_str().unwrap_or("");
            proto::PatchOp::insert_after(node_id, 0, val)
        }
        "remove" => proto::PatchOp::remove(node_id, 0),
        "focus"  => proto::PatchOp::focus(node_id, 0),
        "scroll_to" => proto::PatchOp::scroll_to(node_id, 0),
        _ => {
            warn!("PatchOp inconnue: {}", op_type);
            return None;
        }
    };

    Some(patch)
}
}
