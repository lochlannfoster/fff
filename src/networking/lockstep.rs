use anyhow::Result;
use bincode::{serialize, deserialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};

use crate::engine::input::Command;

// Maximum number of ticks we can get ahead of the slowest player
const MAX_TICK_LEAD: u64 = 5;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkCommand {
    pub tick: u64,
    pub player_id: u8,
    pub commands: Vec<Command>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    Commands(NetworkCommand),
    Ping(u64),
    Pong(u64),
    Hello { player_id: u8, name: String },
    Start { seed: u64, start_tick: u64 },
    Sync { current_tick: u64 },
}

pub struct LockstepNetwork {
    active: bool,
    socket: Option<UdpSocket>,
    players: HashMap<u8, PlayerInfo>,
    local_player_id: u8,
    current_tick: u64,
    command_queue: HashMap<u64, HashMap<u8, Vec<Command>>>,
    message_queue: VecDeque<NetworkMessage>,
    is_host: bool,
    pending_commands: Vec<Command>,
    last_sent_commands_tick: u64,
}

struct PlayerInfo {
    address: SocketAddr,
    name: String,
    last_tick_received: u64,
    ping_ms: u32,
}

impl LockstepNetwork {
    pub fn new() -> Self {
        Self {
            active: false,
            socket: None,
            players: HashMap::new(),
            local_player_id: 0,
            current_tick: 0,
            command_queue: HashMap::new(),
            message_queue: VecDeque::new(),
            is_host: false,
            pending_commands: Vec::new(),
            last_sent_commands_tick: 0,
        }
    }
    
    pub fn host_game(&mut self, port: u16, player_name: String) -> Result<()> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        socket.set_nonblocking(true)?;
        
        self.socket = Some(socket);
        self.is_host = true;
        self.active = true;
        self.local_player_id = 0; // Host is always player 0
        
        // Add ourselves as a player
        self.players.insert(
            0,
            PlayerInfo {
                address: "127.0.0.1:0".parse().unwrap(),
                name: player_name,
                last_tick_received: 0,
                ping_ms: 0,
            },
        );
        
