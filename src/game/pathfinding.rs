use glam::Vec2;
use pathfinding::prelude::astar;
use std::collections::{HashMap, HashSet};


use crate::ecs::resources::{GameMap, PathfindingGrid, PathNode};

/// Convert world position to grid coordinates
pub fn world_to_grid(pos: Vec2, grid_size: f32) -> (i32, i32) {
    let x = (pos.x / grid_size).floor() as i32;
    let y = (pos.y / grid_size).floor() as i32;
    (x, y)
}

/// Convert grid coordinates to world position (center of grid cell)
pub fn grid_to_world(grid_pos: (i32, i32), grid_size: f32) -> Vec2 {
    Vec2::new(
        (grid_pos.0 as f32 + 0.5) * grid_size,
        (grid_pos.1 as f32 + 0.5) * grid_size,
    )
}

/// Grid-based pathfinding using A* algorithm
pub fn find_path(
    start: Vec2,
    goal: Vec2,
    grid: &PathfindingGrid,
    grid_size: f32,
    unit_radius: f32,
) -> Option<Vec<Vec2>> {
    let start_grid = world_to_grid(start, grid_size);
    let goal_grid = world_to_grid(goal, grid_size);
    
    // If start or goal is out of bounds, return None
    if !is_in_bounds(start_grid, grid) || !is_in_bounds(goal_grid, grid) {
        return None;
    }
    
    // If start or goal is not walkable, find nearest walkable cell
    let start_grid = if !is_walkable(start_grid, grid, unit_radius) {
        find_nearest_walkable(start_grid, grid, unit_radius)?
    } else {
        start_grid
    };
    
    let goal_grid = if !is_walkable(goal_grid, grid, unit_radius) {
        find_nearest_walkable(goal_grid, grid, unit_radius)?
    } else {
        goal_grid
    };
    
    // A* pathfinding
    let result = astar(
        &start_grid,
        |&(x, y)| {
            // Get neighbors
            let neighbors = [
                (x - 1, y - 1), (x, y - 1), (x + 1, y - 1),
                (x - 1, y),                 (x + 1, y),
                (x - 1, y + 1), (x, y + 1), (x + 1, y + 1),
            ];
            
            neighbors.iter()
                .filter(|&&pos| is_in_bounds(pos, grid) && is_walkable(pos, grid, unit_radius))
                .map(|&pos| {
                    // Diagonal movement costs more
                    let cost = if pos.0 != x && pos.1 != y {
                        1.4 // sqrt(2)
                    } else {
                        1.0
                    };
                    
                    // Add terrain cost
                    let idx = grid_pos_to_index(pos, grid.width);
                    let node_cost = grid.nodes[idx].cost;
                    
                    (pos, (cost * node_cost) as u32)
                })
                .collect::<Vec<_>>()
        },
        |&(x, y)| {
            // Heuristic function (Manhattan distance)
            (absdiff(x, goal_grid.0) + absdiff(y, goal_grid.1)) as u32
        },
        |&pos| pos == goal_grid,
    );
    
    // Convert the grid path to world coordinates
    match result {
        Some((path, _)) => {
            let world_path: Vec<Vec2> = path.into_iter()
                .map(|grid_pos| grid_to_world(grid_pos, grid_size))
                .collect();
            
            Some(world_path)
        }
        None => None,
    }
}

/// Check if grid position is in bounds
fn is_in_bounds(pos: (i32, i32), grid: &PathfindingGrid) -> bool {
    pos.0 >= 0 && pos.0 < grid.width as i32 && pos.1 >= 0 && pos.1 < grid.height as i32
}

/// Check if a grid position is walkable
fn is_walkable(pos: (i32, i32), grid: &PathfindingGrid, unit_radius: f32) -> bool {
    if !is_in_bounds(pos, grid) {
        return false;
    }
    
    let idx = grid_pos_to_index(pos, grid.width);
    grid.nodes[idx].walkable
}

/// Convert grid position to array index
fn grid_pos_to_index(pos: (i32, i32), width: usize) -> usize {
    (pos.1 as usize) * width + (pos.0 as usize)
}

