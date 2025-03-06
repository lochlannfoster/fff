pub mod commands;
pub mod replay;
pub mod lockstep;

use anyhow::Result;
use std::net::SocketAddr;

/// Trait for network transport implementations
pub trait NetworkTransport {
    /// Initialize the transport
    fn init(&mut self) -> Result<()>;
    
    /// Send data to a specific address
    fn send_to(&self, data: &[u8], addr: SocketAddr) -> Result<()>;
    
    /// Receive data from any address
    fn recv_from(&self) -> Result<Option<(Vec<u8>, SocketAddr)>>;
    
    /// Close the transport
    fn close(&mut self);
    
    /// Check if transport is connected
    fn is_connected(&self) -> bool;
}

/// UDP transport implementation
pub struct UdpTransport {
    socket: Option<std::net::UdpSocket>,
    is_connected: bool,
}

impl UdpTransport {
    pub fn new() -> Self {
        Self {
            socket: None,
            is_connected: false,
        }
    }
    
    pub fn bind(&mut self, address: &str) -> Result<()> {
        let socket = std::net::UdpSocket::bind(address)?;
        socket.set_nonblocking(true)?;
        self.socket = Some(socket);
        self.is_connected = true;
        Ok(())
    }
}

impl NetworkTransport for UdpTransport {
    fn init(&mut self) -> Result<()> {
        if self.socket.is_none() {
            // Bind to any available port
            self.bind("0.0.0.0:0")?;
        }
        Ok(())
    }
    
    fn send_to(&self, data: &[u8], addr: SocketAddr) -> Result<()> {
        if let Some(socket) = &self.socket {
            socket.send_to(data, addr)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Socket not initialized"))
        }
    }
    
    fn recv_from(&self) -> Result<Option<(Vec<u8>, SocketAddr)>> {
        if let Some(socket) = &self.socket {
            let mut buf = [0u8; 1024 * 16]; // 16KB buffer
            match socket.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    Ok(Some((buf[0..len].to_vec(), addr)))
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available
                    Ok(None)
                }
                Err(e) => Err(e.into()),
            }
        } else {
            Err(anyhow::anyhow!("Socket not initialized"))
        }
    }
    
    fn close(&mut self) {
        self.socket = None;
        self.is_connected = false;
    }
    
    fn is_connected(&self) -> bool {
        self.is_connected
    }
}

/// Network session for game multiplayer
pub struct NetworkSession {
    transport: Box<dyn NetworkTransport>,
    local_player_id: Option<u8>,
    host_addr: Option<SocketAddr>,
    player_addrs: std::collections::HashMap<u8, SocketAddr>,
    command_buffer: std::collections::VecDeque<commands::NetworkMessage>,
}

impl NetworkSession {
    pub fn new(transport: Box<dyn NetworkTransport>) -> Self {
        Self {
            transport,
            local_player_id: None,
            host_addr: None,
            player_addrs: std::collections::HashMap::new(),
            command_buffer: std::collections::VecDeque::new(),
        }
    }
    
    pub fn host_game(&mut self, port: u16) -> Result<()> {
        let host_transport = UdpTransport::new();
        self.transport = Box::new(host_transport);
        self.transport.init()?;
        self.local_player_id = Some(0); // Host is always player 0
        
        // In a real implementation, you'd start listening for client connections
        
        Ok(())
    }
    
    pub fn join_game(&mut self, host_address: &str) -> Result<()> {
        let client_transport = UdpTransport::new();
        self.transport = Box::new(client_transport);
        self.transport.init()?;
        
        // Connect to host
        let addr: SocketAddr = host_address.parse()?;
        self.host_addr = Some(addr);
        
        // Send join request
        let join_msg = commands::NetworkMessage::PlayerJoin(commands::PlayerJoinMessage {
            player_id: 255, // Will be assigned by host
            player_name: "Player".to_string(),
            is_observer: false,
        });
        
        let data = bincode::serialize(&join_msg)?;
        self.transport.send_to(&data, addr)?;
        
        Ok(())
    }
    
    pub fn process_messages(&mut self) -> Result<Vec<commands::NetworkMessage>> {
        let mut received_messages = Vec::new();
        
        // Process any buffered messages first
        while let Some(message) = self.command_buffer.pop_front() {
            received_messages.push(message);
        }
        
        // Process incoming network messages
        loop {
            match self.transport.recv_from() {
                Ok(Some((data, src_addr))) => {
                    match bincode::deserialize::<commands::NetworkMessage>(&data) {
                        Ok(message) => {
                            match &message {
                                commands::NetworkMessage::PlayerJoin(join) => {
                                    // Handle player join
                                    self.player_addrs.insert(join.player_id, src_addr);
                                }
                                commands::NetworkMessage::PlayerLeave(leave) => {
                                    // Handle player leave
                                    self.player_addrs.remove(&leave.player_id);
                                }
                                _ => {}
                            }
                            
                            received_messages.push(message);
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!("Failed to deserialize message: {}", e));
                        }
                    }
                }
                Ok(None) => {
                    // No more messages to process
                    break;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Error receiving message: {}", e));
                }
            }
        }
        
        Ok(received_messages)
    }
    
    pub fn send_message(&self, message: commands::NetworkMessage, target_player: Option<u8>) -> Result<()> {
        let data = bincode::serialize(&message)?;
        
        match target_player {
            Some(player_id) => {
                if let Some(addr) = self.player_addrs.get(&player_id) {
                    self.transport.send_to(&data, *addr)?;
                } else if let Some(host_addr) = self.host_addr {
                    // If we don't know the player's address, send to host for relay
                    self.transport.send_to(&data, host_addr)?;
                } else {
                    return Err(anyhow::anyhow!("Unknown player address and no host address"));
                }
            }
            None => {
                // Broadcast to all players
                if self.local_player_id == Some(0) {
                    // Host broadcasts to all clients
                    for addr in self.player_addrs.values() {
                        self.transport.send_to(&data, *addr)?;
                    }
                } else if let Some(host_addr) = self.host_addr {
                    // Client sends to host for relay
                    self.transport.send_to(&data, host_addr)?;
                } else {
                    return Err(anyhow::anyhow!("No host address"));
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_local_player_id(&self) -> Option<u8> {
        self.local_player_id
    }
    
    pub fn get_player_count(&self) -> usize {
        self.player_addrs.len() + 1 // +1 for local player
    }
    
    pub fn is_host(&self) -> bool {
        self.local_player_id == Some(0)
    }
    
    pub fn close(&mut self) {
        // Send leave message if we're connected
        if let Some(player_id) = self.local_player_id {
            let leave_msg = commands::NetworkMessage::PlayerLeave(commands::PlayerLeaveMessage {
                player_id,
                reason: commands::DisconnectReason::Quit,
            });
            
            if let Ok(data) = bincode::serialize(&leave_msg) {
                for addr in self.player_addrs.values() {
                    let _ = self.transport.send_to(&data, *addr);
                }
            }
        }
        
        self.transport.close();
        self.local_player_id = None;
        self.host_addr = None;
        self.player_addrs.clear();
    }
}