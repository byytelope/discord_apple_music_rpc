pub mod app;
pub mod config;
pub mod core;
pub mod integrations;

use app::App;
use config::Config;
use core::{
    error::{AppError, AppResult},
    logging::setup_logging,
};

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = Config::default();

    if let Err(e) = setup_logging(config.log_level, config.max_log_size) {
        eprintln!("Failed to initialize logging: {}", e);
        return Err(AppError::Config("Failed to initialize logging".to_string()));
    }

    let mut app = App::new(config);
    app.run().await
}
