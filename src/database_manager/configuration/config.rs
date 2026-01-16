use tonic::transport::Identity;

use crate::database_manager::address::Address;

#[derive(Clone, Debug)]
pub struct Config {
    pub store_path: String,
    pub address: Address,
    pub id: Option<Identity>,
}

impl Config {
    pub fn new(store_path: String, address: Address, id: Option<Identity>) -> Self {
        Self {
            store_path,
            address,
            id,
        }
    }
}
