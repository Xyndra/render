use render_layout::{positioning::Positioning, sizing::Sizing};

#[derive(Debug)]
pub enum Shapes<'a> {
    Rectangle {
        sizing: Sizing<'a>,
        position: Positioning,
        color: (u8, u8, u8),
        rounding: Option<u32>,
    },
    /// For artistic use, like in a headline
    ConstrainedText {
        sizing: Sizing<'a>,
        position: Positioning,
        content: String,
        font: String,
    },
}
