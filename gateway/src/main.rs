mod core;
mod cli;
mod supervisor;
mod watcher;
mod session;
mod proto;
mod decoder;

use clap::{Parser, Subcommand};
use tokio::net::TcpListener;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_hdr_async, tungstenite::Message};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringEvent {
    pub session_id: String,
    pub direction: String, // "IN" ou "OUT"
    pub pkt_type: u8,
    pub size: usize,
    pub timestamp: i64,
    pub handler: Option<String>,
    pub latency_ms: Option<u128>,
}

#[derive(Parser)]
#[command(name = "nhtml", about = "Le Gateway Native-HTML (v0.2.2)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Crée un nouveau projet NHTML de base
    New {
        /// Le nom du dossier du projet
        name: String,
    },
    #[command(about = "Démarre le Gateway et le serveur PHP")]
    Start {
        #[arg(long, help = "Active le mode debug (logs étendus)")]
        dev: bool,
    },

    #[command(about = "Affiche le contenu de la base de données de session")]
    DbDump,

    /// Décode un message binaire hexadécimal
    Inspect {
        /// Le message hexadécimal (ex: 0100000000...)
        hex: String,
    },

    /// Valide un fichier binaire NBPS
    Validate {
        /// Le chemin du fichier binaire
        path: String,
    },
    /// Ouvre les DevTools NHTML (Dashboard, Time Travel, etc.)
    Devtools,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { name } => {
            cli::create_new_project(name);
        }
        Commands::Start { dev } => {
            start_gateway(*dev).await;
        }
        Commands::DbDump => {
            cli::dump_database();
        }
        Commands::Inspect { hex } => {
            cli::inspect_message(hex);
        }
        Commands::Validate { path } => {
            cli::validate_file(path);
        }
        Commands::Devtools => {
            cli::run_devtools().await;
        }
    }
}

async fn start_gateway(is_debug: bool) {
    if is_debug {
        println!("🚀 NHTML Gateway démarré en mode DEBUG (--dev)");
        
        // Magie de l'expérience développeur : Lancer les DevTools automatiquement !
        let (tx_monitor, _) = broadcast::channel::<MonitoringEvent>(100);
        let rx_monitor = tx_monitor.clone();

        tokio::spawn(async move {
            crate::cli::run_devtools(rx_monitor).await;
        });
        
        start_gateway(true, tx_monitor).await;
    } else {
        println!("🌐 NHTML Gateway démarré en mode PRODUCTION");
        // En prod, on peut quand même lancer le gateway sans moniteur ou avec un moniteur vide
        let (tx_monitor, _) = broadcast::channel::<MonitoringEvent>(1);
        start_gateway(false, tx_monitor).await;
    }
}

