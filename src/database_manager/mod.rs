use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use collection::Collection;
use configuration::Config;
use document::Document;
use tokio::{fs, sync::mpsc, task};
use waitgroup::WaitGroup;

use crate::{lexer::token::TokenType, token_list::TokenList};

pub mod address;
pub mod collection;
pub mod configuration;
pub mod document;

#[derive(Clone, Debug)]
pub struct Database {
    pub name: String,
    pub path: String,
    pub collections: HashSet<Collection>,
    pub current_collection: usize,
    pub config: Arc<Config>,
}

impl PartialEq for Database {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.path == other.path
            && self.collections == other.collections
            && self.current_collection == other.current_collection
            && Arc::ptr_eq(&self.config, &other.config)
    }
}

impl Database {
    pub fn new(
        name: String,
        path: String,
        collections: HashSet<Collection>,
        current_collection: usize,
        config: Arc<Config>,
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
    ) -> anyhow::Result<(String, bool)> {
        let result: String = match token_list.tokens[0].tok_type {
            TokenType::Create => self.f_create(token_list).await?,
            TokenType::Drop => self.f_drop(token_list).await?,
            TokenType::Use => self.f_use(token_list).await?,
            TokenType::Show => self.f_show(token_list).await?,
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
    async fn f_create(&mut self, mut token_list: TokenList<'_>) -> anyhow::Result<String> {
        token_list.next(1);

        let created_type: String = match token_list.current_token.tok_type {
            TokenType::Db => {
                token_list.next(1);

                let name: &str = token_list.current_token.slice;

                if fs::read_dir(format!("{}/{}", self.config.store_path, name))
                    .await
                    .is_ok()
                {
                    return Ok(format!("Database \"{name}\" already exists\n\r"));
                }

                fs::create_dir_all(format!("{}/{}", self.config.store_path, name)).await?;

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

                fs::create_dir_all(format!("{}/{}/{}", self.config.store_path, self.name, name))
                    .await?;

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

    async fn f_show(&self, mut token_list: TokenList<'_>) -> anyhow::Result<String> {
        token_list.next(1);

        let mut output_stream = String::new();

        match token_list.current_token.tok_type {
            TokenType::Dbs => {
                let mut entries: fs::ReadDir = fs::read_dir(&self.config.store_path).await?;

                while let Some(db_entry) = entries.next_entry().await? {
                    let db_path: PathBuf = db_entry.path();

                    if db_path.is_dir() {
                        let name: &str = db_path
                            .file_name()
                            .ok_or(anyhow::anyhow!("Invalid filename"))?
                            .to_str()
                            .ok_or(anyhow::anyhow!("Invalid UTF-8"))?;

                        output_stream.push_str(format!("{name}\n\r").as_str());
                    }
                }

                if output_stream.is_empty() {
                    output_stream = String::from("No databases found\n\r");
                }
            }
            TokenType::Identifier(_) => {
                let name: &str = token_list.current_token.slice;

                let is_error: String = self.read_names(&mut output_stream, name).await?;

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

    async fn f_drop(&mut self, mut token_list: TokenList<'_>) -> anyhow::Result<String> {
        token_list.next(1);

        let config_store_path: &String = &self.config.store_path;

        let output_stream: String = match token_list.current_token.tok_type {
            TokenType::Db => {
                token_list.next(1);

                fs::remove_dir_all(format!(
                    "{}/{}",
                    config_store_path, token_list.current_token.slice
                ))
                .await?;

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
                ))
                .await?;

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

    async fn f_use(&mut self, mut token_list: TokenList<'_>) -> anyhow::Result<String> {
        token_list.next(1);

        let config_store_path: &String = &self.config.store_path;
        let requested_db_name = token_list.current_token.slice;
        let path_string: String = format!("{config_store_path}/{requested_db_name}");
        let path: &Path = Path::new(&path_string);

        // 1. Check if the database directory exists asynchronously
        if let Ok(metadata) = tokio::fs::metadata(path).await {
            if metadata.is_dir() {
                let name: &str = path
                    .file_name()
                    .ok_or(anyhow::anyhow!("Invalid filename"))?
                    .to_str()
                    .ok_or(anyhow::anyhow!("Invalid UTF-8"))?;

                if name == requested_db_name {
                    self.path = path
                        .to_str()
                        .ok_or(anyhow::anyhow!("Invalid UTF-8"))?
                        .to_string();

                    self.current_collection = 0;
                    self.name = name.to_string();

                    let mut db_entries: fs::ReadDir = tokio::fs::read_dir(&self.path).await?;
                    let mut collections: HashSet<Collection> = HashSet::new();

                    while let Some(db_entry) = db_entries.next_entry().await? {
                        let db_path: PathBuf = db_entry.path();

                        if db_entry.file_type().await?.is_dir() {
                            let collection_name: String = db_path
                                .file_name()
                                .ok_or(anyhow::anyhow!("Invalid filename"))?
                                .to_str()
                                .ok_or(anyhow::anyhow!("Invalid UTF-8"))?
                                .to_string();

                            let (tx, mut rx): (mpsc::Sender<Document>, mpsc::Receiver<Document>) =
                                mpsc::channel(4);

                            let wg: WaitGroup = WaitGroup::new();

                            let mut doc_entries: fs::ReadDir =
                                tokio::fs::read_dir(&db_path).await?;
                            let mut has_files: bool = false;

                            while let Some(doc_entry) = doc_entries.next_entry().await? {
                                let doc_path: PathBuf = doc_entry.path();

                                if doc_entry.file_type().await?.is_file() {
                                    has_files = true;

                                    let tx: mpsc::Sender<Document> = tx.clone();
                                    let worker: waitgroup::Worker = wg.worker();

                                    task::spawn(async move {
                                        if let Some(os_name) = doc_path.file_name() {
                                            if let Some(name_str) = os_name.to_str() {
                                                let document_name = name_str
                                                    .split('.')
                                                    .next()
                                                    .ok_or(anyhow::anyhow!("Invalid UTF-8"))?
                                                    .to_string();

                                                tx.send(Document::new(
                                                    document_name,
                                                    doc_path
                                                        .to_str()
                                                        .ok_or(anyhow::anyhow!("Invalid UTF-8"))?
                                                        .to_string(),
                                                ))
                                                .await?;
                                            }
                                        }

                                        drop(worker);
                                        Ok::<(), anyhow::Error>(())
                                    });
                                }
                            }

                            // 4. Only process results if we actually found files
                            if has_files {
                                wg.wait().await;
                                drop(tx); // Close the channel so the loop below terminates

                                let mut documents: Vec<Document> = vec![];

                                // Note: This loop blocks the thread if the channel is slow,
                                // but since we awaited 'wg', the buffer should be ready or closed.
                                while let Some(document) = rx.recv().await {
                                    documents.push(document);
                                }

                                let collection: Collection = Collection::new(
                                    collection_name,
                                    db_path
                                        .to_str()
                                        .ok_or(anyhow::anyhow!("Invalid UTF-8"))?
                                        .to_string(),
                                    documents,
                                );

                                collections.insert(collection);
                            }
                        }
                    }
                    self.collections = collections;
                }
            } else {
                // Path exists but is not a directory
                return Ok(String::from("Error: database not found\n\r"));
            }
        } else {
            // Path does not exist
            return Ok(String::from("Error: database not found\n\r"));
        }

        Ok(format!("Using database: {}\n\r", self.name))
    }

    // Utilities
    async fn read_names(&self, output_stream: &mut String, name: &str) -> anyhow::Result<String> {
        let dir_res: Result<fs::ReadDir, io::Error> =
            tokio::fs::read_dir(format!("{}/{}", self.config.store_path, name)).await;

        let mut dir: fs::ReadDir = match dir_res {
            Ok(dir) => dir,
            Err(_) => return Ok(String::from("Error: no such database\n\r")),
        };

        while let Some(db_entry) = dir.next_entry().await? {
            let db_path: PathBuf = db_entry.path();

            if db_entry.file_type().await?.is_dir() {
                let name: &str = db_path
                    .file_name()
                    .ok_or(anyhow::anyhow!("Invalid filename"))?
                    .to_str()
                    .ok_or(anyhow::anyhow!("Invalid UTF-8"))?;

                output_stream.push_str(name);
                output_stream.push_str("\n\r");
            }
        }

        Ok(String::new())
    }
}
