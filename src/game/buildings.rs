use std::collections::HashMap;
use glam::Vec2;

use crate::ecs::components::{BuildingType, ResourceType, UnitType};

/// Building data structure containing properties for each building type
pub struct BuildingData {
    pub building_type: BuildingType,
    pub name: String,
    pub description: String,
    pub health: f32,
    pub size: Vec2,
    pub build_time: f32,
    pub costs: HashMap<ResourceType, f32>,
    pub texture_name: String,
    pub can_produce: Vec<UnitType>,
    pub provides_supply: u32,
    pub tech_requirements: Vec<crate::ecs::resources::TechType>,
    pub sight_range: f32,
    pub attack_damage: Option<f32>,
    pub attack_range: Option<f32>,
    pub attack_speed: Option<f32>,
}

impl BuildingData {
    /// Get building data for a specific building type
    pub fn get(building_type: BuildingType) -> Self {
        match building_type {
            BuildingType::Headquarters => Self::headquarters(),
            BuildingType::Barracks => Self::barracks(),
            BuildingType::Factory => Self::factory(),
            BuildingType::ResourceCollector => Self::resource_collector(),
            BuildingType::ResearchCenter => Self::research_center(),
            BuildingType::DefenseTower => Self::defense_tower(),
        }
    }
    
    /// Helper to create cost map
    fn create_costs(mineral: f32, gas: f32) -> HashMap<ResourceType, f32> {
        let mut costs = HashMap::new();
        if mineral > 0.0 {
            costs.insert(ResourceType::Mineral, mineral);
        }
        if gas > 0.0 {
            costs.insert(ResourceType::Gas, gas);
        }
        costs
    }
    
    /// Headquarters building data
    pub fn headquarters() -> Self {
        Self {
            building_type: BuildingType::Headquarters,
            name: "Command Center".to_string(),
            description: "Main base building that produces workers and provides supply.".to_string(),
            health: 1500.0,
            size: Vec2::new(4.0, 4.0),
            build_time: 120.0,
            costs: Self::create_costs(400.0, 0.0),
            texture_name: "building_hq",
            can_produce: vec![UnitType::Worker],
            provides_supply: 10,
            tech_requirements: vec![],
            sight_range: 100.0,
            attack_damage: None,
            attack_range: None,
            attack_speed: None,
        }
    }
    
    /// Barracks building data
    pub fn barracks() -> Self {
        Self {
            building_type: BuildingType::Barracks,
            name: "Barracks".to_string(),
            description: "Training facility for infantry units.".to_string(),
            health: 1000.0,
            size: Vec2::new(3.0, 3.0),
            build_time: 60.0,
            costs: Self::create_costs(150.0, 0.0),
            texture_name: "building_barracks",
            can_produce: vec![UnitType::Soldier, UnitType::Scout],
            provides_supply: 0,
            tech_requirements: vec![],
            sight_range: 80.0,
            attack_damage: None,
            attack_range: None,
            attack_speed: None,
        }
    }
    
    /// Factory building data
    pub fn factory() -> Self {
        Self {
            building_type: BuildingType::Factory,
            name: "Factory".to_string(),
            description: "Production facility for advanced combat units.".to_string(),
            health: 1200.0,
            size: Vec2::new(3.0, 3.0),
            build_time: 90.0,
            costs: Self::create_costs(200.0, 100.0),
            texture_name: "building_factory",
            can_produce: vec![UnitType::Tank],
            provides_supply: 0,
            tech_requirements: vec![crate::ecs::resources::TechType::AdvancedUnits],
            sight_range: 80.0,
            attack_damage: None,
            attack_range: None,
            attack_speed: None,
        }
    }
    
    /// Resource collector building data
    pub fn resource_collector() -> Self {
        Self {
            building_type: BuildingType::ResourceCollector,
            name: "Resource Collector".to_string(),
            description: "Increases resource gathering efficiency nearby.".to_string(),
            health: 800.0,
            size: Vec2::new(2.0, 2.0),
            build_time: 45.0,
            costs: Self::create_costs(100.0, 50.0),
            texture_name: "building_resource",
            can_produce: vec![],
            provides_supply: 0,
            tech_requirements: vec![],
            sight_range: 60.0,
            attack_damage: None,
            attack_range: None,
            attack_speed: None,
        }
    }
    
