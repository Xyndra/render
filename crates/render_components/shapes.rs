use render_layout::Sizing;

#[derive(Debug)]
pub enum Shapes<'a> {
    Rectangle {
        sizing: Sizing<'a>,
        color: (u8, u8, u8),
        rounding: Option<u32>,
    },
    ConstrainedText {
        sizing: Sizing<'a>,
        content: String,
        font: String,
    },
}
