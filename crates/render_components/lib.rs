pub mod empty_component;

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

pub enum Events {
    MouseMove { x: i32, y: i32 },
    MouseClick { x: i32, y: i32 },
    KeyPress { key: char },
}

pub trait RenderComponent {
    fn render_html(&self, slot: &str) -> String;
    fn render_native(&self) -> Shapes;
    fn handle_event(&mut self, event: Events);
}
