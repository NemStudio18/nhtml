/// socket/mod.rs
/// Serveur WebSocket — gère les sessions, reçoit les EVENT,
/// dispatche vers PHP, renvoie les PATCH.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use tracing::{info, warn, error};

use crate::compiler::{NhtmlCompiler, CompileResult};
use crate::compiler::handler_table::{HandlerTable, build_from_tree};
use crate::proto;

use crate::core::{SessionState, Node, NodeType, PatchOp as CorePatchOp, EventLogEntry};

// ─── État de session ────────────────────────────────────────────────────────

pub struct Session {
    pub state: SessionState,
    pub php_script: String,
    pub handler_table: HandlerTable,
}

impl Session {
    fn new(id: String, result: &CompileResult, php_script: String) -> Self {
        let handler_table = build_from_tree(&result.root);
        Self { 
            state: SessionState::new(id), 
            php_script, 
            handler_table 
        }
    }
}

// ─── Point d'entrée ─────────────────────────────────────────────────────────

pub async fn serve(port: u16, root: String, entry: String, php: String) {
    let addr     = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await
        .expect("Impossible de lier le port");

    info!("Gateway en écoute sur ws://{}", addr);

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        info!("Nouvelle connexion depuis {}", addr);

        let session_id = uuid::Uuid::new_v4().to_string();

        let nhtml_path = format!("{}/{}", root, entry);
        let source = match std::fs::read_to_string(&nhtml_path) {
            Ok(s) => s,
            Err(e) => {
                error!("Impossible de lire {} : {}", nhtml_path, e);
                continue;
            }
        };

        let compile_result = NhtmlCompiler::compile(&source);
        let php_script     = format!("{}/{}", root, php);

        tokio::spawn(handle_connection(
            stream,
            session_id,
            compile_result,
            php_script,
        ));
    }
}

// ─── Gestion d'une connexion ────────────────────────────────────────────────

