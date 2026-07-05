// WARNING: AI GENERATED; UNDER REVIEW

use render::run_default;
use render_components::primitives::{Rectangle, Text};
use render_layout::{EventHandler, InternalLayoutable, LayoutType, Layoutable, Layouted};
use render_proc_macro::layoutable;

/// A showcase of the Text primitive with diverse character sets:
/// ASCII, extended Latin, Arabic (RTL + shaping), and compound emojis.
#[layoutable]
pub(crate) struct App {}

impl Layoutable for App {
    fn children(&self) -> Vec<Layouted<dyn InternalLayoutable>> {
        let mut children: Vec<Layouted<dyn InternalLayoutable>> = Vec::new();

        // ── Row 1 — ASCII ──────────────────────────────────────────
        children.push(bg_rect(0.02, 0.02, 0.98, 0.14, (240, 240, 240, 255)));
        children.push(make_text(
            "ASCII: The quick brown fox jumps over the lazy dog. 0123456789 !@#$%^&*()",
            28.0,
            (0, 0, 0, 255),
            0.03,
            0.03,
            0.97,
            0.13,
        ));

        // ── Row 2 — Extended Latin / diacritics ────────────────────
        children.push(bg_rect(0.02, 0.16, 0.98, 0.28, (230, 240, 255, 255)));
        children.push(make_text(
            "Extended: Ñoño café résumé naïve über ångström ß Ørsted Łódź Əмали",
            26.0,
            (0, 0, 0, 255),
            0.03,
            0.17,
            0.97,
            0.27,
        ));

        // ── Row 3 — Arabic (RTL + shaping) ─────────────────────────
        children.push(bg_rect(0.02, 0.30, 0.98, 0.46, (255, 240, 230, 255)));
        children.push(make_text(
            "Arabic: مرحبا بالعالم هذا اختبار للنص العربي",
            30.0,
            (0, 0, 0, 255),
            0.03,
            0.31,
            0.97,
            0.45,
        ));

        // ── Row 4 — Compound / ZWJ emojis ──────────────────────────
        children.push(bg_rect(0.02, 0.48, 0.98, 0.64, (230, 255, 230, 255)));
        children.push(make_text(
            "Emojis: 🐕‍🦺 👨‍👩‍👧‍👦 🏳️‍🌈 👩‍💻 🧑‍🚀👨‍🍳",
            32.0,
            (0, 0, 0, 255),
            0.03,
            0.49,
            0.97,
            0.63,
        ));

        // ── Row 5 — CJK ───────────────────────────────────────────
        children.push(bg_rect(0.02, 0.66, 0.98, 0.78, (255, 230, 255, 255)));
        children.push(make_text(
            "CJK: 日本語テスト 你好世界 한국어 테스트",
            28.0,
            (0, 0, 0, 255),
            0.03,
            0.67,
            0.97,
            0.77,
        ));

        // ── Row 6 — Mixed with auto-fit (large font, will shrink) ──
        children.push(bg_rect(0.02, 0.80, 0.98, 0.98, (245, 245, 220, 255)));
        children.push(make_text(
            "Auto-fit: This text starts at 48px but the renderer shrinks it to fit within the rectangle boundaries without any overflow!",
            48.0,
            (50, 50, 150, 255),
            0.03, 0.81, 0.97, 0.97,
        ));

        children
    }
}

impl EventHandler for App {}

fn bg_rect(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: (u8, u8, u8, u8),
) -> Layouted<dyn InternalLayoutable> {
    let mut rect = Rectangle::new();
    rect.color = color;
    Layouted::new(rect, LayoutType::AbsoluteFrFrFrFr(x1, y1, x2 - x1, y2 - y1))
}

fn make_text(
    content: &str,
    font_size: f32,
    color: (u8, u8, u8, u8),
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> Layouted<dyn InternalLayoutable> {
    let mut text = Text::new();
    text.text = content.to_string();
    text.font_size = font_size;
    text.color = color;
    Layouted::new(text, LayoutType::AbsoluteFrFrFrFr(x1, y1, x2 - x1, y2 - y1))
}

fn main() {
    let mut component = App::default();
    component.children = component.children();
    run_default(component);
}
