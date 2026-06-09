use render::run_default;
use render_components::primitives::Rectangle;
use render_layout::{EventHandler, InternalLayoutable, LayoutType, Layoutable, Layouted};
use render_proc_macro::layoutable;

#[layoutable]
pub(crate) struct App {}

impl Layoutable for App {
    fn children(&self) -> Vec<Layouted<dyn InternalLayoutable>> {
        let mut rect = Rectangle::new();
        rect.color = (255, 255, 255, 255);
        rect.rounding = (0.2, 0.5, 1.0, 0.0).into();
        let layouted_rect = Layouted::new(rect, LayoutType::AbsoluteFrFrFrFr(0.1, 0.1, 0.9, 0.9));
        vec![layouted_rect]
    }
}

impl EventHandler for App {}

fn main() {
    let mut component = App::default();
    component.children = component.children(); // Generate the child rectangle with the specified color
    run_default(component);
}
