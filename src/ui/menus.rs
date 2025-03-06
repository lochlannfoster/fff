use glam::{Vec2, Vec4};
use wgpu::RenderPass;
use std::collections::HashMap;

use crate::ui::{UiPipeline, UiElement, UiElementType, UiAlignment, UiColorScheme};

/// Button UI element
pub struct Button {
    position: Vec2,
    size: Vec2,
    text: String,
    visible: bool,
    enabled: bool,
    hovered: bool,
    clicked: bool,
    color_normal: Vec4,
    color_hovered: Vec4,
    color_pressed: Vec4,
    color_disabled: Vec4,
    text_color: Vec4,
    text_color_disabled: Vec4,
    icon: Option<String>,
    alignment: UiAlignment,
    on_click: Option<Box<dyn Fn() -> bool>>,
}

impl Button {
    pub fn new(
        position: Vec2,
        size: Vec2,
        text: &str,
        color_scheme: &UiColorScheme,
    ) -> Self {
        Self {
            position,
            size,
            text: text.to_string(),
            visible: true,
            enabled: true,
            hovered: false,
            clicked: false,
            color_normal: color_scheme.button,
            color_hovered: color_scheme.button_hover,
            color_pressed: color_scheme.button_active,
            color_disabled: Vec4::new(0.3, 0.3, 0.3, 0.5),
            text_color: color_scheme.text,
            text_color_disabled: Vec4::new(0.5, 0.5, 0.5, 0.5),
            icon: None,
            alignment: UiAlignment::Center,
            on_click: None,
        }
    }
    
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }
    
    pub fn with_alignment(mut self, alignment: UiAlignment) -> Self {
        self.alignment = alignment;
        self
    }
    
    pub fn with_colors(
        mut self,
        normal: Vec4,
        hovered: Vec4,
        pressed: Vec4,
        disabled: Vec4,
    ) -> Self {
        self.color_normal = normal;
        self.color_hovered = hovered;
        self.color_pressed = pressed;
        self.color_disabled = disabled;
        self
    }
    
    pub fn with_text_colors(mut self, normal: Vec4, disabled: Vec4) -> Self {
        self.text_color = normal;
        self.text_color_disabled = disabled;
        self
    }
    
    pub fn set_on_click<F: Fn() -> bool + 'static>(&mut self, callback: F) {
        self.on_click = Some(Box::new(callback));
    }
    
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
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
        point.x >= self.position.x &&
        point.x <= self.position.x + self.size.x &&
        point.y >= self.position.y &&
        point.y <= self.position.y + self.size.y
    }
    
    fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render the button using the UI pipeline
        // This is a placeholder
    }
    
    fn handle_click(&mut self, position: Vec2) -> bool {
        if !self.enabled || !self.visible {
            return false;
        }
        
        self.clicked = true;
        
        if let Some(callback) = &self.on_click {
            callback()
        } else {
            true
        }
    }
}

/// Panel UI element
pub struct Panel {
    position: Vec2,
    size: Vec2,
    visible: bool,
    color: Vec4,
    border_color: Vec4,
    border_width: f32,
    title: Option<String>,
    draggable: bool,
    elements: HashMap<String, Box<dyn UiElement>>,
}

impl Panel {
    pub fn new(
        position: Vec2,
        size: Vec2,
        color_scheme: &UiColorScheme,
    ) -> Self {
        Self {
            position,
            size,
            visible: true,
            color: color_scheme.background,
            border_color: color_scheme.border,
            border_width: 1.0,
            title: None,
            draggable: false,
            elements: HashMap::new(),
        }
    }
    
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }
    
    pub fn with_border(mut self, width: f32, color: Vec4) -> Self {
        self.border_width = width;
        self.border_color = color;
        self
    }
    
    pub fn with_draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }
    
    pub fn add_element(&mut self, id: &str, element: Box<dyn UiElement>) {
        self.elements.insert(id.to_string(), element);
    }
    
    pub fn get_element(&self, id: &str) -> Option<&Box<dyn UiElement>> {
        self.elements.get(id)
    }
    
    pub fn get_element_mut(&mut self, id: &str) -> Option<&mut Box<dyn UiElement>> {
        self.elements.get_mut(id)
    }
    
    pub fn remove_element(&mut self, id: &str) -> Option<Box<dyn UiElement>> {
        self.elements.remove(id)
    }
}

