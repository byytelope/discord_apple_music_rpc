use std::fs;
use std::io::Write;
use std::process::Command;

use crate::core::{
    constants::BUNDLE_ID,
    error::{PipeBoomError, PipeBoomResult},
};

pub fn setup_launch_agent() -> PipeBoomResult<()> {
    let home = std::env::var("HOME").expect("HOME not set");
    let service_name = BUNDLE_ID;
    let plist_dir = format!("{}/Library/LaunchAgents", home);
    let plist_file = format!("{}/{}.plist", plist_dir, service_name);
    let exe_path = std::env::current_exe()
        .map_err(|e| PipeBoomError::Setup(format!("Failed to get current exe path: {e}")))?;
    let install_dir = exe_path
        .parent()
        .ok_or_else(|| PipeBoomError::Setup("Failed to get binary directory".to_string()))?
        .to_string_lossy()
        .to_string();

    fs::create_dir_all(&plist_dir)?;

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{service_name}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{install_dir}/pipeboom</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{home}/Library/Logs/pipeboom.log</string>
    <key>StandardErrorPath</key>
    <string>{home}/Library/Logs/pipeboom.err</string>
    <key>WorkingDirectory</key>
    <string>{home}</string>
</dict>
</plist>
"#,
        service_name = service_name,
        install_dir = install_dir,
        home = home
    );

    let mut file = fs::File::create(&plist_file)?;
    file.write_all(plist_content.as_bytes())?;

    let _ = Command::new("launchctl")
        .arg("unload")
        .arg(&plist_file)
        .output();

    let output = Command::new("launchctl")
        .arg("load")
        .arg(&plist_file)
        .output()?;

    if !output.status.success() {
        return Err(PipeBoomError::Setup(format!(
            "Failed to load Launch Agent: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    println!("Launch Agent installed and loaded successfully!");
    println!("Logs will be available at:");
    println!("  Stdout: {}/Library/Logs/pipeboom.log", home);
    println!("  Stderr: {}/Library/Logs/pipeboom.err", home);
    println!("Make sure {} is in your PATH", install_dir);

    Ok(())
}

pub fn uninstall_launch_agent() -> PipeBoomResult<()> {
    let home = std::env::var("HOME").expect("HOME not set");
    let plist_file = format!("{}/Library/LaunchAgents/{}.plist", home, BUNDLE_ID);

    if fs::metadata(&plist_file).is_err() {
        return Err(PipeBoomError::Setup(format!(
            "Launch Agent {} not found",
            plist_file
        )));
    }

    let _ = Command::new("launchctl")
        .arg("unload")
        .arg(&plist_file)
        .output();

    fs::remove_file(&plist_file)?;

    println!("Launch Agent uninstalled successfully!");

    Ok(())
}
