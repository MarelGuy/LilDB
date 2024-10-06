use core::error::Error;
use std::{net, process};

use local_ip_address::local_ip;

use super::configuration::Configuration;

pub struct Address {
    pub use_addr: String,
    pub show_addr: String,
}

impl Address {
    pub async fn new(config: &Configuration) -> Result<Self, Box<dyn Error>> {
        if config.show_local_ip.unwrap_or_default() && config.show_public_ip.unwrap_or_default() {
            print!("Error: cannot use both show_local_ip and show_public_ip in config.toml\n\r");

            process::exit(1)
        }

        let mut use_addr: String = "127.0.0.1".into();
        let mut show_addr: String = "127.0.0.1".into();

        let use_port: u16 = config.port.unwrap_or_default();

        if config.show_local_ip.unwrap_or_default() {
            let local_ip: net::IpAddr = local_ip()?;

            use_addr = local_ip.to_string();
            show_addr = local_ip.to_string();
        }

        if config.show_public_ip.unwrap_or_default() {
            let public_ip: String = reqwest::get("https://api.ipify.org").await?.text().await?;

            use_addr = "0.0.0.0".into();
            show_addr = public_ip;
        }

        Ok(Self {
            use_addr: format!("{}:{}", use_addr, use_port),
            show_addr: format!("{}:{}", show_addr, use_port),
        })
    }
}
