use crate::{translation::Translations, utils::read_line};

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

    pub fn render_view(&self) -> String {
        println!("\n{}", Translations::t("menu_title"));
        for main_option in &self.0 {
            println!("{}", Translations::t(&main_option.display_value))
        }

        read_line(Translations::t("menu_enter_choice"))
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

pub enum MainMenuOption {
    AddUrl,
    ListQueue,
    Download,
    ViewHistory,
    ClearQueue,
    Settings,
    Exit,
}
