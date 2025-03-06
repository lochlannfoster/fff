use bevy_ecs::prelude::*;
use glam::Vec2;
use rand::{Rng, thread_rng};

use crate::ecs::components::{
    Unit, UnitType, Transform, Owner, 
    Movement, AttackTarget, Collider
};
use crate::ecs::resources::GameTime;

/// Auto-combat unit behavior system for autobattler
pub fn autobattler_unit_behavior_system(
    mut commands: Commands,
    mut query: Query<(
        Entity, 
        &Unit, 
        &Transform, 
        &Owner,
        Option<&mut Movement>,
        Option<&AttackTarget>
    )>,
    enemy_query: Query<(Entity, &Transform, &Owner, &Unit, &Collider)>,
    time: Res<GameTime>,
) {
    let mut rng = thread_rng();

    for (entity, unit, transform, owner, mut movement, attack_target) in query.iter_mut() {
        // Skip units that are already attacking
        if attack_target.is_some() {
            continue;
        }

        // Find nearest enemy
        let mut nearest_enemy: Option<(Entity, Vec2, f32)> = None;
        
        for (enemy_entity, enemy_transform, enemy_owner, _, enemy_collider) in enemy_query.iter() {
            // Skip units from same team
            if enemy_owner.0 == owner.0 {
                continue;
            }

            let distance = (enemy_transform.position - transform.position).length();
            
            // Check if enemy is within sight range and closer than current nearest
            if distance <= unit.sight_range && 
               (nearest_enemy.is_none() || distance < nearest_enemy.unwrap().2) {
                nearest_enemy = Some((enemy_entity, enemy_transform.position, distance));
            }
        }

        // Auto-targeting based on unit type
        if let Some((enemy_entity, enemy_pos, _)) = nearest_enemy {
            // Automatically set attack target
            commands.entity(entity).insert(AttackTarget {
                target_entity: enemy_entity,
            });

            // Move towards target if not in attack range
            if let Some(mut movement) = movement {
                let distance_to_target = (enemy_pos - transform.position).length();
                
                // If not in attack range, move closer
                if distance_to_target > unit.attack_range {
                    // Create simple path to enemy
                    movement.path = vec![enemy_pos];
                    movement.path_index = 0;
                }
            }
        }
    }
}

/// Advanced targeting strategy for different unit types
fn advanced_targeting(
    unit: &Unit,
    transform: &Transform,
    unit_type: UnitType,
    enemies: &[(Entity, Vec2, f32, UnitType)],
) -> Option<Entity> {
    match unit_type {
        UnitType::Soldier => {
            // Prioritize weak targets first
            enemies.iter()
                .min_by_key(|&(_, _, _, enemy_type)| match enemy_type {
                    UnitType::Worker => 0,
                    UnitType::Scout => 1,
                    UnitType::Healer => 2,
                    UnitType::Soldier => 3,
                    UnitType::Tank => 4,
                })
                .map(|&(entity, _, _, _)| entity)
        }
        UnitType::Tank => {
            // Tanks target units that can damage them least
            enemies.iter()
                .min_by_key(|&(_, _, _, enemy_type)| match enemy_type {
                    UnitType::Healer => 0,
                    UnitType::Worker => 1,
                    UnitType::Scout => 2,
                    UnitType::Soldier => 3,
                    UnitType::Tank => 4,
                })
                .map(|&(entity, _, _, _)| entity)
        }
        UnitType::Scout => {
            // Scouts prioritize soft targets
            enemies.iter()
                .min_by_key(|&(_, _, _, enemy_type)| match enemy_type {
                    UnitType::Worker => 0,
                    UnitType::Healer => 1,
                    UnitType::Scout => 2,
                    UnitType::Soldier => 3,
                    UnitType::Tank => 4,
                })
                .map(|&(entity, _, _, _)| entity)
        }
        _ => None
    }
}