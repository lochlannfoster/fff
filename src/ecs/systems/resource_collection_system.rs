use bevy_ecs::prelude::*;
use glam::Vec2;

use crate::ecs::components::{Transform, Unit, UnitType, Owner, Resource, ResourceType, HarvestTarget};
use crate::ecs::resources::{GameTime, PlayerResources, TechState};
use crate::game::tech::{apply_tech_effect, TechEffectType};

pub fn resource_collection_system(
    mut commands: Commands,
    time: Res<GameTime>,
    mut query: Query<(Entity, &Unit, &Transform, &Owner, Option<&HarvestTarget>)>,
    mut resource_query: Query<(Entity, &mut Resource, &Transform)>,
    mut player_resources: ResMut<PlayerResources>,
    tech_state: Res<TechState>,
) {
    // For each worker unit
    for (entity, unit, transform, owner, harvest_target) in query.iter() {
        // Skip non-worker units
        if unit.unit_type != UnitType::Worker {
            continue;
        }
        
        // If unit is not currently harvesting, find a resource
        if harvest_target.is_none() {
            // Auto-assign nearest resource if close enough
            let mut nearest_resource = None;
            let mut nearest_distance = f32::MAX;
            
            for (resource_entity, resource, resource_transform) in resource_query.iter() {
                let distance = (resource_transform.position - transform.position).length();
                
                if distance < 200.0 && distance < nearest_distance {
                    nearest_resource = Some((resource_entity, resource.resource_type));
                    nearest_distance = distance;
                }
            }
            
            // Assign target if found
            if let Some((resource_entity, _)) = nearest_resource {
                if nearest_distance < 10.0 {
                    commands.entity(entity).insert(HarvestTarget {
                        target_entity: resource_entity,
                    });
                }
            }
            
            continue;
        }
        
        // Process harvesting for units with targets
        if let Some(harvest_target) = harvest_target {
            // Check if resource still exists
            if let Ok((resource_entity, mut resource, resource_transform)) = resource_query.get_mut(harvest_target.target_entity) {
                let distance = (resource_transform.position - transform.position).length();
                
                // Check if in range to harvest
                if distance <= 10.0 {
                    // Calculate harvest amount (per second)
                    let base_harvest_rate = 10.0; // Units per second
                    
                    // Apply tech effects to harvest rate
                    let harvest_rate = apply_tech_effect(
                        &tech_state,
                        owner.0,
                        base_harvest_rate,
                        TechEffectType::ResourceGathering
                    );
                    
                    // Calculate amount harvested this frame
                    let amount = harvest_rate * time.delta_time;
                    
                    // Don't harvest more than what's available
                    let actual_amount = amount.min(resource.amount);
                    
                    if actual_amount > 0.0 {
                        // Reduce resource amount
                        resource.amount -= actual_amount;
                        
                        // Add to player resources
                        let key = (owner.0, resource.resource_type);
                        if let Some(current) = player_resources.resources.get_mut(&key) {
                            *current += actual_amount;
                        } else {
                            player_resources.resources.insert(key, actual_amount);
                        }
                        
                        // Update income rate
                        *player_resources.income_rate.entry(key).or_insert(0.0) = harvest_rate;
                        
                        // Remove resource if depleted
                        if resource.amount <= 0.0 {
                            commands.entity(resource_entity).despawn();
                            commands.entity(entity).remove::<HarvestTarget>();
                        }
                    }
                }
            } else {
                // Resource no longer exists, remove target
                commands.entity(entity).remove::<HarvestTarget>();
            }
        }
    }
}