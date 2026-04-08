// WARNING: FULLY AI GENERATED; PLEASE REVIEW
//
// Rectangle shader for WGSL
// This shader renders solid-colored rectangles

struct Uniform {
    // Rectangle position and size in pixel coordinates
    rect_position: vec2f,
    rect_size: vec2f,
    // Screen dimensions in pixels
    screen_size: vec2f,
    // Padding for vec4 alignment
    // Rounding radius (0 for sharp corners)
    rounding: f32,
    _padding1: f32,
    // Rectangle color (RGBA)
    color: vec4f,
}

@group(0) @binding(0) var<uniform> uniforms: Uniform;

struct VertexInput {
    @location(0) position: vec2f,
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
    @location(1) local_position: vec2f,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    // Transform from local quad coordinates [0,1] to pixel coordinates
    let pixel_position = uniforms.rect_position + input.position * uniforms.rect_size;

    // Convert from pixel coordinates to normalized device coordinates (NDC)
    // NDC ranges from -1 to 1, with (0,0) at center
    // Screen coordinates have (0,0) at top-left, so we need to flip Y
    let ndc_position = vec2f(
        (pixel_position.x / uniforms.screen_size.x) * 2.0 - 1.0,
        1.0 - (pixel_position.y / uniforms.screen_size.y) * 2.0  // Flip Y for screen coordinates
    );

    var output: VertexOutput;
    output.position = vec4f(ndc_position, 0.0, 1.0);
    output.color = uniforms.color;
    output.local_position = input.position;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
    var color = input.color;

    // Handle rounded corners if rounding > 0
    if uniforms.rounding > 0.0 {
        // Convert local position [0,1] to pixel coordinates within the rectangle
        let local_pixel = input.local_position * uniforms.rect_size;

        // Calculate distance to nearest corner
        let corner_size = vec2f(uniforms.rounding, uniforms.rounding);

        // Check each corner region
        if local_pixel.x < uniforms.rounding && local_pixel.y < uniforms.rounding {
            // Top-left corner
            let corner_center = corner_size;
            let dist = distance(local_pixel, corner_center);
            if dist > uniforms.rounding {
                discard;
            }
        } else if local_pixel.x > uniforms.rect_size.x - uniforms.rounding && local_pixel.y < uniforms.rounding {
            // Top-right corner
            let corner_center = vec2f(uniforms.rect_size.x - uniforms.rounding, uniforms.rounding);
            let dist = distance(local_pixel, corner_center);
            if dist > uniforms.rounding {
                discard;
            }
        } else if local_pixel.x < uniforms.rounding && local_pixel.y > uniforms.rect_size.y - uniforms.rounding {
            // Bottom-left corner
            let corner_center = vec2f(uniforms.rounding, uniforms.rect_size.y - uniforms.rounding);
            let dist = distance(local_pixel, corner_center);
            if dist > uniforms.rounding {
                discard;
            }
        } else if local_pixel.x > uniforms.rect_size.x - uniforms.rounding && local_pixel.y > uniforms.rect_size.y - uniforms.rounding {
            // Bottom-right corner
            let corner_center = vec2f(uniforms.rect_size.x - uniforms.rounding, uniforms.rect_size.y - uniforms.rounding);
            let dist = distance(local_pixel, corner_center);
            if dist > uniforms.rounding {
                discard;
            }
        }
    }

    return color;
}
