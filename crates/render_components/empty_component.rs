use crate::{Events, RenderComponent, Shapes};

pub struct EmptyComponent;
impl RenderComponent for EmptyComponent {
    fn render_html(&self, slot: &str) -> String {
        format!(r#"<div>{}</div>"#, slot)
    }

    fn render_native(&self) -> Shapes {
        Shapes::Rectangle {
            width: 0,
            height: 0,
        }
    }

    fn handle_event(&mut self, _event: Events) {}
}
