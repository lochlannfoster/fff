#!/bin/bash

# Simple build script for Rusty RTS

# Create assets directory structure if it doesn't exist
mkdir -p assets/shaders
mkdir -p assets/textures
mkdir -p assets/audio

# Ensure the sprite shader is in place
cat > assets/shaders/sprite.wgsl << 'EOF'
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
EOF

# Create UI shader
cat > assets/shaders/ui.wgsl << 'EOF'
// UI vertex and fragment shader
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Convert from pixel coordinates to clip space
    let clip_pos = vec2<f32>(
        vertex.position.x / 512.0 - 1.0,  // Convert to -1 to 1
        1.0 - vertex.position.y / 384.0   // Flip Y for clip space
    );
    out.clip_position = vec4<f32>(clip_pos, 0.0, 1.0);
    out.tex_coords = vertex.tex_coords;
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
EOF

# Check if Cargo.toml exists, otherwise create it
if [ ! -f "Cargo.toml" ]; then
  cat > Cargo.toml << 'EOF'
[package]
name = "rusty_rts"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]

[dependencies]
# Graphics
wgpu = { version = "0.17", features = ["spirv"] }
winit = "0.28"
pollster = "0.3"         # For async to sync conversion in wgpu
bytemuck = { version = "1.14", features = ["derive"] } # For GPU data conversion
glam = "0.24"            # Math library for graphics
image = "0.24"           # Image loading

# ECS and game logic
bevy_ecs = "0.11"        # Just the ECS part of Bevy
ron = "0.8"              # Rusty Object Notation for config files
noise = "0.8"            # For procedural terrain generation
pathfinding = "4.3"      # A* implementation

# Networking
bincode = "1.3"          # Binary serialization
serde = { version = "1.0", features = ["derive"] }
quinn = "0.10"           # QUIC protocol implementation (UDP+TLS)

# Utils
log = "0.4"
env_logger = "0.10"
anyhow = "1.0"           # Error handling
thiserror = "1.0"        # Error definition

[profile.dev.package."*"]
# Compile dependencies with optimizations in dev mode
opt-level = 3

[profile.dev]
opt-level = 1

[profile.release]
opt-level = 3
lto = "thin"
EOF
fi

# Build the project
echo "Building Rusty RTS..."
cargo build

# Check if the build was successful
if [ $? -eq 0 ]; then
    echo "Build successful! You can run the game with 'cargo run'"
else
    echo "Build failed. Please check the error messages above."
fi
