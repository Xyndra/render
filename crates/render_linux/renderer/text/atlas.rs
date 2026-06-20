// WARNING: AI GENERATED; UNDER REVIEW

use std::collections::HashMap;

use super::{ATLAS_PADDING, ATLAS_SIZE};
use wgpu::{
    Device, Extent3d, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub(super) struct GlyphKey {
    pub(super) glyph_id: u32,
    /// Font size multiplied by 10 and truncated — gives 0.1 px resolution.
    pub(super) size_tenths: u32,
    /// Index into the font fallback list.
    pub(super) font_index: u32,
}

pub(super) struct GlyphEntry {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) bearing_x: f32,
    pub(super) bearing_y: f32,
    pub(super) advance: f32,
    pub(super) uv_min: [f32; 2],
    pub(super) uv_max: [f32; 2],
}

pub(super) struct GlyphAtlas {
    pub(super) texture: Texture,
    pub(super) texture_view: TextureView,
    format: TextureFormat,
    cursor_x: u32,
    cursor_y: u32,
    shelf_height: u32,
    pub(super) glyphs: HashMap<GlyphKey, GlyphEntry>,
}

impl GlyphAtlas {
    pub(super) fn new(device: &Device, format: TextureFormat) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Text Glyph Atlas"),
            size: Extent3d {
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

    pub(super) fn clear(&mut self, queue: &Queue) {
        let bytes_per_pixel = match self.format {
            TextureFormat::R8Unorm => 1,
            TextureFormat::Rgba8Unorm => 4,
            _ => 1,
        };
        let data = vec![0u8; (ATLAS_SIZE * ATLAS_SIZE * bytes_per_pixel) as usize];
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ATLAS_SIZE * bytes_per_pixel),
                rows_per_image: Some(ATLAS_SIZE),
            },
            Extent3d {
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

    /// Reserve a `w × h` region and return its top-left origin.
    ///
    /// A transparent border of [`ATLAS_PADDING`] pixels is left around every
    /// glyph so that linear filtering at the edge of a glyph quad samples
    /// transparent texels instead of bleeding into the neighbouring glyph —
    /// this is what eliminates the one-pixel vertical-line artefacts between
    /// adjacent glyphs.  The returned position is the inner origin (where the
    /// glyph's own pixels go); the border stays zeroed.
    pub(super) fn allocate(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        if w == 0 || h == 0 {
            return Some((0, 0));
        }
        let pad = ATLAS_PADDING;
        let aw = w + pad * 2;
        let ah = h + pad * 2;
        if self.cursor_x + aw > ATLAS_SIZE {
            self.cursor_y += self.shelf_height;
            self.cursor_x = 0;
            self.shelf_height = 0;
        }
        if self.cursor_y + ah > ATLAS_SIZE {
            return None;
        }
        let pos = (self.cursor_x + pad, self.cursor_y + pad);
        self.cursor_x += aw;
        self.shelf_height = self.shelf_height.max(ah);
        Some(pos)
    }
}
