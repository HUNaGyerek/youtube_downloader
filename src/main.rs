use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::{atomic::AtomicBool, Arc};

mod app_config;
mod downloader;
mod installer;
mod models;
mod runtime;
mod utils;
mod views;

use app_config::Config;
use installer::{check_ffmpeg, check_yt_dlp, install_ffmpeg, install_yt_dlp};
use models::translation::Translations;

use crate::runtime::{Runtime, RuntimeTrait};
use crate::views::main::{MainMenuOption, MainView};
use crate::views::View;

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

    let mut runtime = Runtime {
        url_buffer: Arc::new(Mutex::new(VecDeque::new())),
        state: Arc::new(AtomicBool::new(true)),
    };

    // Load configuration
    let config = Config::load();

    // Initialize translations
    Translations::init(&config);

    println!(
        "{} {}",
        Translations::t("current_language", None),
        Translations::t(
            &format!("language_{}", config.language.to_string().to_lowercase()),
            None
        )
    );
    println!(
        "{} {}",
        Translations::t("download_directory", None),
        config.get_download_dir()
    );

    // Main program loop
    while runtime.start() {
        // MAIN MENU
        let main_view = MainView::new();
        let main_choice = main_view.render_view();

        MainMenuOption::create_menu(&main_choice, &mut runtime);
    }

    println!("{}", Translations::t("app_stopped", None));
    Ok(())
}
