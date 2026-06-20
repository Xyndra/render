use std::any::Any;

pub use render_layout::Primitive;
mod rectangle;
pub use rectangle::Rectangle;
mod text;
use render_layout::{ConvertedPrimitive, InternalLayoutable};
pub use text::Text;

// AI generated method
pub fn primitve_from_any(any: &dyn Any) -> ConvertedPrimitive {
    // ── Register concrete primitive types here ────────────
    if let Some(rect) = any.downcast_ref::<Rectangle>() {
        let mut new_rect = Rectangle::new();
        new_rect.color = rect.color;
        new_rect.rounding = rect.rounding;
        new_rect.set_x(rect.get_x());
        new_rect.set_y(rect.get_y());
        new_rect.set_width(rect.get_width());
        new_rect.set_height(rect.get_height());
        return Some(Box::new(new_rect));
    }
    if let Some(text) = any.downcast_ref::<Text>() {
        let mut new_text = Text::new();
        new_text.text = text.text.clone();
        new_text.color = text.color;
        new_text.font_size = text.font_size;
        new_text.set_x(text.get_x());
        new_text.set_y(text.get_y());
        new_text.set_width(text.get_width());
        new_text.set_height(text.get_height());
        return Some(Box::new(new_text));
    }
    None
}