impl UiElement for Panel {
    fn get_type(&self) -> UiElementType {
        UiElementType::Panel
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
        point.x >= self.position.x &&
        point.x <= self.position.x + self.size.x &&
        point.y >= self.position.y &&
        point.y <= self.position.y + self.size.y
    }
    
    fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render the panel and its elements
        // This is a placeholder
    }
    
    fn handle_click(&mut self, position: Vec2) -> bool {
        // Check if any child element was clicked
        for element in self.elements.values_mut() {
            if element.is_visible() && element.contains_point(position) {
                return element.handle_click(position);
            }
        }
        
        // Just return true if panel itself was clicked
        true
    }
}

/// Label UI element
pub struct Label {
    position: Vec2,
    size: Vec2,
    text: String,
    visible: bool,
    color: Vec4,
    font_size: f32,
    alignment: UiAlignment,
}

impl Label {
    pub fn new(
        position: Vec2,
        size: Vec2,
        text: &str,
        color_scheme: &UiColorScheme,
    ) -> Self {
        Self {
            position,
            size,
            text: text.to_string(),
            visible: true,
            color: color_scheme.text,
            font_size: 16.0,
            alignment: UiAlignment::Center,
        }
    }
    
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }
    
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    
    pub fn with_alignment(mut self, alignment: UiAlignment) -> Self {
        self.alignment = alignment;
        self
    }
    
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}

impl UiElement for Label {
    fn get_type(&self) -> UiElementType {
        UiElementType::Text
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
        point.x >= self.position.x &&
        point.x <= self.position.x + self.size.x &&
        point.y >= self.position.y &&
        point.y <= self.position.y + self.size.y
    }
    
    fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render the text
        // This is a placeholder
    }
    
    fn handle_click(&mut self, _position: Vec2) -> bool {
        // Labels don't handle clicks
        false
    }
}

/// Checkbox UI element
pub struct Checkbox {
    position: Vec2,
    size: Vec2,
    text: String,
    visible: bool,
    enabled: bool,
    checked: bool,
    color: Vec4,
    text_color: Vec4,
    check_color: Vec4,
    on_change: Option<Box<dyn Fn(bool) -> bool>>,
}

impl Checkbox {
    pub fn new(
        position: Vec2,
        size: Vec2,
        text: &str,
        color_scheme: &UiColorScheme,
    ) -> Self {
        Self {
            position,
            size,
            text: text.to_string(),
            visible: true,
            enabled: true,
            checked: false,
            color: color_scheme.button,
            text_color: color_scheme.text,
            check_color: color_scheme.accent,
            on_change: None,
        }
    }
    
    pub fn with_checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
    
    pub fn with_colors(mut self, box_color: Vec4, text_color: Vec4, check_color: Vec4) -> Self {
        self.color = box_color;
        self.text_color = text_color;
        self.check_color = check_color;
        self
    }
    
    pub fn set_on_change<F: Fn(bool) -> bool + 'static>(&mut self, callback: F) {
        self.on_change = Some(Box::new(callback));
    }
    
    pub fn is_checked(&self) -> bool {
        self.checked
    }
    
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }
}

impl UiElement for Checkbox {
    fn get_type(&self) -> UiElementType {
        UiElementType::Button // Reuse button type
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
        point.x >= self.position.x &&
        point.x <= self.position.x + self.size.x &&
        point.y >= self.position.y &&
        point.y <= self.position.y + self.size.y
    }
    
    fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render the checkbox
        // This is a placeholder
    }
    
    fn handle_click(&mut self, _position: Vec2) -> bool {
        if !self.enabled || !self.visible {
            return false;
        }
        
        // Toggle checked state
        self.checked = !self.checked;
        
        if let Some(callback) = &self.on_change {
            callback(self.checked)
        } else {
            true
        }
    }
}

/// Slider UI element
pub struct Slider {
    position: Vec2,
    size: Vec2,
    visible: bool,
    enabled: bool,
    value: f32,
    min_value: f32,
    max_value: f32,
    track_color: Vec4,
    handle_color: Vec4,
    handle_size: f32,
    label: Option<String>,
    on_change: Option<Box<dyn Fn(f32) -> bool>>,
}

