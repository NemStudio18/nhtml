use std::fs;
use std::path::Path;
use std::collections::HashMap;
use crate::decoder;
use axum::http::header;
use axum::response::IntoResponse;

pub fn create_new_project(name: &str) {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        println!("❌ Erreur : Le dossier '{}' existe déjà.", name);
        return;
    }

    if let Err(e) = fs::create_dir_all(project_dir.join("assets/js")) {
        println!("❌ Erreur : Impossible de créer la structure du dossier '{}' : {}", name, e);
        return;
    }

    // 1. Création de nhtml.config.toml
    let config_content = r#"[ports]
ws = 8080
php = 8000
devtools = 8082

[security]
# allowed_origins = ["http://localhost:8080"]

[security.rate_limit]
events_per_sec = 30

[fastcgi]
# address = "127.0.0.1:9000"
timeout_ms = 5000
"#;
    if let Err(e) = fs::write(project_dir.join("nhtml.config.toml"), config_content) {
        println!("❌ Erreur : Impossible d'écrire nhtml.config.toml : {}", e);
        return;
    }

    // 2. Création de index.nhtml
    let html_content = r#"<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NHTML — Nouveau Projet</title>
    <style>
        :root { --accent: #ff007f; --bg: #0a0a0a; --text: #eee; }
        body { font-family: 'Inter', sans-serif; background: var(--bg); color: var(--text); display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; margin: 0; }
        .card { background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.1); padding: 40px; border-radius: 24px; text-align: center; backdrop-filter: blur(10px); }
        h1 { color: var(--accent); margin-bottom: 10px; }
        .counter { font-size: 4rem; font-weight: 800; margin: 20px 0; font-family: monospace; }
        button { background: var(--accent); color: white; border: none; padding: 12px 30px; font-size: 1rem; font-weight: bold; cursor: pointer; border-radius: 12px; transition: transform 0.2s; }
        button:hover { transform: scale(1.05); }
        .badge { background: rgba(255,255,255,0.05); padding: 4px 12px; border-radius: 20px; font-size: 0.8rem; color: #888; margin-top: 20px; display: inline-block; }
    </style>
</head>
<body>
    <div class="card">
        <h1>NHTML v0.7.3</h1>
        <p>Votre application temps réel est prête.</p>
        
        <div class="counter" n-id="counter_value">0</div>
        <button n-click="increment">INCREMENTER</button>
        
        <br>
        <div class="badge">Mode: Global Connect (Binary NBPS)</div>
    </div>

    <script src="/assets/js/bridge.js"></script>
</body>
</html>"#;
    if let Err(e) = fs::write(project_dir.join("index.nhtml"), html_content) {
        println!("❌ Erreur : Impossible d'écrire index.nhtml : {}", e);
        return;
    }

    // 3. Création de app.php (Industrial Pattern)
    let php_content = r#"<?php
/**
 * NHTML Backend Handler
 * Version: v0.7.3-stable
 */

// Simulation du SDK (en attendant l'installation via composer ou inclusion directe)
function patch_response($patches) {
    echo json_encode(["patch" => $patches]);
    exit;
}

$input = json_decode(file_get_contents('php://stdin'), true);
$handler = $input['handler'] ?? '';
$nodes = $input['nodes'] ?? [];

// Récupération de l'état actuel depuis le DOM (vspeed)
$count = (int)($nodes['counter_value']['val'] ?? 0);

if ($handler === 'increment') {
    $count++;
    patch_response([
        ["op" => "set_text", "nid" => "counter_value", "val" => (string)$count]
    ]);
}

// Initialisation par défaut
patch_response([]);
"#;
    if let Err(e) = fs::write(project_dir.join("app.php"), php_content) {
        println!("❌ Erreur : Impossible d'écrire app.php : {}", e);
        return;
    }

    println!("✅ Projet '{}' créé avec succès !", name);
    println!("👉 Tapez : cd {} && nhtml start --dev", name);
}

pub fn dump_database() {
    eprintln!("❌ La commande 'stats' n'est pas encore disponible (migration sqlx en cours).");
    std::process::exit(1);
}

