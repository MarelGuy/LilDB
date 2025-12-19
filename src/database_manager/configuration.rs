use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Configuration {
    pub store_path: String,
    pub port: u16,
    pub show_local_ip: bool,
    pub show_public_ip: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            store_path: "./dbstore".into(),
            port: 8080,
            show_local_ip: false,
            show_public_ip: false,
        }
    }
}

impl Configuration {
    pub fn new(config_file_path: &str) -> anyhow::Result<Self> {
        let toml_file: String = std::fs::read_to_string(config_file_path)?;

        let config: Self = toml::from_str(&toml_file)?;

        Ok(config)
    }
}
