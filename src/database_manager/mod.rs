use std::{
    collections::HashSet,
    error::Error,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

use collection::Collection;
use configuration::Configuration;
use document::Document;

use crate::{lexer::token::TokenType, token_list::TokenList};

pub mod address;
pub mod collection;
pub mod configuration;
pub mod document;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Database {
    pub name: String,
    pub path: String,
    pub collections: HashSet<Collection>,
    pub current_collection: usize,
    pub config: Configuration,
}

impl Database {
    pub fn new(
        name: String,
        path: String,
        collections: HashSet<Collection>,
        current_collection: usize,
        config: Configuration,
    ) -> Self {
        Self {
            name,
            path,
            collections,
            current_collection,
            config,
        }
    }

    pub fn process_tokens(
        self: &mut Self,
        token_list: TokenList,
    ) -> Result<(String, bool), Box<dyn Error>> {
        let mut result: String = String::new();

        match token_list.tokens[0].token_type {
            TokenType::Exit => {
                return Ok(("".into(), true));
            }
            TokenType::Create => {
                result = self.f_create(token_list)?;
            }
            // TokenType::Drop => {
            //     result = f_drop::f_drop(token_list, database)?;
            // }
            TokenType::Use => {
                result = self.f_use(token_list)?;
            }
            // TokenType::Delete => {
            //     result = f_delete::f_delete(token_list, database)?;
            // }
            // TokenType::Help => {
            //     result = f_help::f_help(token_list, database)?;
            // }
            // TokenType::Insert => {
            //     result = f_insert::f_insert(token_list, database)?;
            // }
            // TokenType::Update => {
            //     result = f_update::f_update(token_list, database)?;
            // }
            // TokenType::Find => {
            //     result = f_find::f_find(token_list, database)?;
            // }
            _ => {
                result = format!("\nUnknown command: {}", token_list.tokens[0].slice);
            }
        }

        Ok((result, false))
    }

    // TODO: add path management and fix name files
    fn f_create(self: &mut Self, mut token_list: TokenList) -> Result<String, Box<dyn Error>> {
        token_list.next(1);

        let mut created_type: String = String::new();

        if token_list.current_token.token_type == TokenType::Db {
            token_list.next(1);

            let name: &str = token_list.current_token.slice;

            fs::create_dir_all(format!(
                "{}/{}",
                self.config.store_path.clone().unwrap(),
                name
            ))?;

            created_type = "database".into();
        }

        if token_list.current_token.token_type == TokenType::Collection {
            if self.name == "" {
                print!("\n\r");
                return Ok(String::from(
                    "Error: no database provided. Select one with \"use db <name>\"",
                ));
            } else {
                token_list.next(1);

                let name: &str = token_list.current_token.slice;

                fs::create_dir_all(format!(
                    "{}/{}/{}",
                    self.config.store_path.clone().unwrap(),
                    self.name,
                    name
                ))?;

                self.collections.insert(Collection::new(
                    name.to_string(),
                    name.to_string(),
                    vec![],
                ));

                created_type = "collection".into();
            }
        }

        print!("\n\r");

        Ok(format!(
            "Created {} {}",
            created_type, token_list.current_token.slice
        ))
    }

    fn f_use(self: &mut Self, mut token_list: TokenList) -> Result<String, Box<dyn Error>> {
        token_list.next(1);

        let config_store_path: String = self.config.store_path.clone().unwrap();
        let path: &Path = Path::new(&config_store_path);

        for entry in fs::read_dir(path)? {
            let entry: DirEntry = entry?;
            let path: PathBuf = entry.path();

            if path.is_dir() {
                let name: &str = path.file_name().unwrap().to_str().unwrap();

                if name == token_list.current_token.slice {
                    self.path = path.to_str().unwrap().to_string();
                    self.current_collection = 0;
                    self.name = name.to_string();

                    for db_entry in fs::read_dir(self.path.clone())? {
                        let db_entry: DirEntry = db_entry?;
                        let db_path: PathBuf = db_entry.path();

                        if db_path.is_dir() {
                            let collection_name: &str =
                                db_path.file_name().unwrap().to_str().unwrap();

                            let mut documents: Vec<Document> = vec![];

                            for doc_entry in fs::read_dir(db_path.clone())? {
                                let doc_entry: DirEntry = doc_entry?;
                                let doc_path: PathBuf = doc_entry.path();

                                if doc_path.is_file() {
                                    let document_name = doc_path
                                        .file_name()
                                        .unwrap()
                                        .to_str()
                                        .unwrap()
                                        .split(".")
                                        .next()
                                        .unwrap()
                                        .to_string();

                                    documents.push(Document::new(
                                        document_name,
                                        doc_path.to_str().unwrap().to_string(),
                                    ));
                                }
                            }

                            let collection: Collection = Collection::new(
                                collection_name.to_string(),
                                db_path.to_str().unwrap().to_string(),
                                documents,
                            );

                            self.collections.insert(collection);
                        }
                    }
                }
            } else if path.is_file() {
                continue;
            }
        }

        print!("{:?}", self);

        // Ok(format!("Using database: {}", self.name))
        Ok(format!("Using database: {:?}", self.clone()))
    }
}
