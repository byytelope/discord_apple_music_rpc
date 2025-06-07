use std::{fs, time::SystemTime};

use fern::{Dispatch, log_file};
use log::LevelFilter;

use crate::core::error::{PipeBoomError, PipeBoomResult};

pub fn setup_logging(verbosity: LevelFilter, max_log_size: u64) -> PipeBoomResult<()> {
    let home_dir = std::env::var("HOME").map_err(|e| {
        PipeBoomError::Config(format!("Failed to get HOME environment variable: {}", e))
    })?;

    let log_path = format!("{}/Library/Logs/pipeboom.log", home_dir);
    let err_log_path = format!("{}/Library/Logs/pipeboom.err", home_dir);

    for path in [&log_path, &err_log_path] {
        if let Ok(meta) = fs::metadata(path) {
            if meta.len() > max_log_size * 1024_u64.pow(2) {
                log::warn!(
                    "Log file '{}' larger than {}MB. Removing...",
                    path,
                    max_log_size
                );
                fs::remove_file(path).map_err(|e| PipeBoomError::Io(e.to_string()))?;
            }
        }
    }

    let base_config = Dispatch::new()
        .level(verbosity)
        .level_for("surf", LevelFilter::Warn)
        .format(|out, msg, rec| {
            out.finish(format_args!(
                "{} [{} -> {}::{}::{}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                rec.level(),
                rec.target(),
                rec.file().unwrap_or_default(),
                rec.line().unwrap_or_default(),
                msg
            ))
        });

    let general_log = log_file(&log_path).map_err(|e| PipeBoomError::Io(e.to_string()))?;

    let error_log = Dispatch::new()
        .level(LevelFilter::Error)
        .chain(log_file(&err_log_path).map_err(|e| PipeBoomError::Io(e.to_string()))?);

    base_config
        .chain(general_log)
        .chain(error_log)
        .apply()
        .map_err(|e| PipeBoomError::Internal(format!("Failed to initialize logger: {}", e)))?;

    Ok(())
}
