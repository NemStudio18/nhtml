/// socket/mod.rs
/// Serveur WebSocket — gère les sessions, reçoit les EVENT,
/// dispatche vers PHP, renvoie les PATCH.

use std::sync::Arc;
use tokio::net::TcpStream;
#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(unix)]
use tokio_util::either::Either;
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
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::compiler::{NhtmlCompiler, CompileResult};
use crate::compiler::handler_table::{HandlerTable, build_from_tree};
use crate::proto;
use crate::core::SessionState;
use crate::session::SessionManager;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::sync::Mutex;
use fastcgi_client::conn::KeepAlive;

type HmacSha256 = Hmac<Sha256>;
#[cfg(unix)]
type FpmStream = Either<TcpStream, UnixStream>;
#[cfg(not(unix))]
type FpmStream = TcpStream;

type FpmClient = fastcgi_client::Client<fastcgi_client::io::TokioCompat<FpmStream>, KeepAlive>;

/// Pool de connexions FastCGI pour la réutilisation des sockets.
pub struct FpmPool {
    addr: String,
    clients: Mutex<Vec<FpmClient>>,
    max_size: usize,
    current_size: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl FpmPool {
    pub fn new(addr: String) -> Self {
        Self {
            addr,
            clients: Mutex::new(Vec::new()),
            max_size: 100, // Limite par défaut à 100 connexions
            current_size: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    pub async fn acquire(&self) -> crate::core::Result<FpmClient> {
        {
            let mut clients = self.clients.lock().await;
            if let Some(client) = clients.pop() {
                return Ok(client);
            }
        }
        
        // Vérifier si on peut encore créer une connexion
        let curr = self.current_size.load(std::sync::atomic::Ordering::SeqCst);
        if curr >= self.max_size {
            return Err(crate::core::GatewayError::SocketError(format!("FpmPool saturé ({} connexions)", self.max_size)));
        }
        
        self.current_size.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        let is_unix = self.addr.starts_with('/') || self.addr.starts_with("./") || self.addr.contains(".sock") || self.addr.starts_with("unix:");

        #[cfg(unix)]
        {
            if is_unix {
                let clean_addr = self.addr.strip_prefix("unix:").unwrap_or(&self.addr);
                info!("[FPM] Connexion via Unix Socket: {}", clean_addr);
                let stream = UnixStream::connect(clean_addr).await
                    .map_err(|e| crate::core::GatewayError::SocketError(format!("Connexion FPM Unix échouée sur {} : {}", clean_addr, e)))?;
                return Ok(fastcgi_client::Client::new_keep_alive_tokio(Either::Right(stream)));
            }
        }

        #[cfg(not(unix))]
        if is_unix {
            return Err(crate::core::GatewayError::SocketError("Les sockets Unix ne sont pas supportés sur cette plateforme.".to_string()));
        }

        // Mode TCP (Commun à tous ou fallback)
        let stream = TcpStream::connect(&self.addr).await
            .map_err(|e| crate::core::GatewayError::SocketError(format!("Connexion FPM TCP échouée sur {} : {}", self.addr, e)))?;
        
        #[cfg(unix)]
        return Ok(fastcgi_client::Client::new_keep_alive_tokio(Either::Left(stream)));
        
        #[cfg(not(unix))]
        return Ok(fastcgi_client::Client::new_keep_alive_tokio(stream));
    }

    pub async fn release(&self, client: FpmClient) {
        let mut clients = self.clients.lock().await;
        clients.push(client);
    }
}

pub fn verify_hmac(secret: &[u8], data: &[u8], signature: &[u8]) -> bool {
    if let Ok(mut mac) = HmacSha256::new_from_slice(secret) {
        mac.update(data);
        mac.verify_slice(signature).is_ok()
    } else {
        error!("HMAC: Erreur lors de la création du contexte (clé invalide?)");
        false
    }
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
    gateway_id: String,
    port: u16, 
    root: String, 
    _entry: String, 
    _php: String, 
    fpm_addr: Option<String>,
    fpm_timeout: u64,
    sm: Arc<SessionManager>, 
    tx_monitor: tokio::sync::broadcast::Sender<crate::MonitoringEvent>, 
    tx_app_broadcast: tokio::sync::broadcast::Sender<Arc<Vec<u8>>>,
    tx_reload: tokio::sync::broadcast::Sender<()>,
    security: Option<crate::config::SecurityConfig>,
) {
    let fpm_pool = fpm_addr.as_ref().map(|addr| Arc::new(FpmPool::new(addr.clone())));
    
    let rate_limiter = security.as_ref()
        .and_then(|s| s.rate_limit.as_ref())
        .and_then(|r| r.events_per_sec)
        .map(|limit| Arc::new(RateLimiter::new(limit)));

    let shared_state = Arc::new(GatewayState {
        gateway_id,
        root: root.clone(),
        sm: sm.clone(),
        fpm_addr,
        fpm_pool,
        fpm_timeout,
        tx_monitor,
        tx_app_broadcast,
        tx_reload,
        rate_limiter,
    });

    let app = Router::new()
        .route("/ws", get(handle_ws_route))
        .route("/", get(handle_http_root))
        .route("/*path", get(handle_http))
        .with_state(shared_state.clone());

    let addr_res = format!("0.0.0.0:{}", port).parse::<std::net::SocketAddr>();
    let addr = match addr_res {
        Ok(a) => a,
        Err(_) => {
            error!("❌ Erreur: Port invalide '{}'", port);
            return;
        }
    };
    
    // Vérification TLS
    if let Some(sec) = security {
        if let Some(tls) = sec.tls {
            if tls.enabled {
                info!("🚀 Gateway Sécurisée (HTTPS/WSS) en écoute sur https://{}", addr);
                let config_res = axum_server::tls_rustls::RustlsConfig::from_pem_file(
                    &tls.cert,
                    &tls.key
                ).await;

                let config = match config_res {
                    Ok(c) => c,
                    Err(e) => {
                        error!("❌ Erreur TLS: Impossible de charger les certificats ({} / {}) : {}", tls.cert, tls.key, e);
                        error!("⚠️ Fallback: Démarrage en mode HTTP standard...");
                        axum_server::bind(addr).serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.ok();
                        return;
                    }
                };

                if let Err(e) = axum_server::bind_rustls(addr, config)
                    .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
                    .await {
                    error!("❌ Gateway TLS Error: {}", e);
                }
                return;
            }
        }
    }

    info!("Gateway Industrielle en écoute sur http://{}", addr);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("❌ Gateway: Impossible de lier le port {} : {}", port, e);
            return;
        }
    };
    if let Err(e) = axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await {
        error!("❌ Gateway: Erreur lors de l'exécution du serveur Axum : {}", e);
    }
}

async fn handle_ws_route(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<GatewayState>>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    ws: WebSocketUpgrade,
) -> Response {
    let sid = params.get("sid").cloned().unwrap_or_else(|| "AUTO".to_string());
    let path = params.get("path").cloned().unwrap_or_else(|| "".to_string());
    
    let ip = addr.ip().to_string();
    
    info!("[WS] Upgrade request from {} for path: {} (sid: {})", ip, path, sid);
    metrics::gauge!("nhtml_active_clients").increment(1.0);
    ws.on_upgrade(move |socket| handle_ws_wrapper(socket, path, sid, ip, state))
}

struct GatewayState {
    pub gateway_id: String,
    root: String,
    sm: Arc<SessionManager>,
    fpm_addr: Option<String>,
    fpm_pool: Option<Arc<FpmPool>>,
    fpm_timeout: u64,
    tx_monitor: tokio::sync::broadcast::Sender<crate::MonitoringEvent>,
    tx_app_broadcast: tokio::sync::broadcast::Sender<Arc<Vec<u8>>>,
    tx_reload: tokio::sync::broadcast::Sender<()>,
    rate_limiter: Option<Arc<RateLimiter>>,
}

/// Simple Rate Limiter Token Bucket
pub struct RateLimiter {
    pub limit: u32,
    pub ips: Mutex<HashMap<String, (Instant, u32)>>,
}

impl RateLimiter {
    pub fn new(limit: u32) -> Self {
        Self {
            limit,
            ips: Mutex::new(HashMap::new()),
        }
    }

