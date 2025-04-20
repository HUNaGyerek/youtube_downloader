use chrono;
use std::fs;
use std::path::Path;

use crate::models::{History, Music};

pub fn load_history() -> History {
    let history_path = "download_history.json";
    if Path::new(history_path).exists() {
        match fs::read_to_string(history_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(history) => return history,
                Err(e) => {
                    println!("Error parsing history file: {}", e);
                    return History {
                        downloads: Vec::new(),
                    };
                }
            },
            Err(_) => {
                return History {
                    downloads: Vec::new(),
                }
            }
        }
    }
    History {
        downloads: Vec::new(),
    }
}

pub fn save_history(history: &History) -> Result<(), Box<dyn std::error::Error>> {
    let history_path = "download_history.json";
    let json = serde_json::to_string_pretty(history)?;
    fs::write(history_path, json)?;
    Ok(())
}

pub fn add_to_history(history: &mut History, video: Music) {
    // Update downloaded_at timestamp
    let mut video = video;
    let now = chrono::Local::now().to_string();
    video.downloaded_at = Some(now);

    // Add to history
    history.downloads.push(video);
    if let Err(e) = save_history(history) {
        println!("Failed to save download history: {}", e);
    }
}
