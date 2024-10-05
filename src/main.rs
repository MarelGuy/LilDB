use database_manager::{address::Address, configuration::Configuration, Database};
use db_functions::{f_create, f_delete, f_drop, f_find, f_help, f_insert, f_update, f_use};
use lexer::token::TokenType;
use lildb::{
    lil_db_shell_server::{LilDbShell, LilDbShellServer},
    CommandRequest, CommandResponse,
};
use std::{error::Error, net};
use token_list::TokenList;
use tokio::{signal, sync::mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status, Streaming};

mod database_manager;
mod db_functions;
mod lexer;
mod token_list;

pub mod lildb {
    tonic::include_proto!("lildb");
}

#[derive(Default)]
pub struct MyLilDBShell {}

#[tonic::async_trait]
impl LilDbShell for MyLilDBShell {
    type RunCommandStream = ReceiverStream<Result<CommandResponse, Status>>;

    async fn run_command(
        &self,
        request: Request<Streaming<CommandRequest>>,
    ) -> Result<Response<Self::RunCommandStream>, Status> {
        let mut stream: Streaming<CommandRequest> = request.into_inner();
        let (tx, rx) = mpsc::channel(4);

        // TODO: Better config management
        let config: Configuration = Configuration::new("./db_config.toml".into());

        let mut database: Database = Database::new(String::new(), String::new(), vec![], config);

        tokio::spawn(async move {
            while let Ok(req) = stream.message().await {
                match req {
                    Some(req) => {
                        let command: String = req.command;

                        let output_tuple: (String, bool) = lex_input(command, &mut database);

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

fn lex_input(input: String, database: &mut Database) -> (String, bool) {
    let lexer: lexer::Lexer<'_> = lexer::Lexer::new(&input);

    let mut token_list: TokenList = TokenList::new(vec![]);

    for token in lexer {
        if token.token_type != TokenType::Null
            && token.token_type != TokenType::Space
            && token.token_type != TokenType::LineFeed
            && token.token_type != TokenType::Tab
        {
            token_list.tokens.push(token)
        }
    }

    token_list.current_token = token_list.tokens[0];

    let (result, exit) = process_tokens(token_list, database).unwrap();

    (result, exit)
}

fn process_tokens(
    token_list: TokenList,
    database: &mut Database,
) -> Result<(String, bool), Box<dyn Error>> {
    let mut result: String = String::new();

    match token_list.tokens[0].token_type {
        TokenType::Exit => {
            return Ok(("".into(), true));
        }
        TokenType::Create => {
            result = f_create::f_create(token_list, database)?;
        }
        TokenType::Drop => {
            result = f_drop::f_drop(token_list, database)?;
        }
        TokenType::Use => {
            result = f_use::f_use(token_list, database)?;
        }
        TokenType::Delete => {
            result = f_delete::f_delete(token_list, database)?;
        }
        TokenType::Help => {
            result = f_help::f_help(token_list, database)?;
        }
        TokenType::Insert => {
            result = f_insert::f_insert(token_list, database)?;
        }
        TokenType::Update => {
            result = f_update::f_update(token_list, database)?;
        }
        TokenType::Find => {
            result = f_find::f_find(token_list, database)?;
        }
        _ => {
            result = format!("\nUnknown command: {}", token_list.tokens[0].slice);
        }
    }

    Ok((result, false))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("LilDB - 0.0.0");

    let config: Configuration = Configuration::new("./db_config.toml".into());

    let address: Address = Address::new(&config).await?;

    let addr: net::SocketAddr = address.use_addr.to_string().parse()?;
    let ddb_shell: MyLilDBShell = MyLilDBShell::default();

    let server = Server::builder()
        .add_service(LilDbShellServer::new(ddb_shell))
        .serve(addr);

    println!("Listening on http://{}", address.show_addr);

    tokio::select! {
        _ = server => println!("Server terminated"),
        _ = signal::ctrl_c() => println!("Ctrl+C received, shutting down"),
    }

    Ok(())
}
