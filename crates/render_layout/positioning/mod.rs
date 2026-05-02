#[derive(Debug, Clone, Default)]
pub struct Positioning {
    pub x: PositioningType,
    pub y: PositioningType,
}

#[derive(Debug, Clone, Default)]
pub enum PositioningType {
    #[default]
    Auto,
    Relative(u32),
    /// must be between 0 and 1
    Fractional(f64),
    /// Left/Top
    Start,
    Center,
    /// Right/Bottom
    End,
}
