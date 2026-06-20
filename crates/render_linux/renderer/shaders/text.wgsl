// WARNING: AI GENERATED; UNDER REVIEW

struct Uniforms {
    screen_size: vec2<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var text_texture: texture_2d<f32>;

@group(0) @binding(2)
var glyph_sampler: sampler;

@group(0) @binding(3)
var color_texture: texture_2d<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tex_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) tex_index: u32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Convert pixel coordinates to NDC (y-down → y-up)
    let x = (in.position.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let y = 1.0 - (in.position.y / uniforms.screen_size.y) * 2.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = in.uv;
    out.tex_index = in.tex_index;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.tex_index == 1u) {
        // Color emoji: sample RGBA directly from the color atlas.
        return textureSample(color_texture, glyph_sampler, in.uv);
    }
    // Text glyph: the atlas is R8Unorm — the red channel holds coverage.
    let alpha = textureSample(text_texture, glyph_sampler, in.uv).r;
    return vec4<f32>(uniforms.color.rgb, uniforms.color.a * alpha);
}
