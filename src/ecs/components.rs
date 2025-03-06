use bevy_ecs::prelude::*;
use glam::Vec2;
use std::collections::VecDeque;
use serde::{Serialize, Deserialize};

/// Entity position, rotation, and scale
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
        }
    }
}

/// Entity owner (player id)
#[derive(Component, Debug, Clone, Copy)]
pub struct Owner(pub u8);

/// Movement component with path following
#[derive(Component, Debug)]
pub struct Movement {
    pub path: Vec<Vec2>,
    pub path_index: usize,
    pub target: Option<Vec2>,
    pub velocity: Vec2,
}

/// Collision detection component
#[derive(Component, Debug, Clone)]
pub struct Collider {
    pub radius: f32,
    pub collision_layer: u32,
    pub collision_mask: u32,
}

/// Resource types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Mineral,
    Gas,
    Energy,
}

/// Resource component
#[derive(Component, Debug)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub amount: f32,
}

/// Unit types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    Worker,
    Soldier,
    Scout,
    Tank,
    Healer,
}

/// Unit component
#[derive(Component, Debug)]
pub struct Unit {
    pub unit_type: UnitType,
    pub health: f32,
    pub max_health: f32,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub attack_speed: f32,
    pub movement_speed: f32,
    pub sight_range: f32,
    pub buildable: bool,
}

/// Building types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    Headquarters,
    Barracks,
    Factory,
    ResourceCollector,
    ResearchCenter,
    DefenseTower,
}

/// Building component
#[derive(Component, Debug)]
pub struct Building {
    pub building_type: BuildingType,
    pub health: f32,
    pub max_health: f32,
    pub production_queue: VecDeque<UnitType>,
    pub production_progress: Option<f32>,
    pub construction_progress: Option<f32>,
    pub rally_point: Option<Vec2>,
}

/// Attack target component
#[derive(Component, Debug)]
pub struct AttackTarget {
    pub target_entity: Entity,
}

/// Harvesting target component
#[derive(Component, Debug)]
pub struct HarvestTarget {
    pub target_entity: Entity,
}

/// Build target component
#[derive(Component, Debug)]
pub struct BuildTarget {
    pub position: Vec2,
    pub building_type: BuildingType,
}

/// Minimap marker for entities
#[derive(Component, Debug)]
pub struct MinimapMarker {
    pub color: [u8; 4],
    pub shape: MinimapShape,
}

/// Minimap marker shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinimapShape {
    Circle,
    Square,
    Triangle,
    Diamond,
}

/// Tags for selectable entities
#[derive(Component)]
pub struct Selectable;

/// Tag for currently selected entities
#[derive(Component)]
pub struct Selected;

/// Health bar component
#[derive(Component)]
pub struct HealthBar;

/// Component to track entities in a control group
#[derive(Component)]
pub struct ControlGroup(pub u8);

/// Order timer for units (cooldown between accepting new orders)
#[derive(Component)]
pub struct OrderTimer {
    pub timer: f32,
}

/// Animation component
#[derive(Component)]
pub struct Animation {
    pub current_frame: usize,
    pub frames: Vec<usize>,
    pub frame_time: f32,
    pub timer: f32,
    pub is_looping: bool,
}

/// Construction site marker
#[derive(Component)]
pub struct ConstructionSite {
    pub building_type: BuildingType,
    pub progress: f32,
}

/// Research status component
#[derive(Component)]
pub struct ResearchStatus {
    pub tech_type: crate::ecs::resources::TechType,
    pub progress: f32,
    pub total_time: f32,
}

/// Component for fog of war visibility
#[derive(Component)]
pub struct FogOfWarVisible {
    pub last_seen_tick: u64,
    pub visible_to_players: Vec<u8>,
}