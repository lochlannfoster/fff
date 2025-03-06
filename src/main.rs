mod engine;
mod ecs;
mod game;
mod networking;
mod ui;

use anyhow::Result;
use log::{info, error};
use winit::{
    event::{Event, WindowEvent, ElementState, KeyboardInput, VirtualKeyCode, MouseButton},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use glam::Vec2;

use crate::ecs::components::*;
use crate::ecs::resources::*;
use crate::engine::camera::CameraController;
use crate::engine::input::Command;
use crate::game::{GamePhase, GameState};
use crate::ui::menu_callbacks::MenuCallbacks;

const TICK_RATE: f64 = 20.0; // 20 ticks per second
const MS_PER_TICK: f64 = 1000.0 / TICK_RATE;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    info!("Starting Rusty RTS game engine");

    // Create window and event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Rusty RTS")
        .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
        .build(&event_loop)?;

    // Initialize renderer
    let mut renderer = pollster::block_on(engine::renderer::Renderer::new(&window))?;
    
    // Initialize engine systems
    let mut input_handler = engine::input::InputHandler::new();
    let mut time_system = engine::time::TimeSystem::new(TICK_RATE);
    
    // Initialize ECS world
    let mut world = init_world();
    
    // Create game state
    let mut game_state = game::GameState::new();
    game_state.phase = GamePhase::MainMenu; // Start at main menu
    
    // Initialize UI manager
    let mut ui_manager = ui::UiManager::new(
        renderer.get_device().clone(),
        renderer.get_queue().clone(),
        1024, 768, 
        renderer.get_surface_format(),
    )?;
    
    // Set up UI screens
    setup_ui(&mut ui_manager);
    
    // Set up menu callbacks
    let menu_callbacks = MenuCallbacks::new(&mut game_state, &mut ui_manager);
    menu_callbacks.attach_callbacks(&mut ui_manager);
    
    // Initialize camera controller
    let mut camera_controller = CameraController::new(256.0, 256.0, 1024.0, 768.0);
    
    // Initialize network if multiplayer enabled
    let mut network = networking::lockstep::LockstepNetwork::new();
    
    // Generate a test map (this would normally happen when starting a game)
    let map_params = game::map::MapGenerationParams {
        width: 256,
        height: 256,
        seed: 12345,
        player_count: 2,
        water_threshold: 0.3,
        mountain_threshold: 0.7,
        forest_threshold: 0.6,
        resource_density: 0.01,
    };
    
    let game_map = game::map::generate_map(&map_params);
    world.insert_resource(game_map);
    
    // Add a few test entities
    spawn_test_entities(&mut world);
    
    // Flag to track if selection is in progress
    let mut selection_in_progress = false;
    let mut selection_start: Option<Vec2> = None;
    
    // Game loop - using winit event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                info!("Window close requested");
                *control_flow = ControlFlow::Exit;
            }
            
            Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                // Update renderer
                renderer.resize(new_size);
                
                // Update UI
                ui_manager.resize(new_size.width, new_size.height);
                
                // Update camera controller
                camera_controller.resize(new_size.width as f32, new_size.height as f32);
            }
            
            Event::WindowEvent { 
                event: WindowEvent::KeyboardInput { 
                    input: KeyboardInput { 
                        state: ElementState::Pressed, 
                        virtual_keycode: Some(VirtualKeyCode::Escape), 
                        .. 
                    }, 
                    .. 
                }, 
                .. 
            } => {
                // Toggle game pause with Escape key
                if game_state.phase == GamePhase::Playing {
                    game_state.phase = GamePhase::Paused;
                    ui_manager.set_active_screen("pause");
                } else if game_state.phase == GamePhase::Paused {
                    game_state.phase = GamePhase::Playing;
                    ui_manager.set_active_screen("game");
                }
            }
            
            Event::WindowEvent { 
                event: WindowEvent::MouseInput { 
                    state: ElementState::Pressed, 
                    button: MouseButton::Left, 
                    .. 
                }, 
                .. 
            } => {
                // Start selection
                if game_state.phase == GamePhase::Playing {
                    selection_in_progress = true;
                    selection_start = Some(input_handler.get_mouse_position());
                }
            }
            
            Event::WindowEvent { 
                event: WindowEvent::MouseInput { 
                    state: ElementState::Released, 
                    button: MouseButton::Left, 
                    .. 
                }, 
                .. 
            } => {
                // End selection and process it
                if game_state.phase == GamePhase::Playing && selection_in_progress {
                    selection_in_progress = false;
                    
                    let mouse_pos = input_handler.get_mouse_position();
                    
                    // First, check if UI handled the click
                    let handled_by_ui = ui_manager.handle_input(mouse_pos);
                    
                    if !handled_by_ui && selection_start.is_some() {
                        let start_pos = selection_start.unwrap();
                        
                        // Convert screen coordinates to world coordinates
                        let world_start = camera_controller.screen_to_world(start_pos);
                        let world_end = camera_controller.screen_to_world(mouse_pos);
                        
                        // Check if click or drag
                        let drag_threshold = 5.0;
                        if (start_pos - mouse_pos).length_squared() < drag_threshold * drag_threshold {
                            // Single click - select unit at position
                            input_handler.handle_command(Command::Select(world_end));
                        } else {
                            // Drag - area selection
                            input_handler.handle_command(Command::MultiSelect(world_start, world_end));
                        }
                    }
                    
                    selection_start = None;
                }
            }
            
            Event::WindowEvent { 
                event: WindowEvent::MouseInput { 
                    state: ElementState::Released, 
                    button: MouseButton::Right, 
                    .. 
                }, 
                .. 
            } => {
                // Issue move/attack command
                if game_state.phase == GamePhase::Playing {
                    let mouse_pos = input_handler.get_mouse_position();
                    
                    // Convert screen coordinates to world coordinates
                    let world_pos = camera_controller.screen_to_world(mouse_pos);
                    
                    // Check if Alt is pressed for attack-move
                    if input_handler.is_key_pressed(VirtualKeyCode::LAlt) || 
                       input_handler.is_key_pressed(VirtualKeyCode::RAlt) {
                        input_handler.handle_command(Command::Attack(world_pos));
                    } else {
                        input_handler.handle_command(Command::Move(world_pos));
                    }
                }
            }
            
            Event::WindowEvent { event, .. } => {
                // Process regular window events
                input_handler.handle_window_event(&event);
                
                // Handle camera movement from WASD keys
                if game_state.phase == GamePhase::Playing {
                    let mut movement = Vec2::ZERO;
                    
                    if input_handler.is_key_pressed(VirtualKeyCode::W) || 
                       input_handler.is_key_pressed(VirtualKeyCode::Up) {
                        movement.y -= 1.0;
                    }
                    
                    if input_handler.is_key_pressed(VirtualKeyCode::S) || 
                       input_handler.is_key_pressed(VirtualKeyCode::Down) {
                        movement.y += 1.0;
                    }
                    
                    if input_handler.is_key_pressed(VirtualKeyCode::A) || 
                       input_handler.is_key_pressed(VirtualKeyCode::Left) {
                        movement.x -= 1.0;
                    }
                    
                    if input_handler.is_key_pressed(VirtualKeyCode::D) || 
                       input_handler.is_key_pressed(VirtualKeyCode::Right) {
                        movement.x += 1.0;
                    }
                    
                    if movement != Vec2::ZERO {
                        camera_controller.move_camera(movement.normalize());
                    }
                    
                    // Handle zoom with + and - keys
                    if input_handler.is_key_pressed(VirtualKeyCode::Equals) || 
                       input_handler.is_key_pressed(VirtualKeyCode::Plus) {
                        camera_controller.zoom_camera(0.1);
                    }
                    
                    if input_handler.is_key_pressed(VirtualKeyCode::Minus) {
                        camera_controller.zoom_camera(-0.1);
                    }
                }
            }
            
            Event::MainEventsCleared => {
                // Get delta time
                let delta_time = time_system.get_delta_time();
                
                // Update camera
                camera_controller.update(delta_time);
                
                // Update camera state in renderer
                renderer.update_camera(camera_controller.position, camera_controller.zoom);
                
                // Update camera state in ECS world
                world.insert_resource(camera_controller.get_camera_state());
                
                // Tick game logic at fixed rate, potentially multiple times per frame
                while time_system.should_tick() {
                    // Only update game logic when not in menu or pause
                    if game_state.phase == GamePhase::Playing {
                        // Process inputs
                        let commands = input_handler.get_commands();
                        
                        // Process commands
                        process_commands(&mut world, &commands);
                        
                        // Network sync step
                        if network.is_active() {
                            if let Err(e) = network.send_commands(&commands) {
                                error!("Network error sending commands: {}", e);
                            }
                            network.receive_commands();
                        }
                        
                        // Run ECS systems
                        run_game_systems(&mut world, &game_state);
                        
                        // Advance game state
                        game_state.update();
                        
                        // Check for game over conditions
                        if let Some(winner) = game_state.winner {
                            game_state.phase = GamePhase::GameOver;
                            // Set game over UI
                            let winner_name = format!("Player {}", winner + 1);
                            ui_manager.set_active_screen("game_over");
                        }
                    }
                    
                    // Update time system
                    time_system.tick_completed();
                }
                
                // Update UI
                ui_manager.update(&game_state);
                
                // Update selection info in HUD
                if game_state.phase == GamePhase::Playing {
                    // Get selection state
                    let selection_state = world.resource::<SelectionState>();
                    
                    // Count units and get building type
                    let mut unit_count = 0;
                    let mut building_type = None;
                    
                    for &entity in &selection_state.selected_entities {
                        if let Some(unit) = world.get::<Unit>(entity) {
                            unit_count += 1;
                        } else if let Some(building) = world.get::<Building>(entity) {
                            building_type = Some(building.building_type);
                        }
                    }
                }
                
                // Render current game state (not tied to tick rate)
                match renderer.render(&world, &ui_manager) {
                    Ok(_) => {}
                    Err(e) => error!("Render error: {}", e),
                }
            }
            
            _ => {}
        }
    });
}

