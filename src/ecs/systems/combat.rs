use bevy_ecs::prelude::*;
use glam::Vec2;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;

use crate::ecs::components::{
    Transform, Unit, Building, Owner, AttackTarget, Movement,
    UnitType, BuildingType,
};
use crate::ecs::resources::{GameTime};

/// Component for tracking attack cooldown
#[derive(Component, Debug)]
pub struct AttackCooldown {
    pub remaining: f32,
    pub base_cooldown: f32,
}

/// Component for projectiles
#[derive(Component, Debug)]
pub struct Projectile {
    pub source_entity: Entity,
    pub target_entity: Entity,
    pub damage: f32,
    pub speed: f32,
    pub max_distance: f32,
    pub traveled_distance: f32,
    pub aoe_radius: Option<f32>,
}

/// Component for effects like explosions
#[derive(Component, Debug)]
pub struct Effect {
    pub effect_type: EffectType,
    pub duration: f32,
    pub elapsed: f32,
    pub scale: f32,
}

/// Types of visual effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    Explosion,
    Fire,
    Smoke,
    Heal,
    Shield,
}

/// Damage type for combat calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Energy,
    Explosive,
}

/// Armor type for damage reduction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorType {
    Light,
    Medium,
    Heavy,
    Building,
}

/// Weapon type for combat calculations
#[derive(Debug, Clone)]
pub struct WeaponData {
    pub damage: f32,
    pub range: f32,
    pub cooldown: f32, // Time between attacks in seconds
    pub damage_type: DamageType,
    pub projectile_speed: Option<f32>, // None for instant-hit weapons
    pub splash_radius: Option<f32>,    // None for single-target weapons
    pub accuracy: f32,                 // 0.0 to 1.0
}

/// Data mapping armor types to damage multipliers for each damage type
pub struct DamageTable {
    pub multipliers: HashMap<(DamageType, ArmorType), f32>,
}

impl Default for DamageTable {
    fn default() -> Self {
        let mut multipliers = HashMap::new();
        
        // Physical damage
        multipliers.insert((DamageType::Physical, ArmorType::Light), 1.25);
        multipliers.insert((DamageType::Physical, ArmorType::Medium), 1.0);
        multipliers.insert((DamageType::Physical, ArmorType::Heavy), 0.75);
        multipliers.insert((DamageType::Physical, ArmorType::Building), 0.5);
        
        // Energy damage
        multipliers.insert((DamageType::Energy, ArmorType::Light), 1.0);
        multipliers.insert((DamageType::Energy, ArmorType::Medium), 1.25);
        multipliers.insert((DamageType::Energy, ArmorType::Heavy), 1.0);
        multipliers.insert((DamageType::Energy, ArmorType::Building), 0.75);
        
        // Explosive damage
        multipliers.insert((DamageType::Explosive, ArmorType::Light), 1.0);
        multipliers.insert((DamageType::Explosive, ArmorType::Medium), 0.75);
        multipliers.insert((DamageType::Explosive, ArmorType::Heavy), 1.25);
        multipliers.insert((DamageType::Explosive, ArmorType::Building), 1.5);
        
        Self { multipliers }
    }
}

/// Get the weapon data for a unit type
pub fn get_weapon_data(unit_type: UnitType) -> Option<WeaponData> {
    match unit_type {
        UnitType::Worker => Some(WeaponData {
            damage: 3.0,
            range: 10.0,
            cooldown: 1.0,
            damage_type: DamageType::Physical,
            projectile_speed: None,
            splash_radius: None,
            accuracy: 0.9,
        }),
        UnitType::Soldier => Some(WeaponData {
            damage: 10.0,
            range: 50.0,
            cooldown: 0.8,
            damage_type: DamageType::Physical,
            projectile_speed: Some(200.0),
            splash_radius: None,
            accuracy: 0.85,
        }),
        UnitType::Scout => Some(WeaponData {
            damage: 6.0,
            range: 40.0,
            cooldown: 0.5,
            damage_type: DamageType::Physical,
            projectile_speed: Some(250.0),
            splash_radius: None,
            accuracy: 0.95,
        }),
        UnitType::Tank => Some(WeaponData {
            damage: 30.0,
            range: 70.0,
            cooldown: 2.0,
            damage_type: DamageType::Explosive,
            projectile_speed: Some(150.0),
            splash_radius: Some(20.0),
            accuracy: 0.8,
        }),
        UnitType::Healer => None, // No weapon, healing ability instead
    }
}

