use std::error::Error;

use render_layout::{
    EventHandler, InternalLayoutable,
    LayoutType::AbsoluteBg,
    Layoutable, Layouted, Primitive, general_layout,
    row::{RowDirection, row_layout},
};
use render_proc_macro::layoutable;

use crate::EmptyComponent;

#[layoutable(custom_layout)]
pub struct RowComponent {}

impl LayoutFunction for RowComponent {
    fn layout_func(
        &mut self,
        area: (u32, u32, u32, u32),
        scale: f64,
    ) -> Result<Vec<Box<dyn Primitive>>, Box<dyn Error>> {
        general_layout(
            self,
            area,
            &|area, children| row_layout(area, children, RowDirection::LTR),
            scale,
        )
    }
}

impl Layoutable for RowComponent {
    fn children(&self) -> Vec<Layouted<dyn InternalLayoutable>> {
        if self.children.is_empty() {
            vec![]
        } else {
            let filler = EmptyComponent::new();
            vec![Layouted::new(filler, AbsoluteBg)]
        }
    }
}

impl EventHandler for RowComponent {}
