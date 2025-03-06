use glam::{Vec2, Vec4};
use wgpu::RenderPass;
use std::any::Any;

use crate::ui::{UiElement, UiElementType, UiPipeline, UiAlignment};

pub struct Button {
    position: Vec2,
    size: Vec2,
    text: String,
    visible: bool,
    enabled: bool,
    hovered: bool,
    color_normal: Vec4,
    color_hovered: Vec4,
    color_pressed: Vec4,
    color_disabled: Vec4,
    text_color: Vec4,
    alignment: UiAlignment,
    callback: Option<Box<dyn Fn() -> bool + 'static>>,
}

impl Button {
    pub fn new(position: Vec2, size: Vec2, text: &str) -> Self {
        Self {
            position,
            size,
            text: text.to_string(),
            visible: true,
            enabled: true,
            hovered: false,
            color_normal: Vec4::new(0.3, 0.3, 0.3, 1.0),
            color_hovered: Vec4::new(0.4, 0.4, 0.4, 1.0),
            color_pressed: Vec4::new(0.5, 0.5, 0.5, 1.0),
            color_disabled: Vec4::new(0.2, 0.2, 0.2, 0.5),
            text_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            alignment: UiAlignment::Center,
            callback: None,
        }
    }
    
    pub fn with_colors(mut self, normal: Vec4, hovered: Vec4, pressed: Vec4, disabled: Vec4) -> Self {
        self.color_normal = normal;
        self.color_hovered = hovered;
        self.color_pressed = pressed;
        self.color_disabled = disabled;
        self
    }
    
    pub fn with_text_color(mut self, color: Vec4) -> Self {
        self.text_color = color;
        self
    }
    
    pub fn with_alignment(mut self, alignment: UiAlignment) -> Self {
        self.alignment = alignment;
        self
    }
    
    pub fn set_callback<F: Fn() -> bool + 'static>(&mut self, callback: F) {
        self.callback = Some(Box::new(callback));
    }
    
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }
    
    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }
    
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl UiElement for Button {
    fn get_type(&self) -> UiElementType {
        UiElementType::Button
    }
    
    fn get_position(&self) -> Vec2 {
        self.position
    }
    
    fn get_size(&self) -> Vec2 {
        self.size
    }
    
    fn is_visible(&self) -> bool {
        self.visible
    }
    
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    
    fn contains_point(&self, point: Vec2) -> bool {
        self.visible && self.enabled &&
        point.x >= self.position.x &&
        point.x <= self.position.x + self.size.x &&
        point.y >= self.position.y &&
        point.y <= self.position.y + self.size.y
    }
    
    fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        if !self.visible {
            return;
        }
        
        // Set up pipeline
        render_pass.set_pipeline(&ui_pipeline.pipeline);
        render_pass.set_vertex_buffer(0, ui_pipeline.vertex_buffer.slice(..));
        render_pass.set_index_buffer(ui_pipeline.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        
        // In a real implementation, we would:
        // 1. Create vertices for the button based on position and size
        // 2. Update vertex buffer or use instance data
        // 3. Set the proper color based on state (normal, hovered, pressed, disabled)
        // 4. Draw the button background
        // 5. Draw the button text
        
        // For now, we'll just draw the button using the default quad
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
    
    fn handle_click(&mut self, _position: Vec2) -> bool {
        if !self.visible || !self.enabled {
            return false;
        }
        
        if let Some(callback) = &self.callback {
            callback()
        } else {
            true
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}