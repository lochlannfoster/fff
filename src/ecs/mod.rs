pub mod components;
pub mod resources;
pub mod systems;
pub mod combat;
pub mod init;

use bevy_ecs::prelude::*;
use glam::Vec2;

use crate::ecs::components::*;
use crate::ecs::resources::*;
use crate::game::pathfinding;

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
            }
        }
    }
}

/// System to handle unit production in buildings
pub fn building_production_system(
    mut commands: Commands,
    time: Res<GameTime>,
    mut query: Query<(Entity, &mut Building, &Transform, &Owner)>,
    game_state: Res<GameState>,
) {
    for (entity, mut building, transform, owner) in query.iter_mut() {
        // Skip buildings that are still under construction
        if building.construction_progress.is_some() {
            // Update construction progress
            let progress = building.construction_progress.as_mut().unwrap();
            *progress += time.delta_time * 0.1; // Adjust rate as needed
            
            if *progress >= 1.0 {
                // Construction complete
                building.construction_progress = None;
            }
            
            // Skip production logic if still under construction
            continue;
        }
        
        // Process building production queue
        if let Some(progress) = &mut building.production_progress {
            // Building is currently producing something
            *progress += time.delta_time * 0.1; // Adjust rate as needed
            
            if *progress >= 1.0 {
                // Production complete
                if let Some(unit_type) = building.production_queue.pop_front() {
                    // Spawn the produced unit
                    spawn_unit(&mut commands, unit_type, transform.position, owner.0);
                }
                
                // Check if there's another unit in the queue
                if let Some(next_unit) = building.production_queue.front() {
                    // Start producing the next unit
                    *progress = 0.0;
                } else {
                    // Nothing left in the queue
                    building.production_progress = None;
                }
            }
        } else if !building.production_queue.is_empty() {
            // Start producing the first unit in the queue
            building.production_progress = Some(0.0);
        }
    }
}

/// System to handle resource collection by worker units
pub fn resource_collection_system(
    mut commands: Commands,
    time: Res<GameTime>,
    mut query: Query<(Entity, &Unit, &mut Transform, &Owner, Option<&mut Movement>)>,
    mut resource_query: Query<(Entity, &mut Resource, &Transform)>,
    mut player_resources: ResMut<PlayerResources>,
) {
    // For each worker unit
    for (entity, unit, mut transform, owner, movement) in query.iter_mut() {
        // Skip non-worker units
        if unit.unit_type != UnitType::Worker {
            continue;
        }
        
        // Find nearest resource within gathering range
        let mut nearest_resource = None;
        let mut nearest_distance = f32::MAX;
        
        for (resource_entity, resource, resource_transform) in resource_query.iter() {
            let distance = (resource_transform.position - transform.position).length();
            
            // Check if within gathering range
            if distance < 50.0 && distance < nearest_distance {
                nearest_resource = Some((resource_entity, resource.resource_type.clone(), distance));
                nearest_distance = distance;
            }
        }
        
        // If a resource is found within range, gather it
        if let Some((resource_entity, resource_type, distance)) = nearest_resource {
            // If we're close enough, gather the resource
            if distance < 10.0 {
                // Update worker animation/state
                // ...
                
                // Add resources to player
                let gather_rate = 1.0; // Resources per second
                let amount = gather_rate * time.delta_time;
                
                let key = (owner.0, resource_type);
                if let Some(current) = player_resources.resources.get_mut(&key) {
                    *current += amount;
                } else {
                    player_resources.resources.insert(key, amount);
                }
                
                // Also update income rate
                *player_resources.income_rate.entry(key).or_insert(0.0) = gather_rate;
                
                // Deplete the resource
                if let Ok((_, mut resource, _)) = resource_query.get_mut(resource_entity) {
                    resource.amount -= amount;
                    
                    // Remove the resource if depleted
                    if resource.amount <= 0.0 {
                        commands.entity(resource_entity).despawn();
                    }
                }
            } else if let Some(mut movement) = movement {
                // Move toward the resource if not close enough
                if movement.path.is_empty() {
                    // Set path to resource
                    let resource_transform = resource_query.get(resource_entity).unwrap().2;
                    movement.path = vec![resource_transform.position];
                    movement.path_index = 0;
                }
            }
        }
    }
}

