mod types;

#[derive(Debug, Clone, Default)]
pub struct Sizing<'a> {
    pub width: SizingType<'a>,
    pub height: SizingType<'a>,
}

type Margins<'a> = (
    Option<Box<SizingType<'a>>>,
    Option<Box<SizingType<'a>>>,
    Option<Box<SizingType<'a>>>,
    Option<Box<SizingType<'a>>>,
);

#[derive(Debug, Clone, Default)]
pub enum SizingType<'a> {
    Fixed(u32),
    DPI(u32),
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
    WithMargin(Box<SizingType<'a>>, Margins<'a>),
    Constrained(Box<SizingType<'a>>, Box<SizingType<'a>>),
    Dependent(&'a SizingType<'a>, fn(&SizingType<'a>) -> SizingType<'a>),
}
