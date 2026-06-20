// WARNING: AI GENERATED; UNDER REVIEW

use ab_glyph::{Font, FontArc, FontVec, GlyphId, PxScale, ScaleFont};
use bytemuck::{Pod, Zeroable, cast_slice};
use fontdb::{Family, Source};
use read_fonts::TableProvider;
use render_components::primitives::Text;
use render_layout::InternalLayoutable;
use rustybuzz::{Direction, Face, UnicodeBuffer, script};
use skrifa::color::ColorPainter;
use skrifa::{MetadataProvider, color};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::fs::read;
use std::ops::Range;
use std::slice;
use swash::FontRef;
use swash::scale::image::Content;
use unicode_bidi::{BidiInfo, Level};
use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, BlendState, BufferDescriptor,
    BufferUsages, ColorTargetState, ColorWrites, FragmentState, FrontFace, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPipelineDescriptor, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor, VertexAttribute, VertexBufferLayout,
    VertexState, VertexStepMode, include_wgsl, vertex_attr_array,
};

// ── Constants ──────────────────────────────────────────────────

const ATLAS_SIZE: u32 = 2048;
const MIN_FONT_SIZE: f32 = 6.0;
const LINE_HEIGHT_RATIO: f32 = 1.2;
const MAX_TEXT_VERTICES: usize = 16384; // 4096 glyphs × 4 verts

// ── Script & direction detection (unicode-script) ─────────────

fn is_rtl_script(s: Script) -> bool {
    matches!(
        s,
        Script::Arabic
            | Script::Hebrew
            | Script::Thaana
            | Script::Syriac
            | Script::Nko
            | Script::Mandaic
            | Script::Samaritan
    )
}

fn to_rustybuzz_script(s: Script) -> rustybuzz::Script {
    match s {
        Script::Latin => script::LATIN,
        Script::Arabic => script::ARABIC,
        Script::Hebrew => script::HEBREW,
        Script::Han => script::HAN,
        Script::Hiragana => script::HIRAGANA,
        Script::Katakana => script::KATAKANA,
        Script::Hangul => script::HANGUL,
        Script::Cyrillic => script::CYRILLIC,
        Script::Greek => script::GREEK,
        Script::Devanagari => script::DEVANAGARI,
        Script::Thai => script::THAI,
        Script::Georgian => script::GEORGIAN,
        Script::Armenian => script::ARMENIAN,
        _ => script::COMMON,
    }
}

fn detect_script_and_dir(text: &str) -> (rustybuzz::Script, Direction) {
    for c in text.chars() {
        let s = c.script();
        if s == Script::Common || s == Script::Inherited {
            continue;
        }
        let dir = if is_rtl_script(s) {
            Direction::RightToLeft
        } else {
            Direction::LeftToRight
        };
        return (to_rustybuzz_script(s), dir);
    }
    (script::LATIN, Direction::LeftToRight)
}

// ── Font group tracking ────────────────────────────────────────

/// Ranges of font indices for each priority group.
struct FontGroupRanges {
    latin: Range<usize>,
    cjk: Range<usize>,
    arabic: Range<usize>,
    emoji: Range<usize>,
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

fn is_emoji_token(token: &str) -> bool {
    token.chars().any(|c| {
        matches!(
            c as u32,
            0x1F600..=0x1F64F     // Emoticons
            | 0x1F300..=0x1F5FF   // Misc Symbols and Pictographs
            | 0x1F680..=0x1F6FF   // Transport and Map
            | 0x1F1E0..=0x1F1FF   // Flags
            | 0x1F900..=0x1F9FF   // Supplemental Symbols
            | 0x1FA00..=0x1FA6F   // Chess Symbols
            | 0x1FA70..=0x1FAFF   // Symbols Extended-A
            | 0x2600..=0x26FF     // Misc symbols
            | 0x2700..=0x27BF     // Dingbats
            | 0xFE0F              // Variation Selector 16
            | 0x200D              // ZWJ
        )
    })
}

/// Find the best single font for an entire word.
/// Prefers fonts from the group matching the script, then falls back to all fonts.
fn find_best_font_for_word(
    word: &str,
    fonts: &[FontArc],
    script: rustybuzz::Script,
    groups: &FontGroupRanges,
) -> usize {
    let covers = |font: &FontArc| -> bool {
        word.chars()
            .filter(|c| !c.is_whitespace())
            .all(|c| font.glyph_id(c).0 != 0)
    };

    // Determine preferred group based on script.
    let preferred: &Range<usize> = if is_emoji_token(word) {
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
            word.chars()
                .filter(|c| !c.is_whitespace())
                .filter(|c| font.glyph_id(*c).0 != 0)
                .count()
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}

// ── Text shaping with rustybuzz ────────────────────────────────

struct ShapedGlyph {
    glyph_id: u32,
    x_advance: f32,
    x_offset: f32,
    y_offset: f32,
}

/// Shape a text segment with rustybuzz using the given font.
fn shape_run(
    text: &str,
    font_data: &[u8],
    face_index: u32,
    font_size: f32,
    direction: Direction,
    script: rustybuzz::Script,
) -> Vec<ShapedGlyph> {
    let face = match Face::from_slice(font_data, face_index) {
        Some(f) => f,
        None => return Vec::new(),
    };

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(direction);
    buffer.set_script(script);

    let output = rustybuzz::shape(&face, &[], buffer);
    let infos = output.glyph_infos();
    let positions = output.glyph_positions();

    let units_per_em = face.units_per_em() as f32;
    let scale = font_size / units_per_em;

    eprintln!(
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
            eprintln!(
                "[SHAPE]   gid={} rb_advance={:.2} rb_x_offset={:.2} rb_y_offset={:.2}",
                info.glyph_id, rb_adv, rb_xoff, rb_yoff
            );
            ShapedGlyph {
                glyph_id: info.glyph_id,
                x_advance: rb_adv,
                x_offset: rb_xoff,
                y_offset: rb_yoff,
            }
        })
        .collect();
    result
}

// ── Word wrapping ──────────────────────────────────────────────

struct LayoutGlyph {
    glyph_id: u32,
    x: f32,
    x_offset: f32,
    y_offset: f32,
    font_index: u16,
}

struct LayoutLine {
    glyphs: Vec<LayoutGlyph>,
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
/// `max_width` pixels.  Every word goes through `rustybuzz::shape` as a whole
/// (no per-character font splitting) for proper kerning, ligatures, and
/// complex-script support.  Uses Unicode BiDi for proper RTL/LTR reordering.
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