/// Get the weapon data for a building type
pub fn get_building_weapon(building_type: BuildingType) -> Option<WeaponData> {
    match building_type {
        BuildingType::DefenseTower => Some(WeaponData {
            damage: 15.0,
            range: 100.0,
            cooldown: 1.0,
            damage_type: DamageType::Energy,
            projectile_speed: Some(300.0),
            splash_radius: None,
            accuracy: 0.9,
        }),
        _ => None, // Other buildings don't have weapons
    }
}

/// Get the armor type for a unit type
pub fn get_unit_armor_type(unit_type: UnitType) -> ArmorType {
    match unit_type {
        UnitType::Worker => ArmorType::Light,
        UnitType::Scout => ArmorType::Light,
        UnitType::Soldier => ArmorType::Medium,
        UnitType::Tank => ArmorType::Heavy,
        UnitType::Healer => ArmorType::Light,
    }
}

/// Get the armor type for a building type
pub fn get_building_armor_type(_building_type: BuildingType) -> ArmorType {
    ArmorType::Building
}

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
    
    // Update cooldowns and process attacks for units
    for (entity, mut unit, transform, owner, attack_target, cooldown) in unit_query.iter_mut() {
        // Skip if unit has no target
        if attack_target.is_none() {
            continue;
        }
        
        // Get target entity
        let target_entity = attack_target.unwrap().target_entity;
        
        // Skip if target doesn't exist
        if !transform_query.contains(target_entity) {
            continue;
        }
        
        // Get target position
        let target_transform = transform_query.get(target_entity).unwrap();
        let target_position = target_transform.position;
        let unit_position = transform.position;
        
        // Get weapon data
        if let Some(weapon) = get_weapon_data(unit.unit_type) {
            // Calculate distance to target
            let distance = (target_position - unit_position).length();
            
            // Check if target is in range
            if distance <= weapon.range {
                // Check attack cooldown
                let can_attack = match cooldown {
                    Some(mut cooldown) => {
                        // Update cooldown
                        cooldown.remaining -= time.delta_time;
                        
                        // Check if we can attack
                        if cooldown.remaining <= 0.0 {
                            // Reset cooldown
                            cooldown.remaining = cooldown.base_cooldown;
                            true
                        } else {
                            false
                        }
                    }
                    None => {
                        // No cooldown component, create one
                        commands.entity(entity).insert(AttackCooldown {
                            remaining: weapon.cooldown,
                            base_cooldown: weapon.cooldown,
                        });
                        
                        // Can attack immediately
                        true
                    }
                };
                
                if can_attack {
                    // Check for accuracy hit/miss
                    let accuracy_roll = rng.gen_range(0.0..1.0);
                    if accuracy_roll <= weapon.accuracy {
                        // Hit successful - process attack
                        process_attack(
                            &mut commands,
                            entity,
                            target_entity,
                            &weapon,
                            unit_position,
                            target_position,
                        );
                    } else {
                        // Miss - show miss effect
                        let miss_position = target_position + Vec2::new(
                            rng.gen_range(-5.0..5.0),
                            rng.gen_range(-5.0..5.0),
                        );
                        
                        // Could spawn a "miss" effect here
                    }
                }
            }
        }
    }
    
    // Update cooldowns and process attacks for buildings
    for (entity, mut building, transform, owner, attack_target, cooldown) in building_query.iter_mut() {
        // Skip if building has no target or no weapon
        if attack_target.is_none() {
            continue;
        }
        
        if let Some(weapon) = get_building_weapon(building.building_type) {
            // Get target entity
            let target_entity = attack_target.unwrap().target_entity;
            
            // Skip if target doesn't exist
            if !transform_query.contains(target_entity) {
                continue;
            }
            
            // Get target position
            let target_transform = transform_query.get(target_entity).unwrap();
            let target_position = target_transform.position;
            let building_position = transform.position;
            
            // Calculate distance to target
            let distance = (target_position - building_position).length();
            
            // Check if target is in range
            if distance <= weapon.range {
                // Check attack cooldown
                let can_attack = match cooldown {
                    Some(mut cooldown) => {
                        // Update cooldown
                        cooldown.remaining -= time.delta_time;
                        
                        // Check if we can attack
                        if cooldown.remaining <= 0.0 {
                            // Reset cooldown
                            cooldown.remaining = cooldown.base_cooldown;
                            true
                        } else {
                            false
                        }
                    }
                    None => {
                        // No cooldown component, create one
                        commands.entity(entity).insert(AttackCooldown {
                            remaining: weapon.cooldown,
                            base_cooldown: weapon.cooldown,
                        });
                        
                        // Can attack immediately
                        true
                    }
                };
                
                if can_attack {
                    // Buildings typically have perfect accuracy
                    process_attack(
                        &mut commands,
                        entity,
                        target_entity,
                        &weapon,
                        building_position,
                        target_position,
                    );
                }
            }
        }
    }
    
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
            // Apply damage
            apply_damage(
                &mut commands,
                &damage_table,
                projectile.source_entity,
                projectile.target_entity,
                projectile.damage,
                projectile.aoe_radius,
                &unit_query,
                &building_query,
                &transform_query,
                transform.position,
            );
            
            // Spawn hit effect
            spawn_hit_effect(&mut commands, transform.position, 0.5);
            
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
                // Would also update color/alpha in a real implementation
            }
            EffectType::Fire => {
                // Flicker fire
                let flicker = 1.0 + 0.2 * (time.current_tick as f32 * 10.0).sin();
                transform.scale = Vec2::splat(effect.scale * flicker);
            }
            // Handle other effect types...
            _ => {}
        }
    }
}

