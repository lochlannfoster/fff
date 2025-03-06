use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::ecs::components::{Owner, Building, BuildingType, ResourceType};
use crate::ecs::resources::{GameTime, PlayerResources};

pub fn economy_system(
    time: Res<GameTime>,
    mut player_resources: ResMut<PlayerResources>,
    building_query: Query<(&Building, &Owner)>,
) {
    // Track current income/expense rates
    let mut income_rates: HashMap<(u8, ResourceType), f32> = HashMap::new();
    
    // Apply base income and compute building maintenance costs
    for (building, owner) in building_query.iter() {
        // Skip buildings under construction
        if building.construction_progress.is_some() {
            continue;
        }
        
        // Apply different effects based on building type
        match building.building_type {
            BuildingType::ResourceCollector => {
                // Resource collectors boost mineral income
                let key = (owner.0, ResourceType::Mineral);
                *income_rates.entry(key).or_insert(0.0) += 5.0; // 5 minerals per second
            },
            BuildingType::Headquarters => {
                // Headquarters generate a small amount of energy
                let key = (owner.0, ResourceType::Energy);
                *income_rates.entry(key).or_insert(0.0) += 0.5; // 0.5 energy per second
            },
            _ => {
                // Other buildings have maintenance costs
                let key = (owner.0, ResourceType::Energy);
                *income_rates.entry(key).or_insert(0.0) -= 0.1; // -0.1 energy per second
            }
        }
    }
    
    // Update resource income rates
    for (key, rate) in income_rates {
        player_resources.income_rate.insert(key, rate);
    }
    
    // Apply income/expense rates to resources
    for (&key, &rate) in player_resources.income_rate.iter() {
        let amount = rate * time.delta_time;
        if let Some(current) = player_resources.resources.get_mut(&key) {
            *current += amount;
            
            // Ensure resources don't go negative (except energy)
            if key.1 != ResourceType::Energy && *current < 0.0 {
                *current = 0.0;
            }
        } else {
            player_resources.resources.insert(key, amount);
        }
    }
}