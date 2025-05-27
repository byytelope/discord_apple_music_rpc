use crate::{
    config::settings::Config,
    core::{
        error::{AppError, AppResult},
        models::PlayerState,
        utils::macos_ver,
    },
    integrations::{
        apple_music::{get_current_song, get_is_open, get_player_state},
        discord::DiscordRpcClient,
        itunes_api::get_details,
    },
};
use tokio::time::sleep;

pub struct App {
    discord_client: Option<DiscordRpcClient>,
    app_name: &'static str,
    config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
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
            discord_client: None,
            app_name,
            config,
        }
    }

    pub async fn run(&mut self) -> AppResult<()> {
        log::info!("Starting RPC...");
        log::info!("Waiting for Discord and {}...", self.app_name);

        loop {
            self.wait_for_applications().await?;
            self.discord_client = None;

            if let Err(e) = self.initialize_discord_client() {
                log::warn!("Failed to initialize Discord client: {}", e);
                continue;
            }

            if let Err(e) = self.run_player_loop().await {
                log::warn!("Player loop error: {}", e);
            }

            if let Some(client) = self.discord_client.as_mut() {
                if client.is_connected {
                    if let Err(e) = client.close() {
                        log::warn!("Error closing Discord client: {}", e);
                    }
                }
            }

            log::info!("Restarting connection cycle...");
        }
    }

    async fn wait_for_applications(&self) -> AppResult<()> {
        loop {
            sleep(self.config.poll_interval).await;
            let discord_is_open = get_is_open("Discord")?;
            let music_app_is_open = get_is_open(self.app_name)?;

            if discord_is_open && music_app_is_open {
                break;
            }
        }
        Ok(())
    }

    fn initialize_discord_client(&mut self) -> AppResult<()> {
        let mut discord_client = DiscordRpcClient::new(self.config.discord_app_id);
        discord_client.connect()?;

        self.discord_client = Some(discord_client);
        Ok(())
    }

    async fn run_player_loop(&mut self) -> AppResult<()> {
        let discord_client = self.discord_client.as_mut().ok_or_else(|| {
            AppError::Internal(
                "Discord client not initialized in player loop. This should not happen."
                    .to_string(),
            )
        })?;

        loop {
            sleep(self.config.poll_interval).await;

            if !get_is_open("Discord")? {
                log::info!("Discord closed. Exiting player loop.");
                break;
            }

            if !get_is_open(self.app_name)? {
                log::info!(
                    "{} closed. Clearing activity and exiting player loop.",
                    self.app_name
                );
                if let Err(e) = discord_client.clear_activity() {
                    log::warn!("Failed to clear Discord activity: {}", e);
                    break;
                }
                break;
            }

            match get_player_state(self.app_name)? {
                PlayerState::Playing => {
                    if let Some(song) = get_current_song(self.app_name)? {
                        log::info!("Currently playing: {:#?}", song);

                        let details = get_details(&song).await?;
                        log::info!("Song details: {:#?}", details);

                        if let Err(e) = discord_client.update_activity(&song, &details) {
                            log::warn!("Failed to update Discord activity: {}", e);
                            break;
                        }
                    } else {
                        log::debug!(
                            "Player state is Playing, but no current song information available. Clearing activity."
                        );
                        if let Err(e) = discord_client.clear_activity() {
                            log::warn!("Failed to clear Discord activity: {}", e);
                            break;
                        }
                    }
                }
                _ => {
                    log::debug!("Player state is not Playing. Clearing activity.");
                    if let Err(e) = discord_client.clear_activity() {
                        log::warn!("Failed to clear Discord activity: {}", e);
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
