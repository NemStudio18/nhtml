use std::fs;
use std::path::Path;
use crate::decoder;

pub fn create_new_project(name: &str) {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        println!("❌ Erreur : Le dossier '{}' existe déjà.", name);
        return;
    }

    fs::create_dir_all(project_dir).expect("Impossible de créer le dossier du projet");

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
    fs::write(project_dir.join("index.nhtml"), html_content).unwrap();

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
    fs::write(project_dir.join("app.php"), php_content).unwrap();

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
    let mut stmt = conn.prepare("SELECT session_id, node_id, value, version FROM nodes").expect("Erreur SELECT");
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?, row.get::<_, String>(2)?, row.get::<_, u32>(3)?))
    }).expect("Erreur query");

    for r in rows {
        let (sid, nid, val, ver) = r.unwrap();
        println!("Session: {} | Node: {} | Value: '{}' | Ver: {}", sid, nid, val, ver);
    }

    println!("\n--- EVENT LOG (Derniers 10) ---");
    let mut stmt = conn.prepare("SELECT timestamp, session_id, event_type, node_id FROM event_log ORDER BY id DESC LIMIT 10").expect("Erreur SELECT");
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, u32>(3)?))
    }).expect("Erreur query");

    for r in rows {
        let (ts, sid, ev, nid) = r.unwrap();
        println!("[{}] {} | {} on Node {}", ts, sid, ev, nid);
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
        .route("/ws", get(move |ws: WebSocketUpgrade| {
            let tx = tx_monitor.clone();
            async move { ws.on_upgrade(move |socket| handle_devtools_ws(socket, tx)) }
        }));

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await
        .unwrap_or_else(|_| panic!("Impossible de lier le port DevTools sur {}", addr));
    
    if let Some(ref t) = token {
        println!("🚀 DevTools NHTML disponibles sur http://{}?token={}", addr, t);
    } else {
        println!("🚀 DevTools NHTML disponibles sur http://{}", addr);
        if host != "127.0.0.1" && host != "localhost" {
            println!("⚠️ ATTENTION : Les DevTools sont exposés sur {} SANS TOKEN. C'est dangereux en production !", host);
        }
    }
    
    axum::serve(listener, app).await.unwrap();
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
        let pkt = crate::proto::patch(&[crate::proto::PatchOp::replace_inner(500, 1, &session_list_html)]);
        let mut s = ws_sender.lock().await;
        let _ = s.send(WsMessage::Binary(pkt)).await;
    }

    // Boucle de relai Monitoring ENRICHI (GATEWAY -> DASHBOARD)
    let mut rx_packet_monitor = tx_monitor.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx_packet_monitor.recv().await {
            // Pour enrichir l'info, on peut décoder un échantillon si c'est IN ou OUT
            // On va essayer de reconstruire un paquet binaire minimal pour le décodeur si besoin
            // MAIS : le MonitoringEvent n'a pas les DATA brutes pour l'instant.
            // On va devoir modifier MonitoringEvent pour inclure un extrait ou le type décodé.
            
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

            let ratio_html = if let Some(r) = event.compression_ratio {
                if r < 1.0 {
                    let gain = (1.0 - r) * 100.0;
                    format!("<span style='color: #0f0; font-size: 0.65rem; background: rgba(0,255,0,0.1); padding: 1px 4px; border-radius: 3px;'>-{:.1}%</span>", gain)
                } else { "".to_string() }
            } else { "".to_string() };

            let latency_html = if let Some(lat) = event.latency_ms {
                let color = if lat < 20 { "#00ff88" } else if lat < 50 { "var(--green)" } else if lat < 100 { "#ff8800" } else { "#ff0000" };
                let speed_label = if lat < 30 { "⚡ ULTRA" } else if lat < 100 { "🚀 FAST" } else { "🐢 SLOW" };
                format!("<span style='color: {}; font-size: 0.65rem; border: 1px solid {}; padding: 1px 4px; border-radius: 3px; font-weight: bold;'>{} {}ms</span>", color, color, speed_label, lat)
            } else { "".to_string() };

            let handler_html = if let Some(h) = event.handler {
                format!("<span style='background: rgba(255,255,255,0.1); padding: 2px 5px; border-radius: 3px; font-size: 0.6rem; margin-right: 10px;'>{}</span>", h)
            } else { "".to_string() };

            let row_html = format!(
                "<div class='net-row' style='display: grid; grid-template-columns: 80px 100px 140px 80px 1fr; gap: 10px; padding: 8px 20px; border-bottom: 1px solid rgba(255,255,255,0.05); font-family: monospace; font-size: 11px; align-items: center;'>
                    <span style='color: {}'>{}</span>
                    <span style='font-weight:bold; color:{}'>{}</span>
                    <span>{}</span>
                    <span>{}</span>
                    <span style='color: var(--text-dim); display: flex; align-items: center;'>{} B | {}</span>
                </div>",
                if event.direction == "IN" { "#0f0" } else { "#ff007f" },
                event.direction,
                if event.direction == "IN" { "var(--text)" } else { "var(--accent)" },
                type_name,
                latency_html,
                ratio_html,
                event.size,
                handler_html
            );

            let mut ops = Vec::new();
            ops.push(crate::proto::PatchOp::append_html(600, 1, &row_html)); 
            
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
                    let mut html_list = String::new();
                    if sessions.is_empty() {
                        html_list.push_str("<div style='color:#ff5555; padding:20px;'>Aucune session n'a encore été enregistrée dans la BDD.</div>");
                    } else {
                        for (i, s) in sessions.iter().enumerate() {
                            html_list.push_str(&format!(
                                "<div style='display:flex; gap:10px; margin-bottom:10px;'>
                                    <button n-id=\"{}\" n-click=\"load\" class=\"session-btn\" style='flex:1; margin-bottom:0;'>▶ Charger : {}</button>
                                    <button onclick=\"startCompare('{}', this)\" style='background:var(--surface); border:1px solid rgba(255,255,255,0.1); width:60px; font-size:0.6rem; cursor:pointer;'>CMP</button>
                                </div>",
                                1000 + i, s, s
                            ));
                        }
                    }
                    ops.push(crate::proto::PatchOp::replace_inner(500, 1, &html_list)); 
                    
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
                                    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                                        if let Ok(mut stmt) = conn.prepare("SELECT node_id, value, version FROM patch_history WHERE session_id = ? ORDER BY id ASC") {
                                            if let Ok(rows) = stmt.query_map([&selected_session], |row| {
                                                Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?, row.get::<_, u32>(2)?))
                                            }) {
                                                for r in rows.flatten() {
                                                    history.push(r);
                                                }
                                            }
                                        }
                                    }
                                    let total = history.len();
                                    let mut ops = Vec::new();
                                    ops.push(crate::proto::PatchOp::set_attr(400, 1, "style", "display: none;"));
                                    ops.push(crate::proto::PatchOp::set_attr(401, 1, "style", "display: flex; flex-direction: column; flex: 1;"));
                                    ops.push(crate::proto::PatchOp::set_text(301, 1, &format!("SID: {}", selected_session)));
                                    ops.push(crate::proto::PatchOp::set_text(302, 1, &format!("Step 0 / {}", total)));
                                    ops.push(crate::proto::PatchOp::set_attr(102, 1, "style", "width: 0%;"));
                                    ops.push(crate::proto::PatchOp::set_text(100, 1, "En attente du flux..."));
                                    ops.push(crate::proto::PatchOp::set_text(101, 1, ""));
                                    
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
                                        ops.push(crate::proto::PatchOp::set_text(100, 1, &format!("<div style='color: var(--accent); margin-bottom: 10px;'>=> Modification du Noeud [{}]</div><div style='font-size: 1.5rem;'>{}</div><div style='color: var(--text-dim); margin-top: 10px;'>(Version interne: {})</div>", step_nid, step_val, step_ver)));
                                        let pct = if *total > 0 { (*step as f32 / *total as f32) * 100.0 } else { 100.0 };
                                        ops.push(crate::proto::PatchOp::set_attr(102, 1, "style", &format!("width: {:.1}%", pct)));
                                        ops.push(crate::proto::PatchOp::set_text(302, 1, &format!("Step {} / {}", *step, *total)));
                                        ops.push(crate::proto::PatchOp::set_text(101, 1, &format!("<div class='log-entry'>Etape {} : Noeud {} mis à jour -> '{}'</div>", *step, step_nid, step_val)));
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
                0x04 => { // REFRESH or COMPARE
                    println!("DEBUG: Commande 0x04 reçue (REFRESH/COMPARE)");
                    if data.len() < 5 {
                        // REFRESH Simple
                        let mut sessions: Vec<String> = Vec::new();
                        println!("DEBUG: Tentative lecture DB: {}", db_path);
                        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                            if let Ok(mut stmt) = conn.prepare("SELECT DISTINCT session_id FROM patch_history") {
                                if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                                    sessions = rows.flatten().collect();
                                }
                            }
                        }
                        println!("DEBUG: Sessions trouvées: {}", sessions.len());
                        let mut html = String::new();
                        for sid in &sessions {
                            html.push_str(&format!(
                                "<button class='session-btn' n-click='replay' n-val='{}'>
                                    <div style='display:flex; justify-content:space-between; align-items:center;'>
                                        <span>SID: {}</span>
                                        <span onclick='event.stopPropagation(); startCompare(\"{}\", this)' style='background:rgba(255,255,255,0.05); padding:2px 8px; border-radius:4px; font-size:0.6rem; border:1px solid rgba(255,255,255,0.1);'>CMP</span>
                                    </div>
                                </button>", sid, if sid.len() > 8 { &sid[0..8] } else { sid }, sid
                            ));
                        }
                        if html.is_empty() { html = "Aucune session.".to_string(); }
                        println!("DEBUG: Envoi PATCH ({} octets)", html.len());
                        let pkt = crate::proto::patch(&[crate::proto::PatchOp::replace_inner(500, 1, &html)]);
                        let mut s = ws_sender.lock().await;
                        let _ = s.send(WsMessage::Binary(pkt)).await;
                    } else {
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
                            let mut stmt = conn.prepare("SELECT node_id, value FROM nodes WHERE session_id = ?").expect("Err");
                            let nodes_a: std::collections::HashMap<u32, String> = stmt.query_map([&sid1], |row| Ok((row.get(0)?, row.get(1)?))).unwrap().flatten().collect();
                            let nodes_b: std::collections::HashMap<u32, String> = stmt.query_map([&sid2], |row| Ok((row.get(0)?, row.get(1)?))).unwrap().flatten().collect();

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
                        ops.push(crate::proto::PatchOp::replace_inner(500, 1, &comparison));
                        let pkt = crate::proto::patch(&ops);
                        let mut s = ws_sender.lock().await;
                        let _ = s.send(WsMessage::Binary(pkt)).await;
                    }
                },
                _ => {}
            }
        }
    }
}


