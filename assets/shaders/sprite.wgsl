// Basic sprite shader for RTS entities

// Vertex shader uniforms
struct Uniforms {
    view_projection: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Vertex shader inputs
struct VertexInput {
    @location(0) position: vec3<f32>,
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
    
    // Apply view-projection matrix to get screen position
    out.clip_position = uniforms.view_projection * vec4<f32>(vertex.position, 1.0);
    out.tex_coords = vertex.tex_coords;
    out.color = vertex.color;
    
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple colored fragment
    return in.color;
}

// Alternative fragment shader for textured sprites (when we have textures)
@fragment
fn fs_textured(in: VertexOutput) -> @location(0) vec4<f32> {
    // This would sample a texture, but for now we just use flat colors
    return in.color;
}