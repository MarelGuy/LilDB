use std::sync::Arc;

use lildb::{lil_db_shell_server::LilDbShell, CommandRequest, CommandResponse};
use tokio::sync::{mpsc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::info;

use crate::{
    database_manager::Database,
    lex_input,
    lildb::{self, ConnectRequest, ConnectResponse, DisconnectRequest, DisconnectResponse},
};

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
        let (tx, rx) = mpsc::channel(1024);

        let db: Arc<RwLock<Database>> = self.database.clone();

        tokio::spawn(async move {
            while let Some(req) = stream.message().await.unwrap_or(None) {
                let command: String = req.command;
                let needs_update: bool;
                let output_tuple: (String, bool, Database);

                {
                    let db_read: RwLockReadGuard<'_, Database> = db.read().await;
                    let db_clone: Database = db_read.clone();

                    output_tuple = lex_input(command, db_clone).await;

                    needs_update = db_read.clone() != output_tuple.2;
                }

                if needs_update {
                    let new_db: Database = output_tuple.2.clone();
                    let mut db_write: RwLockWriteGuard<'_, Database> = db.write().await;

                    *db_write = new_db;
                }

                let output: String = output_tuple.0;

                if tx.send(Ok(CommandResponse { output })).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn connect_to_db(
        &self,
        request: Request<ConnectRequest>,
    ) -> Result<Response<ConnectResponse>, Status> {
        info!("Ip connected: {}", request.get_ref().ip);

        return Ok(Response::new(ConnectResponse {
            success: true,
            message: "Connected!".into(),
        }));
    }

    async fn disconnect_from_db(
        &self,
        request: Request<DisconnectRequest>,
    ) -> Result<Response<DisconnectResponse>, Status> {
        info!("Ip disconnected: {}", request.get_ref().ip);

        return Ok(Response::new(DisconnectResponse {
            success: true,
            message: "Disconnected!".into(),
        }));
    }
}
