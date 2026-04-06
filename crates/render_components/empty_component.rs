use crate::{RenderComponent, Shapes};
use render_layout::{Sizing, SizingType};

#[derive(Default)]
pub struct EmptyComponent {
    width: u32,
    height: u32,
}
impl RenderComponent for EmptyComponent {
    fn get_sizing(&'_ self) -> Sizing<'_> {
        Sizing::new(SizingType::Grow(1), SizingType::Grow(1))
    }

    fn render(&'_ self) -> Vec<Shapes<'_>> {
        vec![Shapes::Rectangle {
            sizing: Sizing::new(SizingType::Grow(1), SizingType::Grow(1)),
            color: (255, 255, 0),
            rounding: None,
        }]
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}
