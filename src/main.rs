mod app;
mod config;
mod core;
mod integrations;
mod ipc;

use app::App;
use config::settings::Config;
use core::{
    error::{AppError, AppResult},
    logging::setup_logging,
};

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = Config::default();

    setup_logging(config.log_level, config.max_log_size).map_err(|e| {
        eprintln!("Failed to initialize logging: {}", e);
        AppError::Config("Failed to initialize logging".to_string())
    })?;

    log::info!("Starting Pipeboom v{}", env!("CARGO_PKG_VERSION"));
    log::info!("Configuration: {:?}", config);

    let mut app = App::default();

    match app.run(config).await {
        Ok(_) => {
            log::info!("Pipeboom shut down successfully");
            Ok(())
        }
        Err(e) => {
            log::error!("Pipeboom error: {}", e);
            Err(e)
        }
    }
}
