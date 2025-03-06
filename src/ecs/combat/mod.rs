pub mod components;
pub mod system;

pub use components::*;
pub use system::*;

use bevy_ecs::prelude::*;
use glam::Vec2;
use std::collections::{HashMap, VecDeque};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::ecs::components::{UnitType, BuildingType, ResourceType, Transform, Owner, Unit, Building};
use crate::ecs::resources::{GameMap, PlayerResources};

pub mod systems;
pub mod components;

// Re-export the combat system
pub use systems::combat_system;
pub use components::DamageTable;