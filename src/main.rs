use database_manager::{address::Address, configuration::Configuration, Database};
use lexer::token::TokenType;
use lildb::lil_db_shell_server::LilDbShellServer;
use std::{collections::HashSet, error::Error, net, sync::Arc, time::Duration};
use token_list::TokenList;
use tokio::{signal, sync::RwLock};
use tonic::transport::Server;
use tonic_grpc_manager::MyLilDBShell;
use tracing::{error, info};

mod database_manager;
mod lexer;
mod token_list;
mod tonic_grpc_manager;

pub mod lildb {
    tonic::include_proto!("lildb");
}

async fn lex_input(input: String, mut database: Database) -> (String, bool, Database) {
    let lexer: lexer::Lexer<'_> = lexer::Lexer::new(&input);

    let mut token_list: TokenList = TokenList::new(vec![]);

    for token in lexer {
        if token.tok_type != TokenType::Null
            && token.tok_type != TokenType::Space
            && token.tok_type != TokenType::LineFeed
            && token.tok_type != TokenType::Tab
        {
            token_list.tokens.push(token);
        }
    }

    token_list.current_token = token_list.tokens[0];

    let (result, exit) = database.process_tokens(token_list).await.unwrap();

    (result, exit, database)
}

#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    info!("LilDB - 0.0.0");

    let config: Configuration = Configuration::new("./db_config.toml".into());

    let address: Address = Address::new(&config).await?;

    let addr: net::SocketAddr = address.use_addr.to_string().parse()?;

    let database: Database = Database::new(
        String::new(),
        String::new(),
        HashSet::new(),
        0_usize,
        config,
    );

    let ddb_shell: MyLilDBShell = MyLilDBShell::new(Arc::new(RwLock::new(database)));

    let server = Server::builder()
        .http2_keepalive_interval(Some(Duration::from_secs(5)))
        .http2_keepalive_timeout(Some(Duration::from_secs(10)))
        .add_service(LilDbShellServer::new(ddb_shell))
        .serve(addr);

    info!("Listening on https://{}", address.show_addr);

    tokio::select! {
        _ = server => error!("Server terminated"),
        _ = signal::ctrl_c() => info!("Ctrl+C received, shutting down"),
    }

    Ok(())
}
