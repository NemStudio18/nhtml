use tokio::process::Command;
use tokio::signal;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{BufReader, AsyncBufReadExt};
use tokio::sync::broadcast;
use crate::MonitoringEvent;

pub async fn start_php_server(
    port: u16, 
    tx_monitor: broadcast::Sender<MonitoringEvent>,
    tx_app_broadcast: broadcast::Sender<Vec<u8>>
) {
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

    let mut child = Command::new(&php_bin)
        .arg("-S")
        .arg(format!("127.0.0.1:{}", port))
        .arg("router.php")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Échec du lancement du serveur PHP");

    println!("✅ Supervisor: Serveur PHP opérationnel sur le port {}.", port);

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

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

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Erreur lors de l'écoute du signal Ctrl+C");
        println!("\n🛑 Supervisor: Signal d'arrêt reçu, fermeture du serveur PHP...");
        let _ = child.kill().await;
        std::process::exit(0);
    });
}

fn handle_log_line(line: &str, tx_m: &broadcast::Sender<MonitoringEvent>, tx_a: &broadcast::Sender<Vec<u8>>) {
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
    });

    // 2. Browser Console (NBPS 0x10)
    let _ = tx_a.send(log_pkt);
}
