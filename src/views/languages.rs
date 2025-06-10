use crate::{
    models::{language::Language, translation::Translations},
    utils::read_line,
    views::View,
};

#[derive(Debug)]
pub struct LanguageView(Vec<LanguageViewOption>);
impl LanguageView {
    pub fn new() -> Self {
        let mut languages = vec![];
        for language in Language::all() {
            languages.push(LanguageViewOption::new(
                Language::from(language),
                &Translations::t(&format!("language_{}", language.to_string().to_lowercase())),
            ));
        }

        Self(languages)
    }
}

impl View for LanguageView {
    type Output = LanguageMenuOption;

    fn render_view(&self) -> Self::Output {
        println!("\n{}", Translations::t("settings_title"));

        for (idx, option) in self.0.iter().enumerate() {
            println!("{}. {}", idx + 1, Translations::t(&option.display_value));
        }
        println!("{}. {}", self.0.len() + 1, Translations::t("language_back"));

        let input: i8 = read_line(Translations::tf2(
            "language_enter_choice",
            "1",
            &(self.0.len() + 1).to_string(),
        ))
        .parse()
        .unwrap();

        if &(input as usize) > &self.0.len() && input <= 0 {
            println!(
                "{}",
                Translations::tf2("invalid_choice", "1", &self.0.len().to_string())
            );
            self.render_view();
        }

        return match input {
            n if (n as usize == self.0.len() + 1) => return LanguageMenuOption::Back,
            _ => self
                .0
                .iter()
                .enumerate()
                .find(|(i, _)| i + 1 == input as usize)
                .map(|(_, l)| LanguageMenuOption::Language(l._option))
                .expect("Invalid input"),
        };
    }
}

pub enum LanguageMenuOption {
    Language(Language),
    Back,
}

#[derive(Debug)]
pub struct LanguageViewOption {
    _option: Language,
    pub display_value: String,
}
impl LanguageViewOption {
    pub fn new(_option: Language, display_value: &str) -> Self {
        Self {
            _option,
            display_value: display_value.to_string(),
        }
    }
}