/// Initialize the ECS world with starting resources
fn init_world() -> bevy_ecs::world::World {
    let mut world = bevy_ecs::world::World::new();
    
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
    world.insert_resource(ecs::systems::combat::DamageTable::default());
    
    world
}

/// Set up UI screens
fn setup_ui(ui_manager: &mut ui::UiManager) {
    // Create UI screens based on UI factory
    let color_scheme = ui::UiColorScheme::default();
    let factory = ui::menus::MenuFactory::new(color_scheme, 1024, 768);
    
    // Main menu
    let main_menu_elements = factory.create_main_menu();
    for (id, element) in main_menu_elements {
        ui_manager.add_element(&format!("main_menu_{}", id), element);
    }
    
    // Settings menu
    let settings_elements = factory.create_settings_menu();
    for (id, element) in settings_elements {
        ui_manager.add_element(&format!("settings_{}", id), element);
    }
    
    // Game setup menu
    let setup_elements = factory.create_game_setup_menu();
    for (id, element) in setup_elements {
        ui_manager.add_element(&format!("setup_{}", id), element);
    }
    
    // Pause menu
    let pause_elements = factory.create_pause_menu();
    for (id, element) in pause_elements {
        ui_manager.add_element(&format!("pause_{}", id), element);
    }
    
    // Game over menu
    let game_over_elements = factory.create_game_over_menu("Unknown");
    for (id, element) in game_over_elements {
        ui_manager.add_element(&format!("game_over_{}", id), element);
    }
    
    // Set initial active screen
    ui_manager.set_active_screen("main_menu");
}

