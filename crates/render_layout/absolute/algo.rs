use crate::{InternalLayoutable, LayoutType, Layouted};
use std::{error::Error, slice::IterMut};

pub fn absolute_layout(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    children: IterMut<'_, Layouted<dyn InternalLayoutable + 'static>>,
) -> Result<(), Box<dyn Error>> {
    for child in children {
        match child.layout_type {
            LayoutType::AbsolutePxPxGrowGrow(ex, ey) => {
                bounds_check(width, height, ex, ey)?;
                child.effective_layout = Some((x + ex, y + ey, width - ex, height - ey));
            }
            _ => {
                return Err(format!(
                    "Unsupported layout type {:?} for absolute layout",
                    child.layout_type
                )
                .into());
            }
        }
    }
    Ok(())
}

fn bounds_check(w: u32, h: u32, x: u32, y: u32) -> Result<(), Box<dyn Error>> {
    if x > w || y > h {
        return Err(format!("({}, {}) is out of bounds for size ({}, {})", x, y, w, h).into());
    }
    Ok(())
}
