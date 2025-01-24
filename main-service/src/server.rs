use crate::infrastructure::config::{Config};
use crate::infrastructure::errors::AppError;
use crate::route::AppRoute;
use anyhow::Context;
use axum::{serve};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;
use tokio::signal;
use tracing::info;

pub struct AppServer;

impl AppServer {
    pub async fn start(config: Arc<Config>) -> anyhow::Result<()> {
        let address = format!("{}:{}", "0.0.0.0", config.port);
        let tcp_listener = tokio::net::TcpListener::bind(address)
            .await
            .context("Failed to bind TCP listener")?;

        let local_addr = tcp_listener
            .local_addr()
            .context("Failed to get local address")?;

        info!("server has launched on {local_addr} 🚀");

        let db = Arc::new(Self::create_db_conn(&config).await?);
        let router = AppRoute::init(db, config);

        serve(tcp_listener, router)
            .with_graceful_shutdown(Self::shutdown_signal())
            .await
            .context("Failed to start server")?;

        Ok(())
    }

    pub async fn create_db_conn(config: &Config) -> Result<DatabaseConnection, AppError> {
        let connection_options = config.get_connection_options();
        let db_conn = Database::connect(connection_options)
            .await?;

        Ok(db_conn)
    }

    async fn shutdown_signal() {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            () = ctrl_c => {},
            () = terminate => {},
        }

        tracing::warn!("❌ Signal received, starting graceful shutdown...");
    }
}
