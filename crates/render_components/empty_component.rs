use crate::primitives::Rectangle;
use render_layout::{
    EventHandler, InternalLayoutable, Layoutable,
    sizing::{Sizing, SizingType},
};
use render_proc_macro::layoutable;

#[layoutable]
pub struct EmptyComponent {
    pub color: (u8, u8, u8, u8),
}

impl Layoutable for EmptyComponent {
    fn get_sizing(&'_ self) -> Sizing<'_> {
        Sizing {
            width: SizingType::Grow(1),
            height: SizingType::Grow(1),
        }
    }
    fn children(&self) -> Vec<Box<dyn InternalLayoutable>> {
        let mut rect = Rectangle::new();
        println!("Color: {:?}", self.color);
        rect.color = self.color;
        vec![Box::new(rect)]
    }
}

impl EventHandler for EmptyComponent {}
