use bevy_ecs::prelude::*;
use glam::Vec2;

use crate::ecs::components::{
    Unit, UnitType, Owner, Transform, Collider, 
    Movement, MinimapMarker, MinimapShape
};
use crate::ecs::resources::{TechState, TechEffectType, PlayerResources};
use crate::game::tech::{TechData, apply_tech_effect};
use crate::game::buildings::BuildingData;
use crate::ecs::components::{BuildingType, ResourceType};

/// Unit spawn parameters
pub struct UnitSpawnParams {
    pub unit_type: UnitType,
    pub owner: u8,
    pub position: Vec2,
}

/// Calculate training time for a unit
pub fn calculate_training_time(
    unit_type: UnitType, 
    tech_state: &TechState, 
    player_id: u8
) -> f32 {
    let base_times = match unit_type {
        UnitType::Worker => 15.0,
        UnitType::Soldier => 25.0,
        UnitType::Scout => 20.0,
        UnitType::Tank => 40.0,
        UnitType::Healer => 30.0,
    };

    // Apply tech effects to reduce training time
    apply_tech_effect(
        tech_state, 
        player_id, 
        base_times, 
        TechEffectType::BuildTime
    )
}

/// Check if a unit can be trained
pub fn can_train_unit(
    unit_type: UnitType,
    player_resources: &PlayerResources, 
    player_id: u8,
    tech_state: &TechState,
) -> bool {
    let costs = match unit_type {
        UnitType::Worker => {
            let mut costs = HashMap::new();
            costs.insert(ResourceType::Mineral, 50.0);
            costs
        },
        UnitType::Soldier => {
            let mut costs = HashMap::new();
            costs.insert(ResourceType::Mineral, 75.0);
            costs.insert(ResourceType::Energy, 10.0);
            costs
        },
        UnitType::Scout => {
            let mut costs = HashMap::new();
            costs.insert(ResourceType::Mineral, 60.0);
            costs.insert(ResourceType::Energy, 5.0);
            costs
        },
        UnitType::Tank => {
            let mut costs = HashMap::new();
            costs.insert(ResourceType::Mineral, 150.0);
            costs.insert(ResourceType::Gas, 50.0);
            costs
        },
        UnitType::Healer => {
            let mut costs = HashMap::new();
            costs.insert(ResourceType::Mineral, 100.0);
            costs.insert(ResourceType::Energy, 25.0);
            costs
        },
    };

    // Check if player has enough resources
    for (&resource_type, &cost) in &costs {
        let current = player_resources.resources
            .get(&(player_id, resource_type))
            .copied()
            .unwrap_or(0.0);
        
        if current < cost {
            return false;
        }
    }

    // Check tech requirements
    match unit_type {
        UnitType::Tank => {
            let tech_researched = tech_state.researched
                .get(&(player_id, TechType::AdvancedUnits))
                .copied()
                .unwrap_or(false);
            
            if !tech_researched {
                return false;
            }
        },
        _ => {}
    }

    true
}

/// Check if a worker can build a specific building
pub fn can_build_building(
    unit: &Unit, 
    building_type: BuildingType, 
    player_resources: &PlayerResources,
    player_id: u8,
) -> bool {
    // Ensure it's a worker
    if unit.unit_type != UnitType::Worker {
        return false;
    }

    // Get building costs
    let building_data = BuildingData::get(building_type);
    let costs = building_data.costs;

    // Check resource availability
    for (&resource_type, &cost) in &costs {
        let current = player_resources.resources
            .get(&(player_id, resource_type))
            .copied()
            .unwrap_or(0.0);
        
        if current < cost {
            return false;
        }
    }

    true
}

/// Check if a worker can repair a building
pub fn can_repair_building(
    worker: &Unit, 
    building: &Building,
    transform: &Transform,
    worker_transform: &Transform,
) -> bool {
    // Check if unit is a worker
    if worker.unit_type != UnitType::Worker {
        return false;
    }

    // Check repair distance (e.g., 10 units)
    let distance = (transform.position - worker_transform.position).length();
    
    // Check if building needs repair
    distance <= 10.0 && 
    building.health < building.max_health
}

