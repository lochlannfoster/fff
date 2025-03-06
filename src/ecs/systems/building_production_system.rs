// src/ecs/systems/building_production_system.rs

use bevy_ecs::prelude::*;
use std::collections::VecDeque;

use crate::ecs::components::{Building, BuildingType, UnitType, Transform, Owner};
use crate::ecs::resources::{GameTime, PlayerResources, TechState};
use crate::game::{buildings::BuildingData, units::spawn_unit};

pub fn building_production_system(
    mut commands: Commands,
    time: Res<GameTime>,
    mut query: Query<(Entity, &mut Building, &Transform, &Owner)>,
    mut player_resources: ResMut<PlayerResources>,
    tech_state: Res<TechState>,
) {
    for (entity, mut building, transform, owner) in query.iter_mut() {
        // Skip buildings that are still under construction
        if let Some(construction_progress) = &mut building.construction_progress {
            // Update construction progress
            *construction_progress += time.delta_time / BuildingData::get(building.building_type).build_time;
            
            if *construction_progress >= 1.0 {
                // Construction complete
                building.construction_progress = None;
            }
            
            continue;
        }
        
        // Process production queue
        if let Some(progress) = &mut building.production_progress {
            // Building is producing something
            if let Some(&unit_type) = building.production_queue.front() {
                // Calculate training time
                let base_train_time = match unit_type {
                    UnitType::Worker => 15.0,
                    UnitType::Soldier => 25.0,
                    UnitType::Scout => 20.0,
                    UnitType::Tank => 40.0,
                    UnitType::Healer => 30.0,
                };
                
                // Update progress
                *progress += time.delta_time / base_train_time;
                
                // Check if production is complete
                if *progress >= 1.0 {
                    // Production complete, spawn the unit
                    if let Some(unit_type) = building.production_queue.pop_front() {
                        // Calculate spawn position
                        let spawn_offset = Vec2::new(
                            building.rally_point.map_or(0.0, |p| p.x - transform.position.x),
                            building.rally_point.map_or(0.0, |p| p.y - transform.position.y),
                        ).normalize_or_zero() * 15.0;
                        
                        let spawn_pos = transform.position + spawn_offset;
                        
                        // Spawn the unit
                        spawn_unit(
                            &mut commands,
                            crate::game::units::UnitSpawnParams {
                                unit_type,
                                owner: owner.0,
                                position: spawn_pos,
                            },
                            &tech_state,
                        );
                    }
                    
                    // Check if there's more in the queue
                    if building.production_queue.is_empty() {
                        building.production_progress = None;
                    } else {
                        // Start next unit production
                        *progress = 0.0;
                    }
                }
            }
        } else if !building.production_queue.is_empty() {
            // Start producing the first unit in the queue
            building.production_progress = Some(0.0);
        }
    }
}