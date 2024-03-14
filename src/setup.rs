use std::{fs, time::SystemTime};

pub fn setup_logging(verbosity: log::LevelFilter) -> Result<(), fern::InitError> {
    let log_path = format!(
        "{}/Library/Logs/discord_apple_music_rpc.log",
        std::env::var("HOME").unwrap()
    );

    if let Ok(log_meta) = fs::metadata(&log_path) {
        if log_meta.len() > 20 * 1024 * 1024 {
            println!("Log file larger than 20mb. Removing...");
            fs::remove_file(&log_path)?;
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
        .chain(fern::log_file(log_path)?)
        .apply()?;

    Ok(())
}
