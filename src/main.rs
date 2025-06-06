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
use ipc::commands::{IpcCommand, send_command};
use std::env;

#[tokio::main]
async fn main() -> AppResult<()> {
    let args = env::args().collect::<Vec<String>>();
    let config = Config::default();

    setup_logging(config.log_level, config.max_log_size).map_err(|e| {
        eprintln!("Failed to initialize logging: {}", e);
        AppError::Config("Failed to initialize logging".to_string())
    })?;

    if args.len() == 1 {
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
    } else {
        let command = match args[1].as_str() {
            "start" => IpcCommand::Start,
            "stop" => IpcCommand::Stop,
            "current-song" => IpcCommand::CurrentSong,
            "status" => IpcCommand::Status,
            "shutdown" => IpcCommand::Shutdown,
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("Usage: {} <command>", args[0]);
                eprintln!("Commands: start, stop, current-song, status, shutdown");
                return Ok(());
            }
        };

        send_command(std::env::temp_dir().join("pipeboom.sock"), command).await?;

        Ok(())
    }
}
