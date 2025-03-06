use bevy_ecs::prelude::*;
use glam::Vec2;
use std::collections::{HashMap, VecDeque};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::ecs::components::{UnitType, BuildingType, Transform, Owner, Unit, Building, AttackTarget};
use crate::ecs::resources::{GameMap, PlayerResources, GameTime};
use crate::ecs::combat::components::{DamageTable, AttackCooldown, Projectile, Effect, EffectType};

/// System to process attacks and combat
pub fn combat_system(
    mut commands: Commands,
    time: Res<GameTime>,
    damage_table: Res<DamageTable>,
    mut unit_query: Query<(
        Entity,
        &mut Unit,
        &Transform,
        &Owner,
        Option<&AttackTarget>,
        Option<&mut AttackCooldown>,
    )>,
    mut building_query: Query<(
        Entity,
        &mut Building,
        &Transform,
        &Owner,
        Option<&AttackTarget>,
        Option<&mut AttackCooldown>,
    )>,
    transform_query: Query<&Transform>,
    mut projectile_query: Query<(Entity, &mut Projectile, &mut Transform)>,
    mut effect_query: Query<(Entity, &mut Effect, &mut Transform)>,
    mut rng: Local<Option<StdRng>>,
) {
    // Initialize RNG if needed
    if rng.is_none() {
        *rng = Some(StdRng::seed_from_u64(time.current_tick));
    }
    let rng = rng.as_mut().unwrap();
    
    // Update projectiles
    for (entity, mut projectile, mut transform) in projectile_query.iter_mut() {
        // Skip if target no longer exists
        if !transform_query.contains(projectile.target_entity) {
            commands.entity(entity).despawn();
            continue;
        }
        
        // Get target position
        let target_transform = transform_query.get(projectile.target_entity).unwrap();
        let target_position = target_transform.position;
        
        // Calculate direction to target
        let direction = (target_position - transform.position).normalize_or_zero();
        
        // Move projectile
        let distance_to_move = projectile.speed * time.delta_time;
        transform.position += direction * distance_to_move;
        
        // Update rotation to face direction
        if direction != Vec2::ZERO {
            transform.rotation = direction.y.atan2(direction.x);
        }
        
        // Update traveled distance
        projectile.traveled_distance += distance_to_move;
        
        // Check if projectile has reached target or max distance
        let distance_to_target = (target_position - transform.position).length();
        if distance_to_target < 5.0 || projectile.traveled_distance >= projectile.max_distance {
            // Despawn projectile
            commands.entity(entity).despawn();
        }
    }
    
    // Update effects
    for (entity, mut effect, mut transform) in effect_query.iter_mut() {
        // Update effect timer
        effect.elapsed += time.delta_time;
        
        // Check if effect has expired
        if effect.elapsed >= effect.duration {
            commands.entity(entity).despawn();
            continue;
        }
        
        // Update effect based on type
        match effect.effect_type {
            EffectType::Explosion => {
                // Scale up and fade out explosion
                let progress = effect.elapsed / effect.duration;
                transform.scale = Vec2::splat(effect.scale * (1.0 + progress * 2.0));
            }
            EffectType::Fire => {
                // Flicker fire
                let flicker = 1.0 + 0.2 * (time.current_tick as f32 * 10.0).sin();
                transform.scale = Vec2::splat(effect.scale * flicker);
            }
            _ => {}
        }
    }
}