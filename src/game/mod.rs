use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use glam::Vec2;

use crate::ecs::components::{UnitType, BuildingType, ResourceType};

/// Game state enum to track which phase the game is in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    MainMenu,
    Loading,
    Playing,
    Paused,
    GameOver,
}

/// Primary game state container
#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
    pub phase: GamePhase,
    pub current_tick: u64,
    pub is_multiplayer: bool,
    pub winner: Option<u8>,
    pub player_count: u8,
    pub seed: u64,
    pub game_speed: f32,
    
    // Player-specific state
    pub player_resources: HashMap<(u8, ResourceType), f32>,
    pub player_supply: HashMap<u8, (u32, u32)>, // (current, max) supply
    pub player_scores: HashMap<u8, u32>,
    pub settings: GameSettings,
}

/// Game settings
#[derive(Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub fog_of_war_enabled: bool,
    pub game_speed: f32,
    pub auto_save_enabled: bool,
    pub auto_save_interval: f32,
    pub show_fps: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fog_of_war_enabled: true,
            game_speed: 1.0,
            auto_save_enabled: false,
            auto_save_interval: 300.0, // 5 minutes
            show_fps: false,
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        let mut player_resources = HashMap::new();
        let mut player_supply = HashMap::new();
        let mut player_scores = HashMap::new();
        
        // Initialize default resources for 2 players
        player_resources.insert((0, ResourceType::Mineral), 500.0);
        player_resources.insert((0, ResourceType::Gas), 200.0);
        player_resources.insert((0, ResourceType::Energy), 0.0);
        
        player_resources.insert((1, ResourceType::Mineral), 500.0);
        player_resources.insert((1, ResourceType::Gas), 200.0);
        player_resources.insert((1, ResourceType::Energy), 0.0);
        
        // Initialize supply
        player_supply.insert(0, (0, 10));
        player_supply.insert(1, (0, 10));
        
        // Initialize scores
        player_scores.insert(0, 0);
        player_scores.insert(1, 0);
        
        Self {
            phase: GamePhase::MainMenu,
            current_tick: 0,
            is_multiplayer: false,
            winner: None,
            player_count: 2,
            seed: 12345, // Default seed, should be randomized for real games
            game_speed: 1.0,
            player_resources,
            player_supply,
            player_scores,
            settings: GameSettings::default(),
        }
        }
    }
    
pub struct GameState {
    pub phase: GamePhase,
    pub current_tick: u64,
    pub is_multiplayer: bool,
    pub winner: Option<u8>,
    pub player_count: u8,
    pub seed: u64,
    pub game_speed: f32,
    
    // Player-specific state
    pub player_resources: HashMap<(u8, ResourceType), f32>,
    pub player_supply: HashMap<u8, (u32, u32)>, // (current, max) supply
    pub player_scores: HashMap<u8, u32>,
    pub settings: GameSettings,
}

/// Game settings
pub struct GameSettings {
    pub fog_of_war_enabled: bool,
    pub game_speed: f32,
    pub auto_save_enabled: bool,
    pub auto_save_interval: f32,
    pub show_fps: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fog_of_war_enabled: true,
            game_speed: 1.0,
            auto_save_enabled: false,
            auto_save_interval: 300.0, // 5 minutes
            show_fps: false,
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        let mut player_resources = HashMap::new();
        let mut player_supply = HashMap::new();
        let mut player_scores = HashMap::new();
        
        // Initialize default resources for 2 players
        player_resources.insert((0, ResourceType::Mineral), 500.0);
        player_resources.insert((0, ResourceType::Gas), 200.0);
        player_resources.insert((0, ResourceType::Energy), 0.0);
        
        player_resources.insert((1, ResourceType::Mineral), 500.0);
        player_resources.insert((1, ResourceType::Gas), 200.0);
        player_resources.insert((1, ResourceType::Energy), 0.0);
        
        // Initialize supply
        player_supply.insert(0, (0, 10));
        player_supply.insert(1, (0, 10));
        
        // Initialize scores
        player_scores.insert(0, 0);
        player_scores.insert(1, 0);
        