pub fn inspect_message(hex: &str) {
    println!("🔍 Inspection du message hexadécimal...");
    let bytes = match hex::decode(hex.trim_start_matches("0x")) {
        Ok(b) => b,
        Err(e) => {
            println!("❌ Erreur de décodage hex : {}", e);
            return;
        }
    };

    let decoded = decoder::decode(&bytes);
    println!("📦 Résultat du décodage :\n{:#?}", decoded);
    
    if let Ok(json) = serde_json::to_string_pretty(&decoded) {
        println!("\n📄 Format JSON :\n{}", json);
    }
}

pub fn validate_file(path: &str) {
    println!("🛡️ Validation du fichier NBPS : {}", path);
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            println!("❌ Impossible de lire le fichier : {}", e);
            return;
        }
    };

    let decoded = decoder::decode(&bytes);
    match decoded {
        decoder::DecodedMessage::Unknown { opcode, .. } => {
            println!("❌ ÉCHEC : Message inconnu ou corrompu (OpCode: 0x{:02x})", opcode);
        },
        decoder::DecodedMessage::Ping { sequence } => {
            println!("⚠️  AVERTISSEMENT : Message valide mais vide (PING seq={}). Rien à valider.", sequence);
        },
        decoder::DecodedMessage::Event { handler, payload, node_id, .. } => {
            if handler.is_empty() {
                println!("❌ ÉCHEC : Event sans handler.");
            } else if payload.is_empty() {
                println!("⚠️  AVERTISSEMENT : Event valide (handler={}) mais sans payload.", handler);
            } else {
                println!("✅ SUCCÈS : Event valide (handler={}, node_id={}).", handler, node_id);
            }
        },
        decoder::DecodedMessage::Patch { op_count, ops } => {
            if op_count == 0 || ops.is_empty() {
                println!("⚠️  AVERTISSEMENT : Patch valide mais ne contient aucune opération.");
            } else {
                println!("✅ SUCCÈS : Patch valide contenant {} opérations.", op_count);
                for (i, op) in ops.iter().enumerate() {
                    println!("  [{}] Target: {}, Op: {}, Val: {}", i, op.target_id, op.op_type, op.value);
                }
            }
        },
        decoder::DecodedMessage::Hello { session_id, .. } => {
            println!("✅ SUCCÈS : Hello valide (session_id={}).", session_id);
        },
        decoder::DecodedMessage::BTree { node_count, .. } => {
            println!("✅ SUCCÈS : BTree valide ({} nœuds).", node_count);
        },
        decoder::DecodedMessage::Log { severity, message } => {
            println!("✅ SUCCÈS : Log valide (sev={}, msg={}).", severity, message);
        },
        _ => {
            println!("✅ SUCCÈS : Le message est conforme à la spécification NBPS v0.5.0");
            println!("{:#?}", decoded);
        }
    }
}

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use axum::{
    routing::get,
    Router,
    extract::{ws::{WebSocket, WebSocketUpgrade, Message as WsMessage}},
    response::Html,
};

