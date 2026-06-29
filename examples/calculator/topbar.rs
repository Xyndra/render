use render_components::primitives::Rectangle;
use render_layout::{InternalLayoutable, LayoutType, Layoutable, Layouted};
use render_proc_macro::layoutable;

#[layoutable]
pub(crate) struct TopBar {}

impl Layoutable for TopBar {
    fn children(&self) -> Vec<Layouted<dyn InternalLayoutable>> {
        let mut bg_rect = Rectangle::new();
        bg_rect.color = (255, 255, 255, 255);
        bg_rect.rounding = (0.0, 0.0, 0.75, 0.75).into();
        let bg = Layouted::new(bg_rect, LayoutType::AbsoluteBg);
        vec![bg]
    }
}
