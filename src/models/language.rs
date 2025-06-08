use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

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
        Language::Hungarian
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
