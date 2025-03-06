use std::time::{Duration, Instant};

/// Time system to manage game timing, ticks, and frame pacing
pub struct TimeSystem {
    tick_rate: f64,              // Ticks per second
    ms_per_tick: f64,            // Milliseconds per tick
    last_update: Instant,        // When the last update occurred
    accumulated_time: f64,       // Time accumulated since last tick
    current_tick: u64,           // Current tick counter
    time_scale: f64,             // Time scaling factor (1.0 = normal speed)
    frame_times: Vec<f64>,       // Recent frame times for FPS calculation
    fps: f64,                    // Current calculated FPS
    delta_time: f64,             // Time between frames
    elapsed_time: f64,           // Total elapsed time in seconds
}

impl TimeSystem {
    pub fn new(tick_rate: f64) -> Self {
        let ms_per_tick = 1000.0 / tick_rate;
        
        Self {
            tick_rate,
            ms_per_tick,
            last_update: Instant::now(),
            accumulated_time: 0.0,
            current_tick: 0,
            time_scale: 1.0,
            frame_times: Vec::with_capacity(120),
            fps: 0.0,
            delta_time: 1.0 / 60.0, // Default to 60 FPS
            elapsed_time: 0.0,
        }
    }
    
    /// Check if we should perform a tick based on accumulated time
    pub fn should_tick(&mut self) -> bool {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update).as_secs_f64() * 1000.0; // in ms
        self.last_update = now;
        
        // Calculate delta time in seconds
        self.delta_time = delta / 1000.0;
        
        // Update elapsed time
        self.elapsed_time += self.delta_time;
        
        // Update FPS calculation
        self.update_fps(delta);
        
        // Accumulate time (scaled by time_scale)
        self.accumulated_time += delta * self.time_scale;
        
        // Check if we should perform a tick
        if self.accumulated_time >= self.ms_per_tick {
            return true;
        }
        
        false
    }
    
    /// Call this when a tick is complete to update timing
    pub fn tick_completed(&mut self) {
        self.accumulated_time -= self.ms_per_tick;
        self.current_tick += 1;
        
        // Prevent accumulating too much time
        // (Useful to avoid spiral of death if game lags and can't catch up)
        const MAX_ACCUMULATED_TIME: f64 = 200.0; // ms
        if self.accumulated_time > MAX_ACCUMULATED_TIME {
            self.accumulated_time = MAX_ACCUMULATED_TIME;
        }
    }
    
    /// Set time scale factor (1.0 = normal, 2.0 = double speed, 0.5 = half speed)
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.max(0.1).min(10.0);
    }
    
    /// Get current time scale
    pub fn get_time_scale(&self) -> f64 {
        self.time_scale
    }
    
    /// Get current tick
    pub fn get_current_tick(&self) -> u64 {
        self.current_tick
    }
    
    /// Get delta time (time between frames) in seconds
    pub fn get_delta_time(&self) -> f32 {
        self.delta_time as f32
    }
    
    /// Get current FPS
    pub fn get_fps(&self) -> f64 {
        self.fps
    }
    
    /// Get total elapsed time in seconds
    pub fn get_elapsed_time(&self) -> f64 {
        self.elapsed_time
    }
    
    /// Update FPS calculation with new frame time
    fn update_fps(&mut self, frame_time_ms: f64) {
        // Add current frame time
        self.frame_times.push(frame_time_ms);
        
        // Keep only recent frames for calculation (last second)
        while self.frame_times.len() > 120 {
            self.frame_times.remove(0);
        }
        
        // Calculate average frame time
        if !self.frame_times.is_empty() {
            let sum: f64 = self.frame_times.iter().sum();
            let avg_frame_time = sum / self.frame_times.len() as f64;
            
            // Convert to FPS
            self.fps = 1000.0 / avg_frame_time;
        }
    }
    
    /// Get a formatted time string (HH:MM:SS)
    pub fn get_formatted_time(&self) -> String {
        let total_seconds = self.elapsed_time as u64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
    
    /// Reset the time system
    pub fn reset(&mut self) {
        self.last_update = Instant::now();
        self.accumulated_time = 0.0;
        self.current_tick = 0;
        self.elapsed_time = 0.0;
        self.frame_times.clear();
    }
}