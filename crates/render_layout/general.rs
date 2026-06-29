use std::{error::Error, ops::Deref, slice::IterMut};

use render_events::Events;

use crate::{Area, EventHandler, InternalLayoutable, LayoutType, Primitive};

pub struct Layouted<T: ?Sized> {
    pub element: Box<T>,
    pub layout_type: LayoutType,
    /// (x, y, width, height)
    pub effective_layout: Option<(u32, u32, u32, u32)>,
}

impl Layouted<dyn InternalLayoutable> {
    pub fn new<T: InternalLayoutable + 'static>(value: T, layout_type: LayoutType) -> Self {
        Layouted {
            element: Box::new(value) as Box<dyn InternalLayoutable>,
            layout_type,
            effective_layout: None,
        }
    }
}

impl<T> Deref for Layouted<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

pub type ConvertedPrimitive = Option<Box<dyn Primitive>>;
pub type ChildIterator<'a> = IterMut<'a, Layouted<dyn InternalLayoutable + 'static>>;

// WARNING: AI generated
/// TODO: Adjust this comment to reflect refactor
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
#[allow(clippy::type_complexity)]
pub fn general_layout<T: InternalLayoutable + ?Sized>(
    base_component: &mut T,
    area: Area,
    specific_layout: &dyn Fn(Area, ChildIterator<'_>) -> Result<(), Box<dyn Error>>,
    scale: f64,
) -> Result<Vec<Box<dyn Primitive>>, Box<dyn Error>> {
    // TODO: add some protections against stack overflow since this is recursive
    let mut primitives: Vec<Box<dyn Primitive>> = Vec::new();

    specific_layout(area, base_component.get_children_mut().iter_mut())?;

    for child in base_component.get_children_mut().iter_mut() {
        let (ex, ey, ewidth, eheight) = child.effective_layout.unwrap();
        // TODO: actual layout – resolve sizing & positioning for this child
        // before collecting its shapes.
        child.element.handle_event(Events::Move { x: ex, y: ey });
        child.element.handle_event(Events::Resize {
            width: ewidth,
            height: eheight,
        });

        if child.element.children().is_empty() {
            // Leaf node – likely a primitive.  Pass a reference to try_convert
            // via `as_any()` so the concrete type is available for
            // downcasting inside the callback.
            if let Some(primitive) = child.element.into_primitive() {
                primitives.push(primitive);
            }
        } else {
            // Container node – recurse into its children.
            let child_primitives = child.element.layout(scale)?;
            primitives.extend(child_primitives);
        }
    }

    Ok(primitives)
}