impl Slider {
    pub fn new(
        position: Vec2,
        size: Vec2,
        color_scheme: &UiColorScheme,
    ) -> Self {
        Self {
            position,
            size,
            visible: true,
            enabled: true,
            value: 0.5,
            min_value: 0.0,
            max_value: 1.0,
            track_color: color_scheme.background,
            handle_color: color_scheme.accent,
            handle_size: 16.0,
            label: None,
            on_change: None,
        }
    }
    
    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.min_value = min;
        self.max_value = max;
        self
    }
    
    pub fn with_value(mut self, value: f32) -> Self {
        self.value = value.max(self.min_value).min(self.max_value);
        self
    }
    
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }
    
    pub fn with_colors(mut self, track_color: Vec4, handle_color: Vec4) -> Self {
        self.track_color = track_color;
        self.handle_color = handle_color;
        self
    }
    
    pub fn set_on_change<F: Fn(f32) -> bool + 'static>(&mut self, callback: F) {
        self.on_change = Some(Box::new(callback));
    }
    
    pub fn get_value(&self) -> f32 {
        self.value
    }
    
    pub fn set_value(&mut self, value: f32) {
        self.value = value.max(self.min_value).min(self.max_value);
    }
}

impl UiElement for Slider {
    fn get_type(&self) -> UiElementType {
        UiElementType::Button // Reuse button type
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
        point.x >= self.position.x &&
        point.x <= self.position.x + self.size.x &&
        point.y >= self.position.y &&
        point.y <= self.position.y + self.size.y
    }
    
    fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render the slider
        // This is a placeholder
    }
    
    fn handle_click(&mut self, position: Vec2) -> bool {
        if !self.enabled || !self.visible {
            return false;
        }
        
        // Calculate normalized position
        let normalized = (position.x - self.position.x) / self.size.x;
        let normalized = normalized.max(0.0).min(1.0);
        
        // Convert to value
        let value = self.min_value + normalized * (self.max_value - self.min_value);
        self.value = value;
        
        if let Some(callback) = &self.on_change {
            callback(value)
        } else {
            true
        }
    }
}
/// Dropdown UI element
pub struct Dropdown {
    position: Vec2,
    size: Vec2,
    visible: bool,
    enabled: bool,
    options: Vec<String>,
    selected_index: usize,
    is_open: bool,
    color: Vec4,
    text_color: Vec4,
    highlight_color: Vec4,
    on_select: Option<Box<dyn Fn(usize, &str) -> bool>>,
}

impl Dropdown {
    pub fn new(
        position: Vec2,
        size: Vec2,
        options: Vec<String>,
        color_scheme: &UiColorScheme,
    ) -> Self {
        Self {
            position,
            size,
            visible: true,
            enabled: true,
            options,
            selected_index: 0,
            is_open: false,
            color: color_scheme.button,
            text_color: color_scheme.text,
            highlight_color: color_scheme.accent,
            on_select: None,
        }
    }
    
    pub fn with_selected_index(mut self, index: usize) -> Self {
        if index < self.options.len() {
            self.selected_index = index;
        }
        self
    }
    
    pub fn with_colors(mut self, color: Vec4, text_color: Vec4, highlight_color: Vec4) -> Self {
        self.color = color;
        self.text_color = text_color;
        self.highlight_color = highlight_color;
        self
    }
    
    pub fn set_on_select<F: Fn(usize, &str) -> bool + 'static>(&mut self, callback: F) {
        self.on_select = Some(Box::new(callback));
    }
    
    pub fn get_selected_index(&self) -> usize {
        self.selected_index
    }
    
    pub fn get_selected_option(&self) -> Option<&str> {
        self.options.get(self.selected_index).map(|s| s.as_str())
    }
    
    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected_index = index;
        }
    }
    
    pub fn set_options(&mut self, options: Vec<String>) {
        self.options = options;
        if self.selected_index >= self.options.len() && !self.options.is_empty() {
            self.selected_index = 0;
        }
    }
}

