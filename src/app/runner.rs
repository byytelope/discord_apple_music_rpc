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
use std::thread;

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
            self.initialize_discord_client()?;
            self.run_player_loop().await?;
        }
    }

    async fn wait_for_applications(&self) -> AppResult<()> {
        loop {
            thread::sleep(self.config.poll_interval);
            let discord_is_open = get_is_open("Discord")?;
            let music_app_is_open = get_is_open(self.app_name)?;

            if discord_is_open && music_app_is_open {
                break;
            }
        }
        Ok(())
    }

    fn initialize_discord_client(&mut self) -> AppResult<()> {
        let client_id = self.config.discord_app_id.parse::<u64>()?;
        let mut discord_client = DiscordRpcClient::new(client_id);
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
            thread::sleep(self.config.poll_interval);

            if !get_is_open("Discord")? {
                log::info!("Discord closed. Exiting player loop.");
                break;
            }

            if !get_is_open(self.app_name)? {
                log::info!(
                    "{} closed. Clearing activity and exiting player loop.",
                    self.app_name
                );
                discord_client.clear_activity()?;
                break;
            }

            match get_player_state(self.app_name)? {
                PlayerState::Playing => {
                    if let Some(song) = get_current_song(self.app_name)? {
                        log::info!("Currently playing: {:#?}", song);

                        let details = get_details(&song).await?;
                        log::info!("Song details: {:#?}", details);

                        discord_client.update_activity(&song, &details)?;
                    } else {
                        log::debug!(
                            "Player state is Playing, but no current song information available. Clearing activity."
                        );
                        discord_client.clear_activity()?;
                    }
                }
                _ => {
                    log::debug!("Player state is not Playing. Clearing activity.");
                    discord_client.clear_activity()?;
                }
            }
        }

        Ok(())
    }
}