        // Shape the whole word with one font (no per-character splitting).
        let (script, direction) = detect_script_and_dir(token);
        let font_idx = find_best_font_for_word(token, fonts, script, font_groups);
        let mut shaped = shape_run(
            token,
            &font_data[font_idx],
            face_indices[font_idx],
            font_size,
            direction,
            script,
        );

        // Replace rustybuzz advances with ab_glyph advances for consistent
        // cursor positioning.  The two libraries parse font metrics slightly
        // differently; using ab_glyph's advances (the same source used for
        // glyph rasterisation bearing/width) eliminates systematic gaps.
        {
            let font = &fonts[font_idx];
            let scaled = font.as_scaled(PxScale::from(font_size));
            for g in &mut shaped {
                let ab_gid = GlyphId(g.glyph_id as u16);
                g.x_advance = scaled.h_advance(ab_gid);
            }
        }

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
                        font_index: font_idx as u16,
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

/// Compute the text layout that fits into `max_width × max_height`.
/// Returns `(actual_font_size, lines)`.
fn fit_text(
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

    let mut size = requested_size;
    loop {
        let lines = shape_and_wrap(
            text,
            fonts,
            font_data,
            face_indices,
            size,
            max_width,
            font_groups,
        );
        let scaled = fonts[0].as_scaled(PxScale::from(size));
        let ascent = scaled.ascent();
        let descent = scaled.descent();
        let line_h = size * LINE_HEIGHT_RATIO;
        let n = lines.len() as f32;
        let total_h = ascent + descent + (n - 1.0).max(0.0) * line_h;

        if total_h <= max_height || size <= MIN_FONT_SIZE {
            return (size.max(MIN_FONT_SIZE), lines);
        }
        size = (size * 0.9).max(MIN_FONT_SIZE);
    }
}

// ── Glyph atlas ───────────────────────────────────────────────

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct GlyphKey {
    glyph_id: u32,
    /// Font size multiplied by 10 and truncated — gives 0.1 px resolution.
    size_tenths: u32,
    /// Index into the font fallback list.
    font_index: u32,
}

struct GlyphEntry {
    width: u32,
    height: u32,
    bearing_x: f32,
    bearing_y: f32,
    advance: f32,
    uv_min: [f32; 2],
    uv_max: [f32; 2],
}

struct GlyphAtlas {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    format: TextureFormat,
    cursor_x: u32,
    cursor_y: u32,
    shelf_height: u32,
    glyphs: HashMap<GlyphKey, GlyphEntry>,
}

impl GlyphAtlas {
    fn new(device: &wgpu::Device, format: TextureFormat) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Text Glyph Atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        Self {
            texture,
            texture_view,
            format,
            cursor_x: 0,
            cursor_y: 0,
            shelf_height: 0,
            glyphs: HashMap::new(),
        }
    }

    fn clear(&mut self, queue: &wgpu::Queue) {
        let bytes_per_pixel = match self.format {
            TextureFormat::R8Unorm => 1,
            TextureFormat::Rgba8Unorm => 4,
            _ => 1,
        };
        let data = vec![0u8; (ATLAS_SIZE * ATLAS_SIZE * bytes_per_pixel) as usize];
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ATLAS_SIZE * bytes_per_pixel),
                rows_per_image: Some(ATLAS_SIZE),
            },
            wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
        );
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.shelf_height = 0;
        self.glyphs.clear();
    }

    fn allocate(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        if w == 0 || h == 0 {
            return Some((0, 0));
        }
        if self.cursor_x + w > ATLAS_SIZE {
            self.cursor_y += self.shelf_height;
            self.cursor_x = 0;
            self.shelf_height = 0;
        }
        if self.cursor_y + h > ATLAS_SIZE {
            return None;
        }
        let pos = (self.cursor_x, self.cursor_y);
        self.cursor_x += w;
        self.shelf_height = self.shelf_height.max(h);
        Some(pos)
    }
}

// ── GPU vertex / uniform types ─────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TextUniform {
    screen_size: [f32; 2],
    _pad: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct TextVertex {
    position: [f32; 2],
    uv: [f32; 2],
    /// 0 = text atlas (R8, uniform colour), 1 = color atlas (RGBA, direct).
    tex_index: u32,
}

impl TextVertex {
    const ATTRIBS: [VertexAttribute; 3] = vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Uint32,
    ];

    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// ── COLRv1 colour-emoji rasterisation ──────────────────────────

