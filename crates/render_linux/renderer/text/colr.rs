// WARNING: AI GENERATED; UNDER REVIEW

use ab_glyph::{Font, FontArc, GlyphId, PxScale};
use skrifa::color::{self, ColorPainter};
use skrifa::raw::TableProvider;

use crate::debug;

pub(super) struct ColrV1Painter {
    palette: Vec<[u8; 4]>,
    font: FontArc,
    font_size: f32,
    units_per_em: f32,
    current_glyph_id: Option<u16>,
    clip_box: Option<[f32; 4]>, // [x_min, y_min, x_max, y_max] in pixels
    /// Stack of saved clip states: (glyph_id, clip_box) pairs.
    clip_stack: Vec<(Option<u16>, Option<[f32; 4]>)>,
    output: Vec<u8>,
    pub(super) output_w: u32,
    pub(super) output_h: u32,
    offset_x: f32,
    offset_y: f32,
    layer_stack: Vec<Vec<u8>>,
}

impl ColrV1Painter {
    pub(super) fn new(palette: Vec<[u8; 4]>, font: FontArc, font_size: f32) -> Self {
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

    pub(super) fn into_rgba(self) -> Option<(Vec<u8>, u32, u32, f32, f32)> {
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
        debug!("[COLRv1] push_transform");
    }
    fn pop_transform(&mut self) {
        debug!("[COLRv1] pop_transform");
    }

    fn push_layer(&mut self, _mode: color::CompositeMode) {
        debug!("[COLRv1] push_layer");
        self.layer_stack
            .push(vec![0u8; (self.output_w * self.output_h * 4) as usize]);
    }

    fn pop_layer(&mut self) {
        debug!("[COLRv1] pop_layer");
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
                debug!("[COLRv1] fill: unsupported brush type, skipping");
                return;
            }
        };
        let colour = self
            .palette
            .get(palette_index as usize)
            .copied()
            .unwrap_or([255, 0, 255, 255]);

        debug!(
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
                    debug!("[COLRv1] fill Path A: succeeded for gid={}", gid);
                    return;
                }
            }
            debug!(
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

            debug!(
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
        debug!("[COLRv1] push_clip_glyph: gid={}", glyph_id.to_u32());
        self.clip_stack.push((self.current_glyph_id, self.clip_box));
        self.current_glyph_id = Some(glyph_id.to_u32() as u16);
        self.clip_box = None;
    }

    fn push_clip_box(&mut self, bbox: skrifa::raw::types::BoundingBox<f32>) {
        let scale = self.font_size / self.units_per_em;
        debug!(
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
        debug!("[COLRv1] pop_clip");
        if let Some((gid, cb)) = self.clip_stack.pop() {
            self.current_glyph_id = gid;
            self.clip_box = cb;
        } else {
            self.current_glyph_id = None;
            self.clip_box = None;
        }
    }
}

pub(super) fn read_cpal(font: &skrifa::FontRef) -> Vec<[u8; 4]> {
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
