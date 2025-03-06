use std::collections::HashMap;

use crate::ecs::resources::TechType;
use crate::ecs::components::ResourceType;

/// Technology data structure containing properties for each technology
pub struct TechData {
    pub tech_type: TechType,
    pub name: String,
    pub description: String,
    pub research_time: f32,
    pub costs: HashMap<ResourceType, f32>,
    pub icon_name: String,
    pub prerequisites: Vec<TechType>,
    pub effects: Vec<TechEffect>,
}

/// Effect of researching a technology
#[derive(Debug, Clone)]
pub enum TechEffect {
    UnitDamageMultiplier(f32),          // +X% to unit damage
    UnitHealthMultiplier(f32),          // +X% to unit health
    UnitSpeedMultiplier(f32),           // +X% to unit movement speed
    UnitSightRangeMultiplier(f32),      // +X% to unit sight range
    UnitAttackRangeMultiplier(f32),     // +X% to unit attack range
    UnitAttackSpeedMultiplier(f32),     // +X% to unit attack speed
    BuildingHealthMultiplier(f32),      // +X% to building health
    ResourceGatheringMultiplier(f32),   // +X% to resource gathering speed
    ResourceYieldMultiplier(f32),       // +X% to resource amount gained
    UnlockUnit(crate::ecs::components::UnitType),         // Unlock new unit type
    UnlockBuilding(crate::ecs::components::BuildingType), // Unlock new building type
    ReducedBuildTime(f32),              // -X% to build time
    ReducedResearchTime(f32),           // -X% to research time
}

