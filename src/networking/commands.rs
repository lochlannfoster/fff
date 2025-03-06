use serde::{Serialize, Deserialize};
use glam::Vec2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    PlayerJoin(PlayerJoinMessage),
    PlayerLeave(PlayerLeaveMessage),
    // Other message types...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerJoinMessage {
    pub player_id: u8,
    pub player_name: String,
    pub is_observer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLeaveMessage {
    pub player_id: u8,
    pub reason: DisconnectReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisconnectReason {
    Quit,
    NetworkError,
    Timeout,
}