impl UiElement for Dropdown {
    fn get_type(&self) -> UiElementType {
        UiElementType::Button // Reuse button type
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
        if !self.is_open {
            // Just check main dropdown area
            return point.x >= self.position.x &&
                   point.x <= self.position.x + self.size.x &&
                   point.y >= self.position.y &&
                   point.y <= self.position.y + self.size.y;
        } else {
            // Check dropdown area plus dropdown list
            let dropdown_height = self.options.len() as f32 * self.size.y;
            
            return point.x >= self.position.x &&
                   point.x <= self.position.x + self.size.x &&
                   point.y >= self.position.y &&
                   point.y <= self.position.y + self.size.y + dropdown_height;
        }
    }
    
    fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render the dropdown
        // This is a placeholder
    }
    
    fn handle_click(&mut self, position: Vec2) -> bool {
        if !self.enabled || !self.visible || self.options.is_empty() {
            return false;
        }
        
        if !self.is_open {
            // Open the dropdown
            self.is_open = true;
            return true;
        } else {
            // Check if click is on the main dropdown area
            if position.y >= self.position.y && position.y <= self.position.y + self.size.y {
                // Close the dropdown
                self.is_open = false;
                return true;
            }
            
            // Check if click is on one of the options
            let option_height = self.size.y;
            let option_index = ((position.y - (self.position.y + self.size.y)) / option_height) as usize;
            
            if option_index < self.options.len() {
                // Select this option
                let old_index = self.selected_index;
                self.selected_index = option_index;
                self.is_open = false;
                
                if old_index != option_index {
                    if let Some(callback) = &self.on_select {
                        return callback(option_index, &self.options[option_index]);
                    }
                }
                
                return true;
            }
            
            // Click outside dropdown, close it
            self.is_open = false;
            return true;
        }
    }
}

/// Progress bar UI element
pub struct ProgressBar {
    position: Vec2,
    size: Vec2,
    visible: bool,
    value: f32,
    background_color: Vec4,
    fill_color: Vec4,
    border_color: Vec4,
    border_width: f32,
    label: Option<String>,
    show_percentage: bool,
}

impl ProgressBar {
    pub fn new(
        position: Vec2,
        size: Vec2,
        color_scheme: &UiColorScheme,
    ) -> Self {
        Self {
            position,
            size,
            visible: true,
            value: 0.0,
            background_color: color_scheme.background,
            fill_color: color_scheme.accent,
            border_color: color_scheme.border,
            border_width: 1.0,
            label: None,
            show_percentage: false,
        }
    }
    
    pub fn with_value(mut self, value: f32) -> Self {
        self.value = value.max(0.0).min(1.0);
        self
    }
    
    pub fn with_colors(mut self, background: Vec4, fill: Vec4, border: Vec4) -> Self {
        self.background_color = background;
        self.fill_color = fill;
        self.border_color = border;
        self
    }
    
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }
    
    pub fn with_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }
    
    pub fn set_value(&mut self, value: f32) {
        self.value = value.max(0.0).min(1.0);
    }
    
    pub fn get_value(&self) -> f32 {
        self.value
    }
}

impl UiElement for ProgressBar {
    fn get_type(&self) -> UiElementType {
        UiElementType::ProgressBar
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
        point.x >= self.position.x &&
        point.x <= self.position.x + self.size.x &&
        point.y >= self.position.y &&
        point.y <= self.position.y + self.size.y
    }
    
    fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render the progress bar
        // This is a placeholder
    }
    
    fn handle_click(&mut self, _position: Vec2) -> bool {
        // Progress bars don't handle clicks
        false
    }
}

/// Image UI element
pub struct Image {
    position: Vec2,
    size: Vec2,
    visible: bool,
    texture_name: String,
    tint_color: Vec4,
    on_click: Option<Box<dyn Fn() -> bool>>,
}

impl Image {
    pub fn new(
        position: Vec2,
        size: Vec2,
        texture_name: &str,
    ) -> Self {
        Self {
            position,
            size,
            visible: true,
            texture_name: texture_name.to_string(),
            tint_color: Vec4::new(1.0, 1.0, 1.0, 1.0), // No tint
            on_click: None,
        }
    }
    
    pub fn with_tint(mut self, tint: Vec4) -> Self {
        self.tint_color = tint;
        self
    }
    
    pub fn set_on_click<F: Fn() -> bool + 'static>(&mut self, callback: F) {
        self.on_click = Some(Box::new(callback));
    }
    
    pub fn set_texture(&mut self, texture_name: &str) {
        self.texture_name = texture_name.to_string();
    }
}

