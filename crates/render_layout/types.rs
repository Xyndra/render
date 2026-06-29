#[derive(Debug, Clone, Copy)]
pub enum LayoutType {
    AbsolutePxPxPxPx(u32, u32, u32, u32),
    AbsoluteDDDD(f64, f64, f64, f64),
    AbsoluteFrFrFrFr(f32, f32, f32, f32),
    AbsolutePxPxPxGrow(u32, u32, u32),
    AbsolutePxPxGrowPx(u32, u32, u32),
    AbsolutePxPxGrowGrow(u32, u32),
    AbsoluteBg,
    // Possibly add mixed variants using a macro
}
