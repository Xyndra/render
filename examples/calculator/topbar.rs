use render_components::{
    RowComponent,
    primitives::{Rectangle, Text},
};
use render_layout::{
    InternalLayoutable,
    LayoutType::{self, AbsoluteFrFrFrFr, RowRemainder},
    Layoutable, Layouted,
};
use render_proc_macro::layoutable;

#[layoutable]
pub(crate) struct TopBar {}

impl Layoutable for TopBar {
    fn children(&self) -> Vec<Layouted<dyn InternalLayoutable>> {
        let mut bg_rect = Rectangle::new();
        bg_rect.color = (255, 255, 255, 255);
        bg_rect.rounding = (0.0, 0.0, 0.75, 0.75).into();
        let bg = Layouted::new(bg_rect, LayoutType::AbsoluteBg);
        let mut row = RowComponent::new();
        let mut row_elements: Vec<Layouted<dyn InternalLayoutable>> = vec![];
        let mut eq_text = Text::new();
        eq_text.text = "Test".into();
        row_elements.push(Layouted::new(eq_text, RowRemainder(2)));
        let mut eq_sign = Text::new();
        eq_sign.text = "=".into();
        row_elements.push(Layouted::new(eq_sign, LayoutType::RowPx(30)));
        let mut res_text = Text::new();
        res_text.text = "ABC".into();
        row_elements.push(Layouted::new(res_text, RowRemainder(1)));
        row.get_children_mut().append(&mut row_elements);
        let placed_row = Layouted::new(row, AbsoluteFrFrFrFr(0.1, 0.1, 0.8, 0.8));
        vec![bg, placed_row]
    }
}
