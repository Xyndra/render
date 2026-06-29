// WARNING: AI GENERATED; UNDER REVIEW

use render_layout::Layoutable;
use render_proc_macro::layoutable;

/// Text primitive that renders text within a rectangle.
///
/// Text is automatically word-wrapped. If the text does not fit at thprimitivee
/// requested `font_size`, the renderer will reduce the font size until
/// the text fits within the assigned rectangle — overflows are never allowed.
#[layoutable(custom_default, primitive)]
pub struct Text {
    pub text: String,
    pub color: (u8, u8, u8, u8),
    /// Requested font size in pixels. The actual rendered size may be
    /// smaller so that the text fits inside the layout rectangle.
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
            font_size: 16.0,
        }
    }
}
