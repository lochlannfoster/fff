use bevy_ecs::prelude::*;
use glam::Vec2;

use crate::ecs::components::{Unit, Building, Transform, Owner, ResourceType};
use crate::ecs::resources::{GameTime, PlayerResources};
use crate::game::units;

pub fn repair_system(
    mut commands: Commands,
    time: Res<GameTime>,
    mut worker_query: Query<(Entity, &Unit, &Transform, &Owner)>,
    mut building_query: Query<(Entity, &mut Building, &Transform, &Owner)>,
    mut player_resources: ResMut<PlayerResources>,
) {
    for (worker_entity, worker, worker_transform, worker_owner) in worker_query.iter_mut() {
        for (building_entity, mut building, building_transform, building_owner) in building_query.iter_mut() {
            // Check if worker can repair this building
            if building_owner.0 == worker_owner.0 && 
               units::can_repair_building(worker, &building, building_transform, worker_transform) {
                
                // Repair cost and speed
                let repair_cost_per_second = 1.0;
                let repair_speed = 10.0; // HP per second
                
                // Check if player can afford repair
                if let Some(current) = player_resources.resources.get_mut(&(worker_owner.0, ResourceType::Mineral)) {
                    if *current >= repair_cost_per_second * time.delta_time {
                        // Deduct repair cost
                        *current -= repair_cost_per_second * time.delta_time;
                        
                        // Repair building
                        let repair_amount = repair_speed * time.delta_time;
                        building.health = (building.health + repair_amount).min(building.max_health);
                    }
                }
            }
        }
    }
}