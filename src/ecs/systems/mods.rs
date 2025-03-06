pub mod combat;
pub mod resource_collection_system;
pub mod building_production_system;
pub mod economy_system;

// Re-export systems for easier access
pub use combat::combat_system;
pub use resource_collection_system::resource_collection_system;
pub use building_production_system::building_production_system;
pub use economy_system::economy_system;
pub use super::update_movement_system;
pub use super::collision_detection_system;
pub use super::unit_behavior_system;
pub use super::fog_of_war_system;