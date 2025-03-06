use bevy_ecs::prelude::*;
use glam::Vec2;
use std::collections::{HashMap, VecDeque};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::ecs::components::{UnitType, BuildingType, ResourceType, Transform, Owner, Unit, Building};
use crate::ecs::resources::{GameMap, PlayerResources};
use crate::engine::input::Command;

/// AI difficulty level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiDifficulty {
    Easy,
    Medium,
    Hard,
}

/// AI personality type that affects strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiPersonality {
    Rusher,    // Aggressive early game
    Boomer,    // Economy focused
    Techer,    // Tech focused
    Balanced,  // Mix of strategies
}

/// Main AI controller for a computer player
pub struct AiController {
    player_id: u8,
    difficulty: AiDifficulty,
    personality: AiPersonality,
    rng: StdRng,
    
    // Strategy state
    build_order: VecDeque<AiBuildTask>,
    attack_squads: Vec<AiSquad>,
    defense_squads: Vec<AiSquad>,
    economy_state: AiEconomyState,
    
    // Timers
    decision_timer: f32,
    scout_timer: f32,
    attack_timer: f32,
}

#[derive(Debug, Clone)]
enum AiBuildTask {
    BuildUnit(UnitType),
    BuildBuilding(BuildingType, Option<Vec2>),
    Research(crate::ecs::resources::TechType),
}