// SPLIT 1
/// Spawn a new unit entity
pub fn spawn_unit(
    commands: &mut Commands,
    params: UnitSpawnParams,
    tech_state: &TechState,
) -> Option<Entity> {
    // Calculate unit stats with tech effects
    let (health, attack_damage, attack_range, attack_speed, movement_speed, sight_range) = 
        calculate_unit_stats(params.unit_type, tech_state, params.owner);

    let entity = commands.spawn((
        Unit {
            unit_type: params.unit_type,
            health,
            max_health: health,
            attack_damage,
            attack_range,
            attack_speed,
            movement_speed,
            sight_range,
            buildable: params.unit_type == UnitType::Worker,
        },
        Transform {
            position: params.position,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
        },
        Owner(params.owner),
        Movement {
            path: Vec::new(),
            path_index: 0,
            target: None,
            velocity: Vec2::ZERO,
        },
        Collider {
            radius: match params.unit_type {
                UnitType::Tank => 8.0,
                UnitType::Worker | UnitType::Healer => 4.0,
                _ => 5.0,
            },
            collision_layer: 1, // Unit layer
            collision_mask: 1 | 2, // Collide with units and buildings
        },
        MinimapMarker {
            color: match params.owner {
                0 => [0, 0, 255, 255],   // Blue
                1 => [255, 0, 0, 255],   // Red
                2 => [0, 255, 0, 255],   // Green
                3 => [255, 255, 0, 255], // Yellow
                _ => [255, 255, 255, 255], // White
            },
            shape: match params.unit_type {
                UnitType::Worker => MinimapShape::Circle,
                UnitType::Tank => MinimapShape::Square,
                _ => MinimapShape::Triangle,
            },
        },
    )).id();

    Some(entity)
}

// SPLIT 2
/// Calculate unit stats with tech effects applied  
fn calculate_unit_stats(
    unit_type: UnitType, 
    tech_state: &TechState, 
    player_id: u8
) -> (f32, f32, f32, f32, f32, f32) {
    let (base_health, base_damage, base_range, base_attack_speed, base_movement, base_sight) = 
        match unit_type {
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

    // Apply tech multipliers
    let health = apply_tech_effect(
        tech_state, 
        player_id, 
        base_health, 
        TechEffectType::UnitHealth
    );

    let damage = apply_tech_effect(
        tech_state, 
        player_id, 
        base_damage, 
        TechEffectType::UnitDamage
    );

    let attack_range = apply_tech_effect(
        tech_state, 
        player_id, 
        base_range, 
        TechEffectType::UnitAttackRange  
    );

    let attack_speed = apply_tech_effect(
        tech_state, 
        player_id, 
        base_attack_speed, 
        TechEffectType::UnitAttackSpeed
    );

    let movement_speed = apply_tech_effect(
        tech_state, 
        player_id, 
        base_movement, 
        TechEffectType::UnitSpeed
    );

    let sight_range = apply_tech_effect(
        tech_state, 
        player_id, 
        base_sight, 
        TechEffectType::UnitSightRange
    );

    (health, damage, attack_range, attack_speed, movement_speed, sight_range)
}

// SPLIT 3
/// System to handle unit behavior and AI
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
                let heal_range = unit.attack_range;
                
                // Query for friendly units
                let mut friendly_query = world.query::<(Entity, &Unit, &Transform, &Owner)>();
                
                let mut lowest_health_target = None;
                let mut lowest_health_percentage = f32::MAX;
                
                for (friendly_entity, friendly_unit, friendly_transform, friendly_owner) in friendly_query.iter() {
                    // Skip if not same owner or full health
                    if friendly_owner.0 != owner.0 || 
                       friendly_unit.health >= friendly_unit.max_health {
                        continue;
                    }
                    
                    // Calculate health percentage  
                    let health_percentage = friendly_unit.health / friendly_unit.max_health;
                    
                    // Calculate distance
                    let distance = (friendly_transform.position - transform.position).length();
                    
                    // Check if within heal range and lowest health seen
                    if distance <= heal_range && health_percentage < lowest_health_percentage {
                        lowest_health_target = Some(friendly_entity);
                        lowest_health_percentage = health_percentage;
                    }
                }
                
                // If a low health target is found, move to heal
                if let Some(heal_target) = lowest_health_target {
                    commands.entity(entity).insert(HealTarget {
                        target_entity: heal_target,
                    });
                    
                    // Move to target if not in heal range  
                    if let Some(mut movement) = movement {
                        let target_transform = world.get::<Transform>(heal_target).unwrap();
                        let distance = (target_transform.position - transform.position).length();
                        
                        if distance > unit.attack_range {
                            movement.path = vec![target_transform.position];
                            movement.path_index = 0;
                        }
                    }
                }
            }
        }
    }
}

// SPLIT 4
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