impl UiElement for Image {
    fn get_type(&self) -> UiElementType {
        UiElementType::Image
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
        point.x >= self.position.x &&
        point.x <= self.position.x + self.size.x &&
        point.y >= self.position.y &&
        point.y <= self.position.y + self.size.y
    }
    
    fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, ui_pipeline: &'a UiPipeline) {
        // In a real implementation, this would render the image
        // This is a placeholder
    }
    
    fn handle_click(&mut self, _position: Vec2) -> bool {
        if !self.visible {
            return false;
        }
        
        if let Some(callback) = &self.on_click {
            callback()
        } else {
            false
        }
    }
}
/// Menu factory for creating common menu screens
pub struct MenuFactory {
    color_scheme: UiColorScheme,
    screen_size: Vec2,
}

impl MenuFactory {
    pub fn new(color_scheme: UiColorScheme, screen_width: u32, screen_height: u32) -> Self {
        Self {
            color_scheme,
            screen_size: Vec2::new(screen_width as f32, screen_height as f32),
        }
    }
    
    pub fn create_main_menu(&self) -> HashMap<String, Box<dyn UiElement>> {
        let mut elements = HashMap::new();
        
        // Title
        elements.insert("title".to_string(), Box::new(Label::new(
            Vec2::new(self.screen_size.x / 2.0 - 200.0, 100.0),
            Vec2::new(400.0, 80.0),
            "Rusty RTS",
            &self.color_scheme,
        ).with_font_size(48.0)));
        
        // Play button
        elements.insert("play_button".to_string(), Box::new(Button::new(
            Vec2::new(self.screen_size.x / 2.0 - 100.0, 250.0),
            Vec2::new(200.0, 50.0),
            "New Game",
            &self.color_scheme,
        )));
        
        // Multiplayer button
        elements.insert("multiplayer_button".to_string(), Box::new(Button::new(
            Vec2::new(self.screen_size.x / 2.0 - 100.0, 320.0),
            Vec2::new(200.0, 50.0),
            "Multiplayer",
            &self.color_scheme,
        )));
        
        // Settings button
        elements.insert("settings_button".to_string(), Box::new(Button::new(
            Vec2::new(self.screen_size.x / 2.0 - 100.0, 390.0),
            Vec2::new(200.0, 50.0),
            "Settings",
            &self.color_scheme,
        )));
        
        // Exit button
        elements.insert("exit_button".to_string(), Box::new(Button::new(
            Vec2::new(self.screen_size.x / 2.0 - 100.0, 460.0),
            Vec2::new(200.0, 50.0),
            "Exit Game",
            &self.color_scheme,
        )));
        
        elements
    }
    
    pub fn create_settings_menu(&self) -> HashMap<String, Box<dyn UiElement>> {
        let mut elements = HashMap::new();
        
        // Title
        elements.insert("title".to_string(), Box::new(Label::new(
            Vec2::new(self.screen_size.x / 2.0 - 200.0, 50.0),
            Vec2::new(400.0, 50.0),
            "Settings",
            &self.color_scheme,
        ).with_font_size(32.0)));
        
        // Panel
        let mut panel = Panel::new(
            Vec2::new(self.screen_size.x / 2.0 - 250.0, 120.0),
            Vec2::new(500.0, 400.0),
            &self.color_scheme,
        );
        
        // Game settings
        let music_volume_slider = Box::new(Slider::new(
            Vec2::new(150.0, 20.0),
            Vec2::new(250.0, 30.0),
            &self.color_scheme,
        ).with_label("Music Volume").with_value(0.7));
        
        let sfx_volume_slider = Box::new(Slider::new(
            Vec2::new(150.0, 60.0),
            Vec2::new(250.0, 30.0),
            &self.color_scheme,
        ).with_label("Sound Effects Volume").with_value(0.8));
        
        let fullscreen_checkbox = Box::new(Checkbox::new(
            Vec2::new(150.0, 100.0),
            Vec2::new(250.0, 30.0),
            "Fullscreen",
            &self.color_scheme,
        ));
        
        let show_fps_checkbox = Box::new(Checkbox::new(
            Vec2::new(150.0, 140.0),
            Vec2::new(250.0, 30.0),
            "Show FPS",
            &self.color_scheme,
        ));
        
        let fog_of_war_checkbox = Box::new(Checkbox::new(
            Vec2::new(150.0, 180.0),
            Vec2::new(250.0, 30.0),
            "Enable Fog of War",
            &self.color_scheme,
        ).with_checked(true));
        
        let game_speed_slider = Box::new(Slider::new(
            Vec2::new(150.0, 220.0),
            Vec2::new(250.0, 30.0),
            &self.color_scheme,
        ).with_label("Game Speed").with_range(0.5, 2.0).with_value(1.0));
        
        let back_button = Box::new(Button::new(
            Vec2::new(150.0, 320.0),
            Vec2::new(200.0, 40.0),
            "Back",
            &self.color_scheme,
        ));
        
        panel.add_element("music_volume", music_volume_slider);
        panel.add_element("sfx_volume", sfx_volume_slider);
        panel.add_element("fullscreen", fullscreen_checkbox);
        panel.add_element("show_fps", show_fps_checkbox);
        panel.add_element("fog_of_war", fog_of_war_checkbox);
        panel.add_element("game_speed", game_speed_slider);
        panel.add_element("back_button", back_button);
        
        elements.insert("settings_panel".to_string(), Box::new(panel));
        
        elements
    }
    
