use core::error::Error;

use crate::{database_manager::Database, token_list::TokenList};

pub fn f_delete(
    _token_list: TokenList,
    _database: &mut Database,
) -> Result<String, Box<dyn Error>> {
    Ok(String::new())
}