pub fn run_benchmark(path: &str) {
    println!("🧪 NHTML Benchmark Tool v0.4.0");
    println!("--------------------------------------------------");
    
    let (html_content, label) = if let Ok(content) = std::fs::read_to_string(path) {
        (content, format!("Fichier: {}", path))
    } else {
        (
            "<html><body><h1>Hello World</h1><p>Ceci est un test de benchmark pour le protocole NHTML.</p><ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul></body></html>".to_string(),
            "Démonstration (Sample)".to_string()
        )
    };

    let html_size = html_content.len();
    
    // Simulation B-TREE (un seul nœud contenant tout le HTML pour le test de poids)
    let nodes = vec![(1, 1, "root".to_string(), html_content.clone())];
    let (btree_pkt, ratio) = crate::proto::btree(&nodes);
    let binary_size = btree_pkt.len();
    let _compressed_size = binary_size; // btree_pkt est déjà le paquet final (compressé si utile)
    
    // On recalcule pour le détail
    let tree_payload = crate::proto::serialize_nodes(&nodes);
    let raw_binary_size = tree_payload.len() + 14 + 5; // Header NBPS + Header B-TREE
    
    println!("📊 Cible : {}", label);
    println!("--------------------------------------------------");
    println!("{:<20} | {:>10} | {:>10}", "Format", "Taille (B)", "Gain");
    println!("--------------------------------------------------");
    println!("{:<20} | {:>10} | {:>10}", "HTML Brut", html_size, "-");
    
    let bin_gain = if html_size > 0 { (1.0 - (raw_binary_size as f32 / html_size as f32)) * 100.0 } else { 0.0 };
    println!("{:<20} | {:>10} | {:>9.1}%", "NHTML Binaire (Raw)", raw_binary_size, bin_gain);
    
    let total_gain = if html_size > 0 { (1.0 - (binary_size as f32 / html_size as f32)) * 100.0 } else { 0.0 };
    println!("{:<20} | {:>10} | {:>9.1}%", "NHTML + Zstd", binary_size, total_gain);
    println!("--------------------------------------------------");
    println!("✨ Ratio de compression Zstd : {:.1}%", (1.0 - ratio) * 100.0);
    println!("🚀 NHTML est {:.1}x plus efficace que le HTML brut.", html_size as f32 / binary_size as f32);
}
