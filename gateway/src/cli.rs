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
    <title>NHTML Counter</title>
    <style>
        body { font-family: sans-serif; text-align: center; margin-top: 50px; }
        .counter { font-size: 3rem; margin: 20px; font-weight: bold; }
        button { padding: 10px 20px; font-size: 1.2rem; cursor: pointer; }
    </style>
</head>
<body>
    <h1>Démo NHTML v0.2.2</h1>
    <div class="counter" n-id="counter_value">0</div>
    <button n-id="btn_increment" n-click="increment">Incrémenter</button>

    <!-- Injection Polyfill -->
    <script type="module">
        import { initNhtml } from '/polyfill/bridge.js';
        initNhtml('ws://127.0.0.1:8080');
    </script>
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

use axum::{
    routing::get,
    Router,
    extract::{Path as AxumPath, ws::{WebSocket, WebSocketUpgrade, Message as WsMessage}},
    response::Html,
};

pub async fn run_replay(session_id: String) {
    println!("⏱️ Initialisation du Replay pour la session : {}", session_id);
    
    let sid_for_html = session_id.clone();
    let sid_for_ws = session_id.clone();

    let app = Router::new()
        .route("/replay/:sid", get(move |AxumPath(sid): AxumPath<String>| async move {
            let html = include_str!("../static/replay.html");
            // Injecter le SID dans le titre ou une variable globale si besoin, 
            // mais replay.html le récupère déjà via l'URL.
            Html(html.to_string())
        }))
        .route("/replay_ws/:sid", get(move |ws: WebSocketUpgrade, AxumPath(sid): AxumPath<String>| {
            async move {
                ws.on_upgrade(move |socket| handle_replay_ws(socket, sid))
            }
        }));

    let addr = "127.0.0.1:8081";
    let listener = tokio::net::TcpListener::bind(addr).await.expect("Impossible de lier le port 8081");
    println!("🚀 Interface de rejeu disponible sur http://{}/replay/{}", addr, session_id);
    
    axum::serve(listener, app).await.unwrap();
}

async fn handle_replay_ws(mut socket: WebSocket, session_id: String) {
    println!("🔌 Client Replay connecté pour : {}", session_id);
    
    // 1. Déterminer le chemin de la base de données
    let paths = ["nhtml_sessions.db", "gateway/nhtml_sessions.db", "../nhtml_sessions.db", "../../nhtml_sessions.db"];
    let mut db_path = "nhtml_sessions.db".to_string();
    for p in paths {
        if std::path::Path::new(p).exists() {
            db_path = p.to_string();
            break;
        }
    }

    let (initial_nodes, events) = {
        let conn = rusqlite::Connection::open(db_path).unwrap();

        // Collecter les nodes
        let mut stmt_nodes = conn.prepare("SELECT node_id, value FROM nodes WHERE session_id = ?").unwrap();
        let node_rows = stmt_nodes.query_map([&session_id], |row| {
            Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?))
        }).unwrap();
        let mut initial_nodes = Vec::new();
        for n in node_rows {
            let (id, val) = n.unwrap();
            initial_nodes.push(format!("{{\"id\": {}, \"val\": \"{}\"}}", id, val));
        }

        // Collecter les événements
        let mut stmt_events = conn.prepare("SELECT event_type, node_id FROM event_log WHERE session_id = ? ORDER BY id ASC").unwrap();
        let event_rows = stmt_events.query_map([&session_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        }).unwrap();
        let mut events = Vec::new();
        for e in event_rows {
            events.push(e.unwrap());
        }
        
        (initial_nodes, events)
    };

    // 2. Envoyer l'état initial
    let init_msg = format!("INIT: [{}]", initial_nodes.join(","));
    if socket.send(WsMessage::Text(init_msg)).await.is_err() {
        return;
    }

    // 3. Streamer les événements
    for (ev_type, nid) in events {
        // Simulation de délai
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        let msg = format!("Replay: {} on Node {}", ev_type, nid);
        if socket.send(WsMessage::Text(msg)).await.is_err() {
            break;
        }
    }
    
    println!("🏁 Replay terminé pour : {}", session_id);
}

use tokio::net::TcpListener;