/// Process an attack from one entity to another
fn process_attack(
    commands: &mut Commands,
    attacker: Entity,
    target: Entity,
    weapon: &WeaponData,
    attacker_pos: Vec2,
    target_pos: Vec2,
) {
    if let Some(projectile_speed) = weapon.projectile_speed {
        // Ranged attack - spawn projectile
        let direction = (target_pos - attacker_pos).normalize_or_zero();
        
        // Create a new projectile entity
        commands.spawn((
            Projectile {
                source_entity: attacker,
                target_entity: target,
                damage: weapon.damage,
                speed: projectile_speed,
                max_distance: weapon.range * 1.5, // Allow some extra distance
                traveled_distance: 0.0,
                aoe_radius: weapon.splash_radius,
            },
            Transform {
                position: attacker_pos,
                rotation: direction.y.atan2(direction.x),
                scale: Vec2::splat(1.0),
            },
            // Would also add a Sprite component in a real implementation
        ));
    } else {
        // Instant-hit attack
        // Apply damage directly
        // The damage application function would be implemented elsewhere
    }
}

/// Apply damage to a target and nearby entities if AOE
fn apply_damage(
    commands: &mut Commands,
    damage_table: &DamageTable,
    attacker: Entity,
    target: Entity,
    base_damage: f32,
    aoe_radius: Option<f32>,
    unit_query: &Query<(
        Entity,
        &mut Unit,
        &Transform,
        &Owner,
        Option<&AttackTarget>,
        Option<&mut AttackCooldown>,
    )>,
    building_query: &Query<(
        Entity,
        &mut Building,
        &Transform,
        &Owner,
        Option<&AttackTarget>,
        Option<&mut AttackCooldown>,
    )>,
    transform_query: &Query<&Transform>,
    impact_position: Vec2,
) {
    // First, apply damage to the direct target
    apply_damage_to_entity(commands, damage_table, attacker, target, base_damage, unit_query, building_query);
    
    // If this is an AOE attack, apply reduced damage to nearby entities
    if let Some(radius) = aoe_radius {
        // Get attacker owner to avoid friendly fire
        let attacker_owner = get_entity_owner(attacker, unit_query, building_query);
        
        // Find nearby units
        for (entity, unit, transform, owner, _, _) in unit_query.iter() {
            // Skip the main target and friendly units
            if entity == target || Some(owner.0) == attacker_owner {
                continue;
            }
            
            // Check distance
            let distance = (transform.position - impact_position).length();
            if distance <= radius {
                // Calculate damage falloff based on distance
                let damage_multiplier = 1.0 - (distance / radius).min(1.0);
                let aoe_damage = base_damage * damage_multiplier * 0.5; // AOE deals 50% at most
                
                apply_damage_to_entity(commands, damage_table, attacker, entity, aoe_damage, unit_query, building_query);
            }
        }
        
        // Find nearby buildings
        for (entity, building, transform, owner, _, _) in building_query.iter() {
            // Skip the main target and friendly buildings
            if entity == target || Some(owner.0) == attacker_owner {
                continue;
            }
            
            // Check distance
            let distance = (transform.position - impact_position).length();
            if distance <= radius {
                // Calculate damage falloff based on distance
                let damage_multiplier = 1.0 - (distance / radius).min(1.0);
                let aoe_damage = base_damage * damage_multiplier * 0.5; // AOE deals 50% at most
                
                apply_damage_to_entity(commands, damage_table, attacker, entity, aoe_damage, unit_query, building_query);
            }
        }
        
        // Spawn explosion effect
        spawn_explosion_effect(commands, impact_position, radius / 20.0);
    }
}