/// Spawn test entities for development
fn spawn_test_entities(world: &mut bevy_ecs::world::World) {
    use glam::Vec2;
    
    // Player 0 (Blue) entities
    
    // Headquarters
    world.spawn((
        Building {
            building_type: BuildingType::Headquarters,
            health: 1000.0,
            max_health: 1000.0,
            production_queue: std::collections::VecDeque::new(),
            production_progress: None,
            construction_progress: None,
            rally_point: None,
        },
        Transform {
            position: Vec2::new(50.0, 50.0),
            rotation: 0.0,
            scale: Vec2::new(2.0, 2.0),
        },
        Owner(0),
        Collider {
            radius: 15.0,
            collision_layer: 2, // Building layer
            collision_mask: 1 | 2, // Collide with units and buildings
        },
        MinimapMarker {
            color: [0, 0, 255, 255], // Blue
            shape: MinimapShape::Square,
        },
    ));
    
    // A few worker units
    for i in 0..5 {
        let pos = Vec2::new(80.0 + i as f32 * 10.0, 50.0);
        world.spawn((
            Unit {
                unit_type: UnitType::Worker,
                health: 30.0,
                max_health: 30.0,
                attack_damage: 3.0,
                attack_range: 10.0,
                attack_speed: 1.0,
                movement_speed: 80.0,
                sight_range: 100.0,
                buildable: true,
            },
            Transform {
                position: pos,
                rotation: 0.0,
                scale: Vec2::new(1.0, 1.0),
            },
            Owner(0),
            Movement {
                path: Vec::new(),
                path_index: 0,
                target: None,
                velocity: Vec2::ZERO,
            },
            Collider {
                radius: 4.0,
                collision_layer: 1, // Unit layer
                collision_mask: 1 | 2, // Collide with units and buildings
            },
            MinimapMarker {
                color: [0, 0, 255, 255], // Blue
                shape: MinimapShape::Circle,
            },
            Selectable,
        ));
    }
    
    // Player 1 (Red) entities
    
    // Headquarters
    world.spawn((
        Building {
            building_type: BuildingType::Headquarters,
            health: 1000.0,
            max_health: 1000.0,
            production_queue: std::collections::VecDeque::new(),
            production_progress: None,
            construction_progress: None,
            rally_point: None,
        },
        Transform {
            position: Vec2::new(200.0, 200.0),
            rotation: 0.0,
            scale: Vec2::new(2.0, 2.0),
        },
        Owner(1),
        Collider {
            radius: 15.0,
            collision_layer: 2, // Building layer
            collision_mask: 1 | 2, // Collide with units and buildings
        },
        MinimapMarker {
            color: [255, 0, 0, 255], // Red
            shape: MinimapShape::Square,
        },
    ));
    
    // A few worker units for player 1
    for i in 0..3 {
        let pos = Vec2::new(230.0 + i as f32 * 10.0, 200.0);
        world.spawn((
            Unit {
                unit_type: UnitType::Worker,
                health: 30.0,
                max_health: 30.0,
                attack_damage: 3.0,
                attack_range: 10.0,
                attack_speed: 1.0,
                movement_speed: 80.0,
                sight_range: 100.0,
                buildable: true,
            },
            Transform {
                position: pos,
                rotation: 0.0,
                scale: Vec2::new(1.0, 1.0),
            },
            Owner(1),
            Movement {
                path: Vec::new(),
                path_index: 0,
                target: None,
                velocity: Vec2::ZERO,
            },
            Collider {
                radius: 4.0,
                collision_layer: 1, // Unit layer
                collision_mask: 1 | 2, // Collide with units and buildings
            },
            MinimapMarker {
                color: [255, 0, 0, 255], // Red
                shape: MinimapShape::Circle,
            },
            Selectable,
        ));
    }
    
    // Add some resources
    world.spawn((
        Resource {
            resource_type: ResourceType::Mineral,
            amount: 1000.0,
        },
        Transform {
            position: Vec2::new(100.0, 100.0),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
        },
        Collider {
            radius: 8.0,
            collision_layer: 4, // Resource layer
            collision_mask: 0, // Nothing collides with resources
        },
    ));
    
    world.spawn((
        Resource {
            resource_type: ResourceType::Gas,
            amount: 800.0,
        },
        Transform {
            position: Vec2::new(150.0, 100.0),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
        },
        Collider {
            radius: 8.0,
            collision_layer: 4, // Resource layer
            collision_mask: 0, // Nothing collides with resources
        },
    ));
}

