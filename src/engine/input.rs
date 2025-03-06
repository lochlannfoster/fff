use bevy_ecs::prelude::*;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    MoveCamera(Vec2),
    ZoomCamera(f32),
    Select(Vec2),
    MultiSelect(Vec2, Vec2),
    Move(Vec2),
    Attack(Vec2),
    Build(BuildingCommand),
    CancelBuild,
    Train(UnitCommand),
    CancelTrain,
    UseAbility(AbilityCommand),
    Gather(Vec2),
    Patrol(Vec2, Vec2),
    Stop,
    SetRallyPoint(Vec2),
    GroupAssign(u8),
    GroupSelect(u8),
    Pause,
    Resume,
    
    // New commands for enhanced worker control
    RepairBuilding(Entity),
    BuildBuilding {
        building_type: BuildingType,
        position: Vec2,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingCommand {
    pub building_type: u8,
    pub position: Vec2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitCommand {
    pub unit_type: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityCommand {
    pub ability_id: u8,
    pub target_position: Option<Vec2>,
    pub target_entity_id: Option<u32>,
}

pub struct InputHandler {
    camera_position: Vec2,
    camera_zoom: f32,
    mouse_position: Vec2,
    left_mouse_down: bool,
    right_mouse_down: bool,
    selection_start: Option<Vec2>,
    keys_down: HashSet<VirtualKeyCode>,
    pending_commands: Vec<Command>,
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            camera_position: Vec2::ZERO,
            camera_zoom: 1.0,
            mouse_position: Vec2::ZERO,
            left_mouse_down: false,
            right_mouse_down: false,
            selection_start: None,
            keys_down: HashSet::new(),
            pending_commands: Vec::new(),
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
        }
    }
    
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = Vec2::new(position.x as f32, position.y as f32);
                
                // If left mouse is down and we have a selection start, this is a drag
                if self.left_mouse_down && self.selection_start.is_some() {
                    // Update UI for selection rectangle, but don't issue command yet
                }
                
                // Camera movement via edge scrolling
                let edge_scroll_margin = 20.0;
                let scroll_speed = 5.0;
                let mut scroll_dir = Vec2::ZERO;
                
                if self.mouse_position.x < edge_scroll_margin {
                    scroll_dir.x = -1.0;
                } else if self.mouse_position.x > 1024.0 - edge_scroll_margin {
                    scroll_dir.x = 1.0;
                }
                
                if self.mouse_position.y < edge_scroll_margin {
                    scroll_dir.y = -1.0;
                } else if self.mouse_position.y > 768.0 - edge_scroll_margin {
                    scroll_dir.y = 1.0;
                }
                
                if scroll_dir != Vec2::ZERO {
                    self.camera_position += scroll_dir * scroll_speed;
                    self.pending_commands.push(Command::MoveCamera(scroll_dir * scroll_speed));
                }
            }
            
            WindowEvent::MouseInput { state, button, .. } => {
                match button {
                    MouseButton::Left => {
                        match state {
                            ElementState::Pressed => {
                                self.left_mouse_down = true;
                                self.selection_start = Some(self.mouse_position);
                            }
                            ElementState::Released => {
                                self.left_mouse_down = false;
                                
                                if let Some(start) = self.selection_start {
                                    // Check if this was a click or a drag
                                    let drag_threshold = 5.0;
                                    if (start - self.mouse_position).length_squared() < drag_threshold * drag_threshold {
                                        // This was a click
                                        self.pending_commands.push(Command::Select(self.mouse_position));
                                    } else {
                                        // This was a drag - multi-select
                                        self.pending_commands.push(Command::MultiSelect(start, self.mouse_position));
                                    }
                                }
                                
                                self.selection_start = None;
                            }
                        }
                    }
                    
                    MouseButton::Right => {
                        match state {
                            ElementState::Pressed => {
                                self.right_mouse_down = true;
                            }
                            ElementState::Released => {
                                self.right_mouse_down = false;
                                
                                // Right click gives move or attack command depending on context
                                if self.shift_pressed {
                                    // Queue command
                                    if self.alt_pressed {
                                        // Alt+right click = attack move
                                        self.pending_commands.push(Command::Attack(self.mouse_position));
                                    } else {
                                        // Shift+right click = queue move
                                        self.pending_commands.push(Command::Move(self.mouse_position));
                                    }
                                } else {
                                    // Direct command
                                    if self.alt_pressed {
                                        // Alt+right click = attack move
                                        self.pending_commands.push(Command::Attack(self.mouse_position));
                                    } else {
                                        // Right click = move or gather depending on target
                                        self.pending_commands.push(Command::Move(self.mouse_position));
                                    }
                                }
                            }
                        }
                    }
                    
                    _ => {}
                }
            }
            
            WindowEvent::MouseWheel { delta, .. } => {
                let zoom_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 0.1,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.001,
                };
                
                self.camera_zoom = (self.camera_zoom + zoom_delta).max(0.5).min(2.0);
                self.pending_commands.push(Command::ZoomCamera(zoom_delta));
            }
            
            WindowEvent::KeyboardInput { input, .. } => {
                self.handle_keyboard_input(input);
            }
            
            _ => {}
        }
    }
    
    fn handle_keyboard_input(&mut self, input: &KeyboardInput) {
        if let Some(keycode) = input.virtual_keycode {
            match input.state {
                ElementState::Pressed => {
                    // Add key to pressed keys
                    self.keys_down.insert(keycode);
                    
                    // Update modifier key states
                    match keycode {
                        VirtualKeyCode::LShift | VirtualKeyCode::RShift => self.shift_pressed = true,
                        VirtualKeyCode::LControl | VirtualKeyCode::RControl => self.ctrl_pressed = true,
                        VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => self.alt_pressed = true,
                        _ => {}
                    }
                    
                    // Process key presses
                    match keycode {
                        // Camera controls
                        VirtualKeyCode::W => self.pending_commands.push(Command::MoveCamera(Vec2::new(0.0, -10.0))),
                        VirtualKeyCode::S => self.pending_commands.push(Command::MoveCamera(Vec2::new(0.0, 10.0))),
                        VirtualKeyCode::A => self.pending_commands.push(Command::MoveCamera(Vec2::new(-10.0, 0.0))),
                        VirtualKeyCode::D => self.pending_commands.push(Command::MoveCamera(Vec2::new(10.0, 0.0))),
                        
                        // Group controls
                        VirtualKeyCode::Key1 if self.ctrl_pressed => self.pending_commands.push(Command::GroupAssign(0)),
                        VirtualKeyCode::Key2 if self.ctrl_pressed => self.pending_commands.push(Command::GroupAssign(1)),
                        VirtualKeyCode::Key3 if self.ctrl_pressed => self.pending_commands.push(Command::GroupAssign(2)),
                        VirtualKeyCode::Key4 if self.ctrl_pressed => self.pending_commands.push(Command::GroupAssign(3)),
                        VirtualKeyCode::Key5 if self.ctrl_pressed => self.pending_commands.push(Command::GroupAssign(4)),
                        
                        VirtualKeyCode::Key1 if !self.ctrl_pressed => self.pending_commands.push(Command::GroupSelect(0)),
                        VirtualKeyCode::Key2 if !self.ctrl_pressed => self.pending_commands.push(Command::GroupSelect(1)),
                        VirtualKeyCode::Key3 if !self.ctrl_pressed => self.pending_commands.push(Command::GroupSelect(2)),
                        VirtualKeyCode::Key4 if !self.ctrl_pressed => self.pending_commands.push(Command::GroupSelect(3)),
                        VirtualKeyCode::Key5 if !self.ctrl_pressed => self.pending_commands.push(Command::GroupSelect(4)),
                        
                        // Game commands
                        VirtualKeyCode::Escape => self.pending_commands.push(Command::CancelBuild),
                        VirtualKeyCode::Space => self.pending_commands.push(Command::Pause),
                        VirtualKeyCode::S if self.ctrl_pressed => self.pending_commands.push(Command::Stop),
                        
                        _ => {}
                    }
                }
                
                ElementState::Released => {
                    // Remove key from pressed keys
                    self.keys_down.remove(&keycode);
                    
                    // Update modifier key states
                    match keycode {
                        VirtualKeyCode::LShift | VirtualKeyCode::RShift => self.shift_pressed = false,
                        VirtualKeyCode::LControl | VirtualKeyCode::RControl => self.ctrl_pressed = false,
                        VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => self.alt_pressed = false,
                        _ => {}
                    }
                }
            }
        }
    }
    
    pub fn get_commands(&mut self) -> Vec<Command> {
        std::mem::take(&mut self.pending_commands)
    }
    
    pub fn get_mouse_position(&self) -> Vec2 {
        self.mouse_position
    }
    
    pub fn get_camera_position(&self) -> Vec2 {
        self.camera_position
    }
    
    pub fn get_camera_zoom(&self) -> f32 {
        self.camera_zoom
    }
    
    pub fn is_selection_active(&self) -> bool {
        self.selection_start.is_some() && self.left_mouse_down
    }
    
    pub fn get_selection_rectangle(&self) -> Option<(Vec2, Vec2)> {
        self.selection_start.map(|start| (start, self.mouse_position))
    }
    
    pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.keys_down.contains(&key)
    }
    
    pub fn handle_command(&mut self, command: Command) {
        self.pending_commands.push(command);
    }
}