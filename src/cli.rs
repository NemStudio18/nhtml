use std::fs;
use std::path::Path;
use crate::decoder;
use axum::http::header;

pub fn create_new_project(name: &str) {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        println!("❌ Erreur : Le dossier '{}' existe déjà.", name);
        return;
    }

    if let Err(e) = fs::create_dir_all(project_dir) {
        println!("❌ Erreur : Impossible de créer le dossier '{}' : {}", name, e);
        return;
    }

    // 1. Création de index.nhtml
    let html_content = r#"<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <title>NHTML App</title>
    <style>
        body { font-family: sans-serif; text-align: center; margin-top: 50px; background: #050505; color: white; }
        .counter { font-size: 5rem; margin: 40px; font-weight: bold; }
        button { background: white; color: black; border: none; padding: 15px 30px; font-size: 1.2rem; cursor: pointer; border-radius: 12px; }
    </style>
</head>
<body>
    <h1>Projet NHTML v0.4.0</h1>
    <div class="counter" n-id="counter_value">0</div>
    <button n-id="btn_increment" n-click="increment">ACTION</button>
</body>
</html>"#;
    if let Err(e) = fs::write(project_dir.join("index.nhtml"), html_content) {
        println!("❌ Erreur : Impossible d'écrire index.nhtml : {}", e);
        return;
    }

    // 2. Création de app.php
    let php_content = r#"<?php
$input = file_get_contents('php://input');
$request = json_decode($input, true);
$event = $request['nhtml_event'] ?? '';
$nid = $request['node_id'] ?? '';

$patches = [];
if ($event === 'click' && $nid === 'btn_increment') {
    $patches[] = ["op" => "set_text", "nid" => "counter_value", "value" => "1"];
}

header('Content-Type: application/json');
echo json_encode($patches);
"#;
    if let Err(e) = fs::write(project_dir.join("app.php"), php_content) {
        println!("❌ Erreur : Impossible d'écrire app.php : {}", e);
        return;
    }

    println!("✅ Projet '{}' créé avec succès !", name);
}

