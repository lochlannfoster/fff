mod engine;
mod ecs;
mod game;
mod networking;
mod ui;

use anyhow::Result;
use log::{info, error, warn};
use winit::event_loop::EventLoop;
use glam::Vec2;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::ecs::components::*;
use crate::ecs::resources::*;
use crate::game::{GamePhase, GameState};
// TODO: Implement autobattler menu factory

const TICK_RATE: f64 = 20.0; // 20 ticks per second
const MS_PER_TICK: f64 = 1000.0 / TICK_RATE;

/// Sophisticated army composition strategy
struct ArmyCompositionStrategy {
    soldier_ratio: f32,
    scout_ratio: f32,
    tank_ratio: f32,
    healer_ratio: f32,
}

impl ArmyCompositionStrategy {
    fn new(complexity: f32) -> Self {
        match complexity {
            x if x < 0.33 => Self {
                soldier_ratio: 0.6,
                scout_ratio: 0.2,
                tank_ratio: 0.1,
                healer_ratio: 0.1,
            },
            x if x < 0.66 => Self {
                soldier_ratio: 0.4,
                scout_ratio: 0.2,
                tank_ratio: 0.3,
                healer_ratio: 0.1,
            },
            _ => Self {
                soldier_ratio: 0.3,
                scout_ratio: 0.1,
                tank_ratio: 0.4,
                healer_ratio: 0.2,
            }
        }
    }

    fn suggest_next_unit(&mut self, current_units: &[UnitType]) -> UnitType {
        let counts = current_units.iter().fold(
            [0, 0, 0, 0], 
            |mut acc, &unit| {
                match unit {
                    UnitType::Soldier => acc[0] += 1,
                    UnitType::Scout => acc[1] += 1,
                    UnitType::Tank => acc[2] += 1,
                    UnitType::Healer => acc[3] += 1,
                }
                acc
            }
        );

        let total_units = current_units.len() as f32;
        let ratios = [
            counts[0] as f32 / total_units,
            counts[1] as f32 / total_units,
            counts[2] as f32 / total_units,
            counts[3] as f32 / total_units,
        ];

        let target_ratios = [
            self.soldier_ratio,
            self.scout_ratio, 
            self.tank_ratio, 
            self.healer_ratio
        ];

        // Find unit type furthest from its target ratio
        target_ratios.iter()
            .zip(ratios.iter())
            .enumerate()
            .max_by(|(_, &(target, _)), (_, &(other_target, _))| 
                f32::abs(target - ratios[_]).partial_cmp(&f32::abs(other_target - ratios[_])).unwrap()
            )
            .map(|(idx, _)| match idx {
                0 => UnitType::Soldier,
                1 => UnitType::Scout,
                2 => UnitType::Tank,
                3 => UnitType::Healer,
                _ => UnitType::Soldier,
            })
            .unwrap_or(UnitType::Soldier)
    }
}

/// Enhanced autobattler initialization
fn initialize_autobattler(
    world: &mut bevy_ecs::world::World, 
    game_state: &mut GameState,
    army_strategy: &mut ArmyCompositionStrategy,
) {
    // Generate map with autobattler-friendly parameters
    let map_params = game::map::MapGenerationParams {
        width: 300,
        height: 300,
        seed: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        player_count: 2,
        water_threshold: 0.2,
        mountain_threshold: 0.8,
        forest_threshold: 0.5,
        resource_density: 0.02,
    };
    
    let game_map = game::map::generate_map(&map_params);
    world.insert_resource(DamageTable::default());

    // Player starting positions
    let start_positions = [
        Vec2::new(50.0, 50.0),   // Player 0
        Vec2::new(250.0, 250.0), // Player 1
    ];
    
    // Spawn headquarters and initial army for each player
    for (player_id, &pos) in start_positions.iter().enumerate() {
        // Spawn headquarters
        world.spawn((
            Building {
                building_type: BuildingType::Headquarters,
                health: 1500.0,
                max_health: 1500.0,
                production_queue: std::collections::VecDeque::new(),
                production_progress: None,
                construction_progress: None,
                rally_point: None,
            },
            Transform {
                position: pos,
                rotation: 0.0,
                scale: Vec2::new(2.0, 2.0),
            },
            Owner(player_id as u8),
            Collider {
                radius: 15.0,
                collision_layer: 2, // Building layer
                collision_mask: 1 | 2, // Collide with units and buildings
            },
        ));

        // Initial army generation using strategy
        let mut player_units = Vec::new();
        for _ in 0..10 {
            let unit_type = army_strategy.suggest_next_unit(&player_units);
            player_units.push(unit_type);

            let offset = Vec2::new(
                rand::thread_rng().gen_range(-20.0..20.0),
                rand::thread_rng().gen_range(-20.0..20.0)
            );
            
            game::units::spawn_unit(
                &mut world.commands(), 
                game::units::UnitSpawnParams {
                    unit_type,
                    owner: player_id as u8,
                    position: pos + offset,
                },
                &world.resource::<TechState>()
            );
        }
    }

    // Configure game state
    game_state.start_game(
        false,  // Single player
        2,      // Two players
        map_params.seed
    );
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting Rusty Autobattler");

    // Create game window
    let (mut engine, event_loop) = engine::Engine::new("Rusty Autobattler", 1024, 768).await?;
    
    // Load game assets
    engine.load_assets()?;

    // Create army composition strategy
    let mut army_strategy = ArmyCompositionStrategy::new(0.5); // Medium complexity

    // Initialize autobattler game state
    initialize_autobattler(&mut engine.world, &mut engine.game_state, &mut army_strategy);

    // Optional: Add simple networking for potential multiplayer
    if let Err(e) = engine.enable_networking(true, None) {
        warn!("Failed to enable networking: {}", e);
    }

    // Run the game
    engine.run(event_loop)
}