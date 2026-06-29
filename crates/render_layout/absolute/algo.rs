use crate::{Area, ChildIterator, LayoutType};
use std::error::Error;

pub fn absolute_layout(area: Area, children: ChildIterator) -> Result<(), Box<dyn Error>> {
    let (x, y, width, height) = area;
    for child in children {
        match child.layout_type {
            LayoutType::AbsolutePxPxGrowGrow(ex, ey) => {
                bounds_check(width, height, ex, ey)?;
                child.effective_layout = Some((x + ex, y + ey, width - ex, height - ey));
            }
            LayoutType::AbsoluteBg => child.effective_layout = Some((x, y, width, height)),
            LayoutType::AbsoluteFrFrFrFr(ex1, ey1, ex2, ey2) => {
                if ex1 >= 1.0 || ey1 >= 1.0 || ex2 > 1.0 || ey2 > 1.0 {
                    return Err("Fraction size bigger than 1".into());
                }
                // TODO: bounds check
                child.effective_layout = Some((
                    x + (width as f32 * ex1) as u32,
                    y + (height as f32 * ey1) as u32,
                    (width as f32 * (ex2 - ex1)) as u32,
                    (height as f32 * (ey2 - ey1)) as u32,
                ))
            }
            LayoutType::AbsolutePxPxGrowPx(ex, ey, h) => {
                bounds_check(width, height, ex, ey)?;
                bounds_check(width, height, ex, h)?;
                child.effective_layout = Some((x + ex, y + ey, width - ex, y + h));
            }
            LayoutType::AbsolutePxPxPxGrow(ex, ey, w) => {
                bounds_check(width, height, ex, ey)?;
                bounds_check(width, height, w, ey)?;
                child.effective_layout = Some((x + ex, y + ey, x + w, height - ey));
            }
            LayoutType::AbsolutePxPxPxPx(ex, ey, w, h) => {
                bounds_check(width, height, ex, ey)?;
                bounds_check(width, height, w, h)?;
                child.effective_layout = Some((x + ex, y + ey, x + w, y + h));
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