/// System to handle fog of war updates
pub fn fog_of_war_system(
    query: Query<(&Transform, &Unit, &Owner)>,
    building_query: Query<(&Transform, &Building, &Owner)>,
    mut game_map: ResMut<GameMap>,
) {
    // Clear existing visibility
    for visibility_set in game_map.fog_of_war.values_mut() {
        visibility_set.clear();
    }
    
    // Calculate visible tiles for each player's units
    for player_id in 0..8 {
        let mut unit_positions = Vec::new();
        
        // Add units
        for (transform, unit, owner) in query.iter() {
            if owner.0 == player_id {
                unit_positions.push((transform.position, unit.sight_range));
            }
        }
        
        // Add buildings
        for (transform, building, owner) in building_query.iter() {
            if owner.0 == player_id {
                // Different building types have different sight ranges
                let sight_range = match building.building_type {
                    BuildingType::Headquarters => 120.0,
                    BuildingType::DefenseTower => 150.0,
                    _ => 80.0,
                };
                
                unit_positions.push((transform.position, sight_range));
            }
        }
        
        // Calculate visible tiles
        let visible_tiles = pathfinding::calculate_visible_tiles(&game_map, &unit_positions, 8.0);
        
        // Update fog of war for this player
        game_map.fog_of_war.insert(player_id, visible_tiles);
    }
}

/// Helper function to spawn a new unit
fn spawn_unit(
    commands: &mut Commands,
    unit_type: UnitType,
    position: Vec2,
    owner: u8,
) {
    // Get unit stats based on type
    let (health, attack_damage, attack_range, attack_speed, movement_speed, sight_range) = match unit_type {
        UnitType::Worker => (
            30.0,   // Health
            3.0,    // Attack damage
            10.0,   // Attack range
            1.0,    // Attack speed
            80.0,   // Movement speed
            100.0,  // Sight range
        ),
        UnitType::Soldier => (
            60.0,   // Health
            10.0,   // Attack damage
            50.0,   // Attack range
            0.8,    // Attack speed
            60.0,   // Movement speed
            120.0,  // Sight range
        ),
        UnitType::Scout => (
            40.0,   // Health
            6.0,    // Attack damage
            40.0,   // Attack range
            0.5,    // Attack speed
            120.0,  // Movement speed
            150.0,  // Sight range
        ),
        UnitType::Tank => (
            120.0,  // Health
            30.0,   // Attack damage
            70.0,   // Attack range
            2.0,    // Attack speed
            40.0,   // Movement speed
            100.0,  // Sight range
        ),
        UnitType::Healer => (
            40.0,   // Health
            0.0,    // Attack damage
            60.0,   // Heal range
            1.0,    // Heal speed
            50.0,   // Movement speed
            120.0,  // Sight range
        ),
    };
    
    // Spawn unit entity with components
    commands.spawn((
        Unit {
            unit_type,
            health,
            max_health: health,
            attack_damage,
            attack_range,
            attack_speed,
            movement_speed,
            sight_range,
            buildable: unit_type == UnitType::Worker,
        },
        Transform {
            position,
            rotation: 0.0,
            scale: Vec2::splat(1.0),
        },
        Owner(owner),
        Movement {
            path: Vec::new(),
            path_index: 0,
            target: None,
            velocity: Vec2::ZERO,
        },
        Collider {
            radius: match unit_type {
                UnitType::Tank => 8.0,
                UnitType::Worker | UnitType::Healer => 4.0,
                _ => 5.0,
            },
            collision_layer: 1, // Unit layer
            collision_mask: 1 | 2, // Collide with units and buildings
        },
        MinimapMarker {
            color: match owner {
                0 => [0, 0, 255, 255],   // Blue
                1 => [255, 0, 0, 255],   // Red
                2 => [0, 255, 0, 255],   // Green
                3 => [255, 255, 0, 255], // Yellow
                _ => [255, 255, 255, 255], // White
            },
            shape: match unit_type {
                UnitType::Worker => MinimapShape::Circle,
                UnitType::Tank => MinimapShape::Square,
                _ => MinimapShape::Triangle,
            },
        },
        // Would also add a Sprite component in a real implementation
    ));
}

