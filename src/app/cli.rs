use std::{
    env::{home_dir, temp_dir},
    path::PathBuf,
    time::Duration,
};

use clap::{Parser, Subcommand, ValueEnum};
use log::LevelFilter;

use crate::ipc::commands::IpcCommand;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    /// Override poll interval (seconds)
    #[arg(long, value_parser = parse_duration, default_value = "1")]
    pub poll_interval: Duration,

    /// Override log level
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    /// Override max log size (MB)
    #[arg(long, default_value_t = 20)]
    pub max_log_size: u64,

    /// Override socket path
    #[arg(long, default_value_os_t = home_dir().unwrap_or(temp_dir()).join(".pipeboom.sock"))]
    pub socket_path: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Set up the Launch Agent and start the service
    Setup,
    /// Uninstall the Launch Agent
    Uninstall,
    /// Service control commands
    #[command(subcommand)]
    Service(IpcCommand),
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    if let Ok(seconds) = s.parse::<u8>() {
        if (1..=10).contains(&seconds) {
            Ok(Duration::from_secs(seconds.into()))
        } else {
            Err("Must be between 1-10".into())
        }
    } else {
        Err("Not a number".into())
    }
}
