use std::io;

use rfd::FileDialog;

use crate::{
    app_config::{Config, Language},
    display_language_menu,
    translation::Translations,
};

pub enum SettingsOptionValue {
    Language,
    Directory,
    Coloring,
    Back,
}

pub struct SettingsOption {
    _value: SettingsOptionValue,
    pub language: String,
}

impl SettingsOption {
    pub fn new(_value: SettingsOptionValue, language: &str) -> Self {
        Self {
            _value,
            language: language.to_string(),
        }
    }
}

pub struct Settings {
    pub options: Vec<SettingsOption>,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            options: vec![
                SettingsOption::new(SettingsOptionValue::Language, "settings_language"),
                SettingsOption::new(SettingsOptionValue::Directory, "settings_set_directory"),
                SettingsOption::new(SettingsOptionValue::Coloring, "settings_coloring"),
                SettingsOption::new(SettingsOptionValue::Back, "settings_back"),
            ],
        }
    }

    pub fn create_menu(value: SettingsOptionValue) {
        let mut config = Config::load();
        match value {
            SettingsOptionValue::Language => {
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

                        config.set_language(selected_lang).unwrap();
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
            SettingsOptionValue::Directory => {
                if let Some(new_dir) = FileDialog::new().pick_folder() {
                    let dir_str = new_dir.display().to_string();
                    config.set_download_dir(dir_str.clone()).unwrap();
                    println!("{}", Translations::tf("dir_set", &dir_str));
                } else {
                    println!("{}", Translations::t("no_dir_selected"));
                }
            }
            SettingsOptionValue::Coloring => {
                let coloring = !config.coloring;

                config.set_coloring(coloring).unwrap();
                Translations::set_coloring(coloring);

                println!("{}", Translations::t("coloring_toggled"));
            }
            SettingsOptionValue::Back => println!("{}", Translations::t("return_to_menu")),
        }
    }
}
