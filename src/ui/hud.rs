use glam::{Vec2, Vec4};
use wgpu::RenderPass;
use std::collections::HashMap;

use crate::ecs::components::{UnitType, BuildingType, ResourceType};
use crate::game::GameState;
use crate::ui::{UiPipeline, UiElement, UiElementType};

/// Resource display for the HUD
struct ResourceDisplay {
    position: Vec2,
    size: Vec2,
    visible: bool,
    resources: HashMap<ResourceType, f32>,
}

/// Unit info panel for the HUD
struct UnitInfoPanel {
    position: Vec2,
    size: Vec2,
    visible: bool,
    selected_units: Vec<UnitInfo>,
}

/// Building info panel for the HUD
struct BuildingInfoPanel {
    position: Vec2,
    size: Vec2,
    visible: bool,
    selected_building: Option<BuildingInfo>,
}

/// Action buttons panel for the HUD
struct ActionPanel {
    position: Vec2,
    size: Vec2,
    visible: bool,
    buttons: Vec<ActionButton>,
}

/// Command card for the HUD
struct CommandCard {
    position: Vec2,
    size: Vec2,
    visible: bool,
    commands: Vec<CommandButton>,
}

/// Simple information about a selected unit
#[derive(Clone)]
struct UnitInfo {
    unit_type: UnitType,
    health: f32,
    max_health: f32,
    entity_id: u32,
}

/// Simple information about a selected building
struct BuildingInfo {
    building_type: BuildingType,
    health: f32,
    max_health: f32,
    entity_id: u32,
    production_progress: Option<f32>,
    construction_progress: Option<f32>,
}

/// Action button for unit/building commands
struct ActionButton {
    position: Vec2,
    size: Vec2,
    visible: bool,
    action_type: ActionType,
    enabled: bool,
    tooltip: String,
}

/// Command button for specific commands
struct CommandButton {
    position: Vec2,
    size: Vec2,
    visible: bool,
    command_type: CommandType,
    enabled: bool,
    tooltip: String,
}

/// Types of actions that can be performed
enum ActionType {
    Move,
    Attack,
    Stop,
    Hold,
    Patrol,
    Build(BuildingType),
    Train(UnitType),
    Research,
    Gather,
    Repair,
    Cancel,
}

/// Types of commands
enum CommandType {
    UseAbility(u8),
    SetRallyPoint,
    Upgrade,
    Special,
}

/// Main HUD class
pub struct Hud {
    resource_display: ResourceDisplay,
    unit_info_panel: UnitInfoPanel,
    building_info_panel: BuildingInfoPanel,
    action_panel: ActionPanel,
    command_card: CommandCard,
    screen_size: Vec2,
    visible: bool,
}

impl Hud {
    pub fn new() -> Self {
        Self {
            resource_display: ResourceDisplay {
                position: Vec2::new(10.0, 10.0),
                size: Vec2::new(200.0, 40.0),
                visible: true,
                resources: HashMap::new(),
            },
            unit_info_panel: UnitInfoPanel {
                position: Vec2::new(10.0, 60.0),
                size: Vec2::new(200.0, 100.0),
                visible: false,
                selected_units: Vec::new(),
            },
            building_info_panel: BuildingInfoPanel {
                position: Vec2::new(10.0, 60.0),
                size: Vec2::new(200.0, 100.0),
                visible: false,
                selected_building: None,
            },
            action_panel: ActionPanel {
                position: Vec2::new(220.0, 60.0),
                size: Vec2::new(300.0, 100.0),
                visible: false,
                buttons: Vec::new(),
            },
            command_card: CommandCard {
                position: Vec2::new(530.0, 60.0),
                size: Vec2::new(200.0, 100.0),
                visible: false,
                commands: Vec::new(),
            },
            screen_size: Vec2::new(800.0, 600.0),
            visible: true,
        }
    }
    
    pub fn update(&mut self, game_state: &GameState) {
        // Update resource display
        for (&(player_id, resource_type), &amount) in &game_state.player_resources {
            if player_id == 0 { // Local player
                self.resource_display.resources.insert(resource_type, amount);
            }
        }
        
        // Update panels based on selection state
        // In a real implementation, this would use the ECS world to get info about selected entities
    }
    
    pub fn set_selected_units(&mut self, units: Vec<UnitInfo>) {
        self.unit_info_panel.selected_units = units;
        self.unit_info_panel.visible = !units.is_empty();
        self.building_info_panel.visible = false;
        
        // Update action panel based on selection
        self.update_action_panel();
    }
    
    pub fn set_selected_building(&mut self, building: Option<BuildingInfo>) {
        self.building_info_panel.selected_building = building;
        self.building_info_panel.visible = building.is_some();
        self.unit_info_panel.visible = false;
        
        // Update action panel based on selection
        self.update_action_panel();
    }
    