    pub fn create_multiplayer_menu(&self) -> HashMap<String, Box<dyn UiElement>> {
        let mut elements = HashMap::new();
        
        // Title
        elements.insert("title".to_string(), Box::new(Label::new(
            Vec2::new(self.screen_size.x / 2.0 - 200.0, 50.0),
            Vec2::new(400.0, 50.0),
            "Multiplayer",
            &self.color_scheme,
        ).with_font_size(32.0)));
        
        // Panel
        let mut panel = Panel::new(
            Vec2::new(self.screen_size.x / 2.0 - 250.0, 120.0),
            Vec2::new(500.0, 400.0),
            &self.color_scheme,
        );
        
        // Host game button
        let host_button = Box::new(Button::new(
            Vec2::new(150.0, 50.0),
            Vec2::new(200.0, 40.0),
            "Host Game",
            &self.color_scheme,
        ));
        
        // Join game button
        let join_button = Box::new(Button::new(
            Vec2::new(150.0, 110.0),
            Vec2::new(200.0, 40.0),
            "Join Game",
            &self.color_scheme,
        ));
        
        // IP address input (placeholder; in a real implementation, this would be a text input)
        let ip_label = Box::new(Label::new(
            Vec2::new(50.0, 170.0),
            Vec2::new(100.0, 30.0),
            "IP Address:",
            &self.color_scheme,
        ));
        
        // Server list (placeholder; in a real implementation, this would be a list box)
        let server_list_label = Box::new(Label::new(
            Vec2::new(50.0, 220.0),
            Vec2::new(100.0, 30.0),
            "Available Servers:",
            &self.color_scheme,
        ));
        
        // Back button
        let back_button = Box::new(Button::new(
            Vec2::new(150.0, 320.0),
            Vec2::new(200.0, 40.0),
            "Back",
            &self.color_scheme,
        ));
        
        panel.add_element("host_button", host_button);
        panel.add_element("join_button", join_button);
        panel.add_element("ip_label", ip_label);
        panel.add_element("server_list_label", server_list_label);
        panel.add_element("back_button", back_button);
        
        elements.insert("multiplayer_panel".to_string(), Box::new(panel));
        
        elements
    }
    
