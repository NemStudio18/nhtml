use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Default)]
pub struct NhtmlConfig {
    pub ports: Option<PortsConfig>,
    pub fastcgi: Option<FastCgiConfig>,
    #[allow(dead_code)]
    pub dev: Option<DevConfig>,
    pub security: Option<SecurityConfig>,
    pub database: Option<DatabaseConfig>,
    pub cluster: Option<ClusterConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ClusterConfig {
    pub enabled: bool,
    pub redis_url: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct DatabaseConfig {
    pub driver: Option<String>,
    pub uri: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct SecurityConfig {
    pub tls: Option<TlsConfig>,
    pub rate_limit: Option<RateLimitConfig>,
    pub allowed_origins: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert: String,
    pub key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    pub events_per_sec: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FastCgiConfig {
    pub address: Option<String>,
    pub addresses: Option<Vec<String>>,
    pub strategy: Option<String>, // "round-robin", "least-conn"
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct PortsConfig {
    pub ws: Option<u16>,
    pub php: Option<u16>,
    pub devtools: Option<u16>,
    #[allow(dead_code)]
    pub http: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct DevConfig {
    #[allow(dead_code)]
    pub auto_reload: Option<bool>,
}

impl NhtmlConfig {
    pub fn load() -> Self {
        let path = "nhtml.config.toml";
        if std::path::Path::new(path).exists() {
            if let Ok(content) = fs::read_to_string(path) {
                match toml::from_str(&content) {
                    Ok(config) => {
                        println!("📄 Fichier de configuration nhtml.config.toml détecté et chargé.");
                        return config;
                    }
                    Err(e) => {
                        eprintln!("❌ ERREUR FATALE: Fichier nhtml.config.toml invalide.");
                        eprintln!("Détails de l'erreur TOML : {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("❌ ERREUR FATALE: Impossible de lire nhtml.config.toml.");
                std::process::exit(1);
            }
        }
        NhtmlConfig::default()
    }
}
