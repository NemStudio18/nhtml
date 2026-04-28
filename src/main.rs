use tracing::{info, error};
use clap::{Parser, Subcommand};
use tokio::sync::broadcast;
use nhtml_gateway::{MonitoringEvent, cli, supervisor, socket, session, watcher, config};

#[derive(Parser)]
#[command(name = "nhtml-gateway")]
#[command(about = "NHTML Gateway - NBPS v0.7.0", long_about = None)]
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
        
        /// URI de la base de données (ex: mysql://user:pass@host/db)
        #[arg(long)]
        db_uri: Option<String>,

        /// Active les logs au format JSON (pour ELK/Datadog)
        #[arg(long)]
        json: bool,
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
    /// Partage le projet via un tunnel sécurisé
    Share {
        /// Port local à exposer (default: 8080)
        #[arg(short, long, default_value_t = 8080)]
        port: u16,

        /// Active les logs au format JSON (pour ELK/Datadog)
        #[arg(long)]
        json: bool,
    },
    /// Compile le projet pour la production
    Build {
        /// Active l'optimisation maximale
        #[arg(long)]
        production: bool,

        /// Chemin de sortie (default: dist)
        #[arg(short, long, default_value = "dist")]
        output: String,
    },
}

#[tokio::main]
async fn main() {
    let mut cli = Cli::parse();
    
    // Initialisation du logging
    if let Commands::Start { json, .. } = cli.command {
        if json {
            tracing_subscriber::fmt().json().init();
        } else {
            tracing_subscriber::fmt::init();
        }
    } else {
        tracing_subscriber::fmt::init();
    }

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
    let (tx_reload, _) = broadcast::channel::<()>(10);

    // Initialisation des métriques Prometheus (v0.7.0)
    let _ = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install()
        .expect("failed to install Prometheus recorder");
    info!("📊 Metrics: Exportateur Prometheus prêt sur le port standard des métriques.");

    match cli.command {
        Commands::New { name } => {
            cli::create_new_project(&name);
        }
        Commands::Start { dev, port, path, entry, php, fpm, db_uri, json: _ } => {
            println!("🛰️ NHTML Gateway v0.7.0");
            println!("📂 Projet : {}", path);
            println!("🌐 Port   : {}", port);
            
            let final_db_uri = db_uri.or(config.database.as_ref().and_then(|d| d.uri.clone()));
            if let Some(ref uri) = final_db_uri {
                println!("🗄️ Database : {} (Driver détecté)", uri);
            } else {
                println!("🗄️ Database : SQLite (Embarqué)");
            }

            // Priorité : Argument CLI > Fichier Config
            let fpm_addr = fpm.or(config.fastcgi.as_ref().and_then(|f| {
                if f.address.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    None
                } else {
                    f.address.clone()
                }
            }));
            let fpm_timeout = config.fastcgi.as_ref().and_then(|f| f.timeout_ms).unwrap_or(5000);
            
            if let Some(ref addr) = fpm_addr {
                println!("🚀 Mode Performance : PHP-FPM via {} (timeout: {}ms)", addr, fpm_timeout);
            }
            
            // Lancement du superviseur PHP avec Auto-Restart
            let php_port = config.ports.as_ref().and_then(|p| p.php).unwrap_or(8000);
            let tx_m = tx_monitor.clone();
            let tx_a = tx_app_broadcast.clone();
            let php_path = path.clone();
            tokio::spawn(async move {
                loop {
                    info!("⚙️ Supervisor: Démarrage du serveur PHP sur le port {}...", php_port);
                    let res = supervisor::start_php_server(
                        php_port, 
                        php_path.clone(),
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

            // Watcher si mode DEV
            let is_dev = dev || config.dev.as_ref().and_then(|d| d.auto_reload).unwrap_or(false);
            if is_dev {
                let tx_r = tx_reload.clone();
                watcher::start_watcher(tx_r);
            }

            // Cluster (Redis sync)
            let gid = uuid::Uuid::new_v4().to_string();
            info!("🆔 Gateway ID: {}", gid);

            if let Some(ref cluster) = config.cluster {
                if cluster.enabled {
                    let tx_a = tx_app_broadcast.clone();
                    let url = cluster.redis_url.clone();
                    let my_gid = gid.clone();
                    tokio::spawn(async move {
                        nhtml_gateway::cluster::start_cluster_bridge(my_gid, url, tx_a).await;
                    });
                }
            }

            info!("🚀 NHTML Gateway starting...");
            let sm = match crate::session::SessionManager::new().await {
                Ok(s) => s,
                Err(e) => {
                    error!("❌ Fatal: Impossible d'initialiser le SessionManager : {}", e);
                    return;
                }
            };
            socket::serve(gid, port, path, entry, php, fpm_addr, fpm_timeout, std::sync::Arc::new(sm), tx_monitor, tx_app_broadcast, tx_reload, config.security.clone()).await;
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
        Commands::Share { port, json: _ } => {
            cli::run_share(port);
        }
        Commands::Build { production, output } => {
            cli::run_build(production, &output);
        }
    }
}
