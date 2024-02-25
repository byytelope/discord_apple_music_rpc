use std::process::Command;

use http_cache_surf::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use percent_encoding::utf8_percent_encode;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    models::{Album, PlayerState, Song},
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

pub async fn get_album(song_info: &Song) -> surf::Result<Album> {
    let query = format!("{} {}", song_info.artist, song_info.album);
    let encoded_query = utf8_percent_encode(query.as_str(), FRAGMENT).collect::<String>();
    let entity = match encoded_query.find("%26") {
        None => "album",
        Some(_) => "song",
    };

    let url = format!(
        "https://itunes.apple.com/search?media=music&entity={}&limit=1&term={}",
        entity, encoded_query
    );

    let res = surf::client()
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: MokaManager::default(),
            options: HttpCacheOptions::default(),
        }))
        .recv_json::<serde_json::Value>(surf::get(url))
        .await?;

    let obj_arr = res.get("results").unwrap();

    if let Some(obj) = obj_arr.get(0) {
        let artwork = obj
            .get("artworkUrl100")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "no_art".to_string());

        let url = obj
            .get("collectionViewUrl")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "".to_string());

        Ok(Album::new(artwork, url))
    } else {
        Ok(Album::new("no_art".to_string(), "".to_string()))
    }
}
