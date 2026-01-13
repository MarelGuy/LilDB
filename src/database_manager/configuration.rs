use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Configuration {
    pub store_path: String,
    pub port: u16,
    pub show_local_ip: bool,
    pub show_public_ip: bool,
    pub tls: Option<(String, String)>, // None = no tls, (key, cert) pair
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            store_path: "./dbstore".into(),
            port: 8080,
            show_local_ip: false,
            show_public_ip: false,
            tls: None,
        }
    }
}

impl Configuration {
    pub async fn new(config_file_path: &str) -> Self {
        if let Ok(file) = tokio::fs::read_to_string(config_file_path).await {
            if let Ok(config) = toml::from_str(&file) {
                return config;
            }

            tracing::warn!("Error in configuration file, running with default configuration...");
        }

        tracing::warn!("Configuration file missing, running with default configuration...");

        Self::default()
    }
}