impl TechData {
    /// Get technology data for a specific tech type
    pub fn get(tech_type: TechType) -> Self {
        match tech_type {
            TechType::ImprovedHarvesting => Self::improved_harvesting(),
            TechType::ImprovedWeapons => Self::improved_weapons(),
            TechType::ImprovedArmor => Self::improved_armor(),
            TechType::AdvancedUnits => Self::advanced_units(),
            TechType::AdvancedBuildings => Self::advanced_buildings(),
            TechType::ImprovedHealing => Self::improved_healing(),
            TechType::ImprovedSpeed => Self::improved_speed(),
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
    
    /// Improved harvesting technology data
    pub fn improved_harvesting() -> Self {
        Self {
            tech_type: TechType::ImprovedHarvesting,
            name: "Improved Harvesting".to_string(),
            description: "Increases resource gathering speed by 20%.".to_string(),
            research_time: 60.0,
            costs: Self::create_costs(100.0, 100.0),
            icon_name: "tech_harvesting",
            prerequisites: vec![],
            effects: vec![
                TechEffect::ResourceGatheringMultiplier(1.2),
            ],
        }
    }
    
    /// Improved weapons technology data
    pub fn improved_weapons() -> Self {
        Self {
            tech_type: TechType::ImprovedWeapons,
            name: "Improved Weapons".to_string(),
            description: "Increases unit attack damage by 25%.".to_string(),
            research_time: 80.0,
            costs: Self::create_costs(150.0, 150.0),
            icon_name: "tech_weapons",
            prerequisites: vec![],
            effects: vec![
                TechEffect::UnitDamageMultiplier(1.25),
            ],
        }
    }
    
    /// Improved armor technology data
    pub fn improved_armor() -> Self {
        Self {
            tech_type: TechType::ImprovedArmor,
            name: "Improved Armor".to_string(),
            description: "Increases unit and building health by 20%.".to_string(),
            research_time: 70.0,
            costs: Self::create_costs(125.0, 125.0),
            icon_name: "tech_armor",
            prerequisites: vec![],
            effects: vec![
                TechEffect::UnitHealthMultiplier(1.2),
                TechEffect::BuildingHealthMultiplier(1.2),
            ],
        }
    }
    
    /// Advanced units technology data
    pub fn advanced_units() -> Self {
        Self {
            tech_type: TechType::AdvancedUnits,
            name: "Advanced Units".to_string(),
            description: "Unlocks advanced combat units.".to_string(),
            research_time: 120.0,
            costs: Self::create_costs(200.0, 200.0),
            icon_name: "tech_advanced_units",
            prerequisites: vec![TechType::ImprovedWeapons],
            effects: vec![
                TechEffect::UnlockUnit(crate::ecs::components::UnitType::Tank),
            ],
        }
    }
    
    /// Advanced buildings technology data
    pub fn advanced_buildings() -> Self {
        Self {
            tech_type: TechType::AdvancedBuildings,
            name: "Advanced Buildings".to_string(),
            description: "Unlocks advanced structures.".to_string(),
            research_time: 100.0,
            costs: Self::create_costs(150.0, 200.0),
            icon_name: "tech_advanced_buildings",
            prerequisites: vec![],
            effects: vec![
                TechEffect::UnlockBuilding(crate::ecs::components::BuildingType::DefenseTower),
                TechEffect::BuildingHealthMultiplier(1.15),
            ],
        }
    }
    
    /// Improved healing technology data
    pub fn improved_healing() -> Self {
        Self {
            tech_type: TechType::ImprovedHealing,
            name: "Improved Healing".to_string(),
            description: "Increases healing effectiveness by 30%.".to_string(),
            research_time: 60.0,
            costs: Self::create_costs(100.0, 150.0),
            icon_name: "tech_healing",
            prerequisites: vec![],
            effects: vec![
                TechEffect::UnitHealthMultiplier(1.1),
                // In a real implementation, there would be a specific healing multiplier
            ],
        }
    }
    
    /// Improved speed technology data
    pub fn improved_speed() -> Self {
        Self {
            tech_type: TechType::ImprovedSpeed,
            name: "Improved Speed".to_string(),
            description: "Increases unit movement speed by 15%.".to_string(),
            research_time: 70.0,
            costs: Self::create_costs(125.0, 100.0),
            icon_name: "tech_speed",
            prerequisites: vec![],
            effects: vec![
                TechEffect::UnitSpeedMultiplier(1.15),
                TechEffect::UnitAttackSpeedMultiplier(1.1),
            ],
        }
    }
}

/// Get all technologies in a dependency tree order
pub fn get_tech_tree() -> Vec<TechData> {
    let mut techs = Vec::new();
    let mut added = HashMap::new();
    
    // Helper function to add a tech and its prerequisites
    fn add_tech_with_prerequisites(
        tech_type: TechType,
        techs: &mut Vec<TechData>,
        added: &mut HashMap<TechType, bool>,
    ) {
        // Skip if already added
        if *added.entry(tech_type).or_insert(false) {
            return;
        }
        
        let tech_data = TechData::get(tech_type);
        
        // First add all prerequisites
        for prereq in &tech_data.prerequisites {
            add_tech_with_prerequisites(*prereq, techs, added);
        }
        
        // Then add this tech
        techs.push(tech_data);
        added.insert(tech_type, true);
    }
    
    // Add all techs in the correct order
    add_tech_with_prerequisites(TechType::ImprovedHarvesting, &mut techs, &mut added);
    add_tech_with_prerequisites(TechType::ImprovedWeapons, &mut techs, &mut added);
    add_tech_with_prerequisites(TechType::ImprovedArmor, &mut techs, &mut added);
    add_tech_with_prerequisites(TechType::AdvancedUnits, &mut techs, &mut added);
    add_tech_with_prerequisites(TechType::AdvancedBuildings, &mut techs, &mut added);
    add_tech_with_prerequisites(TechType::ImprovedHealing, &mut techs, &mut added);
    add_tech_with_prerequisites(TechType::ImprovedSpeed, &mut techs, &mut added);
    
    techs
}

/// Apply technology effects to game stats
pub fn apply_tech_effect(
    tech_state: &crate::ecs::resources::TechState,
    player_id: u8,
    base_value: f32,
    effect_type: TechEffectType,
) -> f32 {
    let mut value = base_value;
    
    // Get all researched techs for this player
    let researched_techs: Vec<TechType> = tech_state.researched.iter()
        .filter_map(|(&(pid, tech_type), &researched)| {
            if pid == player_id && researched {
                Some(tech_type)
            } else {
                None
            }
        })
        .collect();
    
    // Apply effects from all researched techs
    for tech_type in &researched_techs {
        let tech_data = TechData::get(*tech_type);
        
        for effect in &tech_data.effects {
            match effect {
                TechEffect::UnitDamageMultiplier(multiplier) if effect_type == TechEffectType::UnitDamage => {
                    value *= multiplier;
                }
                TechEffect::UnitHealthMultiplier(multiplier) if effect_type == TechEffectType::UnitHealth => {
                    value *= multiplier;
                }
                TechEffect::BuildingHealthMultiplier(multiplier) if effect_type == TechEffectType::BuildingHealth => {
                    value *= multiplier;
                }
                TechEffect::UnitSpeedMultiplier(multiplier) if effect_type == TechEffectType::UnitSpeed => {
                    value *= multiplier;
                }
                TechEffect::ResourceGatheringMultiplier(multiplier) if effect_type == TechEffectType::ResourceGathering => {
                    value *= multiplier;
                }
                TechEffect::UnitAttackSpeedMultiplier(multiplier) if effect_type == TechEffectType::UnitAttackSpeed => {
                    value *= multiplier;
                }
                TechEffect::UnitAttackRangeMultiplier(multiplier) if effect_type == TechEffectType::UnitAttackRange => {
                    value *= multiplier;
                }
                TechEffect::UnitSightRangeMultiplier(multiplier) if effect_type == TechEffectType::UnitSightRange => {
                    value *= multiplier;
                }
                TechEffect::ResourceYieldMultiplier(multiplier) if effect_type == TechEffectType::ResourceYield => {
                    value *= multiplier;
                }
                TechEffect::ReducedBuildTime(multiplier) if effect_type == TechEffectType::BuildTime => {
                    value /= multiplier; // Reduce time by dividing
                }
                TechEffect::ReducedResearchTime(multiplier) if effect_type == TechEffectType::ResearchTime => {
                    value /= multiplier; // Reduce time by dividing
                }
                _ => {}
            }
        }
    }
    
    value
}

/// Types of tech effects for querying
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechEffectType {
    UnitDamage,
    UnitHealth,
    UnitSpeed,
    UnitAttackSpeed,
    UnitAttackRange,
    UnitSightRange,
    BuildingHealth,
    ResourceGathering,
    ResourceYield,
    BuildTime,
    ResearchTime,
}

/// Check if a technology is available to research
pub fn is_tech_available(
    tech_type: TechType,
    tech_state: &crate::ecs::resources::TechState,
    player_id: u8,
) -> bool {
    // Check if already researched
    if *tech_state.researched.get(&(player_id, tech_type)).unwrap_or(&false) {
        return false;
    }
    
    // Check if already in progress
    if tech_state.in_progress.contains_key(&(player_id, tech_type)) {
        return false;
    }
    
    // Check prerequisites
    let tech_data = TechData::get(tech_type);
    
    for prereq in &tech_data.prerequisites {
        if !tech_state.researched.get(&(player_id, *prereq)).unwrap_or(&false) {
            return false;
        }
    }
    
    true
}