use std::time::Duration;

use crate::{
    core::{
        error::{PipeBoomError, PipeBoomResult},
        models::PlayerState,
    },
    integrations::{
        apple_music::{get_current_song, get_is_open, get_player_state},
        discord::DiscordClient,
        itunes_api::get_details,
    },
};
use tokio::{
    sync::{mpsc, oneshot},
    time::sleep,
};

#[derive(Debug)]
pub enum Control {
    Start,
    Stop,
    Shutdown,
    GetStatus(oneshot::Sender<bool>),
}

pub struct Controller {
    discord_client: Option<DiscordClient>,
    app_name: &'static str,
    poll_interval: Duration,
    is_running: bool,
}

impl Controller {
    pub fn new(app_name: &'static str, poll_interval: Duration) -> Self {
        Self {
            discord_client: None,
            app_name,
            poll_interval,
            is_running: false,
        }
    }

    pub async fn run(mut self, mut control_rx: mpsc::UnboundedReceiver<Control>) {
        loop {
            tokio::select! {
                Some(control) = control_rx.recv() => {
                    match control {
                        Control::Start => {
                            if let Err(e) = self.start().await {
                                log::error!("Failed to start player: {}", e);
                            }
                        }
                        Control::Stop => {
                            if let Err(e) = self.stop().await {
                                log::error!("Failed to stop player: {}", e);
                            }
                        }
                        Control::Shutdown => {
                            log::info!("Player controller shutting down");
                            let _ = self.stop().await;
                            break;
                        }
                        Control::GetStatus(sender) => {
                            let _ = sender.send(self.is_running);
                        }
                    }
                }
                _ = sleep(self.poll_interval), if self.is_running => {
                    if let Err(e) = self.run_cycle().await {
                        if e.is_recoverable() {
                            log::warn!("Recoverable player error: {}", e);
                        } else {
                            log::error!("Fatal player error: {}", e);
                            let _ = self.stop().await;
                        }
                    }
                }
            }
        }
    }

    async fn start(&mut self) -> PipeBoomResult<()> {
        if self.is_running {
            return Ok(());
        }

        log::info!("Starting player controller");
        self.wait_for_applications().await?;
        self.initialize_discord_client()?;
        self.is_running = true;

        Ok(())
    }

    async fn stop(&mut self) -> PipeBoomResult<()> {
        if !self.is_running {
            return Ok(());
        }

        log::info!("Stopping player controller");
        self.is_running = false;

        if let Some(client) = self.discord_client.as_mut() {
            if client.is_connected {
                if let Err(e) = client.close() {
                    log::warn!("Error closing Discord client: {}", e);
                }
            }
        }
        self.discord_client = None;

        Ok(())
    }

    async fn wait_for_applications(&self) -> PipeBoomResult<()> {
        log::info!("Waiting for Discord and {}...", self.app_name);

        loop {
            let discord_open = get_is_open("Discord").map_err(|e| {
                PipeBoomError::Internal(format!("Failed to check if Discord is open: {}", e))
            })?;

            let music_open = get_is_open(self.app_name).map_err(|e| {
                PipeBoomError::Internal(format!(
                    "Failed to check if {} is open: {}",
                    self.app_name, e
                ))
            })?;

            if discord_open && music_open {
                log::info!("Both Discord and {} are now open", self.app_name);
                break;
            }

            log::debug!(
                "Waiting for apps - Discord: {}, {}: {}",
                discord_open,
                self.app_name,
                music_open
            );
            sleep(self.poll_interval).await;
        }

        Ok(())
    }

    fn initialize_discord_client(&mut self) -> PipeBoomResult<()> {
        log::info!("Initializing Discord client");

        let mut discord_client = DiscordClient::new();
        discord_client.connect()?;

        self.discord_client = Some(discord_client);
        log::info!("Discord client connected successfully");

        Ok(())
    }

    async fn run_cycle(&mut self) -> PipeBoomResult<()> {
        let discord_client = self.discord_client.as_mut().ok_or_else(|| {
            PipeBoomError::Internal("Discord client not initialized in player cycle".to_string())
        })?;

        if !get_is_open("Discord").map_err(|e| {
            PipeBoomError::Internal(format!("Failed to check Discord status: {}", e))
        })? {
            log::info!("Discord closed. Stopping player");
            return Err(PipeBoomError::Discord(
                "Discord application closed".to_string(),
            ));
        }

        if !get_is_open(self.app_name).map_err(|e| {
            PipeBoomError::Internal(format!("Failed to check {} status: {}", self.app_name, e))
        })? {
            log::info!(
                "{} closed. Clearing activity and stopping player",
                self.app_name
            );
            discord_client.clear_activity()?;
            return Err(PipeBoomError::Internal(format!("{} closed", self.app_name)));
        }

        let player_state = get_player_state(self.app_name)
            .map_err(|e| PipeBoomError::Internal(format!("Failed to get player state: {}", e)))?;

        match player_state {
            PlayerState::Playing => {
                if let Some(song) = get_current_song(self.app_name).map_err(|e| {
                    PipeBoomError::Internal(format!("Failed to get current song: {}", e))
                })? {
                    log::debug!("Currently playing: {} - {}", song.artist, song.name);

                    let details = get_details(&song).await?;
                    log::debug!("Song details retrieved successfully");

                    discord_client.update_activity(&song, &details)?;
                } else {
                    log::debug!("Player is playing but no song info available. Clearing activity.");
                    discord_client.clear_activity()?;
                }
            }
            _ => {
                log::debug!("Player state is {:?}. Clearing activity.", player_state);
                discord_client.clear_activity()?;
            }
        }

        Ok(())
    }
}