    fn update_action_panel(&mut self) {
        // Clear current buttons
        self.action_panel.buttons.clear();
        
        // Create buttons based on selection
        if self.unit_info_panel.visible {
            // Common unit actions
            self.action_panel.buttons.push(ActionButton {
                position: Vec2::new(0.0, 0.0), // Relative to panel
                size: Vec2::new(32.0, 32.0),
                visible: true,
                action_type: ActionType::Move,
                enabled: true,
                tooltip: "Move".to_string(),
            });
            
            self.action_panel.buttons.push(ActionButton {
                position: Vec2::new(36.0, 0.0), // Relative to panel
                size: Vec2::new(32.0, 32.0),
                visible: true,
                action_type: ActionType::Attack,
                enabled: true,
                tooltip: "Attack".to_string(),
            });
            
            self.action_panel.buttons.push(ActionButton {
                position: Vec2::new(72.0, 0.0), // Relative to panel
                size: Vec2::new(32.0, 32.0),
                visible: true,
                action_type: ActionType::Stop,
                enabled: true,
                tooltip: "Stop".to_string(),
            });
            
            // Check if any unit is a worker
            let has_worker = self.unit_info_panel.selected_units.iter()
                .any(|unit| unit.unit_type == UnitType::Worker);
            
            if has_worker {
                self.action_panel.buttons.push(ActionButton {
                    position: Vec2::new(0.0, 36.0), // Relative to panel
                    size: Vec2::new(32.0, 32.0),
                    visible: true,
                    action_type: ActionType::Build(BuildingType::Barracks),
                    enabled: true,
                    tooltip: "Build Barracks".to_string(),
                });
                
                self.action_panel.buttons.push(ActionButton {
                    position: Vec2::new(36.0, 36.0), // Relative to panel
                    size: Vec2::new(32.0, 32.0),
                    visible: true,
                    action_type: ActionType::Gather,
                    enabled: true,
                    tooltip: "Gather Resources".to_string(),
                });
            }
        } else if self.building_info_panel.visible {
            // Building actions
            if let Some(ref building) = self.building_info_panel.selected_building {
                match building.building_type {
                    BuildingType::Headquarters => {
                        self.action_panel.buttons.push(ActionButton {
                            position: Vec2::new(0.0, 0.0), // Relative to panel
                            size: Vec2::new(32.0, 32.0),
                            visible: true,
                            action_type: ActionType::Research,
                            enabled: true,
                            tooltip: "Research Technology".to_string(),
                        });
                    }
                    _ => {}
                }
                
                // For buildings under construction, add cancel button
                if building.construction_progress.is_some() {
                    self.action_panel.buttons.push(ActionButton {
                        position: Vec2::new(0.0, 72.0), // Relative to panel
                        size: Vec2::new(32.0, 32.0),
                        visible: true,
                        action_type: ActionType::Cancel,
                        enabled: true,
                        tooltip: "Cancel Construction".to_string(),
                    });
                }
            }
        }
        
        // Set action panel visibility based on buttons
        self.action_panel.visible = !self.action_panel.buttons.is_empty();
    }
    
    pub fn handle_input(&mut self, position: Vec2) -> bool {
        // Check if any action button was clicked
        if self.action_panel.visible {
            for button in &self.action_panel.buttons {
                if button.visible && button.enabled {
                    let absolute_pos = self.action_panel.position + button.position;
                    if position.x >= absolute_pos.x && 
                       position.x <= absolute_pos.x + button.size.x &&
                       position.y >= absolute_pos.y && 
                       position.y <= absolute_pos.y + button.size.y {
                        // Button was clicked, handle the action
                        return self.handle_action(&button.action_type);
                    }
                }
            }
        }
        
        // Check if any command button was clicked
        if self.command_card.visible {
            for button in &self.command_card.commands {
                if button.visible && button.enabled {
                    let absolute_pos = self.command_card.position + button.position;
                    if position.x >= absolute_pos.x && 
                       position.x <= absolute_pos.x + button.size.x &&
                       position.y >= absolute_pos.y && 
                       position.y <= absolute_pos.y + button.size.y {
                        // Button was clicked, handle the command
                        return self.handle_command(&button.command_type);
                    }
                }
            }
        }
        
        false
    }
    
