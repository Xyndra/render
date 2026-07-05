// WARNING: AI GENERATED; UNDER REVIEW

use ab_glyph::{Font, FontArc, FontVec, GlyphId, PxScale, ScaleFont};
use bytemuck::cast_slice;
use fontdb::{Family, Source};
use render_components::primitives::Text;
use render_layout::InternalLayoutable;
use skrifa::MetadataProvider;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::fs::read;
use swash::FontRef;
use swash::scale::image::Content;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, BlendState, BufferDescriptor,
    BufferUsages, ColorTargetState, ColorWrites, FragmentState, FrontFace, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPipelineDescriptor, TextureFormat, VertexState, include_wgsl,
};

mod atlas;
mod colr;
pub(crate) mod debug;
mod shaping;
mod types;

use atlas::{GlyphAtlas, GlyphEntry, GlyphKey};
use colr::{ColrV1Painter, read_cpal};
use shaping::{FontGroupRanges, fit_text};
use types::{TextUniform, TextVertex};

use crate::debug;

// ── Constants ──────────────────────────────────────────────────

const ATLAS_SIZE: u32 = 2048;
/// Transparent border (in atlas pixels) kept around every glyph so that
/// linear filtering at glyph-quad edges does not bleed into neighbours.
const ATLAS_PADDING: u32 = 1;
const MIN_FONT_SIZE: f32 = 6.0;
const LINE_HEIGHT_RATIO: f32 = 1.2;
const MAX_TEXT_VERTICES: usize = 16384; // 4096 glyphs × 4 verts

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

        let shader = device.create_shader_module(include_wgsl!("../shaders/text.wgsl"));

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

    pub(crate) fn load_system_fonts()
    -> Option<(Vec<FontArc>, Vec<Vec<u8>>, Vec<u32>, FontGroupRanges)> {
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
                        debug!(
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
                debug!(
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
                    debug!(
                        "[COLRv1] glyph {} found in COLRv1 table, attempting paint...",
                        gid.0
                    );
                    let palette = read_cpal(&font_ref);
                    let mut painter =
                        ColrV1Painter::new(palette, self.fonts[font_index].clone(), font_size);
                    let paint_result =
                        color_glyph.paint(skrifa::instance::LocationRef::default(), &mut painter);
                    debug!(
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
