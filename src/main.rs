mod engine;
mod ecs;
mod game;
mod networking;
mod ui;

use anyhow::Result;
use log::{info, error};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const TICK_RATE: f64 = 20.0; // 20 ticks per second
const MS_PER_TICK: f64 = 1000.0 / TICK_RATE;

fn main() -> Result<()> {
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
    let mut world = ecs::init_world();
    
    // Create game state
    let mut game_state = game::GameState::new();
    
    // Initialize network if needed
    let mut network = networking::lockstep::LockstepNetwork::new();
    
    // Game loop - using winit event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                info!("Window close requested");
                *control_flow = ControlFlow::Exit;
            }
            
            Event::WindowEvent { event, .. } => {
                // Handle window events and input
                input_handler.handle_window_event(&event);
            }
            
            Event::MainEventsCleared => {
                // Tick game logic at fixed rate, potentially multiple times per frame
                while time_system.should_tick() {
                    // Process inputs
                    let commands = input_handler.get_commands();
                    
                    // Network sync step
                    if network.is_active() {
                        network.send_commands(&commands);
                        network.receive_commands();
                    }
                    
                    // Run ECS systems
                    ecs::run_game_systems(&mut world, &game_state);
                    
                    // Advance game state
                    game_state.update();
                    
                    // Update time system
                    time_system.tick_completed();
                }
                
                // Render current game state (not tied to tick rate)
                match renderer.render(&world) {
                    Ok(_) => {}
                    Err(e) => error!("Render error: {}", e),
                }
            }
            
            _ => {}
        }
    });
}