/// Apply damage to a specific entity
fn apply_damage_to_entity(
    commands: &mut Commands,
    damage_table: &DamageTable,
    attacker: Entity,
    target: Entity,
    base_damage: f32,
    unit_query: &Query<(
        Entity,
        &mut Unit,
        &Transform,
        &Owner,
        Option<&AttackTarget>,
        Option<&mut AttackCooldown>,
    )>,
    building_query: &Query<(
        Entity,
        &mut Building,
        &Transform,
        &Owner,
        Option<&AttackTarget>,
        Option<&mut AttackCooldown>,
    )>,
) {
    // Get attacker damage type
    let damage_type = get_attacker_damage_type(attacker, unit_query, building_query);
    
    // Try to apply damage to a unit
    if let Ok((_, mut unit, transform, _, _, _)) = unit_query.get_mut(target) {
        // Get armor type
        let armor_type = get_unit_armor_type(unit.unit_type);
        
        // Calculate actual damage
        let damage_multiplier = damage_table.multipliers
            .get(&(damage_type, armor_type))
            .copied()
            .unwrap_or(1.0);
        
        let actual_damage = base_damage * damage_multiplier;
        
        // Apply damage
        unit.health -= actual_damage;
        
        // Check if unit is destroyed
        if unit.health <= 0.0 {
            // Spawn death effect
            spawn_death_effect(commands, transform.position, 1.0);
            
            // Despawn unit
            commands.entity(target).despawn();
        }
    }
    // Try to apply damage to a building
    else if let Ok((_, mut building, transform, _, _, _)) = building_query.get_mut(target) {
        // Get armor type
        let armor_type = get_building_armor_type(building.building_type);
        
        // Calculate actual damage
        let damage_multiplier = damage_table.multipliers
            .get(&(damage_type, armor_type))
            .copied()
            .unwrap_or(1.0);
        
        let actual_damage = base_damage * damage_multiplier;
        
        // Apply damage
        building.health -= actual_damage;
        
        // Check if building is destroyed
        if building.health <= 0.0 {
            // Spawn destruction effect
            spawn_building_destruction_effect(commands, transform.position, 2.0);
            
            // Despawn building
            commands.entity(target).despawn();
        }
    }
}