        Ok(())
    }
    
    pub fn join_game(&mut self, host_address: &str, player_name: String) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        
        // Connect to host
        let host_addr = host_address.parse()?;
        
        self.socket = Some(socket);
        self.is_host = false;
        self.active = true;
        
        // Send hello message to host
        self.send_to_host(NetworkMessage::Hello {
            player_id: 255, // Will be assigned by host
            name: player_name,
        })?;
        
        Ok(())
    }
    
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    pub fn process_messages(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        
        let socket = match &self.socket {
            Some(s) => s,
            None => return Ok(()),
        };
        
        // Buffer for incoming data
        let mut buf = [0u8; 1024];
        
        // Process all pending messages
        loop {
            match socket.recv_from(&mut buf) {
                Ok((bytes_received, src_addr)) => {
                    // Deserialize the message
                    match deserialize::<NetworkMessage>(&buf[0..bytes_received]) {
                        Ok(message) => self.handle_message(message, src_addr)?,
                        Err(e) => eprintln!("Failed to deserialize network message: {}", e),
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No more messages to process
                    break;
                }
                Err(e) => {
                    eprintln!("Error receiving network message: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    pub fn send_commands(&mut self, commands: &[Command]) -> Result<()> {
        if !self.active || commands.is_empty() {
            return Ok(());
        }
        
        // Add commands to pending list
        self.pending_commands.extend_from_slice(commands);
        
        // Only send commands periodically (e.g., every tick)
        if self.current_tick == self.last_sent_commands_tick {
            return Ok(());
        }
        
        // Create network command
        let net_command = NetworkCommand {
            tick: self.current_tick + MAX_TICK_LEAD, // Commands will be executed in the future
            player_id: self.local_player_id,
            commands: std::mem::take(&mut self.pending_commands),
        };
        
        // Send command to all players (or just host if client)
        if self.is_host {
            for (&player_id, player_info) in self.players.iter() {
                if player_id != self.local_player_id {
                    self.send_to(
                        NetworkMessage::Commands(net_command.clone()),
                        player_info.address,
                    )?;
                }
            }
        } else {
            // Send only to host
            self.send_to_host(NetworkMessage::Commands(net_command))?;
        }
        
        self.last_sent_commands_tick = self.current_tick;
        
        Ok(())
    }
    
    pub fn receive_commands(&mut self) -> HashMap<u8, Vec<Command>> {
        // Get commands for current tick
        match self.command_queue.remove(&self.current_tick) {
            Some(commands) => {
                // Advance tick
                self.current_tick += 1;
                commands
            }
            None => {
                // If no commands for this tick, still advance unless we're too far ahead
                let min_tick = self.players.values()
                    .map(|p| p.last_tick_received)
                    .min()
                    .unwrap_or(0);
                
                if self.current_tick - min_tick < MAX_TICK_LEAD {
                    self.current_tick += 1;
                }
                
                HashMap::new()
            }
        }
    }
    
    fn handle_message(&mut self, message: NetworkMessage, src_addr: SocketAddr) -> Result<()> {
        match message {
            NetworkMessage::Commands(cmd) => {
                // Store commands in queue for appropriate tick
                let player_cmds = self.command_queue
                    .entry(cmd.tick)
                    .or_insert_with(HashMap::new);
                
                player_cmds.insert(cmd.player_id, cmd.commands);
                
                // Update last tick received for this player
                if let Some(player) = self.players.get_mut(&cmd.player_id) {
                    player.last_tick_received = cmd.tick;
                }
                
                // If host, relay commands to other players
                if self.is_host {
                    for (&player_id, player_info) in self.players.iter() {
                        if player_id != cmd.player_id && player_id != self.local_player_id {
                            self.send_to(
                                NetworkMessage::Commands(cmd.clone()),
                                player_info.address,
                            )?;
                        }
                    }
                }
            }
            NetworkMessage::Hello { player_id, name } => {
                if self.is_host {
                    // Assign a player ID and add to our list
                    let new_player_id = self.players.keys().max().unwrap_or(&0) + 1;
                    
                    self.players.insert(
                        new_player_id,
                        PlayerInfo {
                            address: src_addr,
                            name,
                            last_tick_received: self.current_tick,
                            ping_ms: 0,
                        },
                    );
                    
                    // Send join confirmation with assigned ID
                    self.send_to(
                        NetworkMessage::Hello {
                            player_id: new_player_id,
                            name: "Host".to_string(),
                        },
                        src_addr,
                    )?;
                } else if player_id != 255 {
                    // We've been assigned a player ID by the host
                    self.local_player_id = player_id;
                    
                    // Add host to our players list
                    self.players.insert(
                        0, // Host is always player 0
                        PlayerInfo {
                            address: src_addr,
                            name,
                            last_tick_received: self.current_tick,
                            ping_ms: 0,
                        },
                    );
                }
            }
            NetworkMessage::Start { seed, start_tick } => {
                // Game starting command (host to clients)
                if !self.is_host {
                    self.current_tick = start_tick;
                    // Initialize game with seed
                }
            }
            NetworkMessage::Ping(timestamp) => {
                // Reply with pong
                self.send_to(NetworkMessage::Pong(timestamp), src_addr)?;
            }
            NetworkMessage::Pong(timestamp) => {
                // Calculate ping time
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                
                let ping_ms = (now - timestamp) as u32;
                
                // Update player's ping time
                for player in self.players.values_mut() {
                    if player.address == src_addr {
                        player.ping_ms = ping_ms;
                        break;
                    }
                }
            }
            NetworkMessage::Sync { current_tick } => {
                // Handle sync message (used for catching up)
                if !self.is_host && current_tick > self.current_tick {
                    // We're behind, fast forward
                    self.current_tick = current_tick;
                }
            }
        }
        
        Ok(())
    }
    
    fn send_to(&self, message: NetworkMessage, addr: SocketAddr) -> Result<()> {
        if let Some(socket) = &self.socket {
            let data = serialize(&message)?;
            socket.send_to(&data, addr)?;
        }
        
        Ok(())
    }
    
    fn send_to_host(&self, message: NetworkMessage) -> Result<()> {
        if let Some(host) = self.players.get(&0) {
            self.send_to(message, host.address)?;
        }
        
        Ok(())
    }
}