use std::process::Command;

use http_cache_surf::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use percent_encoding::utf8_percent_encode;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    models::{ApiResults, PlayerState, Song, SongDetails},
    utils::FRAGMENT,
};

fn run_osascript<T: DeserializeOwned>(script: String) -> Result<T, serde_json::Error> {
    let function = format!("(() => JSON.stringify({}))();", script);
    let output = Command::new("osascript")
        .arg("-l")
        .arg("JavaScript")
        .arg("-e")
        .arg(&function)
        .output()
        .map_err(|err| log::error!("{}", err))
        .unwrap()
        .stdout;
    let res = String::from_utf8_lossy(&output).to_string();

    serde_json::from_str(&res)
}

pub fn get_is_open(app_name: &str) -> bool {
    let script = format!(
        "Application('System Events').processes['{}'].exists()",
        app_name
    );

    run_osascript(script)
        .map_err(|err| log::error!("{}", err))
        .unwrap()
}

pub fn get_player_state(app_name: &str) -> PlayerState {
    let script = format!("Application('{}').playerState()", app_name);
    run_osascript(script)
        .map_err(|err| log::error!("{}", err))
        .unwrap()
}

pub fn get_current_song(app_name: &str) -> Option<Song> {
    let script = format!(
        "{{
          ...Application('{0}').currentTrack().properties(),
          playerPosition: Application('{0}').playerPosition(),
        }}",
        app_name
    );

    match run_osascript::<Value>(script) {
        Ok(val) => {
            if let Some(album) = val.get("album").and_then(|album| album.as_str()) {
                if !album.is_empty() {
                    return serde_json::from_value::<Song>(val)
                        .map_err(|err| log::error!("{}", err))
                        .ok();
                }
            }
        }
        Err(err) => log::error!("{}", err),
    }

    None
}

pub async fn get_details(song_info: &Song) -> surf::Result<SongDetails> {
    let main_url =
        "https://itunes.apple.com/search?media=music&entity={entity}&limit=3&term={term}";
    let song_query = format!("{} {}", song_info.artist.replace('&', ""), song_info.name);

    // Searching without '*' is more accurate
    let encoded_song_query = utf8_percent_encode(&song_query, FRAGMENT)
        .collect::<String>()
        .replace('*', "");

    let song_url = main_url
        .replace("{entity}", "song")
        .replace("{term}", &encoded_song_query);

    let song_res = surf::client()
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: MokaManager::default(),
            options: HttpCacheOptions::default(),
        }))
        .recv_json::<ApiResults>(surf::get(song_url))
        .await?;

    if song_res.result_count == 0 {
        let album_artist = if !song_info.album_artist.is_empty() {
            song_info.album_artist.to_string()
        } else {
            song_info
                .artist
                .split_once(',')
                .map(|(first, _)| first)
                .or_else(|| song_info.artist.split_once('&').map(|(first, _)| first))
                .unwrap_or(&song_info.artist)
                .trim()
                .to_string()
        };

        let album_query = format!("{} {}", album_artist, song_info.album);

        // Searching without '*' is more accurate
        let encoded_album_query = utf8_percent_encode(album_query.as_str(), FRAGMENT)
            .collect::<String>()
            .replace('*', "");

        let album_url = main_url
            .replace("{entity}", "album")
            .replace("{term}", &encoded_album_query);

        let album_res = surf::client()
            .with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: MokaManager::default(),
                options: HttpCacheOptions::default(),
            }))
            .recv_json::<ApiResults>(surf::get(album_url))
            .await?;

        if album_res.result_count > 0 {
            let album = &album_res.results[0];
            let artwork = if !album.artwork_url.is_empty() {
                album.artwork_url.to_string()
            } else {
                "no_art".to_string()
            };
            Ok(SongDetails::new(
                artwork,
                album.album_url.to_string(),
                album.album_url.to_string(),
            ))
        } else {
            Ok(SongDetails::new(
                "no_art".to_string(),
                "".to_string(),
                "".to_string(),
            ))
        }
    } else {
        Ok(SongDetails::new(
            song_res.results[0].artwork_url.to_string(),
            song_res.results[0].album_url.to_string(),
            song_res.results[0].song_url.clone().unwrap_or_default(),
        ))
    }
}
