mod factory;
mod callbacks;

pub use factory::MenuFactory;
pub use callbacks::MenuCallbacks;

use std::collections::HashMap;

use crate::ui::{
    UiElement, 
    UiColorScheme
};

/// Creates pre-defined menu layouts and manages menu interactions
pub struct MenuManager {
    color_scheme: UiColorScheme,
    current_screen: String,
    elements: HashMap<String, Box<dyn UiElement>>,
}

impl MenuManager {
    /// Create a new menu manager
    pub fn new(color_scheme: UiColorScheme, screen_width: u32, screen_height: u32) -> Self {
        let factory = MenuFactory::new(color_scheme.clone(), screen_width, screen_height);
        
        let mut manager = Self {
            color_scheme,
            current_screen: "main_menu".to_string(),
            elements: HashMap::new(),
        };

        // Populate initial menu screens
        manager.populate_menus(&factory);
        
        manager
    }

    /// Populate menu screens using the menu factory
    fn populate_menus(&mut self, factory: &MenuFactory) {
        // Create and add main menu elements
        let main_menu_elements = factory.create_main_menu();
        for (id, element) in main_menu_elements {
            self.elements.insert(format!("main_menu_{}", id), element);
        }

        // Create and add other menu screens similarly
        let settings_elements = factory.create_settings_menu();
        for (id, element) in settings_elements {
            self.elements.insert(format!("settings_{}", id), element);
        }

        // Add other menu screens...
    }

    /// Switch to a specific menu screen
    pub fn set_screen(&mut self, screen_name: &str) {
        // Hide all elements
        for element in self.elements.values_mut() {
            element.set_visible(false);
        }

        // Show elements for the specified screen
        for (id, element) in &mut self.elements {
            if id.starts_with(&format!("{}_", screen_name)) {
                element.set_visible(true);
            }
        }

        self.current_screen = screen_name.to_string();
    }

    /// Get the current active screen
    pub fn current_screen(&self) -> &str {
        &self.current_screen
    }
}