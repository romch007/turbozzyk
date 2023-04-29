extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub mod utils;
pub mod youtube;

use anyhow::Result;
use std::{env, fs, path::Path};
use youtube::YouTube;

const ZYKK_PLAYLIST_URL: &str =
    "https://www.youtube.com/playlist?list=PL8-0BGNjDQR5DfYVRduznf9b404uYXtu5";

fn ensure_data_dir(dirname: &String) -> Result<()> {
    if !Path::new(dirname).exists() {
        info!("Data dir '{}' does not exist, creating it...", dirname);
        fs::create_dir(dirname)?;
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