        Self {
            phase: GamePhase::MainMenu,
            current_tick: 0,
            is_multiplayer: false,
            winner: None,
            player_count: 2,
            seed: 12345, // Default seed, should be randomized for real games
            game_speed: 1.0,
            player_resources,
            player_supply,
            player_scores,
            settings: GameSettings::default(),
        }
    }
    
    pub fn update(&mut self) {
        self.current_tick += 1;
        
        // Check for game over conditions
        self.check_victory_conditions();
    }
    
    pub fn start_game(&mut self, multiplayer: bool, player_count: u8, seed: u64) {
        self.phase = GamePhase::Playing;
        self.is_multiplayer = multiplayer;
        self.player_count = player_count;
        self.seed = seed;
        self.current_tick = 0;
        self.winner = None;
        
        // Reset player resources and supply
        for player_id in 0..player_count {
            self.player_resources.insert((player_id, ResourceType::Mineral), 500.0);
            self.player_resources.insert((player_id, ResourceType::Gas), 200.0);
            self.player_resources.insert((player_id, ResourceType::Energy), 0.0);
            
            self.player_supply.insert(player_id as u8, (0, 10));
            self.player_scores.insert(player_id as u8, 0);
        }
    }
    
    pub fn pause(&mut self) {
        if self.phase == GamePhase::Playing {
            self.phase = GamePhase::Paused;
        }
    }
    
    pub fn resume(&mut self) {
        if self.phase == GamePhase::Paused {
            self.phase = GamePhase::Playing;
        }
    }
    
    pub fn can_afford(&self, player_id: u8, costs: &HashMap<ResourceType, f32>) -> bool {
        for (res_type, cost) in costs {
            let current = self.player_resources.get(&(player_id, *res_type)).copied().unwrap_or(0.0);
            if current < *cost {
                return false;
            }
        }
        true
    }
    
    pub fn deduct_resources(&mut self, player_id: u8, costs: &HashMap<ResourceType, f32>) {
        for (res_type, cost) in costs {
            if let Some(current) = self.player_resources.get_mut(&(player_id, *res_type)) {
                *current -= cost;
            }
        }
    }
    
    pub fn add_resources(&mut self, player_id: u8, resources: &HashMap<ResourceType, f32>) {
        for (res_type, amount) in resources {
            if let Some(current) = self.player_resources.get_mut(&(player_id, *res_type)) {
                *current += amount;
            } else {
                self.player_resources.insert((player_id, *res_type), *amount);
            }
        }
    }
    
    pub fn use_supply(&mut self, player_id: u8, amount: u32) -> bool {
        if let Some((current, max)) = self.player_supply.get_mut(&player_id) {
            if *current + amount <= *max {
                *current += amount;
                return true;
            }
        }
        false
    }
    
    pub fn add_max_supply(&mut self, player_id: u8, amount: u32) {
        if let Some((current, max)) = self.player_supply.get_mut(&player_id) {
            *max += amount;
        } else {
            self.player_supply.insert(player_id, (0, amount));
        }
    }
    
    fn check_victory_conditions(&mut self, world: &World) {
        let mut active_players = 0;
        let mut last_active_player = 0;
    
        // Count players with active headquarters
        let mut hq_query = world.query::<(&Building, &Owner)>();
        let mut player_hqs: HashMap<u8, usize> = HashMap::new();
    
        for (building, owner) in hq_query.iter() {
            if building.building_type == BuildingType::Headquarters && building.health > 0.0 {
                *player_hqs.entry(owner.0).or_insert(0) += 1;
            }
        }
    
        for (player_id, hq_count) in player_hqs.iter() {
            if *hq_count > 0 {
                active_players += 1;
                last_active_player = *player_id;
            }
        }
    
        // If only one player remains, they win
        if active_players == 1 {
            self.winner = Some(last_active_player);
            self.phase = GamePhase::GameOver;
        }
        // If no players remain, game is a draw
        else if active_players == 0 {
            self.phase = GamePhase::GameOver;
        }
    }
}