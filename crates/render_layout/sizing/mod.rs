mod types;
pub use crate::sizing::types::SizingType;

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
            width: width,
            resolved_width: None,
            height: height,
            resolved_height: None,
        }
    }
}
