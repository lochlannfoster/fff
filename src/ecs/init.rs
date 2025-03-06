use bevy_ecs::prelude::*;
use crate::ecs::resources::*;
use crate::ecs::systems::combat::components::DamageTable;

/// Initialize the ECS world with starting resources
pub fn init_world() -> World {
    let mut world = World::new();
    
    // Add resources
    world.insert_resource(GameTime::default());
    world.insert_resource(PlayerResources::default());
    world.insert_resource(TechState::default());
    world.insert_resource(GameSettings::default());
    world.insert_resource(PlayerInfo::default());
    world.insert_resource(SelectionState::default());
    world.insert_resource(ControlGroups::default());
    world.insert_resource(InputActionQueue::default());
    world.insert_resource(CameraState::default());
    
    // Add combat-specific resources
    world.insert_resource(DamageTable::default());
    
    world
}