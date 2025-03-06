use anyhow::Result;
use wgpu::{
    Device, Queue, Surface, SurfaceConfiguration, 
    Adapter, Instance, InstanceDescriptor, Backends,
    ShaderModule, PipelineLayout, RenderPipeline,
    TextureFormat, PresentMode, Buffer, BindGroup,
};
use winit::window::Window;
use bevy_ecs::world::World;
use glam::{Vec2, Vec4, Mat4};
use std::collections::HashMap;

use crate::ecs::components::{Transform, Unit, Building, Owner, Resource, MinimapMarker, UnitType, BuildingType, ResourceType, Selected};
use crate::ui::UiManager;

// Vertex format for entities (sprites)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

// Uniforms for camera and transforms
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_projection: [[f32; 4]; 4],
}

pub struct Renderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    sprite_pipeline: RenderPipeline,
    camera_uniform_buffer: Buffer,
    camera_bind_group: BindGroup,
    view_projection: Mat4,
    camera_position: Vec2,
    camera_zoom: f32,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    
    // Placeholder colored rectangles for different entity types
    unit_colors: HashMap<UnitType, [f32; 4]>,
    building_colors: HashMap<BuildingType, [f32; 4]>,
    resource_colors: HashMap<ResourceType, [f32; 4]>,
    player_colors: HashMap<u8, [f32; 4]>,
}