async fn start_gateway(is_debug: bool, tx_monitor: broadcast::Sender<MonitoringEvent>) {
    // 1. Démarrage du Supervisor (PHP)
    let php_port = 8000;
    tokio::spawn(supervisor::start_php_server(php_port));

    // 2. Démarrage du Watcher (Hot Reload)
    let (tx_reload, _) = broadcast::channel::<()>(16);
    let tx_watcher = tx_reload.clone();
    if is_debug {
        watcher::start_watcher(tx_watcher);
    }

    // 3. Initialisation du SessionManager (SQLite)
    let session_manager = std::sync::Arc::new(
        session::SessionManager::new().await.expect("Échec de l'initialisation de SQLite")
    );

    // 4. Démarrage du serveur WebSocket
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Impossible de lier le port WS 8080");
    println!("📍 Gateway (WebSocket) à l'écoute sur : ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let is_debug = is_debug;
        let mut rx_reload = tx_reload.subscribe();
        let manager = session_manager.clone();
        let tx_monitor = tx_monitor.clone();

        tokio::spawn(async move {
            let mut session_id = uuid::Uuid::new_v4().to_string();

            // Handshake personnalisé pour extraire le SID de l'URL
            let ws_stream = match accept_hdr_async(stream, |req: &tokio_tungstenite::tungstenite::handshake::server::Request, res: tokio_tungstenite::tungstenite::handshake::server::Response| {
                if let Some(query) = req.uri().query() {
                    let params: Vec<_> = query.split('&').collect();
                    for p in params {
                        if p.starts_with("sid=") {
                            session_id = p[4..].to_string();
                        }
                    }
                }
                Ok(res)
            }).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("❌ Erreur handshake WS: {}", e);
                    return;
                }
            };

            let (mut ws_sender, mut ws_receiver) = ws_stream.split();
            let tx_monitor = tx_monitor.clone();
            
            println!("🔌 Nouvelle connexion client WS (Session: {})", session_id);

            // Le serveur attendra désormais le HELLO du client avant de répondre.
            // Cela évite les collisions de paquets au démarrage.

            loop {
                tokio::select! {
                    msg = ws_receiver.next() => {
                        match msg {
                            Some(Ok(message)) => {
                                if message.is_binary() {
                                    let data = message.into_data();
                                    
                                    if is_debug {
                                        println!("📥 IN HEX: {:02X?}", data);
                                        let decoded = decoder::decode(&data);
                                        println!("📥 IN DEC: {:?}", decoded);
                                    }
                                    // Spécial HELLO : On extrait le SID potentiel du client
                                    if !data.is_empty() && data[0] == 0x01 && data.len() > 5 {
                                        let client_sid = String::from_utf8_lossy(&data[5..]).to_string();
                                        if !client_sid.is_empty() && client_sid != session_id {
                                            if is_debug { println!("🔄 Session switch: {} -> {}", session_id, client_sid); }
                                            session_id = client_sid;
                                        }
                                    }

                                        tx_monitor.send(MonitoringEvent {
                                        session_id: session_id.clone(),
                                        direction: "IN".to_string(),
                                        pkt_type: data[0],
                                        size: data.len(),
                                        timestamp: chrono::Utc::now().timestamp_millis(),
                                        handler: None,
                                        latency_ms: None,
                                    }).ok();

                                            if let Some(packets) = handle_binary_packet(&data, is_debug, php_port, &session_id, &manager, &tx_monitor).await {
                                        for patch_data in packets {
                                            if is_debug {
                                                let decoded = decoder::decode(&patch_data);
                                                println!("📤 OUT DEC: {:?}", decoded);
                                                println!("📤 OUT HEX: {:02X?}", patch_data);
                                            }
                                            match ws_sender.send(Message::Binary(patch_data)).await {
                                                Ok(_) => { if is_debug { println!("✅ Message envoyé avec succès au client."); } }
                                                Err(e) => { println!("❌ Erreur fatale lors de l'envoi WS: {:?}", e); break; }
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                if is_debug { println!("❌ Erreur WebSocket: {:?}", e); }
                                break;
                            }
                            None => {
                                if is_debug { println!("ℹ️ Flux WebSocket fermé par le client (None)"); }
                                break;
                            }
                        }
                    }
                    
                    // Écoute des signaux de Hot Reload du Watcher
                    Ok(_) = rx_reload.recv() => {
                        println!("🔥 Envoi du paquet RELOAD (0x09) au client...");
                        let reload_packet = vec![0x09];
                        let _ = ws_sender.send(Message::Binary(reload_packet)).await;
                    }
                }
            }
            println!("🔌 Client déconnecté");
        });
    }
}

async fn handle_binary_packet(
    data: &[u8], 
    debug: bool, 
    php_port: u16,
    session_id: &str,
    manager: &session::SessionManager,
    tx_monitor: &broadcast::Sender<MonitoringEvent>
) -> Option<Vec<Vec<u8>>> {
    if data.is_empty() { return None; }
    
    let mut response_packets = Vec::new();
    let pkt_type = data[0];
    let start_time = std::time::Instant::now();

    match pkt_type {
        0x01 => { // HELLO — Handshake & State Sync
            let last_ver = if data.len() >= 5 {
                u32::from_be_bytes([data[1], data[2], data[3], data[4]])
            } else { 0 };

            if debug { println!("👋 HELLO reçu (Session: {}, LastVer: {})", session_id, last_ver); }
            
            // 1. Récupérer TOUS les nœuds existants pour cette session
            let all_nodes = manager.get_all_nodes(session_id.to_string()).await.unwrap_or_default();

            if !all_nodes.is_empty() {
                if debug { println!("📦 Full-Path : Envoi du B-TREE Snapshot ({} nœuds)", all_nodes.len()); }
                response_packets.push(proto::btree(&all_nodes));
                return Some(response_packets);
            }

            // 2. Si aucun nœud (nouvelle session), on appelle PHP init
            if debug { println!("🆕 Nouvelle session : Appel de PHP init"); }
            let client = reqwest::Client::new();
            let res = client.post(format!("http://127.0.0.1:{}/counter/app.php", php_port))
                .json(&serde_json::json!({ 
                    "nhtml_event": "init",
                    "current_state": {} 
                }))
                .send()
                .await;

            if let Ok(r) = res {
                if let Ok(body) = r.text().await {
                    if debug { println!("✅ Réponse PHP (Init) : {}", body); }
                    
                    let mut binary_ops = Vec::new();
                    if let Ok(patches) = serde_json::from_str::<serde_json::Value>(&body) {
                        if let Some(patch_list) = patches.as_array() {
                            for p in patch_list {
                                if p.get("op") == Some(&serde_json::json!("set_text")) {
                                    if let (Some(nid_json), Some(val)) = (p.get("nid"), p.get("value")) {
                                        let nid_str = nid_json.as_str().unwrap_or("");
                                        let node_id = match nid_str {
                                            "counter_value" => 2,
                                            _ => nid_str.parse::<u16>().unwrap_or(0)
                                        };
                                        if node_id > 0 {
                                            let val_str = val.as_str().unwrap_or("").to_string();
                                            let new_ver = manager.update_node(session_id.to_string(), node_id as u32, val_str.clone()).await.unwrap_or(0);
                                            binary_ops.push(proto::PatchOp::set_text(node_id, new_ver, &val_str));
                                        }
                                    }
                                } else if p.get("op") == Some(&serde_json::json!("log")) {
                                    if let Some(val) = p.get("value") {
                                        let msg = val.as_str().unwrap_or("");
                                        response_packets.push(proto::log_msg(1, msg)); // Severity 1 = INFO
                                    }
                                }
                            }
                        }
                    }

                    if !binary_ops.is_empty() {
                        response_packets.push(proto::patch(&binary_ops));
                    }
                    
                    if response_packets.is_empty() {
                        response_packets.push(vec![0x01, 0x00, 0x00, 0x00, 0x00]);
                    }
                    return Some(response_packets);
                }
            }
            response_packets.push(vec![0x01, 0x00, 0x00, 0x00, 0x00]);
            Some(response_packets)
        },
        0x02 => { // EVENT
            if data.len() >= 5 {
                let node_id = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
                if debug { println!("⚡ EVENT reçu (nid: {}, session: {})", node_id, session_id); }
                
                // P0 : Logging de l'événement dans SQLite
                let _ = manager.log_event(session_id.to_string(), node_id, "CLICK".to_string(), "nhtml_event: click".to_string()).await;

                // [P3] Sécurité : Vérifier si le nœud est valide pour cette session
                if node_id != 1 {
                    let (val, _) = manager.get_node_state(session_id.to_string(), node_id).await.unwrap_or(("".to_string(), 0));
                    if val.is_empty() {
                        if debug { println!("⚠️ ALERTE SÉCURITÉ : Événement rejeté (nœud inconnu: {} pour session: {})", node_id, session_id); }
                        return None;
                    }
                }

                // P0 : Récupération de l'état actuel pour rendre PHP stateless
                let (current_val, _) = manager.get_node_state(session_id.to_string(), 2).await.unwrap_or(("0".to_string(), 0));
                
                let payload = serde_json::json!({
                    "nhtml_event": "click",
                    "node_id": if node_id == 1 { "btn_increment" } else { "unknown" },
                    "current_state": { "counter_value": current_val }
                });

                if debug { println!("📡 Envoi à PHP : {}", payload); }

                let res = client.post(format!("http://127.0.0.1:{}/counter/app.php", php_port))
                    .json(&payload)
                    .send()
                    .await;
                
                if let Ok(r) = res {
                    if let Ok(body) = r.text().await {
                        if debug { println!("✅ Réponse PHP : {}", body); }
                        
                        // P0 : Analyse des patches pour versionnage et conversion BINAIRE
                        let mut binary_ops = Vec::new();

                        if let Ok(patches) = serde_json::from_str::<serde_json::Value>(&body) {
                            if let Some(patch_list) = patches.as_array() {
                                for p in patch_list {
                                    if let Some(op) = p.get("op") {
                                        if op == "set_text" {
                                            if let (Some(nid_json), Some(val)) = (p.get("nid"), p.get("value")) {
                                                let nid_str = nid_json.as_str().unwrap_or("");
                                                let node_id = match nid_str {
                                                    "counter_value" => 2,
                                                    _ => nid_str.parse::<u16>().unwrap_or(0)
                                                };
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
                        }
                        
                    if !binary_ops.is_empty() {
                        let patch_data = proto::patch(&binary_ops);
                        
                        // Envoi au Monitor enrichi (OUT)
                        tx_monitor.send(MonitoringEvent {
                            session_id: session_id.to_string(),
                            direction: "OUT".to_string(),
                            pkt_type: patch_data[0],
                            size: patch_data.len(),
                            timestamp: chrono::Utc::now().timestamp_millis(),
                            handler: Some("click".to_string()),
                            latency_ms: Some(start_time.elapsed().as_millis()),
                        }).ok();

                        return Some(vec![patch_data]);
                    }
                        
                        return Some(body.into_bytes());
                    }
                }
            }
            None
        },
        _ => {
            if debug { println!("❓ Paquet inconnu : type 0x{:02X}", pkt_type); }
            None
        }
    }
}

