use tokio::process::Command;
use tokio::signal;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{BufReader, AsyncBufReadExt};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration, Instant};
use tracing::{error, info, warn};
use crate::MonitoringEvent;

const MAX_BACKOFF_SECS: u64 = 60;
const RESET_AFTER_SECS: u64 = 300; // Reset backoff after 5min of uptime

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

    let abs_path_raw = std::fs::canonicalize(&path).unwrap_or_else(|_| std::path::PathBuf::from(&path));
    let abs_str = abs_path_raw.to_string_lossy().to_string();
    let clean_abs_path = if abs_str.starts_with(r"\\?\") {
        abs_str[4..].to_string()
    } else {
        abs_str
    };

    let router_path = std::fs::canonicalize("router.php").unwrap_or_else(|_| std::path::PathBuf::from("router.php"));

    let mut attempt: u32 = 0;

    loop {
        // Exponential backoff before each restart (except first launch)
        if attempt > 0 {
            let backoff_secs = std::cmp::min(2u64.pow(attempt - 1), MAX_BACKOFF_SECS);
            warn!(
                "⚠️ Supervisor: Redémarrage #{} du serveur PHP dans {}s...",
                attempt, backoff_secs
            );
            // Notify browser clients of the restart
            let log_pkt = crate::proto::log_msg(1, &format!("[SUPERVISOR] PHP restart #{} in {}s...", attempt, backoff_secs));
            let mut msg = Vec::new();
            msg.push(crate::proto::SCOPE_ALL);
            msg.push(6);
            msg.extend_from_slice(b"SYSTEM");
            msg.extend_from_slice(&log_pkt);
            let _ = tx_app_broadcast.send(std::sync::Arc::new(msg));

            sleep(Duration::from_secs(backoff_secs)).await;
        }

        info!("⚙️ Supervisor: Lancement du serveur PHP (tentative #{}) avec : {}", attempt + 1, php_bin);

        let mut child = match Command::new(&php_bin)
            .arg("-S")
            .arg(format!("127.0.0.1:{}", port))
            .arg(&router_path)
            .current_dir(&clean_abs_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn() {
                Ok(c) => c,
                Err(e) => {
                    error!("❌ Supervisor: Échec du lancement du serveur PHP ({}) : {}", php_bin, e);
                    attempt += 1;
                    continue;
                }
            };

        info!("✅ Supervisor: Serveur PHP opérationnel sur le port {}.", port);
        let started_at = Instant::now();

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        if stdout.is_none() || stderr.is_none() {
            error!("❌ Supervisor: Impossible de capturer stdout/stderr du serveur PHP.");
            attempt += 1;
            continue;
        }

        let stdout = stdout.ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Stdout pipe failed"))?;
        let stderr = stderr.ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Stderr pipe failed"))?;

        let tx_m = tx_monitor.clone();
        let tx_a = tx_app_broadcast.clone();

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

        // Wait for process exit OR Ctrl+C
        tokio::select! {
            res = child.wait() => {
                let status = res?;
                let uptime = started_at.elapsed().as_secs();

                if uptime >= RESET_AFTER_SECS {
                    // Process lived long enough — reset backoff
                    info!("ℹ️ Supervisor: PHP vécu {}s. Réinitialisation du backoff.", uptime);
                    attempt = 0;
                } else {
                    attempt += 1;
                }

                error!("⚠️ Supervisor: PHP arrêté (statut: {}, uptime: {}s). Tentative #{} à venir.", status, uptime, attempt);
            }
            _ = signal::ctrl_c() => {
                info!("\n🛑 Supervisor: Signal d'arrêt reçu, fermeture du serveur PHP...");
                let _ = child.kill().await;
                return Ok(());
            }
        }
    }
}

fn handle_log_line(line: &str, tx_m: &broadcast::Sender<crate::MonitoringEvent>, tx_a: &broadcast::Sender<std::sync::Arc<Vec<u8>>>) {
    let log_pkt = crate::proto::log_msg(2, line); // INFO
    
    let _ = tx_m.send(MonitoringEvent {
        session_id: "SYSTEM".to_string(),
        direction: "OUT".to_string(),
        pkt_type: 0x10,
        size: log_pkt.len(),
        timestamp: chrono::Utc::now().timestamp_millis().to_string(),
        handler: Some("PHP_LOG".to_string()),
        latency_ms: None,
        compression_ratio: None,
        node_id: None,
        details: Some(line.to_string()),
    });

    let mut msg = Vec::new();
    msg.push(crate::proto::SCOPE_ALL);
    msg.push(6);
    msg.extend_from_slice(b"SYSTEM");
    msg.extend_from_slice(&log_pkt);
    
    let _ = tx_a.send(std::sync::Arc::new(msg));
}

