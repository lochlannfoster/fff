use std::collections::HashMap;

use crate::ecs::resources::TechState;
use crate::ecs::components::ResourceType;

/// Technology data structure containing properties for each technology
pub struct TechData {
    pub tech_type: crate::ecs::resources::TechType,
    pub name: String,
    pub description: String,
    pub research_time: f32,
    pub costs: HashMap<ResourceType, f32>,
    pub icon_name: String,
    pub prerequisites: Vec<crate::ecs::resources::TechType>,
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

/// Apply technology effects to game stats
pub fn apply_tech_effect(
    tech_state: &TechState,
    player_id: u8,
    base_value: f32,
    effect_type: TechEffectType,
) -> f32 {
    let mut value = base_value;
    
    // Get all researched techs for this player
    let researched_techs: Vec<crate::ecs::resources::TechType> = tech_state.researched.iter()
        .filter_map(|(&(pid, tech_type), &researched)| {
            if pid == player_id && researched {
                Some(tech_type)
            } else {
                None
            }
        })
        .collect();
    
    // Apply effects from all researched techs
    for &tech_type in &researched_techs {
        // In a real implementation, we would get the TechData and apply the effects
        // This is a simplified version that just applies some default multipliers
        match effect_type {
            TechEffectType::UnitDamage => value *= 1.1,
            TechEffectType::UnitHealth => value *= 1.1,
            TechEffectType::UnitSpeed => value *= 1.05,
            TechEffectType::ResourceGathering => value *= 1.2,
            _ => {}
        }
    }
    
    value
}