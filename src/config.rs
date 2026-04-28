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

#[derive(Debug, Deserialize)]
pub struct FastCgiConfig {
    pub address: Option<String>,
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
        if let Ok(content) = fs::read_to_string("nhtml.config.toml") {
            if let Ok(config) = toml::from_str(&content) {
                println!("📄 Fichier de configuration nhtml.config.toml détecté et chargé.");
                return config;
            } else {
                eprintln!("⚠️ Fichier nhtml.config.toml trouvé mais format invalide.");
            }
        }
        NhtmlConfig::default()
    }
}
