use std::thread;
use std::time::Duration;

use discord_rich_presence::{
    activity::{Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};
use osascript::{self, JavaScript};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt};

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

fn truncate(text: &str, max_length: usize) -> &str {
    match text.char_indices().nth(max_length) {
        Some((idx, _)) => &text[..idx],
        None => text,
    }
}

fn macos_ver() -> f32 {
    let sys = System::new_all();
    sys.os_version().unwrap().parse::<f32>().unwrap()
}

fn get_is_open(app_name: &str) -> bool {
    let is_open_script = {
        let code_str = format!(
            "return Application('System Events').processes['{}'].exists();",
            app_name
        );
        JavaScript::new(code_str.as_str())
    };

    is_open_script.execute().unwrap()
}

fn get_state(app_name: &str) -> String {
    let player_state_script = {
        let code_str = format!("return Application('{}').playerState();", app_name);
        JavaScript::new(code_str.as_str())
    };

    player_state_script.execute::<String>().unwrap()
}

fn get_current_song(app_name: &str) -> Song {
    let song_details_script = {
        let code_str = format!(
            "const music = Application('{}');
        return {{
          ...music.currentTrack().properties(),
          playerPosition: music.playerPosition(),
        }};",
            app_name
        );
        JavaScript::new(code_str.as_str())
    };

    song_details_script.execute::<Song>().unwrap()
}

async fn get_album(song_info: &Song) -> Result<Album, reqwest::Error> {
    let query = format!("{} {}", song_info.artist, song_info.album);
    let encoded_query = utf8_percent_encode(query.as_str(), FRAGMENT).collect::<String>();
    let entity = {
        let _entity = encoded_query.find("%26");
        if _entity.is_none() {
            "album"
        } else {
            "song"
        }
    };
    let url = format!(
        "https://itunes.apple.com/search?media=music&entity={}&limit=1&term={}",
        entity, encoded_query
    );
    let res = reqwest::get(url).await?.json::<serde_json::Value>().await?;
    let obj = res.get("results").unwrap().get(0).unwrap();
    let artwork = obj.get("artworkUrl100");
    let url = obj.get("collectionViewUrl");

    Ok(Album {
        artwork: if artwork.is_none() {
            "apple_music_logo".to_string()
        } else {
            artwork.unwrap().to_string()
        },
        url: if url.is_none() {
            "".to_string()
        } else {
            url.unwrap().to_string()
        },
    })
}

fn start(current_song: Song, album_info: &Album) {
    let assets = Assets::new()
        .large_image(&album_info.artwork)
        .large_text(&current_song.album);
    let buttons: Vec<Button> = vec![Button::new("Listen on Apple Music", &album_info.url)];

    let mut client = DiscordIpcClient::new("996864734957670452").expect("Invalid client ID.");
    client.connect().expect("Client failed to connect.");
    client
        .set_activity(
            Activity::new()
                .state(truncate(&current_song.artist, 128))
                .details(truncate(&current_song.name, 128))
                .buttons(buttons)
                .assets(assets),
        )
        .expect("Failed to set activity.");

    thread::sleep(Duration::from_secs(10));
    client.close().expect("Failed to close client.");
}

#[tokio::main]
async fn main() {
    let app_name = if macos_ver() >= 10.15 {
        "Music"
    } else {
        "iTunes"
    };

    let is_open = get_is_open(app_name);
    let player_state = get_state(app_name);
    let current_song = get_current_song(app_name);
    let album_info = get_album(&current_song).await.unwrap();

    println!("{} is open: {}", app_name, is_open);
    println!("Player status: {}", player_state);
    println!("Currently playing: {:#?}", current_song);
    println!("Album info: {:#?}", album_info);

    start(current_song, &album_info);
}
