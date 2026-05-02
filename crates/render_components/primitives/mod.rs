use std::any::Any;

pub use render_layout::Primitive;
mod rectangle;
pub use rectangle::Rectangle;
use render_layout::{InternalLayoutable, layouting::ConvertedPrimitive};

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
    // Add more arms: if let Some(c) = any.downcast_ref::<Circle>() { ... }
    None
}