async fn handle_connection(
    stream     : tokio::net::TcpStream,
    session_id : String,
    result     : CompileResult,
    php_script : String,
)
{
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => { error!("WebSocket handshake échoué: {}", e); return; }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let session = Session::new(session_id, &result, php_script);

    // ── Séquence d'initialisation ──────────────────────────────────────────

    // 1. HELLO (On utilise un ID u32 court pour le binaire, tiré du hash de l'UUID par exemple, ou juste 0 pour l'instant)
    let hello = proto::hello(0, 5000);
    ws_sender.send(Message::Binary(hello)).await.ok();

    // 2. B-TREE
    let btree_pkt = proto::btree(&result.btree_bytes);
    ws_sender.send(Message::Binary(btree_pkt)).await.ok();
    info!("[{}] B-TREE envoyé ({} bytes)", session.state.session_id, result.btree_bytes.len());

    // 3. BIND × N
    for bind_pkt in &result.bind_packets {
        ws_sender.send(Message::Binary(bind_pkt.clone())).await.ok();
    }
    info!("[{}] {} paquets BIND envoyés", session_id, result.bind_packets.len());

    // ── Boucle de messages ─────────────────────────────────────────────────

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                if data.is_empty() { continue; }

                match data[0] {
                    proto::PKT_EVENT => {
                        handle_event(&data, &session, &mut ws_sender).await;
                    }
                    proto::PKT_PING => {
                        // Répondre PONG
                        let seq = data.get(4).copied().unwrap_or(0);
                        ws_sender.send(Message::Binary(
                            vec![proto::PKT_PING, 0x00, 0x02, 0x01, seq]
                        )).await.ok();
                    }
                    proto::PKT_ERR => {
                        warn!("[{}] ERR reçu du client", session_id);
                    }
                    t => {
                        warn!("[{}] Paquet inattendu type=0x{:02X}", session_id, t);
                    }
                }
            }
            Ok(Message::Close(_)) => {
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
    ws_sender  : &mut (impl SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
)
{
    // Parser le paquet EVENT (spec v0.2)
    // [0x03][2B len][2B source_id][1B event_type][1B handler_len][handler][2B payload_len][payload]
    if data.len() < 7 { return; }

    let source_id    = (data[1] as u16) << 8 | data[2] as u16;
    // Note: data[0] = type, data[1..2] = length (déjà consommé par assembler côté client)
    // Ici on parse directement le paquet complet reçu via WS
    let source_id    = ((data[3] as u16) << 8) | data[4] as u16;
    let event_type   = data[5];
    let handler_len  = data[6] as usize;

    if data.len() < 7 + handler_len { return; }

    let handler = std::str::from_utf8(&data[7..7 + handler_len])
        .unwrap_or("")
        .to_string();

    let payload_start = 7 + handler_len + 2;
    let payload = if payload_start <= data.len() {
        &data[payload_start..]
    } else {
        &[]
    };

    info!("[{}] EVENT node={} handler={}", session.state.session_id, source_id, handler);

    // ─── Record Engine (P0) ──────────────────────────────────────────────
    let log_entry = EventLogEntry {
        timestamp: chrono::Utc::now(),
        event_type: "CLICK".to_string(), // Simplifié pour le proto
        node_id: source_id as u32,
        payload: String::from_utf8_lossy(payload).to_string(),
    };
    
    // Si on était en mode --dev, on pourrait tracer ce log
    // trace_packet("RECORD", &log_entry, true);

    // Appeler PHP avec le contexte complet
    let patches = call_php(
        &session.php_script,
        &session.handler_table,
        source_id,
        event_type,
        &handler,
        payload,
    ).await;

    // Envoyer les PATCH résultants
    if !patches.is_empty() {
        let patch_pkt = proto::patch(&patches);
        ws_sender.send(Message::Binary(patch_pkt)).await.ok();
    }
}

// ─── Bridge PHP ─────────────────────────────────────────────────────────────

async fn call_php(
    php_script    : &str,
    handler_table : &HandlerTable,
    source_id     : u16,
    event_type    : u8,
    handler       : &str,
    payload       : &[u8],
) -> Vec<proto::PatchOp>
{
    use std::process::Stdio;
    use tokio::process::Command;

    // Construire le contexte JSON transmis au PHP via stdin
    let context = serde_json::json!({
        "handler"      : handler,
        "source_id"    : source_id,
        "event_type"   : event_type,
        "payload"      : String::from_utf8_lossy(payload),
        "handler_table": handler_table.to_json(),
        "nid_map"      : handler_table.nid_map,
    });

    let input = context.to_string();

    // php -f script.php — lit le contexte sur stdin, retourne JSON sur stdout
    let output = Command::new("php")
        .arg("-f")
        .arg(php_script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| async move {
            use tokio::io::AsyncWriteExt;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(input.as_bytes()).await.ok();
            }
            child.wait_with_output().await
        })
        .await;

    match output {
        Ok(out) if out.status.success() => {
            parse_php_response(&out.stdout, handler_table)
        }
        Ok(out) => {
            error!("PHP stderr: {}", String::from_utf8_lossy(&out.stderr));
            vec![]
        }
        Err(e) => {
            error!("Erreur lancement PHP: {}", e);
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
    // Le PHP retourne un tableau JSON de PatchOp
    // [ {"op":"set_text","nid":"compteur","value":"5"}, ... ]
    let json = match std::str::from_utf8(stdout) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let ops: Vec<serde_json::Value> = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(e) => {
            error!("Réponse PHP invalide: {} — {}", e, json);
            return vec![];
        }
    };

    let mut patch_ops = Vec::new();

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
                let val = op["value"].as_str().unwrap_or("");
                proto::PatchOp::set_text(node_id, val)
            }
            "add_class" => {
                let cls = op["value"].as_str().unwrap_or("");
                proto::PatchOp::add_class(node_id, cls)
            }
            "del_class" => {
                let cls = op["value"].as_str().unwrap_or("");
                proto::PatchOp::del_class(node_id, cls)
            }
            "set_attr" => {
                let key = op["key"].as_str().unwrap_or("");
                let val = op["value"].as_str().unwrap_or("");
                proto::PatchOp::set_attr(node_id, key, val)
            }
            "set_style" => {
                let prop = op["prop"].as_str().unwrap_or("");
                let val  = op["value"].as_str().unwrap_or("");
                proto::PatchOp::set_style(node_id, prop, val)
            }
            "replace_inner" => {
                let val = op["value"].as_str().unwrap_or("");
                proto::PatchOp::replace_inner(node_id, val)
            }
            "remove" => proto::PatchOp::remove(node_id),
            "focus"  => proto::PatchOp::focus(node_id),
            _ => {
                warn!("PatchOp inconnue: {}", op_type);
                continue;
            }
        };

        patch_ops.push(patch);
    }

    patch_ops
}
