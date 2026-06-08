use render_components::primitives::Rectangle;
use render_layout::{
    EventHandler, InternalLayoutable, Layoutable,
    sizing::{Sizing, SizingType},
};
use render_proc_macro::layoutable;

#[layoutable]
pub(crate) struct App {}

impl Layoutable for App {
    fn get_sizing(&self) -> Sizing {
        Sizing {
            width: SizingType::Grow(1),
            height: SizingType::Grow(1),
        }
    }
    fn children(&self) -> Vec<Box<dyn InternalLayoutable>> {
        let mut top_bar = Rectangle::new();
        top_bar.color = (255, 255, 255, 255);
        top_bar.sizing = Some(Sizing {
            width: SizingType::Grow(1),
            height: SizingType::DPICm(2.0),
        });
        let mut main_area = Rectangle::new();
        main_area.color = (200, 200, 200, 255);
        main_area.sizing = Some(Sizing {
            width: SizingType::Grow(1),
            height: SizingType::Grow(1),
        });
        vec![Box::new(main_area), Box::new(top_bar)]
    }
}

impl EventHandler for App {}
