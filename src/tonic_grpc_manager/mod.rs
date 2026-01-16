use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::info;

use crate::{
    database_manager::Database,
    lex_input,
    lildb::{
        lil_db_shell_service_server::LilDbShellService, ConnectToDbRequest, ConnectToDbResponse,
        DisconnectFromDbRequest, DisconnectFromDbResponse, RunCommandRequest, RunCommandResponse,
    },
};

pub struct MyLilDBShell {
    pub database: Arc<Mutex<Database>>,
}

impl MyLilDBShell {
    pub fn new(database: Arc<Mutex<Database>>) -> Self {
        Self { database }
    }
}

#[tonic::async_trait]
impl LilDbShellService for MyLilDBShell {
    type RunCommandStream = ReceiverStream<Result<RunCommandResponse, Status>>;

    async fn run_command(
        &self,
        request: Request<Streaming<RunCommandRequest>>,
    ) -> Result<Response<Self::RunCommandStream>, Status> {
        let mut stream: Streaming<RunCommandRequest> = request.into_inner();
        let (tx, rx) = mpsc::channel(1024);

        let db: Arc<Mutex<Database>> = self.database.clone();

        tokio::spawn(async move {
            while let Some(req) = stream.message().await.unwrap_or(None) {
                let command: String = req.command;

                let execution_result: Result<(String, bool, Arc<Mutex<Database>>), anyhow::Error> =
                    { lex_input(command, db.clone()).await };

                let output_message: String = match execution_result {
                    Ok((output, _should_exit, _)) => output,
                    Err(e) => {
                        format!("Error executing command: {e}\n")
                    }
                };

                if tx
                    .send(Ok(RunCommandResponse {
                        output: output_message,
                    }))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn connect_to_db(
        &self,
        request: Request<ConnectToDbRequest>,
    ) -> Result<Response<ConnectToDbResponse>, Status> {
        info!("New session with id: {}", request.get_ref().session_id);

        return Ok(Response::new(ConnectToDbResponse {
            success: true,
            message: "Connected!".into(),
        }));
    }

    async fn disconnect_from_db(
        &self,
        request: Request<DisconnectFromDbRequest>,
    ) -> Result<Response<DisconnectFromDbResponse>, Status> {
        info!(
            "Session disconnected with id: {}",
            request.get_ref().session_id
        );

        return Ok(Response::new(DisconnectFromDbResponse {
            success: true,
            message: "Disconnected!".into(),
        }));
    }
}
