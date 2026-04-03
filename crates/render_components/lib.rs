use render_events::Events;

pub mod empty_component;

#[derive(Debug)]
pub enum Shapes {
    Rectangle {
        width: u32,
        height: u32,
    },
    ConstrainedText {
        content: String,
        font: String,
        width: u32,
        height: u32,
    },
}

pub trait RenderComponent {
    fn render_html(&self, slot: &str) -> String;
    fn render_native(&self) -> Shapes;
    fn handle_event(&mut self, event: Events);
}
