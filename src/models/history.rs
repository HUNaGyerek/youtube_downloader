use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::models::music::Music;

#[derive(Debug, Deserialize, Serialize)]
pub struct History {
    pub downloads: Vec<Music>,
}

impl History {
    pub fn load() -> Self {
        let history_path = "download_history.json";
        if Path::new(history_path).exists() {
            match fs::read_to_string(history_path) {
                Ok(content) => match serde_json::from_str::<History>(&content) {
                    Ok(history) => return history,
                    Err(e) => {
                        println!("Error parsing history file: {}", e);
                        return Self {
                            downloads: Vec::new(),
                        };
                    }
                },
                Err(_) => {
                    return Self {
                        downloads: Vec::new(),
                    }
                }
            }
        }
        Self {
            downloads: Vec::new(),
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let history_path = "download_history.json";
        let json = serde_json::to_string_pretty(self)?;
        fs::write(history_path, json)?;
        Ok(())
    }

    pub fn add(&mut self, video: &Music) {
        let mut video_copy = video.clone();
        let now = chrono::Local::now().to_string();
        video_copy.downloaded_at = Some(now);

        self.downloads.push(video_copy);
        if let Err(e) = self.save() {
            println!("Failed to save download history: {}", e);
        }
    }
}
