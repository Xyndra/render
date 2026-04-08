use crate::{RenderComponent, Shapes};
use render_layout::{
    positioning::{Positioning, PositioningType},
    sizing::{Sizing, SizingType},
};

#[derive(Default)]
pub struct EmptyComponent {
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}
impl RenderComponent for EmptyComponent {
    fn get_sizing(&'_ self) -> Sizing<'_> {
        Sizing::new(SizingType::Grow(1), SizingType::Grow(1))
    }

    fn render(&'_ self) -> Vec<Shapes<'_>> {
        vec![Shapes::Rectangle {
            // sizing: Sizing::new(SizingType::Grow(1), SizingType::Grow(1)),
            sizing: Sizing {
                width: SizingType::Grow(1),
                resolved_width: Some(self.width),
                height: SizingType::Grow(1),
                resolved_height: Some(self.height),
            },
            // position: Positioning::default(),
            position: Positioning {
                x: Some(PositioningType::Auto),
                resolved_x: Some(self.x),
                y: Some(PositioningType::Auto),
                resolved_y: Some(self.y),
            },
            color: (255, 255, 0),
            rounding: None,
        }]
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn reposition(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
    }
}
