/// socket/mod.rs
/// Serveur WebSocket — gère les sessions, reçoit les EVENT,
/// dispatche vers PHP, renvoie les PATCH.

use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tracing::{info, warn, error};
use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::{
    extract::{Query, State, Path as AxPath},
    response::{Html, IntoResponse, Response},
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

// ─── État de session ────────────────────────────────────────────────────────

pub struct Session {
    pub state: SessionState,
    pub php_script: String,
    pub handler_table: HandlerTable,
    pub sm: Arc<SessionManager>,
}

impl Session {
    fn new(id: String, result: &CompileResult, php_script: String, sm: Arc<SessionManager>) -> Self {
        let handler_table = build_from_tree(&result.root);
        Self { 
            state: SessionState::new(id), 
            php_script, 
            handler_table,
            sm
        }
    }
}

// ─── Point d'entrée ─────────────────────────────────────────────────────────

pub async fn serve(port: u16, root: String, _entry: String, _php: String, sm: Arc<SessionManager>, tx_monitor: tokio::sync::broadcast::Sender<crate::MonitoringEvent>, tx_app_broadcast: tokio::sync::broadcast::Sender<Vec<u8>>) {
    let shared_state = Arc::new(GatewayState {
        root: root.clone(),
        sm: sm.clone(),
        php_script_base: _php,
        tx_monitor,
        tx_app_broadcast,
    });

    let app = Router::new()
        .route("/ws", get(handle_ws_route))
        .route("/", get(handle_http))
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
    php_script_base: String,
    tx_monitor: tokio::sync::broadcast::Sender<crate::MonitoringEvent>,
    tx_app_broadcast: tokio::sync::broadcast::Sender<Vec<u8>>,
}

async fn handle_http(
    AxPath(path): AxPath<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<GatewayState>>,
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

    let mime = mime_guess::from_path(final_path_obj).first_or_octet_stream();
    let mut mime_str = mime.to_string();
    if mime_str.contains("text/") || mime_str.contains("javascript") {
        if !mime_str.contains("charset") {
            mime_str.push_str("; charset=utf-8");
        }
    }

    if path.ends_with(".nhtml") {
        let mut html = String::from_utf8_lossy(&content).to_string();
        // INJECTION DU BRIDGE (Automatique)
        let bridge_script = r#"<script src="/assets/js/bridge.js" charset="UTF-8"></script>"#;
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
    mut socket: WebSocket,
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

    if let Err(e) = sm.register_session(session_id.clone(), requested_path.clone()).await {
        warn!("[{}] Impossible d'enregistrer la session en DB : {}", session_id, e);
    }

    // Charger les états depuis SQLite si existants
    if let Ok(db_nodes) = sm.get_all_nodes(session_id.clone()).await {
        if !db_nodes.is_empty() {
            info!("[{}] Restauration de {} nœuds depuis SQLite", session_id, db_nodes.len());
            for (db_id, db_ver, db_tag, db_val) in db_nodes {
                // On cherche le nœud par son tag métier (NID) car les IDs binaires peuvent varier
                if let Some(state_node) = result.states.iter_mut().find(|s| s.2 == db_tag) {
                    state_node.1 = db_ver;
                    state_node.3 = db_val;
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
    let session = Session::new(session_id.clone(), &result, php_script, sm.clone());

    // ── Séquence d'initialisation ──────────────────────────────────────────

    // 1. HELLO
    let hello = proto::hello(&session_id, 5000);
    ws_sender.send(WsMessage::Binary(hello)).await.ok();

    // 2. B-TREE
    let btree_pkt = proto::wrap_btree(&result.btree_bytes);
    ws_sender.send(WsMessage::Binary(btree_pkt)).await.ok();
    info!("[{}] B-TREE envoyé ({} bytes)", session_id, result.btree_bytes.len());

    // ─── Hydratation initiale (Appel PHP Init) ───────────────────────────
    let init_patches = call_php(
        &session.php_script,
        &session.handler_table,
        0, 0, "init", &[], 
        &session_id, 0,
        sm.clone()
    ).await;
    
    if !init_patches.is_empty() {
        let mut final_patches = Vec::new();
        for mut op in init_patches {
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
                if let Ok(new_ver) = session.sm.update_node(session_id.clone(), op.target_id as u32, nid, val).await {
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

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(WsMessage::Binary(data)) => {
                if data.is_empty() { continue; }

                match data[0] {
                    0x02 => { // EVENT
                        handle_event(&data, &session, &mut ws_sender).await;
                    }
                    0x01 => { // HELLO (Client → Server)
                        info!("[{}] HELLO reçu du client", session_id);
                    }
                    0x09 => { // PING
                        // Répondre PONG (Type 0x09, Payload = Sequence)
                        let seq = data.get(5).copied().unwrap_or(0);
                        ws_sender.send(WsMessage::Binary(proto::ping(seq))).await.ok();
                    }
                    t => {
                        warn!("[{}] Paquet inattendu type=0x{:02X}", session_id, t);
                    }
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

// ─── Dispatch EVENT → PHP ───────────────────────────────────────────────────

async fn handle_event(
    data       : &[u8],
    session    : &Session,
    ws_sender  : &mut (impl SinkExt<WsMessage, Error = axum::Error> + Unpin),
)
{
    // Parser le paquet EVENT envoyé par bridge.js (v0.4.0)
    // Format: [0x02][4B node_id][1B handler_len][handler_bytes][2B payload_len][payload]
    if data.len() < 8 { return; }

    let node_id = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
    let handler_len = data[5] as usize;
    
    if data.len() < 8 + handler_len { return; }
    let handler_bytes = &data[6..6 + handler_len];
    let handler_from_pkt = String::from_utf8_lossy(handler_bytes).to_string();

    let payload_offset = 6 + handler_len;
    let payload_len = u16::from_be_bytes([data[payload_offset], data[payload_offset+1]]) as usize;
    
    let payload = if data.len() >= payload_offset + 2 + payload_len {
        &data[payload_offset + 2 .. payload_offset + 2 + payload_len]
    } else {
        &data[payload_offset + 2 ..]
    };

    // Si le paquet contient un handler, on l'utilise, sinon on cherche dans la table
    let handler = if !handler_from_pkt.is_empty() {
        handler_from_pkt
    } else {
        session.handler_table.by_id.get(&(node_id as u16))
            .and_then(|e| e.handler.clone())
            .unwrap_or_else(|| "".to_string())
    };

    info!("[{}] EVENT node={} handler='{}' payload_len={}", 
        session.state.session_id, node_id, handler, payload_len);

    // --- Mise à jour AUTO de l'état local depuis le payload (Industrial Sync) ---
    if let Ok(json_payload) = serde_json::from_slice::<serde_json::Value>(payload) {
        if let Some(obj) = json_payload.as_object() {
            for (nid, val) in obj {
                if let Some(val_str) = val.as_str() {
                    if let Some(target_id) = session.handler_table.nid_map.get(nid) {
                        session.sm.update_node(session.state.session_id.clone(), *target_id as u32, nid.clone(), val_str.to_string()).await.ok();
                    }
                }
            }
        }
    }

    // Appeler PHP avec le contexte complet
    let patches = call_php(
        &session.php_script,
        &session.handler_table,
        node_id as u16,
        0, // event_type
        &handler,
        payload,
        &session.state.session_id,
        session.state.last_version,
        session.sm.clone()
    ).await;

    // Envoyer les PATCH résultants
    if !patches.is_empty() {
        let mut final_patches = Vec::new();
        // Persister les changements en DB et mettre à jour les versions
        for mut op in patches {
            let nid = session.handler_table.by_id.get(&(op.target_id))
                .and_then(|e| e.n_id.clone())
                .unwrap_or_else(|| "".to_string());
            
            if !nid.is_empty() {
                // On ne persiste que le CONTENU du nœud (SET_TEXT, REPLACE_INNER, APPEND_HTML)
                // Persister les styles ou attributs ici corromprait la valeur principale du nœud.
                let val = match op.op_type {
                    0x01 | 0x0A | 0x0B => {
                        if op.data.len() >= 2 { Some(String::from_utf8_lossy(&op.data[2..]).to_string()) } else { None }
                    }
                    _ => None
                };
                
                if let Some(v) = val {
                    if let Ok(new_ver) = session.sm.update_node(session.state.session_id.clone(), op.target_id as u32, nid, v).await {
                        op.version = new_ver;
                    }
                }
            }
            final_patches.push(op);
        }

        let patch_pkt = proto::patch(&final_patches);
        ws_sender.send(WsMessage::Binary(patch_pkt)).await.ok();
    }
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
) -> Vec<proto::PatchOp>
{
    use std::process::Stdio;
    use tokio::process::Command;
    use tokio::io::AsyncWriteExt;
    use serde_json::json;

    // Récupérer tous les nœuds actuels pour cette session
    let db_nodes = sm.get_all_nodes(session_id.to_string()).await.unwrap_or_default();
    let mut nodes_map = serde_json::Map::new();
    for (_id, ver, tag, val) in db_nodes {
        nodes_map.insert(tag.clone(), json!({
            "ver": ver,
            "val": val
        }));
    }

    // Construire le contexte JSON transmis au PHP via stdin
    let context = json!({
        "handler": handler,
        "source_id": source_id,
        "session_id": session_id,
        "payload": String::from_utf8_lossy(payload),
        "last_version": last_version,
        "handler_table": handler_table.to_json(),
        "nid_map": handler_table.nid_map,
        "nodes": nodes_map, // AJOUTÉ : État complet des nœuds
    });

    let input = context.to_string();

    info!("[{}] Appel PHP: {} | Handler: {} | NodeID: {}", 
        session_id, php_script, handler, source_id);

    // php -f script.php — lit le contexte sur stdin, retourne JSON sur stdout
    let mut child = Command::new("php")
        .arg("-f")
        .arg(php_script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            error!("Impossible de lancer PHP: {}", e);
            e
        })
        .expect("Échec du lancement de PHP");

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes()).await;
    }

    let output = child.wait_with_output().await;

    match output {
        Ok(out) if out.status.success() => {
            parse_php_response(&out.stdout, handler_table)
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            error!("[NHTML] PHP erreur (exit {}): {}", out.status.code().unwrap_or(-1), stderr);
            vec![]
        }
        Err(e) => {
            error!("Erreur exécution PHP: {}", e);
            vec![]
        }
    }
}

// ─── Parser la réponse PHP ──────────────────────────────────────────────────

fn parse_php_response(
    stdout        : &[u8],
    handler_table : &HandlerTable,
) -> Vec<proto::PatchOp>
{
    // Nettoyage éventuel du stdout (si PHP a affiché des warnings avant le JSON)
    let json_str = std::str::from_utf8(stdout).unwrap_or("");
    let json_start = json_str.find('[').unwrap_or(0);
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

    let mut patch_ops = Vec::new();
    let Some(ops) = ops_array else { return vec![]; };

    for op in ops {
        let op_type = op["op"].as_str().unwrap_or("");

        // Résoudre le n-id métier en node_id binaire
        let node_id = op["nid"].as_str()
            .and_then(|nid| handler_table.resolve_nid(nid))
            .or_else(|| op["node_id"].as_u64().map(|n| n as u16))
            .unwrap_or(0);

        if node_id == 0 {
            warn!("PatchOp sans node_id résolvable: {:?}", op);
            continue;
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
            "replace_inner" => {
                let val = op["val"].as_str().unwrap_or("");
                proto::PatchOp::replace_inner(node_id, 0, val)
            }
            "append_html" => {
                let val = op["val"].as_str().unwrap_or("");
                proto::PatchOp::append_html(node_id, 0, val)
            }
            "remove" => proto::PatchOp::remove(node_id, 0),
            "focus"  => proto::PatchOp::focus(node_id, 0),
            _ => {
                warn!("PatchOp inconnue: {}", op_type);
                continue;
            }
        };

        patch_ops.push(patch);
    }

    patch_ops
}