struct ColrV1Painter {
    palette: Vec<[u8; 4]>,
    font: FontArc,
    font_size: f32,
    units_per_em: f32,
    current_glyph_id: Option<u16>,
    clip_box: Option<[f32; 4]>, // [x_min, y_min, x_max, y_max] in pixels
    /// Stack of saved clip states: (glyph_id, clip_box) pairs.
    clip_stack: Vec<(Option<u16>, Option<[f32; 4]>)>,
    output: Vec<u8>,
    output_w: u32,
    output_h: u32,
    offset_x: f32,
    offset_y: f32,
    layer_stack: Vec<Vec<u8>>,
}

impl ColrV1Painter {
    fn new(palette: Vec<[u8; 4]>, font: FontArc, font_size: f32) -> Self {
        let buf = (font_size * 3.0).ceil() as u32;
        let off = font_size * 1.5;
        let upem = font.units_per_em().unwrap_or(1000.0);
        Self {
            palette,
            font,
            font_size,
            units_per_em: upem,
            current_glyph_id: None,
            clip_box: None,
            clip_stack: Vec::new(),
            output: vec![0u8; (buf * buf * 4) as usize],
            output_w: buf,
            output_h: buf,
            offset_x: off,
            offset_y: off,
            layer_stack: Vec::new(),
        }
    }

    fn into_rgba(self) -> Option<(Vec<u8>, u32, u32, f32, f32)> {
        let (mut min_x, mut min_y) = (self.output_w, self.output_h);
        let (mut max_x, mut max_y) = (0u32, 0u32);
        for y in 0..self.output_h {
            for x in 0..self.output_w {
                let a = self.output[((y * self.output_w + x) * 4 + 3) as usize];
                if a > 0 {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x + 1);
                    max_y = max_y.max(y + 1);
                }
            }
        }
        if min_x >= max_x || min_y >= max_y {
            return None;
        }
        let cw = max_x - min_x;
        let ch = max_y - min_y;
        let mut cropped = vec![0u8; (cw * ch * 4) as usize];
        for y in 0..ch {
            let src_row = ((min_y + y) * self.output_w + min_x) as usize * 4;
            let dst_row = (y * cw) as usize * 4;
            cropped[dst_row..dst_row + cw as usize * 4]
                .copy_from_slice(&self.output[src_row..src_row + cw as usize * 4]);
        }
        let bx = min_x as f32 - self.offset_x;
        let by = min_y as f32 - self.offset_y;
        Some((cropped, cw, ch, bx, by))
    }

    fn current_buffer(&mut self) -> &mut Vec<u8> {
        match self.layer_stack.last_mut() {
            Some(buf) => buf,
            None => &mut self.output,
        }
    }

    fn paint_coverage(
        coverage: &[f32],
        gw: u32,
        gh: u32,
        bounds_min_x: f32,
        bounds_min_y: f32,
        offset_x: f32,
        offset_y: f32,
        output_w: u32,
        output_h: u32,
        colour: &[u8; 4],
        buf: &mut [u8],
    ) {
        for py in 0..gh {
            for px in 0..gw {
                let cov = coverage[(py * gw + px) as usize];
                if cov <= 0.0 {
                    continue;
                }
                let ox = (offset_x + bounds_min_x + px as f32) as i32;
                let oy = (offset_y + bounds_min_y + py as f32) as i32;
                if ox < 0 || oy < 0 || ox as u32 >= output_w || oy as u32 >= output_h {
                    continue;
                }
                let ti = ((oy as u32 * output_w + ox as u32) * 4) as usize;
                let sa = cov * (colour[3] as f32 / 255.0);
                let da = buf[ti + 3] as f32 / 255.0;
                let oa = sa + da * (1.0 - sa);
                if oa > 0.0 {
                    for c in 0..3 {
                        let s = colour[c] as f32 / 255.0 * sa;
                        let d = buf[ti + c] as f32 / 255.0 * da * (1.0 - sa);
                        buf[ti + c] = (((s + d) / oa) * 255.0) as u8;
                    }
                    buf[ti + 3] = (oa * 255.0) as u8;
                }
            }
        }
    }
}

impl ColorPainter for ColrV1Painter {
    fn push_transform(&mut self, _transform: color::Transform) {
        eprintln!("[COLRv1] push_transform");
    }
    fn pop_transform(&mut self) {
        eprintln!("[COLRv1] pop_transform");
    }

    fn push_layer(&mut self, _mode: color::CompositeMode) {
        eprintln!("[COLRv1] push_layer");
        self.layer_stack
            .push(vec![0u8; (self.output_w * self.output_h * 4) as usize]);
    }

    fn pop_layer(&mut self) {
        eprintln!("[COLRv1] pop_layer");
        if let Some(layer) = self.layer_stack.pop() {
            let dst = self.current_buffer();
            for i in (0..layer.len()).step_by(4) {
                let sa = layer[i + 3] as f32 / 255.0;
                let da = dst[i + 3] as f32 / 255.0;
                let oa = sa + da * (1.0 - sa);
                if oa > 0.0 {
                    for c in 0..3 {
                        let s = layer[i + c] as f32 / 255.0 * sa;
                        let d = dst[i + c] as f32 / 255.0 * da * (1.0 - sa);
                        dst[i + c] = (((s + d) / oa) * 255.0) as u8;
                    }
                    dst[i + 3] = (oa * 255.0) as u8;
                }
            }
        }
    }