/// Process game commands
fn process_commands(world: &mut bevy_ecs::world::World, commands: &[Command]) {
    let mut selection_state = world.resource_mut::<SelectionState>();
    
    for command in commands {
        match command {
            Command::Select(position) => {
                // Handle unit selection at position
                let pos = *position;
                
                // Clear current selection
                for entity in &selection_state.selected_entities {
                    if let Some(mut entity_mut) = world.get_entity_mut(*entity) {
                        entity_mut.remove::<Selected>();
                    }
                }
                selection_state.selected_entities.clear();
                
                // Find entity at click position
                let mut query = world.query::<(bevy_ecs::entity::Entity, &Transform, &Collider, Option<&Selectable>)>();
                for (entity, transform, collider, selectable) in query.iter(world) {
                    if selectable.is_some() {
                        let distance = (transform.position - pos).length();
                        if distance <= collider.radius {
                            // Select this entity
                            selection_state.selected_entities.push(entity);
                            let mut entity_mut = world.entity_mut(entity);
                            entity_mut.insert(Selected);
                        }
                    }
                }
            },
            Command::MultiSelect(start, end) => {
                // Handle multi-selection with box
                let min_x = start.x.min(end.x);
                let max_x = start.x.max(end.x);
                let min_y = start.y.min(end.y);
                let max_y = start.y.max(end.y);
                
                // Clear current selection
                for entity in &selection_state.selected_entities {
                    if let Some(mut entity_mut) = world.get_entity_mut(*entity) {
                        entity_mut.remove::<Selected>();
                    }
                }
                selection_state.selected_entities.clear();
                
                // Find entities in selection box
                let mut query = world.query::<(bevy_ecs::entity::Entity, &Transform, Option<&Selectable>)>();
                for (entity, transform, selectable) in query.iter(world) {
                    if selectable.is_some() {
                        let pos = transform.position;
                        if pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y {
                            // Select this entity
                            selection_state.selected_entities.push(entity);
                            let mut entity_mut = world.entity_mut(entity);
                            entity_mut.insert(Selected);
                        }
                    }
                }
            },
            Command::Move(target) => {
                // Command selected units to move
                let target_pos = *target;
                
                for &entity in &selection_state.selected_entities {
                    if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                        if let Some(mut movement) = entity_mut.get_mut::<Movement>() {
                            // Set direct path to target (a real game would use pathfinding)
                            movement.path = vec![target_pos];
                            movement.path_index = 0;
                            
                            // Remove any attack target
                            entity_mut.remove::<AttackTarget>();
                        }
                    }
                }
            },
            Command::Attack(target) => {
                // Command selected units to attack move
                let target_pos = *target;
                
                // Find enemy entity near target position
                let mut nearest_enemy: Option<(bevy_ecs::entity::Entity, f32)> = None;
                
                let mut query = world.query::<(bevy_ecs::entity::Entity, &Transform, &Owner)>();
                
                // First, determine owner of selected units
                let mut player_id = None;
                for &entity in &selection_state.selected_entities {
                    if let Ok(owner) = world.get_entity(entity).unwrap().get::<Owner>() {
                        player_id = Some(owner.0);
                        break;
                    }
                }
                
                if let Some(pid) = player_id {
                    // Find nearest enemy
                    for (entity, transform, owner) in query.iter(world) {
                        if owner.0 != pid {
                            let distance = (transform.position - target_pos).length();
                            if nearest_enemy.is_none() || distance < nearest_enemy.unwrap().1 {
                                nearest_enemy = Some((entity, distance));
                            }
                        }
                    }
                }
                
                // Command units to attack or move
                for &entity in &selection_state.selected_entities {
                    if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                        // If enemy found, attack it
                        if let Some((enemy_entity, _)) = nearest_enemy {
                            entity_mut.insert(AttackTarget {
                                target_entity: enemy_entity,
                            });
                            
                            // Also set movement to target position
                            if let Some(mut movement) = entity_mut.get_mut::<Movement>() {
                                movement.path = vec![target_pos];
                                movement.path_index = 0;
                            }
                        } else {
                            // No enemy found, just move
                            if let Some(mut movement) = entity_mut.get_mut::<Movement>() {
                                movement.path = vec![target_pos];
                                movement.path_index = 0;
                            }
                        }
                    }
                }
            },
            // Handle other commands...
            _ => {}
        }
    }
}

/// Run all game systems for the current tick
fn run_game_systems(world: &mut bevy_ecs::world::World, game_state: &game::GameState) {
    // Create a schedule
    let mut schedule = bevy_ecs::schedule::Schedule::default();
    
    // Add systems to the schedule in the desired order
    schedule.add_system(ecs::systems::update_movement_system);
    schedule.add_system(ecs::systems::collision_detection_system);
    schedule.add_system(ecs::systems::unit_behavior_system);
    schedule.add_system(ecs::systems::building_production_system);
    schedule.add_system(ecs::systems::resource_collection_system);
    schedule.add_system(ecs::systems::economy_system);
    schedule.add_system(ecs::systems::fog_of_war_system);
    schedule.add_system(ecs::systems::combat::combat_system);
    
    // Run the schedule
    schedule.run(world);
    
    // Update global resources
    let mut game_time = world.resource_mut::<GameTime>();
    game_time.current_tick += 1;
    game_time.elapsed_time += game_time.delta_time;
}