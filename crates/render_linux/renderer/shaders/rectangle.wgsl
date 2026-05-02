// WARNING: FULLY AI GENERATED; UNDER REVIEW
struct Uniform {
    // Rect position and size in pixels
    rect_position: vec2f,
    rect_size: vec2f,
    // Screen dimensions in pixels (used for normalization later)
    screen_size: vec2f,
    // Rounding radius (0 for sharp corners)
    rounding: f32,
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
    // go from quad verices(mix of 0s and 1s depending on corner) to screen pixel coords
    let pixel_position = uniforms.rect_position + input.position * uniforms.rect_size;

    // Convert from pixel coordinates to normalized coordinates (-1 to 1)
    let normalized_position = vec2f(
        (pixel_position.x / uniforms.screen_size.x) * 2.0 - 1.0,
        1.0 - (pixel_position.y / uniforms.screen_size.y) * 2.0  // Flip Y since screen Y goes down but normalized Y goes up
    );

    var output: VertexOutput;
    output.position = vec4f(normalized_position, 0.0, 1.0);
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
