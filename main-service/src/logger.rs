use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::infrastructure::config::CargoEnv;

pub struct Logger;
impl Logger {
    pub fn init(cargo_env: CargoEnv) -> WorkerGuard {
        let console_logger = std::io::stdout();
        let (non_blocking, guard) = tracing_appender::non_blocking(console_logger);

        // Set the default verbosity level for the root of the dependency graph.
        // env var: `RUST_LOG`
        let env_filter =
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "geoform_backend=debug,tower_http=debug".into()
            });
        
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
            .init();

        tracing::info!("Logger initialized");
        guard
    }
}