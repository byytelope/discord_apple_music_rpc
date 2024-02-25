mod models;
mod osascript;
mod utils;

use std::{thread, time::Duration};

use crate::{
    models::PlayerState,
    osascript::{get_album, get_current_song, get_is_open, get_player_state},
    utils::{current_time_as_u64, macos_ver, setup_logging, truncate},
};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    setup_logging(log::LevelFilter::Info).expect("Failed to initialize logs");

    log::info!("Starting RPC...");

    let client_id = std::env::var("CLIENT_ID")
        .map_err(|err| log::error!("{}", err))
        .unwrap()
        .parse::<u64>()
        .map_err(|err| log::error!("{}", err))
        .unwrap();

    let app_name = if macos_ver() >= 10.15 {
        "Music"
    } else {
        "iTunes"
    };

    log::info!("Waiting for Discord...");

    'main: loop {
        thread::sleep(Duration::from_secs(1));

        let is_discord_open = get_is_open("Discord");
        let is_music_open = get_is_open(app_name);

        if !is_music_open || !is_discord_open {
            continue;
        }

        let mut client = discord_presence::Client::new(client_id);

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

        client.start();
        client
            .block_until_event(discord_presence::Event::Ready)
            .map_err(|err| log::error!("{}", err))
            .unwrap();

        if !discord_presence::Client::is_ready() {
            continue 'main;
        }

        'player: loop {
            thread::sleep(Duration::from_secs(1));

            let player_state = get_player_state(app_name);
            log::info!("Player status: {:?}", player_state);

            if let PlayerState::Playing = player_state {
                if let Some(current_song) = get_current_song(app_name) {
                    log::info!("Currently playing: {:#?}", current_song);

                    let album_info = get_album(&current_song).await.unwrap();
                    log::info!("Album info: {:#?}", album_info);

                    client
                        .set_activity(|act| {
                            act.state(truncate(&current_song.artist, 128))
                                .details(truncate(&current_song.name, 128))
                                .timestamps(|stamp| {
                                    stamp.start(
                                        current_time_as_u64() - current_song.player_position as u64,
                                    )
                                })
                                .assets(|ass| {
                                    ass.small_image("apple_music_logo")
                                        .large_image(&album_info.artwork)
                                        .large_text(truncate(&current_song.album, 128))
                                })
                                .append_buttons(|butt| {
                                    let url = if !album_info.url.is_empty() {
                                        &album_info.url
                                    } else {
                                        "https://music.apple.com/"
                                    };

                                    butt.label("Listen on Apple Music").url(url)
                                })
                        })
                        .map_err(|err| {
                            log::error!("{}", err);
                        })
                        .unwrap();
                } else {
                    continue 'player;
                }
            } else if get_is_open("Discord") {
                let client_err = client
                    .clear_activity()
                    .map_err(|err| log::error!("{}", err))
                    .is_err();

                if client_err {
                    break 'player;
                }
            } else {
                break 'player;
            }

            continue;
        }
    }
}
