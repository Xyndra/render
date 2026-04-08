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
    WithMargin(
        Box<SizingType<'a>>,
        (
            Option<Box<SizingType<'a>>>,
            Option<Box<SizingType<'a>>>,
            Option<Box<SizingType<'a>>>,
            Option<Box<SizingType<'a>>>,
        ),
    ),
    Constrained(Box<SizingType<'a>>, Box<SizingType<'a>>),
    Dependent(&'a SizingType<'a>, fn(&SizingType<'a>) -> SizingType<'a>),
}

impl<'a> SizingType<'a> {
    pub fn between(a: SizingType<'a>, b: SizingType<'a>) -> SizingType<'a> {
        SizingType::Constrained(Box::new(a), Box::new(b))
    }

    pub fn horizontal_margin(
        s: SizingType<'a>,
        horizontal_margin: Box<SizingType<'a>>,
    ) -> SizingType<'a> {
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

    pub fn vertical_margin(
        s: SizingType<'a>,
        vertical_margin: Box<SizingType<'a>>,
    ) -> SizingType<'a> {
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
}
