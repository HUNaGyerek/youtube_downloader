pub mod languages;
pub mod main;
pub mod settings;

pub trait View {
    type Output;
    fn render_view(&self) -> Self::Output;
}
