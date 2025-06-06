use crate::core::models::PlayerState;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

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
