use render_layout::Layoutable;
use render_proc_macro::layoutable;

/// Only for simple rectangles for efficiency. For artistic use, use other primitives!
#[layoutable(custom_default, primitive)]
pub struct Rectangle {
    pub color: (u8, u8, u8, u8),
    /// Percentages(between 0.0 and 1.0) for (left-top, right-top, left-bottom, right-bottom)
    pub rounding: Option<(f32, f32, f32, f32)>,
}

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
