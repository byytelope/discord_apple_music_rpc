use std::{process::Command, thread, time::Duration};

use discord_rich_presence::{
    activity::{Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};
use osascript::JavaScript;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::{Deserialize, Serialize};

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

#[derive(Debug, Serialize, Deserialize)]
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
        let artwork = artwork.replace('"', " ").trim().to_string();
        let url = url.replace('"', " ").trim().to_string();
        Self { artwork, url }
    }
}

struct MusicDetails {
    open_state: bool,
    player_state: String,
    song: Song,
    album: Album,
}

enum Error {
    FailedLmao,
    FailedOhNo,
}

fn get_macos() -> f32 {
    let sw_vers = Command::new("sw_vers")
        .output()
        .expect("Failed to run sw_vers");

    let os_release = String::from_utf8(sw_vers.stdout).expect("Failed to parse stdout to string");
    let product_version = os_release
        .lines()
        .find(|line| line.trim().starts_with("ProductVersion:"))
        .and_then(|line| {
            let parts: Vec<&str> = line.split(':').collect();
            parts.get(1).map(|version| version.trim())
        })
        .expect("ProductVersion not found");
    let mut version_parts = product_version.split('.').take(2).map(|part| {
        part.parse::<f32>()
            .expect("Failed to parse version to floating")
    });

    let major_version = version_parts
        .next()
        .expect("Failed to parse major macos version");
    let minor_version = version_parts
        .next()
        .expect("Failed to parse minor macos version");
    major_version + minor_version * 0.1
}

fn get_open(name: &str) -> bool {
    let script = {
        let aplscrpt = format!(
            "return Application('System Events').processes['{}'].exists();",
            name
        );
        JavaScript::new(aplscrpt.as_str())
    };
    script
        .execute()
        .map_err(|_| println!("Failed to get {} state", name))
        .unwrap()
}

fn get_state(name: &str) -> String {
    let script = {
        let code_str = format!("return Application('{}').playerState();", name);
        JavaScript::new(code_str.as_str())
    };
    script
        .execute()
        .map_err(|_| println!("Failed to get state of app: {name}"))
        .unwrap()
}

fn get_current_song(name: &str) -> Result<Song, Error> {
    let script = {
        let code_str = format!(
            "const music = Application('{}');
            return {{
                ...music.currentTrack().properties(),
                playerPosition: music.playerPosition(),
            }};",
            name
        );
        JavaScript::new(code_str.as_str())
    };
    let song = script.execute::<Song>().map_err(|err| {
        println!("Failed to get music details: {:?}", err);
        Error::FailedLmao
    })?;
    Ok(song)
}

async fn get_album(song: &Song) -> Result<Album, Error> {
    let query = format!("{} {}", song.artist, song.album);
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
    let res = reqwest::get(url)
        .await
        .map_err(|_| {
            println!("Apple Music Lookup fucked for: {}", &song.name);
            Error::FailedLmao
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| Error::FailedLmao)?;
    let obj = res
        .get("results")
        .expect("Failed to get results")
        .get(0)
        .map(Result::Ok)
        .unwrap_or_else(|| Err(Error::FailedLmao))?;
    let artwork = obj.get("artworkUrl100");
    let url = obj.get("collectionViewUrl");

    let artwork = if artwork.is_none() {
        "apple_music_logo".to_string()
    } else {
        #[allow(clippy::unnecessary_unwrap)]
        artwork.unwrap().to_string()
    };
    let url = if url.is_none() {
        "".to_string()
    } else {
        #[allow(clippy::unnecessary_unwrap)]
        url.unwrap().to_string()
    };
    Ok(Album::new(artwork, url))
}

fn truncate(text: &str, max_length: usize) -> &str {
    match text.char_indices().nth(max_length) {
        Some((idx, _)) => &text[..idx],
        None => text,
    }
}

async fn get_client() -> Result<DiscordIpcClient, Error> {
    let mut client = DiscordIpcClient::new("1208073237347438702").expect("Invalid client ID.");
    client.connect().map_err(|_| Error::FailedOhNo)?;
    Ok(client)
}

async fn load(song: &Song, album: &Album, client: &mut DiscordIpcClient) -> Result<(), Error> {
    let assets = Assets::new()
        .large_image(&album.artwork)
        .large_text(&song.album);
    let buttons: Vec<Button> = vec![Button::new("Listen on Apple Music", &album.url)];

    client
        .set_activity(
            Activity::new()
                .state(truncate(&song.artist, 128))
                .details(truncate(&song.name, 128))
                .buttons(buttons)
                .assets(assets),
        )
        .map_err(|_| Error::FailedOhNo)?;
    Ok(())
}

async fn fetch_song_and_load(
    name: &str,
    client: &mut DiscordIpcClient,
) -> Result<MusicDetails, Error> {
    let open_state = get_open(name);
    let player_state = get_state(name);
    let song = get_current_song(name)?;
    let album = get_album(&song).await?;
    load(&song, &album, client).await?;
    Ok(MusicDetails {
        open_state,
        player_state,
        song,
        album,
    })
}

#[tokio::main]
async fn main() {
    let app = if get_macos() >= 10.15 {
        "Music"
    } else {
        "iTunes"
    };

    let mut client;

    loop {
        loop {
            let state = get_open("Discord");
            if state {
                client = match get_client().await {
                    Ok(client) => client,
                    Err(_) => continue,
                };
                break;
            } else {
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        }

        'while_connected: loop {
            let mut prev_track = "not_initialized".to_string();
            while get_open(app) && get_state(app).contains("playing") && get_open("Discord") {
                let music = match fetch_song_and_load(app, &mut client).await {
                    Ok(music) => music,
                    Err(err) => match err {
                        Error::FailedLmao => continue,
                        Error::FailedOhNo => break 'while_connected,
                    },
                };
                let currect_track = music.song.name.clone();

                if currect_track != prev_track {
                    println!("{} is open: {}", app, music.open_state);
                    println!("Player status: {}", music.player_state);
                    println!("Currently playing: {:#?}", music.song);
                    println!("Album info: {:#?}", music.album);
                }
                thread::sleep(Duration::from_secs(2));
                prev_track = currect_track.clone();
                if !get_open(app) && get_open("Discord") {
                    client.clear_activity().unwrap();
                }
            }
        }
    }
}
