use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::Mutex;

use crate::app_config::Config;
use crate::models::language::Language;

pub type TranslationMap = HashMap<String, String>;

// ANSI color codes enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnsiColor {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    Bold,
    Reset,
}

impl AnsiColor {
    pub fn code(&self) -> &'static str {
        match self {
            AnsiColor::Red => "\x1b[31m",
            AnsiColor::Green => "\x1b[32m",
            AnsiColor::Yellow => "\x1b[33m",
            AnsiColor::Blue => "\x1b[34m",
            AnsiColor::Magenta => "\x1b[35m",
            AnsiColor::Cyan => "\x1b[36m",
            AnsiColor::BrightRed => "\x1b[91m",
            AnsiColor::BrightGreen => "\x1b[92m",
            AnsiColor::BrightYellow => "\x1b[93m",
            AnsiColor::BrightBlue => "\x1b[94m",
            AnsiColor::BrightMagenta => "\x1b[95m",
            AnsiColor::BrightCyan => "\x1b[96m",
            AnsiColor::Bold => "\x1b[1m",
            AnsiColor::Reset => "\x1b[0m",
        }
    }
}

lazy_static! {
    static ref TRANSLATIONS: Mutex<Translations> = Mutex::new(Translations::new());

    // Map of color tags to their ANSI color codes
    static ref COLOR_TAGS: HashMap<&'static str, AnsiColor> = {
        let mut m = HashMap::new();
        m.insert("red", AnsiColor::Red);
        m.insert("green", AnsiColor::Green);
        m.insert("yellow", AnsiColor::Yellow);
        m.insert("blue", AnsiColor::Blue);
        m.insert("magenta", AnsiColor::Magenta);
        m.insert("cyan", AnsiColor::Cyan);
        m.insert("bright_red", AnsiColor::BrightRed);
        m.insert("bright_green", AnsiColor::BrightGreen);
        m.insert("bright_yellow", AnsiColor::BrightYellow);
        m.insert("bright_blue", AnsiColor::BrightBlue);
        m.insert("bright_magenta", AnsiColor::BrightMagenta);
        m.insert("bright_cyan", AnsiColor::BrightCyan);
        m.insert("b", AnsiColor::Bold);
        m
    };
}

pub struct Translations {
    current_language: Language,
    strings: HashMap<Language, TranslationMap>,
    coloring: bool,
}

impl Translations {
    pub fn new() -> Self {
        let mut translations = Self {
            current_language: Language::Hungarian,
            strings: HashMap::new(),
            coloring: false,
        };

        // Load all languages using the Language.all() method
        for language in Language::all() {
            let filename = language.to_filename();
            match load_language_file(&filename) {
                Ok(map) => {
                    translations.strings.insert(language, map);
                }
                Err(e) => {
                    eprintln!("Error loading {}: {}", filename, e);
                    eprintln!(
                        "Make sure the {} file exists in the languages directory.",
                        filename
                    );
                    process::exit(1);
                }
            }
        }

        translations
    }

    pub fn init(config: &Config) {
        let mut translations = TRANSLATIONS.lock().unwrap();
        translations.current_language = config.language.clone();
    }

    pub fn get(key: &str) -> String {
        let translations = TRANSLATIONS.lock().unwrap();

        let mut result = String::new();

        if let Some(lang_map) = translations.strings.get(&translations.current_language) {
            if let Some(text) = lang_map.get(key) {
                result = text.clone();
            }
        }

        // Fall back to English if the key doesn't exist in current language
        if result.is_empty() && translations.current_language != Language::English {
            if let Some(eng_map) = translations.strings.get(&Language::English) {
                if let Some(text) = eng_map.get(key) {
                    result = text.clone();
                }
            }
        }

        // Last resort - return the key itself
        if result.is_empty() {
            result = key.to_string();
        }

        // Apply color formatting
        apply_color_formatting(&result)
    }

    pub fn change_language(language: Language) {
        let mut translations = TRANSLATIONS.lock().unwrap();
        translations.current_language = language;
    }

    pub fn set_coloring(coloring: bool) {
        let mut translations = TRANSLATIONS.lock().unwrap();
        translations.coloring = coloring;
    }

    pub fn t(key: &str, args: Option<&[&str]>) -> String {
        Self::get(&replace_all(key, args))
    }
}

// Apply color formatting to text with XML-like color tags
fn apply_color_formatting(text: &str) -> String {
    let config = Config::load();

    if !config.coloring {
        // Remove all tags
        let mut result = text.to_string();
        for (tag, _) in COLOR_TAGS.iter() {
            let open_tag = format!("<{}>", tag);
            let close_tag = format!("</{}>", tag);
            result = result.replace(&open_tag, "").replace(&close_tag, "");
        }
        return result;
    }

    let mut result = String::new();
    let mut tag_stack: Vec<AnsiColor> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    result.push_str(AnsiColor::Reset.code());

    while i < chars.len() {
        if chars[i] == '<' {
            // Find the closing '>' character
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '>' {
                j += 1;
            }

            if j < chars.len() {
                let tag: String = chars[i + 1..j].iter().collect();
                let is_closing = tag.starts_with('/');
                let tag_name = if is_closing { &tag[1..] } else { &tag };

                if let Some(color) = COLOR_TAGS.get(tag_name) {
                    if is_closing {
                        // Remove the last matching tag from stack
                        if let Some(pos) = tag_stack.iter().rposition(|&c| c == *color) {
                            tag_stack.remove(pos);
                        }
                        result.push_str(AnsiColor::Reset.code());
                        // Re-apply all remaining styles
                        for color in &tag_stack {
                            result.push_str(color.code());
                        }
                    } else {
                        // Apply the color immediately
                        result.push_str(color.code());
                        tag_stack.push(*color);
                    }
                } else {
                    // Not a valid tag, include it literally
                    result.push('<');
                    result.push_str(&tag);
                    result.push('>');
                }

                i = j + 1;
                continue;
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    // Reset formatting at end
    // result.push_str(AnsiColor::Reset.code());
    result
}

fn load_language_file(filename: &str) -> Result<TranslationMap, Box<dyn std::error::Error>> {
    let path = Path::new("languages").join(filename);
    if path.exists() {
        let content = fs::read_to_string(path)?;
        let parsed: HashMap<String, String> = toml::from_str(&content)?;

        Ok(parsed)
    } else {
        Err(format!(
            "Language file '{}' not found in the languages directory",
            filename
        )
        .into())
    }
}

fn replace_all(template: &str, replacements: Option<&[&str]>) -> String {
    let mut result = String::new();
    let mut parts = template.split("{}");
    let mut iter = replacements.into_iter().flatten();

    if let Some(first) = parts.next() {
        result.push_str(first);
    }

    for part in parts {
        match iter.next() {
            Some(value) => result.push_str(value),
            None => result.push_str("{}"),
        }
        result.push_str(part);
    }

    result
}