/// System to maintain unit behavior and AI
pub fn unit_behavior_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Unit,
        &Transform,
        &Owner,
        Option<&AttackTarget>,
        Option<&mut Movement>,
    )>,
    enemy_query: Query<(Entity, &Transform, &Owner), (With<Unit>, With<Building>)>,
    time: Res<GameTime>,
) {
    for (entity, unit, transform, owner, attack_target, movement) in query.iter_mut() {
        // Skip units that are already attacking
        if attack_target.is_some() {
            continue;
        }
        
        // Automatic target acquisition for combat units
        match unit.unit_type {
            UnitType::Worker => {
                // Workers only attack if directly threatened
                let threat_range = 30.0;
                if let Some((enemy_entity, _, _)) = find_closest_enemy(
                    transform.position,
                    threat_range,
                    owner.0,
                    &enemy_query,
                ) {
                    // Worker is threatened, attack in self-defense
                    commands.entity(entity).insert(AttackTarget {
                        target_entity: enemy_entity,
                    });
                }
            }
            UnitType::Soldier | UnitType::Scout | UnitType::Tank => {
                // Combat units actively seek targets within their sight range
                let acquisition_range = unit.sight_range * 0.8; // 80% of sight range
                if let Some((enemy_entity, enemy_pos, _)) = find_closest_enemy(
                    transform.position,
                    acquisition_range,
                    owner.0,
                    &enemy_query,
                ) {
                    // Set attack target
                    commands.entity(entity).insert(AttackTarget {
                        target_entity: enemy_entity,
                    });
                    
                    // Move to target if not in attack range
                    let distance = (enemy_pos - transform.position).length();
                    if distance > unit.attack_range && movement.is_some() {
                        let mut movement = movement.unwrap();
                        movement.path = vec![enemy_pos];
                        movement.path_index = 0;
                    }
                }
            }
            UnitType::Healer => {
                // Healers look for damaged friendly units
                // This would be implemented in a real game
            }
        }
    }
}

/// Helper function to find the closest enemy
fn find_closest_enemy(
    position: Vec2,
    range: f32,
    owner: u8,
    enemy_query: &Query<(Entity, &Transform, &Owner), (With<Unit>, With<Building>)>,
) -> Option<(Entity, Vec2, u8)> {
    let mut closest_enemy = None;
    let mut closest_distance = f32::MAX;
    
    for (entity, transform, entity_owner) in enemy_query.iter() {
        // Skip owned entities
        if entity_owner.0 == owner {
            continue;
        }
        
        let distance = (transform.position - position).length();
        if distance < range && distance < closest_distance {
            closest_enemy = Some((entity, transform.position, entity_owner.0));
            closest_distance = distance;
        }
    }
    
    closest_enemy
}

/// System to update the game's economic state
pub fn economy_system(
    mut player_resources: ResMut<PlayerResources>,
    time: Res<GameTime>,
) {
    // Apply income rates
    for (&(player_id, resource_type), &rate) in player_resources.income_rate.iter() {
        let amount = rate * time.delta_time;
        if let Some(current) = player_resources.resources.get_mut(&(player_id, resource_type)) {
            *current += amount;
        }
    }
    
    // Apply maintenance costs
    // This would be implemented in a real game
}