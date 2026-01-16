use database_manager::{configuration::RawConfig, Database};
use lexer::token::TokenType;
use std::{collections::HashSet, sync::Arc, time::Duration};
use token_list::TokenList;
use tokio::{signal, sync::Mutex};
use tonic::transport::{Server, ServerTlsConfig};
use tonic_grpc_manager::MyLilDBShell;
use tracing::{error, info};

#[cfg(feature = "tracy")]
use std::alloc::System;

use crate::{
    database_manager::configuration::Config,
    lildb::lil_db_shell_service_server::LilDbShellServiceServer,
};

mod database_manager;
mod lexer;
mod token_list;
mod tonic_grpc_manager;

pub mod lildb {
    tonic::include_proto!("lildb");
}

#[cfg(feature = "tracy")]
#[global_allocator]
static GLOBAL: tracy_client::ProfiledAllocator<System> =
    tracy_client::ProfiledAllocator::new(System, 100);

async fn lex_input(
    input: String,
    database: Arc<Mutex<Database>>,
) -> anyhow::Result<(String, bool, Arc<Mutex<Database>>)> {
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

    let (result, exit) = database.lock().await.process_tokens(token_list).await?;

    Ok((result, exit, database))
}

#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing_subscriber::fmt::init();
    info!("LilDB - 0.0.0");

    let config: RawConfig = RawConfig::new("./db_config.toml").await;

    let config: Config = config.check_config().await?;
    let config_arc: Arc<Config> = Arc::new(config);

    let database: Database = Database::new(
        String::new(),
        String::new(),
        HashSet::new(),
        0_usize,
        config_arc.clone(),
    );

    let server: Server = Server::builder();

    let server = if let Some(id) = &config_arc.id {
        server.tls_config(ServerTlsConfig::new().identity(id.clone()))?
    } else {
        server
    };

    let ddb_shell: MyLilDBShell = MyLilDBShell::new(Arc::new(Mutex::new(database)));

    let server = server
        .http2_keepalive_interval(Some(Duration::from_secs(5)))
        .http2_keepalive_timeout(Some(Duration::from_secs(10)))
        .add_service(LilDbShellServiceServer::new(ddb_shell))
        .serve(config_arc.address.use_addr.parse()?);

    let is_http_or_s = if config_arc.id.is_some() {
        "https"
    } else {
        "http"
    };

    info!(
        "Server started on \"{is_http_or_s}://{}\"",
        config_arc.address.show_addr
    );

    #[cfg(feature = "tracy")]
    {
        info!("Tracy is active");
    }

    tokio::select! {
        _ = server => error!("Server terminated"),
        _ = signal::ctrl_c() => info!("Ctrl+C received, shutting down"),
    }

    Ok(())
}
