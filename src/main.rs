use std::{thread, time::Duration};

use discord_rich_presence::{
    activity::{Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};
use dotenv::dotenv;
use http_cache_surf::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use osascript::{self, JavaScript};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::{Deserialize, Serialize};
use sysinfo::System;

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

#[derive(Serialize, Deserialize, Debug)]
struct Song {
    id: u32,
    name: String,
    artist: String,
    album: String,
    year: u32,
    duration: f32,
    #[serde(rename = "playerPosition")]
    player_position: f32,
}

#[derive(Debug)]
struct Album {
    artwork: String,
    url: String,
}

impl Album {
    fn new(artwork: String, url: String) -> Self {
        Self {
            artwork: artwork.replace('"', ""),
            url: url.replace('"', ""),
        }
    }
}

fn truncate(text: &str, max_length: usize) -> &str {
    match text.char_indices().nth(max_length) {
        Some((idx, _)) => &text[..idx],
        None => text,
    }
}

fn macos_ver() -> f32 {
    System::os_version()
        .unwrap()
        .rsplit_once('.')
        .unwrap()
        .0
        .parse::<f32>()
        .unwrap()
}

fn get_is_open(app_name: &str) -> bool {
    let script = {
        let apple_script = format!(
            "return Application('System Events').processes['{}'].exists();",
            app_name
        );
        JavaScript::new(apple_script.as_str())
    };

    script.execute().unwrap()
}

fn get_player_state(app_name: &str) -> String {
    let script = {
        let apple_script = format!("return Application('{}').playerState();", app_name);
        JavaScript::new(apple_script.as_str())
    };

    script.execute::<String>().unwrap()
}

fn get_current_song(app_name: &str) -> Song {
    let script = {
        let apple_script = format!(
            "const music = Application('{}');
        return {{
          ...music.currentTrack().properties(),
          playerPosition: music.playerPosition(),
        }};",
            app_name
        );
        JavaScript::new(apple_script.as_str())
    };

    script.execute::<Song>().unwrap()
}

async fn get_album(song_info: &Song) -> surf::Result<Album> {
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
    let obj = res.get("results").unwrap().get(0).unwrap();
    let artwork = obj.get("artworkUrl100");
    let url = obj.get("collectionViewUrl");

    Ok(Album::new(
        match artwork.is_none() {
            true => "apple_music_logo".to_string(),
            false => artwork.unwrap().to_string(),
        },
        match url.is_none() {
            true => "".to_string(),
            false => url.unwrap().to_string(),
        },
    ))
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client_id = std::env::var("CLIENT_ID");
    let mut client =
        DiscordIpcClient::new(&client_id.expect("Client ID not set.")).expect("Invalid client ID.");
    client.connect().expect("Client failed to connect.");

    let app_name = if macos_ver() >= 10.15 {
        "Music"
    } else {
        "iTunes"
    };

    loop {
        let is_discord_open = get_is_open("Discord");
        let is_music_open = get_is_open(app_name);

        if !is_music_open || !is_discord_open {
            break;
        }

        let player_state = get_player_state(app_name);
        let current_song = get_current_song(app_name);
        let album_info = get_album(&current_song).await.unwrap();

        println!("{} is open: {}", app_name, is_music_open);
        println!("Player status: {}", player_state);
        println!("Currently playing: {:#?}", current_song);
        println!("Album info: {:#?}", album_info);

        let assets = Assets::new()
            .large_image(&album_info.artwork)
            .large_text(&current_song.album);
        let buttons: Vec<Button> = vec![Button::new("Listen on Apple Music", &album_info.url)];
        client
            .set_activity(
                Activity::new()
                    .state(truncate(&current_song.artist, 128))
                    .details(truncate(&current_song.name, 128))
                    .assets(assets)
                    .buttons(buttons),
            )
            .expect("Failed to set activity.");

        thread::sleep(Duration::from_secs(1));
    }

    // client.close().expect("Failed to close client.");
}
