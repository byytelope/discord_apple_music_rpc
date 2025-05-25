pub const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

use std::{
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use percent_encoding::{AsciiSet, CONTROLS};

use crate::error::{AppError, AppResult};

pub fn truncate(text: &str, max_length: usize) -> &str {
    match text.char_indices().nth(max_length) {
        Some((idx, _)) => &text[..idx],
        None => text,
    }
}

pub fn current_time_as_u64() -> AppResult<u64> {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)?;

    Ok(since_the_epoch.as_secs())
}

pub fn macos_ver() -> AppResult<f32> {
    let output_result = Command::new("sh")
        .arg("-c")
        .arg("sw_vers | grep ProductVersion | awk '{print $2}'")
        .output();

    let output = match output_result {
        Ok(o) => {
            if !o.status.success() {
                let stderr = String::from_utf8_lossy(&o.stderr);
                return Err(AppError::Io(format!("sw_vers command failed: {}", stderr)));
            }
            o.stdout
        }
        Err(e) => return Err(e.into()),
    };

    let ver_str = String::from_utf8_lossy(&output);
    let ver_parts = ver_str.trim().split('.').collect::<Vec<&str>>();

    if ver_parts.len() < 2 {
        return Err(AppError::Parse(format!(
            "Unexpected macOS version format: {}",
            ver_str.trim()
        )));
    }
    let major = ver_parts[0];
    let minor = ver_parts[1];

    let ver_float_str = format!("{}.{}", major, minor);

    Ok(ver_float_str.parse::<f32>()?)
}
