use crate::{Area, ChildIterator, LayoutType};
use std::error::Error;

pub enum RowDirection {
    LTR,
    RTL,
}

pub fn row_layout(
    area: Area,
    children: ChildIterator,
    dir: RowDirection,
) -> Result<(), Box<dyn Error>> {
    let (x, y, width, height) = area;
    let mut assignable_width = width;
    let mut remainders = 0;
    // skope out width
    for child in children.as_ref() {
        match child.layout_type {
            LayoutType::RowPx(px) => {
                if let Some(new_assignable_width) = assignable_width.checked_sub(px) {
                    assignable_width = new_assignable_width;
                } else {
                    return Err("Row elements don't fit into row".into());
                }
            }
            LayoutType::RowFr(fr) => {
                if let Some(new_assignable_width) =
                    assignable_width.checked_sub(((width as f64) * (fr as f64)) as u32)
                {
                    assignable_width = new_assignable_width;
                } else {
                    return Err("Row elements don't fit into row".into());
                }
            }
            LayoutType::RowRemainder(r) => remainders += r,
            _ => {
                return Err(format!(
                    "Unsupported layout type {:?} for row layout",
                    child.layout_type
                )
                .into());
            }
        }
    }
    // actually layout
    let mut rx = match dir {
        RowDirection::LTR => x,
        RowDirection::RTL => x + width,
    };
    for child in children {
        match child.layout_type {
            LayoutType::RowPx(px) => {
                let ex = match dir {
                    RowDirection::LTR => rx,
                    RowDirection::RTL => rx - px,
                };
                child.effective_layout = Some((ex, y, px, height));
                match dir {
                    RowDirection::LTR => rx += px,
                    RowDirection::RTL => rx -= px,
                }
            }
            LayoutType::RowFr(fr) => {
                let px = ((width as f64) * (fr as f64)) as u32;
                let ex = match dir {
                    RowDirection::LTR => rx,
                    RowDirection::RTL => rx - px,
                };
                child.effective_layout = Some((ex, y, px, height));
                match dir {
                    RowDirection::LTR => rx += px,
                    RowDirection::RTL => rx -= px,
                }
            }
            LayoutType::RowRemainder(r) => {
                let px = ((width as f64) * (r as f64 / remainders as f64)) as u32;
                let ex = match dir {
                    RowDirection::LTR => rx,
                    RowDirection::RTL => rx - px,
                };
                child.effective_layout = Some((ex, y, px, height));
                match dir {
                    RowDirection::LTR => rx += px,
                    RowDirection::RTL => rx -= px,
                }
            }
            _ => {
                return Err(format!(
                    "Unsupported layout type {:?} for row layout",
                    child.layout_type
                )
                .into());
            }
        }
    }
    Ok(())
}
