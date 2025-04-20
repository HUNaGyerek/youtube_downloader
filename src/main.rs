use rayon::prelude::*;
use std::io::{self, Write};
use std::sync::Mutex;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::collections::VecDeque;
use rfd::FileDialog;
use dirs;

mod models;
mod installers;
mod downloaders;
mod utils;

use models::Music;
use installers::{check_ffmpeg, check_yt_dlp, install_ffmpeg, install_yt_dlp, get_yt_dlp_path};
use downloaders::{fetch_playlist_videos, download_video};
use utils::{load_history, add_to_history};

fn print_menu() {
    println!("\n===== YouTube Downloader =====");
    println!("1. Add URL to download queue");
    println!("2. List queued downloads");
    println!("3. Start downloads");
    println!("4. Set download directory");
    println!("5. View download history");
    println!("6. Clear download queue");
    println!("7. Exit");
    print!("\nEnter choice (1-7): ");
    io::stdout().flush().unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("YouTube Downloader v0.2.0");
    
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
    let url_buffer: Arc<Mutex<VecDeque<Music>>> = Arc::new(Mutex::new(VecDeque::new()));
    let running = Arc::new(AtomicBool::new(true));
    
    // Default download directory
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let music_dir = home_dir.join("Music");
    let download_dir = Arc::new(Mutex::new(music_dir.to_string_lossy().to_string()));
    
    // Load download history
    let history = Arc::new(Mutex::new(load_history()));

    // Main program loop
    while running.load(Ordering::SeqCst) {
        print_menu();
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();
        
        match choice {
            "1" => {
                // First check if yt-dlp is installed
                if get_yt_dlp_path().is_none() {
                    println!("yt-dlp is not installed or not found. Attempting to install...");
                    match install_yt_dlp() {
                        Ok(_) => println!("yt-dlp installed successfully."),
                        Err(e) => {
                            println!("Failed to install yt-dlp: {}. Cannot proceed with download.", e);
                            continue; // Skip to next iteration of the loop
                        }
                    }
                    
                    // Verify installation was successful
                    if get_yt_dlp_path().is_none() {
                        println!("yt-dlp was not found after installation. Please install it manually.");
                        continue; // Skip to next iteration
                    }
                }
                
                print!("Enter YouTube URL: ");
                io::stdout().flush().unwrap();
                
                let mut url = String::new();
                io::stdin().read_line(&mut url).unwrap();
                let url = url.trim().to_string();
                
                if !url.is_empty() {
                    println!("Fetching video information...");
                    match fetch_playlist_videos(&url) {
                        Ok(videos) => {
                            let mut buffer = url_buffer.lock().unwrap();
                            for video in videos {
                                let title = video.title.clone().unwrap_or_else(|| "Unknown".to_string());
                                println!("Added to queue: {}", title);
                                buffer.push_back(video);
                            }
                        },
                        Err(e) => println!("Error fetching video info: {}", e),
                    }
                }
            },
            "2" => {
                let buffer = url_buffer.lock().unwrap();
                if buffer.is_empty() {
                    println!("Download queue is empty");
                } else {
                    println!("\n--- Download Queue ---");
                    for (i, video) in buffer.iter().enumerate() {
                        let title = video.title.clone().unwrap_or_else(|| "Unknown".to_string());
                        println!("{}. {}", i+1, title);
                    }
                }
            },
            "3" => {
                let urls: Vec<Music> = url_buffer.lock().unwrap().drain(..).collect();
                if urls.is_empty() {
                    println!("No URLs in queue to download.");
                } else {
                    println!("Starting download for {} videos...", urls.len());
                    let dir = download_dir.lock().unwrap().clone();
                    
                    let successful_downloads = Arc::new(Mutex::new(Vec::new()));
                    let successful_clone = Arc::clone(&successful_downloads);
                    
                    let results: Vec<Result<(), Box<dyn std::error::Error + Send + Sync>>> = urls.par_iter().map(|video| {
                        if let Ok(()) = download_video(video, &dir) {
                            successful_clone.lock().unwrap().push(video.clone());
                            Ok(())
                        } else {
                            Err("Download failed".into())
                        }
                    }).collect();
                    
                    let success_count = results.iter().filter(|r| r.is_ok()).count();
                    let fail_count = results.len() - success_count;
                    
                    println!("\nDownload summary:");
                    println!("- Successfully downloaded: {}/{}", success_count, urls.len());
                    println!("- Failed downloads: {}", fail_count);
                    
                    // Add successful downloads to history
                    let successful = successful_downloads.lock().unwrap();
                    if !successful.is_empty() {
                        let mut history = history.lock().unwrap();
                        for video in successful.iter() {
                            add_to_history(&mut history, video.clone());
                        }
                    }
                }
            },
            "4" => {
                if let Some(new_dir) = FileDialog::new().pick_folder() {
                    *download_dir.lock().unwrap() = new_dir.display().to_string();
                    println!("Download directory set to: {}", *download_dir.lock().unwrap());
                } else {
                    println!("No directory selected.");
                }
            },
            "5" => {
                let history = history.lock().unwrap();
                if history.downloads.is_empty() {
                    println!("No download history available.");
                } else {
                    println!("\n--- Download History ---");
                    for (i, video) in history.downloads.iter().enumerate().rev().take(10) {
                        let title = video.title.clone().unwrap_or_else(|| "Unknown".to_string());
                        let date = video.downloaded_at.clone().unwrap_or_else(|| "Unknown".to_string());
                        println!("{}. {} - Downloaded on {}", history.downloads.len() - i, title, date);
                    }
                    
                    if history.downloads.len() > 10 {
                        println!("...and {} more", history.downloads.len() - 10);
                    }
                }
            },
            "6" => {
                let mut buffer = url_buffer.lock().unwrap();
                let count = buffer.len();
                buffer.clear();
                println!("Cleared {} items from the download queue", count);
            },
            "7" => {
                println!("Exiting...");
                running.store(false, Ordering::SeqCst);
                break;
            },
            _ => println!("Invalid choice. Please enter a number between 1 and 7."),
        }
    }
    
    println!("Application stopped.");
    Ok(())
}