use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use image::{GenericImageView, RgbaImage};
use wgpu::{Device, Queue, Texture, TextureView, Sampler, TextureFormat};

/// Asset type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Texture,
    Sound,
    Shader,
}

/// Represents a loaded texture
pub struct TextureAsset {
    pub texture: Texture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub width: u32,
    pub height: u32,
}

/// Represents a loaded sound
pub struct SoundAsset {
    // We'll use placeholder fields for now
    pub data: Vec<u8>,
    pub sample_rate: u32,
}

/// Asset manager to load and cache game assets
pub struct AssetManager {
    assets_path: PathBuf,
    textures: HashMap<String, Arc<TextureAsset>>,
    sounds: HashMap<String, Arc<SoundAsset>>,
    device: Device,
    queue: Queue,
}

impl AssetManager {
    pub fn new(assets_path: impl AsRef<Path>, device: Device, queue: Queue) -> Self {
        Self {
            assets_path: assets_path.as_ref().to_path_buf(),
            textures: HashMap::new(),
            sounds: HashMap::new(),
            device,
            queue,
        }
    }
    
    /// Load a texture from a file
    pub fn load_texture(&mut self, name: &str, path: &str) -> Result<Arc<TextureAsset>> {
        let key = name.to_string();
        
        // Return cached texture if already loaded
        if let Some(texture) = self.textures.get(&key) {
            return Ok(texture.clone());
        }
        
        // Load the image
        let full_path = self.assets_path.join("textures").join(path);
        let image = image::open(full_path)?;
        let rgba_image = image.to_rgba8();
        
        let dimensions = image.dimensions();
        
        // Create the texture
        let texture_asset = create_texture(
            &self.device,
            &self.queue,
            &rgba_image,
            dimensions.0,
            dimensions.1,
            Some(name),
        )?;
        
        // Cache and return
        let texture_arc = Arc::new(texture_asset);
        self.textures.insert(key, texture_arc.clone());
        
        Ok(texture_arc)
    }
    
    /// Load a sound from a file
    pub fn load_sound(&mut self, name: &str, path: &str) -> Result<Arc<SoundAsset>> {
        let key = name.to_string();
        
        // Return cached sound if already loaded
        if let Some(sound) = self.sounds.get(&key) {
            return Ok(sound.clone());
        }
        
        // In a real implementation, we would load and decode the audio file
        // For now, we'll just create a placeholder sound asset
        let full_path = self.assets_path.join("audio").join(path);
        let data = std::fs::read(full_path)?;
        
        let sound_asset = SoundAsset {
            data,
            sample_rate: 44100, // Default sample rate
        };
        
        // Cache and return
        let sound_arc = Arc::new(sound_asset);
        self.sounds.insert(key, sound_arc.clone());
        
        Ok(sound_arc)
    }
    
    /// Get a loaded texture
    pub fn get_texture(&self, name: &str) -> Option<Arc<TextureAsset>> {
        self.textures.get(name).cloned()
    }
    
    /// Get a loaded sound
    pub fn get_sound(&self, name: &str) -> Option<Arc<SoundAsset>> {
        self.sounds.get(name).cloned()
    }
    
    /// Clear unused assets from memory
    pub fn clear_unused(&mut self) {
        // Remove textures with only one reference (the one in our HashMap)
        self.textures.retain(|_, texture| Arc::strong_count(texture) > 1);
        
        // Remove sounds with only one reference
        self.sounds.retain(|_, sound| Arc::strong_count(sound) > 1);
    }
}

/// Helper function to create a texture from an image
fn create_texture(
    device: &Device,
    queue: &Queue,
    rgba_image: &RgbaImage,
    width: u32,
    height: u32,
    label: Option<&str>,
) -> Result<TextureAsset> {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba_image,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );
    
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    
    Ok(TextureAsset {
        texture,
        view,
        sampler,
        width,
        height,
    })
}