#[derive(Debug)]
struct AiSquad {
    units: Vec<Entity>,
    role: SquadRole,
    target: Option<Vec2>,
    state: SquadState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SquadRole {
    Attack,
    Defense,
    Scout,
    Harass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SquadState {
    Forming,
    Moving,
    Attacking,
    Retreating,
}

#[derive(Debug)]
struct AiEconomyState {
    desired_workers: u32,
    desired_bases: u32,
    current_workers: u32,
    current_bases: u32,
    resource_targets: HashMap<ResourceType, u32>,
}

impl AiController {
    pub fn new(player_id: u8, difficulty: AiDifficulty, personality: AiPersonality, seed: u64) -> Self {
        let rng = StdRng::seed_from_u64(seed + player_id as u64);
        
        // Initialize with different build orders based on personality
        let build_order = match personality {
            AiPersonality::Rusher => vec![
                AiBuildTask::BuildUnit(UnitType::Worker),
                AiBuildTask::BuildUnit(UnitType::Worker),
                AiBuildTask::BuildBuilding(BuildingType::Barracks, None),
                AiBuildTask::BuildUnit(UnitType::Soldier),
                AiBuildTask::BuildUnit(UnitType::Soldier),
                AiBuildTask::BuildUnit(UnitType::Soldier),
            ],
            AiPersonality::Boomer => vec![
                AiBuildTask::BuildUnit(UnitType::Worker),
                AiBuildTask::BuildUnit(UnitType::Worker),
                AiBuildTask::BuildUnit(UnitType::Worker),
                AiBuildTask::BuildBuilding(BuildingType::ResourceCollector, None),
                AiBuildTask::BuildUnit(UnitType::Worker),
                AiBuildTask::BuildUnit(UnitType::Worker),
            ],
            AiPersonality::Techer => vec![
                AiBuildTask::BuildUnit(UnitType::Worker),
                AiBuildTask::BuildUnit(UnitType::Worker),
                AiBuildTask::BuildBuilding(BuildingType::ResearchCenter, None),
                AiBuildTask::Research(crate::ecs::resources::TechType::ImprovedHarvesting),
                AiBuildTask::BuildUnit(UnitType::Worker),
            ],
            AiPersonality::Balanced => vec![
                AiBuildTask::BuildUnit(UnitType::Worker),
                AiBuildTask::BuildUnit(UnitType::Worker),
                AiBuildTask::BuildBuilding(BuildingType::Barracks, None),
                AiBuildTask::BuildUnit(UnitType::Worker),
                AiBuildTask::BuildUnit(UnitType::Soldier),
            ],
        };
        
        Self {
            player_id,
            difficulty,
            personality,
            rng,
            build_order: build_order.into(),
            attack_squads: Vec::new(),
            defense_squads: Vec::new(),
            economy_state: AiEconomyState {
                desired_workers: 10,
                desired_bases: 1,
                current_workers: 0,
                current_bases: 1,
                resource_targets: HashMap::new(),
            },
            decision_timer: 0.0,
            scout_timer: 0.0,
            attack_timer: 0.0,
        }
    }
    
    // Main update function called each game tick
    pub fn update(
        &mut self,
        world: &World,
        elapsed_time: f32,
        delta_time: f32,
    ) -> Vec<Command> {
        let mut commands = Vec::new();
        
        // Update timers
        self.decision_timer += delta_time;
        self.scout_timer += delta_time;
        self.attack_timer += delta_time;
        
        // Make decisions at fixed intervals (based on difficulty)
        let decision_interval = match self.difficulty {
            AiDifficulty::Easy => 2.0,    // Slower decisions
            AiDifficulty::Medium => 1.0,
            AiDifficulty::Hard => 0.5,    // Faster decisions
        };
        
        if self.decision_timer >= decision_interval {
            self.decision_timer = 0.0;
            
            // Update economy state
            self.update_economy_state(world);
            
            // Execute build order if possible
            if let Some(commands_to_issue) = self.process_build_order(world) {
                commands.extend(commands_to_issue);
            }
            
            // Update squad assignments
            self.update_squads(world);
            
            // Issue squad commands
            commands.extend(self.command_squads(world));
            
            // Decide on next strategic moves
            self.update_strategy(world, elapsed_time);
        }
        
        // Check if it's time to scout
        if self.scout_timer >= 30.0 {
            self.scout_timer = 0.0;
            
            // Find a good spot to scout
            if let Some(scout_pos) = self.choose_scout_target(world) {
                commands.push(Command::Move(scout_pos));
            }
        }
        
        // Check if it's time to attack
        if self.attack_timer >= 60.0 && self.personality == AiPersonality::Rusher {
            self.attack_timer = 0.0;
            
            // Launch attack if we have enough units
            if self.attack_squads.len() > 0 {
                // Find a target to attack
                if let Some(target_pos) = self.choose_attack_target(world) {
                    commands.push(Command::Attack(target_pos));
                }
            }
        }
        
        commands
    }
    
    // Update the economy tracking
    fn update_economy_state(&mut self, world: &World) {
        // Count our units and buildings
        self.economy_state.current_workers = 0;
        self.economy_state.current_bases = 0;
        
        // Query for our units
        let mut query = world.query::<(&Unit, &Owner)>();
        for (unit, owner) in query.iter(world) {
            if owner.0 == self.player_id {
                if unit.unit_type == UnitType::Worker {
                    self.economy_state.current_workers += 1;
                }
            }
        }
        
        // Query for our buildings
        let mut building_query = world.query::<(&Building, &Owner)>();
        for (building, owner) in building_query.iter(world) {
            if owner.0 == self.player_id {
                if building.building_type == BuildingType::Headquarters {
                    self.economy_state.current_bases += 1;
                }
            }
        }
        
        // Adjust desired worker count based on bases and personality
        match self.personality {
            AiPersonality::Boomer => {
                self.economy_state.desired_workers = self.economy_state.current_bases * 15;
            }
            AiPersonality::Rusher => {
                self.economy_state.desired_workers = self.economy_state.current_bases * 8;
            }
            _ => {
                self.economy_state.desired_workers = self.economy_state.current_bases * 12;
            }
        }
    }
    
    // Process the next item in the build order
    fn process_build_order(&mut self, world: &World) -> Option<Vec<Command>> {
        if self.build_order.is_empty() {
            // Generate a new task if build order is empty
            self.generate_next_task();
        }
        
        // Peek at the next task
        if let Some(task) = self.build_order.front() {
            match task {
                AiBuildTask::BuildUnit(unit_type) => {
                    // Check if we can afford this unit
                    if self.can_afford_unit(*unit_type, world) {
                        // Find a building that can produce this unit
                        if let Some(building_entity) = self.find_production_building(*unit_type, world) {
                            // Remove the task from the queue
                            self.build_order.pop_front();
                            
                            // Return command to build the unit
                            return Some(vec![Command::Train(crate::engine::input::UnitCommand {
                                unit_type: *unit_type as u8,
                            })]);
                        }
                    }
                }
                
                AiBuildTask::BuildBuilding(building_type, position) => {
                    // Check if we can afford this building
                    if self.can_afford_building(*building_type, world) {
                        // Find a position to build if none specified
                        let build_pos = position.unwrap_or_else(|| self.find_building_position(*building_type, world));
                        
                        // Remove the task from the queue
                        self.build_order.pop_front();
                        
                        // Return command to build the building
                        return Some(vec![Command::Build(crate::engine::input::BuildingCommand {
                            building_type: *building_type as u8,
                            position: build_pos,
                        })]);
                    }
                }
                
                AiBuildTask::Research(tech_type) => {
                    // Check if we can afford this research
                    if self.can_afford_research(*tech_type, world) {
                        // Find a research building
                        if let Some(_research_building) = self.find_research_building(world) {
                            // Remove the task from the queue
                            self.build_order.pop_front();
                            
                            // Return command to research (would need a proper command for this)
                            return Some(vec![Command::UseAbility(crate::engine::input::AbilityCommand {
                                ability_id: *tech_type as u8,
                                target_position: None,
                                target_entity_id: None,
                            })]);
                        }
                    }
                }
            }
        }
        
        None
    }
    
    // Generate the next strategic task
    fn generate_next_task(&mut self) {
        // Different logic based on personality and current state
        if self.economy_state.current_workers < self.economy_state.desired_workers {
            // Need more workers
            self.build_order.push_back(AiBuildTask::BuildUnit(UnitType::Worker));
        } else if self.economy_state.current_bases < self.economy_state.desired_bases {
            // Need more bases
            self.build_order.push_back(AiBuildTask::BuildBuilding(BuildingType::Headquarters, None));
        } else {
            // Build military
            match self.personality {
                AiPersonality::Rusher => {
                    // Rushers favor basic military units
                    let unit_choice = if self.rng.gen_bool(0.7) {
                        UnitType::Soldier
                    } else {
                        UnitType::Scout
                    };
                    self.build_order.push_back(AiBuildTask::BuildUnit(unit_choice));
                }
                
                AiPersonality::Boomer => {
                    // Boomers focus on economy and defenses
                    if self.rng.gen_bool(0.3) {
                        self.build_order.push_back(AiBuildTask::BuildUnit(UnitType::Worker));
                    } else if self.rng.gen_bool(0.5) {
                        self.build_order.push_back(AiBuildTask::BuildBuilding(BuildingType::DefenseTower, None));
                    } else {
                        self.build_order.push_back(AiBuildTask::BuildUnit(UnitType::Soldier));
                    }
                }
                
                AiPersonality::Techer => {
                    // Techers focus on advanced units and research
                    if self.rng.gen_bool(0.4) {
                        self.build_order.push_back(AiBuildTask::BuildUnit(UnitType::Tank));
                    } else if self.rng.gen_bool(0.3) {
                        self.build_order.push_back(AiBuildTask::Research(
                            crate::ecs::resources::TechType::ImprovedWeapons
                        ));
                    } else {
                        self.build_order.push_back(AiBuildTask::BuildUnit(UnitType::Soldier));
                    }
                }
                
                AiPersonality::Balanced => {
                    // Balanced approach
                    let roll = self.rng.gen_range(0..10);
                    match roll {
                        0..=3 => self.build_order.push_back(AiBuildTask::BuildUnit(UnitType::Soldier)),
                        4..=5 => self.build_order.push_back(AiBuildTask::BuildUnit(UnitType::Tank)),
                        6 => self.build_order.push_back(AiBuildTask::BuildUnit(UnitType::Worker)),
                        7 => self.build_order.push_back(AiBuildTask::BuildBuilding(BuildingType::DefenseTower, None)),
                        8 => self.build_order.push_back(AiBuildTask::Research(
                            crate::ecs::resources::TechType::ImprovedWeapons
                        )),
                        _ => self.build_order.push_back(AiBuildTask::BuildUnit(UnitType::Scout)),
                    }
                }
            }
        }
    }
    
    // Check if we can afford a unit
    fn can_afford_unit(&self, unit_type: UnitType, world: &World) -> bool {
        // In a real game, this would check actual costs against current resources
        // Simplified version for this example
        true
    }
    
    // Check if we can afford a building
    fn can_afford_building(&self, building_type: BuildingType, world: &World) -> bool {
        // In a real game, this would check actual costs against current resources
        // Simplified version for this example
        true
    }
    
    // Check if we can afford research
    fn can_afford_research(&self, tech_type: crate::ecs::resources::TechType, world: &World) -> bool {
        // In a real game, this would check actual costs against current resources
        // Simplified version for this example
        true
    }
    
    // Find a building that can produce a unit type
    fn find_production_building(&self, unit_type: UnitType, world: &World) -> Option<Entity> {
        // In a real game, this would check for valid production buildings
        // Simplified version for this example
        None
    }
    
    // Find a research building
    fn find_research_building(&self, world: &World) -> Option<Entity> {
        // In a real game, this would find a research center
        // Simplified version for this example
        None
    }
    
    // Find a suitable position for a new building
    fn find_building_position(&self, building_type: BuildingType, world: &World) -> Vec2 {
        // In a real game, this would use pathfinding to find valid build locations
        // Simplified placeholder
        Vec2::new(100.0, 100.0)
    }
    
    // Update squad assignments for military units
    fn update_squads(&mut self, world: &World) {
        // In a real game, this would organize units into tactical squads
        // Simplified version for this example
    }
    
    // Command squads to move, attack, etc.
    fn command_squads(&self, world: &World) -> Vec<Command> {
        // Issue commands to each squad based on their role and state
        // Simplified version for this example
        Vec::new()
    }
    
    // Update the overall strategy
    fn update_strategy(&mut self, world: &World, elapsed_time: f32) {
        // In a real game, this would adjust strategy based on game state
        // Simplified version for this example
        
        // Adjust desired bases as game progresses
        if elapsed_time > 300.0 && self.economy_state.desired_bases < 2 {
            self.economy_state.desired_bases = 2;
        }
        
        if elapsed_time > 600.0 && self.economy_state.desired_bases < 3 {
            self.economy_state.desired_bases = 3;
        }
    }
    
    // Choose a location to scout
    fn choose_scout_target(&self, world: &World) -> Option<Vec2> {
        // In a real game, this would look for unexplored areas or enemy bases
        // Simplified placeholder
        Some(Vec2::new(self.rng.gen_range(100.0..900.0), self.rng.gen_range(100.0..700.0)))
    }
    
    // Choose a target to attack
    fn choose_attack_target(&self, world: &World) -> Option<Vec2> {
        // In a real game, this would find enemy buildings or units to attack
        // Simplified placeholder
        Some(Vec2::new(self.rng.gen_range(100.0..900.0), self.rng.gen_range(100.0..700.0)))
    }
}