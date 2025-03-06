use glam::{Vec2, Vec4};
use wgpu::RenderPass;
use std::collections::HashMap;

use crate::game::GameState;
use crate::ecs::resources::GameMap;
use crate::ecs::components::{Owner, UnitType, BuildingType};
use crate::ui::{UiPipeline, UiElement, UiElementType};

/// Minimap for the RTS game
pub struct Minimap {
    position: Vec2,
    size: Vec2,
    visible: bool,
    texture_data: Vec<u8>,
    texture_width: u32,
    texture_height: u32,
    camera_position: Vec2,
    camera_size: Vec2,
    map_width: u32,
    map_height: u32,
    unit_markers: Vec<UnitMarker>,
    building_markers: Vec<BuildingMarker>,
    player_colors: HashMap<u8, [u8; 4]>,
}

/// Marker for units on the minimap
struct UnitMarker {
    position: Vec2,
    color: [u8; 4],
    unit_type: UnitType,
    entity_id: u32,
}

/// Marker for buildings on the minimap
struct BuildingMarker {
    position: Vec2,
    size: Vec2,
    color: [u8; 4],
    building_type: BuildingType,
    entity_id: u32,
}

impl Minimap {
    pub fn new() -> Self {
        // Default player colors
        let mut player_colors = HashMap::new();
        player_colors.insert(0, [0, 0, 255, 255]);     // Blue
        player_colors.insert(1, [255, 0, 0, 255]);     // Red
        player_colors.insert(2, [0, 255, 0, 255]);     // Green
        player_colors.insert(3, [255, 255, 0, 255]);   // Yellow
        player_colors.insert(4, [128, 0, 128, 255]);   // Purple
        player_colors.insert(5, [0, 255, 255, 255]);   // Cyan
        player_colors.insert(6, [255, 128, 0, 255]);   // Orange
        player_colors.insert(7, [255, 0, 255, 255]);   // Magenta
        
        Self {
            position: Vec2::new(0.0, 0.0),
            size: Vec2::new(150.0, 150.0),
            visible: true,
            texture_data: Vec::new(),
            texture_width: 256,
            texture_height: 256,
            camera_position: Vec2::new(0.0, 0.0),
            camera_size: Vec2::new(0.0, 0.0),
            map_width: 256,
            map_height: 256,
            unit_markers: Vec::new(),
            building_markers: Vec::new(),
            player_colors,
        }
    }
    
    pub fn update(&mut self, game_state: &GameState) {
        // In a real implementation, this would update unit and building markers
        // from the ECS world
    }
    
    pub fn set_map_data(&mut self, map: &GameMap) {
        self.map_width = map.width;
        self.map_height = map.height;
        
        // Generate minimap texture from map data
        self.texture_data = crate::game::map::generate_minimap(map);
        self.texture_width = map.width;
        self.texture_height = map.height;
    }
    
    pub fn set_camera(&mut self, position: Vec2, view_width: f32, view_height: f32) {
        self.camera_position = position;
        self.camera_size = Vec2::new(view_width, view_height);
    }
    
    pub fn update_unit_positions(&mut self, units: &[(u32, UnitType, Vec2, u8)]) {
        // Clear existing markers
        self.unit_markers.clear();
        
        // Add new markers
        for &(entity_id, unit_type, position, owner) in units {
            let color = self.player_colors.get(&owner).copied().unwrap_or([255, 255, 255, 255]);
            
            self.unit_markers.push(UnitMarker {
                position,
                color,
                unit_type,
                entity_id,
            });
        }
    }
    
    pub fn update_building_positions(&mut self, buildings: &[(u32, BuildingType, Vec2, Vec2, u8)]) {
        // Clear existing markers
        self.building_markers.clear();
        
        // Add new markers
        for &(entity_id, building_type, position, size, owner) in buildings {
            let color = self.player_colors.get(&owner).copied().unwrap_or([255, 255, 255, 255]);
            
            self.building_markers.push(BuildingMarker {
                position,
                size,
                color,
                building_type,
                entity_id,
            });
        }
    }
    
    pub fn handle_input(&mut self, position: Vec2) -> bool {
        // Check if click is within minimap
        if position.x >= self.position.x && 
           position.x <= self.position.x + self.size.x &&
           position.y >= self.position.y && 
           position.y <= self.position.y + self.size.y {
            
            // Convert click to map coordinates
            let relative_x = (position.x - self.position.x) / self.size.x;
            let relative_y = (position.y - self.position.y) / self.size.y;
            
            let map_x = relative_x * self.map_width as f32;
            let map_y = relative_y * self.map_height as f32;
            
            // This would normally issue a command to move the camera to this location
            println!("Minimap clicked at map coordinates ({}, {})", map_x, map_y);
            
            return true;
        }
        
        false
    }
    
    pub fn resize(&mut self, screen_width: u32, screen_height: u32) {
        // Position minimap at bottom right
        self.position = Vec2::new(
            screen_width as f32 - self.size.x - 10.0,
            screen_height as f32 - self.size.y - 10.0,
        );
    }
    
    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        if !self.visible {
            return;
        }
        
        // In a real implementation, this would render:
        // 1. The minimap background texture
        // 2. Unit and building markers
        // 3. Camera view rectangle
        // 4. Fog of war overlay
    }
    
    fn convert_world_to_minimap(&self, world_pos: Vec2) -> Vec2 {
        // Convert from world coordinates to minimap coordinates
        let minimap_x = (world_pos.x / self.map_width as f32) * self.size.x + self.position.x;
        let minimap_y = (world_pos.y / self.map_height as f32) * self.size.y + self.position.y;
        Vec2::new(minimap_x, minimap_y)
    }
    
    fn convert_minimap_to_world(&self, minimap_pos: Vec2) -> Vec2 {
        // Convert from minimap coordinates to world coordinates
        let world_x = ((minimap_pos.x - self.position.x) / self.size.x) * self.map_width as f32;
        let world_y = ((minimap_pos.y - self.position.y) / self.size.y) * self.map_height as f32;
        Vec2::new(world_x, world_y)
    }
}