    pub fn create_game_setup_menu(&self) -> HashMap<String, Box<dyn UiElement>> {
        let mut elements = HashMap::new();
        
        // Title
        elements.insert("title".to_string(), Box::new(Label::new(
            Vec2::new(self.screen_size.x / 2.0 - 200.0, 50.0),
            Vec2::new(400.0, 50.0),
            "Game Setup",
            &self.color_scheme,
        ).with_font_size(32.0)));
        
        // Panel
        let mut panel = Panel::new(
            Vec2::new(self.screen_size.x / 2.0 - 250.0, 120.0),
            Vec2::new(500.0, 400.0),
            &self.color_scheme,
        );
        
        // Map selection
        let map_label = Box::new(Label::new(
            Vec2::new(50.0, 30.0),
            Vec2::new(100.0, 30.0),
            "Map:",
            &self.color_scheme,
        ));
        
        let map_dropdown = Box::new(Dropdown::new(
            Vec2::new(150.0, 30.0),
            Vec2::new(250.0, 30.0),
            vec![
                "Small Map (2 Players)".to_string(),
                "Medium Map (4 Players)".to_string(),
                "Large Map (8 Players)".to_string(),
            ],
            &self.color_scheme,
        ));
        
        // Players
        let players_label = Box::new(Label::new(
            Vec2::new(50.0, 70.0),
            Vec2::new(100.0, 30.0),
            "Players:",
            &self.color_scheme,
        ));
        
        let players_dropdown = Box::new(Dropdown::new(
            Vec2::new(150.0, 70.0),
            Vec2::new(250.0, 30.0),
            vec![
                "2 Players".to_string(),
                "3 Players".to_string(),
                "4 Players".to_string(),
            ],
            &self.color_scheme,
        ));
        
        // AI players
        let ai_label = Box::new(Label::new(
            Vec2::new(50.0, 110.0),
            Vec2::new(100.0, 30.0),
            "AI Players:",
            &self.color_scheme,
        ));
        
        let ai_dropdown = Box::new(Dropdown::new(
            Vec2::new(150.0, 110.0),
            Vec2::new(250.0, 30.0),
            vec![
                "None".to_string(),
                "1 AI".to_string(),
                "2 AI".to_string(),
                "3 AI".to_string(),
            ],
            &self.color_scheme,
        ));
        
        // AI difficulty
        let difficulty_label = Box::new(Label::new(
            Vec2::new(50.0, 150.0),
            Vec2::new(100.0, 30.0),
            "Difficulty:",
            &self.color_scheme,
        ));
        
        let difficulty_dropdown = Box::new(Dropdown::new(
            Vec2::new(150.0, 150.0),
            Vec2::new(250.0, 30.0),
            vec![
                "Easy".to_string(),
                "Medium".to_string(),
                "Hard".to_string(),
            ],
            &self.color_scheme,
        ));
        
        // Starting resources
        let resources_label = Box::new(Label::new(
            Vec2::new(50.0, 190.0),
            Vec2::new(100.0, 30.0),
            "Resources:",
            &self.color_scheme,
        ));
        
        let resources_dropdown = Box::new(Dropdown::new(
            Vec2::new(150.0, 190.0),
            Vec2::new(250.0, 30.0),
            vec![
                "Low".to_string(),
                "Normal".to_string(),
                "High".to_string(),
            ],
            &self.color_scheme,
        ).with_selected_index(1));
        
        // Victory condition
        let victory_label = Box::new(Label::new(
            Vec2::new(50.0, 230.0),
            Vec2::new(100.0, 30.0),
            "Victory:",
            &self.color_scheme,
        ));
        
        let victory_dropdown = Box::new(Dropdown::new(
            Vec2::new(150.0, 230.0),
            Vec2::new(250.0, 30.0),
            vec![
                "Annihilation".to_string(),
                "Time Limit".to_string(),
                "Resource Control".to_string(),
            ],
            &self.color_scheme,
        ));
        
        // Start game button
        let start_button = Box::new(Button::new(
            Vec2::new(150.0, 270.0),
            Vec2::new(200.0, 40.0),
            "Start Game",
            &self.color_scheme,
        ));
        
        // Back button
        let back_button = Box::new(Button::new(
            Vec2::new(150.0, 320.0),
            Vec2::new(200.0, 40.0),
            "Back",
            &self.color_scheme,
        ));
        
        panel.add_element("map_label", map_label);
        panel.add_element("map_dropdown", map_dropdown);
        panel.add_element("players_label", players_label);
        panel.add_element("players_dropdown", players_dropdown);
        panel.add_element("ai_label", ai_label);
        panel.add_element("ai_dropdown", ai_dropdown);
        panel.add_element("difficulty_label", difficulty_label);
        panel.add_element("difficulty_dropdown", difficulty_dropdown);
        panel.add_element("resources_label", resources_label);
        panel.add_element("resources_dropdown", resources_dropdown);
        panel.add_element("victory_label", victory_label);
        panel.add_element("victory_dropdown", victory_dropdown);
        panel.add_element("start_button", start_button);
        panel.add_element("back_button", back_button);
        
        elements.insert("setup_panel".to_string(), Box::new(panel));
        
        elements
    }
    
