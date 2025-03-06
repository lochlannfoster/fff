use bevy_ecs::prelude::*;
use glam::Vec2;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

use crate::ecs::components::ResourceType;

/// Game time resource
#[derive(Resource)]
pub struct GameTime {
    pub current_tick: u64,
    pub elapsed_time: f32,
    pub delta_time: f32,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            current_tick: 0,
            elapsed_time: 0.0,
            delta_time: 1.0 / 60.0, // Default to 60 FPS
        }
    }
}

/// Terrain tile types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainTile {
    Ground,
    Water,
    Mountain,
    Forest,
}

/// Game map resource
#[derive(Resource)]
pub struct GameMap {
    pub width: u32,
    pub height: u32,
    pub terrain_tiles: Vec<TerrainTile>,
    pub resource_positions: Vec<(Vec2, ResourceType, f32)>,
    pub starting_positions: Vec<Vec2>,
    pub pathfinding_grid: Option<PathfindingGrid>,
    pub fog_of_war: HashMap<u8, HashSet<u32>>, // Player ID -> Set of visible tile indices
}

impl Default for GameMap {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
            terrain_tiles: Vec::new(),
            resource_positions: Vec::new(),
            starting_positions: Vec::new(),
            pathfinding_grid: None,
            fog_of_war: HashMap::new(),
        }
    }
}

/// Pathfinding grid node
#[derive(Debug, Clone)]
pub struct PathNode {
    pub walkable: bool,
    pub cost: f32,
}

/// Pathfinding grid resource
#[derive(Clone)]
pub struct PathfindingGrid {
    pub width: usize,
    pub height: usize,
    pub nodes: Vec<PathNode>,
}

/// Player resources
#[derive(Resource)]
pub struct PlayerResources {
    pub resources: HashMap<(u8, ResourceType), f32>, // (Player ID, Resource Type) -> Amount
    pub income_rate: HashMap<(u8, ResourceType), f32>, // (Player ID, Resource Type) -> Income per second
}

impl Default for PlayerResources {
    fn default() -> Self {
        let mut resources = HashMap::new();
        // Initial resources for player 0
        resources.insert((0, ResourceType::Mineral), 500.0);
        resources.insert((0, ResourceType::Gas), 200.0);
        resources.insert((0, ResourceType::Energy), 0.0);
        
        Self {
            resources,
            income_rate: HashMap::new(),
        }
    }
}

/// Technology types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechType {
    ImprovedHarvesting,
    ImprovedWeapons,
    ImprovedArmor,
    AdvancedUnits,
    AdvancedBuildings,
    ImprovedHealing,
    ImprovedSpeed,
}

/// Technology research state
#[derive(Resource)]
pub struct TechState {
    pub researched: HashMap<(u8, TechType), bool>, // (Player ID, Tech Type) -> Is Researched
    pub in_progress: HashMap<(u8, TechType), f32>, // (Player ID, Tech Type) -> Progress (0.0 to 1.0)
}

impl Default for TechState {
    fn default() -> Self {
        Self {
            researched: HashMap::new(),
            in_progress: HashMap::new(),
        }
    }
}

/// Game settings resource
#[derive(Resource)]
pub struct GameSettings {
    pub fog_of_war_enabled: bool,
    pub game_speed: f32,
    pub auto_save_enabled: bool,
    pub auto_save_interval: f32,
    pub show_fps: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fog_of_war_enabled: true,
            game_speed: 1.0,
            auto_save_enabled: false,
            auto_save_interval: 300.0, // 5 minutes
            show_fps: false,
        }
    }
}

/// Player info resource
#[derive(Resource)]
pub struct PlayerInfo {
    pub player_names: HashMap<u8, String>,
    pub player_colors: HashMap<u8, [u8; 4]>,
    pub ai_players: HashSet<u8>,
    pub local_player_id: u8,
}

impl Default for PlayerInfo {
    fn default() -> Self {
        let mut player_names = HashMap::new();
        player_names.insert(0, "Player".to_string());
        
        let mut player_colors = HashMap::new();
        player_colors.insert(0, [0, 0, 255, 255]); // Blue
        player_colors.insert(1, [255, 0, 0, 255]); // Red
        player_colors.insert(2, [0, 255, 0, 255]); // Green
        player_colors.insert(3, [255, 255, 0, 255]); // Yellow
        
        Self {
            player_names,
            player_colors,
            ai_players: HashSet::new(),
            local_player_id: 0,
        }
    }
}

/// Selection state resource
#[derive(Resource)]
pub struct SelectionState {
    pub selected_entities: Vec<Entity>,
    pub selection_start: Option<Vec2>,
    pub selection_end: Option<Vec2>,
    pub drag_selecting: bool,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected_entities: Vec::new(),
            selection_start: None,
            selection_end: None,
            drag_selecting: false,
        }
    }
}

/// Control groups resource
#[derive(Resource)]
pub struct ControlGroups {
    pub groups: HashMap<u8, Vec<Entity>>, // Group ID -> Entities
}

impl Default for ControlGroups {
    fn default() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }
}

/// Input action queue
#[derive(Resource)]
pub struct InputActionQueue {
    pub actions: Vec<crate::engine::input::Command>,
}

impl Default for InputActionQueue {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
        }
    }
}

/// Camera state resource
#[derive(Resource)]
pub struct CameraState {
    pub position: Vec2,
    pub zoom: f32,
    pub view_width: f32,
    pub view_height: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            view_width: 1024.0,
            view_height: 768.0,
        }
    }
}