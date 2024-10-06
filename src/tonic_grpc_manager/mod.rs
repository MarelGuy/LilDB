use std::sync::Arc;

use lildb::{lil_db_shell_server::LilDbShell, CommandRequest, CommandResponse};
use tokio::sync::{mpsc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

use crate::{database_manager::Database, lex_input, lildb};

#[derive(Default)]
pub struct MyLilDBShell {
    pub database: Arc<RwLock<Database>>,
}

impl MyLilDBShell {
    pub fn new(database: Arc<RwLock<Database>>) -> Self {
        Self { database }
    }
}

#[tonic::async_trait]
impl LilDbShell for MyLilDBShell {
    type RunCommandStream = ReceiverStream<Result<CommandResponse, Status>>;

    async fn run_command(
        &self,
        request: Request<Streaming<CommandRequest>>,
    ) -> Result<Response<Self::RunCommandStream>, Status> {
        let mut stream: Streaming<CommandRequest> = request.into_inner();
        let (tx, rx) = mpsc::channel(4);

        let db: Arc<RwLock<Database>> = self.database.clone();
        // TODO: Better config management

        tokio::spawn(async move {
            while let Ok(req) = stream.message().await {
                match req {
                    Some(req) => {
                        let command: String = req.command;

                        let db_read: RwLockReadGuard<'_, Database> = db.read().await;
                        let db_clone: Database = db_read.clone();

                        drop(db_read);

                        let output_tuple: (String, bool, Database) =
                            lex_input(command, db_clone.clone());

                        if db_clone != output_tuple.2 {
                            let new_db: Database = output_tuple.2;
                            let mut db_write: RwLockWriteGuard<'_, Database> = db.write().await; // Acquire a write lock
                            *db_write = new_db;
                        }

                        let output: String = output_tuple.0;

                        tx.send(Ok(CommandResponse { output })).await.unwrap();
                    }
                    _ => break,
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
