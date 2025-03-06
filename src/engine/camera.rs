use glam::Vec2;
use crate::ecs::resources::CameraState;

/// Camera controller for the game view
pub struct CameraController {
    pub position: Vec2,
    pub zoom: f32,
    pub view_width: f32,
    pub view_height: f32,
    pub world_width: f32,
    pub world_height: f32,
    pub movement_speed: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl CameraController {
    pub fn new(world_width: f32, world_height: f32, view_width: f32, view_height: f32) -> Self {
        Self {
            position: Vec2::new(world_width / 2.0, world_height / 2.0),
            zoom: 1.0,
            view_width,
            view_height,
            world_width,
            world_height,
            movement_speed: 200.0,
            zoom_speed: 0.1,
            min_zoom: 0.5,
            max_zoom: 2.0,
        }
    }
    
    /// Update camera position and zoom
    pub fn update(&mut self, delta_time: f32) {
        // Add any physics or smoothing update here if needed
        // Clamp position to world bounds
        let half_view_width = self.view_width / (2.0 * self.zoom);
        let half_view_height = self.view_height / (2.0 * self.zoom);
        
        self.position.x = self.position.x.clamp(
            half_view_width,
            self.world_width - half_view_width,
        );
        
        self.position.y = self.position.y.clamp(
            half_view_height,
            self.world_height - half_view_height,
        );
    }
    
    /// Move camera by direction vector
    pub fn move_camera(&mut self, direction: Vec2) {
        let speed = self.movement_speed / self.zoom; // Adjust speed based on zoom level
        self.position += direction * speed;
    }
    
    /// Zoom camera by delta amount
    pub fn zoom_camera(&mut self, delta: f32) {
        self.zoom = (self.zoom + delta * self.zoom_speed).clamp(self.min_zoom, self.max_zoom);
    }
    
    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let half_view_width = self.view_width / 2.0;
        let half_view_height = self.view_height / 2.0;
        
        let screen_center = Vec2::new(half_view_width, half_view_height);
        let screen_to_center = screen_pos - screen_center;
        
        // Scale by zoom and add camera position
        let world_pos = self.position + screen_to_center / self.zoom;
        
        world_pos
    }
    
    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let half_view_width = self.view_width / 2.0;
        let half_view_height = self.view_height / 2.0;
        
        let world_to_camera = world_pos - self.position;
        
        // Scale by zoom and add screen center
        let screen_pos = Vec2::new(half_view_width, half_view_height) + world_to_camera * self.zoom;
        
        screen_pos
    }
    
    /// Resize the view
    pub fn resize(&mut self, width: f32, height: f32) {
        self.view_width = width;
        self.view_height = height;
        
        // Clamp position after resize to prevent showing outside world bounds
        self.update(0.0);
    }
    
    /// Get the current camera state as a resource
    pub fn get_camera_state(&self) -> CameraState {
        CameraState {
            position: self.position,
            zoom: self.zoom,
            view_width: self.view_width,
            view_height: self.view_height,
        }
    }
    
    /// Calculate the view-projection matrix for rendering
    pub fn calculate_view_projection_matrix(&self) -> glam::Mat4 {
        // First, create orthographic projection matrix
        let left = -self.view_width / (2.0 * self.zoom);
        let right = self.view_width / (2.0 * self.zoom);
        let bottom = self.view_height / (2.0 * self.zoom);
        let top = -self.view_height / (2.0 * self.zoom);
        
        let ortho = glam::Mat4::orthographic_rh(left, right, bottom, top, -1.0, 1.0);
        
        // Then, create view matrix (camera transform)
        let view = glam::Mat4::from_translation(glam::Vec3::new(-self.position.x, -self.position.y, 0.0));
        
        // Combine for view-projection matrix
        ortho * view
    }
    
    /// Get visible world bounds
    pub fn get_visible_bounds(&self) -> (Vec2, Vec2) {
        let half_view_width = self.view_width / (2.0 * self.zoom);
        let half_view_height = self.view_height / (2.0 * self.zoom);
        
        let min = Vec2::new(
            self.position.x - half_view_width,
            self.position.y - half_view_height,
        );
        
        let max = Vec2::new(
            self.position.x + half_view_width,
            self.position.y + half_view_height,
        );
        
        (min, max)
    }
}