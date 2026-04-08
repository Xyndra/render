mod types;
pub use crate::positioning::types::PositioningType;

#[derive(Debug, Clone, Default)]
pub struct Positioning {
    pub x: Option<PositioningType>,
    pub resolved_x: Option<u32>,
    pub y: Option<PositioningType>,
    pub resolved_y: Option<u32>,
}

impl Positioning {
    pub fn new(x: PositioningType, y: PositioningType) -> Self {
        Self {
            x: Some(x),
            resolved_x: None,
            y: Some(y),
            resolved_y: None,
        }
    }
}
