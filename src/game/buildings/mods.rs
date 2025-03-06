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
            texture_name: "building_hq".to_string(),
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
            texture_name: "building_barracks".to_string(),
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
            texture_name: "building_factory".to_string(),
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
            texture_name: "building_resource".to_string(),
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
            texture_name: "building_research".to_string(),
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
            texture_name: "building_defense".to_string(),
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