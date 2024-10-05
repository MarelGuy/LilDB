use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Configuration {
    pub store_path: Option<String>,
    pub port: Option<u16>,
    pub show_local_ip: Option<bool>,
    pub show_public_ip: Option<bool>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            store_path: Some("./dbstore".into()),
            port: Some(8080_u16),
            show_local_ip: Some(false),
            show_public_ip: Some(false),
        }
    }
}

impl Configuration {
    pub fn new(config_file_path: String) -> Self {
        let toml_file: String = std::fs::read_to_string(config_file_path).unwrap();

        let config: Self = toml::from_str(&toml_file).unwrap();

        config
    }
}
