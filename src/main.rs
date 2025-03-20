use rayon::prelude::*;
use serde::Deserialize;
use youtube_dl::YoutubeDl;
use std::process::Command;
use std::io::{self, Write, BufRead};
use std::sync::Mutex;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::collections::VecDeque;

#[derive(Debug, Deserialize)]
struct Music {
    url: String,
}

fn check_ffmpeg() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_ok()
}

fn check_yt_dlp() -> bool {
    Command::new("yt-dlp")
        .arg("--version")
        .output()
        .is_ok()
}

fn install_ffmpeg() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        println!("Installing ffmpeg via winget...");
        let status = Command::new("winget")
            .args(&["install", "ffmpeg"])
            .status()?;

        if !status.success() {
            return Err("Failed to install ffmpeg".into());
        }
    }

    #[cfg(target_os = "linux")]
    {
        println!("Installing ffmpeg via package manager...");
        if Command::new("apt")
            .args(&["install", "-y", "ffmpeg"])
            .status()
            .is_err()
        {
            Command::new("yum")
                .args(&["install", "-y", "ffmpeg"])
                .status()?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        println!("Installing ffmpeg via Homebrew...");
        Command::new("brew")
            .args(&["install", "ffmpeg"])
            .status()?;
    }

    Ok(())
}

fn install_yt_dlp() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        println!("Installing yt-dlp via winget...");
        let status = Command::new("winget")
            .args(&["install", "yt-dlp"])
            .status()?;

        if !status.success() {
            return Err("Failed to install yt-dlp".into());
        }
    }

    #[cfg(target_os = "linux")]
    {
        println!("Installing yt-dlp via package manager...");
        if Command::new("apt")
            .args(&["install", "-y", "yt-dlp"])
            .status()
            .is_err()
        {
            Command::new("yum")
                .args(&["install", "-y", "yt-dlp"])
                .status()?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        println!("Installing yt-dlp via Homebrew...");
        Command::new("brew")
            .args(&["install", "yt-dlp"])
            .status()?;
    }

    Ok(())
}

fn fetch_playlist_videos(url: &str) -> Result<Vec<Music>, Box<dyn std::error::Error>> {
    let output = YoutubeDl::new(url)
        .flat_playlist(true) // Get the full playlist info
        .socket_timeout("15")
        .run()?;

    let mut videos = Vec::new();

    if let Some(playlist) = output.clone().into_playlist() {
        if let Some(entries) = playlist.entries {
            for video in entries {
                videos.push(Music {
                    url: video.url.unwrap_or_else(|| "Unknown".to_string()),
                });
            }
        } else {
            return Err("No videos found".into());
        }
    } else {
        videos.push(Music {
            url: url.to_string(),
        });
    }
    
    Ok(videos)
}

fn download_video(url: &str) {
    let status = YoutubeDl::new(url)
        .extract_audio(true)
        .extra_arg("--audio-format")
        .extra_arg("mp3")
        .extra_arg("--embed-metadata")
        // .extra_arg("--list-thumbnails")
        .extra_arg("--embed-thumbnail")
        .output_template("%(title)s.%(ext)s")
        .download_to("./Music");

    match status {
        Ok(_) => {
            println!("Downloaded {}", url);
        }
        Err(e) => {
            println!("Error downloading {}: {}", url, e);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if ffmpeg is installed
    if !check_ffmpeg() {
        println!("ffmpeg not found. Installing...");
        install_ffmpeg()?;
    }

    // Check if yt-dlp is installed
    if !check_yt_dlp() {
        println!("yt-dlp not found. Installing...");
        install_yt_dlp()?;
    }

    // Create a buffer list for URLs
    let url_buffer: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let buffer_clone = Arc::clone(&url_buffer);

    // Start command listener in a separate thread
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        
        while running.load(Ordering::SeqCst) {
            print!("Enter command (add <url>, download, or quit): ");
            stdout.flush().unwrap();
            
            let mut input = String::new();
            stdin.lock().read_line(&mut input).unwrap();
            let input = input.trim();
            
            if input.starts_with("add ") {
                let url = input.trim_start_matches("add ").trim().to_string();
                buffer_clone.lock().unwrap().push_back(url.clone());
                println!("Added to buffer: {}", url);
            } else if input == "download" {
                let urls: Vec<String> = buffer_clone.lock().unwrap().drain(..).collect();
                if urls.is_empty() {
                    println!("No URLs in buffer to download.");
                } else {
                    println!("Starting download for buffered URLs...");
                    urls.par_iter().for_each(|url| download_video(url));
                    println!("Download completed!");
                }
            } else if input == "quit" {
                println!("Stopping application...");
                running.store(false, Ordering::SeqCst);
                break;
            } else {
                println!("Unknown command. Available commands: add <url>, download, quit");
            }
        }
    });

    // Keep the main thread alive until stop command is received
    while r.load(Ordering::SeqCst) {
        thread::sleep(std::time::Duration::from_millis(100));
    }
    
    println!("Application stopped.");
    Ok(())
}
