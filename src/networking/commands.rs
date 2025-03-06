use serde::{Serialize, Deserialize};
use glam::Vec2;

use crate::engine::input::Command;

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Player command message
    Command(CommandMessage),
    
    /// Player joined the game
    PlayerJoin(PlayerJoinMessage),
    
    /// Player left the game
    PlayerLeave(PlayerLeaveMessage),
    
    /// Game start message with seed and settings
    GameStart(GameStartMessage),
    
    /// Game state synchronization message
    GameSync(GameSyncMessage),
    
    /// Ping request message
    Ping(u64),
    
    /// Ping response message
    Pong(u64),
    
    /// Chat message
    Chat(ChatMessage),
    
    /// Error message
    Error(ErrorMessage),
}

/// Command message sent by players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMessage {
    pub player_id: u8,
    pub tick: u64,
    pub sequence: u32,
    pub commands: Vec<Command>,
}

/// Player join message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerJoinMessage {
    pub player_id: u8,
    pub player_name: String,
    pub is_observer: bool,
}

/// Player leave message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLeaveMessage {
    pub player_id: u8,
    pub reason: DisconnectReason,
}

/// Disconnect reasons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisconnectReason {
    Quit,
    Timeout,
    Kicked,
    Error,
}

/// Game start message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStartMessage {
    pub seed: u64,
    pub start_tick: u64,
    pub map_name: String,
    pub player_count: u8,
    pub player_positions: Vec<(u8, u8)>, // Player ID to starting position index
    pub options: GameOptions,
}

/// Game options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameOptions {
    pub fog_of_war_enabled: bool,
    pub starting_resources: ResourceLevels,
    pub victory_condition: VictoryCondition,
    pub game_speed: f32,
}

/// Resource starting levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceLevels {
    Low,
    Normal,
    High,
}

/// Victory conditions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VictoryCondition {
    Annihilation,
    CapitalElimination,
    ResourceControl(u32), // Number of resource points to control
    TimeLimit(u32),       // Time limit in minutes
}

/// Game sync message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSyncMessage {
    pub tick: u64,
    pub checksums: Vec<(u8, u32)>, // Player ID to game state checksum
    pub unit_counts: Vec<(u8, u32)>, // Player ID to unit count
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub player_id: u8,
    pub message: String,
    pub target: ChatTarget,
}

/// Chat message target
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatTarget {
    All,
    Team,
    Player(u8),
}

/// Error message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub code: u32,
    pub message: String,
}

/// Command logger for replay functionality
pub struct CommandLogger {
    commands: Vec<(u64, u8, Vec<Command>)>, // (Tick, Player ID, Commands)
    enabled: bool,
}

impl CommandLogger {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            enabled: true,
        }
    }
    
    pub fn log_commands(&mut self, tick: u64, player_id: u8, commands: &[Command]) {
        if !self.enabled {
            return;
        }
        
        self.commands.push((tick, player_id, commands.to_vec()));
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn get_commands(&self) -> &[(u64, u8, Vec<Command>)] {
        &self.commands
    }
    
    pub fn clear(&mut self) {
        self.commands.clear();
    }
    
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let serialized = bincode::serialize(&self.commands)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            
        std::fs::write(path, serialized)
    }
    
    pub fn load_from_file(&mut self, path: &str) -> std::io::Result<()> {
        let data = std::fs::read(path)?;
        
        self.commands = bincode::deserialize(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            
        Ok(())
    }
}

/// Calculate checksum from game state for desync detection
pub fn calculate_game_checksum(
    entities: &[(u32, Vec2, u8)], // Entity ID, Position, Owner
    resources: &[(u8, crate::ecs::components::ResourceType, f32)], // Player ID, Resource Type, Amount
) -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    
    // Hash entity data
    for (id, pos, owner) in entities {
        id.hash(&mut hasher);
        (pos.x.to_bits()).hash(&mut hasher);
        (pos.y.to_bits()).hash(&mut hasher);
        owner.hash(&mut hasher);
    }
    
    // Hash resource data
    for (player_id, res_type, amount) in resources {
        player_id.hash(&mut hasher);
        std::mem::discriminant(res_type).hash(&mut hasher);
        (amount.to_bits()).hash(&mut hasher);
    }
    
    // Return lower 32 bits of the hash
    hasher.finish() as u32
}