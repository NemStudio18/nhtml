mod core;
mod cli;
mod supervisor;
mod watcher;
mod session;
mod proto;
mod decoder;
mod config;
mod compiler;
mod socket;

use tracing::{info, error};
use clap::{Parser, Subcommand};
use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

#[derive(Parser)]
#[command(name = "nhtml-gateway")]
#[command(about = "NHTML Gateway - NBPS v0.4.0", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Crée un nouveau projet NHTML
    New { name: String },
    /// Démarre le Gateway pour un projet
    Start {
        /// Mode développement (watcher)
        #[arg(long)]
        dev: bool,

        /// Port d'écoute (default: 8080)
        #[arg(short, long, default_value_t = 8080)]
        port: u16,

        /// Chemin du projet (default: .)
        #[arg(short = 'd', long, default_value = ".")]
        path: String,

        /// Fichier d'entrée (default: index.nhtml)
        #[arg(short, long, default_value = "index.nhtml")]
        entry: String,

        /// Script PHP (default: app.php)
        #[arg(short = 's', long, default_value = "app.php")]
        php: String,

        /// Adresse PHP-FPM (ex: 127.0.0.1:9000 ou unix:/var/run/php-fpm.sock)
        #[arg(long)]
        fpm: Option<String>,
    },
    /// Inspecte un paquet binaire NBPS (hex)
    Inspect { hex: String },
    /// Valide un fichier binaire NBPS
    Validate { path: String },
    /// Affiche les statistiques des sessions
    Stats,
    /// Lance un benchmark de performance
    Bench { path: String },
    /// Lance les DevTools
    Devtools,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringEvent {
    pub direction: String,
    pub pkt_type: u8,
    pub size: usize,
    pub session_id: String,
    pub timestamp: String,
    pub latency_ms: Option<u64>,
    pub compression_ratio: Option<f32>,
    pub handler: Option<String>,
    pub details: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let mut cli = Cli::parse();
    let config = crate::config::NhtmlConfig::load();

    // Appliquer la config sur les ports si non spécifiés explicitement
    // Note: Clap a déjà mis les valeurs par défaut, on écrase si config présente
    if let Commands::Start { ref mut port, .. } = cli.command {
        if let Some(ref ports) = config.ports {
            if let Some(ws_port) = ports.ws {
                *port = ws_port;
            }
        }
    }

    // Channels globaux
    let (tx_monitor, _) = broadcast::channel::<MonitoringEvent>(100);
    let (tx_app_broadcast, _) = broadcast::channel::<std::sync::Arc<Vec<u8>>>(1024);

    match cli.command {
        Commands::New { name } => {
            cli::create_new_project(&name);
        }
        Commands::Start { dev: _, port, path, entry, php, fpm } => {
            println!("🛰️ NHTML Gateway v0.4.0");
            println!("📂 Projet : {}", path);
            println!("🌐 Port   : {}", port);

            // Priorité : Argument CLI > Fichier Config
            let fpm_addr = fpm.or(config.fastcgi.as_ref().and_then(|f| f.address.clone()));
            let fpm_timeout = config.fastcgi.as_ref().and_then(|f| f.timeout_ms).unwrap_or(5000);
            
            if let Some(ref addr) = fpm_addr {
                println!("🚀 Mode Performance : PHP-FPM via {} (timeout: {}ms)", addr, fpm_timeout);
            }
            
            // Lancement du superviseur PHP avec Auto-Restart
            let php_port = config.ports.as_ref().and_then(|p| p.php).unwrap_or(8000);
            let tx_m = tx_monitor.clone();
            let tx_a = tx_app_broadcast.clone();
            tokio::spawn(async move {
                loop {
                    info!("⚙️ Supervisor: Démarrage du serveur PHP sur le port {}...", php_port);
                    let res = supervisor::start_php_server(
                        php_port, 
                        tx_m.clone(), 
                        tx_a.clone()
                    ).await;
                    
                    if let Err(e) = res {
                        error!("❌ Supervisor: Le serveur PHP a crashé : {}. Tentative de redémarrage dans 2s...", e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            });

            // Lancement des DevTools
            let devtools_port = config.ports.as_ref().and_then(|p| p.devtools).unwrap_or(8081);
            let devtools_tx = tx_monitor.clone();
            tokio::spawn(async move {
                cli::run_devtools(devtools_tx, "127.0.0.1".to_string(), devtools_port, None).await;
            });

            info!("🚀 NHTML Gateway starting...");
            let sm = match crate::session::SessionManager::new().await {
                Ok(s) => s,
                Err(e) => {
                    error!("❌ Fatal: Impossible d'initialiser le SessionManager : {}", e);
                    return;
                }
            };
            socket::serve(port, path, entry, php, fpm_addr, fpm_timeout, std::sync::Arc::new(sm), tx_monitor, tx_app_broadcast).await;
        }
        Commands::Devtools => {
            let devtools_port = config.ports.as_ref().and_then(|p| p.devtools).unwrap_or(8081);
            cli::run_devtools(tx_monitor, "127.0.0.1".to_string(), devtools_port, None).await;
        }
        Commands::Inspect { hex } => {
            cli::inspect_message(&hex);
        }
        Commands::Validate { path } => {
            cli::validate_file(&path);
        }
        Commands::Stats => {
            cli::dump_database();
        }
        Commands::Bench { path } => {
            cli::run_benchmark(&path);
        }
    }
}
