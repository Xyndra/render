use render_components::primitives::Rectangle;
use render_layout::{EventHandler, InternalLayoutable, LayoutType, Layoutable, Layouted};
use render_proc_macro::layoutable;

use crate::topbar::TopBar;

#[layoutable]
pub(crate) struct App {}

impl Layoutable for App {
    fn children(&self) -> Vec<Layouted<dyn InternalLayoutable>> {
        let top_bar = Layouted::new(TopBar::new(), LayoutType::AbsolutePxPxGrowPx(0, 0, 80));
        let mut main_area_rect = Rectangle::new();
        main_area_rect.color = (200, 200, 200, 255);
        let main_area = Layouted::new(main_area_rect, LayoutType::AbsoluteBg);
        vec![main_area, top_bar]
    }
}

impl EventHandler for App {}
