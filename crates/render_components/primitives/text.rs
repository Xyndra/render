use render_layout::Layoutable;
use render_proc_macro::layoutable;

/// Text primitive that renders text within a rectangle.
#[layoutable(custom_default, primitive)]
pub struct Text {
    pub text: String,
    pub color: (u8, u8, u8, u8),
    /// Requested font size in pixels.
    /// The renderer may shrink it to make it fit.
    /// The default, 0, makes it use the maximum size that fits
    pub font_size: f32,
}

impl Layoutable for Text {}

impl Default for Text {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            children: vec![],
            text: String::new(),
            color: (0, 0, 0, 255),
            font_size: 0.0,
        }
    }
}
