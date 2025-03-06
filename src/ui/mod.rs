pub mod hud;
pub mod minimap;
pub mod menus;

use anyhow::Result;
use glam::{Vec2, Vec4};
use wgpu::{Device, Queue, RenderPass};
use std::collections::HashMap;

use crate::engine::assets::TextureAsset;
use crate::ecs::resources::GameMap;
use crate::ecs::components::ResourceType;
use crate::game::GameState;

// UI Element types
pub enum UiElementType {
    Button,
    Panel,
    Text,
    Image,
    ProgressBar,
}

// UI Element alignment
pub enum UiAlignment {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

// UI color scheme
pub struct UiColorScheme {
    pub background: Vec4,
    pub foreground: Vec4,
    pub accent: Vec4,
    pub button: Vec4,
    pub button_hover: Vec4,
    pub button_active: Vec4,
    pub text: Vec4,
    pub border: Vec4,
}

impl Default for UiColorScheme {
    fn default() -> Self {
        Self {
            background: Vec4::new(0.1, 0.1, 0.1, 0.8),
            foreground: Vec4::new(0.2, 0.2, 0.2, 0.9),
            accent: Vec4::new(0.0, 0.5, 0.8, 1.0),
            button: Vec4::new(0.3, 0.3, 0.3, 1.0),
            button_hover: Vec4::new(0.4, 0.4, 0.4, 1.0),
            button_active: Vec4::new(0.5, 0.5, 0.5, 1.0),
            text: Vec4::new(0.9, 0.9, 0.9, 1.0),
            border: Vec4::new(0.5, 0.5, 0.5, 1.0),
        }
    }
}

// UI Element trait
pub trait UiElement {
    fn get_type(&self) -> UiElementType;
    fn get_position(&self) -> Vec2;
    fn get_size(&self) -> Vec2;
    fn is_visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
    fn contains_point(&self, point: Vec2) -> bool;
    fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline);
    fn handle_click(&mut self, position: Vec2) -> bool;
}

// UI Pipeline for rendering UI elements
pub struct UiPipeline {
    device: Device,
    queue: Queue,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    text_atlas: Option<TextureAsset>,
    ui_textures: HashMap<String, TextureAsset>,
}

impl UiPipeline {
    pub fn new(device: Device, queue: Queue, surface_format: wgpu::TextureFormat) -> Result<Self> {
        // Create bind group layout for textures
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("UI Texture Bind Group Layout"),
            entries: &[
                // Texture binding
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler binding
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        
        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Load shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../assets/shaders/ui.wgsl").into()),
        });
        
        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    // Vertex layout
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<UiVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            // Position
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            // Texture coordinates
                            wgpu::VertexAttribute {
                                offset: 8,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            // Color
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        Ok(Self {
            device,
            queue,
            pipeline,
            bind_group_layout,
            text_atlas: None,
            ui_textures: HashMap::new(),
        })
    }
    
    pub fn add_texture(&mut self, name: &str, texture: TextureAsset) {
        self.ui_textures.insert(name.to_string(), texture);
    }
    
    pub fn set_text_atlas(&mut self, texture: TextureAsset) {
        self.text_atlas = Some(texture);
    }
    
    pub fn create_bind_group(&self, texture: &TextureAsset) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("UI Texture Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        })
    }
}

// UI vertex data
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

// UI Manager to handle all UI elements
pub struct UiManager {
    screen_size: Vec2,
    ui_elements: HashMap<String, Box<dyn UiElement>>,
    ui_pipeline: UiPipeline,
    color_scheme: UiColorScheme,
    active_screen: String,
    hud: hud::Hud,
    minimap: minimap::Minimap,
}

impl UiManager {
    pub fn new(
        device: Device,
        queue: Queue,
        screen_width: u32,
        screen_height: u32,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self> {
        let ui_pipeline = UiPipeline::new(device, queue, surface_format)?;
        
        Ok(Self {
            screen_size: Vec2::new(screen_width as f32, screen_height as f32),
            ui_elements: HashMap::new(),
            ui_pipeline,
            color_scheme: UiColorScheme::default(),
            active_screen: "game".to_string(),
            hud: hud::Hud::new(),
            minimap: minimap::Minimap::new(),
        })
    }
    
    pub fn add_element(&mut self, id: &str, element: Box<dyn UiElement>) {
        self.ui_elements.insert(id.to_string(), element);
    }
    
    pub fn remove_element(&mut self, id: &str) {
        self.ui_elements.remove(id);
    }
    
    pub fn handle_input(&mut self, position: Vec2) -> bool {
        // Check if any UI element was clicked
        for element in self.ui_elements.values_mut() {
            if element.is_visible() && element.contains_point(position) {
                return element.handle_click(position);
            }
        }
        
        // Check HUD elements
        if self.hud.handle_input(position) {
            return true;
        }
        
        // Check minimap
        if self.minimap.handle_input(position) {
            return true;
        }
        
        false
    }
    
    pub fn update(&mut self, game_state: &GameState) {
        // Update HUD with game state
        self.hud.update(game_state);
        
        // Update minimap
        self.minimap.update(game_state);
    }
    
    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        // Set pipeline
        render_pass.set_pipeline(&self.ui_pipeline.pipeline);
        
        // Render all visible UI elements
        for element in self.ui_elements.values() {
            if element.is_visible() {
                element.render(render_pass, &self.ui_pipeline);
            }
        }
        
        // Render HUD
        self.hud.render(render_pass, &self.ui_pipeline);
        
        // Render minimap
        self.minimap.render(render_pass, &self.ui_pipeline);
    }
    
    pub fn set_active_screen(&mut self, screen_id: &str) {
        self.active_screen = screen_id.to_string();
        
        // Hide all elements not on this screen
        for (id, element) in self.ui_elements.iter_mut() {
            element.set_visible(id.starts_with(&format!("{}_", screen_id)));
        }
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_size = Vec2::new(width as f32, height as f32);
        
        // Update minimap position
        self.minimap.resize(width, height);
        
        // Update HUD layout
        self.hud.resize(width, height);
    }
}