    pub fn create_pause_menu(&self) -> HashMap<String, Box<dyn UiElement>> {
        let mut elements = HashMap::new();
        
        // Title
        elements.insert("title".to_string(), Box::new(Label::new(
            Vec2::new(self.screen_size.x / 2.0 - 200.0, 50.0),
            Vec2::new(400.0, 50.0),
            "Game Paused",
            &self.color_scheme,
        ).with_font_size(32.0)));
        
        // Panel
        let mut panel = Panel::new(
            Vec2::new(self.screen_size.x / 2.0 - 150.0, 120.0),
            Vec2::new(300.0, 300.0),
            &self.color_scheme,
        );
        
        // Resume button
        let resume_button = Box::new(Button::new(
            Vec2::new(50.0, 50.0),
            Vec2::new(200.0, 40.0),
            "Resume Game",
            &self.color_scheme,
        ));
        
        // Settings button
        let settings_button = Box::new(Button::new(
            Vec2::new(50.0, 100.0),
            Vec2::new(200.0, 40.0),
            "Settings",
            &self.color_scheme,
        ));
        
        // Save game button
        let save_button = Box::new(Button::new(
            Vec2::new(50.0, 150.0),
            Vec2::new(200.0, 40.0),
            "Save Game",
            &self.color_scheme,
        ));
        
        // Load game button
        let load_button = Box::new(Button::new(
            Vec2::new(50.0, 200.0),
            Vec2::new(200.0, 40.0),
            "Load Game",
            &self.color_scheme,
        ));
        
        // Quit button
        let quit_button = Box::new(Button::new(
            Vec2::new(50.0, 250.0),
            Vec2::new(200.0, 40.0),
            "Quit to Main Menu",
            &self.color_scheme,
        ));
        
        panel.add_element("resume_button", resume_button);
        panel.add_element("settings_button", settings_button);
        panel.add_element("save_button", save_button);
        panel.add_element("load_button", load_button);
        panel.add_element("quit_button", quit_button);
        
        elements.insert("pause_panel".to_string(), Box::new(panel));
        
        elements
    }
    
    pub fn create_game_over_menu(&self, winner_name: &str) -> HashMap<String, Box<dyn UiElement>> {
        let mut elements = HashMap::new();
        
        // Title
        elements.insert("title".to_string(), Box::new(Label::new(
            Vec2::new(self.screen_size.x / 2.0 - 200.0, 50.0),
            Vec2::new(400.0, 50.0),
            "Game Over",
            &self.color_scheme,
        ).with_font_size(32.0)));
        
        // Winner label
        elements.insert("winner_label".to_string(), Box::new(Label::new(
            Vec2::new(self.screen_size.x / 2.0 - 200.0, 120.0),
            Vec2::new(400.0, 50.0),
            &format!("{} has won the game!", winner_name),
            &self.color_scheme,
        ).with_font_size(24.0)));
        
        // Panel for buttons
        let mut panel = Panel::new(
            Vec2::new(self.screen_size.x / 2.0 - 150.0, 200.0),
            Vec2::new(300.0, 200.0),
            &self.color_scheme,
        );
        
        // Play again button
        let play_again_button = Box::new(Button::new(
            Vec2::new(50.0, 50.0),
            Vec2::new(200.0, 40.0),
            "Play Again",
            &self.color_scheme,
        ));
        
        // View replay button
        let replay_button = Box::new(Button::new(
            Vec2::new(50.0, 100.0),
            Vec2::new(200.0, 40.0),
            "View Replay",
            &self.color_scheme,
        ));
        
        // Main menu button
        let main_menu_button = Box::new(Button::new(
            Vec2::new(50.0, 150.0),
            Vec2::new(200.0, 40.0),
            "Main Menu",
            &self.color_scheme,
        ));
        
        panel.add_element("play_again_button", play_again_button);
        panel.add_element("replay_button", replay_button);
        panel.add_element("main_menu_button", main_menu_button);
        
        elements.insert("game_over_panel".to_string(), Box::new(panel));
        
        elements
    }
}