    fn fill(&mut self, brush: color::Brush<'_>) {
        let palette_index = match brush {
            color::Brush::Solid { palette_index, .. } => palette_index,
            _ => {
                eprintln!("[COLRv1] fill: unsupported brush type, skipping");
                return;
            }
        };
        let colour = self
            .palette
            .get(palette_index as usize)
            .copied()
            .unwrap_or([255, 0, 255, 255]);

        eprintln!(
            "[COLRv1] fill: palette_index={} colour={:?} glyph_id={:?} clip_box={:?}",
            palette_index, colour, self.current_glyph_id, self.clip_box
        );

        // ── Path A: fill via glyph outline ──────────────────────
        if let Some(gid) = self.current_glyph_id {
            let ab_gid = GlyphId(gid);
            let scaled = ab_gid.with_scale(PxScale::from(self.font_size));
            if let Some(outlined) = self.font.outline_glyph(scaled) {
                let bounds = outlined.px_bounds();
                let gw = (bounds.max.x - bounds.min.x).ceil() as u32;
                let gh = (bounds.max.y - bounds.min.y).ceil() as u32;
                if gw > 0 && gh > 0 {
                    let mut coverage = vec![0.0f32; (gw * gh) as usize];
                    outlined.draw(|x, y, c| {
                        let idx = (y * gw + x) as usize;
                        if idx < coverage.len() {
                            coverage[idx] = c;
                        }
                    });

                    let offset_x = self.offset_x;
                    let offset_y = self.offset_y;
                    let output_w = self.output_w;
                    let output_h = self.output_h;
                    let buf = self.current_buffer();
                    Self::paint_coverage(
                        &coverage,
                        gw,
                        gh,
                        bounds.min.x,
                        bounds.min.y,
                        offset_x,
                        offset_y,
                        output_w,
                        output_h,
                        &colour,
                        buf,
                    );
                    eprintln!("[COLRv1] fill Path A: succeeded for gid={}", gid);
                    return;
                }
            }
            eprintln!(
                "[COLRv1] fill Path A: outline_glyph({}) returned None, falling back",
                gid
            );
        }

        // ── Path B: fill via clip box (rectangular region) ──────
        // The clip box coordinates come from skrifa in font units (Y-up),
        // but the output buffer uses screen coordinates (Y-down).
        // We must flip the Y axis when mapping to buffer coordinates.
        //
        // First try the current clip_box, then search the clip_stack for a
        // fallback (needed when push_clip_glyph cleared the outer clip_box
        // but the glyph outline couldn't be rendered via ab_glyph).
        let cb = self
            .clip_box
            .or_else(|| self.clip_stack.iter().rev().find_map(|(_, cb)| *cb));
        if let Some(cb) = cb {
            let offset_x = self.offset_x;
            let offset_y = self.offset_y;
            let output_w = self.output_w;
            let output_h = self.output_h;
            let buf = self.current_buffer();
            let x0 = (cb[0] + offset_x).ceil().max(0.0) as i32;
            // Y flip: font Y-up (positive=above) → buffer Y-down (positive=below)
            // cb[3] is y_max (highest point) → smallest buffer Y
            // cb[1] is y_min (lowest point) → largest buffer Y
            let y0 = (offset_y - cb[3]).ceil().max(0.0) as i32;
            let x1 = (cb[2] + offset_x).floor().min(output_w as f32) as i32;
            let y1 = (offset_y - cb[1]).floor().min(output_h as f32) as i32;

            eprintln!(
                "[COLRv1 fill Path B] clip_box=[{:.1},{:.1},{:.1},{:.1}] offset=({:.1},{:.1}) buffer rect=[{},{} → {},{}] colour={:?}",
                cb[0], cb[1], cb[2], cb[3], offset_x, offset_y, x0, y0, x1, y1, colour
            );
            let sa = colour[3] as f32 / 255.0;
            for oy in y0..y1 {
                for ox in x0..x1 {
                    let ti = ((oy as u32 * output_w + ox as u32) * 4) as usize;
                    let da = buf[ti + 3] as f32 / 255.0;
                    let oa = sa + da * (1.0 - sa);
                    if oa > 0.0 {
                        for c in 0..3 {
                            let s = colour[c] as f32 / 255.0 * sa;
                            let d = buf[ti + c] as f32 / 255.0 * da * (1.0 - sa);
                            buf[ti + c] = (((s + d) / oa) * 255.0) as u8;
                        }
                        buf[ti + 3] = (oa * 255.0) as u8;
                    }
                }
            }
        }
    }

    fn push_clip_glyph(&mut self, glyph_id: skrifa::GlyphId) {
        eprintln!("[COLRv1] push_clip_glyph: gid={}", glyph_id.to_u32());
        self.clip_stack.push((self.current_glyph_id, self.clip_box));
        self.current_glyph_id = Some(glyph_id.to_u32() as u16);
        self.clip_box = None;
    }

    fn push_clip_box(&mut self, bbox: read_fonts::types::BoundingBox<f32>) {
        let scale = self.font_size / self.units_per_em;
        eprintln!(
            "[COLRv1] push_clip_box: bbox=[{:.1},{:.1},{:.1},{:.1}] scale={:.6} pixel=[{:.1},{:.1},{:.1},{:.1}]",
            bbox.x_min,
            bbox.y_min,
            bbox.x_max,
            bbox.y_max,
            scale,
            bbox.x_min * scale,
            bbox.y_min * scale,
            bbox.x_max * scale,
            bbox.y_max * scale
        );
        self.clip_stack.push((self.current_glyph_id, self.clip_box));
        self.current_glyph_id = None;
        self.clip_box = Some([
            bbox.x_min * scale,
            bbox.y_min * scale,
            bbox.x_max * scale,
            bbox.y_max * scale,
        ]);
    }

