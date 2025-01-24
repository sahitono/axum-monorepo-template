use crate::cli::AppRunCli;
use crate::infrastructure::config::read_config;
use crate::logger::Logger;
use crate::server::AppServer;
use clap::Parser;
use migration::{Migrator, MigratorTrait};
use std::sync::Arc;

pub(crate) mod dto;
pub(crate) mod extractor;
pub(crate) mod infrastructure;
mod logger;
mod middleware;
pub(crate) mod repository;
mod route;
mod server;
pub(crate) mod service;
pub(crate) mod utils;
mod cli;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli_config = AppRunCli::parse();
    let _guard = Logger::init(cli_config.cargo_env);

    let config = Arc::new(
        read_config(&cli_config.config, Some(cli_config.cargo_env))
            .await?,
    );

    tracing::info!("Migrating database started");
    let db = AppServer::create_db_conn(&config).await?;
    Migrator::up(&db, None).await?;
    db.close().await?;
    tracing::info!("Migrating database finished");

    AppServer::start(config).await?;

    Ok(())
}
