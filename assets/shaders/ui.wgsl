// UI Shader for RTS UI elements

// Vertex shader inputs
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
};

// Vertex shader outputs / Fragment shader inputs
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

// Vertex shader
@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Convert from pixel coordinates to clip space
    // Assuming top-left is (0,0) and bottom-right is (screen_width, screen_height)
    let screen_size = vec2<f32>(800.0, 600.0); // This should be provided via a uniform
    let clip_pos = vec2<f32>(
        vertex.position.x / screen_size.x * 2.0 - 1.0,  // Convert to -1 to 1
        -(vertex.position.y / screen_size.y * 2.0 - 1.0) // Y is flipped in clip space
    );
    
    out.clip_position = vec4<f32>(clip_pos, 0.0, 1.0);
    out.tex_coords = vertex.tex_coords;
    out.color = vertex.color;
    
    return out;
}

// Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the texture
    var tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // Multiply by vertex color
    return tex_color * in.color;
}

// Alternative fragment shader for solid color shapes (no texture)
@fragment
fn fs_color(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// Fragment shader for text rendering
@fragment
fn fs_text(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the font texture (assumed to be grayscale)
    var alpha = textureSample(t_diffuse, s_diffuse, in.tex_coords).r;
    
    // Use the alpha from the texture, but color from the vertex
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}