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
            LayoutType::AbsoluteBg => child.effective_layout = Some((x, y, width, height)),
            LayoutType::AbsoluteFrFrFrFr(ex1, ey1, ex2, ey2) => {
                if ex1 >= 1.0 || ey1 >= 1.0 || ex2 > 1.0 || ey2 > 1.0 {
                    return Err(format!("Fraction size bigger than 1").into());
                }
                // TODO: bounds check
                child.effective_layout = Some((
                    x + (width as f32 * ex1) as u32,
                    y + (height as f32 * ey1) as u32,
                    (width as f32 * (ex2 - ex1)) as u32,
                    (height as f32 * (ey2 - ey1)) as u32,
                ))
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
