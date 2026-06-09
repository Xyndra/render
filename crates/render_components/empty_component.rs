use crate::primitives::Rectangle;
use render_layout::{EventHandler, InternalLayoutable, LayoutType, Layoutable, Layouted};
use render_proc_macro::layoutable;

#[layoutable]
pub struct EmptyComponent {
    pub color: (u8, u8, u8, u8),
}

impl Layoutable for EmptyComponent {
    fn children(&self) -> Vec<Layouted<dyn InternalLayoutable>> {
        let mut rect = Rectangle::new();
        println!("Color: {:?}", self.color);
        rect.color = self.color;
        vec![Layouted::new(rect, LayoutType::AbsolutePxPxGrowGrow(0, 0))]
    }
}

impl EventHandler for EmptyComponent {}
