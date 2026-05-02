use crate::sizing::SizingType;

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
