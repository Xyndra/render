mod types;

use std::rc::Rc;

#[derive(Debug, Clone, Default)]
pub struct Sizing {
    pub width: SizingType,
    pub height: SizingType,
}

type Margins = (
    Option<Box<SizingType>>,
    Option<Box<SizingType>>,
    Option<Box<SizingType>>,
    Option<Box<SizingType>>,
);

#[derive(Debug, Clone, Default)]
pub enum SizingType {
    Fixed(u32),
    DPICm(f64),
    #[default]
    FitContent,
    /// Useful for something like a spacer
    Shrink,
    /// Grow using fractional units (like 1fr in CSS)
    Grow(u8),
    /// Percentage fill
    Fill(f64),
    /// Aspect ratio to the other sizing
    AspectRatio(f64),
    /// order: left, top, right, bottom
    WithMargin(Box<SizingType>, Margins),
    Constrained(Box<SizingType>, Box<SizingType>),
    Dependent(Rc<SizingType>, fn(&SizingType) -> SizingType),
}
