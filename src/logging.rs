use std::{fs, time::SystemTime};

use crate::error::{AppError, AppResult};

pub fn setup_logging(verbosity: log::LevelFilter, max_log_size: u64) -> AppResult<()> {
    let home_dir = std::env::var("HOME")
        .map_err(|e| AppError::Config(format!("Failed to get HOME environment variable: {}", e)))?;

    let log_path = format!("{}/Library/Logs/pipeboom.log", home_dir);
    let err_log_path = format!("{}/Library/Logs/pipeboom.err", home_dir);

    for path in [&log_path, &err_log_path] {
        if let Ok(meta) = fs::metadata(path) {
            if meta.len() > max_log_size {
                println!(
                    "Log file '{}' larger than {} bytes. Removing...",
                    path, max_log_size
                );
                fs::remove_file(path).map_err(|e| AppError::Io(e.to_string()))?;
            }
        }
    }

    let base_config = fern::Dispatch::new()
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
        });

    let general_log = fern::log_file(&log_path).map_err(|e| AppError::Io(e.to_string()))?;

    let error_log = fern::Dispatch::new()
        .level(log::LevelFilter::Error)
        .chain(fern::log_file(&err_log_path).map_err(|e| AppError::Io(e.to_string()))?);

    base_config
        .chain(general_log)
        .chain(error_log)
        .apply()
        .map_err(|e| AppError::Internal(format!("Failed to initialize logger: {}", e)))?;

    Ok(())
}
