use render_components::primitives::Rectangle;
use render_layout::{EventHandler, InternalLayoutable, LayoutType, Layoutable, Layouted};
use render_proc_macro::layoutable;

#[layoutable]
pub(crate) struct App {}

impl Layoutable for App {
    fn children(&self) -> Vec<Layouted<dyn InternalLayoutable>> {
        let mut top_bar_rect = Rectangle::new();
        top_bar_rect.color = (255, 255, 255, 255);
        let top_bar = Layouted::new(top_bar_rect, LayoutType::AbsoluteBg);
        let mut main_area_rect = Rectangle::new();
        main_area_rect.color = (200, 200, 200, 255);
        let main_area = Layouted::new(main_area_rect, LayoutType::AbsolutePxPxGrowGrow(0, 50));
        vec![top_bar, main_area]
    }
}

impl EventHandler for App {}
