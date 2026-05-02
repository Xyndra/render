use render_layout::{
    Layoutable, Primitive,
    sizing::{Sizing, SizingType},
};
use render_proc_macro::layoutable;

#[layoutable(custom_default)]
pub struct Rectangle {
    pub color: (u8, u8, u8, u8),
    pub rounding: Option<f32>,
}

impl Primitive for Rectangle {}

impl Layoutable for Rectangle {
    fn get_sizing(&'_ self) -> Sizing<'_> {
        Sizing {
            width: SizingType::Grow(1),
            height: SizingType::Grow(1),
        }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            children: vec![],
            color: (0, 0, 0, 255),
            rounding: None,
        }
    }
}
