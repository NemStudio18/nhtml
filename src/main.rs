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

use tracing::info;
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
    },
    /// Inspecte un paquet binaire NBPS (hex)
    Inspect { hex: String },
    /// Valide un fichier binaire NBPS
    Validate { path: String },
    /// Affiche les statistiques des sessions
    Stats,
    /// Lance un benchmark de performance
    Bench { path: String },
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

    // Channel global pour le monitoring
    let (tx_monitor, _) = broadcast::channel::<MonitoringEvent>(100);

    match cli.command {
        Commands::New { name } => {
            cli::create_new_project(&name);
        }
        Commands::Start { dev: _, port, path, entry, php } => {
            println!("🛰️ NHTML Gateway v0.4.0");
            println!("📂 Projet : {}", path);
            println!("🌐 Port   : {}", port);
            
            info!("🚀 NHTML Gateway starting...");
            let sm = crate::session::SessionManager::new().await.expect("Impossible d'init le SessionManager");
            socket::serve(port, path, entry, php, std::sync::Arc::new(sm)).await;
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