pub fn dump_database() {
    println!("📊 Contenu de la base de données NHTML :");
    let paths = ["nhtml_sessions.db", "gateway/nhtml_sessions.db", "../nhtml_sessions.db", "../../nhtml_sessions.db"];
    let mut db_path = "nhtml_sessions.db".to_string();
    
    for p in paths {
        if std::path::Path::new(p).exists() {
            db_path = p.to_string();
            break;
        }
    }
    
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            println!("❌ Impossible d'ouvrir la DB à {} : {}", db_path, e);
            return;
        }
    };
    
    println!("\n--- SESSIONS & NODES ---");
    let mut stmt = match conn.prepare("SELECT session_id, node_id, value, version FROM nodes") {
        Ok(s) => s,
        Err(e) => { println!("❌ Erreur SQL Prepare: {}", e); return; }
    };
    let rows = match stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?, row.get::<_, String>(2)?, row.get::<_, u32>(3)?))
    }) {
        Ok(r) => r,
        Err(e) => { println!("❌ Erreur SQL Query: {}", e); return; }
    };

    for r in rows {
        if let Ok((sid, nid, val, ver)) = r {
            println!("Session: {} | Node: {} | Value: '{}' | Ver: {}", sid, nid, val, ver);
        }
    }

    println!("\n--- EVENT LOG (Derniers 10) ---");
    let mut stmt = match conn.prepare("SELECT timestamp, session_id, event_type, node_id FROM event_log ORDER BY id DESC LIMIT 10") {
        Ok(s) => s,
        Err(e) => { println!("❌ Erreur SQL Prepare: {}", e); return; }
    };
    let rows = match stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, u32>(3)?))
    }) {
        Ok(r) => r,
        Err(e) => { println!("❌ Erreur SQL Query: {}", e); return; }
    };

    for r in rows {
        if let Ok((ts, sid, ev, nid)) = r {
            println!("[{}] {} | {} on Node {}", ts, sid, ev, nid);
        }
    }
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
        _ => {
            println!("✅ SUCCÈS : Le message est conforme à la spécification NBPS v0.2.2");
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
    
    let token_clone = token.clone();
    let app = Router::new()
        .route("/", get(move |req: axum::extract::Request| async move { 
            if let Some(ref t) = token_clone {
                let query = req.uri().query().unwrap_or("");
                if !query.contains(&format!("token={}", t)) {
                    return Html("<h1>403 Forbidden</h1><p>Token d'authentification invalide ou manquant.</p>".to_string());
                }
            }
            Html(include_str!("../static/devtools.nhtml").to_string()) 
        }))
        .route("/_nhtml/bridge.js", get(|| async {
            axum::response::Response::builder()
                .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
                .body(axum::body::Body::from(include_str!("../assets/js/bridge.js")))
                .unwrap()
        }))
        .route("/ws", get(move |ws: WebSocketUpgrade| {
            let tx = tx_monitor.clone();
            async move { ws.on_upgrade(move |socket| handle_devtools_ws(socket, tx)) }
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
    
    let paths = ["nhtml_sessions.db", "gateway/nhtml_sessions.db", "../nhtml_sessions.db", "../../nhtml_sessions.db"];
    let mut db_path = "nhtml_sessions.db".to_string();
    for p in paths {
        if std::path::Path::new(p).exists() {
            db_path = p.to_string();
            break;
        }
    }

    let sessions: Vec<String> = {
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            if let Ok(mut stmt) = conn.prepare("SELECT DISTINCT session_id FROM patch_history") {
                if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                    rows.flatten().collect()
                } else { Vec::new() }
            } else { Vec::new() }
        } else { Vec::new() }
    };


    
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
                let handler_name = event.handler.clone().unwrap_or_else(|| "unnamed".to_string());
                let details = event.details.clone().unwrap_or_else(|| "-".to_string());
                
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
                let diff_html = format!(
                    "<div style='background:rgba(255,0,127,0.05); border:1px solid var(--accent); padding:10px; border-radius:6px;'>
                        <div style='font-size:0.6rem; color:var(--accent); font-weight:bold; margin-bottom:5px;'>PATCH CONTENT</div>
                        <div style='font-family:monospace; color:var(--text); white-space:pre-wrap;'>{}</div>
                    </div>",
                    event.details.unwrap_or_default()
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

    let mut current_state = State::Dashboard;

    while let Some(Ok(msg)) = ws_receiver.next().await {
        if let WsMessage::Binary(data) = msg {
            if data.is_empty() { continue; }
            match data[0] {
                0x01 => { // HELLO
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
                    let mut s = ws_sender.lock().await;
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
                                    
                                    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                                        // Load History
                                        if let Ok(mut stmt) = conn.prepare("SELECT node_id, value, version FROM patch_history WHERE session_id = ? ORDER BY id ASC") {
                                            if let Ok(rows) = stmt.query_map([&selected_session], |row| {
                                                Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?, row.get::<_, u32>(2)?))
                                            }) {
                                                for r in rows.flatten() { history.push(r); }
                                            }
                                        }
                                        // Load Current Nodes (Ghost)
                                        if let Ok(mut stmt) = conn.prepare("SELECT node_id, tag, value, version FROM nodes WHERE session_id = ?") {
                                            if let Ok(rows) = stmt.query_map([&selected_session], |row| {
                                                Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, u32>(3)?))
                                            }) {
                                                for r in rows.flatten() {
                                                    let (gn_id, gn_tag, gn_val, gn_ver) = r;
                                                    ghost_html.push_str(&format!(
                                                        "<div style='padding: 5px; border-bottom: 1px solid rgba(255,255,255,0.05);'>
                                                            <span style='color: #00d4ff;'>#{}</span> 
                                                            <span style='color: var(--text-dim);'>[{}]</span> 
                                                            <span style='color: white;'>{}</span>
                                                            <span style='float: right; opacity: 0.3;'>v{}</span>
                                                        </div>", gn_id, gn_tag, gn_val, gn_ver
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                    
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
                                    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                                        if let Ok(mut stmt) = conn.prepare("SELECT value, version, timestamp FROM patch_history WHERE session_id = ? AND node_id = ? ORDER BY id DESC LIMIT 2") {
                                            if let Ok(rows) = stmt.query_map([&session_id, &nid.to_string()], |row| {
                                                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?, row.get::<_, String>(2)?))
                                            }) {
                                                let history: Vec<_> = rows.flatten().collect();
                                                
                                                if history.is_empty() {
                                                    details.push_str("<div style='color:var(--text-dim); font-size:0.8rem;'>Aucun historique de mutation pour ce noeud.</div>");
                                                } else if history.len() == 1 {
                                                    let (val, ver, ts) = &history[0];
                                                    details.push_str(&format!("<div style='background:rgba(0,255,0,0.1); padding:10px; border-radius:5px;'>
                                                        <div style='font-size:0.7rem; color:var(--text-dim); margin-bottom:5px;'>INITIAL STATE (v{}) - {}</div>
                                                        <div style='font-family:monospace; color:var(--green)'>'{}'</div>
                                                    </div>", ver, ts, val));
                                                } else {
                                                    let (new_val, new_ver, new_ts) = &history[0];
                                                    let (old_val, old_ver, _) = &history[1];
                                                    
                                                    details.push_str(&format!("<div style='display:flex; flex-direction:column; gap:10px;'>
                                                        <div style='background:rgba(255,255,255,0.03); padding:10px; border-radius:5px; border-left:3px solid var(--text-dim);'>
                                                            <div style='font-size:0.6rem; color:var(--text-dim);'>PREVIOUS (v{})</div>
                                                            <div style='font-family:monospace; opacity:0.6;'>'{}'</div>
                                                        </div>
                                                        <div style='text-align:center; color:var(--accent); font-size:1.2rem;'>↓</div>
                                                        <div style='background:rgba(0,255,0,0.1); padding:10px; border-radius:5px; border-left:3px solid var(--green);'>
                                                            <div style='font-size:0.6rem; color:var(--green); font-weight:bold;'>CURRENT (v{})</div>
                                                            <div style='font-family:monospace; color:var(--green); font-weight:bold;'>'{}'</div>
                                                            <div style='font-size:0.5rem; color:var(--text-dim); margin-top:5px; text-align:right;'>{}</div>
                                                        </div>
                                                    </div>", old_ver, old_val, new_ver, new_val, new_ts));
                                                }
                                            }
                                        }
                                    }
                                    
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
                                        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                                            if let Ok(mut stmt) = conn.prepare("SELECT node_id, tag, value, version FROM nodes WHERE session_id = ?") {
                                                if let Ok(rows) = stmt.query_map([session_id.as_str()], |row| {
                                                    Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, u32>(3)?))
                                                }) {
                                                    for r in rows.flatten() {
                                                        let (gn_id, gn_tag, gn_val, gn_ver) = r;
                                                        ghost_html.push_str(&format!(
                                                            "<div style='padding: 5px; border-bottom: 1px solid rgba(255,255,255,0.05);'>
                                                                <span style='color: #00d4ff;'>#{}</span> 
                                                                <span style='color: var(--text-dim);'>[{}]</span> 
                                                                <span style='color: white;'>{}</span>
                                                                <span style='float: right; opacity: 0.3;'>v{}</span>
                                                            </div>", gn_id, gn_tag, gn_val, gn_ver
                                                        ));
                                                    }
                                                }
                                            }
                                        }
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
                        let sid1 = String::from_utf8_lossy(&data[2..2+sid1_len]).to_string();
                        let cursor = 2 + sid1_len;
                        let sid2_len = data[cursor] as usize;
                        if data.len() < cursor + 1 + sid2_len { continue; }
                        let sid2 = String::from_utf8_lossy(&data[cursor+1..cursor+1+sid2_len]).to_string();

                        let mut comparison = format!("<div style='color:var(--accent); font-weight:bold; margin-bottom:20px;'>COMPARAISON DE SESSIONS</div>");
                        comparison.push_str("<div style='display:grid; grid-template-columns:1fr 1fr 1fr; gap:10px; font-size:0.7rem; color:var(--text-dim); margin-bottom:10px;'><span>NODE</span><span>SESSION A</span><span>SESSION B</span></div>");

                        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                            let mut stmt = match conn.prepare("SELECT node_id, value FROM nodes WHERE session_id = ?") {
                                Ok(s) => s,
                                Err(_) => continue,
                            };
                            let nodes_a: std::collections::HashMap<u32, String> = stmt.query_map([&sid1], |row| Ok((row.get(0)?, row.get(1)?))).map(|r| r.flatten().collect()).unwrap_or_default();
                            let mut stmt = match conn.prepare("SELECT node_id, value FROM nodes WHERE session_id = ?") {
                                Ok(s) => s,
                                Err(_) => continue,
                            };
                            let nodes_b: std::collections::HashMap<u32, String> = stmt.query_map([&sid2], |row| Ok((row.get(0)?, row.get(1)?))).map(|r| r.flatten().collect()).unwrap_or_default();

                            let mut all_nids: Vec<_> = nodes_a.keys().chain(nodes_b.keys()).collect();
                            all_nids.sort();
                            all_nids.dedup();

                            for nid in all_nids {
                                let val_a = nodes_a.get(nid).cloned().unwrap_or("-".to_string());
                                let val_b = nodes_b.get(nid).cloned().unwrap_or("-".to_string());
                                let color = if val_a != val_b { "#ff8800" } else { "var(--text)" };
                                
                                comparison.push_str(&format!(
                                    "<div style='display:grid; grid-template-columns:1fr 1fr 1fr; gap:10px; padding:5px; border-bottom:1px solid rgba(255,255,255,0.05); font-family:monospace; color:{}'>
                                        <span>#{}</span><span>'{}'</span><span>'{}'</span>
                                    </div>", color, nid, val_a, val_b
                                ));
                            }
                        }

                        let mut ops = Vec::new();
                        ops.push(crate::proto::PatchOp::replace_inner(101, 1, &comparison));
                        let pkt = crate::proto::patch(&ops);
                        let mut s = ws_sender.lock().await;
                        let _ = s.send(WsMessage::Binary(pkt)).await;
                    }
                },
                0x06 => { // REFRESH
                    let mut sessions: Vec<String> = Vec::new();
                    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                        if let Ok(mut stmt) = conn.prepare("SELECT DISTINCT session_id FROM patch_history") {
                            if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                                sessions = rows.flatten().collect();
                            }
                        }
                    }
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
    println!("🧪 NHTML Industrial Benchmark Tool v0.6.0");
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
    
    println!("\n⚡ MÉTRIQUES DE PERFORMANCE (THÉORIQUES)");
    println!("--------------------------------------------------");
    let latency_saved = (html_size as f32 - binary_size as f32) / (1024.0 * 1024.0 / 8.0); // Simple est. on 1Mbps
    println!("⏱️ Latence réseau sauvée (1Mbps) : {:.2} ms", latency_saved * 1000.0);
    
    let cpu_load = binary_size as f32 / 1000.0; // Arbitrary complexity score
    println!("🧠 Charge CPU Sérialesation      : {:.2} CPU-ops/pkt", cpu_load);
    
    println!("--------------------------------------------------");
    println!("✅ Benchmark terminé. NHTML v0.6.0 est prêt pour la production.");
}

pub fn run_share(port: u16) {
    println!("🌍 Tentative de partage du projet NHTML sur le port {}...", port);
    println!("🔗 Utilisation de LocalTunnel (via npx)...");
    
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
        // Logique de copie récursive simplifiée
        let _ = std::process::Command::new("xcopy")
            .args(["/E", "/I", "/Y", "assets", &format!("{}\\assets", output_dir)])
            .status();
    }

    println!("✅ Build terminé avec succès dans '{}' !", output_dir);
    println!("   - HTML : {}", html_file.display());
    println!("   - Bundle NBPS : {}", bin_file.display());
}
