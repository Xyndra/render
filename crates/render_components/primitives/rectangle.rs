use render_layout::{Layoutable, Primitive};
use render_proc_macro::layoutable;

/// Only for simple rectangles for efficiency. For artistic use, use other primitives!
#[layoutable(custom_default)]
pub struct Rectangle {
    pub color: (u8, u8, u8, u8),
    pub rounding: Option<f32>,
}

impl Primitive for Rectangle {}

impl Layoutable for Rectangle {}

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
