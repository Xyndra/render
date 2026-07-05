// WARNING: AI GENERATED; UNDER REVIEW

use render_layout::Layoutable;
use render_proc_macro::layoutable;

/// Text primitive that renders text within a rectangle.
///
/// Text is automatically word-wrapped. A `font_size` of `0` (the default)
/// enables auto-fit: the renderer picks the largest font size whose wrapped
/// text still fits inside the assigned rectangle. A non-zero `font_size`
/// requests a specific size; if the text does not fit at that size the
/// renderer shrinks it until it fits — overflows are never allowed.
#[layoutable(custom_default, primitive)]
pub struct Text {
    pub text: String,
    pub color: (u8, u8, u8, u8),
    /// Requested font size in pixels. A value of `0` (the default) enables
    /// auto-fit: the renderer selects the largest font size that fits inside
    /// the layout rectangle. A non-zero value requests a specific size; the
    /// renderer may shrink it to make the text fit.
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
