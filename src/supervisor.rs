use tokio::process::Command;
use tokio::signal;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{BufReader, AsyncBufReadExt};
use tokio::sync::broadcast;
use tracing::error;
use crate::MonitoringEvent;

pub async fn start_php_server(
    port: u16, 
    path: String,
    tx_monitor: broadcast::Sender<crate::MonitoringEvent>,
    tx_app_broadcast: broadcast::Sender<std::sync::Arc<Vec<u8>>>
) -> std::io::Result<()> {
    let php_bin = if Path::new("./php.exe").exists() {
        "./php.exe".to_string()
    } else if Path::new("./php/php.exe").exists() {
        "./php/php.exe".to_string()
    } else if Path::new("./bin/php.exe").exists() {
        "./bin/php.exe".to_string()
    } else {
        "php".to_string() 
    };

    println!("⚙️ Supervisor: Tentative de lancement avec : {}", php_bin);

    let abs_path_raw = std::fs::canonicalize(&path).unwrap_or_else(|_| std::path::PathBuf::from(&path));
    let abs_str = abs_path_raw.to_string_lossy().to_string();
    let clean_abs_path = if abs_str.starts_with(r"\\?\") {
        abs_str[4..].to_string()
    } else {
        abs_str
    };

    let router_path = std::fs::canonicalize("router.php").unwrap_or_else(|_| std::path::PathBuf::from("router.php"));
    
    let mut child = match Command::new(&php_bin)
        .arg("-S")
        .arg(format!("127.0.0.1:{}", port))
        .arg(router_path)
        .current_dir(clean_abs_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
            Ok(c) => c,
            Err(e) => {
                error!("❌ Supervisor: Échec du lancement du serveur PHP ({}) : {}", php_bin, e);
                return Err(e);
            }
        };

    println!("✅ Supervisor: Serveur PHP opérationnel sur le port {}.", port);

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    if stdout.is_none() || stderr.is_none() {
        error!("❌ Supervisor: Impossible de capturer stdout/stderr du serveur PHP.");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Pipe failed"));
    }

    let stdout = stdout.ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Stdout pipe failed"))?;
    let stderr = stderr.ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Stderr pipe failed"))?;

    let tx_m = tx_monitor.clone();
    let tx_a = tx_app_broadcast.clone();

    // Loop pour capturer les logs et les diffuser
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        let mut err_reader = BufReader::new(stderr).lines();

        loop {
            tokio::select! {
                Ok(Some(line)) = reader.next_line() => {
                    handle_log_line(&line, &tx_m, &tx_a);
                }
                Ok(Some(line)) = err_reader.next_line() => {
                    handle_log_line(&line, &tx_m, &tx_a);
                }
                else => break,
            }
        }
    });

    // ATTENDRE que le processus se termine OU qu'on reçoive un Ctrl+C
    tokio::select! {
        res = child.wait() => {
            let status = res?;
            error!("⚠️ Supervisor: Le serveur PHP s'est arrêté avec le statut : {}", status);
        }
        _ = signal::ctrl_c() => {
            println!("\n🛑 Supervisor: Signal d'arrêt reçu, fermeture du serveur PHP...");
            let _ = child.kill().await;
            // On peut propager une erreur pour arrêter la boucle de redémarrage si on veut, 
            // mais ici on va laisser la boucle redémarrer ou s'arrêter via le main
        }
    }

    Ok(())
}

fn handle_log_line(line: &str, tx_m: &broadcast::Sender<crate::MonitoringEvent>, tx_a: &broadcast::Sender<std::sync::Arc<Vec<u8>>>) {
    let log_pkt = crate::proto::log_msg(2, line); // INFO
    
    // 1. Dashboard (Struct MonitoringEvent)
    let _ = tx_m.send(MonitoringEvent {
        session_id: "SYSTEM".to_string(),
        direction: "OUT".to_string(),
        pkt_type: 0x10, // LOG OpCode
        size: log_pkt.len(),
        timestamp: chrono::Utc::now().timestamp_millis().to_string(),
        handler: Some("PHP_LOG".to_string()),
        latency_ms: None,
        compression_ratio: None,
        node_id: None,
        details: Some(line.to_string()),
    });

    // 2. Browser Console (NBPS 0x10) - Format Broadcaster v0.6.0
    let mut msg = Vec::new();
    msg.push(crate::proto::SCOPE_ALL); // 0x02
    msg.push(6); // len("SYSTEM")
    msg.extend_from_slice(b"SYSTEM");
    msg.extend_from_slice(&log_pkt);
    
    let _ = tx_a.send(std::sync::Arc::new(msg));
}