impl Renderer {
    pub async fn new(window: &Window) -> Result<Self> {
        // Create instance
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        // Create surface
        let surface = unsafe { instance.create_surface(&window) }?;
        
        // Find adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find an appropriate adapter"))?;
        
        // Create device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;
        
        // Configure surface
        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
            
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        
        // Create camera uniform buffer
        let view_projection = create_view_projection_matrix(
            Vec2::new(128.0, 128.0), // Center of the map initially
            1.0, // Initial zoom
            config.width as f32 / config.height as f32, // Aspect ratio
        );
        
        let camera_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Update camera uniforms
        let camera_uniforms = Uniforms {
            view_projection: view_projection.to_cols_array_2d(),
        };
        
        queue.write_buffer(
            &camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniforms]),
        );
        
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Create bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Load shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../assets/shaders/sprite.wgsl").into()),
        });
        
        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create sprite pipeline
        let sprite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            // Position
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            // Texture coordinates
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            // Color
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
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
                    format,
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
        
        // Create vertex buffer with placeholder quad
        let vertices = create_quad_vertices();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        // Create index buffer with quad indices
        let indices = create_quad_indices();
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        // Set up entity color placeholders
        let mut unit_colors = HashMap::new();
        unit_colors.insert(UnitType::Worker, [0.0, 0.8, 0.0, 1.0]); // Green
        unit_colors.insert(UnitType::Soldier, [0.8, 0.0, 0.0, 1.0]); // Red
        unit_colors.insert(UnitType::Scout, [0.0, 0.0, 0.8, 1.0]); // Blue
        unit_colors.insert(UnitType::Tank, [0.8, 0.8, 0.0, 1.0]); // Yellow
        unit_colors.insert(UnitType::Healer, [0.8, 0.0, 0.8, 1.0]); // Purple
        
        let mut building_colors = HashMap::new();
        building_colors.insert(BuildingType::Headquarters, [0.7, 0.7, 0.7, 1.0]); // Light Gray
        building_colors.insert(BuildingType::Barracks, [0.6, 0.3, 0.3, 1.0]); // Brown
        building_colors.insert(BuildingType::Factory, [0.4, 0.4, 0.4, 1.0]); // Dark Gray
        building_colors.insert(BuildingType::ResourceCollector, [0.3, 0.6, 0.3, 1.0]); // Dark Green
        building_colors.insert(BuildingType::ResearchCenter, [0.3, 0.3, 0.6, 1.0]); // Dark Blue
        building_colors.insert(BuildingType::DefenseTower, [0.6, 0.6, 0.3, 1.0]); // Brown Yellow
        
        let mut resource_colors = HashMap::new();
        resource_colors.insert(ResourceType::Mineral, [0.0, 0.5, 1.0, 1.0]); // Light Blue
        resource_colors.insert(ResourceType::Gas, [0.0, 1.0, 0.5, 1.0]); // Light Green
        resource_colors.insert(ResourceType::Energy, [1.0, 1.0, 0.0, 1.0]); // Yellow
        
        let mut player_colors = HashMap::new();
        player_colors.insert(0, [0.0, 0.0, 1.0, 1.0]); // Blue
        player_colors.insert(1, [1.0, 0.0, 0.0, 1.0]); // Red
        player_colors.insert(2, [0.0, 1.0, 0.0, 1.0]); // Green
        player_colors.insert(3, [1.0, 1.0, 0.0, 1.0]); // Yellow
        
        Ok(Self {
            surface,
            device,
            queue,
            config,
            sprite_pipeline,
            camera_uniform_buffer,
            camera_bind_group,
            view_projection,
            camera_position: Vec2::new(128.0, 128.0),
            camera_zoom: 1.0,
            vertex_buffer,
            index_buffer,
            unit_colors,
            building_colors,
            resource_colors,
            player_colors,
        })
    }
    
    pub fn render(&mut self, world: &World, ui_manager: &UiManager) -> Result<()> {
        // Get a frame to render to
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            // Begin render pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });
            
            // Render game world entities
            self.render_world(&mut render_pass, world);
            
            // Render UI
            ui_manager.render(&mut render_pass);
        }
        
        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
    
    fn render_world<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, world: &'a World) {
        // Set the pipeline
        render_pass.set_pipeline(&self.sprite_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        
        // Render all entities
        // First, render resources
        let mut resource_query = world.query::<(&Resource, &Transform)>();
        for (resource, transform) in resource_query.iter(world) {
            let color = self.resource_colors.get(&resource.resource_type).unwrap_or(&[1.0, 1.0, 1.0, 1.0]);
            let model = self.calculate_model_matrix(transform, 0.8); // Smaller size for resources
            
            // Set push constants (in real implementation, would use instance rendering)
            // For now, just render a colored quad
            render_pass.draw_indexed(0..6, 0, 0..1);
        }
        
        // Render buildings
        let mut building_query = world.query::<(&Building, &Transform, &Owner, Option<&Selected>)>();
        for (building, transform, owner, selected) in building_query.iter(world) {
            let base_color = self.building_colors.get(&building.building_type).unwrap_or(&[1.0, 1.0, 1.0, 1.0]);
            let player_color = self.player_colors.get(&owner.0).unwrap_or(&[1.0, 1.0, 1.0, 1.0]);
            
            // Mix base color with player color
            let color = [
                base_color[0] * 0.5 + player_color[0] * 0.5,
                base_color[1] * 0.5 + player_color[1] * 0.5,
                base_color[2] * 0.5 + player_color[2] * 0.5,
                1.0,
            ];
            
            // Scale for building size - headquarters bigger than other buildings
            let scale = if building.building_type == BuildingType::Headquarters {
                2.0
            } else {
                1.5
            };
            
            let model = self.calculate_model_matrix(transform, scale);
            
            // Draw the building
            render_pass.draw_indexed(0..6, 0, 0..1);
            
            // Draw selection indicator if selected
            if selected.is_some() {
                // Draw outline
                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }
        
        // Render units
        let mut unit_query = world.query::<(&Unit, &Transform, &Owner, Option<&Selected>)>();
        for (unit, transform, owner, selected) in unit_query.iter(world) {
            let base_color = self.unit_colors.get(&unit.unit_type).unwrap_or(&[1.0, 1.0, 1.0, 1.0]);
            let player_color = self.player_colors.get(&owner.0).unwrap_or(&[1.0, 1.0, 1.0, 1.0]);
            
            // Mix base color with player color
            let color = [
                base_color[0] * 0.3 + player_color[0] * 0.7,
                base_color[1] * 0.3 + player_color[1] * 0.7,
                base_color[2] * 0.3 + player_color[2] * 0.7,
                1.0,
            ];
            
            let model = self.calculate_model_matrix(transform, 0.5); // Units are smaller
            
            // Draw the unit
            render_pass.draw_indexed(0..6, 0, 0..1);
            
            // Draw selection indicator if selected
            if selected.is_some() {
                // Draw outline
                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }
    }
    
    fn calculate_model_matrix(&self, transform: &Transform, scale_multiplier: f32) -> Mat4 {
        // Calculate model matrix from transform
        let translate = Mat4::from_translation(glam::Vec3::new(
            transform.position.x,
            transform.position.y,
            0.0,
        ));
        
        let rotate = Mat4::from_rotation_z(transform.rotation);
        
        let scale = Mat4::from_scale(glam::Vec3::new(
            transform.scale.x * scale_multiplier,
            transform.scale.y * scale_multiplier,
            1.0,
        ));
        
        translate * rotate * scale
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            // Update the camera projection matrix
            self.view_projection = create_view_projection_matrix(
                self.camera_position,
                self.camera_zoom,
                new_size.width as f32 / new_size.height as f32,
            );
            
            let camera_uniforms = Uniforms {
                view_projection: self.view_projection.to_cols_array_2d(),
            };
            
            self.queue.write_buffer(
                &self.camera_uniform_buffer,
                0,
                bytemuck::cast_slice(&[camera_uniforms]),
            );
        }
    }
    
    pub fn update_camera(&mut self, position: Vec2, zoom: f32) {
        self.camera_position = position;
        self.camera_zoom = zoom;
        
        // Update camera matrix
        self.view_projection = create_view_projection_matrix(
            position,
            zoom,
            self.config.width as f32 / self.config.height as f32,
        );
        
        let camera_uniforms = Uniforms {
            view_projection: self.view_projection.to_cols_array_2d(),
        };
        
        self.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniforms]),
        );
    }
    
    pub fn get_device(&self) -> &Device {
        &self.device
    }
    
    pub fn get_queue(&self) -> &Queue {
        &self.queue
    }
    
    pub fn get_surface_format(&self) -> TextureFormat {
        self.config.format
    }
}

// Helper functions for creating basic geometry

fn create_quad_vertices() -> [Vertex; 4] {
    [
        Vertex {
            position: [-0.5, -0.5, 0.0],
            tex_coords: [0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            tex_coords: [1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex {
            position: [0.5, 0.5, 0.0],
            tex_coords: [1.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex {
            position: [-0.5, 0.5, 0.0],
            tex_coords: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
    ]
}

fn create_quad_indices() -> [u16; 6] {
    [0, 1, 2, 0, 2, 3]
}

fn create_view_projection_matrix(position: Vec2, zoom: f32, aspect_ratio: f32) -> Mat4 {
    // Calculate view matrix (camera position)
    let view = Mat4::from_translation(glam::Vec3::new(-position.x, -position.y, 0.0));
    
    // Calculate projection matrix (orthographic for 2D)
    let half_width = 400.0 / zoom;
    let half_height = half_width / aspect_ratio;
    
    let projection = Mat4::orthographic_rh(
        -half_width,
        half_width,
        -half_height,
        half_height,
        -100.0,
        100.0,
    );
    
    projection * view
}