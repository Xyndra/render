use render_events::Events;
use render_layout::{
    InternalLayoutable, Primitive,
    layouting::{ConvertedPrimitive, layout},
};
use std::any::Any;

use crate::primitives::*;

pub mod empty_component;
pub mod primitives;

pub trait EventHandler: InternalLayoutable {
    fn handle_event(&mut self, event: Events) {
        match event {
            Events::Resize { width, height } => self.handle_resize(width, height),
            Events::Move { x, y } => self.handle_move(x, y),
            Events::Hover { x, y } => {
                self.handle_hover(x, y);
            }
            _ => {
                todo!(
                    "Handle other events in RenderComponent. Got event: {:?}",
                    event
                )
            }
        }
    }

    fn handle_resize(&mut self, width: u32, height: u32) {
        self.set_width(width);
        self.set_height(height);
    }
    fn handle_move(&mut self, x: u32, y: u32) {
        self.set_x(x);
        self.set_y(y);
    }

    fn handle_hover(&mut self, _x: u32, _y: u32) {
        // By default, do nothing
    }

    // WARNING: AI GENERATED METHOD.
    /// Resolve this component tree into [`render_layout::Primitive`]s for the renderer.
    ///
    /// The layout function cannot downcast `Box<dyn InternalLayoutable>` to
    /// concrete primitive types itself because `render_layout` does not know
    /// about them.  Instead it accepts a *converter* closure that receives each
    /// leaf node as a `Box<dyn Any>` and returns `Ok(Box<dyn Primitive>)` for
    /// every concrete type it recognises.
    ///
    /// Extend the converter below whenever you add a new primitive type.
    fn render(&mut self) -> Vec<Box<dyn Primitive>> {
        let converter = |any: &dyn Any| -> ConvertedPrimitive {
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
        };

        let width = self.get_width();
        let height = self.get_height();

        layout(self.as_layoutable(), width, height, &converter)
    }
}
