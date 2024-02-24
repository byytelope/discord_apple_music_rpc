pub const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use percent_encoding::{AsciiSet, CONTROLS};

pub fn truncate(text: &str, max_length: usize) -> &str {
    match text.char_indices().nth(max_length) {
        Some((idx, _)) => &text[..idx],
        None => text,
    }
}

pub fn current_time_as_u64() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|err| log::error!("{}", err))
        .unwrap();

    since_the_epoch.as_secs()
}

pub fn macos_ver() -> f32 {
    let output = Command::new("sh")
        .arg("-c")
        .arg("sw_vers | grep ProductVersion | awk '{print $2}'")
        .output()
        .map_err(|err| log::error!("{}", err))
        .unwrap()
        .stdout;

    let ver_str = String::from_utf8_lossy(&output);
    let ver_parts = ver_str.split('.').collect::<Vec<&str>>();

    let major = ver_parts[0];
    let minor = ver_parts[1];

    let ver_float_str = format!("{}.{}", major, minor);

    ver_float_str
        .parse::<f32>()
        .map_err(|err| log::error!("{}", err))
        .unwrap()
}

pub fn setup_logging(verbosity: log::LevelFilter) -> Result<(), fern::InitError> {
    let log_path = format!(
        "{}/Library/Logs/discord_apple_music_rpc.log",
        std::env::var("HOME").unwrap()
    );

    if let Ok(log_meta) = fs::metadata(&log_path) {
        if log_meta.len() > 20 * 1024 * 1024 {
            log::info!("Log file larger than 20mb. Removing...");
            fs::remove_file(&log_path)?;
        }
    }

    fern::Dispatch::new()
        .level(verbosity)
        .format(|out, msg, rec| {
            out.finish(format_args!(
                "{} [{}::{}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                rec.target(),
                rec.level(),
                msg
            ))
        })
        .chain(fern::log_file(log_path)?)
        .apply()?;

    Ok(())
}
