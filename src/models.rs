use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PlayerState {
    Playing,
    Paused,
    Stopped,
    FastForwarding,
    Rewinding,
    Unknown,
}

impl<'a> Deserialize<'a> for PlayerState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let state = String::deserialize(deserializer)?;

        Ok(match state.as_str() {
            "playing" => PlayerState::Playing,
            "paused" => PlayerState::Paused,
            "stopped" => PlayerState::Stopped,
            "fastForwarding" => PlayerState::FastForwarding,
            "rewinding" => PlayerState::Rewinding,
            _ => PlayerState::Unknown,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Song {
    pub id: u32,
    pub name: String,
    pub artist: String,
    pub album: String,
    #[serde(rename = "albumArtist")]
    pub album_artist: String,
    pub year: u32,
    pub duration: f32,
    #[serde(rename = "playerPosition")]
    pub player_position: f32,
}

#[derive(Debug)]
pub struct Album {
    pub artwork: String,
    pub url: String,
}

impl Album {
    pub fn new(artwork: String, url: String) -> Self {
        Self {
            artwork: artwork.replace('"', ""),
            url: url.replace('"', ""),
        }
    }
}
