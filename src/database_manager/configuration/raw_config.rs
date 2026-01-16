use std::path::Path;

use anyhow::bail;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tonic::transport::Identity;
use tracing::{error, info, warn};

use crate::database_manager::address::Address;

use super::Config;

pub const DEFAULT_PORT: u16 = 44080;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash, PartialEq)]
pub struct RawConfig {
    pub store_path: Option<String>,
    pub port: Option<u16>,
    pub show_local_ip: Option<bool>,
    pub show_public_ip: Option<bool>,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
}

impl Default for RawConfig {
    fn default() -> Self {
        Self {
            store_path: Some("./dbstore".into()),
            port: Some(DEFAULT_PORT),
            show_local_ip: Some(false),
            show_public_ip: Some(false),
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

impl RawConfig {
    pub async fn new(config_file_path: &str) -> Self {
        let config = if let Ok(file) = tokio::fs::read_to_string(config_file_path).await {
            match toml::from_str::<Self>(&file) {
                Ok(de_config) => de_config,
                Err(err_config) => {
                    tracing::warn!(
                        "Error in configuration file: {} running with default configuration...",
                        err_config
                    );

                    Self::default()
                }
            }
        } else {
            tracing::warn!("Configuration file missing, running with default configuration...");

            Self::default()
        };

        config
    }

    pub async fn check_config(&self) -> anyhow::Result<Config> {
        let mut path: String = String::new();

        if let Some(self_store_path) = &self.store_path {
            let store_path: &Path = Path::new(self_store_path);

            let does_exist: bool = store_path.exists();
            let is_dir: bool = store_path.is_dir();

            if !does_exist {
                warn!("Couldn't open store_path");

                info!("Trying to create a directory at: {}", self_store_path);

                let try_create: Result<(), std::io::Error> =
                    tokio::fs::create_dir(self_store_path).await;

                if let Err(not_create) = try_create {
                    error!(
                        "Couldn't create directory at {}: {}",
                        self_store_path, not_create
                    );

                    bail!("Exiting...")
                }
            }

            if does_exist && !is_dir {
                error!(
                    "Couldn't create directory at {}: File with the same name as store_path exists",
                    self_store_path
                );

                bail!("Exiting...");
            }

            path = store_path.to_string_lossy().into();
        }

        let address: Address = Address::new(
            self.show_public_ip.unwrap_or(false),
            self.show_local_ip.unwrap_or(false),
            self.port.unwrap_or(DEFAULT_PORT),
        )
        .await?;

        if (TcpListener::bind(&address.use_addr).await).is_err() {
            error!("Address already in use: {}", address.use_addr);

            bail!("Exiting...");
        }

        if self.tls_cert_path.is_some() && self.tls_key_path.is_none() {
            error!("You must specify both tls_cert_path and tls_key_path: tls_key_path is missing");

            bail!("Exiting...");
        }

        if self.tls_cert_path.is_none() && self.tls_key_path.is_some() {
            error!(
                "You must specify both tls_cert_path and tls_key_path: tls_cert_path is missing"
            );

            bail!("Exiting...");
        }

        let id =
            if let (Some(cert_path), Some(key_path)) = (&self.tls_cert_path, &self.tls_key_path) {
                let cert: String = tokio::fs::read_to_string(cert_path).await?;
                let key: String = tokio::fs::read_to_string(key_path).await?;

                Some(Identity::from_pem(cert, key))
            } else {
                None
            };

        Ok(Config::new(path, address, id))
    }
}