/// Get the owner of an entity
fn get_entity_owner(
    entity: Entity,
    unit_query: &Query<(
        Entity,
        &mut Unit,
        &Transform,
        &Owner,
        Option<&AttackTarget>,
        Option<&mut AttackCooldown>,
    )>,
    building_query: &Query<(
        Entity,
        &mut Building,
        &Transform,
        &Owner,
        Option<&AttackTarget>,
        Option<&mut AttackCooldown>,
    )>,
) -> Option<u8> {
    // Check if entity is a unit
    if let Ok((_, _, _, owner, _, _)) = unit_query.get(entity) {
        return Some(owner.0);
    }
    
    // Check if entity is a building
    if let Ok((_, _, _, owner, _, _)) = building_query.get(entity) {
        return Some(owner.0);
    }
    
    None
}

/// Get damage type for an attacker
fn get_attacker_damage_type(
    entity: Entity,
    unit_query: &Query<(
        Entity,
        &mut Unit,
        &Transform,
        &Owner,
        Option<&AttackTarget>,
        Option<&mut AttackCooldown>,
    )>,
    building_query: &Query<(
        Entity,
        &mut Building,
        &Transform,
        &Owner,
        Option<&AttackTarget>,
        Option<&mut AttackCooldown>,
    )>,
) -> DamageType {
    // Check if attacker is a unit
    if let Ok((_, unit, _, _, _, _)) = unit_query.get(entity) {
        if let Some(weapon) = get_weapon_data(unit.unit_type) {
            return weapon.damage_type;
        }
    }
    
    // Check if attacker is a building
    if let Ok((_, building, _, _, _, _)) = building_query.get(entity) {
        if let Some(weapon) = get_building_weapon(building.building_type) {
            return weapon.damage_type;
        }
    }
    
    // Default damage type
    DamageType::Physical
}

/// Spawn a hit effect
fn spawn_hit_effect(commands: &mut Commands, position: Vec2, scale: f32) {
    commands.spawn((
        Effect {
            effect_type: EffectType::Explosion,
            duration: 0.3,
            elapsed: 0.0,
            scale,
        },
        Transform {
            position,
            rotation: 0.0,
            scale: Vec2::splat(scale),
        },
        // Would also add a Sprite component in a real implementation
    ));
}

/// Spawn an explosion effect
fn spawn_explosion_effect(commands: &mut Commands, position: Vec2, scale: f32) {
    commands.spawn((
        Effect {
            effect_type: EffectType::Explosion,
            duration: 0.5,
            elapsed: 0.0,
            scale,
        },
        Transform {
            position,
            rotation: 0.0,
            scale: Vec2::splat(scale),
        },
        // Would also add a Sprite component in a real implementation
    ));
}

/// Spawn a death effect for units
fn spawn_death_effect(commands: &mut Commands, position: Vec2, scale: f32) {
    commands.spawn((
        Effect {
            effect_type: EffectType::Fire,
            duration: 1.0,
            elapsed: 0.0,
            scale,
        },
        Transform {
            position,
            rotation: 0.0,
            scale: Vec2::splat(scale),
        },
        // Would also add a Sprite component in a real implementation
    ));
}

/// Spawn a building destruction effect
fn spawn_building_destruction_effect(commands: &mut Commands, position: Vec2, scale: f32) {
    commands.spawn((
        Effect {
            effect_type: EffectType::Explosion,
            duration: 2.0,
            elapsed: 0.0,
            scale,
        },
        Transform {
            position,
            rotation: 0.0,
            scale: Vec2::splat(scale),
        },
        // Would also add a Sprite component in a real implementation
    ));
}