/// Find the nearest walkable grid position
fn find_nearest_walkable(
    pos: (i32, i32),
    grid: &PathfindingGrid,
    unit_radius: f32,
) -> Option<(i32, i32)> {
    // Search in expanding rings
    for radius in 1..10 {
        for y in -radius..=radius {
            for x in -radius..=radius {
                // Only check positions on the ring perimeter
                if x.abs() == radius || y.abs() == radius {
                    let check_pos = (pos.0 + x, pos.1 + y);
                    if is_in_bounds(check_pos, grid) && is_walkable(check_pos, grid, unit_radius) {
                        return Some(check_pos);
                    }
                }
            }
        }
    }
    
    None
}

/// Generate a pathfinding grid from a game map
pub fn generate_pathfinding_grid(map: &GameMap, grid_size: f32) -> PathfindingGrid {
    let width = (map.width as f32 / grid_size).ceil() as usize;
    let height = (map.height as f32 / grid_size).ceil() as usize;
    
    let mut nodes = Vec::with_capacity(width * height);
    
    // Initialize all nodes as walkable with default cost
    for y in 0..height {
        for x in 0..width {
            let world_pos = grid_to_world((x as i32, y as i32), grid_size);
            
            // Default to walkable
            let mut node = PathNode {
                walkable: true,
                cost: 1.0,
            };
            
            // Check terrain type at this position
            if let Some(terrain_idx) = get_terrain_at(world_pos, map) {
                let terrain = &map.terrain_tiles[terrain_idx];
                match terrain {
                    crate::ecs::resources::TerrainTile::Ground => {
                        // Basic terrain, default cost
                        node.cost = 1.0;
                    }
                    crate::ecs::resources::TerrainTile::Water => {
                        // Water is not walkable for land units
                        node.walkable = false;
                    }
                    crate::ecs::resources::TerrainTile::Mountain => {
                        // Mountains are not walkable
                        node.walkable = false;
                    }
                    crate::ecs::resources::TerrainTile::Forest => {
                        // Forests slow movement
                        node.cost = 2.0;
                    }
                }
            }
            
            nodes.push(node);
        }
    }
    
    PathfindingGrid {
        width,
        height,
        nodes,
    }
}

/// Helper function to get terrain index at a world position
fn get_terrain_at(pos: Vec2, map: &GameMap) -> Option<usize> {
    // In a real implementation, this would check the actual map data
    // This is a placeholder
    if pos.x < 0.0 || pos.y < 0.0 || pos.x >= map.width as f32 || pos.y >= map.height as f32 {
        return None;
    }
    
    let x = pos.x as usize;
    let y = pos.y as usize;
    let idx = y * map.width as usize + x;
    
    if idx < map.terrain_tiles.len() {
        Some(idx)
    } else {
        None
    }
}

/// Update the pathfinding grid with dynamic obstacles
pub fn update_grid_with_obstacles(
    grid: &mut PathfindingGrid,
    obstacles: &[(Vec2, f32)], // Position and radius pairs
    grid_size: f32,
) {
    // Reset grid to initial state (would need to store the original grid)
    // For each obstacle, mark grid cells as unwalkable
    for (position, radius) in obstacles {
        let grid_pos = world_to_grid(*position, grid_size);
        let grid_radius = (radius / grid_size).ceil() as i32;
        
        // Mark cells within radius as unwalkable
        for y in -grid_radius..=grid_radius {
            for x in -grid_radius..=grid_radius {
                let pos = (grid_pos.0 + x, grid_pos.1 + y);
                
                // Skip if out of bounds
                if !is_in_bounds(pos, grid) {
                    continue;
                }
                
                // Check if cell center is within obstacle radius
                let cell_center = grid_to_world(pos, grid_size);
                let distance = (*position - cell_center).length();
                
                if distance <= *radius {
                    let idx = grid_pos_to_index(pos, grid.width);
                    grid.nodes[idx].walkable = false;
                }
            }
        }
    }
}

