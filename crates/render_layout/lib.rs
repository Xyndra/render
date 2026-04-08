// TODO: reference clay
pub mod positioning;
pub mod sizing;

pub struct Layout<'a> {
    pub sizing: sizing::Sizing<'a>,
    pub positioning: positioning::Positioning,
}
