use crate::core::{
    error::{AppError, AppResult},
    models::{Song, SongDetails},
    utils::{current_time_as_u64, truncate},
};
use discord_presence::{Client, Event};

pub struct DiscordRpcClient {
    client: Client,
}

impl DiscordRpcClient {
    pub fn new(client_id: u64) -> Self {
        let client = Client::new(client_id);

        client
            .on_ready(|_ctx| {
                log::info!("Connected to Discord RPC");
            })
            .persist();

        client
            .on_error(|ctx| {
                log::error!("{:?}", ctx.event);
            })
            .persist();

        Self { client }
    }

    fn is_connected(&self) -> bool {
        discord_presence::Client::is_ready()
    }

    pub fn connect(&mut self) -> AppResult<()> {
        if self.is_connected() {
            return Err(AppError::Discord(
                "Discord client is already connected".into(),
            ));
        }
        self.client.start();
        self.client.block_until_event(Event::Ready)?;
        Ok(())
    }

    pub fn update_activity(&mut self, song: &Song, details: &SongDetails) -> AppResult<()> {
        if !self.is_connected() {
            return Err("Discord connection lost".into());
        }

        let current_time = match current_time_as_u64() {
            Ok(time) => time,
            Err(e) => {
                log::error!("Failed to get current time for Discord activity: {}", e);
                0
            }
        };

        self.client.set_activity(|act| {
            act.state(truncate(&song.artist, 128))
                ._type(discord_presence::models::ActivityType::Listening)
                .details(truncate(&song.name, 128))
                .timestamps(|stamp| stamp.start(current_time - song.player_position as u64))
                .assets(|ass| {
                    ass.small_image("apple_music_logo")
                        .large_image(&details.artwork)
                        .large_text(truncate(&song.album, 128))
                })
                .append_buttons(|butt| {
                    let url = if !details.song_url.is_empty() {
                        &details.song_url
                    } else if !details.album_url.is_empty() {
                        &details.album_url
                    } else {
                        "https://music.apple.com/"
                    };
                    butt.label("Listen on Apple Music").url(url)
                })
        })?;

        Ok(())
    }

    pub fn clear_activity(&mut self) -> AppResult<()> {
        if !self.is_connected() {
            return Ok(());
        }

        match self.client.clear_activity() {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.to_string().contains("Io Error") {
                    Ok(())
                } else {
                    Err(e.into())
                }
            }
        }
    }
}
