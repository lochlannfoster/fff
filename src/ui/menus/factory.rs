use glam::Vec2;
use std::collections::HashMap;

use crate::ui::{UiElement, UiColorScheme, UiAlignment};

/// Creates pre-defined menu layouts
pub struct MenuFactory {
    color_scheme: UiColorScheme,
    screen_size: Vec2,
}

impl MenuFactory {
    /// Create a new menu factory with given screen dimensions
    pub fn new(color_scheme: UiColorScheme, screen_width: u32, screen_height: u32) -> Self {
        Self {
            color_scheme,
            screen_size: Vec2::new(screen_width as f32, screen_height as f32),
        }
    }

    /// Create main menu screen elements
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
        elements.insert("play_button".to_string(), Box::new(UiButton::new(
            Vec2::new(self.screen_size.x / 2.0 - 100.0, 250.0),
            Vec2::new(200.0, 50.0),
            "New Game",
            &self.color_scheme,
        )));

        // Multiplayer button
        elements.insert("multiplayer_button".to_string(), Box::new(UiButton::new(
            Vec2::new(self.screen_size.x / 2.0 - 100.0, 320.0),
            Vec2::new(200.0, 50.0),
            "Multiplayer",
            &self.color_scheme,
        )));

        // Settings button
        elements.insert("settings_button".to_string(), Box::new(UiButton::new(
            Vec2::new(self.screen_size.x / 2.0 - 100.0, 390.0),
            Vec2::new(200.0, 50.0),
            "Settings",
            &self.color_scheme,
        )));

        // Exit button
        elements.insert("exit_button".to_string(), Box::new(UiButton::new(
            Vec2::new(self.screen_size.x / 2.0 - 100.0, 460.0),
            Vec2::new(200.0, 50.0),
            "Exit Game",
            &self.color_scheme,
        )));

        elements
    }

    /// Create settings menu screen elements
    pub fn create_settings_menu(&self) -> HashMap<String, Box<dyn UiElement>> {
        let mut elements = HashMap::new();

        // Title
        elements.insert("title".to_string(), Box::new(Label::new(
            Vec2::new(self.screen_size.x / 2.0 - 200.0, 50.0),
            Vec2::new(400.0, 50.0),
            "Settings",
            &self.color_scheme,
        ).with_font_size(32.0)));

        // Settings panel
        let mut panel = Panel::new(
            Vec2::new(self.screen_size.x / 2.0 - 250.0, 120.0),
            Vec2::new(500.0, 400.0),
            &self.color_scheme,
        );

        // Music volume slider
        let music_volume_slider = Slider::new(
            Vec2::new(150.0, 50.0),
            Vec2::new(250.0, 30.0),
            &self.color_scheme,
        )
        .with_label("Music Volume")
        .with_value(0.7);

        // Sound effects volume slider
        let sfx_volume_slider = Slider::new(
            Vec2::new(150.0, 100.0),
            Vec2::new(250.0, 30.0),
            &self.color_scheme,
        )
        .with_label("Sound Effects Volume")
        .with_value(0.8);

        // Fullscreen checkbox
        let fullscreen_checkbox = Checkbox::new(
            Vec2::new(150.0, 150.0),
            Vec2::new(250.0, 30.0),
            "Fullscreen",
            &self.color_scheme,
        );

        // V-Sync checkbox
        let vsync_checkbox = Checkbox::new(
            Vec2::new(150.0, 200.0),
            Vec2::new(250.0, 30.0),
            "V-Sync",
            &self.color_scheme,
        );

        // Difficulty dropdown
        let difficulty_dropdown = Dropdown::new(
            Vec2::new(150.0, 250.0),
            Vec2::new(250.0, 30.0),
            vec![
                "Easy".to_string(),
                "Normal".to_string(),
                "Hard".to_string(),
            ],
            &self.color_scheme,
        );

        // Save and Back buttons
        let save_button = UiButton::new(
            Vec2::new(150.0, 350.0),
            Vec2::new(120.0, 40.0),
            "Save",
            &self.color_scheme,
        );

        let back_button = UiButton::new(
            Vec2::new(280.0, 350.0),
            Vec2::new(120.0, 40.0),
            "Back",
            &self.color_scheme,
        );

        // Add elements to panel
        panel.add_element("music_volume", Box::new(music_volume_slider));
        panel.add_element("sfx_volume", Box::new(sfx_volume_slider));
        panel.add_element("fullscreen", Box::new(fullscreen_checkbox));
        panel.add_element("vsync", Box::new(vsync_checkbox));
        panel.add_element("difficulty", Box::new(difficulty_dropdown));
        panel.add_element("save_button", Box::new(save_button));
        panel.add_element("back_button", Box::new(back_button));

        // Add panel to elements
        elements.insert("settings_panel".to_string(), Box::new(panel));

        elements
    }

    // More methods for creating other menu screens would follow...
}