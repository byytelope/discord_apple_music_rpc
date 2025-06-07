use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{Activity, Assets, Button, Timestamps},
};

use crate::core::{
    constants::DISCORD_APP_ID,
    error::{PipeBoomError, PipeBoomResult},
    models::{Song, SongDetails},
    utils::{current_time_as_u64, truncate},
};

pub struct DiscordClient {
    client: DiscordIpcClient,
    pub is_connected: bool,
}

impl DiscordClient {
    pub fn new() -> Self {
        let client = DiscordIpcClient::new(DISCORD_APP_ID);

        Self {
            client,
            is_connected: false,
        }
    }

    pub fn connect(&mut self) -> PipeBoomResult<()> {
        if self.is_connected {
            return Err(PipeBoomError::Discord(
                "Tried connecting to Discord IPC with an existing connection".into(),
            ));
        }

        self.client.connect()?;
        self.is_connected = true;
        Ok(())
    }

    pub fn close(&mut self) -> PipeBoomResult<()> {
        if self.is_connected {
            let _ = self.clear_activity();

            match self.client.close() {
                Ok(_) => log::debug!("Discord IPC connection closed successfully"),
                Err(e) => log::warn!("Error closing Discord IPC connection: {}", e),
            }
        }

        self.is_connected = false;
        Ok(())
    }

    pub fn update_activity(&mut self, song: &Song, details: &SongDetails) -> PipeBoomResult<()> {
        if !self.is_connected {
            return Ok(());
        }

        let current_time = match current_time_as_u64() {
            Ok(time) => time,
            Err(e) => {
                log::error!("Failed to get current time for Discord activity: {}", e);
                0
            }
        };

        let timestamps = Timestamps::new().start(
            (current_time - song.player_position as u64)
                .try_into()
                .unwrap_or(0),
        );

        let assets = Assets::new()
            .small_image("apple_music_logo")
            .large_image(&details.artwork)
            .large_text(truncate(&song.album, 128));

        let buttons = vec![
            Button::new(
                "Listen on Apple Music",
                if !details.song_url.is_empty() {
                    &details.song_url
                } else if !details.album_url.is_empty() {
                    &details.album_url
                } else {
                    "https://music.apple.com/"
                },
            ),
            Button::new(
                "Share your AM status too!",
                "https://shadhaan.me/api/projects/pipeboom",
            ),
        ];

        let activity = Activity::new()
            .state(truncate(&song.artist, 128))
            .activity_type(discord_rich_presence::activity::ActivityType::Listening)
            .details(truncate(&song.name, 128))
            .timestamps(timestamps)
            .assets(assets)
            .buttons(buttons);

        if let Err(e) = self.client.set_activity(activity) {
            log::warn!("Failed to update Discord activity: {}", e);
            return Err(PipeBoomError::Discord(e.to_string()));
        }

        Ok(())
    }

    pub fn clear_activity(&mut self) -> PipeBoomResult<()> {
        if !self.is_connected {
            return Ok(());
        }

        if let Err(e) = self.client.clear_activity() {
            log::warn!("Failed to clear Discord activity: {}", e);
            return Err(PipeBoomError::Discord(e.to_string()));
        }

        Ok(())
    }
}
