use render_events::Events;

use crate::{EventHandler, InternalLayoutable, Primitive};
use std::any::Any;

pub type ConvertedPrimitive = Option<Box<dyn Primitive>>;

// WARNING: AI generated
/// Resolve a component tree into a flat list of [`Primitive`]s for the renderer.
///
/// The `try_convert` callback is called for every **leaf** child (i.e. one
/// whose `children()` returns an empty vec).  It receives the child as a
/// `Box<dyn Any>` (converted via [`InternalLayoutable::into_any`]) and
/// should return `Some(Box<dyn Primitive>)` for every concrete primitive
/// type it recognises.  If the type is unknown the callback returns `None`
/// and the leaf is silently skipped.
///
/// Container children (those whose `children()` is non-empty) are recursed
/// into automatically – they are never passed to `try_convert`.
pub fn layout<T: InternalLayoutable + ?Sized>(
    base_component: &mut T,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    try_convert: &dyn Fn(&dyn Any) -> ConvertedPrimitive,
) -> Vec<Box<dyn Primitive>> {
    // TODO: add some protections against stack overflow since this is recursive
    let mut primitives: Vec<Box<dyn Primitive>> = Vec::new();

    for child in base_component.get_children_mut().iter_mut() {
        // TODO: actual layout – resolve sizing & positioning for this child
        // before collecting its shapes.
        child.handle_event(Events::Move { x, y });
        child.handle_event(Events::Resize { width, height });

        if child.children().is_empty() {
            // Leaf node – likely a primitive.  Pass a reference to try_convert
            // via `as_any()` so the concrete type is available for
            // downcasting inside the callback.
            if let Some(primitive) = try_convert(child.as_any()) {
                primitives.push(primitive);
            }
        } else {
            // Container node – recurse into its children.
            let child_primitives = child.layout(try_convert);
            primitives.extend(child_primitives);
        }
    }

    primitives
}
