use std::{fs, time::SystemTime};

use crate::error::{AppError, AppResult};

pub fn setup_logging(verbosity: log::LevelFilter, max_log_size: u64) -> AppResult<()> {
    let log_path = format!(
        "{}/Library/Logs/damr.log",
        std::env::var("HOME").map_err(|e| {
            AppError::Config(format!("Failed to get HOME environment variable: {}", e))
        })?
    );

    if let Ok(log_meta) = fs::metadata(&log_path) {
        if log_meta.len() > max_log_size {
            println!("Log file larger than {} bytes. Removing...", max_log_size);
            fs::remove_file(&log_path).map_err(|e| AppError::Io(e.to_string()))?;
        }
    }

    fern::Dispatch::new()
        .level(verbosity)
        .format(|out, msg, rec| {
            out.finish(format_args!(
                "{} [{}::{}::{} -> {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                rec.target(),
                rec.file().unwrap_or_default(),
                rec.line().unwrap_or_default(),
                rec.level(),
                msg
            ))
        })
        .chain(fern::log_file(log_path).map_err(|e| AppError::Io(e.to_string()))?)
        .apply()
        .map_err(|e| AppError::Internal(format!("Failed to initialize logger: {}", e)))?;

    Ok(())
}