    fn handle_action(&self, action_type: &ActionType) -> bool {
        // In a real implementation, this would issue the corresponding command
        // to the game systems
        match action_type {
            ActionType::Move => {
                // Set mode to move command
                println!("Move command selected");
            }
            ActionType::Attack => {
                // Set mode to attack command
                println!("Attack command selected");
            }
            ActionType::Stop => {
                // Issue stop command to selected units
                println!("Stop command issued");
            }
            ActionType::Hold => {
                // Issue hold position command
                println!("Hold position command issued");
            }
            ActionType::Patrol => {
                // Set mode to patrol command
                println!("Patrol command selected");
            }
            ActionType::Build(building_type) => {
                // Set mode to build specified building
                println!("Build {:?} command selected", building_type);
            }
            ActionType::Train(unit_type) => {
                // Queue unit for training
                println!("Train {:?} command issued", unit_type);
            }
            ActionType::Research => {
                // Open research menu
                println!("Research menu opened");
            }
            ActionType::Gather => {
                // Set mode to gather resources
                println!("Gather command selected");
            }
            ActionType::Repair => {
                // Set mode to repair
                println!("Repair command selected");
            }
            ActionType::Cancel => {
                // Cancel current construction/training
                println!("Cancel command issued");
            }
        }
        
        true
    }
    
    fn handle_command(&self, command_type: &CommandType) -> bool {
        // In a real implementation, this would issue the corresponding command
        match command_type {
            CommandType::UseAbility(ability_id) => {
                // Use the specified ability
                println!("Using ability {}", ability_id);
            }
            CommandType::SetRallyPoint => {
                // Set mode to specify rally point
                println!("Set rally point command selected");
            }
            CommandType::Upgrade => {
                // Open upgrade menu
                println!("Upgrade menu opened");
            }
            CommandType::Special => {
                // Trigger special ability
                println!("Special ability triggered");
            }
        }
        
        true
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_size = Vec2::new(width as f32, height as f32);
        
        // Position resource display at top left
        self.resource_display.position = Vec2::new(10.0, 10.0);
        
        // Position panels at bottom of screen
        let panel_y = height as f32 - 110.0;
        self.unit_info_panel.position = Vec2::new(10.0, panel_y);
        self.building_info_panel.position = Vec2::new(10.0, panel_y);
        self.action_panel.position = Vec2::new(220.0, panel_y);
        self.command_card.position = Vec2::new(530.0, panel_y);
    }
    
    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        if !self.visible {
            return;
        }
        
        // Render resource display
        if self.resource_display.visible {
            self.render_resource_display(render_pass, ui_pipeline);
        }
        
        // Render unit info panel
        if self.unit_info_panel.visible {
            self.render_unit_info_panel(render_pass, ui_pipeline);
        }
        
        // Render building info panel
        if self.building_info_panel.visible {
            self.render_building_info_panel(render_pass, ui_pipeline);
        }
        
        // Render action panel
        if self.action_panel.visible {
            self.render_action_panel(render_pass, ui_pipeline);
        }
        
        // Render command card
        if self.command_card.visible {
            self.render_command_card(render_pass, ui_pipeline);
        }
    }
    
    fn render_resource_display<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render the resource counters
        // using the UI pipeline, textures, and text
    }
    
    fn render_unit_info_panel<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render the unit info panel
    }
    
    fn render_building_info_panel<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render the building info panel
    }
    
    fn render_action_panel<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render all action buttons
    }
    
    fn render_command_card<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render all command buttons
    }
}

                            action_type: ActionType::Train(UnitType::Worker),
                            enabled: true,
                            tooltip: "Train Worker".to_string(),
                        });
                    }
                    BuildingType::Barracks => {
                        self.action_panel.buttons.push(ActionButton {
                            position: Vec2::new(0.0, 0.0), // Relative to panel
                            size: Vec2::new(32.0, 32.0),
                            visible: true,
                            action_type: ActionType::Train(UnitType::Soldier),
                            enabled: true,
                            tooltip: "Train Soldier".to_string(),
                        });
                        
                        self.action_panel.buttons.push(ActionButton {
                            position: Vec2::new(36.0, 0.0), // Relative to panel
                            size: Vec2::new(32.0, 32.0),
                            visible: true,
                            action_type: ActionType::Train(UnitType::Scout),
                            enabled: true,
                            tooltip: "Train Scout".to_string(),
                        });
                    }
                    BuildingType::Factory => {
                        self.action_panel.buttons.push(ActionButton {
                            position: Vec2::new(0.0, 0.0), // Relative to panel
                            size: Vec2::new(32.0, 32.0),
                            visible: true,
                            action_type: ActionType::Train(UnitType::Tank),
                            enabled: true,
                            tooltip: "Train Tank".to_string(),
                        });
                    }
                    BuildingType::ResearchCenter => {
                        self.action_panel.buttons.push(ActionButton {
                            position: Vec2::new(0.0, 0.0), // Relative to panel
                            size: Vec2::new(32.0, 32.0),
                            visible: true,