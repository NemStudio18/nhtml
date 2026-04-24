mod core;
mod cli;
mod supervisor;
mod watcher;
mod session;
mod proto;
mod decoder;
mod config;

use clap::{Parser, Subcommand};
use tokio::net::TcpListener;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_hdr_async, tungstenite::Message};
use serde::{Serialize, Deserialize};
use serde_json;
use uuid::Uuid;
use chrono::Utc;
use axum::{Router, routing::get, response::IntoResponse, http::{StatusCode, header, HeaderMap}, extract::Request as AxumRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringEvent {
    pub session_id: String,
    pub direction: String, // "IN" ou "OUT"
    pub pkt_type: u8,
    pub size: usize,
    pub timestamp: i64,
    pub handler: Option<String>,
    pub latency_ms: Option<u128>,
    pub compression_ratio: Option<f32>,
}

#[derive(Parser)]
#[command(name = "nhtml", about = "Le Gateway Native-HTML (v0.2.2)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New { name: String },
    Start {
        #[arg(long)]
        dev: bool,
        #[arg(long)]
        ws_port: Option<u16>,
        #[arg(long)]
        php_port: Option<u16>,
    },
    DbDump,
    Inspect { hex: String },
    Validate { path: String },
    Devtools {
        #[arg(long)]
        port: Option<u16>,
    },
    Bench { path: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config_file = config::NhtmlConfig::load();

    match &cli.command {
        Commands::New { name } => { cli::create_new_project(name); }
        Commands::Start { dev, ws_port, php_port } => {
            let (tx_monitor, _) = broadcast::channel::<MonitoringEvent>(100);
            
            let is_dev = *dev || config_file.dev.and_then(|d| d.auto_reload).unwrap_or(false);
            let final_ws_port = ws_port.or(config_file.ports.as_ref().and_then(|p| p.ws)).unwrap_or(8080);
            let final_php_port = php_port.or(config_file.ports.as_ref().and_then(|p| p.php)).unwrap_or(8000);
            let final_http_port = config_file.ports.as_ref().and_then(|p| p.http).unwrap_or(3000);

            if is_dev {
                let tx_monitor_for_devtools = tx_monitor.clone();
                let devtools_port = config_file.ports.as_ref().and_then(|p| p.devtools).unwrap_or(8081);
                tokio::spawn(async move {
                    crate::cli::run_devtools_on_port(tx_monitor_for_devtools, devtools_port).await;
                });
            }
            start_gateway(is_dev, final_ws_port, final_php_port, final_http_port, tx_monitor).await;
        }
        Commands::DbDump => { cli::dump_database(); }
        Commands::Inspect { hex } => { cli::inspect_message(hex); }
        Commands::Validate { path } => { cli::validate_file(path); }
        Commands::Devtools { port } => {
            let final_port = port.or(config_file.ports.as_ref().and_then(|p| p.devtools)).unwrap_or(8081);
            let (tx_monitor, _) = broadcast::channel::<MonitoringEvent>(100);
            cli::run_devtools_on_port(tx_monitor, final_port).await;
        }
        Commands::Bench { path } => { cli::run_benchmark(path); }
    }
}

async fn start_gateway(is_debug: bool, ws_port: u16, php_port: u16, http_port: u16, tx_monitor: broadcast::Sender<MonitoringEvent>) {
    if is_debug {
        println!("🚀 NHTML Gateway démarré en mode DEBUG (--dev)");
    } else {
        println!("🌐 NHTML Gateway démarré en mode PRODUCTION");
    }
    let (tx_app_broadcast, _) = broadcast::channel::<Vec<u8>>(128);
    tokio::spawn(supervisor::start_php_server(php_port, tx_monitor.clone(), tx_app_broadcast.clone()));

    let (tx_reload, _) = broadcast::channel::<()>(16);
    if is_debug {
        watcher::start_watcher(tx_reload.clone());
    }

    let session_manager = std::sync::Arc::new(
        session::SessionManager::new().await.expect("Échec de l'initialisation de SQLite")
    );

    let addr = format!("127.0.0.1:{}", ws_port);
    let listener = TcpListener::bind(&addr).await.expect("Impossible de lier le port WS");
    println!("📍 Gateway (WebSocket) à l'écoute sur : ws://{}", addr);

    // --- SERVEUR HTTP AVEC AUTO-INJECTION ---
    tokio::spawn(async move {
        let app = Router::new()
            .route("/_nhtml/bridge.js", get(|| async {
                let body = include_str!("../../nhtml-core/examples/assets/js/bridge.js");
                (StatusCode::OK, [(header::CONTENT_TYPE, "application/javascript")], body)
            }))
            .route("/_nhtml/fzstd.js", get(|| async {
                let body = include_str!("../../nhtml-core/examples/assets/js/fzstd.min.js");
                (StatusCode::OK, [(header::CONTENT_TYPE, "application/javascript")], body)
            }))
            .route("/_nhtml/php-wasm/PhpWeb.mjs", get(|| async {
                let body = include_str!("../../nhtml-core/examples/assets/js/php-wasm/PhpWeb.mjs");
                (StatusCode::OK, [(header::CONTENT_TYPE, "application/javascript")], body)
            }))
            .route("/_nhtml/php-wasm/PhpBase.mjs", get(|| async {
                let body = include_str!("../../nhtml-core/examples/assets/js/php-wasm/PhpBase.mjs");
                (StatusCode::OK, [(header::CONTENT_TYPE, "application/javascript")], body)
            }))
            .route("/_nhtml/php-wasm/php-web.mjs", get(|| async {
                let body = include_str!("../../nhtml-core/examples/assets/js/php-wasm/php-web.mjs");
                (StatusCode::OK, [(header::CONTENT_TYPE, "application/javascript")], body)
            }))
            .route("/_nhtml/php-wasm/php-web.mjs.wasm", get(|| async {
                let body = include_bytes!("../../nhtml-core/examples/assets/js/php-wasm/php-web.mjs.wasm");
                (StatusCode::OK, [(header::CONTENT_TYPE, "application/wasm")], body)
            }))
            .fallback(get(move |req: AxumRequest| async move {
                let path = req.uri().path().trim_start_matches('/').to_string();
                let file_path = if path.is_empty() { "counter/index.nhtml".to_string() } else { path };

                if let Ok(mut content) = std::fs::read_to_string(&file_path) {
                    if file_path.ends_with(".nhtml") {
                        let injection = format!(
                            "\n    <script src=\"/_nhtml/fzstd.js\"></script>\n    <script src=\"/_nhtml/bridge.js\"></script>\n    <script>const nhtml = new NHTMLBridge({{ ws: 'ws://' + window.location.hostname + ':{}?sid=AUTO', debug: true }});</script>",
                            ws_port
                        );
                        content = content.replace("</head>", &format!("{}\n</head>", injection));
                    }
                    return (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], content).into_response();
                }
                (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/plain")], "404 Not Found".to_string()).into_response()
            }));

        let http_addr = format!("127.0.0.1:{}", http_port);
        let http_listener = tokio::net::TcpListener::bind(&http_addr).await
            .unwrap_or_else(|_| panic!("Impossible de lier le port HTTP {}", http_port));
        println!("🌍 Serveur Web NHTML : http://{}", http_addr);
        axum::serve(http_listener, app).await.unwrap();
    });

    while let Ok((stream, _)) = listener.accept().await {
        let is_debug = is_debug;
        let mut rx_reload = tx_reload.subscribe();
        let manager = session_manager.clone();
        let tx_monitor_ws = tx_monitor.clone();
        let tx_app_broadcast_ws = tx_app_broadcast.clone();
        let mut session_id = Uuid::new_v4().to_string();

        tokio::spawn(async move {
            let ws_stream = match accept_hdr_async(stream, |req: &tokio_tungstenite::tungstenite::handshake::server::Request, res: tokio_tungstenite::tungstenite::handshake::server::Response| {
                if let Some(query) = req.uri().query() {
                    for p in query.split('&') {
                        if p.starts_with("sid=") { session_id = p[4..].to_string(); }
                    }
                }
                Ok(res)
            }).await {
                Ok(s) => s,
                Err(e) => { eprintln!("❌ Erreur handshake WS: {}", e); return; }
            };

            let (mut ws_sender, mut ws_receiver) = ws_stream.split();
            let (tx_ws, mut rx_ws) = tokio::sync::mpsc::channel::<Message>(128);
            println!("🔌 Connexion client WS (Session: {})", session_id);

            // Tâche de sortie UNIQUE (Writer)
            tokio::spawn(async move {
                use futures_util::SinkExt;
                while let Some(msg) = rx_ws.recv().await {
                    if let Err(_) = ws_sender.send(msg).await { break; }
                }
            });

            // Relai des logs globaux (Broadcast -> MPSC)
            let mut rx_global = tx_app_broadcast_ws.subscribe();
            let tx_ws_log = tx_ws.clone();
            tokio::spawn(async move {
                loop {
                    match rx_global.recv().await {
                        Ok(msg) => {
                            let _ = tx_ws_log.send(Message::Binary(msg)).await;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(_) => break,
                    }
                }
            });

            loop {
                let tx_ws = tx_ws.clone();
                tokio::select! {
                    msg = ws_receiver.next() => {
                        match msg {
                            Some(Ok(message)) => {
                                if message.is_binary() {
                                    let data = message.into_data();
                                    
                                    tx_monitor_ws.send(MonitoringEvent {
                                        session_id: session_id.clone(),
                                        direction: "IN".to_string(),
                                        pkt_type: data[0],
                                        size: data.len(),
                                        timestamp: chrono::Utc::now().timestamp_millis(),
                                        handler: None,
                                        latency_ms: None,
                                        compression_ratio: None,
                                    }).ok();

                                    if let Some(packets) = handle_binary_packet(&data, is_debug, php_port, &session_id, &manager, &tx_monitor_ws).await {
                                        for patch_data in packets {
                                            let _ = tx_ws.send(Message::Binary(patch_data)).await;
                                        }
                                    }
                                }
                            }
                            _ => break,
                        }
                    }
                    Ok(_) = rx_reload.recv() => {
                        let _ = tx_ws.send(Message::Binary(vec![0x09])).await;
                    }
                }
            }
        });
    }
}

async fn handle_binary_packet(
    data: &[u8], 
    _debug: bool, 
    php_port: u16,
    session_id: &str,
    manager: &session::SessionManager,
    tx_monitor: &broadcast::Sender<MonitoringEvent>
) -> Option<Vec<Vec<u8>>> {
    if data.is_empty() { return None; }
    
    let mut response_packets = Vec::new();
    let pkt_type = data[0];
    let start_time = std::time::Instant::now();
    let client = reqwest::Client::new();

    match pkt_type {
        0x01 => { // HELLO
            let all_nodes = manager.get_all_nodes(session_id.to_string()).await.unwrap_or_default();
            if !all_nodes.is_empty() {
                let (btree_data, ratio) = proto::btree(&all_nodes);
                let final_size = btree_data.len();
                
                tx_monitor.send(MonitoringEvent {
                    session_id: session_id.to_string(),
                    direction: "OUT".to_string(),
                    pkt_type: 0x07,
                    size: final_size,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    handler: Some("sync".to_string()),
                    latency_ms: Some(start_time.elapsed().as_millis()),
                    compression_ratio: Some(ratio),
                }).ok();

                response_packets.push(btree_data);
                return Some(response_packets);
            }
            let res = client.post(format!("http://127.0.0.1:{}/counter/app.php", php_port))
                .json(&serde_json::json!({ "nhtml_event": "init", "current_state": {} }))
                .send().await;

            if let Ok(r) = res {
                if let Ok(body) = r.text().await {
                    let mut binary_ops = Vec::new();
                    if let Ok(patches) = serde_json::from_str::<serde_json::Value>(&body) {
                        if let Some(patch_list) = patches.as_array() {
                            for p in patch_list {
                                if p.get("op") == Some(&serde_json::json!("set_text")) {
                                    if let (Some(nid_json), Some(val)) = (p.get("nid"), p.get("value")) {
                                        let nid_str = nid_json.as_str().unwrap_or("");
                                        let node_id = if nid_str == "counter_value" { 2 } else { nid_str.parse::<u16>().unwrap_or(0) };
                                        if node_id > 0 {
                                            let val_str = val.as_str().unwrap_or("").to_string();
                                            let new_ver = manager.update_node(session_id.to_string(), node_id as u32, val_str.clone()).await.unwrap_or(0);
                                            binary_ops.push(proto::PatchOp::set_text(node_id, new_ver, &val_str));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !binary_ops.is_empty() { response_packets.push(proto::patch(&binary_ops)); }
                }
            }
            if response_packets.is_empty() { response_packets.push(vec![0x01, 0x00, 0x00, 0x00, 0x00]); }
            Some(response_packets)
        },
        0x02 => { // EVENT
            if data.len() < 5 { return None; }
            let node_id = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
            let (current_val, _) = manager.get_node_state(session_id.to_string(), 2).await.unwrap_or(("0".to_string(), 0));
            
            let res = client.post(format!("http://127.0.0.1:{}/counter/app.php", php_port))
                .json(&serde_json::json!({
                    "nhtml_event": "click",
                    "node_id": if node_id == 1 { "btn_increment" } else { "unknown" },
                    "current_state": { "counter_value": current_val }
                })).send().await;

            if let Ok(r) = res {
                if let Ok(body) = r.text().await {
                    let mut binary_ops = Vec::new();
                    if let Ok(patches) = serde_json::from_str::<serde_json::Value>(&body) {
                        if let Some(patch_list) = patches.as_array() {
                            for p in patch_list {
                                if p.get("op") == Some(&serde_json::json!("set_text")) {
                                    if let (Some(nid_json), Some(val)) = (p.get("nid"), p.get("value")) {
                                        let nid_str = nid_json.as_str().unwrap_or("");
                                        let nid = if nid_str == "counter_value" { 2 } else { nid_str.parse::<u16>().unwrap_or(0) };
                                        if nid > 0 {
                                            let val_str = val.as_str().unwrap_or("").to_string();
                                            let new_ver = manager.update_node(session_id.to_string(), nid as u32, val_str.clone()).await.unwrap_or(0);
                                            binary_ops.push(proto::PatchOp::set_text(nid, new_ver, &val_str));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    if !binary_ops.is_empty() {
                        let patch_data = proto::patch(&binary_ops);
                        tx_monitor.send(MonitoringEvent {
                            session_id: session_id.to_string(),
                            direction: "OUT".to_string(),
                            pkt_type: patch_data[0],
                            size: patch_data.len(),
                            timestamp: chrono::Utc::now().timestamp_millis(),
                            handler: Some("click_patch".to_string()),
                            latency_ms: Some(start_time.elapsed().as_millis()),
                            compression_ratio: None,
                        }).ok();
                        response_packets.push(patch_data);
                    } else {
                        response_packets.push(body.into_bytes());
                    }
                    return Some(response_packets);
                }
            }
            None
        },
        _ => None
    }
}