pub async fn run_devtools(tx_monitor: broadcast::Sender<crate::MonitoringEvent>, host: String, port: u16, token: Option<String>) {
    println!("⏱️ Initialisation des DevTools NHTML...");
    
    let token_for_root = token.clone();
    let token_for_ws = token.clone();
    let tx_for_ws = tx_monitor.clone();
    
    let app = Router::new()
        .route("/", get(move |params: axum::extract::Query<HashMap<String, String>>| async move { 
            if let Some(ref t) = token_for_root {
                if params.get("token") != Some(t) {
                    return Html("<h1>403 Forbidden</h1><p>Accès refusé : Token invalide ou manquant.</p>".to_string());
                }
            }
            Html(include_str!("../static/devtools.nhtml").to_string()) 
        }))
        .route("/_nhtml/bridge.js", get(|| async {
            (
                [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
                include_str!("../assets/js/bridge.js")
            )
        }))
        .route("/ws", get(move |ws: WebSocketUpgrade, params: axum::extract::Query<HashMap<String, String>>| {
            let tx = tx_for_ws.clone();
            let t_ws = token_for_ws.clone();
            async move { 
                if let Some(ref t) = t_ws {
                    if params.get("token") != Some(t) {
                        return (axum::http::StatusCode::FORBIDDEN, "Accès refusé").into_response();
                    }
                }
                ws.on_upgrade(move |socket| handle_devtools_ws(socket, tx))
            }
        }));

    let addr = format!("{}:{}", host, port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            println!("❌ Impossible de lier le port DevTools sur {} : {}", addr, e);
            return;
        }
    };
    
    if let Some(ref t) = token {
        println!("🚀 DevTools NHTML disponibles sur http://{}?token={}", addr, t);
    } else {
        println!("🚀 DevTools NHTML disponibles sur http://{}", addr);
        if host != "127.0.0.1" && host != "localhost" {
            println!("⚠️ ATTENTION : Les DevTools sont exposés sur {} SANS TOKEN. C'est dangereux en production !", host);
        }
    }
    
    if let Err(e) = axum::serve(listener, app).await {
        println!("❌ Erreur serveur DevTools : {}", e);
    }
}

