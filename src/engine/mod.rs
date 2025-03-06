pub mod renderer;
pub mod input;
pub mod time;
pub mod audio;
pub mod assets;
pub mod camera;

use anyhow::Result;
use bevy_ecs::prelude::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::ecs;
use crate::ecs::init;
use crate::ecs::systems::combat::components::DamageTable;
use crate::ecs::systems::combat_system;
use crate::game::GameState;
use crate::networking::lockstep::LockstepNetwork;
use crate::ui::UiManager;

/// Main engine struct that coordinates all subsystems
pub struct Engine {
    window: Window,
    renderer: renderer::Renderer,
    input_handler: input::InputHandler,
    time_system: time::TimeSystem,
    asset_manager: assets::AssetManager,
    world: World,
    game_state: GameState,
    network: Option<LockstepNetwork>,
    ui_manager: UiManager,
}

impl Engine {
    pub async fn new(title: &str, width: u32, height: u32) -> Result<(Self, EventLoop<()>)> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(width as f64, height as f64))
            .build(&event_loop)?;
        
        // Initialize subsystems
        let renderer = renderer::Renderer::new(&window).await?;
        let input_handler = input::InputHandler::new();
        let time_system = time::TimeSystem::new(20.0); // 20 ticks per second
        
        // Initialize asset manager
        let asset_manager = assets::AssetManager::new(
            "assets",
            renderer.get_device().clone(),
            renderer.get_queue().clone(),
        );
        
        // Initialize ECS world
        let mut world = ecs::init::init_world();
        
        // Add combat-specific resources
        world.insert_resource(DamageTable::default());
        
        // Create game state
        let game_state = GameState::new();
        
        // Initialize UI manager
        let ui_manager = UiManager::new(
            renderer.get_device().clone(),
            renderer.get_queue().clone(),
            width,
            height,
            renderer.get_surface_format(),
        )?;
        
        Ok((
            Self {
                window,
                renderer,
                input_handler,
                time_system,
                asset_manager,
                world,
                game_state,
                network: None,
                ui_manager,
            },
            event_loop,
        ))
    }
    
    pub fn enable_networking(&mut self, is_host: bool, address: Option<&str>) -> Result<()> {
        let mut network = LockstepNetwork::new();
        
        if is_host {
            network.host_game(12345, "Host".to_string())?;
        } else if let Some(addr) = address {
            network.join_game(addr, "Client".to_string())?;
        } else {
            return Err(anyhow::anyhow!("Client mode requires a host address"));
        }
        
        self.network = Some(network);
        self.game_state.is_multiplayer = true;
        
        Ok(())
    }
    
    pub fn load_assets(&mut self) -> Result<()> {
        // Load textures
        self.asset_manager.load_texture("unit_worker", "units/worker.png")?;
        self.asset_manager.load_texture("unit_soldier", "units/soldier.png")?;
        self.asset_manager.load_texture("unit_scout", "units/scout.png")?;
        self.asset_manager.load_texture("unit_tank", "units/tank.png")?;
        self.asset_manager.load_texture("unit_healer", "units/healer.png")?;
        
        self.asset_manager.load_texture("building_hq", "buildings/headquarters.png")?;
        self.asset_manager.load_texture("building_barracks", "buildings/barracks.png")?;
        self.asset_manager.load_texture("building_factory", "buildings/factory.png")?;
        self.asset_manager.load_texture("building_resource", "buildings/resource_collector.png")?;
        self.asset_manager.load_texture("building_research", "buildings/research_center.png")?;
        self.asset_manager.load_texture("building_defense", "buildings/defense_tower.png")?;
        
        self.asset_manager.load_texture("terrain_ground", "terrain/ground.png")?;
        self.asset_manager.load_texture("terrain_water", "terrain/water.png")?;
        self.asset_manager.load_texture("terrain_mountain", "terrain/mountain.png")?;
        self.asset_manager.load_texture("terrain_forest", "terrain/forest.png")?;
        
        self.asset_manager.load_texture("resource_mineral", "resources/mineral.png")?;
        self.asset_manager.load_texture("resource_gas", "resources/gas.png")?;
        self.asset_manager.load_texture("resource_energy", "resources/energy.png")?;
        
        self.asset_manager.load_texture("effect_explosion", "effects/explosion.png")?;
        self.asset_manager.load_texture("effect_fire", "effects/fire.png")?;
        self.asset_manager.load_texture("effect_smoke", "effects/smoke.png")?;
        
        self.asset_manager.load_texture("ui_panel", "ui/panel.png")?;
        self.asset_manager.load_texture("ui_button", "ui/button.png")?;
        self.asset_manager.load_texture("ui_icons", "ui/icons.png")?;
        self.asset_manager.load_texture("ui_minimap_frame", "ui/minimap_frame.png")?;
        
        // Load sounds
        self.asset_manager.load_sound("sfx_click", "sfx/click.wav")?
        self.asset_manager.load_sound("sfx_click", "sfx/click.wav")?;
        self.asset_manager.load_sound("sfx_select", "sfx/select.wav")?;
        self.asset_manager.load_sound("sfx_move", "sfx/move.wav")?;
        self.asset_manager.load_sound("sfx_attack", "sfx/attack.wav")?;
        self.asset_manager.load_sound("sfx_build", "sfx/build.wav")?;
        self.asset_manager.load_sound("sfx_explosion", "sfx/explosion.wav")?;
        
        Ok(())
    }
    
    pub fn run(mut self, event_loop: EventLoop<()>) -> ! {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            
            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit;
                }
                
                Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                    self.renderer.resize(new_size);
                    self.ui_manager.resize(new_size.width, new_size.height);
                }
                
                Event::WindowEvent { event, .. } => {
                    // Forward window events to input handler
                    self.input_handler.handle_window_event(&event);
                    
                    // Handle UI input
                    if let WindowEvent::MouseInput { state: winit::event::ElementState::Released, button: winit::event::MouseButton::Left, .. } = event {
                        let mouse_pos = self.input_handler.get_mouse_position();
                        if self.ui_manager.handle_input(mouse_pos) {
                            // UI handled the click, no need to forward to game
                        }
                    }
                }
                
                Event::MainEventsCleared => {
                    // Process network messages if networking is enabled
                    if let Some(network) = &mut self.network {
                        if let Err(e) = network.process_messages() {
                            eprintln!("Network error: {}", e);
                        }
                    }
                    
                    // Tick game logic at fixed rate
                    while self.time_system.should_tick() {
                        // Only update if game is playing
                        if self.game_state.phase == crate::game::GamePhase::Playing {
                            // Process inputs
                            let commands = self.input_handler.get_commands();
                            
                            // Send commands to network if multiplayer
                            if let Some(network) = &mut self.network {
                                if let Err(e) = network.send_commands(&commands) {
                                    eprintln!("Error sending commands: {}", e);
                                }
                                
                                // Get commands from other players
                                let network_commands = network.receive_commands();
                                
                                // Process network commands
                                // (In a real implementation, you'd merge these with local commands)
                            }
                            
                            // Run ECS systems including combat
                            self.run_game_systems();
                            
                            // Update game state
                            self.game_state.update();
                            
                            // Update UI
                            self.ui_manager.update(&self.game_state);
                        }
                        
                        // Update time system
                        self.time_system.tick_completed();
                    }
                    
                    // Render current game state
                    self.render().unwrap_or_else(|e| {
                        eprintln!("Render error: {}", e);
                    });
                }
                
                _ => {}
            }
        })
    }
    
    fn run_game_systems(&mut self) {
        // Create a schedule
        let mut schedule = Schedule::default();
        
        // Add systems to the schedule in the desired order
        schedule.add_system(ecs::update_movement_system);
        schedule.add_system(ecs::collision_detection_system);
        schedule.add_system(ecs::unit_behavior_system);
        schedule.add_system(ecs::systems::building_production_system);
        schedule.add_system(ecs::systems::resource_collection_system);
        schedule.add_system(ecs::systems::economy_system);
        schedule.add_system(ecs::fog_of_war_system);
        schedule.add_system(combat_system);
        
        // Run the schedule
        schedule.run(&mut self.world);
        
        // Update global resources
        let mut game_time = self.world.resource_mut::<ecs::resources::GameTime>();
        game_time.current_tick += 1;
        game_time.elapsed_time += game_time.delta_time;
    }
    
    fn render(&mut self) -> Result<()> {
        // Render game world
        self.renderer.render(&self.world, &self.ui_manager)?;
        
        // Render UI on top
        // In a real implementation, this might be handled differently
        
        Ok(())
    }
}