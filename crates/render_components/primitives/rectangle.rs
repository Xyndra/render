use render_layout::{
    Layoutable, Primitive,
    sizing::{Sizing, SizingType},
};
use render_proc_macro::layoutable;

/// Only for simple rectangles for efficiency. For artistic use, use other primitives!
#[layoutable(custom_default)]
pub struct Rectangle {
    pub sizing: Option<Sizing>,
    pub color: (u8, u8, u8, u8),
    pub rounding: Option<f32>,
}

impl Primitive for Rectangle {}

impl Layoutable for Rectangle {
    fn get_sizing(&self) -> Sizing {
        if let Some(sizing) = &self.sizing {
            return sizing.clone();
        }
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
            sizing: None,
            color: (0, 0, 0, 255),
            rounding: None,
        }
    }
}