    fn pop_clip(&mut self) {
        eprintln!("[COLRv1] pop_clip");
        if let Some((gid, cb)) = self.clip_stack.pop() {
            self.current_glyph_id = gid;
            self.clip_box = cb;
        } else {
            self.current_glyph_id = None;
            self.clip_box = None;
        }
    }
}

fn read_cpal(font: &skrifa::FontRef) -> Vec<[u8; 4]> {
    let Ok(cpal) = font.cpal() else {
        return Vec::new();
    };
    let Some(Ok(records)) = cpal.color_records_array() else {
        return Vec::new();
    };
    let n = cpal.num_palette_entries() as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        if let Some(rec) = records.get(i) {
            out.push([rec.red, rec.green, rec.blue, rec.alpha]);
        } else {
            out.push([0, 0, 0, 255]);
        }
    }
    out
}

// ── TextRenderer ───────────────────────────────────────────────

pub struct TextRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    text_atlas: RefCell<GlyphAtlas>,
    color_atlas: RefCell<GlyphAtlas>,
    fonts: Vec<FontArc>,
    font_data: Vec<Vec<u8>>,
    /// TTC face index for each font.
    face_indices: Vec<u32>,
    font_groups: FontGroupRanges,
    next_vertex: Cell<u32>,
    next_index: Cell<u32>,
}

