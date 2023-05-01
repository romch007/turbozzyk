extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub mod utils;
pub mod youtube;

use anyhow::Result;
use id3::TagLike;
use std::{env, fs, path::Path};
use youtube::YouTube;

const ZYKK_PLAYLIST_URL: &str =
    "https://www.youtube.com/playlist?list=PL8-0BGNjDQR5DfYVRduznf9b404uYXtu5";

fn ensure_data_dir(dirname: &String) -> Result<()> {
    if !Path::new(dirname).exists() {
        info!("Data dir '{}' does not exist, creating it...", dirname);
        fs::create_dir(dirname)?;
    } else {
        cleanup_data_dir(dirname)?;
    }

    Ok(())
}

fn cleanup_data_dir(dirname: &String) -> Result<()> {
    info!("Cleaning up data dir...");
    for file in fs::read_dir(dirname)? {
        let file = file?;
        let maybe_filename = file.file_name().into_string();
        let path = file.path();
        if let Ok(filename) = maybe_filename {
            // Remove file if its not an audio file
            if !filename.ends_with(".mp3") {
                fs::remove_file(&path)?;
                continue;
            }

            // Partially downloaded files do not have ID3 metadatas
            let should_remove_file = match id3::Tag::read_from_path(&path) {
                Ok(tag) => tag.title().is_none(),
                Err(id3::Error {
                    kind: id3::ErrorKind::NoTag,
                    ..
                }) => true,
                Err(e) => return Err(anyhow::Error::from(e)),
            };

            if should_remove_file {
                fs::remove_file(&path)?;
            }
        }
    }

    Ok(())
}

fn start_app() -> Result<()> {
    info!("Starting app...");
    let command = env::var("COMMAND").unwrap_or_else(|_| "yt-dlp".to_owned());
    let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "data".to_owned());

    ensure_data_dir(&data_dir)?;

    let mut client = YouTube::new(command, data_dir)?;

    info!("{} version {} found!", client.command, client.version);
    info!("Fetching videos of playlist...");

    let videos = client.get_playlist(ZYKK_PLAYLIST_URL)?;

    info!("Done.");
    let video_count = videos.len();
    let mut i = 0;
    for target_video in &videos {
        i += 1;
        if Path::new(&target_video.path(&client.datadir)).exists() {
            info!(
                "({}/{}) '{}' by '{}' already downloaded, skipping...",
                i, video_count, target_video.title, target_video.channel
            );
            continue;
        }

        info!(
            "({}/{}) Downloading video '{}' by '{}'...",
            i, video_count, target_video.title, target_video.channel
        );
        let res = client.download_video(target_video);
        if let Err(err) = res {
            error!("Failed to download video, reason: {}", err.to_string());
        }
    }

    Ok(())
}

fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    if let Err(e) = start_app() {
        error!("{}", e.to_string());
    }
}
