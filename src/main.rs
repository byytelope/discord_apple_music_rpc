mod app;
mod core;
mod integrations;
mod ipc;

use app::{
    App,
    cli::{Cli, CliCommand},
    setup::{setup_launch_agent, uninstall_launch_agent},
};
use clap::Parser;
use core::{
    error::{PipeBoomError, PipeBoomResult},
    logging::setup_logging,
};
use ipc::commands::{IpcCommand, send_command};

#[tokio::main]
async fn main() -> PipeBoomResult<()> {
    let cli = Cli::parse();

    let poll_interval = cli.poll_interval;
    let log_level = cli.log_level;
    let max_log_size = cli.max_log_size;
    let socket_path = cli.socket_path;

    setup_logging(log_level.into(), max_log_size).map_err(|e| {
        eprintln!("Failed to initialize logging: {}", e);
        PipeBoomError::Config("Failed to initialize logging".to_string())
    })?;

    if let Some(command) = cli.command {
        match command {
            CliCommand::Setup => setup_launch_agent()?,
            CliCommand::Uninstall => uninstall_launch_agent()?,
            CliCommand::Service(ipc_command) => match ipc_command {
                IpcCommand::Start
                | IpcCommand::Stop
                | IpcCommand::CurrentSong
                | IpcCommand::Status
                | IpcCommand::Shutdown => send_command(socket_path, ipc_command).await?,
            },
        }

        Ok(())
    } else {
        let mut app = App::default();
        log::info!("Starting PipeBoom v{}", env!("CARGO_PKG_VERSION"));
        log::info!("Using IPC socket at {:?}", socket_path);
        log::info!("Polling interval: {:?}", poll_interval);
        log::info!("Log level: {:?}", log_level);
        log::info!("Max log size: {}MB", max_log_size);

        match app.run(poll_interval, socket_path).await {
            Ok(_) => {
                log::info!("PipeBoom shut down successfully");
                Ok(())
            }
            Err(e) => {
                log::error!("PipeBoom error: {}", e);
                Err(e)
            }
        }
    }
}
