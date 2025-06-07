use std::{sync::OnceLock, time::Duration};

use http_cache_surf::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use percent_encoding::utf8_percent_encode;

use crate::core::{
    constants::BUNDLE_ID,
    models::{ApiResults, Song, SongDetails},
    utils::FRAGMENT,
};

static HTTP_CLIENT: OnceLock<surf::Client> = OnceLock::new();

pub async fn get_details(song_info: &Song) -> surf::Result<SongDetails> {
    if let Some(song_details) = search_song(song_info).await? {
        return Ok(song_details);
    }

    // Fallback to album search if no song details were found
    search_album(song_info).await
}

fn get_http_client() -> &'static surf::Client {
    HTTP_CLIENT.get_or_init(|| {
        let cache_dir = std::env::temp_dir().join(BUNDLE_ID);
        let cache_options = HttpCacheOptions {
            cache_options: Some(http_cache_surf::CacheOptions {
                immutable_min_time_to_live: Duration::from_secs(604800), // 1 week
                ..Default::default()
            }),
            ..Default::default()
        };
        let cache = Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager { path: cache_dir },
            options: cache_options,
        });

        surf::client().with(cache)
    })
}

async fn search_song(song_info: &Song) -> surf::Result<Option<SongDetails>> {
    let query = format!(
        "{} {} {}",
        song_info.artist.replace('&', ""),
        song_info.name,
        song_info.album
    );
    let results = search_itunes("song", &query).await?;

    if results.result_count > 0 {
        let song = &results.results[0];
        Ok(Some(SongDetails::new(
            song.artwork_url.to_string(),
            song.album_url.to_string(),
            song.song_url.clone().unwrap_or_default(),
        )))
    } else {
        Ok(None)
    }
}

async fn search_album(song_info: &Song) -> surf::Result<SongDetails> {
    let album_artist = get_primary_artist(song_info);
    let query = format!("{} {}", album_artist, song_info.album);
    let results = search_itunes("album", &query).await?;

    if results.result_count > 0 {
        let album = &results.results[0];
        let artwork = if album.artwork_url.is_empty() {
            "no_art".to_string()
        } else {
            album.artwork_url.to_string()
        };

        Ok(SongDetails::new(
            artwork,
            album.album_url.to_string(),
            album.album_url.to_string(),
        ))
    } else {
        Ok(SongDetails::new(
            "no_art".to_string(),
            String::new(),
            String::new(),
        ))
    }
}

async fn search_itunes(entity: &str, query: &str) -> surf::Result<ApiResults> {
    let encoded_query = utf8_percent_encode(query, FRAGMENT)
        .collect::<String>()
        .replace('*', "");
    let url = format!(
        "https://itunes.apple.com/search?media=music&entity={}&limit=1&term={}",
        entity, encoded_query
    );

    log::debug!("Searching iTunes: {}", url);

    get_http_client()
        .recv_json::<ApiResults>(surf::get(url))
        .await
}

fn get_primary_artist(song_info: &Song) -> String {
    if !song_info.album_artist.is_empty() {
        return song_info.album_artist.to_string();
    }

    song_info
        .artist
        .split_once(',')
        .map(|(first, _)| first)
        .or_else(|| song_info.artist.split_once('&').map(|(first, _)| first))
        .unwrap_or(&song_info.artist)
        .trim()
        .to_string()
}
