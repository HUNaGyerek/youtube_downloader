use rfd::FileDialog;

use crate::{
    app_config::Config,
    models::translation::Translations,
    utils::read_line,
    views::{
        languages::{LanguageMenuOption, LanguageView},
        View,
    },
};

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
}

impl View for SettingsView {
    type Output = SettingsMenuOption;

    fn render_view(&self) -> Self::Output {
        println!("\n{}", Translations::t("settings_title", None));

        for option in &self.0 {
            println!("{}", Translations::t(&option.display_value, None));
        }

        let input: i8 = read_line(Translations::t("settings_enter_choice", None))
            .parse()
            .unwrap();
        if &(input as usize) > &self.0.len() && input <= 0 {
            println!("{}", Translations::t("invalid_choice", Some(&["1", "4"])));
            self.render_view();
        }

        SettingsMenuOption::from(input)
    }
}

pub enum SettingsMenuOption {
    Language = 1,
    Directory,
    Coloring,
    Back,
}

impl From<i8> for SettingsMenuOption {
    fn from(value: i8) -> Self {
        match value {
            1 => SettingsMenuOption::Language,
            2 => SettingsMenuOption::Directory,
            3 => SettingsMenuOption::Coloring,
            4 => SettingsMenuOption::Back,
            _ => panic!("Invalid value for SettingsMenuOption"),
        }
    }
}

impl SettingsMenuOption {
    pub fn create_menu(&self) {
        let mut config = Config::load();
        match self {
            SettingsMenuOption::Language => {
                let language_view = LanguageView::new();
                let language_choice = language_view.render_view();

                match language_choice {
                    LanguageMenuOption::Language(language) => {
                        config.set_language(language).unwrap();
                        Translations::change_language(language);
                        println!(
                            "{}",
                            Translations::t(
                                &format!("language_set_{}", language.to_string().to_lowercase()),
                                None
                            )
                        );
                    }
                    LanguageMenuOption::Back => {}
                }
            }
            SettingsMenuOption::Directory => {
                if let Some(new_dir) = FileDialog::new().pick_folder() {
                    let dir_str = new_dir.display().to_string();
                    config.set_download_dir(dir_str.clone()).unwrap();
                    println!("{}", Translations::t("dir_set", Some(&[&dir_str])));
                } else {
                    println!("{}", Translations::t("no_dir_selected", None));
                }
            }
            SettingsMenuOption::Coloring => {
                let coloring = !config.coloring;

                config.set_coloring(coloring).unwrap();
                Translations::set_coloring(coloring);

                println!("{}", Translations::t("coloring_toggled", None));
            }
            SettingsMenuOption::Back => println!("{}", Translations::t("return_to_menu", None)),
        }
    }
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
