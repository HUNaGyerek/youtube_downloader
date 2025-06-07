use std::io;

use rfd::FileDialog;

use crate::{
    app_config::{Config, Language},
    display_language_menu,
    translation::Translations,
    utils::read_line,
};

pub enum SettingsMenuOption {
    Language,
    Directory,
    Coloring,
    Back,
}

pub struct SettingsViewOption {
    _option: SettingsMenuOption,
    pub display_value: String,
}

impl SettingsViewOption {
    pub fn new(_option: SettingsMenuOption, display_value: &str) -> Self {
        Self {
            _option,
            display_value: display_value.to_string(),
        }
    }
}

pub struct SettingsView(Vec<SettingsViewOption>);

impl SettingsView {
    pub fn new() -> Self {
        Self(vec![
            SettingsViewOption::new(SettingsMenuOption::Language, "settings_language"),
            SettingsViewOption::new(SettingsMenuOption::Directory, "settings_set_directory"),
            SettingsViewOption::new(SettingsMenuOption::Coloring, "settings_coloring"),
            SettingsViewOption::new(SettingsMenuOption::Back, "settings_back"),
        ])
    }

    pub fn render_view(&self) -> String {
        println!("\n{}", Translations::t("settings_title"));

        for option in &self.0 {
            println!("{}", Translations::t(&option.display_value));
        }

        read_line(Translations::t("settings_enter_choice"))
    }

    pub fn create_menu(value: SettingsMenuOption) {
        let mut config = Config::load();
        match value {
            SettingsMenuOption::Language => {
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
            SettingsMenuOption::Directory => {
                if let Some(new_dir) = FileDialog::new().pick_folder() {
                    let dir_str = new_dir.display().to_string();
                    config.set_download_dir(dir_str.clone()).unwrap();
                    println!("{}", Translations::tf("dir_set", &dir_str));
                } else {
                    println!("{}", Translations::t("no_dir_selected"));
                }
            }
            SettingsMenuOption::Coloring => {
                let coloring = !config.coloring;

                config.set_coloring(coloring).unwrap();
                Translations::set_coloring(coloring);

                println!("{}", Translations::t("coloring_toggled"));
            }
            SettingsMenuOption::Back => println!("{}", Translations::t("return_to_menu")),
        }
    }
}
