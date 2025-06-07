use rayon::prelude::*;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::Mutex;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

mod app_config;
mod downloader;
mod installer;
mod models;
mod translation;
mod utils;
mod views;

use app_config::{Config, Language};
use downloader::{download_video, fetch_playlist_videos};
use installer::{check_ffmpeg, check_yt_dlp, install_ffmpeg, install_yt_dlp};
use models::Music;
use translation::Translations;
use utils::{add_to_history, load_history};

use crate::views::main::MainView;
use crate::views::settings::{SettingsMenuOption, SettingsView};

// Function to display language selection menu
fn display_language_menu() {
    println!("\n{}", Translations::t("language_select"));

    // Get available languages
    let languages = Language::all();

    // Display each language with its number
    for (i, lang) in languages.iter().enumerate() {
        match lang {
            Language::English => println!("{}. {}", i + 1, Translations::t("language_english")),
            Language::Hungarian => println!("{}. {}", i + 1, Translations::t("language_hungarian")),
        }
    }

    // Display back option
    println!(
        "{}. {}",
        languages.len() + 1,
        Translations::t("language_back")
    );
    print!("\n{}", Translations::t("language_enter_choice"));
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

    // Load configuration
    let config = Arc::new(Mutex::new(Config::load()));

    // Initialize translations
    Translations::init(&config.lock().unwrap());

    println!(
        "{} {}",
        Translations::t("current_language"),
        config.lock().unwrap().language
    );
    println!(
        "{} {}",
        Translations::t("download_directory"),
        config.lock().unwrap().get_download_dir()
    );

    // Load download history
    let history = Arc::new(Mutex::new(load_history()));

    // Main program loop
    while running.load(Ordering::SeqCst) {
        // MAIN MENU
        let main_view = MainView::new();
        let choice = main_view.render_view();

        match choice.as_str() {
            "1" => {
                print!("{}", Translations::t("enter_url"));
                io::stdout().flush().unwrap();

                let mut url = String::new();
                io::stdin().read_line(&mut url).unwrap();
                let url = url.trim().to_string();

                if !url.is_empty() {
                    println!("{}", Translations::t("fetching_info"));
                    match fetch_playlist_videos(&url) {
                        Ok(videos) => {
                            let mut buffer = url_buffer.lock().unwrap();
                            for video in videos {
                                if buffer.contains(&video) {
                                    println!("{}", Translations::t("already_added"));
                                    continue;
                                }
                                let title =
                                    video.title.clone().unwrap_or_else(|| "Unknown".to_string());
                                println!("{}", Translations::tf("added_to_queue", &title));
                                buffer.push_back(video);
                            }
                        }
                        Err(e) => {
                            println!("{}", Translations::tf("error_fetching", &e.to_string()))
                        }
                    }
                }
            }
            "2" => {
                let buffer = url_buffer.lock().unwrap();
                if buffer.is_empty() {
                    println!("{}", Translations::t("download_queue_empty"));
                } else {
                    println!("\n{}", Translations::t("download_queue_title"));
                    for (i, video) in buffer.iter().enumerate() {
                        let title = video.title.clone().unwrap_or_else(|| "Unknown".to_string());
                        println!("{}. {}", i + 1, title);
                    }
                }
            }
            "3" => {
                let urls: Vec<Music> = url_buffer.lock().unwrap().drain(..).collect();
                if urls.is_empty() {
                    println!("{}", Translations::t("no_urls_to_download"));
                } else {
                    println!(
                        "{}",
                        Translations::tf("starting_download", &urls.len().to_string())
                    );
                    let dir = config.lock().unwrap().get_download_dir().to_string();

                    let successful_downloads = Arc::new(Mutex::new(Vec::new()));
                    let successful_clone = Arc::clone(&successful_downloads);

                    let results: Vec<Result<(), Box<dyn std::error::Error + Send + Sync>>> = urls
                        .par_iter()
                        .map(|video| {
                            if let Ok(()) = download_video(video, &dir) {
                                successful_clone.lock().unwrap().push(video.clone());
                                Ok(())
                            } else {
                                Err("Download failed".into())
                            }
                        })
                        .collect();

                    let success_count = results.iter().filter(|r| r.is_ok()).count();
                    let fail_count = results.len() - success_count;

                    println!("\n{}", Translations::t("download_summary"));
                    println!(
                        "{}",
                        Translations::tf2(
                            "download_success",
                            &success_count.to_string(),
                            &urls.len().to_string()
                        )
                    );
                    println!(
                        "{}",
                        Translations::tf("download_fail", &fail_count.to_string())
                    );

                    // Add successful downloads to history
                    let successful = successful_downloads.lock().unwrap();
                    if !successful.is_empty() {
                        let mut history = history.lock().unwrap();
                        for video in successful.iter() {
                            add_to_history(&mut history, video.clone());
                        }
                    }
                }
            }
            "4" => {
                let history = history.lock().unwrap();
                if history.downloads.is_empty() {
                    println!("{}", Translations::t("no_history"));
                } else {
                    println!("\n{}", Translations::t("history_title"));
                    for (i, video) in history.downloads.iter().enumerate().rev().take(10) {
                        let title = video.title.clone().unwrap_or_else(|| "Unknown".to_string());
                        let date = video
                            .downloaded_at
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string());
                        println!(
                            "{}. {} - Downloaded on {}",
                            history.downloads.len() - i,
                            title,
                            date
                        );
                    }

                    if history.downloads.len() > 10 {
                        println!(
                            "{}",
                            Translations::tf(
                                "history_more",
                                &(history.downloads.len() - 10).to_string()
                            )
                        );
                    }
                }
            }
            "5" => {
                let mut buffer = url_buffer.lock().unwrap();
                let count = buffer.len();
                buffer.clear();
                println!("{}", Translations::tf("queue_cleared", &count.to_string()));
            }
            "6" => {
                let settings_view = SettingsView::new();
                let setting_choice = settings_view.render_view();

                match setting_choice.as_str() {
                    "1" => SettingsView::create_menu(SettingsMenuOption::Language),
                    "2" => SettingsView::create_menu(SettingsMenuOption::Directory),
                    "3" => SettingsView::create_menu(SettingsMenuOption::Coloring),
                    "4" => SettingsView::create_menu(SettingsMenuOption::Back),
                    _ => println!("{}", Translations::tf2("invalid_choice", "1", "4")),
                }
            }
            "7" => {
                println!("{}", Translations::t("exiting"));
                running.store(false, Ordering::SeqCst);
                break;
            }
            _ => println!("{}", Translations::tf2("invalid_choice", "1", "8")),
        }
    }

    println!("{}", Translations::t("app_stopped"));
    Ok(())
}
