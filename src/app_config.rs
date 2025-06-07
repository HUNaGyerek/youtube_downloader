use dirs;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English,
    Hungarian,
}

impl Language {
    // Convert the language enum to a lowercase string that can be used for filenames
    pub fn to_filename(&self) -> String {
        format!("{}.toml", self.to_string().to_lowercase())
    }

    // Get all available languages
    pub fn all() -> Vec<Language> {
        vec![Language::English, Language::Hungarian]
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Language::English => write!(f, "English"),
            Language::Hungarian => write!(f, "Hungarian"),
        }
    }
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "english" => Ok(Language::English),
            "hungarian" => Ok(Language::Hungarian),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub language: Language,
    pub download_dir: String,
    pub coloring: bool,
}

impl Default for Config {
    fn default() -> Self {
        // Default to user's Music directory
        let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let music_dir = home_dir.join("Music");

        Self {
            language: Language::default(),
            download_dir: music_dir.to_string_lossy().to_string(),
            coloring: false,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = "config.toml";
        // On some systems, it might be installed but not in PATH
        if Path::new(config_path).exists() {
            match fs::read_to_string(config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        println!("Error parsing config file: {}", e);
                        return Config::default();
                    }
                },
                Err(_) => return Config::default(),
            }
        }
        Config::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = "config.toml";
        let toml = toml::to_string_pretty(self)?;
        let mut file = fs::File::create(config_path)?;
        file.write_all(toml.as_bytes())?;
        Ok(())
    }

    pub fn set_language(&mut self, language: Language) -> Result<(), Box<dyn std::error::Error>> {
        self.language = language;
        self.save()?;
        Ok(())
    }

    pub fn set_download_dir(&mut self, dir: String) -> Result<(), Box<dyn std::error::Error>> {
        self.download_dir = dir;
        self.save()?;
        Ok(())
    }

    pub fn set_coloring(&mut self, coloring: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.coloring = coloring;
        self.save()?;
        Ok(())
    }

    pub fn get_download_dir(&self) -> &str {
        &self.download_dir
    }
}
