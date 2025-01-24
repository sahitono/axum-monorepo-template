use chrono::TimeDelta;
use sea_orm::ConnectOptions;
use serde::{Deserialize};
use tracing::log;
use crate::infrastructure::errors::{AppError, AppResult};

#[derive(clap::ValueEnum, Clone, Debug, Copy, PartialEq)]
pub enum CargoEnv {
    Development,
    Production,
}



#[derive(Deserialize)]
pub struct ConfigUnparsed {
    port: Option<String>,
    database_url: String,
    timeout: Option<u64>,
    jwt_expire: Option<i64>,
    jwt_secret: String,
    host: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub port: String,
    pub database_url: String,
    pub cargo_env: CargoEnv,
    pub timeout: u64,
    pub jwt_expire: TimeDelta,
    pub jwt_secret: String,
    pub host: Option<String>,
}

impl ConfigUnparsed {
    fn to_config(&self, cargo_env: Option<CargoEnv>) -> Config {
        Config {
            port: self.port.clone().unwrap_or_else(|| "8080".to_string()), // Default port is 8080 if not specified
            database_url: self.database_url.clone(),
            cargo_env: cargo_env.unwrap_or(CargoEnv::Development),
            timeout: self.timeout.unwrap_or(30),
            jwt_expire: TimeDelta::seconds(self.jwt_expire.unwrap_or(3600)),
            jwt_secret: self.jwt_secret.clone(),
            host: self.host.clone(),
        }
    }
}

impl Config {
    pub fn get_connection_options(&self) -> ConnectOptions {
        let mut opt = ConnectOptions::new(self.database_url.clone());
        let num_thread = std::thread::available_parallelism().unwrap().get();
        let max_conn: u32 = (num_thread as u32) * 2;
        opt.max_connections(max_conn)
            .sqlx_logging(self.cargo_env == CargoEnv::Development)
            .sqlx_logging_level(log::LevelFilter::Debug);
        opt
    }
}

pub async fn read_config(config_file: &str, cargo_env: Option<CargoEnv>) -> AppResult<Config> {
    tracing::info!("Reading config file at {:?}", config_file);
    let file = tokio::fs::read_to_string(config_file).await?;
    let config: ConfigUnparsed = toml::from_str(&file).map_err(|e| AppError::InternalServerErrorWithContext(e.to_string()))?;
    Ok(config.to_config(cargo_env))
}