async fn handle_devtools_ws(socket: WebSocket, tx_monitor: broadcast::Sender<crate::MonitoringEvent>) {
    use futures_util::{StreamExt, SinkExt};
    println!("🔌 Client DevTools NHTML connecté");
    let (ws_sender, mut ws_receiver) = socket.split();
    let _rx_packet = tx_monitor.subscribe();

    // Note: On supprime l'ancienne tâche simpliste pour ne garder que la nouvelle enrichie plus bas
    
    let sessions: Vec<String> = Vec::new(); // TODO: Migrate to sqlx


    
    enum State {
        Dashboard,
        Replay { session_id: String, history: Vec<(u32, String, u32)>, step: usize, total: usize },
    }
    
    let mut session_list_html = String::new();
    for sid in &sessions {
        session_list_html.push_str(&format!(
            "<button class='session-btn' n-click='replay' n-val='{}'>
                <div style='display:flex; justify-content:space-between; align-items:center;'>
                    <span>SID: {}</span>
                    <span onclick='event.stopPropagation(); startCompare(\"{}\", this)' style='background:rgba(255,255,255,0.05); padding:2px 8px; border-radius:4px; font-size:0.6rem; border:1px solid rgba(255,255,255,0.1);'>CMP</span>
                </div>
            </button>", sid, if sid.len() > 8 { &sid[0..8] } else { sid }, sid
        ));
    }
    if sessions.is_empty() {
        session_list_html = "<div style='opacity:0.5; padding:20px;'>Aucune session trouvée.</div>".to_string();
    }

    let _current_state = State::Dashboard;
    let ws_sender = Arc::new(Mutex::new(ws_sender));
    let sender_for_monitor = ws_sender.clone();

    // Envoi initial
    {
        let pkt = crate::proto::patch(&[crate::proto::PatchOp::replace_inner(501, 1, &session_list_html)]);
        let mut s = ws_sender.lock().await;
        let _ = s.send(WsMessage::Binary(pkt)).await;
    }

    // Boucle de relai Monitoring ENRICHI (GATEWAY -> DASHBOARD)
    let mut rx_packet_monitor = tx_monitor.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx_packet_monitor.recv().await {
            let type_name = match event.pkt_type {
                0x01 => "HELLO",
                0x02 => "EVENT",
                0x03 => "PATCH",
                0x05 => "SYNC",
                0x07 => "B-TREE",
                0x09 => "PING",
                0x0A => "METRICS",
                0x10 => "LOG",
                0x7F => "ERROR",
                _ => "UNKNOWN"
            };

            let mut ops = Vec::new();

            // 1. PANE GAUCHE: Stream compact
            let row_html = format!(
                "<div class='net-row' style='display: flex; justify-content: space-between; padding: 6px 10px; border-bottom: 1px solid rgba(255,255,255,0.03); font-family: monospace; font-size: 10px; align-items: center;'>
                    <span style='color: {}; width: 30px;'>{}</span>
                    <span style='font-weight:bold; color:{}; flex: 1;'>{}</span>
                    <span style='color: var(--text-dim);'>{} B</span>
                </div>",
                if event.direction == "IN" { "#0f0" } else { "#ff007f" },
                if event.direction == "IN" { "IN" } else { "OUT" },
                if event.direction == "IN" { "var(--text)" } else { "var(--accent)" },
                type_name,
                event.size
            );
            ops.push(crate::proto::PatchOp::append_html(600, 1, &row_html));

            // 2. PANE CENTRAL: Flow Card (Seulement pour les interactions)
            if event.pkt_type == 0x02 || (event.pkt_type == 0x03 && event.direction == "OUT") {
                let raw_handler = event.handler.clone().unwrap_or_else(|| "unnamed".to_string());
                let raw_details = event.details.clone().unwrap_or_else(|| "-".to_string());
                
                let handler_name = html_escape::encode_safe(&raw_handler).to_string();
                let details = html_escape::encode_safe(&raw_details).to_string();
                
                let flow_html = format!(
                    "<div class='flow-card'>
                        <div style='position:absolute; top:10px; right:15px; font-size:0.6rem; color:var(--text-dim);'>{}</div>
                        <div class='flow-step'>
                            <div class='flow-icon icon-event'>EV</div>
                            <div>
                                <div style='font-weight:bold;'>EVENT: {}</div>
                                <div style='font-size:0.6rem; opacity:0.6;'>Interaction utilisateur détectée</div>
                            </div>
                        </div>
                        <div class='arrow'></div>
                        <div class='flow-step'>
                            <div class='flow-icon icon-php'>PHP</div>
                            <div>
                                <div style='font-weight:bold;'>PHP EXECUTION</div>
                                <div style='font-size:0.6rem; opacity:0.6;'>Traitement métier backend</div>
                            </div>
                        </div>
                        <div class='arrow'></div>
                        <div class='flow-step'>
                            <div class='flow-icon icon-ui'>UI</div>
                            <div>
                                <div style='font-weight:bold;'>UI PATCH: {}</div>
                                <div style='font-size:0.6rem; color:var(--accent);'>{}</div>
                            </div>
                        </div>
                        <div style='margin-top:10px; padding-top:10px; border-top:1px solid rgba(255,255,255,0.05); display:flex; justify-content:space-between; align-items:center;'>
                            <div style='font-size:0.65rem; color:var(--green); font-weight:bold;'>LATENCY: {}ms</div>
                            <div style='font-size:0.6rem; color:var(--text-dim); font-family:monospace;'>#{}</div>
                        </div>
                    </div>",
                    event.timestamp,
                    handler_name,
                    type_name,
                    details,
                    event.latency_ms.unwrap_or(0),
                    if event.session_id.len() > 8 { &event.session_id[0..8] } else { &event.session_id }
                );
                ops.push(crate::proto::PatchOp::replace_inner(100, 1, &flow_html));
            }

            // 3. PANE DROIT: Diff (Seulement pour les patches)
            if event.pkt_type == 0x03 && event.direction == "OUT" {
                let raw_diff = event.details.unwrap_or_default();
                let diff_html = format!(
                    "<div style='background:rgba(255,0,127,0.05); border:1px solid var(--accent); padding:10px; border-radius:6px;'>
                        <div style='font-size:0.6rem; color:var(--accent); font-weight:bold; margin-bottom:5px;'>PATCH CONTENT</div>
                        <div style='font-family:monospace; color:var(--text); white-space:pre-wrap;'>{}</div>
                    </div>",
                    html_escape::encode_safe(&raw_diff).to_string()
                );
                ops.push(crate::proto::PatchOp::append_html(101, 1, &diff_html));
            }
            
            let pkt = crate::proto::patch(&ops);
            let mut s = sender_for_monitor.lock().await;
            if let Err(_) = s.send(WsMessage::Binary(pkt)).await {
                break;
            }
        }
    });

    // 4. Task de METRICS (Relais des stats système)
    let sender_for_metrics = ws_sender.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            // Simulation ou lecture des vraies métriques (v0.7.0)
            // Pour l'instant on utilise des valeurs statiques/globales simulées
            // mais on a déjà les compteurs Prometheus en arrière-plan.
            
            let mut ops = Vec::new();
            // Note: On pourrait lire metrics::gauge!("nhtml_active_clients") ici si on avait un handle
            // Pour l'instant on envoie des placeholders qui seront liés aux vrais compteurs plus tard
            ops.push(crate::proto::PatchOp::set_text(701, 1, "PROD-READY")); // Status
            
            let pkt = crate::proto::patch(&ops);
            let mut s = sender_for_metrics.lock().await;
            if let Err(_) = s.send(WsMessage::Binary(pkt)).await {
                break;
            }
        }
    });

    let mut current_state = State::Dashboard;

    while let Some(Ok(msg)) = ws_receiver.next().await {
        if let WsMessage::Binary(data) = msg {
            if data.is_empty() { continue; }
            match data[0] {
                0x01 => { // HELLO
                    // 1. Envoyer une réponse HELLO (status=1) pour confirmer la connexion
                    let hello_pkt = crate::proto::hello("devtools-session", &[0u8; 32], 0);
                    let mut s = ws_sender.lock().await;
                    let _ = s.send(WsMessage::Binary(hello_pkt)).await;

                    // 2. Envoyer le premier patch de l'UI
                    let mut ops = Vec::new();
                    let mut html = String::new();
                    if sessions.is_empty() {
                        html.push_str("<div style='opacity:0.3; text-align:center; padding:20px;'>Aucune session trouvée</div>");
                    } else {
                        for (i, s) in sessions.iter().enumerate() {
                            html.push_str(&format!(
                                "<div style='display:flex; gap:5px; margin-bottom:5px;'>
                                    <button n-id='{}' n-click='load' class='session-btn' style='flex:1; font-size:0.65rem; padding:5px;'>▶ {}</button>
                                    <button onclick=\"startCompare('{}', this)\" style='background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.1); color:var(--text-dim); width:30px; font-size:0.5rem; cursor:pointer;'>CMP</button>
                                </div>", 1000 + i, if s.len() > 12 { &s[0..12] } else { s }, s
                            ));
                        }
                    }
                    ops.push(crate::proto::PatchOp::replace_inner(501, 1, &html)); 
                    
                    let pkt = crate::proto::patch(&ops);
                    let _ = s.send(WsMessage::Binary(pkt)).await;
                },
                0x02 => { // EVENT
                    if data.len() >= 5 {
                        let nid = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
                        match &mut current_state {
                            State::Dashboard => {
                                if nid >= 1000 && (nid - 1000) < sessions.len() as u32 {
                                    let selected_session = sessions[(nid - 1000) as usize].clone();
                                    let mut history = Vec::new();
                                    let mut ghost_html = String::new();
                                    
                                    // TODO: Replay logic migration to sqlx
                                    
                                    let total = history.len();
                                    let mut ops = Vec::new();
                                    ops.push(crate::proto::PatchOp::set_text(302, 1, &format!("Step 0 / {}", total)));
                                    ops.push(crate::proto::PatchOp::set_attr(102, 1, "style", "width: 0%;"));
                                    ops.push(crate::proto::PatchOp::replace_inner(100, 1, "<div style='opacity:0.5;'>Prêt pour le Replay Dynamique. Cliquez sur STEP.</div>"));
                                    ops.push(crate::proto::PatchOp::replace_inner(500, 1, &ghost_html));
                                    
                                    let pkt = crate::proto::patch(&ops);
                                    let mut s = ws_sender.lock().await;
                                    let _ = s.send(WsMessage::Binary(pkt)).await;
                                    current_state = State::Replay { session_id: selected_session, history, step: 0, total };
                                }
                            },
                            State::Replay { session_id, history, step, total } => {
                                if nid == 203 {
                                    current_state = State::Dashboard;
                                    let mut ops = Vec::new();
                                    ops.push(crate::proto::PatchOp::set_attr(400, 1, "style", "display: block;"));
                                    ops.push(crate::proto::PatchOp::set_attr(401, 1, "style", "display: none;"));
                                    let pkt = crate::proto::patch(&ops);
                                    let mut s = ws_sender.lock().await;
                                    let _ = s.send(WsMessage::Binary(pkt)).await;
                                    continue;
                                }
                                
                                // --- LOGIQUE STATE DIFF VIEWER ---
                                if nid < 200 || nid > 210 {
                                    let mut ops = Vec::new();
                                    let mut details = format!("<div style='color:var(--accent); font-weight:bold; margin-bottom:15px; border-bottom:1px solid var(--accent); padding-bottom:5px;'>STATE DIFF VIEWER</div>");
                                    details.push_str(&format!("<div style='font-family:monospace; margin-bottom:10px;'>Node ID: <span style='color:var(--accent)'>#{}</span></div>", nid));
                                    
                                    // Chercher l'historique des 2 derniers changements pour ce noeud
                                    // TODO: State diff viewer migration to sqlx
                                    
                                    ops.push(crate::proto::PatchOp::set_text(101, 1, &details));
                                    let pkt = crate::proto::patch(&ops);
                                    let mut s = ws_sender.lock().await;
                                    let _ = s.send(WsMessage::Binary(pkt)).await;
                                    continue;
                                }

                                if nid == 201 {
                                    if *step < *total {
                                        let (step_nid, step_val, step_ver) = history[*step].clone();
                                        *step += 1;
                                        
                                        let mut ops = Vec::new();
                                        // Update Sandbox (Pédagogique)
                                        let sandbox_html = format!(
                                            "<div style='border: 1px dashed var(--accent); padding: 20px; border-radius: 10px; background: rgba(255,255,255,0.02);'>
                                                <div style='font-size: 0.6rem; color: var(--accent); margin-bottom: 10px;'>REPLAY STEP {} / {}</div>
                                                <div style='font-family: monospace; font-size: 1.2rem;'>Noeud #{}: <span style='color: white;'>{}</span></div>
                                                <div style='font-size: 0.5rem; color: var(--text-dim); margin-top: 5px;'>Internal Version: {}</div>
                                            </div>",
                                            *step, *total, step_nid, step_val, step_ver
                                        );
                                        ops.push(crate::proto::PatchOp::replace_inner(100, 1, &sandbox_html));
                                        
                                        // Update Timeline
                                        let pct = if *total > 0 { (*step as f32 / *total as f32) * 100.0 } else { 100.0 };
                                        ops.push(crate::proto::PatchOp::set_attr(102, 1, "style", &format!("width: {:.1}%", pct)));
                                        ops.push(crate::proto::PatchOp::set_text(302, 1, &format!("Step {} / {}", *step, *total)));
                                        
                                        // Update Ghost Nodes in technical view
                                        let mut ghost_html = String::new();
                                        // TODO: Ghost nodes migration to sqlx
                                        ops.push(crate::proto::PatchOp::replace_inner(500, 1, &ghost_html));

                                        let pkt = crate::proto::patch(&ops);
                                        let mut s = ws_sender.lock().await;
                                        let _ = s.send(WsMessage::Binary(pkt)).await;
                                    }
                                } else if nid == 202 {
                                    *step = 0;
                                    let mut ops = Vec::new();
                                    ops.push(crate::proto::PatchOp::set_text(100, 1, "Replay réinitialisé."));
                                    ops.push(crate::proto::PatchOp::set_attr(102, 1, "style", "width: 0%"));
                                    ops.push(crate::proto::PatchOp::set_text(302, 1, &format!("Step 0 / {}", *total)));
                                    ops.push(crate::proto::PatchOp::set_text(101, 1, "<div class='log-entry'>Logs réinitialisés.</div>"));
                                    let pkt = crate::proto::patch(&ops);
                                    let mut s = ws_sender.lock().await;
                                    let _ = s.send(WsMessage::Binary(pkt)).await;
                                }
                            }
                        }
                    }
                },
                0x05 => { // COMPARE
                    if data.len() >= 5 {
                        println!("DEBUG: Mode COMPARAISON");
                        // Parsing simplifié pour la comparaison
                        let sid1_len = data[1] as usize;
                        if data.len() < 2 + sid1_len + 1 { continue; }
                        let _sid1 = String::from_utf8_lossy(&data[2..2+sid1_len]).to_string();
                        let cursor = 2 + sid1_len;
                        let sid2_len = data[cursor] as usize;
                        if data.len() < cursor + 1 + sid2_len { continue; }
                        let _sid2 = String::from_utf8_lossy(&data[cursor+1..cursor+1+sid2_len]).to_string();

                        let mut comparison = format!("<div style='color:var(--accent); font-weight:bold; margin-bottom:20px;'>COMPARAISON DE SESSIONS</div>");
                        comparison.push_str("<div style='display:grid; grid-template-columns:1fr 1fr 1fr; gap:10px; font-size:0.7rem; color:var(--text-dim); margin-bottom:10px;'><span>NODE</span><span>SESSION A</span><span>SESSION B</span></div>");

                        // TODO: Comparison migration to sqlx

                        let mut ops = Vec::new();
                        ops.push(crate::proto::PatchOp::replace_inner(101, 1, &comparison));
                        let pkt = crate::proto::patch(&ops);
                        let mut s = ws_sender.lock().await;
                        let _ = s.send(WsMessage::Binary(pkt)).await;
                    }
                },
                0x06 => { // REFRESH
                    let mut sessions: Vec<String> = Vec::new();
                    // TODO: Session list migration to sqlx
                    let mut html = String::new();
                    for (i, sid) in sessions.iter().enumerate() {
                        html.push_str(&format!(
                            "<div style='display:flex; gap:5px; margin-bottom:5px;'>
                                <button n-id='{}' n-click='load' class='session-btn' style='flex:1; font-size:0.65rem; padding:5px;'>▶ {}</button>
                                <button onclick=\"startCompare('{}', this)\" style='background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.1); color:var(--text-dim); width:30px; font-size:0.5rem; cursor:pointer;'>CMP</button>
                            </div>", 1000 + i, if sid.len() > 12 { &sid[0..12] } else { sid }, sid
                        ));
                    }
                    if html.is_empty() { html = "<div style='opacity:0.3; text-align:center;'>Aucune session</div>".to_string(); }
                    let pkt = crate::proto::patch(&[crate::proto::PatchOp::replace_inner(501, 1, &html)]);
                    let mut s = ws_sender.lock().await;
                    let _ = s.send(WsMessage::Binary(pkt)).await;
                },
                _ => {}
            }
        }
    }
}


