use crate::utils::StripTrailingNewline;
use anyhow::{anyhow, Context, Result};
use id3::TagLike;
use itertools::Itertools;
use std::process::{Command, Stdio};

pub struct YouTube {
    pub command: String,
    pub datadir: String,
    pub version: String,
}

#[derive(Debug)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub channel: String,
}

impl YouTube {
    pub fn new(command: String, datadir: String) -> Result<YouTube> {
        let output = Command::new(&command)
            .arg("--version")
            .output()
            .with_context(|| format!("Cannot find command {}", &command))?;
        let version = String::from_utf8(output.stdout)?.strip_trailing_newline();

        Ok(YouTube {
            command,
            version,
            datadir,
        })
    }

    pub fn get_playlist(&mut self, url: &str) -> Result<Vec<Video>> {
        let output = Command::new(&self.command)
            .arg("--get-id")
            .arg("--get-title")
            .arg("--print=channel")
            .arg("--flat-playlist")
            .arg(url)
            .output()?;

        let str_output = String::from_utf8(output.stdout)?;
        let videos = str_output
            .lines()
            .tuples()
            .map(|(channel, title, id)| Video {
                id: id.to_string(),
                title: title.to_string(),
                channel: channel.to_string(),
            })
            .collect_vec();

        Ok(videos)
    }

    pub fn download_video(&self, video: &Video) -> Result<()> {
        let status = Command::new(&self.command)
            .arg("-f bestaudio")
            .arg("--extract-audio")
            .arg("--audio-quality=0")
            .arg("--audio-format=mp3")
            .arg("--concurrent-fragments=8")
            .arg(format!("--output={}/%(id)s", &self.datadir))
            .arg(video.url())
            .stdout(Stdio::null())
            .status()?;

        if !status.success() {
            return Err(anyhow!("{} exited: {}", self.command, status));
        }

        let mut tag = id3::Tag::new();
        tag.set_title(video.title.clone());
        tag.set_artist(video.channel.clone());
        tag.write_to_path(video.path(&self.datadir), id3::Version::Id3v24)?;

        Ok(())
    }
}

impl Video {
    pub fn url(&self) -> String {
        format!("https://youtube.com/watch?v={}", self.id)
    }

    pub fn path(&self, data_dir: &String) -> String {
        format!("{}/{}.mp3", data_dir, self.id)
    }
}
