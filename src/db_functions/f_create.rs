use core::error::Error;
use std::fs;

use crate::{database_manager::Database, lexer::token::TokenType, token_list::TokenList};

// TODO: add path management and fix name files
pub fn f_create(
    mut token_list: TokenList,
    database: &mut Database,
) -> Result<String, Box<dyn Error>> {
    token_list.next(1);

    if token_list.current_token.token_type == TokenType::Db {
        token_list.next(1);

        let name: &str = token_list.current_token.slice;

        fs::create_dir_all(name)?;
    }

    if token_list.current_token.token_type == TokenType::Collection {
        token_list.next(1);

        let name: &str = token_list.current_token.slice;

        fs::File::create(format!("{}.lildb", name))?;
    }

    println!();
    Ok(String::from("Creato"))
}
