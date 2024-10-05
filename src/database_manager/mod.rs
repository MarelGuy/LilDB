use collection::Collection;
use configuration::Configuration;

pub mod address;
pub mod collection;
pub mod configuration;

#[derive(Clone, Debug)]
pub struct Database {
    name: String,
    path: String,
    colletions: Vec<Collection>,
    config: Configuration,
}

impl Database {
    pub fn new(
        name: String,
        path: String,
        colletions: Vec<Collection>,
        config: Configuration,
    ) -> Self {
        Self {
            name,
            path,
            colletions,
            config,
        }
    }
}
