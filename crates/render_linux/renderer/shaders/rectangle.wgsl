// WARNING: AI GENERATED; UNDER REVIEW
struct Uniform {
    // Rect position and size in pixels
    rect_position: vec2f,
    rect_size: vec2f,
    // Screen dimensions in pixels (used for normalization later)
    screen_size: vec2f,
    rounding: vec4f,
    color: vec4f,
}

@group(0) @binding(0) var<uniform> uniforms: Uniform;

struct VertexInput {
    @location(0) position: vec2f,
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
    @location(1) pixel_position: vec2f,
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
    output.pixel_position = pixel_position;
    return output;
}

// Human starting here
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
    var color = input.color;

    let half_size = uniforms.rect_size / 2.0;
    let origin_position = (input.pixel_position - uniforms.rect_position) - half_size; // (pixel_pos - offset -> coords from 0,0) - size/2 -> center at 0,0
    // rounding order with these if else clauses: w z y x
    if origin_position.y > 0 {
        // Above origin
        if origin_position.x > 0 {
            // Bottom-right
            if origin_position.x < half_size.x - uniforms.rounding.w || origin_position.y < half_size.y - uniforms.rounding.w {
                return color;
            } else {
                let a = origin_position.x - half_size.x + uniforms.rounding.w;
                let b = origin_position.y - half_size.y + uniforms.rounding.w;
                if (a * a)/*a²*/ + (b * b)/*b²*/ < uniforms.rounding.w * uniforms.rounding.w {
                    return color;
                }
                discard;
            }
        } else {
            // Bottom-left
            if -origin_position.x < half_size.x - uniforms.rounding.z || origin_position.y < half_size.y - uniforms.rounding.z {
                return color;
            } else {
                let a = origin_position.x + half_size.x - uniforms.rounding.z;
                let b = origin_position.y - half_size.y + uniforms.rounding.z;
                if (a * a)/*a²*/ + (b * b)/*b²*/ < uniforms.rounding.z * uniforms.rounding.z {
                    return color;
                }
                discard;
            }
        }
    } else {
        // Below origin
        if origin_position.x > 0 {
            // Top-right
            if origin_position.x < half_size.x - uniforms.rounding.y || -origin_position.y < half_size.y - uniforms.rounding.y {
                return color;
            } else {
                let a = origin_position.x - half_size.x + uniforms.rounding.y;
                let b = origin_position.y + half_size.y - uniforms.rounding.y;
                if (a * a)/*a²*/ + (b * b)/*b²*/ < uniforms.rounding.y * uniforms.rounding.y {
                    return color;
                }
                discard;
            }
        } else {
            // Top-left
            if -origin_position.x < half_size.x - uniforms.rounding.x || -origin_position.y < half_size.y - uniforms.rounding.x {
                return color;
            } else {
                let a = origin_position.x + half_size.x - uniforms.rounding.x;
                let b = origin_position.y + half_size.y - uniforms.rounding.x;
                if (a * a)/*a²*/ + (b * b)/*b²*/ < uniforms.rounding.x * uniforms.rounding.x {
                    return color;
                }
                discard;
            }
        }
    }

    return color;
}