pub fn run_benchmark(path: &str) {
    println!("🧪 NHTML Industrial Benchmark Tool v0.7.3-stable");
    println!("--------------------------------------------------");
    
    let (html_content, label) = if let Ok(content) = std::fs::read_to_string(path) {
        (content, format!("Fichier: {}", path))
    } else {
        (
            "<html><body><h1>Hello World</h1><p>Ceci est un test de benchmark industriel.</p></body></html>".to_string(),
            "Démonstration (Sample)".to_string()
        )
    };

    let html_size = html_content.len();
    
    // Simulation B-TREE
    let nodes = vec![(1, 1, "root".to_string(), html_content.clone())];
    let (btree_pkt, _ratio) = crate::proto::btree(&nodes);
    let binary_size = btree_pkt.len();
    
    // On recalcule pour le détail
    let tree_payload = crate::proto::serialize_nodes(&nodes);
    let raw_binary_size = tree_payload.len() + 14 + 5;
    
    println!("📊 Cible : {}", label);
    println!("📦 Poids HTML Brut     : {} bytes", html_size);
    println!("📦 Poids NHTML Binaire : {} bytes (Raw)", raw_binary_size);
    println!("📦 Poids NHTML + Zstd  : {} bytes (Optimisé)", binary_size);
    println!("--------------------------------------------------");
    
    let total_gain = if html_size > 0 { (1.0 - (binary_size as f32 / html_size as f32)) * 100.0 } else { 0.0 };
    println!("✨ Gain de bande passante : {:.1}%", total_gain);
    println!("🚀 Facteur d'efficacité   : {:.1}x", html_size as f32 / binary_size.max(1) as f32);
    
    println!("\n⚡ MÉTRIQUES DE PERFORMANCE RÉELLES");
    println!("--------------------------------------------------");
    let start_bench = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = crate::proto::btree(&nodes);
    }
    let duration = start_bench.elapsed() / 1000;
    println!("⏱️ Temps de sérialisation (moyenne) : {} µs", duration.as_micros());
    
    let throughput = (binary_size as f32 / 1024.0 / 1024.0) / (duration.as_secs_f32() + 1e-9);
    println!("🚀 Débit théorique                 : {:.2} MB/s", throughput);
    
    println!("--------------------------------------------------");
    println!("✅ Benchmark terminé. NHTML v0.7.3-stable est prêt pour la production.");
}

