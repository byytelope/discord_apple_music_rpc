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
pub struct ApiResults {
    #[serde(rename = "resultCount")]
    pub result_count: u32,
    pub results: Vec<ApiResult>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResult {
    #[serde(rename = "wrapperType")]
    pub wrapper_type: String,
    #[serde(rename = "artistName")]
    pub artist_name: String,
    #[serde(rename = "collectionName")]
    pub album_name: String,
    #[serde(rename = "artworkUrl100")]
    pub artwork_url: String,
    #[serde(rename = "collectionViewUrl")]
    pub album_url: String,
    #[serde(rename = "artistViewUrl")]
    pub artist_url: Option<String>,
    #[serde(rename = "trackViewUrl")]
    pub song_url: Option<String>,
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
pub struct SongDetails {
    pub artwork: String,
    pub album_url: String,
    pub song_url: String,
}

impl SongDetails {
    pub fn new(artwork: String, album_url: String, song_url: String) -> Self {
        Self {
            artwork: artwork.replace('"', ""),
            album_url: album_url.replace('"', ""),
            song_url: song_url.replace('"', ""),
        }
    }
}