/// Path smoothing to make paths more natural
pub fn smooth_path(path: &[Vec2], grid: &PathfindingGrid, grid_size: f32, unit_radius: f32) -> Vec<Vec2> {
    if path.len() <= 2 {
        return path.to_vec();
    }
    
    let mut smoothed_path = Vec::new();
    smoothed_path.push(path[0]);
    
    let mut current_idx = 0;
    
    while current_idx < path.len() - 1 {
        // Find furthest visible point from current position
        let mut furthest_visible = current_idx + 1;
        
        for i in (current_idx + 2)..path.len() {
            if has_line_of_sight(path[current_idx], path[i], grid, grid_size, unit_radius) {
                furthest_visible = i;
            } else {
                break;
            }
        }
        
        // Add furthest visible point to smoothed path
        if furthest_visible > current_idx + 1 {
            smoothed_path.push(path[furthest_visible]);
            current_idx = furthest_visible;
        } else {
            smoothed_path.push(path[current_idx + 1]);
            current_idx += 1;
        }
    }
    
    smoothed_path
}

/// Check if there's a clear line of sight between two points
fn has_line_of_sight(start: Vec2, end: Vec2, grid: &PathfindingGrid, grid_size: f32, unit_radius: f32) -> bool {
    let distance = (end - start).length();
    let direction = (end - start).normalize();
    
    // Check points along the line
    let steps = (distance / (grid_size * 0.5)).ceil() as i32;
    
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let point = start + direction * distance * t;
        let grid_pos = world_to_grid(point, grid_size);
        
        if !is_in_bounds(grid_pos, grid) || !is_walkable(grid_pos, grid, unit_radius) {
            return false;
        }
    }
    
    true
}

/// Create a flow field for mass unit movement
pub fn create_flow_field(
    target: Vec2,
    grid: &PathfindingGrid,
    grid_size: f32,
) -> HashMap<(i32, i32), Vec2> {
    let target_grid = world_to_grid(target, grid_size);
    let mut flow_field = HashMap::new();
    
    // Use Dijkstra's algorithm to calculate distance field
    let mut open_set = std::collections::BinaryHeap::new();
    let mut cost_so_far = HashMap::new();
    
    // Use negative cost for max-heap to work as min-heap
    open_set.push(std::cmp::Reverse((0, target_grid)));
    cost_so_far.insert(target_grid, 0);
    
    while let Some(std::cmp::Reverse((cost, current))) = open_set.pop() {
        // Get neighbors
        let (x, y) = current;
        let neighbors = [
            (x - 1, y - 1), (x, y - 1), (x + 1, y - 1),
            (x - 1, y),                 (x + 1, y),
            (x - 1, y + 1), (x, y + 1), (x + 1, y + 1),
        ];
        
        for &next in &neighbors {
            if !is_in_bounds(next, grid) || !is_walkable(next, grid, 0.0) {
                continue;
            }
            
            // Diagonal movement costs more
            let move_cost = if next.0 != x && next.1 != y { 1.4 } else { 1.0 };
            
            // Add terrain cost
            let idx = grid_pos_to_index(next, grid.width);
            let node_cost = grid.nodes[idx].cost;
            let new_cost = cost + (move_cost * node_cost).round() as i32;
            
            if !cost_so_far.contains_key(&next) || new_cost < *cost_so_far.get(&next).unwrap() {
                cost_so_far.insert(next, new_cost);
                open_set.push(std::cmp::Reverse((new_cost, next)));
            }
        }
    }
    
    // Create flow vectors pointing in direction of decreasing cost
    for (&pos, &cost) in &cost_so_far {
        let (x, y) = pos;
        let neighbors = [
            (x - 1, y - 1), (x, y - 1), (x + 1, y - 1),
            (x - 1, y),                 (x + 1, y),
            (x - 1, y + 1), (x, y + 1), (x + 1, y + 1),
        ];
        
        // Find neighbor with lowest cost
        let mut best_dir = Vec2::ZERO;
        let mut best_cost = cost;
        
        for &next in &neighbors {
            if let Some(&next_cost) = cost_so_far.get(&next) {
                if next_cost < best_cost {
                    best_cost = next_cost;
                    best_dir = Vec2::new(
                        (next.0 - x) as f32,
                        (next.1 - y) as f32,
                    ).normalize();
                }
            }
        }
        
        // If no better direction found, this is the target or isolated
        if best_dir != Vec2::ZERO {
            flow_field.insert(pos, best_dir);
        }
    }
    
    flow_field
}