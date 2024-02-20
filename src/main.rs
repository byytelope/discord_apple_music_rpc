use std::{
    thread,
    time::{Duration, SystemTime},
};

use discord_rich_presence::{
    activity::{Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};
use dotenv::dotenv;
use fern::{log_file, Dispatch, InitError};
use http_cache_surf::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use log::{info, LevelFilter};
use osascript::{self, JavaScript};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::{Deserialize, Serialize};
use sysinfo::System;

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

#[derive(Debug)]
enum PlayerState {
    Playing,
    Paused,
    Stopped,
    Unknown,
}

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

fn get_player_state(app_name: &str) -> PlayerState {
    let script = {
        let apple_script = format!("return Application('{}').playerState();", app_name);
        JavaScript::new(apple_script.as_str())
    };

    let state_str = script.execute::<String>().unwrap();

    match state_str.as_str() {
        "playing" => PlayerState::Playing,
        "paused" => PlayerState::Paused,
        "stopped" => PlayerState::Stopped,
        _ => PlayerState::Unknown,
    }
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

fn setup_logging(verbosity: LevelFilter) -> Result<(), InitError> {
    let base_config = Dispatch::new().level(verbosity);
    let file_config = Dispatch::new()
        .format(|out, msg, rec| {
            out.finish(format_args!(
                "{} [{}::{}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                rec.target(),
                rec.level(),
                msg
            ))
        })
        .chain(log_file(
            "/Users/mohamedshadhaan/Library/Logs/discord_apple_music_rpc.log",
        )?);

    // let stdout_config = fern::Dispatch::new()
    //     .format(|out, message, record| {
    //         if record.level() > log::LevelFilter::Info && record.target() == "cmd_program" {
    //             out.finish(format_args!(
    //                 "DEBUG @ {}: {}",
    //                 humantime::format_rfc3339_seconds(SystemTime::now()),
    //                 message
    //             ))
    //         } else {
    //             out.finish(format_args!(
    //                 "[{} {} {}] {}",
    //                 humantime::format_rfc3339_seconds(SystemTime::now()),
    //                 record.level(),
    //                 record.target(),
    //                 message
    //             ))
    //         }
    //     })
    //     .chain(std::io::stdout());

    base_config
        .chain(file_config)
        // .chain(stdout_config)
        .apply()?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    setup_logging(LevelFilter::Info).expect("Failed to initialize logs.");

    println!("Starting RPC...");

    let client_id = std::env::var("CLIENT_ID").expect("Client ID not set.");

    let app_name = if macos_ver() >= 10.15 {
        "Music"
    } else {
        "iTunes"
    };

    info!("Waiting for Discord...");

    loop {
        let is_discord_open = get_is_open("Discord");
        let is_music_open = get_is_open(app_name);

        if !is_music_open || !is_discord_open {
            continue;
        }

        match DiscordIpcClient::new(&client_id) {
            Ok(mut client) => {
                client.connect().expect("Failed to connect to client.");

                loop {
                    let player_state = get_player_state(app_name);
                    info!("Player status: {:?}", player_state);

                    if let PlayerState::Playing = player_state {
                        let current_song = get_current_song(app_name);
                        info!("Currently playing: {:#?}", current_song);

                        let album_info = get_album(&current_song).await.unwrap();
                        info!("Album info: {:#?}", album_info);

                        let assets = Assets::new()
                            .small_image("apple_music_logo")
                            .large_image(&album_info.artwork)
                            .large_text(&current_song.album);
                        let buttons: Vec<Button> = {
                            if album_info.url.is_empty() {
                                vec![]
                            } else {
                                vec![Button::new("Listen on Apple Music", &album_info.url)]
                            }
                        };
                        client
                            .set_activity(
                                Activity::new()
                                    .state(truncate(&current_song.artist, 128))
                                    .details(truncate(&current_song.name, 128))
                                    .assets(assets)
                                    .buttons(buttons),
                            )
                            .expect("Failed to set activity.");
                    } else {
                        client.clear_activity().expect("Failed to clear activity.");
                    }

                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            }
            Err(err) => {
                eprintln!("{}", err)
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}
