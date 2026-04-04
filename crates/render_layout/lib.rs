// TODO: reference clay

#[derive(Debug, Clone, Default)]
pub enum SizingType<'a> {
    Fixed(u32),
    DPI(u32),
    #[default]
    FitContent,
    // Useful for something like a spacer
    Shrink,
    // Grow using fractional units (like 1fr in CSS)
    Grow(u8),
    // Percentage fill
    Fill(f64),
    // Aspect ratio to the other sizing
    AspectRatio(f64),
    Constrained(Box<Sizing<'a>>, Box<Sizing<'a>>),
    Dependent(&'a Sizing<'a>, fn(&Sizing<'a>) -> Sizing<'a>),
}

#[derive(Debug, Clone, Default)]
pub struct Sizing<'a> {
    pub width: SizingType<'a>,
    pub resolved_width: Option<u32>,
    pub height: SizingType<'a>,
    pub resolved_height: Option<u32>,
}

impl<'a> Sizing<'a> {
    pub fn new(width: SizingType<'a>, height: SizingType<'a>) -> Self {
        Self {
            width,
            resolved_width: None,
            height,
            resolved_height: None,
        }
    }
}
