use std::time::Instant;
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

use crate::installer::get_yt_dlp_path;
use crate::models::music::Music;
use crate::models::translation::Translations;

pub fn get_video_info(url: &str) -> Result<Music, Box<dyn std::error::Error>> {
    println!("{}", Translations::t("fetching_video_info", Some(&[url])));

    let mut youtube_dl = YoutubeDl::new(url);

    let yt_dlp_path = get_yt_dlp_path().unwrap();
    youtube_dl.youtube_dl_path(yt_dlp_path);
    // Create and run YoutubeDl with explicit path
    youtube_dl.socket_timeout("15");

    let output = youtube_dl.run()?;

    let title = match output {
        YoutubeDlOutput::SingleVideo(video) => video.title,
        YoutubeDlOutput::Playlist(playlist) => playlist.title,
    };

    Ok(Music {
        url: url.to_string(),
        title,
        downloaded_at: None,
    })
}

pub fn fetch_playlist_videos(url: &str) -> Result<Vec<Music>, Box<dyn std::error::Error>> {
    // Check if yt-dlp is installed, if not try to install it
    let yt_dlp_path = match get_yt_dlp_path() {
        Some(path) => path,
        None => {
            println!("yt-dlp is not installed or not found. Attempting to install...");
            crate::installer::install_yt_dlp()?;

            // Verify installation was successful
            match get_yt_dlp_path() {
                Some(path) => {
                    println!("yt-dlp installed successfully.");
                    path
                }
                None => {
                    return Err(
                        "yt-dlp was not found after installation. Please install it manually."
                            .into(),
                    )
                }
            }
        }
    };

    // Create and run YoutubeDl with explicit path
    let mut youtube_dl = YoutubeDl::new(url);
    youtube_dl.youtube_dl_path(yt_dlp_path);
    youtube_dl.flat_playlist(true);
    youtube_dl.socket_timeout("15");

    let output = youtube_dl.run()?;

    let mut videos = Vec::new();

    if let Some(playlist) = output.clone().into_playlist() {
        println!(
            "Found playlist: {}",
            playlist.title.unwrap_or_else(|| "Unknown".to_string())
        );

        if let Some(entries) = playlist.entries {
            println!("Number of videos in playlist: {}", entries.len());
            for video in entries {
                videos.push(Music {
                    url: video.url.unwrap_or_else(|| "Unknown".to_string()),
                    title: video.title,
                    downloaded_at: None,
                });
            }
        } else {
            return Err("No videos found in playlist".into());
        }
    } else {
        // It's a single video
        let video_info = get_video_info(url)?;
        videos.push(video_info);
    }

    Ok(videos)
}

pub fn download_video(video: &Music, download_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let title = video.title.clone().unwrap_or_else(|| "Unknown".to_string());
    println!("{}", Translations::t("video_downloading", Some(&[&title])));

    let start_time = Instant::now();

    // Check if yt-dlp is installed, if not try to install it
    let yt_dlp_path = match get_yt_dlp_path() {
        Some(path) => path,
        None => {
            println!("yt-dlp is not installed or not found. Attempting to install...");
            crate::installer::install_yt_dlp()?;

            // Verify installation was successful
            match get_yt_dlp_path() {
                Some(path) => {
                    println!("yt-dlp installed successfully.");
                    path
                }
                None => {
                    return Err(
                        "yt-dlp was not found after installation. Please install it manually."
                            .into(),
                    )
                }
            }
        }
    };

    // Create and run YoutubeDl with explicit path
    let mut youtube_dl = YoutubeDl::new(&video.url);
    youtube_dl.youtube_dl_path(yt_dlp_path);
    youtube_dl.extract_audio(true);
    youtube_dl.extra_arg("--audio-format");
    youtube_dl.extra_arg("mp3");
    youtube_dl.extra_arg("--embed-metadata");
    youtube_dl.extra_arg("--embed-thumbnail");
    youtube_dl.output_template("%(title)s.%(ext)s");
    let _ = youtube_dl.download_to(download_dir);

    let status = youtube_dl.run();

    match status {
        Ok(_) => {
            let duration = start_time.elapsed();
            println!(
                "{}",
                Translations::t(
                    "video_downloaded",
                    Some(&[&title, &duration.as_secs().to_string()])
                )
            );
            Ok(())
        }
        Err(e) => {
            println!(
                "{}",
                Translations::t("video_download_failed", Some(&[&title, &e.to_string()]))
            );
            Err(e.into())
        }
    }
}
