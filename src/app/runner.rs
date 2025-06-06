use crate::{
    app::controller::{Control, Controller},
    config::settings::Config,
    core::{error::AppResult, models::PlayerState, utils::macos_ver},
    integrations::apple_music::{get_current_song, get_is_open, get_player_state},
    ipc::{
        commands::{IpcCommand, IpcResponse},
        server::IpcServer,
    },
};
use tokio::sync::{mpsc, oneshot};

pub struct App {
    app_name: &'static str,
    player_control_tx: Option<mpsc::UnboundedSender<Control>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let app_name = match macos_ver() {
            Ok(ver) if ver >= 10.15 => "Music",
            Ok(_) => "iTunes",
            Err(e) => {
                log::warn!(
                    "Failed to determine macOS version: {}. Defaulting app name to 'Music'.",
                    e
                );
                "Music"
            }
        };

        Self {
            app_name,
            player_control_tx: None,
        }
    }

    pub async fn run(&mut self, config: Config) -> AppResult<()> {
        log::info!("Starting Pipeboom v{}", env!("CARGO_PKG_VERSION"));

        let (mut ipc_server, mut request_rx) = IpcServer::new();

        tokio::spawn(async move {
            if let Err(e) = ipc_server.start().await {
                log::error!("IPC server error: {}", e);
            }
        });

        let (player_control_tx, player_control_rx) = mpsc::unbounded_channel();
        self.player_control_tx = Some(player_control_tx);

        let player_controller = Controller::new(self.app_name, config);
        tokio::spawn(async move {
            player_controller.run(player_control_rx).await;
        });

        log::info!("Pipeboom is ready for IPC commands");

        let _ = self.handle_start().await;

        loop {
            tokio::select! {
                Some(request) = request_rx.recv() => {
                    let response = match request.command {
                        IpcCommand::Start => self.handle_start().await,
                        IpcCommand::Stop => self.handle_stop().await,
                        IpcCommand::CurrentSong => self.handle_get_current_song().await,
                        IpcCommand::Status => self.handle_get_status().await,
                        IpcCommand::Shutdown => {
                            log::info!("Received shutdown command via IPC");
                            if request.response_tx.send(IpcResponse::Success).is_err() {
                                log::warn!("Failed to send shutdown response");
                            }
                            break;
                        }
                    };

                    if !matches!(request.command, IpcCommand::Shutdown) && request.response_tx.send(response).is_err() {
                        log::warn!("Failed to send IPC response - client may have disconnected");
                    }
                }
            }
        }

        if let Some(tx) = &self.player_control_tx {
            let _ = tx.send(Control::Shutdown);
        }

        log::info!("Pipeboom shutting down");
        Ok(())
    }

    async fn handle_start(&mut self) -> IpcResponse {
        log::info!("Received start command via IPC");

        if let Some(tx) = &self.player_control_tx {
            if tx.send(Control::Start).is_err() {
                return IpcResponse::Error(
                    "Failed to send start command to player controller".to_string(),
                );
            }
        } else {
            return IpcResponse::Error("Player controller not available".to_string());
        }

        IpcResponse::Success
    }

    async fn handle_stop(&mut self) -> IpcResponse {
        log::info!("Received stop command via IPC");

        if let Some(tx) = &self.player_control_tx {
            if tx.send(Control::Stop).is_err() {
                return IpcResponse::Error(
                    "Failed to send stop command to player controller".to_string(),
                );
            }
        } else {
            return IpcResponse::Error("Player controller not available".to_string());
        }

        IpcResponse::Success
    }

    async fn handle_get_current_song(&self) -> IpcResponse {
        match get_current_song(self.app_name) {
            Ok(song_opt) => {
                if let Some(song) = song_opt {
                    let state = get_player_state(self.app_name).unwrap_or(PlayerState::Unknown);
                    IpcResponse::CurrentSong {
                        title: Some(song.name),
                        artist: Some(song.artist),
                        album: Some(song.album),
                        state,
                    }
                } else {
                    IpcResponse::CurrentSong {
                        title: None,
                        artist: None,
                        album: None,
                        state: PlayerState::Stopped,
                    }
                }
            }
            Err(e) => IpcResponse::Error(format!("Failed to get current song: {}", e)),
        }
    }

    async fn handle_get_status(&self) -> IpcResponse {
        let discord_open = get_is_open("Discord").unwrap_or(false);
        let music_open = get_is_open(self.app_name).unwrap_or(false);

        let running = if let Some(tx) = &self.player_control_tx {
            let (status_tx, status_rx) = oneshot::channel();
            if tx.send(Control::GetStatus(status_tx)).is_ok() {
                status_rx.await.unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        };

        IpcResponse::Status {
            running,
            discord_connected: discord_open,
            discord_open,
            music_app_open: music_open,
        }
    }
}