    /// Research center building data
    pub fn research_center() -> Self {
        Self {
            building_type: BuildingType::ResearchCenter,
            name: "Research Center".to_string(),
            description: "Allows research of new technologies.".to_string(),
            health: 850.0,
            size: Vec2::new(3.0, 3.0),
            build_time: 60.0,
            costs: Self::create_costs(150.0, 100.0),
            texture_name: "building_research",
            can_produce: vec![UnitType::Healer],
            provides_supply: 0,
            tech_requirements: vec![],
            sight_range: 80.0,
            attack_damage: None,
            attack_range: None,
            attack_speed: None,
        }
    }
    
    /// Defense tower building data
    pub fn defense_tower() -> Self {
        Self {
            building_type: BuildingType::DefenseTower,
            name: "Defense Tower".to_string(),
            description: "Static defense structure that attacks nearby enemies.".to_string(),
            health: 500.0,
            size: Vec2::new(2.0, 2.0),
            build_time: 30.0,
            costs: Self::create_costs(75.0, 25.0),
            texture_name: "building_defense",
            can_produce: vec![],
            provides_supply: 0,
            tech_requirements: vec![],
            sight_range: 150.0,
            attack_damage: Some(15.0),
            attack_range: Some(100.0),
            attack_speed: Some(1.0),
        }
    }
}

/// Check if a building location is valid
pub fn is_valid_build_location(
    building_type: BuildingType,
    position: Vec2,
    game_map: &crate::ecs::resources::GameMap,
    existing_buildings: &[(Vec2, Vec2)], // Positions and sizes of existing buildings
) -> bool {
    // Get building data to know the size
    let building_data = BuildingData::get(building_type);
    let half_size = building_data.size * 0.5;
    
    // Check map bounds
    let min_x = position.x - half_size.x;
    let min_y = position.y - half_size.y;
    let max_x = position.x + half_size.x;
    let max_y = position.y + half_size.y;
    
    if min_x < 0.0 || min_y < 0.0 || 
       max_x >= game_map.width as f32 || 
       max_y >= game_map.height as f32 {
        return false;
    }
    
    // Check terrain at building corners
    let corners = [
        Vec2::new(min_x, min_y),
        Vec2::new(max_x, min_y),
        Vec2::new(min_x, max_y),
        Vec2::new(max_x, max_y),
        Vec2::new(position.x, position.y), // Center
    ];
    
    for corner in &corners {
        let x = corner.x as usize;
        let y = corner.y as usize;
        let idx = y * game_map.width as usize + x;
        
        // Check if terrain is buildable
        if idx < game_map.terrain_tiles.len() {
            match game_map.terrain_tiles[idx] {
                crate::ecs::resources::TerrainTile::Water | 
                crate::ecs::resources::TerrainTile::Mountain => {
                    return false;
                }
                _ => {}
            }
        } else {
            return false; // Out of bounds
        }
    }
    
    // Check collision with existing buildings
    for (other_pos, other_size) in existing_buildings {
        let other_half_size = other_size * 0.5;
        
        // Simple AABB collision check
        if !(max_x < other_pos.x - other_half_size.x ||
             min_x > other_pos.x + other_half_size.x ||
             max_y < other_pos.y - other_half_size.y ||
             min_y > other_pos.y + other_half_size.y) {
            return false;
        }
    }
    
    // Check for resources nearby if it's a resource collector
    if building_type == BuildingType::ResourceCollector {
        let mut resource_nearby = false;
        let collection_radius = 100.0;
        
        for (res_pos, _, _) in &game_map.resource_positions {
            if (position - *res_pos).length() < collection_radius {
                resource_nearby = true;
                break;
            }
        }
        
        if !resource_nearby {
            return false;
        }
    }
    
    true
}

/// Find a valid build location near the target position
pub fn find_valid_build_location(
    building_type: BuildingType,
    target_position: Vec2,
    game_map: &crate::ecs::resources::GameMap,
    existing_buildings: &[(Vec2, Vec2)],
) -> Option<Vec2> {
    // First check if target position is valid
    if is_valid_build_location(building_type, target_position, game_map, existing_buildings) {
        return Some(target_position);
    }
    
    // Search in expanding rings
    let building_data = BuildingData::get(building_type);
    let radius_step = building_data.size.x.max(building_data.size.y);
    
    for radius in 1..10 {
        // Try positions in a ring around the target
        let search_radius = radius as f32 * radius_step;
        let num_points = (search_radius * std::f32::consts::PI * 2.0 / radius_step) as usize;
        
        for i in 0..num_points {
            let angle = i as f32 * std::f32::consts::PI * 2.0 / num_points as f32;
            let offset = Vec2::new(angle.cos(), angle.sin()) * search_radius;
            let pos = target_position + offset;
            
            if is_valid_build_location(building_type, pos, game_map, existing_buildings) {
                return Some(pos);
            }
        }
    }
    
    None // No valid location found
}