use std::path::PathBuf;

use crate::core::models::PlayerState;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
    sync::oneshot,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcCommand {
    Start,
    Stop,
    CurrentSong,
    Status,
    Shutdown,
}

#[derive(Debug)]
pub struct IpcRequest {
    pub command: IpcCommand,
    pub response_tx: oneshot::Sender<IpcResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcResponse {
    Success,
    Error(String),
    CurrentSong {
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
        state: PlayerState,
    },
    Status {
        running: bool,
        discord_connected: bool,
        discord_open: bool,
        music_app_open: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcMessage {
    pub command: IpcCommand,
}

pub async fn send_command(
    socket_path: PathBuf,
    command: IpcCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket_path).await?;

    let message = IpcMessage { command };
    let message_json = serde_json::to_string(&message)?;

    stream.write_all(message_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    let mut reader = BufReader::new(&mut stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    let response: IpcResponse = serde_json::from_str(response.trim())?;
    println!("{:#?}", response);

    Ok(())
}
