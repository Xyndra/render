// WARNING: AI GENERATED; UNDER REVIEW

use std::ops::Range;
use std::slice;

use ab_glyph::{Font, FontArc, GlyphId, PxScale, ScaleFont};
use harfrust::{Direction, ShapeOptions, ShaperData, UnicodeBuffer, script};
use unicode_bidi::{BidiInfo, Level};
use unicode_segmentation::UnicodeSegmentation;

use crate::debug;

use super::{LINE_HEIGHT_RATIO, MIN_FONT_SIZE};

// ── Script & direction detection (harfrust) ───────────────────

/// Detect the script and direction of a text run.
///
/// Uses harfrust's HarfBuzz-compatible segment property guessing: the script
/// is derived from the first strong character and the direction from the
/// script.  This replaces a hand-rolled `unicode-script` lookup.
fn detect_script_and_dir(text: &str) -> (harfrust::Script, Direction) {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.guess_segment_properties();
    (buffer.script(), buffer.direction())
}

// ── Font group tracking ────────────────────────────────────────

/// Ranges of font indices for each priority group.
pub(super) struct FontGroupRanges {
    pub(super) latin: Range<usize>,
    pub(super) cjk: Range<usize>,
    pub(super) arabic: Range<usize>,
    pub(super) emoji: Range<usize>,
}

// ── Font helpers ───────────────────────────────────────────────

/// Find the first font that contains a glyph for `c`.
/// Returns the font index, or falls back to 0 if none found.
fn find_font_for_char(c: char, fonts: &[FontArc]) -> usize {
    for (i, font) in fonts.iter().enumerate() {
        if font.glyph_id(c).0 != 0 {
            return i;
        }
    }
    0
}

/// Does this token contain emoji that should be rendered with a colour font?
///
/// Uses the `emojis` crate (Unicode v17 data) instead of hand-rolled ranges,
/// so it stays accurate as new emoji are added.  A token is emoji if it is a
/// complete emoji sequence (flags, ZWJ emoji, keycaps, …) or contains any
/// single-character emoji.
fn is_emoji_token(token: &str) -> bool {
    if emojis::get(token).is_some() {
        return true;
    }
    let mut buf = [0u8; 4];
    token
        .chars()
        .any(|c| emojis::get(c.encode_utf8(&mut buf)).is_some())
}

