use std::sync::{Arc, Mutex};

use rayon::prelude::*;

use crate::{
    app_config::Config,
    downloader::{download_video, fetch_playlist_videos},
    models::{history::History, music::Music, translation::Translations},
    runtime::RuntimeTrait,
    utils::read_line,
    views::{
        settings::{SettingsMenuOption, SettingsView},
        View,
    },
};

pub struct MainView(Vec<MainViewOption>);
impl MainView {
    pub fn new() -> Self {
        Self(vec![
            MainViewOption::new(MainMenuOption::AddUrl, "menu_add_url"),
            MainViewOption::new(MainMenuOption::ListQueue, "menu_list_queue"),
            MainViewOption::new(MainMenuOption::Download, "menu_start_downloads"),
            MainViewOption::new(MainMenuOption::ViewHistory, "menu_view_history"),
            MainViewOption::new(MainMenuOption::ClearQueue, "menu_clear_queue"),
            MainViewOption::new(MainMenuOption::Settings, "menu_settings"),
            MainViewOption::new(MainMenuOption::Exit, "menu_exit"),
        ])
    }
}

impl View for MainView {
    type Output = MainMenuOption;

    fn render_view(&self) -> Self::Output {
        println!("\n{}", Translations::t("menu_title"));
        for main_option in &self.0 {
            println!("{}", Translations::t(&main_option.display_value))
        }

        let input: i8 = read_line(Translations::t("menu_enter_choice"))
            .parse()
            .unwrap();
        if &(input as usize) > &self.0.len() && input <= 0 {
            println!("{}", Translations::tf2("invalid_choice", "1", "7"));
            self.render_view();
        }

        MainMenuOption::from(input)
    }
}

pub enum MainMenuOption {
    AddUrl = 1,
    ListQueue,
    Download,
    ViewHistory,
    ClearQueue,
    Settings,
    Exit,
}

impl From<i8> for MainMenuOption {
    fn from(value: i8) -> Self {
        match value {
            1 => MainMenuOption::AddUrl,
            2 => MainMenuOption::ListQueue,
            3 => MainMenuOption::Download,
            4 => MainMenuOption::ViewHistory,
            5 => MainMenuOption::ClearQueue,
            6 => MainMenuOption::Settings,
            7 => MainMenuOption::Exit,
            _ => panic!("Invalid value for MainMenuOption"),
        }
    }
}

impl MainMenuOption {
    pub fn create_menu<R: RuntimeTrait>(&self, runtime: &mut R) {
        let config = Config::load();
        match &self {
            MainMenuOption::AddUrl => {
                let url = read_line(Translations::t("enter_url"));

                if !url.is_empty() {
                    println!("{}", Translations::t("fetching_info"));
                    match fetch_playlist_videos(&url) {
                        Ok(videos) => {
                            let mut buffer = runtime.get_url_buffer();
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
            MainMenuOption::ListQueue => {
                let buffer = runtime.get_url_buffer();

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
            MainMenuOption::Download => {
                let urls: Vec<Music> = runtime.drain_buffer();
                if urls.is_empty() {
                    println!("{}", Translations::t("no_urls_to_download"));
                } else {
                    println!(
                        "{}",
                        Translations::tf("starting_download", &urls.len().to_string())
                    );
                    let dir = config.get_download_dir().to_string();

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
                        let mut history = History::load();
                        for video in successful.iter() {
                            history.add(video);
                        }
                    }
                }
            }
            MainMenuOption::ViewHistory => {
                let history = History::load();
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
                            "{}. {title} - Downloaded on {date}",
                            history.downloads.len() - i
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
            MainMenuOption::ClearQueue => {
                let count = runtime.get_url_buffer().len();
                runtime.clear_url_buffer();
                println!("{}", Translations::tf("queue_cleared", &count.to_string()));
            }
            MainMenuOption::Settings => {
                let settings_view = SettingsView::new();
                let setting_choice = settings_view.render_view();

                SettingsMenuOption::create_menu(&setting_choice);
            }
            MainMenuOption::Exit => {
                println!("{}", Translations::t("exiting"));
                runtime.stop();
            }
        }
    }
}

pub struct MainViewOption {
    _option: MainMenuOption,
    pub display_value: String,
}
impl MainViewOption {
    pub fn new(_option: MainMenuOption, display_value: &str) -> Self {
        Self {
            _option,
            display_value: display_value.to_string(),
        }
    }
}
