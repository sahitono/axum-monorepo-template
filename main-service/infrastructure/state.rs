use std::sync::Arc;
use axum::extract::FromRef;
use sea_orm::DatabaseConnection;
use crate::infrastructure::config::Config;

#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub config: Arc<Config>,
    pub cache_text: Arc<moka::future::Cache<String, String>>,
}

impl AppState {
    pub fn init(db: Arc<DatabaseConnection>, config: Arc<Config>) -> Self {
        AppState {
            db,
            config,
            cache_text: Arc::new(Self::create_cache()),
        }
    }

    fn create_cache<V>() -> moka::future::Cache<String, V>
    where
        V: Send + Sync + Clone + 'static,
    {
        moka::future::Cache::builder()
            .max_capacity(100)
            .time_to_live(std::time::Duration::from_secs(15 * 60))
            .time_to_idle(std::time::Duration::from_secs(3 * 60))
            .build()
    }
}