impl TextRenderer {
    pub fn new(device: &wgpu::Device, texture_format: TextureFormat) -> Option<Self> {
        let (fonts, font_data, face_indices, font_groups) = Self::load_system_fonts()?;

        let shader = device.create_shader_module(include_wgsl!("shaders/text.wgsl"));

        // ── Buffers ────────────────────────────────────────────────
        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Text Uniform Buffer"),
            size: size_of::<TextUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Text Vertex Buffer"),
            size: (MAX_TEXT_VERTICES * size_of::<TextVertex>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let max_indices = (MAX_TEXT_VERTICES / 4) * 6;
        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Text Index Buffer"),
            size: (max_indices * size_of::<u16>()) as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ── Bind group ─────────────────────────────────────────────
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Text Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Text Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let text_atlas = RefCell::new(GlyphAtlas::new(device, TextureFormat::R8Unorm));
        let color_atlas = RefCell::new(GlyphAtlas::new(device, TextureFormat::Rgba8Unorm));

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Text Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&text_atlas.borrow().texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&color_atlas.borrow().texture_view),
                },
            ],
        });

        // ── Pipeline ───────────────────────────────────────────────
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Text Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[TextVertex::layout()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: texture_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Some(Self {
            pipeline,
            uniform_buffer,
            vertex_buffer,
            index_buffer,
            bind_group,
            text_atlas,
            color_atlas,
            fonts,
            font_data,
            face_indices,
            font_groups,
            next_vertex: Cell::new(0),
            next_index: Cell::new(0),
        })
    }

    // ── Font discovery ─────────────────────────────────────────────

    fn load_system_fonts() -> Option<(Vec<FontArc>, Vec<Vec<u8>>, Vec<u32>, FontGroupRanges)> {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();

        let font_families: &[&[&str]] = &[
            &[
                "DejaVu Sans",
                "Liberation Sans",
                "Noto Sans",
                "Ubuntu",
                "Cantarell",
                "SansSerif",
            ],
            &[
                "Noto Sans CJK SC",
                "Noto Sans CJK JP",
                "Noto Sans CJK TC",
                "Noto Sans CJK KR",
                "Noto Sans CJK HK",
                "Noto Sans SC",
                "Noto Sans JP",
                "Noto Sans TC",
                "Noto Sans KR",
                "Source Han Sans SC",
                "Source Han Sans",
                "Source Han Sans KR",
                "WenQuanYi Micro Hei",
                "WenQuanYi Zen Hei",
                "AR PL UMing CN",
                "Droid Sans Fallback",
                "NanumGothic",
                "NanumBarunGothic",
                "Baekmuk Gulim",
                "Baekmuk Dotum",
                "UnBatang",
                "UnDotum",
            ],
            &["Noto Sans Arabic", "DejaVu Sans", "Scheherazade New"],
            &[
                "Noto Color Emoji",
                "Twitter Color Emoji",
                "Twemoji",
                "JoyPixels",
                "EmojiOne",
                "OpenMoji",
                "Noto Emoji",
                "Symbola",
                "Noto Sans Symbols2",
                "Noto Sans Symbols",
                "Segoe UI Symbol",
            ],
        ];

        let mut fonts = Vec::new();
        let mut font_data = Vec::new();
        let mut face_indices = Vec::new();
        let mut loaded_ids: HashSet<fontdb::ID> = HashSet::new();
        let mut group_boundaries: Vec<usize> = Vec::new();

        // Phase 1: load prioritised families.
        for families in font_families {
            for family in *families {
                let query = fontdb::Query {
                    families: &[Family::Name(family)],
                    ..Default::default()
                };

                if let Some(id) = db.query(&query) {
                    if !loaded_ids.insert(id) {
                        continue;
                    }
                    if let Some((source, index)) = db.face_source(id) {
                        let data: Option<Vec<u8>> = match source {
                            Source::File(path) => read(path).ok(),
                            Source::Binary(arc) => Some(arc.as_ref().as_ref().to_vec()),
                            Source::SharedFile(_path, arc) => Some(arc.as_ref().as_ref().to_vec()),
                        };

                        if let Some(data) = data {
                            let data_for_buzz = data.clone();
                            match FontVec::try_from_vec_and_index(data, index) {
                                Ok(font_vec) => {
                                    fonts.push(font_vec.into());
                                    font_data.push(data_for_buzz);
                                    face_indices.push(index);
                                }
                                Err(_) => {
                                    loaded_ids.remove(&id);
                                    continue;
                                }
                            }
                        } else {
                            loaded_ids.remove(&id);
                        }
                    }
                }
            }
            group_boundaries.push(fonts.len());
        }

        // Phase 2: load every other system font for full coverage.
        for face_info in db.faces() {
            let id = face_info.id;
            if !loaded_ids.insert(id) {
                continue;
            }
            if let Some((source, index)) = db.face_source(id) {
                let data: Option<Vec<u8>> = match source {
                    Source::File(path) => read(path).ok(),
                    Source::Binary(arc) => Some(arc.as_ref().as_ref().to_vec()),
                    Source::SharedFile(_path, arc) => Some(arc.as_ref().as_ref().to_vec()),
                };

                if let Some(data) = data {
                    let data_for_buzz = data.clone();
                    match FontVec::try_from_vec_and_index(data, index) {
                        Ok(font_vec) => {
                            fonts.push(font_vec.into());
                            font_data.push(data_for_buzz);
                            face_indices.push(index);
                        }
                        Err(_) => continue,
                    }
                }
            }
        }

        if fonts.is_empty() {
            return None;
        }

        #[allow(clippy::get_first)]
        let font_groups = FontGroupRanges {
            latin: 0..group_boundaries.get(0).copied().unwrap_or(0),
            cjk: group_boundaries.get(0).copied().unwrap_or(0)
                ..group_boundaries.get(1).copied().unwrap_or(0),
            arabic: group_boundaries.get(1).copied().unwrap_or(0)
                ..group_boundaries.get(2).copied().unwrap_or(0),
            emoji: group_boundaries.get(2).copied().unwrap_or(0)
                ..group_boundaries.get(3).copied().unwrap_or(0),
        };

        Some((fonts, font_data, face_indices, font_groups))
    }

    /// Reset the per-frame write cursors.
    pub fn begin_frame(&self) {
        self.next_vertex.set(0);
        self.next_index.set(0);
    }

    /// Render a single text primitive.
    pub fn render_one<'a>(
        &'a self,
        text: &Text,
        screen_size: (u32, u32),
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        if text.text.is_empty() || text.width == 0 || text.height == 0 {
            return;
        }

        let max_w = text.width as f32;
        let max_h = text.height as f32;

        let (actual_size, lines) = fit_text(
            &text.text,
            &self.fonts,
            &self.font_data,
            &self.face_indices,
            text.font_size,
            max_w,
            max_h,
            &self.font_groups,
        );

        let scaled = self.fonts[0].as_scaled(PxScale::from(actual_size));
        let ascent = scaled.ascent();
        let line_h = actual_size * LINE_HEIGHT_RATIO;

        // ── Build quads ────────────────────────────────────────────
        let mut vertices: Vec<TextVertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let mut text_atlas = self.text_atlas.borrow_mut();
        let mut color_atlas = self.color_atlas.borrow_mut();

        for (li, line) in lines.iter().enumerate() {
            let baseline_y = text.get_y() as f32 + ascent + li as f32 * line_h;
            let mut prev_right_edge: Option<f32> = None;

            for (gi, glyph) in line.glyphs.iter().enumerate() {
                if vertices.len() + 4 > MAX_TEXT_VERTICES {
                    break;
                }

                let gid = GlyphId(glyph.glyph_id as u16);
                let key = GlyphKey {
                    glyph_id: glyph.glyph_id,
                    size_tenths: (actual_size * 10.0) as u32,
                    font_index: glyph.font_index as u32,
                };

                // Rasterise on cache miss.
                let entry_opt = if text_atlas.glyphs.contains_key(&key) {
                    Some((false, text_atlas.glyphs.get(&key).unwrap()))
                } else if color_atlas.glyphs.contains_key(&key) {
                    Some((true, color_atlas.glyphs.get(&key).unwrap()))
                } else {
                    None
                };

                if entry_opt.is_none() {
                    self.rasterise_glyph(
                        &mut text_atlas,
                        &mut color_atlas,
                        queue,
                        gid,
                        actual_size,
                        &key,
                        glyph.font_index as usize,
                    );
                }

                let (is_color, entry) = if let Some(e) = text_atlas.glyphs.get(&key) {
                    (false, e)
                } else if let Some(e) = color_atlas.glyphs.get(&key) {
                    (true, e)
                } else {
                    continue;
                };

                let gx = text.get_x() as f32 + glyph.x + glyph.x_offset + entry.bearing_x;
                let gy = baseline_y + glyph.y_offset + entry.bearing_y;
                let gw = entry.width as f32;
                let gh = entry.height as f32;
                let ti = if is_color { 1u32 } else { 0u32 };

                // Debug: log glyph positioning and gaps
                let right_edge = gx + gw;
                if let Some(prev_re) = prev_right_edge {
                    let gap = gx - prev_re;
                    if gap.abs() > 0.5 {
                        eprintln!(
                            "[GAP] line {} glyph {}: gap={:.2}px | prev_right={:.1} curr_left={:.1} | glyph.x={:.1} x_offset={:.1} bearing_x={:.1} width={:.1} advance={:.1} gid={}",
                            li,
                            gi,
                            gap,
                            prev_re,
                            gx,
                            glyph.x,
                            glyph.x_offset,
                            entry.bearing_x,
                            gw,
                            entry.advance,
                            glyph.glyph_id
                        );
                    }
                }
                eprintln!(
                    "[GLYPH] line {} gi={} gid={} x={:.1} x_off={:.1} bx={:.1} w={:.1} adv={:.1} right={:.1}",
                    li,
                    gi,
                    glyph.glyph_id,
                    gx,
                    glyph.x_offset,
                    entry.bearing_x,
                    gw,
                    entry.advance,
                    right_edge
                );
                prev_right_edge = Some(right_edge);

                let base = vertices.len() as u16;
                vertices.push(TextVertex {
                    position: [gx, gy],
                    uv: entry.uv_min,
                    tex_index: ti,
                });
                vertices.push(TextVertex {
                    position: [gx + gw, gy],
                    uv: [entry.uv_max[0], entry.uv_min[1]],
                    tex_index: ti,
                });
                vertices.push(TextVertex {
                    position: [gx + gw, gy + gh],
                    uv: entry.uv_max,
                    tex_index: ti,
                });
                vertices.push(TextVertex {
                    position: [gx, gy + gh],
                    uv: [entry.uv_min[0], entry.uv_max[1]],
                    tex_index: ti,
                });
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }

        if vertices.is_empty() {
            return;
        }

        let vert_offset = self.next_vertex.get();
        let idx_offset = self.next_index.get();
        let vert_byte = vert_offset as u64 * size_of::<TextVertex>() as u64;
        let idx_byte = idx_offset as u64 * size_of::<u16>() as u64;

        let vert_end = vert_offset as usize + vertices.len();
        let idx_end = idx_offset as usize + indices.len();
        let max_verts = MAX_TEXT_VERTICES;
        let max_idxs = (MAX_TEXT_VERTICES / 4) * 6;
        if vert_end > max_verts || idx_end > max_idxs {
            return;
        }

        // ── Upload & draw ──────────────────────────────────────────
        queue.write_buffer(&self.vertex_buffer, vert_byte, cast_slice(&vertices));
        queue.write_buffer(&self.index_buffer, idx_byte, cast_slice(&indices));

        let uniforms = TextUniform {
            screen_size: [screen_size.0 as f32, screen_size.1 as f32],
            _pad: [0.0; 2],
            color: [
                text.color.0 as f32 / 255.0,
                text.color.1 as f32 / 255.0,
                text.color.2 as f32 / 255.0,
                text.color.3 as f32 / 255.0,
            ],
        };
        queue.write_buffer(&self.uniform_buffer, 0, cast_slice(&[uniforms]));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(vert_byte..));
        render_pass.set_index_buffer(
            self.index_buffer.slice(idx_byte..),
            wgpu::IndexFormat::Uint16,
        );

        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);

        self.next_vertex.set(vert_offset + vertices.len() as u32);
        self.next_index.set(idx_offset + indices.len() as u32);
    }

    // ── Glyph rasterisation ────────────────────────────────────────

    /// Rasterise a single glyph into the appropriate atlas (text or colour).
    /// Tries swash (COLR / CBDT / SVG / sbix), then skrifa COLRv1, then falls
    /// back to ab_glyph outline rasterisation.
    fn rasterise_glyph(
        &self,
        text_atlas: &mut GlyphAtlas,
        color_atlas: &mut GlyphAtlas,
        queue: &wgpu::Queue,
        gid: GlyphId,
        font_size: f32,
        key: &GlyphKey,
        font_index: usize,
    ) {
        // ── Try colour glyph via swash ─────────────────────────
        if let Some(font_ref) = FontRef::from_index(
            &self.font_data[font_index],
            self.face_indices[font_index] as usize,
        ) {
            let mut swash_ctx = swash::scale::ScaleContext::new();
            let mut swash_scaler = swash_ctx.builder(font_ref).size(font_size).build();
            let swash_image = swash::scale::Render::new(&[
                swash::scale::Source::ColorOutline(0),
                swash::scale::Source::ColorBitmap(swash::scale::StrikeWith::BestFit),
            ])
            .render(&mut swash_scaler, gid.0);

            if let Some(img) = swash_image
                && img.content == Content::Color
            {
                let mut rgba = Vec::with_capacity(img.data.len());
                for pixel in img.data.chunks_exact(4) {
                    rgba.push(pixel[2]); // BGRA → RGBA
                    rgba.push(pixel[1]);
                    rgba.push(pixel[0]);
                    rgba.push(pixel[3]);
                }

                let w = img.placement.width;
                let h = img.placement.height;
                if w > 0 && h > 0 && w < ATLAS_SIZE && h < ATLAS_SIZE {
                    let pos = match color_atlas.allocate(w, h) {
                        Some(p) => p,
                        None => {
                            color_atlas.clear(queue);
                            match color_atlas.allocate(w, h) {
                                Some(p) => p,
                                None => return,
                            }
                        }
                    };

                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &color_atlas.texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: pos.0,
                                y: pos.1,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &rgba,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(w * 4),
                            rows_per_image: Some(h),
                        },
                        wgpu::Extent3d {
                            width: w,
                            height: h,
                            depth_or_array_layers: 1,
                        },
                    );

                    let font = &self.fonts[font_index];
                    let scaled = font.as_scaled(PxScale::from(font_size));
                    let advance = scaled.h_advance(gid);

                    color_atlas.glyphs.insert(
                        key.clone(),
                        GlyphEntry {
                            width: w,
                            height: h,
                            bearing_x: img.placement.left as f32,
                            bearing_y: -(img.placement.top as f32),
                            advance,
                            uv_min: [
                                pos.0 as f32 / ATLAS_SIZE as f32,
                                pos.1 as f32 / ATLAS_SIZE as f32,
                            ],
                            uv_max: [
                                (pos.0 + w) as f32 / ATLAS_SIZE as f32,
                                (pos.1 + h) as f32 / ATLAS_SIZE as f32,
                            ],
                        },
                    );
                    return;
                }
            }
        }

        // ── Try COLRv1 via skrifa ──────────────────────────────
        {
            let fd = &self.font_data[font_index];
            let fi = self.face_indices[font_index];
            if let Ok(font_ref) = skrifa::FontRef::from_index(fd, fi) {
                let cg = font_ref.color_glyphs();
                let sgid = skrifa::raw::types::GlyphId::new(gid.0 as u32);
                if let Some(color_glyph) = cg.get(sgid) {
                    eprintln!(
                        "[COLRv1] glyph {} found in COLRv1 table, attempting paint...",
                        gid.0
                    );
                    let palette = read_cpal(&font_ref);
                    let mut painter =
                        ColrV1Painter::new(palette, self.fonts[font_index].clone(), font_size);
                    let paint_result =
                        color_glyph.paint(skrifa::instance::LocationRef::default(), &mut painter);
                    eprintln!(
                        "[COLRv1] paint result: {:?}, output {}x{}",
                        paint_result, painter.output_w, painter.output_h
                    );
                    if paint_result.is_ok()
                        && let Some((rgba, w, h, bx, by)) = painter.into_rgba()
                        && w > 0
                        && h > 0
                        && w < ATLAS_SIZE
                        && h < ATLAS_SIZE
                    {
                        let pos = match color_atlas.allocate(w, h) {
                            Some(p) => p,
                            None => {
                                color_atlas.clear(queue);
                                match color_atlas.allocate(w, h) {
                                    Some(p) => p,
                                    None => return,
                                }
                            }
                        };

                        queue.write_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &color_atlas.texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d {
                                    x: pos.0,
                                    y: pos.1,
                                    z: 0,
                                },
                                aspect: wgpu::TextureAspect::All,
                            },
                            &rgba,
                            wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(w * 4),
                                rows_per_image: Some(h),
                            },
                            wgpu::Extent3d {
                                width: w,
                                height: h,
                                depth_or_array_layers: 1,
                            },
                        );

                        let font = &self.fonts[font_index];
                        let scaled = font.as_scaled(PxScale::from(font_size));
                        let advance = scaled.h_advance(gid);

                        color_atlas.glyphs.insert(
                            key.clone(),
                            GlyphEntry {
                                width: w,
                                height: h,
                                bearing_x: bx,
                                bearing_y: by,
                                advance,
                                uv_min: [
                                    pos.0 as f32 / ATLAS_SIZE as f32,
                                    pos.1 as f32 / ATLAS_SIZE as f32,
                                ],
                                uv_max: [
                                    (pos.0 + w) as f32 / ATLAS_SIZE as f32,
                                    (pos.1 + h) as f32 / ATLAS_SIZE as f32,
                                ],
                            },
                        );
                        return;
                    }
                }
            }
        }

        // ── Fallback: outline rasterisation (R8 text atlas) ────
        let font = &self.fonts[font_index];
        let scaled_glyph = gid.with_scale(PxScale::from(font_size));

        let outlined = match font.outline_glyph(scaled_glyph) {
            Some(o) => o,
            None => return,
        };

        let bounds = outlined.px_bounds();
        let bw = (bounds.max.x - bounds.min.x).ceil() as u32;
        let bh = (bounds.max.y - bounds.min.y).ceil() as u32;

        if bw == 0 || bh == 0 {
            return;
        }

        let mut coverage = vec![0.0f32; (bw * bh) as usize];
        outlined.draw(|x, y, c| {
            let idx = (y * bw + x) as usize;
            if idx < coverage.len() {
                coverage[idx] = c;
            }
        });

        let bitmap: Vec<u8> = coverage.iter().map(|&c| (c * 255.0) as u8).collect();

        let pos = match text_atlas.allocate(bw, bh) {
            Some(p) => p,
            None => {
                text_atlas.clear(queue);
                match text_atlas.allocate(bw, bh) {
                    Some(p) => p,
                    None => return,
                }
            }
        };

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &text_atlas.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: pos.0,
                    y: pos.1,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &bitmap,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bw),
                rows_per_image: Some(bh),
            },
            wgpu::Extent3d {
                width: bw,
                height: bh,
                depth_or_array_layers: 1,
            },
        );

        let scaled = font.as_scaled(PxScale::from(font_size));

        text_atlas.glyphs.insert(
            key.clone(),
            GlyphEntry {
                width: bw,
                height: bh,
                bearing_x: bounds.min.x,
                bearing_y: bounds.min.y,
                advance: scaled.h_advance(gid),
                uv_min: [
                    pos.0 as f32 / ATLAS_SIZE as f32,
                    pos.1 as f32 / ATLAS_SIZE as f32,
                ],
                uv_max: [
                    (pos.0 + bw) as f32 / ATLAS_SIZE as f32,
                    (pos.1 + bh) as f32 / ATLAS_SIZE as f32,
                ],
            },
        );
    }
}
