use rayon::prelude::*;
use rfd::FileDialog;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::Mutex;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

mod config;
mod downloaders;
mod installers;
mod models;
mod translations;
mod utils;

use config::{Config, Language};
use downloaders::{download_video, fetch_playlist_videos};
use installers::{check_ffmpeg, check_yt_dlp, get_yt_dlp_path, install_ffmpeg, install_yt_dlp};
use models::Music;
use translations::Translations;
use utils::{add_to_history, load_history};

fn print_menu() {
    println!("\n{}", Translations::t("menu_title"));
    println!("{}", Translations::t("menu_add_url"));
    println!("{}", Translations::t("menu_list_queue"));
    println!("{}", Translations::t("menu_start_downloads"));
    println!("{}", Translations::t("menu_view_history"));
    println!("{}", Translations::t("menu_clear_queue"));
    println!("{}", Translations::t("menu_settings"));
    println!("{}", Translations::t("menu_exit"));
    print!("\n{}", Translations::t("menu_enter_choice"));
    io::stdout().flush().unwrap();
}

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
                            println!(
                                "Failed to install yt-dlp: {}. Cannot proceed with download.",
                                e
                            );
                            continue; // Skip to next iteration of the loop
                        }
                    }

                    // Verify installation was successful
                    if get_yt_dlp_path().is_none() {
                        println!(
                            "yt-dlp was not found after installation. Please install it manually."
                        );
                        continue; // Skip to next iteration
                    }
                }

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
                println!("\n{}", Translations::t("settings_title"));
                println!("{}", Translations::t("settings_language"));
                println!("{}", Translations::t("settings_set_directory"));
                println!("{}", Translations::t("settings_back"));
                print!("\n{}", Translations::t("settings_enter_choice"));
                io::stdout().flush().unwrap();

                let mut setting_choice = String::new();
                io::stdin().read_line(&mut setting_choice).unwrap();
                let setting_choice = setting_choice.trim();

                match setting_choice {
                    "1" => {
                        // Display language selection menu
                        display_language_menu();

                        let mut lang_choice = String::new();
                        io::stdin().read_line(&mut lang_choice).unwrap();

                        // Parse choice as a number
                        if let Ok(choice_num) = lang_choice.trim().parse::<usize>() {
                            let languages = Language::all();

                            if choice_num >= 1 && choice_num <= languages.len() {
                                // Valid language choice
                                let selected_lang = languages[choice_num - 1];
                                let mut config = config.lock().unwrap();
                                config.set_language(selected_lang)?;
                                Translations::change_language(selected_lang);

                                match selected_lang {
                                    Language::English => {
                                        println!("{}", Translations::t("language_set_english"))
                                    }
                                    Language::Hungarian => {
                                        println!("{}", Translations::t("language_set_hungarian"))
                                    }
                                }
                            } else if choice_num == languages.len() + 1 {
                                // Back option
                                println!("{}", Translations::t("return_to_menu"));
                            } else {
                                // Invalid number
                                println!(
                                    "{}",
                                    Translations::tf2(
                                        "invalid_choice",
                                        "1",
                                        &(languages.len() + 1).to_string()
                                    )
                                );
                            }
                        } else {
                            // Not a number
                            println!(
                                "{}",
                                Translations::tf2(
                                    "invalid_choice",
                                    "1",
                                    &(Language::all().len() + 1).to_string()
                                )
                            );
                        }
                    }
                    "2" => {
                        if let Some(new_dir) = FileDialog::new().pick_folder() {
                            let dir_str = new_dir.display().to_string();
                            let mut config = config.lock().unwrap();
                            config.set_download_dir(dir_str.clone())?;
                            println!("{}", Translations::tf("dir_set", &dir_str));
                        } else {
                            println!("{}", Translations::t("no_dir_selected"));
                        }
                    }
                    "3" => println!("{}", Translations::t("return_to_menu")),
                    _ => println!("{}", Translations::tf2("invalid_choice", "1", "2")),
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
