use std::process::Command;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::core::{
    error::{PipeBoomError, PipeBoomResult},
    models::{PlayerState, Song},
};

fn run_osascript<T: DeserializeOwned>(script: String) -> PipeBoomResult<T> {
    let function = format!("(() => JSON.stringify({}))();", script);
    let command_output = Command::new("osascript")
        .arg("-l")
        .arg("JavaScript")
        .arg("-e")
        .arg(&function)
        .output();

    let output_stdout = match command_output {
        Ok(o) => {
            if !o.status.success() {
                let stderr = String::from_utf8_lossy(&o.stderr);
                return Err(PipeBoomError::AppleMusic(format!(
                    "Osascript execution failed for script '{}': {} (exit code: {})",
                    script,
                    stderr.trim(),
                    o.status.code().unwrap_or(-1)
                )));
            }
            o.stdout
        }
        Err(e) => {
            return Err(PipeBoomError::AppleMusic(format!(
                "Failed to execute osascript: {} (script: {})",
                e, script
            )));
        }
    };

    let res = String::from_utf8_lossy(&output_stdout).to_string();

    serde_json::from_str(&res).map_err(|e| {
        log::debug!(
            "Failed to parse osascript output as JSON: {}, output: '{}'",
            e,
            res
        );
        PipeBoomError::Parse(format!("Failed to parse Apple Music script output: {}", e))
    })
}

pub fn get_is_open(app_name: &str) -> PipeBoomResult<bool> {
    let script = format!(
        "Application('System Events').processes['{}'].exists()",
        app_name
    );

    run_osascript(script)
}

pub fn get_player_state(app_name: &str) -> PipeBoomResult<PlayerState> {
    let script = format!("Application('{}').playerState()", app_name);
    run_osascript(script)
}

pub fn get_current_song(app_name: &str) -> PipeBoomResult<Option<Song>> {
    let script = format!(
        "{{
          ...Application('{0}').currentTrack().properties(),
          playerPosition: Application('{0}').playerPosition(),
        }}",
        app_name
    );

    match run_osascript::<Value>(script) {
        Ok(val) => {
            if val
                .get("album")
                .and_then(|a| a.as_str())
                .is_none_or(|s| s.is_empty())
            {
                return Ok(None);
            }
            serde_json::from_value::<Song>(val)
                .map(Some)
                .map_err(|e| PipeBoomError::Parse(format!("Failed to parse song data: {}", e)))
        }
        Err(PipeBoomError::AppleMusic(msg)) => {
            log::warn!("Assuming no song due to AppleScript error: {}", msg);
            Ok(None)
        }
        Err(e) => {
            log::error!("Failed to get current song: {}", e);
            Err(e)
        }
    }
}
