use std::{
    collections::HashSet,
    error::Error,
    fs::{self, DirEntry, ReadDir},
    io,
    path::{Path, PathBuf},
    sync::mpsc,
};

use collection::Collection;
use configuration::Configuration;
use document::Document;
use tokio::task;
use tracing::error;
use waitgroup::WaitGroup;

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

    pub async fn process_tokens(
        &mut self,
        token_list: TokenList<'_>,
    ) -> Result<(String, bool), Box<dyn Error>> {
        let result: String = match token_list.tokens[0].tok_type {
            TokenType::Create => self.f_create(token_list)?,
            TokenType::Drop => self.f_drop(token_list)?,
            TokenType::Use => self.f_use(token_list).await?,
            TokenType::Show => self.f_show(token_list)?,
            // TokenType::Delete => {
            //     result = f_delete::f_delete(token_list, database)?;
            // }
            TokenType::Help => self.f_help(),
            // TokenType::Insert => {
            //     result = f_insert::f_insert(token_list, database)?;
            // }
            // TokenType::Update => {
            //     result = f_update::f_update(token_list, database)?;
            // }
            // TokenType::Find => {
            //     result = f_find::f_find(token_list, database)?;
            // }
            _ => format!("Unknown command: {}\n\r", token_list.tokens[0].slice),
        };

        Ok((result, false))
    }

    // TODO: add path management and fix name files
    fn f_create(&mut self, mut token_list: TokenList) -> Result<String, Box<dyn Error>> {
        token_list.next(1);

        let created_type: String = match token_list.current_token.tok_type {
            TokenType::Db => {
                token_list.next(1);

                let name: &str = token_list.current_token.slice;

                if fs::read_dir(format!(
                    "{}/{}",
                    self.config.store_path.as_ref().unwrap(),
                    name
                ))
                .is_ok()
                {
                    return Ok(format!("Database \"{name}\" already exists\n\r"));
                }

                fs::create_dir_all(format!(
                    "{}/{}",
                    self.config.store_path.as_ref().unwrap(),
                    name
                ))?;

                "Created database \"".into()
            }
            TokenType::Collection => {
                if self.name.is_empty() {
                    return Ok(String::from(
                        "Error: no database provided. Select one with \"use <name>\"\n\r",
                    ));
                }
                token_list.next(1);

                let name: &str = token_list.current_token.slice;

                fs::create_dir_all(format!(
                    "{}/{}/{}",
                    self.config.store_path.as_ref().unwrap(),
                    self.name,
                    name
                ))?;

                self.collections.insert(Collection::new(
                    name.to_string(),
                    name.to_string(),
                    vec![],
                ));

                "Created collection \"".into()
            }
            _ => "You need to specify either \"db\" or \"collection\" before \"".into(),
        };

        Ok(format!(
            "{}{}\"\n\r",
            created_type, token_list.current_token.slice
        ))
    }

    fn f_show(&self, mut token_list: TokenList<'_>) -> Result<String, Box<dyn Error>> {
        token_list.next(1);

        let mut output_stream = String::new();

        match token_list.current_token.tok_type {
            TokenType::Dbs => {
                for db_entry in fs::read_dir(self.config.store_path.as_ref().unwrap())? {
                    let db_entry: DirEntry = db_entry?;
                    let db_path: PathBuf = db_entry.path();

                    if db_path.is_dir() {
                        let name: &str = db_path.file_name().unwrap().to_str().unwrap();

                        output_stream.push_str(format!("{name}\n\r").as_str());
                    }
                }

                if output_stream.is_empty() {
                    output_stream = String::from("No databases found\n\r");
                }
            }
            TokenType::Identifier(_) => {
                let name: &str = token_list.current_token.slice;

                let is_error: String = self.read_names(&mut output_stream, name)?;

                if !is_error.is_empty() {
                    return Ok(is_error);
                }

                if output_stream.is_empty() {
                    output_stream = String::from("No collections found\n\r");
                }
            }
            _ => {
                if self.name.is_empty() {
                    return Ok(String::from("Error: invalid syntax\n\r"));
                }
            }
        }

        Ok(output_stream[..output_stream.len() - 1].into())
    }

    fn f_drop(&mut self, mut token_list: TokenList<'_>) -> Result<String, Box<dyn Error>> {
        token_list.next(1);

        let config_store_path: &String = self.config.store_path.as_ref().unwrap();

        let output_stream: String = match token_list.current_token.tok_type {
            TokenType::Db => {
                token_list.next(1);

                fs::remove_dir_all(format!(
                    "{}/{}",
                    config_store_path, token_list.current_token.slice
                ))?;

                self.name = String::new();
                self.path = String::new();

                self.current_collection = 0;
                self.collections = HashSet::new();

                format!(
                    "Dropped database \"{}\"\n\r",
                    token_list.current_token.slice
                )
            }
            TokenType::Collection => {
                token_list.next(1);

                self.collections.remove(&Collection::new(
                    token_list.current_token.slice.to_string(),
                    self.name.clone(),
                    vec![],
                ));

                fs::remove_dir_all(format!(
                    "{}/{}/{}",
                    config_store_path, self.name, token_list.current_token.slice
                ))?;

                format!(
                    "Dropped collection \"{}\"\n\r",
                    token_list.current_token.slice
                )
            }
            _ => String::from("Error: invalid syntax\n\r"),
        };

        Ok(output_stream)
    }

    #[allow(clippy::unused_self)]
    fn f_help(&self) -> String {
        String::from(        "Available commands:\n\r\
         -------------------\n\r\
         CREATE DB <database_name>           - Creates a new database.\n\r\
         CREATE COLLECTION <collection_name> - Creates a new collection in the current database.\n\r\
         DROP DB <database_name>             - Deletes a database and all its content.\n\r\
         DROP COLLECTION <collection_name>   - Deletes a collection from the current database.\n\r\
         USE <database_name>                 - Switches the current context to the specified database.\n\r\
         SHOW DBS                            - Lists all available databases.\n\r\
         SHOW <database_name>                - Lists all collections within the specified database. (Currently needs the db name even if you are using one)\n\r\
         HELP                                - Shows this help message.\n\r\
         EXIT                                - Exits the program.\n\r\
         \n\r\
         Unavailable commands (coming soon): DELETE, INSERT, UPDATE, FIND\n\r\
         "    )
    }

    async fn f_use(&mut self, mut token_list: TokenList<'_>) -> Result<String, Box<dyn Error>> {
        token_list.next(1);

        let config_store_path: &String = self.config.store_path.as_ref().unwrap();
        let path_string: String =
            format!("{}/{}", config_store_path, token_list.current_token.slice);
        let path: &Path = Path::new(&path_string);

        if path.is_dir() {
            let name: &str = path.file_name().unwrap().to_str().unwrap();

            if name == token_list.current_token.slice {
                self.path = path.to_str().unwrap().to_string();
                self.current_collection = 0;
                self.name = name.to_string();

                for db_entry in fs::read_dir(&self.path)? {
                    let db_entry: DirEntry = db_entry?;
                    let db_path: PathBuf = db_entry.path();

                    let mut collections: HashSet<Collection> = HashSet::new();

                    if db_path.is_dir() {
                        let collection_name: &str = db_path.file_name().unwrap().to_str().unwrap();

                        let db_path: &PathBuf = &db_path;

                        let (tx, rx): (mpsc::Sender<Document>, mpsc::Receiver<Document>) =
                            mpsc::channel();

                        let wg: WaitGroup = WaitGroup::new();

                        if fs::read_dir(db_path).unwrap().count() != 0 {
                            for doc_entry in fs::read_dir(db_path).unwrap() {
                                let tx: mpsc::Sender<Document> = tx.clone();
                                let worker: waitgroup::Worker = wg.worker();

                                task::spawn(async move {
                                    let doc_entry: DirEntry = doc_entry.unwrap();
                                    let doc_path: PathBuf = doc_entry.path();

                                    if doc_path.is_file() {
                                        let document_name: String = doc_path
                                            .file_name()
                                            .unwrap()
                                            .to_str()
                                            .unwrap()
                                            .split('.')
                                            .next()
                                            .unwrap()
                                            .to_string();

                                        tx.send(Document::new(
                                            document_name,
                                            doc_path.to_str().unwrap().to_string(),
                                        ))
                                        .unwrap_or_else(
                                            |err| error!("Error sending document: {}\n\r", err),
                                        );
                                    }

                                    drop(worker);
                                });
                            }

                            wg.wait().await;

                            drop(tx);

                            let mut documents: Vec<Document> = vec![];

                            while let Ok(document) = rx.recv() {
                                documents.push(document);
                            }

                            let collection: Collection = Collection::new(
                                collection_name.to_string(),
                                db_path.to_str().unwrap().to_string(),
                                documents,
                            );

                            collections.insert(collection);
                        }
                    }
                    self.collections = collections;
                }
            }
        } else {
            return Ok(String::from("Error: database not found\n\r"));
        }
        Ok(format!("Using database: {}\n\r", self.name))
    }

    // Utilities
    fn read_names(&self, output_stream: &mut String, name: &str) -> Result<String, Box<dyn Error>> {
        let dir: Result<ReadDir, io::Error> = fs::read_dir(format!(
            "{}/{}",
            self.config.store_path.as_ref().unwrap(),
            name
        ));

        let dir: ReadDir = match dir {
            Ok(dir) => dir,
            Err(_) => return Ok(String::from("Error: no such database\n\r")),
        };

        for db_entry in dir {
            let db_entry: DirEntry = db_entry?;
            let db_path: PathBuf = db_entry.path();

            if db_path.is_dir() {
                let name: &str = db_path.file_name().unwrap().to_str().unwrap();

                output_stream.push_str(format!("{name}\n\r").as_str());
            }
        }

        Ok(String::new())
    }
}