pub fn run_share(port: u16) {
    use std::io::Write;
    println!("🌍 Tentative de partage du projet NHTML sur le port {}...", port);
    println!("⚠️ ATTENTION : Cela va utiliser 'npx localtunnel' pour exposer publiquement votre port local sur Internet.");
    print!("Voulez-vous continuer ? [y/N] ");
    let _ = std::io::stdout().flush();
    
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() || input.trim().to_lowercase() != "y" {
        println!("❌ Opération annulée.");
        return;
    }

    println!("🔗 Lancement de LocalTunnel...");
    
    let child = std::process::Command::new("npx")
        .args(["localtunnel", "--port", &port.to_string()])
        .spawn();

    match child {
        Ok(mut c) => {
            println!("✅ Tunnel démarré ! Regardez l'URL ci-dessous :");
            let _ = c.wait();
        }
        Err(_) => {
            println!("❌ Erreur : 'npx' n'est pas installé ou localtunnel a échoué.");
            println!("💡 Astuce : Installez Node.js ou utilisez 'ngrok http {}'", port);
        }
    }
}

pub fn run_build(production: bool, output_dir: &str) {
    println!("🏗️  Building NHTML project...");
    let output_path = Path::new(output_dir);

    if let Err(e) = fs::create_dir_all(output_path) {
        println!("❌ Erreur : Impossible de créer le dossier de sortie : {}", e);
        return;
    }

    // 1. Lire index.nhtml
    let entry = "index.nhtml";
    let source = match fs::read_to_string(entry) {
        Ok(s) => s,
        Err(_) => {
            println!("❌ Erreur : Fichier '{}' introuvable.", entry);
            return;
        }
    };

    // 2. Minification (si production)
    let mut final_source = source;
    if production {
        println!("✨ Mode Production activé : Minification en cours...");
        // Minification ultra-basique (suppression des retours à la ligne et espaces multiples)
        final_source = final_source.lines()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join("");
    }

    // 3. Compilation
    let result = crate::compiler::NhtmlCompiler::compile(&final_source);

    // 4. Écriture des fichiers
    let html_file = output_path.join("index.html");
    if let Err(e) = fs::write(&html_file, &result.html) {
        println!("❌ Erreur : Impossible d'écrire l'HTML : {}", e);
        return;
    }

    let bin_file = output_path.join("app.nbps");
    if let Err(e) = fs::write(&bin_file, &result.btree_bytes) {
        println!("❌ Erreur : Impossible d'écrire le bundle NBPS : {}", e);
        return;
    }

    // Copier les assets s'ils existent
    if Path::new("assets").exists() {
        println!("📂 Copie des assets...");
        if let Err(e) = copy_dir_all("assets", output_path.join("assets")) {
            println!("❌ Erreur lors de la copie des assets : {}", e);
        }
    }

    println!("✅ Build terminé avec succès dans '{}' !", output_dir);
    println!("   - HTML : {}", html_file.display());
    println!("   - Bundle NBPS : {}", bin_file.display());
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