    pub async fn check(&self, ip: String) -> bool {
        let mut ips = self.ips.lock().await;
        let now = std::time::Instant::now();
        
        // Anti-leak: Nettoyage agressif si la table devient trop large
        if ips.len() > 1000 {
            ips.retain(|_, (last, _)| now.duration_since(*last).as_secs() < 60);
        }

        let entry = ips.entry(ip).or_insert((now, 0));
        
        if now.duration_since(entry.0).as_secs() >= 1 {
            entry.0 = now;
            entry.1 = 1;
            return true;
        }
        
        if entry.1 < self.limit {
            entry.1 = entry.1.saturating_add(1);
            return true;
        }
        
        false
    }
}

async fn handle_http_root(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<GatewayState>>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    ws: Option<WebSocketUpgrade>,
) -> Response {
    handle_http_inner("".to_string(), params, state, addr.ip().to_string(), ws).await
}

async fn handle_http(
    AxPath(path): AxPath<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<GatewayState>>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    ws: Option<WebSocketUpgrade>,
) -> Response {
    handle_http_inner(path, params, state, addr.ip().to_string(), ws).await
}

async fn handle_http_inner(
    path: String,
    params: HashMap<String, String>,
    state: Arc<GatewayState>,
    ip: String,
    ws: Option<WebSocketUpgrade>,
) -> Response {
    // 1. Si c'est une requête WebSocket, on upgrade
    if let Some(ws) = ws {
        let sid = params.get("sid").cloned().unwrap_or_else(|| "AUTO".to_string());
        let path_param = params.get("path").cloned().unwrap_or_else(|| path.clone());
        let state_clone = state.clone();
        info!("[WS] Upgrade request from {} for path: {} (sid: {})", ip, path_param, sid);
        return ws.on_upgrade(move |socket| handle_ws_wrapper(socket, path_param, sid, ip, state_clone));
    }

    // 2. Résolution sécurisée du chemin (Anti-Path Traversal)
    let safe_path = path.replace("..", "");
    let mut full_path = std::path::PathBuf::from(&state.root);
    full_path.push(safe_path.trim_start_matches('/'));

    let path_obj = full_path.as_path();

    // Si c'est un dossier, on cherche index.nhtml
    let mut final_path = full_path.clone();
    if path_obj.is_dir() {
        let index = if final_path.to_string_lossy().ends_with('/') { "index.nhtml" } else { "index.nhtml" }; // PathBuf::push handles separators
        final_path.push(index);
    }
    
    let final_path_obj = final_path.as_path();
    info!("[HTTP] Request: {} -> Resolved: {}", path, final_path.display());

    if !final_path_obj.exists() {
        warn!("[HTTP] 404 - Not Found: {}", final_path.display());
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
        
        match Response::builder()
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(axum::body::Body::from(html)) {
                Ok(r) => return r,
                Err(e) => {
                    error!("[HTTP] Erreur de construction de réponse : {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Erreur interne").into_response();
                }
            }
    }

    match Response::builder()
        .header(header::CONTENT_TYPE, mime_str)
        .body(axum::body::Body::from(content)) {
            Ok(r) => r,
            Err(e) => {
                error!("[HTTP] Erreur de construction de réponse : {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Erreur interne").into_response()
            }
        }
}

async fn handle_ws_wrapper(socket: WebSocket, path: String, sid: String, ip: String, state: Arc<GatewayState>) {
    handle_connection_axum(socket, path, sid, ip, state).await;
}

async fn handle_connection_axum(
    socket: WebSocket,
    requested_path: String,
    requested_sid: String,
    ip: String,
    state: Arc<GatewayState>,
) {
    let root = &state.root;
    let sm = &state.sm;

    // Résolution du fichier .nhtml
    let mut nhtml_rel_path = if requested_path.is_empty() { "index.nhtml".to_string() } else { requested_path.clone() };
    if nhtml_rel_path.ends_with('/') {
        nhtml_rel_path.push_str("index.nhtml");
    }
    // Résolution intelligente du chemin .nhtml (Priorité au dossier projet --path)
    let mut nhtml_abs_path = format!("{}/{}", root, nhtml_rel_path);
    if !std::path::Path::new(&nhtml_abs_path).exists() {
        nhtml_abs_path = format!("./{}", nhtml_rel_path);
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
        let p = parent.join("app.php");
        std::fs::canonicalize(&p).unwrap_or(p).to_string_lossy().to_string()
    } else {
        let p = std::path::Path::new(root).join("app.php");
        std::fs::canonicalize(&p).unwrap_or(p).to_string_lossy().to_string()
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
    let mut session_rooms: HashSet<String> = sm.get_session_rooms(session_id.clone()).await
        .unwrap_or_default()
        .into_iter()
        .collect();

    let init_res = call_php(
        &session.php_script,
        &session.handler_table,
        0, 0, "init", &[], 
        &session_id, 0,
        sm.clone(),
        state.clone()
    ).await;
    
    let (init_patches, _, init_join, init_leave) = match init_res {
        Ok(p) => p,
        Err(e) => {
            error!("[{}] PHP Init Error: {}", session_id, e);
            (Vec::new(), None, Vec::new(), Vec::new())
        }
    };

    // Traiter les salons initiaux
    for r in init_join {
        let _ = sm.join_room(session_id.clone(), r.clone()).await;
        session_rooms.insert(r);
    }
    for r in init_leave {
        let _ = sm.leave_room(session_id.clone(), r.clone()).await;
        session_rooms.remove(&r);
    }
    
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
    let mut reload_rx = state.tx_reload.subscribe();
    loop {
        tokio::select! {
            // ─── Hot Reload ───
            Ok(_) = reload_rx.recv() => {
                info!("[{}] Envoi du signal de Hot Reload...", session_id);
                let reload_pkt = crate::proto::log_msg(0x11, "RELOAD");
                let _ = ws_sender.send(WsMessage::Binary(reload_pkt)).await;
            }
            // ─── Écouteur de Broadcast Applicatif (Multi-utilisateur) ───────
            Ok(msg_arc) = app_rx.recv() => {
                let msg = &*msg_arc;
                if msg.len() > 3 {
                    let scope_type = msg[0];
                    let gid_len = msg[1] as usize;
                    if msg.len() >= 3 + gid_len {
                        let _sender_gid = String::from_utf8(msg[2..2+gid_len].to_vec()).unwrap_or_else(|_| "unknown".to_string());
                        let sid_len = msg[2+gid_len] as usize;
                        if msg.len() >= 3 + gid_len + sid_len {
                            let sender_sid = String::from_utf8(msg[3+gid_len..3+gid_len+sid_len].to_vec()).unwrap_or_else(|_| "unknown".to_string());
                            let pkt_data = &msg[3+gid_len+sid_len..];
                            
                            let (should_send, final_pkt) = match scope_type {
                                proto::SCOPE_OTHERS => (sender_sid != session_id, pkt_data.to_vec()),
                                proto::SCOPE_ALL => (true, pkt_data.to_vec()),
                                proto::SCOPE_ROOM => {
                                    if pkt_data.len() > 0 {
                                        let rid_len = pkt_data[0] as usize;
                                        if pkt_data.len() >= 1 + rid_len {
                                            let rid = String::from_utf8(pkt_data[1..1+rid_len].to_vec()).unwrap_or_else(|_| "global".to_string());
                                            let payload = &pkt_data[1+rid_len..];
                                            (session_rooms.contains(&rid), payload.to_vec())
                                        } else { (false, Vec::new()) }
                                    } else { (false, Vec::new()) }
                                },
                                proto::SCOPE_DIRECT => {
                                    if pkt_data.len() > 0 {
                                        let tsid_len = pkt_data[0] as usize;
                                        if pkt_data.len() >= 1 + tsid_len {
                                            let tsid = String::from_utf8(pkt_data[1..1+tsid_len].to_vec()).unwrap_or_else(|_| "unknown".to_string());
                                            let payload = &pkt_data[1+tsid_len..];
                                            (tsid == session_id, payload.to_vec())
                                        } else { (false, Vec::new()) }
                                    } else { (false, Vec::new()) }
                                },
                                _ => (false, Vec::new())
                            };
                            
                            if should_send && !final_pkt.is_empty() {
                                let _ = ws_sender.send(WsMessage::Binary(final_pkt)).await;
                            }
                        }
                    }
                }
            }
            msg_opt = ws_receiver.next() => {
                let msg = match msg_opt {
                    Some(m) => m,
                    None => {
                        info!("[{}] Connexion terminée", session_id);
                        metrics::gauge!("nhtml_active_clients").decrement(1.0);
                        break;
                    }
                };

                match msg {
                    Ok(WsMessage::Binary(data)) => {
                        if data.is_empty() { continue; }

                        let type_byte = data[0];
                        metrics::counter!("nhtml_packets_received_total", "type" => type_byte.to_string()).increment(1);

                        if type_byte == 0x02 { // EVENT
                            // RATE LIMIT CHECK
                            if let Some(rl) = &state.rate_limiter {
                                if !rl.check(ip.clone()).await {
                                    warn!("[{}] Rate Limit dépassé pour IP: {}", session_id, ip);
                                    continue;
                                }
                            }

                            monitor_pkt(&state.tx_monitor, "IN", 0x02, data.len(), &session_id, None, Some(format!("Raw Event Data ({} bytes)", data.len())), None).await;
                            let ev_res = handle_event(&data, &mut session, state.clone()).await;
                            
                            // Mise à jour locale des salons
                            for r in ev_res.join_rooms { session_rooms.insert(r); }
                            for r in ev_res.leave_rooms { session_rooms.remove(&r); }
                            
                            if let Some(patch_pkt) = ev_res.patch_pkt {
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
                                
                                let mut msg = Vec::new();
                                msg.push(proto::SCOPE_OTHERS);
                                msg.push(state.gateway_id.len() as u8);
                                msg.extend_from_slice(state.gateway_id.as_bytes());
                                msg.push(session_id.len() as u8);
                                msg.extend_from_slice(session_id.as_bytes());
                                msg.extend_from_slice(&broadcast_pkt);
                                
                                let _ = state.tx_app_broadcast.send(std::sync::Arc::new(msg));
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

struct EventResult {
    patch_pkt: Option<Vec<u8>>,
    join_rooms: Vec<String>,
    leave_rooms: Vec<String>,
}

async fn handle_event(
    data       : &[u8],
    session    : &mut Session,
    state      : Arc<GatewayState>
) -> EventResult
{
    let mut result = EventResult { patch_pkt: None, join_rooms: Vec::new(), leave_rooms: Vec::new() };

    // 1. Décodage v0.5.0 (Sécurité Industrielle)
    let decoded = crate::decoder::decode(data);
    let (seq_id, signature, node_id, handler_name, payload_str) = match decoded {
        crate::decoder::DecodedMessage::Event { seq_id, signature, node_id, handler, payload } => {
            (seq_id, signature, node_id, handler, payload)
        }
        _ => {
            warn!("[{}] Paquet EVENT invalide ou mal formé", session.state.session_id);
            return result;
        }
    };

    // 2. Vérification de la Séquence (Anti-Replay)
    if let Ok(Some((secret, last_seq))) = session.sm.get_session_security(session.state.session_id.clone()).await {
        if seq_id <= last_seq {
            warn!("[{}] REJET: Attaque par rejeu détectée (SeqID {} <= {})", session.state.session_id, seq_id, last_seq);
            return result;
        }

        // 3. Vérification HMAC (Authenticité)
        // Le paquet doit faire au moins 41 octets (Type:1 + Len:4 + Seq:4 + Sig:32)
        if data.len() < 41 {
            error!("[{}] REJET: Paquet EVENT trop court ({})", session.state.session_id, data.len());
            return result;
        }

        let mut sign_data = Vec::new();
        sign_data.push(0x02); // Type
        let total_len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
        sign_data.extend_from_slice(&total_len.to_be_bytes());
        sign_data.extend_from_slice(&seq_id.to_be_bytes());
        
        // On ajoute tout le payload APRES la signature (offset 41)
        sign_data.extend_from_slice(&data[41..]);

        if !verify_hmac(&secret, &sign_data, &signature) {
            error!("[{}] REJET: Signature HMAC invalide ! Payload détourné ?", session.state.session_id);
            return result;
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

    monitor_pkt(&state.tx_monitor, "IN", proto::PKT_EVENT, data.len(), &session.state.session_id, Some(handler.clone()), Some(format!("Payload: {} bytes", payload.len())), None).await;

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
    let php_res = call_php(
        &session.php_script,
        &session.handler_table,
        node_id as u16,
        0, &handler, payload,
        &session.state.session_id,
        session.state.last_version,
        session.sm.clone(),
        state.clone()
    ).await;

    let (patches, broadcast_instr, join_rooms, leave_rooms) = match php_res {
        Ok(p) => p,
        Err(e) => {
            error!("[{}] PHP Error: {}", session.state.session_id, e);
            let err_msg = format!("PHP Backend Error: {}", e);
            
            // Log vers le Dashboard DevTools
            monitor_pkt(&state.tx_monitor, "ERR", proto::PKT_LOG, err_msg.len(), &session.state.session_id, Some(handler.clone()), Some(err_msg.clone()), None).await;

            result.patch_pkt = Some(proto::log_msg(3, &err_msg));
            return result;
        }
    };

    result.join_rooms = join_rooms.clone();
    result.leave_rooms = leave_rooms.clone();

    // 4. Traiter les Salons (Rooms)
    for room in join_rooms {
        let _ = session.sm.join_room(session.state.session_id.clone(), room).await;
    }
    for room in leave_rooms {
        let _ = session.sm.leave_room(session.state.session_id.clone(), room).await;
    }

    if !patches.is_empty() {
        let mut final_patches = Vec::new();
        for (mut op, _) in patches {
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
            final_patches.push(op); 
        }

        if !final_patches.is_empty() {
            if let Some(max_ver) = final_patches.iter().map(|op| op.version).max() {
                if max_ver > 0 { session.state.last_version = max_ver; }
            }
            let elapsed = start.elapsed().as_millis() as u64;
            let patch_pkt = proto::patch(&final_patches);
            monitor_pkt(&state.tx_monitor, "OUT", proto::PKT_PATCH, patch_pkt.len(), &session.state.session_id, Some(handler.clone()), Some(format!("Applied {} patches", final_patches.len())), Some(elapsed)).await;
            result.patch_pkt = Some(patch_pkt);
        }
    }

    // 5. Traitement du Broadcast v0.7.0 (SDK-driven)
    if let Some(bc) = broadcast_instr {
        info!("[{}] BROADCAST via PHP (scope: {})", session.state.session_id, bc.scope);
        let bc_pkt = proto::patch(&bc.patches);
        
        let mut msg = Vec::new();
        let scope_type = match bc.scope.as_str() {
            "room" if bc.room_id.is_some() => proto::SCOPE_ROOM,
            "room" => {
                warn!("[{}] Broadcast ROOM requis mais room_id manquant !", session.state.session_id);
                return result;
            },
            "direct" if bc.target_sid.is_some() => proto::SCOPE_DIRECT,
            "direct" => {
                warn!("[{}] Broadcast DIRECT requis mais target_sid manquant !", session.state.session_id);
                return result;
            },
            "all" => proto::SCOPE_ALL,
            "others" => proto::SCOPE_OTHERS,
            _ => {
                warn!("[{}] Scope de broadcast inconnu: {}", session.state.session_id, bc.scope);
                return result;
            }
        };
        
        msg.push(scope_type);
        msg.push(state.gateway_id.len() as u8);
        msg.extend_from_slice(state.gateway_id.as_bytes());
        msg.push(session.state.session_id.len() as u8);
        msg.extend_from_slice(session.state.session_id.as_bytes());
        
        if scope_type == proto::SCOPE_ROOM {
            let rid = bc.room_id.unwrap_or_else(|| "global".to_string());
            msg.push(rid.len() as u8);
            msg.extend_from_slice(rid.as_bytes());
        } else if scope_type == proto::SCOPE_DIRECT {
            let tsid = bc.target_sid.unwrap_or_else(|| "".to_string());
            msg.push(tsid.len() as u8);
            msg.extend_from_slice(tsid.as_bytes());
        }
        
        msg.extend_from_slice(&bc_pkt);
        let _ = state.tx_app_broadcast.send(Arc::new(msg));
    }

    result
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
        node_id: None, // Node ID could be added if needed
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
    state         : Arc<GatewayState>,
) -> crate::core::Result<(Vec<(proto::PatchOp, bool)>, Option<proto::BroadcastInstruction>, Vec<String>, Vec<String>)>
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

    if let Some(ref pool) = state.fpm_pool {
        call_php_fpm(pool, php_script, &context, handler_table, state.fpm_timeout).await
    } else {
        call_php_process(php_script, &context, handler_table).await
    }
}

async fn call_php_process(
    php_script: &str,
    context: &serde_json::Value,
    handler_table: &HandlerTable,
) -> crate::core::Result<(Vec<(proto::PatchOp, bool)>, Option<proto::BroadcastInstruction>, Vec<String>, Vec<String>)> {
    use std::process::Stdio;
    use tokio::process::Command;
    use tokio::io::AsyncWriteExt;

    let input = context.to_string();
    
    let script_path = std::path::Path::new(php_script);
    let abs_script_path = std::fs::canonicalize(script_path).unwrap_or_else(|_| script_path.to_path_buf());
    let abs_str = abs_script_path.to_string_lossy().to_string();
    let clean_abs_str = if abs_str.starts_with(r"\\?\") {
        abs_str[4..].to_string()
    } else {
        abs_str
    };
    
    let clean_path = std::path::Path::new(&clean_abs_str);
    let script_dir = clean_path.parent().unwrap_or(std::path::Path::new("."));
    let script_dir_str = script_dir.to_string_lossy().to_string();

    let mut child = Command::new("php")
        .arg("-f")
        .arg(&clean_abs_str)
        .current_dir(script_dir_str)
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
    pool: &FpmPool,
    php_script: &str,
    context: &serde_json::Value,
    handler_table: &HandlerTable,
    timeout_ms: u64,
) -> crate::core::Result<(Vec<(proto::PatchOp, bool)>, Option<proto::BroadcastInstruction>, Vec<String>, Vec<String>)> {
    use fastcgi_client::{Params, Request};
    use tokio::time::{timeout, Duration};

    let input = context.to_string();
    
    // Acquérir un client du pool
    let mut client = pool.acquire().await?;

    let mut params = Params::default();
    params.insert("SCRIPT_FILENAME".into(), php_script.into());
    params.insert("REQUEST_METHOD".into(), "POST".into());
    params.insert("CONTENT_TYPE".into(), "application/json".into());
    params.insert("CONTENT_LENGTH".into(), input.len().to_string().into());

    let mut body = input.as_bytes();
    
    // Exécuter la requête (Keep-Alive) avec TIMEOUT
    let output_res = timeout(
        Duration::from_millis(timeout_ms),
        client.execute(Request::new(params, &mut body))
    ).await;

    match output_res {
        Ok(Ok(output)) => {
            // Rendre le client au pool s'il est toujours fonctionnel
            pool.release(client).await;
            let stdout = output.stdout.as_deref().unwrap_or(&[]);
            Ok(parse_php_response(stdout, handler_table))
        }
        Ok(Err(e)) => {
            // Ne PAS rendre le client s'il y a une erreur de socket
            Err(crate::core::GatewayError::FastCgiError(e.to_string()))
        }
        Err(_) => {
            // Timeout atteint
            error!("[FPM] Timeout de {}ms atteint pour {}", timeout_ms, php_script);
            Err(crate::core::GatewayError::FastCgiError(format!("Timeout FastCGI ({}ms)", timeout_ms)))
        }
    }
}

// ─── Parser la réponse PHP ──────────────────────────────────────────────────

fn parse_php_response(
    stdout        : &[u8],
    handler_table : &HandlerTable,
) -> (Vec<(proto::PatchOp, bool)>, Option<proto::BroadcastInstruction>, Vec<String>, Vec<String>)
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
            return (Vec::new(), None, Vec::new(), Vec::new());
        }
    };
    
    // Le SDK PHP renvoie soit un objet {"patch": [...]} soit directement le tableau [...]
    let ops_array = if json.is_array() {
        json.as_array()
    } else {
        json.get("patch").and_then(|v| v.as_array())
    };

    let mut broadcast = None;
    let mut join_rooms = Vec::new();
    let mut leave_rooms = Vec::new();

    if !json.is_array() {
        // --- Broadcast ---
        if let Some(bc) = json.get("broadcast") {
            let scope = bc["scope"].as_str().unwrap_or("others").to_string();
            let room_id = bc.get("room_id").and_then(|v| v.as_str()).map(|s| s.to_string());
            let target_sid = bc.get("target_sid").and_then(|v| v.as_str()).map(|s| s.to_string());
            
            let mut bc_ops = Vec::new();
            if let Some(patches) = bc["patch"].as_array() {
                for op in patches {
                    if let Some(p) = parse_single_op(op, handler_table) {
                        bc_ops.push(p);
                    }
                }
            }
            if !bc_ops.is_empty() {
                broadcast = Some(proto::BroadcastInstruction { scope, room_id, target_sid, patches: bc_ops });
            }
        }

        // --- Join Room ---
        if let Some(j) = json.get("join_room") {
            if let Some(s) = j.as_str() {
                join_rooms.push(s.to_string());
            } else if let Some(arr) = j.as_array() {
                for v in arr {
                    if let Some(s) = v.as_str() { join_rooms.push(s.to_string()); }
                }
            }
        }

        // --- Leave Room ---
        if let Some(l) = json.get("leave_room") {
            if let Some(s) = l.as_str() {
                leave_rooms.push(s.to_string());
            } else if let Some(arr) = l.as_array() {
                for v in arr {
                    if let Some(s) = v.as_str() { leave_rooms.push(s.to_string()); }
                }
            }
        }
    }

    let mut patch_ops = Vec::new();
    let Some(ops) = ops_array else { return (Vec::new(), None, join_rooms, leave_rooms); };

    for op in ops {
        let is_broadcast = op["broadcast"].as_bool().unwrap_or(false);
        if let Some(patch) = parse_single_op(op, handler_table) {
            patch_ops.push((patch, is_broadcast));
        }
    }

    (patch_ops, broadcast, join_rooms, leave_rooms)
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