/// Pick the best font for a piece of text (a word *or* a single grapheme
/// cluster).
///
/// Prefers fonts from the group matching the text's script, then falls back
/// to any font that covers every non-whitespace character, then to the font
/// covering the most characters.  This is the core of the font-fallback
/// system and is called per grapheme cluster so that mixed-script text can
/// use a different font for each cluster.
fn find_best_font_for_text(
    text: &str,
    fonts: &[FontArc],
    script: harfrust::Script,
    groups: &FontGroupRanges,
) -> usize {
    let covers = |font: &FontArc| -> bool {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .all(|c| font.glyph_id(c).0 != 0)
    };

    // Determine preferred group based on script.
    let preferred: &Range<usize> = if is_emoji_token(text) {
        &groups.emoji
    } else if script == script::ARABIC || script == script::HEBREW {
        &groups.arabic
    } else if matches!(
        script,
        script::HAN | script::HIRAGANA | script::KATAKANA | script::HANGUL
    ) {
        &groups.cjk
    } else {
        &groups.latin
    };

    // Try preferred group first.
    for i in preferred.clone() {
        if i < fonts.len() && covers(&fonts[i]) {
            return i;
        }
    }

    // Fall back to all fonts.
    for (i, font) in fonts.iter().enumerate() {
        if covers(font) {
            return i;
        }
    }

    // Best-effort: pick font covering the most characters.
    fonts
        .iter()
        .enumerate()
        .max_by_key(|(_, font)| {
            text.chars()
                .filter(|c| !c.is_whitespace())
                .filter(|c| font.glyph_id(*c).0 != 0)
                .count()
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}

// ── Text shaping with harfrust ─────────────────────────────────

struct ShapedGlyph {
    glyph_id: u32,
    x_advance: f32,
    x_offset: f32,
    y_offset: f32,
    /// Which font in the fallback list produced this glyph.
    font_index: u16,
}

/// Shape a text segment with harfrust using the given font.
fn shape_run(
    text: &str,
    font_data: &[u8],
    face_index: u32,
    font_size: f32,
    direction: Direction,
    script: harfrust::Script,
) -> Vec<ShapedGlyph> {
    let font_ref = match harfrust::FontRef::from_index(font_data, face_index) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let shaper_data = ShaperData::new(&font_ref);
    let shaper = shaper_data.shaper(&font_ref).build();

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(direction);
    buffer.set_script(script);

    let output = shaper.shape(buffer, ShapeOptions::new());
    let infos = output.glyph_infos();
    let positions = output.glyph_positions();

    let units_per_em = shaper.units_per_em() as f32;
    let scale = font_size / units_per_em;

    debug!(
        "[SHAPE] text='{}' font_size={:.1} units_per_em={:.1} scale={:.6}",
        text, font_size, units_per_em, scale
    );

    let result: Vec<ShapedGlyph> = infos
        .iter()
        .zip(positions.iter())
        .map(|(info, pos)| {
            let rb_adv = pos.x_advance as f32 * scale;
            let rb_xoff = pos.x_offset as f32 * scale;
            let rb_yoff = pos.y_offset as f32 * scale;
            debug!(
                "[SHAPE]   gid={} rb_advance={:.2} rb_x_offset={:.2} rb_y_offset={:.2}",
                info.glyph_id, rb_adv, rb_xoff, rb_yoff
            );
            ShapedGlyph {
                glyph_id: info.glyph_id,
                x_advance: rb_adv,
                x_offset: rb_xoff,
                y_offset: rb_yoff,
                font_index: 0,
            }
        })
        .collect();
    result
}

/// Shape a whole token using per-grapheme-cluster font fallback.
///
/// Each grapheme cluster is assigned a font via [`find_best_font_for_text`];
/// consecutive clusters that share a font are merged into a single run and
/// shaped together (so kerning and ligatures still apply within a run), then
/// the runs are concatenated.  Advances are taken from `ab_glyph` so they
/// match the metrics used for rasterisation.  Every glyph is tagged with the
/// font that produced it.
fn shape_token(
    token: &str,
    fonts: &[FontArc],
    font_data: &[Vec<u8>],
    face_indices: &[u32],
    font_size: f32,
    font_groups: &FontGroupRanges,
) -> Vec<ShapedGlyph> {
    // Group consecutive grapheme clusters that resolve to the same font.
    let mut runs: Vec<(String, u16)> = Vec::new();
    for cluster in token.graphemes(true) {
        let (script, _) = detect_script_and_dir(cluster);
        let font_idx = find_best_font_for_text(cluster, fonts, script, font_groups) as u16;
        match runs.last_mut() {
            Some(last) if last.1 == font_idx => last.0.push_str(cluster),
            _ => runs.push((cluster.to_string(), font_idx)),
        }
    }

    let mut out: Vec<ShapedGlyph> = Vec::new();
    for (run_text, font_idx) in &runs {
        let (script, direction) = detect_script_and_dir(run_text);
        let mut shaped = shape_run(
            run_text,
            &font_data[*font_idx as usize],
            face_indices[*font_idx as usize],
            font_size,
            direction,
            script,
        );
        let scaled = fonts[*font_idx as usize].as_scaled(PxScale::from(font_size));
        for g in &mut shaped {
            g.x_advance = scaled.h_advance(GlyphId(g.glyph_id as u16));
            g.font_index = *font_idx;
        }
        out.extend(shaped);
    }
    out
}

// ── Word wrapping ──────────────────────────────────────────────

pub(super) struct LayoutGlyph {
    pub(super) glyph_id: u32,
    pub(super) x: f32,
    pub(super) x_offset: f32,
    pub(super) y_offset: f32,
    pub(super) font_index: u16,
}

pub(super) struct LayoutLine {
    pub(super) glyphs: Vec<LayoutGlyph>,
}

/// Tokenize text into alternating whitespace / non-whitespace runs,
/// using grapheme clusters so that ZWJ emoji sequences stay intact.
fn tokenize_by_graphemes(text: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start: usize = 0;
    let mut pos: usize = 0;
    let mut prev_ws = true; // treat start as whitespace so leading words begin a new token

    for g in text.graphemes(true) {
        let is_ws = g.chars().all(|c| c.is_whitespace());
        if pos > start && is_ws != prev_ws {
            tokens.push(&text[start..pos]);
            start = pos;
        }
        prev_ws = is_ws;
        pos += g.len();
    }
    if start < text.len() {
        tokens.push(&text[start..]);
    }
    tokens
}

/// Shape and word-wrap `text` at the given `font_size` so that no line exceeds
/// `max_width` pixels.  Every token goes through harfrust shaping with
/// per-cluster font fallback for proper kerning, ligatures, and complex-script
/// support.  Uses Unicode BiDi for proper RTL/LTR reordering.
fn shape_and_wrap(
    text: &str,
    fonts: &[FontArc],
    font_data: &[Vec<u8>],
    face_indices: &[u32],
    font_size: f32,
    max_width: f32,
    font_groups: &FontGroupRanges,
) -> Vec<LayoutLine> {
    if text.is_empty() {
        return vec![LayoutLine { glyphs: vec![] }];
    }

    let tokens = tokenize_by_graphemes(text);

    // ── Unicode BiDi reordering ───────────────────────────────────
    // Compute embedding level for each character, then assign each token
    // the maximum level of its characters.  Use `reorder_visual` on the
    // token-level sequence to get the correct visual order.
    let bidi_info = BidiInfo::new(text, None);
    let para = &bidi_info.paragraphs[0];
    let para_level = para.level;
    let levels = &bidi_info.levels[para.range.clone()];

    // Assign each token a level (max of its character levels).
    let mut token_levels: Vec<Level> = Vec::with_capacity(tokens.len());
    let mut byte_pos: usize = para.range.start;
    for token in &tokens {
        let token_start = byte_pos;
        let token_end = byte_pos + token.len();
        let mut max_level = para_level;
        // Find max level for characters in this token.
        // Map byte positions back into the paragraph's level slice.
        for (i, _) in token.char_indices() {
            let abs_byte = token_start + i;
            let rel = abs_byte - para.range.start;
            if rel < levels.len() {
                max_level = std::cmp::max(max_level, levels[rel]);
            }
        }
        token_levels.push(max_level);
        byte_pos = token_end;
    }

    // Reorder tokens into visual order.
    let reorder = BidiInfo::reorder_visual(&token_levels);
    let ordered_tokens: Vec<&str> = reorder.iter().map(|&i| tokens[i]).collect();

    let space_font_idx = find_font_for_char(' ', fonts);
    let space_scaled = fonts[space_font_idx].as_scaled(PxScale::from(font_size));
    let space_adv = space_scaled.h_advance(fonts[space_font_idx].glyph_id(' '));

    let mut lines: Vec<LayoutLine> = Vec::new();
    let mut cur_glyphs: Vec<LayoutGlyph> = Vec::new();
    let mut cur_width: f32 = 0.0;

    for token in &ordered_tokens {
        if token.is_empty() {
            continue;
        }

        if token.chars().all(|c| c.is_whitespace()) {
            cur_width += space_adv;
            continue;
        }

        // Shape the token with per-cluster font fallback so mixed-script text
        // can use a different font for each grapheme cluster.
        let shaped = shape_token(
            token,
            fonts,
            font_data,
            face_indices,
            font_size,
            font_groups,
        );

        let word_width: f32 = shaped.iter().map(|g| g.x_advance).sum();

        // Helper closure: push shaped glyphs for a word.
        let push_word =
            |cur_glyphs: &mut Vec<LayoutGlyph>, cur_width: &mut f32, shaped: &[ShapedGlyph]| {
                for g in shaped {
                    cur_glyphs.push(LayoutGlyph {
                        glyph_id: g.glyph_id,
                        x: *cur_width,
                        x_offset: g.x_offset,
                        y_offset: g.y_offset,
                        font_index: g.font_index,
                    });
                    *cur_width += g.x_advance;
                }
            };

        if cur_glyphs.is_empty() {
            // First word on the line.
            if word_width <= max_width {
                push_word(&mut cur_glyphs, &mut cur_width, &shaped);
            } else {
                // Word too long — break across lines.
                for g in &shaped {
                    if cur_width + g.x_advance > max_width && !cur_glyphs.is_empty() {
                        flush_line(&mut lines, &mut cur_glyphs, &mut cur_width);
                    }
                    push_word(&mut cur_glyphs, &mut cur_width, slice::from_ref(g));
                }
            }
        } else if cur_width + word_width <= max_width {
            // Word fits on current line.
            push_word(&mut cur_glyphs, &mut cur_width, &shaped);
        } else {
            // Start a new line.
            flush_line(&mut lines, &mut cur_glyphs, &mut cur_width);
            if word_width <= max_width {
                push_word(&mut cur_glyphs, &mut cur_width, &shaped);
            } else {
                for g in &shaped {
                    if cur_width + g.x_advance > max_width && !cur_glyphs.is_empty() {
                        flush_line(&mut lines, &mut cur_glyphs, &mut cur_width);
                    }
                    push_word(&mut cur_glyphs, &mut cur_width, slice::from_ref(g));
                }
            }
        }
    }

    if !cur_glyphs.is_empty() {
        lines.push(LayoutLine { glyphs: cur_glyphs });
    }
    if lines.is_empty() {
        lines.push(LayoutLine { glyphs: vec![] });
    }
    lines
}

fn flush_line(lines: &mut Vec<LayoutLine>, glyphs: &mut Vec<LayoutGlyph>, width: &mut f32) {
    lines.push(LayoutLine {
        glyphs: std::mem::take(glyphs),
    });
    *width = 0.0;
}

/// Compute the total rendered height of `lines` at the given `size`.
///
/// The first line contributes `ascent + descent`; each subsequent line adds
/// `size * LINE_HEIGHT_RATIO`.
fn total_height(fonts: &[FontArc], size: f32, lines: &[LayoutLine]) -> f32 {
    let scaled = fonts[0].as_scaled(PxScale::from(size));
    let ascent = scaled.ascent();
    let descent = scaled.descent();
    let line_h = size * LINE_HEIGHT_RATIO;
    let n = lines.len() as f32;
    ascent - descent + (n - 1.0).max(0.0) * line_h
}

/// Compute the width of the widest line, using the same advance-based metric
/// that [`shape_and_wrap`] uses for wrapping decisions.  A line whose width
/// exceeds `max_width` means a single glyph (or unbreakable token) is too wide
/// for the column — the font size must shrink further.
fn max_line_width(fonts: &[FontArc], size: f32, lines: &[LayoutLine]) -> f32 {
    let mut max_w = 0.0f32;
    for line in lines {
        if line.glyphs.is_empty() {
            continue;
        }
        let last = line.glyphs.last().unwrap();
        let scaled = fonts[last.font_index as usize].as_scaled(PxScale::from(size));
        let advance = scaled.h_advance(GlyphId(last.glyph_id as u16));
        max_w = max_w.max(last.x + advance);
    }
    max_w
}

/// Does the laid-out text at `size` fit within `max_width × max_height`?
fn fits(
    fonts: &[FontArc],
    size: f32,
    lines: &[LayoutLine],
    max_width: f32,
    max_height: f32,
) -> bool {
    total_height(fonts, size, lines) <= max_height
        && max_line_width(fonts, size, lines) <= max_width
}

/// Compute the text layout that fits into `max_width × max_height`.
///
/// When `requested_size` is `0` (auto-fit mode) the renderer searches up to
/// `max_height` for the *largest* font size whose wrapped text fits inside the
/// rectangle.  A non-zero `requested_size` caps the search at that value — the
/// text is never rendered larger than requested.
///
/// Both paths check width *and* height: a single glyph wider than the column
/// (e.g. `=` in a narrow box) forces a smaller size, and excessive wrapping
/// from too-wide words can no longer push the bottom edge past the box.
///
/// Returns `(actual_font_size, lines)`.
pub(super) fn fit_text(
    text: &str,
    fonts: &[FontArc],
    font_data: &[Vec<u8>],
    face_indices: &[u32],
    requested_size: f32,
    max_width: f32,
    max_height: f32,
    font_groups: &FontGroupRanges,
) -> (f32, Vec<LayoutLine>) {
    if text.is_empty() || max_width <= 0.0 || max_height <= 0.0 {
        return (requested_size, vec![LayoutLine { glyphs: vec![] }]);
    }

    // Upper bound: auto-fit (requested_size <= 0) searches up to max_height;
    // an explicit size caps the search at the requested value.
    let upper = if requested_size <= 0.0 {
        max_height.max(MIN_FONT_SIZE)
    } else {
        requested_size.max(MIN_FONT_SIZE)
    };

    fit_text_search(
        text,
        fonts,
        font_data,
        face_indices,
        max_width,
        max_height,
        font_groups,
        upper,
    )
}

/// Binary-search for the largest font size whose wrapped text fits inside
/// `max_width × max_height`, bounded above by `upper`.
///
/// The search uses 0.1 px resolution and checks both dimensions via [`fits`],
/// so a single glyph wider than the column forces a smaller size just as an
/// over-tall multi-line layout does.
fn fit_text_search(
    text: &str,
    fonts: &[FontArc],
    font_data: &[Vec<u8>],
    face_indices: &[u32],
    max_width: f32,
    max_height: f32,
    font_groups: &FontGroupRanges,
    upper: f32,
) -> (f32, Vec<LayoutLine>) {
    let min = MIN_FONT_SIZE;
    let max = upper.max(min);

    // Establish the lower bound — if even the minimum doesn't fit, return it
    // (the text overflows at the smallest supported size).
    let mut best_size = min;
    let mut best_lines = shape_and_wrap(
        text,
        fonts,
        font_data,
        face_indices,
        min,
        max_width,
        font_groups,
    );
    if !fits(fonts, min, &best_lines, max_width, max_height) {
        return (min, best_lines);
    }

    // Binary search for the largest fitting size (0.1 px resolution).
    let mut lo = min;
    let mut hi = max;
    while hi - lo > 0.1 {
        let mid = (lo + hi) * 0.5;
        let lines = shape_and_wrap(
            text,
            fonts,
            font_data,
            face_indices,
            mid,
            max_width,
            font_groups,
        );
        if fits(fonts, mid, &lines, max_width, max_height) {
            best_size = mid;
            best_lines = lines;
            lo = mid;
        } else {
            hi = mid;
        }
    }

    (best_size, best_lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ab_glyph::Font;

    fn load_test_fonts() -> Option<(Vec<FontArc>, Vec<Vec<u8>>, Vec<u32>, FontGroupRanges)> {
        super::super::TextRenderer::load_system_fonts()
    }

    /// Actual bottom of rendered text (from text origin y=0), computed from
    /// glyph outline bounds — this is what the renderer actually draws.
    fn actual_bottom(fonts: &[FontArc], size: f32, lines: &[LayoutLine]) -> f32 {
        let scaled = fonts[0].as_scaled(PxScale::from(size));
        let ascent = scaled.ascent();
        let line_h = size * LINE_HEIGHT_RATIO;
        let mut max_bottom = 0.0f32;
        for (li, line) in lines.iter().enumerate() {
            let baseline_y = ascent + li as f32 * line_h;
            for g in &line.glyphs {
                let font = &fonts[g.font_index as usize];
                let glyph = GlyphId(g.glyph_id as u16).with_scale(PxScale::from(size));
                if let Some(outlined) = font.outline_glyph(glyph) {
                    let bounds = outlined.px_bounds();
                    max_bottom = max_bottom.max(baseline_y + g.y_offset + bounds.max.y);
                }
            }
        }
        max_bottom
    }

    /// Actual right edge of rendered text (from text origin x=0), computed
    /// from glyph outline bounds.
    fn actual_right(fonts: &[FontArc], size: f32, lines: &[LayoutLine]) -> f32 {
        let mut max_right = 0.0f32;
        for line in lines {
            for g in &line.glyphs {
                let font = &fonts[g.font_index as usize];
                let glyph = GlyphId(g.glyph_id as u16).with_scale(PxScale::from(size));
                if let Some(outlined) = font.outline_glyph(glyph) {
                    let bounds = outlined.px_bounds();
                    max_right = max_right.max(g.x + g.x_offset + bounds.max.x);
                }
            }
        }
        max_right
    }

    #[test]
    fn total_height_vs_actual_glyph_bounds() {
        let Some((fonts, fd, fi, groups)) = load_test_fonts() else {
            eprintln!("No system fonts, skipping");
            return;
        };
        for (text, size, max_w) in [
            ("Hello", 20.0_f32, 1000.0_f32),
            ("pyggy", 20.0, 1000.0),
            ("The quick brown fox jumps over the lazy dog", 20.0, 100.0),
            ("The quick brown fox jumps over the lazy dog", 40.0, 200.0),
        ] {
            let lines = shape_and_wrap(text, &fonts, &fd, &fi, size, max_w, &groups);
            let th = total_height(&fonts, size, &lines);
            let ab = actual_bottom(&fonts, size, &lines);
            eprintln!(
                "text='{}' size={} lines={} total_h={:.2} actual={:.2} diff={:.2}",
                text,
                size,
                lines.len(),
                th,
                ab,
                ab - th
            );
            assert!(
                ab <= th + 0.5,
                "actual_bottom={:.2} > total_height={:.2} (diff={:.2})",
                ab,
                th,
                ab - th
            );
        }
    }

    #[test]
    fn fit_text_auto_fits_box() {
        let Some((fonts, fd, fi, groups)) = load_test_fonts() else {
            eprintln!("No system fonts, skipping");
            return;
        };
        let text = "The quick brown fox jumps over the lazy dog";
        for (mw, mh) in [
            (100.0_f32, 50.0_f32),
            (200.0, 100.0),
            (50.0, 200.0),
            (300.0, 30.0),
        ] {
            let (size, lines) = fit_text(text, &fonts, &fd, &fi, 0.0, mw, mh, &groups);
            let th = total_height(&fonts, size, &lines);
            let mlw = max_line_width(&fonts, size, &lines);
            let ab = actual_bottom(&fonts, size, &lines);
            let ar = actual_right(&fonts, size, &lines);
            eprintln!(
                "box={}x{}: size={:.1} lines={} th={:.2} mlw={:.2} actual_h={:.2} actual_w={:.2}",
                mw,
                mh,
                size,
                lines.len(),
                th,
                mlw,
                ab,
                ar
            );
            assert!(th <= mh + 0.01, "total_h={:.2} > max_h={:.2}", th, mh);
            assert!(mlw <= mw + 0.01, "max_line_w={:.2} > max_w={:.2}", mlw, mw);
            assert!(
                ab <= mh + 0.5,
                "actual_bottom={:.2} > max_h={:.2} (diff={:.2})",
                ab,
                mh,
                ab - mh
            );
            assert!(
                ar <= mw + 0.5,
                "actual_right={:.2} > max_w={:.2} (diff={:.2})",
                ar,
                mw,
                ar - mw
            );
        }
    }

    #[test]
    fn fit_text_narrow_column_equals() {
        let Some((fonts, fd, fi, groups)) = load_test_fonts() else {
            eprintln!("No system fonts, skipping");
            return;
        };
        let (size, lines) = fit_text("=", &fonts, &fd, &fi, 0.0, 30.0, 500.0, &groups);
        let mlw = max_line_width(&fonts, size, &lines);
        let ar = actual_right(&fonts, size, &lines);
        eprintln!(
            "= in 30px: size={:.1} mlw={:.2} actual_right={:.2}",
            size, mlw, ar
        );
        assert!(mlw <= 30.0 + 0.01, "max_line_w={:.2} > 30", mlw);
        assert!(ar <= 30.0 + 0.5, "actual_right={:.2} > 30", ar);
    }

    #[test]
    fn fit_text_explicit_size_fits() {
        let Some((fonts, fd, fi, groups)) = load_test_fonts() else {
            eprintln!("No system fonts, skipping");
            return;
        };
        let text = "The quick brown fox jumps over the lazy dog";
        let (size, lines) = fit_text(text, &fonts, &fd, &fi, 48.0, 200.0, 50.0, &groups);
        let th = total_height(&fonts, size, &lines);
        let ab = actual_bottom(&fonts, size, &lines);
        eprintln!(
            "explicit 48: size={:.1} lines={} th={:.2} actual_h={:.2}",
            size,
            lines.len(),
            th,
            ab
        );
        assert!(size <= 48.0 + 0.01, "size={:.2} > 48", size);
        assert!(th <= 50.0 + 0.01, "total_h={:.2} > 50", th);
        assert!(ab <= 50.0 + 0.5, "actual_bottom={:.2} > 50", ab);
    }

    #[test]
    fn grapheme_clustering_zwj_emoji() {
        let tokens = tokenize_by_graphemes("👨‍👩‍👧‍👦");
        assert_eq!(
            tokens.len(),
            1,
            "expected 1 token, got {}: {:?}",
            tokens.len(),
            tokens
        );
    }

    #[test]
    fn grapheme_clustering_word_and_space() {
        let tokens = tokenize_by_graphemes("Hello World");
        assert_eq!(
            tokens.len(),
            3,
            "expected [Hello, ' ', World], got {}: {:?}",
            tokens.len(),
            tokens
        );
        assert_eq!(tokens[0], "Hello");
        assert_eq!(tokens[1], " ");
        assert_eq!(tokens[2], "World");
    }
}
