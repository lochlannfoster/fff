use glam::Vec2;
use noise::{NoiseFn, Perlin, Seedable};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashSet;

use crate::ecs::resources::{GameMap, TerrainTile, PathfindingGrid};
use crate::ecs::components::ResourceType;
use crate::game::pathfinding;

/// Map generation parameters
pub struct MapGenerationParams {
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    pub player_count: u8,
    pub water_threshold: f64,
    pub mountain_threshold: f64,
    pub forest_threshold: f64,
    pub resource_density: f32,
}

impl Default for MapGenerationParams {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
            seed: 12345,
            player_count: 2,
            water_threshold: 0.3,
            mountain_threshold: 0.7,
            forest_threshold: 0.6,
            resource_density: 0.01,
        }
    }
}

/// Generate a new random map
pub fn generate_map(params: &MapGenerationParams) -> GameMap {
    let mut rng = StdRng::seed_from_u64(params.seed);
    
    // Create terrain using Perlin noise
    let perlin = Perlin::new().set_seed(params.seed as u32);
    let mut terrain_tiles = Vec::with_capacity((params.width * params.height) as usize);
    
    for y in 0..params.height {
        for x in 0..params.width {
            let nx = x as f64 / params.width as f64;
            let ny = y as f64 / params.height as f64;
            
            // Generate base noise value
            let noise_val = perlin.get([nx * 4.0, ny * 4.0, 0.0]);
            
            // Determine terrain type based on noise
            let terrain = if noise_val < params.water_threshold {
                TerrainTile::Water
            } else if noise_val > params.mountain_threshold {
                TerrainTile::Mountain
            } else if noise_val > params.forest_threshold {
                TerrainTile::Forest
            } else {
                TerrainTile::Ground
            };
            
            terrain_tiles.push(terrain);
        }
    }
    
    // Generate resource positions
    let mut resource_positions = Vec::new();
    let num_resources = (params.width * params.height) as f32 * params.resource_density;
    
    for _ in 0..num_resources as usize {
        // Try to find a valid position for resources (not in water or mountains)
        let mut attempts = 0;
        while attempts < 10 {
            let x = rng.gen_range(0..params.width);
            let y = rng.gen_range(0..params.height);
            let idx = (y * params.width + x) as usize;
            
            if let TerrainTile::Ground | TerrainTile::Forest = terrain_tiles[idx] {
                // Determine resource type
                let resource_type = if rng.gen_bool(0.7) {
                    ResourceType::Mineral
                } else {
                    ResourceType::Gas
                };
                
                // Determine amount
                let amount = match resource_type {
                    ResourceType::Mineral => rng.gen_range(1000.0..2000.0),
                    ResourceType::Gas => rng.gen_range(500.0..1500.0),
                    ResourceType::Energy => rng.gen_range(0.0..100.0),
                };
                
                let pos = Vec2::new(x as f32, y as f32);
                resource_positions.push((pos, resource_type, amount));
                break;
            }
            
            attempts += 1;
        }
    }
    
    // Generate player starting positions
    let starting_positions = generate_starting_positions(
        params.width,
        params.height,
        params.player_count,
        &terrain_tiles,
        &mut rng,
    );
    
    // Create the game map
    let mut map = GameMap {
        width: params.width,
        height: params.height,
        terrain_tiles,
        resource_positions,
        starting_positions,
        pathfinding_grid: None,
        fog_of_war: Default::default(),
    };
    
    // Generate pathfinding grid
    map.pathfinding_grid = Some(pathfinding::generate_pathfinding_grid(&map, 8.0));
    
    map
}

/// Generate fair starting positions for players
fn generate_starting_positions(
    width: u32,
    height: u32,
    player_count: u8,
    terrain_tiles: &[TerrainTile],
    rng: &mut StdRng,
) -> Vec<Vec2> {
    let mut positions = Vec::new();
    
    // For 2 players, use diagonal corners
    if player_count == 2 {
        // Find suitable positions in opposite corners
        let corners = [
            (width / 8, height / 8),                     // Top-left
            (width - width / 8, height / 8),             // Top-right
            (width / 8, height - height / 8),            // Bottom-left
            (width - width / 8, height - height / 8),    // Bottom-right
        ];
        
        // Select 2 opposite corners
        let corner_pairs = [
            (0, 3), // Top-left and bottom-right
            (1, 2), // Top-right and bottom-left
        ];
        
        let pair_idx = rng.gen_range(0..corner_pairs.len());
        let (first, second) = corner_pairs[pair_idx];
        
        // Make sure the areas are suitable (not water/mountains)
        // and find the closest valid spot if not
        for &corner_idx in &[first, second] {
            let (cx, cy) = corners[corner_idx];
            let pos = find_valid_starting_position(cx, cy, width, height, terrain_tiles, rng);
            positions.push(Vec2::new(pos.0 as f32, pos.1 as f32));
        }
    } 
    // For more players, distribute around the map
    else {
        let angle_step = 2.0 * std::f32::consts::PI / player_count as f32;
        let radius = (width.min(height) as f32 * 0.4).min(100.0);
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        
        for i in 0..player_count {
            let angle = i as f32 * angle_step;
            let x = center_x + angle.cos() * radius;
            let y = center_y + angle.sin() * radius;
            
            let grid_x = x as u32;
            let grid_y = y as u32;
            
            let pos = find_valid_starting_position(
                grid_x, grid_y, width, height, terrain_tiles, rng
            );
            
            positions.push(Vec2::new(pos.0 as f32, pos.1 as f32));
        }
    }
    
    positions
}

