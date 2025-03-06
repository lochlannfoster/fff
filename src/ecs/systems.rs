use bevy_ecs::prelude::*;
use glam::Vec2;

use crate::ecs::components::*;
use crate::ecs::resources::*;

/// System to update entity positions based on movement components
pub fn update_movement_system(
    mut query: Query<(&mut Transform, &mut Movement)>,
    time: Res<GameTime>,
) {
    for (mut transform, mut movement) in query.iter_mut() {
        // Skip if no path or at destination
        if movement.path.is_empty() || movement.path_index >= movement.path.len() {
            movement.velocity = Vec2::ZERO;
            continue;
        }
        
        // Get current target position from path
        let target_pos = movement.path[movement.path_index];
        let current_pos = transform.position;
        
        // Calculate direction to target
        let to_target = target_pos - current_pos;
        let distance = to_target.length();
        
        // Check if we've reached the current waypoint
        if distance < 5.0 {
            // Move to next waypoint
            movement.path_index += 1;
            
            // If we've reached the end of the path
            if movement.path_index >= movement.path.len() {
                movement.velocity = Vec2::ZERO;
                continue;
            }
        }
        
        // Otherwise, move toward the target
        if distance > 0.1 {
            let direction = to_target.normalize();
            let speed = 100.0; // Units per second
            movement.velocity = direction * speed;
            
            // Update position
            transform.position += movement.velocity * time.delta_time;
            
            // Update rotation to face movement direction
            transform.rotation = direction.y.atan2(direction.x);
        }
    }
}

/// System to handle collision detection and resolution
pub fn collision_detection_system(
    mut query: Query<(Entity, &Transform, &Collider, Option<&mut Movement>)>,
) {
    // Collect all entities with colliders
    let entities: Vec<(Entity, Transform, Collider, bool)> = query
        .iter()
        .map(|(entity, transform, collider, movement)| 
            (entity, *transform, collider.clone(), movement.is_some()))
        .collect();
    
    // Check for collisions between all pairs
    for i in 0..entities.len() {
        for j in (i+1)..entities.len() {
            let (entity_a, transform_a, collider_a, has_movement_a) = &entities[i];
            let (entity_b, transform_b, collider_b, has_movement_b) = &entities[j];
            
            // Skip if entities are not set to collide with each other
            if (collider_a.collision_layer & collider_b.collision_mask == 0) &&
               (collider_b.collision_layer & collider_a.collision_mask == 0) {
                continue;
            }
            
            // Calculate distance between entities
            let distance = (transform_a.position - transform_b.position).length();
            let min_distance = collider_a.radius + collider_b.radius;
            
            // Check for collision
            if distance < min_distance {
                // Handle collision for entities with movement components
                if *has_movement_a || *has_movement_b {
                    // Get the entities again but with mutable references
                    if let Ok([(_, _, _, Some(mut movement_a)), (_, _, _, Some(mut movement_b))]) = 
                        query.get_many_mut([*entity_a, *entity_b]) {
                        
                        // Simple collision resolution - stop movement
                        if *has_movement_a {
                            movement_a.velocity = Vec2::ZERO;
                        }
                        
                        if *has_movement_b {
                            movement_b.velocity = Vec2::ZERO;
                        }
                    }
                }
                
                // Additional collision effects could be implemented here
                // (damage, knockback, etc.)
            }
        }
    }
}

/// System to handle unit AI and behavior
pub fn unit_behavior_system(
    mut query: Query<(Entity, &Unit, &Transform, &Owner, Option<&AttackTarget>, Option<&mut Movement>)>,
    transform_query: Query<&Transform>,
    time: Res<GameTime>,
) {
    for (entity, unit, transform, owner, attack_target, movement) in query.iter_mut() {
        // Handle attack behavior if unit has a target
        if let Some(attack_target) = attack_target {
            if let Ok(target_transform) = transform_query.get(attack_target.target_entity) {
                // Check if target is in range
                let distance = (transform.position - target_transform.position).length();
                
                if distance <= unit.attack_range {
                    // We're in range to attack - combat system will handle the actual attack
                    
                    // If we have movement, stop moving when in attack range
                    if let Some(mut movement) = movement {
                        movement.velocity = Vec2::ZERO;
                        movement.path.clear();
                    }
                } else {
                    // Target not in range, move toward it if we can
                    if let Some(mut movement) = movement {
                        // If we don't have a path or our target moved significantly
                        if movement.path.is_empty() || 
                           (movement.path.last().is_some() && 
                            (movement.path.last().unwrap() - target_transform.position).length_squared() > 100.0) {
                            
                            // Request a new path to the target
                            // In a real implementation, this would call the pathfinding system
                            movement.path = vec![target_transform.position];
                            movement.path_index = 0;
                        }
                    }
                }
            }
        }
    }
}