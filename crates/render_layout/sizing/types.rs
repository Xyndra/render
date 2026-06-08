use std::cmp::min;

use crate::sizing::SizingType;

impl SizingType {
    pub fn between(a: SizingType, b: SizingType) -> SizingType {
        SizingType::Constrained(Box::new(a), Box::new(b))
    }

    pub fn horizontal_margin(s: SizingType, horizontal_margin: Box<SizingType>) -> SizingType {
        SizingType::WithMargin(
            Box::new(s),
            (
                Some(horizontal_margin.clone()),
                None,
                Some(horizontal_margin),
                None,
            ),
        )
    }

    pub fn vertical_margin(s: SizingType, vertical_margin: Box<SizingType>) -> SizingType {
        SizingType::WithMargin(
            Box::new(s),
            (
                None,
                Some(vertical_margin.clone()),
                None,
                Some(vertical_margin),
            ),
        )
    }

    pub fn get_minimum(&self, dpi: u32) -> u32 {
        match &self {
            SizingType::Fixed(f) => *f,
            SizingType::DPICm(cm) => ((cm / 2.54) * dpi as f64) as u32,
            SizingType::Grow(_) => 0,
            SizingType::Constrained(a, b) => min(a.get_minimum(dpi), b.get_minimum(dpi)),
            SizingType::WithMargin(_s, _) => todo!(),
            SizingType::FitContent => todo!(),
            SizingType::Shrink => todo!(),
            SizingType::Fill(_) => todo!(),
            SizingType::AspectRatio(_) => todo!(),
            SizingType::Dependent(_s, _) => todo!(),
        }
    }
}
