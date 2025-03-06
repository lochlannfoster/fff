use bevy_ecs::prelude::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

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

/// Data mapping armor types to damage multipliers for each damage type
#[derive(Resource)]
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