/// Find a valid position near the given coordinates
fn find_valid_starting_position(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    terrain_tiles: &[TerrainTile],
    rng: &mut StdRng,
) -> (u32, u32) {
    // Check if the initial position is valid
    let idx = (y * width + x) as usize;
    if idx < terrain_tiles.len() {
        match terrain_tiles[idx] {
            TerrainTile::Ground => return (x, y),
            _ => {}
        }
    }
    
    // Search in expanding rings until a valid position is found
    for radius in 1..20 {
        let mut valid_positions = Vec::new();
        
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                // Only check the perimeter
                if dx.abs() == radius || dy.abs() == radius {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    
                    // Skip if out of bounds
                    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                        continue;
                    }
                    
                    let idx = (ny as u32 * width + nx as u32) as usize;
                    if idx < terrain_tiles.len() && terrain_tiles[idx] == TerrainTile::Ground {
                        valid_positions.push((nx as u32, ny as u32));
                    }
                }
            }
        }
        
        if !valid_positions.is_empty() {
            return valid_positions[rng.gen_range(0..valid_positions.len())];
        }
    }
    
    // Fallback - just find any valid position
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if terrain_tiles[idx] == TerrainTile::Ground {
                return (x, y);
            }
        }
    }
    
    // Absolute fallback - return original position even if invalid
    (x, y)
}

/// Create a minimap texture from the terrain data
pub fn generate_minimap(map: &GameMap) -> Vec<u8> {
    let width = map.width as usize;
    let height = map.height as usize;
    let mut minimap_data = vec![0u8; width * height * 4]; // RGBA format
    
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let pixel_idx = idx * 4;
            
            // Set color based on terrain type
            match map.terrain_tiles[idx] {
                TerrainTile::Ground => {
                    minimap_data[pixel_idx] = 120;     // R - Light brown
                    minimap_data[pixel_idx + 1] = 100; // G
                    minimap_data[pixel_idx + 2] = 80;  // B
                    minimap_data[pixel_idx + 3] = 255; // A
                }
                TerrainTile::Water => {
                    minimap_data[pixel_idx] = 64;      // R - Blue
                    minimap_data[pixel_idx + 1] = 100; // G
                    minimap_data[pixel_idx + 2] = 200; // B
                    minimap_data[pixel_idx + 3] = 255; // A
                }
                TerrainTile::Mountain => {
                    minimap_data[pixel_idx] = 100;     // R - Gray
                    minimap_data[pixel_idx + 1] = 100; // G
                    minimap_data[pixel_idx + 2] = 100; // B
                    minimap_data[pixel_idx + 3] = 255; // A
                }
                TerrainTile::Forest => {
                    minimap_data[pixel_idx] = 40;      // R - Green
                    minimap_data[pixel_idx + 1] = 120; // G
                    minimap_data[pixel_idx + 2] = 40;  // B
                    minimap_data[pixel_idx + 3] = 255; // A
                }
            }
        }
    }
    
    // Add resources to minimap
    for (pos, resource_type, _) in &map.resource_positions {
        let x = pos.x as usize;
        let y = pos.y as usize;
        
        if x < width && y < height {
            let pixel_idx = (y * width + x) * 4;
            
            match resource_type {
                ResourceType::Mineral => {
                    // Blue minerals
                    minimap_data[pixel_idx] = 100;     // R
                    minimap_data[pixel_idx + 1] = 150; // G
                    minimap_data[pixel_idx + 2] = 255; // B
                }
                ResourceType::Gas => {
                    // Green gas
                    minimap_data[pixel_idx] = 150;     // R
                    minimap_data[pixel_idx + 1] = 255; // G
                    minimap_data[pixel_idx + 2] = 150; // B
                }
                ResourceType::Energy => {
                    // Yellow energy
                    minimap_data[pixel_idx] = 255;     // R
                    minimap_data[pixel_idx + 1] = 255; // G
                    minimap_data[pixel_idx + 2] = 0;   // B
                }
            }
        }
    }
    
    // Add starting positions to minimap
    for pos in &map.starting_positions {
        let x = pos.x as usize;
        let y = pos.y as usize;
        
        if x < width && y < height {
            // Mark starting positions with bright red
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    
                    if nx < width && ny < height {
                        let pixel_idx = (ny * width + nx) * 4;
                        minimap_data[pixel_idx] = 255;     // R
                        minimap_data[pixel_idx + 1] = 0;   // G
                        minimap_data[pixel_idx + 2] = 0;   // B
                    }
                }
            }
        }
    }
    
    minimap_data
}

/// Apply fog of war to the map for a specific player
pub fn update_fog_of_war(map: &mut GameMap, player_id: u8, visible_tiles: HashSet<u32>) {
    map.fog_of_war.insert(player_id, visible_tiles);
}

/// Calculate visible tiles based on unit positions and sight ranges
pub fn calculate_visible_tiles(
    map: &GameMap,
    unit_positions: &[(Vec2, f32)], // Position and sight range pairs
    grid_size: f32,
) -> HashSet<u32> {
    let mut visible_tiles = HashSet::new();
    
    for (position, sight_range) in unit_positions {
        let center_x = position.x / grid_size;
        let center_y = position.y / grid_size;
        let radius = sight_range / grid_size;
        
        // Mark all tiles within sight range as visible
        let min_x = ((center_x - radius).floor() as i32).max(0);
        let max_x = ((center_x + radius).ceil() as i32).min(map.width as i32 - 1);
        let min_y = ((center_y - radius).floor() as i32).max(0);
        let max_y = ((center_y + radius).ceil() as i32).min(map.height as i32 - 1);
        
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance <= radius {
                    let tile_idx = (y as u32 * map.width + x as u32);
                    visible_tiles.insert(tile_idx);
                }
            }
        }
    }
    
    visible_tiles
}