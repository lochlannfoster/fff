// src/ui/menu_callbacks.rs

use std::rc::Rc;
use std::cell::RefCell;

use crate::game::{GameState, GamePhase};
use crate::ui::UiManager;

pub struct MenuCallbacks {
    game_state: Rc<RefCell<GameState>>,
    ui_manager: Rc<RefCell<UiManager>>,
}

impl MenuCallbacks {
    pub fn new(game_state: &mut GameState, ui_manager: &mut UiManager) -> Self {
        Self {
            game_state: Rc::new(RefCell::new(game_state.clone())),
            ui_manager: Rc::new(RefCell::new(ui_manager.clone())),
        }
    }
    
    pub fn attach_callbacks(&self, ui_manager: &mut UiManager) {
        // Main Menu
        self.attach_main_menu_callbacks(ui_manager);
        
        // Settings Menu
        self.attach_settings_menu_callbacks(ui_manager);
        
        // Game Setup Menu
        self.attach_game_setup_callbacks(ui_manager);
        
        // Pause Menu
        self.attach_pause_menu_callbacks(ui_manager);
        
        // Game Over Menu
        self.attach_game_over_callbacks(ui_manager);
    }
    
    fn attach_main_menu_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        
        // Play button
        if let Some(element) = ui_manager.get_element_mut("main_menu_play_button") {
            let ui_manager_clone = ui_manager_clone.clone();
            element.set_on_click(move || {
                ui_manager_clone.borrow_mut().set_active_screen("game_setup");
                true
            });
        }
        
        // Multiplayer button
        if let Some(element) = ui_manager.get_element_mut("main_menu_multiplayer_button") {
            let ui_manager_clone = ui_manager_clone.clone();
            element.set_on_click(move || {
                ui_manager_clone.borrow_mut().set_active_screen("multiplayer");
                true
            });
        }
        
        // Settings button
        if let Some(element) = ui_manager.get_element_mut("main_menu_settings_button") {
            let ui_manager_clone = ui_manager_clone.clone();
            element.set_on_click(move || {
                ui_manager_clone.borrow_mut().set_active_screen("settings");
                true
            });
        }
        
        // Exit button
        if let Some(element) = ui_manager.get_element_mut("main_menu_exit_button") {
            element.set_on_click(move || {
                std::process::exit(0);
            });
        }
    }
    
    fn attach_settings_menu_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        
        // Back button
        if let Some(element) = ui_manager.get_element_mut("settings_back_button") {
            let ui_manager_clone = ui_manager_clone.clone();
            element.set_on_click(move || {
                ui_manager_clone.borrow_mut().set_active_screen("main_menu");
                true
            });
        }
        
        // Fog of war checkbox
        if let Some(element) = ui_manager.get_element_mut("settings_fog_of_war") {
            let game_state_clone = game_state_clone.clone();
            element.set_on_change(move |checked| {
                let mut settings = &mut game_state_clone.borrow_mut().settings;
                settings.fog_of_war_enabled = checked;
                true
            });
        }
        
        // Game speed slider
        if let Some(element) = ui_manager.get_element_mut("settings_game_speed") {
            let game_state_clone = game_state_clone.clone();
            element.set_on_change(move |value| {
                let mut settings = &mut game_state_clone.borrow_mut().settings;
                settings.game_speed = value;
                true
            });
        }
    }
    
    fn attach_game_setup_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        
        // Start button
        if let Some(element) = ui_manager.get_element_mut("game_setup_start_button") {
            let game_state_clone = game_state_clone.clone();
            let ui_manager_clone = ui_manager_clone.clone();
            element.set_on_click(move || {
                // Start the game
                let mut game_state = game_state_clone.borrow_mut();
                game_state.phase = GamePhase::Playing;
                
                // Generate a random seed
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                    
                game_state.start_game(false, game_state.player_count, seed);
                
                ui_manager_clone.borrow_mut().set_active_screen("game");
                true
            });
        }
        
        // Back button
        if let Some(element) = ui_manager.get_element_mut("game_setup_back_button") {
            let ui_manager_clone = ui_manager_clone.clone();
            element.set_on_click(move || {
                ui_manager_clone.borrow_mut().set_active_screen("main_menu");
                true
            });
        }
    }
    
    fn attach_pause_menu_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        
        // Resume button
        if let Some(element) = ui_manager.get_element_mut("pause_resume_button") {
            let game_state_clone = game_state_clone.clone();
            let ui_manager_clone = ui_manager_clone.clone();
            element.set_on_click(move || {
                let mut game_state = game_state_clone.borrow_mut();
                game_state.resume();
                ui_manager_clone.borrow_mut().set_active_screen("game");
                true
            });
        }
        
        // Quit button
        if let Some(element) = ui_manager.get_element_mut("pause_quit_button") {
            let game_state_clone = game_state_clone.clone();
            let ui_manager_clone = ui_manager_clone.clone();
            element.set_on_click(move || {
                let mut game_state = game_state_clone.borrow_mut();
                game_state.phase = GamePhase::MainMenu;
                ui_manager_clone.borrow_mut().set_active_screen("main_menu");
                true
            });
        }
    }
    
    fn attach_game_over_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        
        // Main menu button
        if let Some(element) = ui_manager.get_element_mut("game_over_main_menu_button") {
            let game_state_clone = game_state_clone.clone();
            let ui_manager_clone = ui_manager_clone.clone();
            element.set_on_click(move || {
                let mut game_state = game_state_clone.borrow_mut();
                game_state.phase = GamePhase::MainMenu;
                ui_manager_clone.borrow_mut().set_active_screen("main_menu");
                true
            });
        }
    }
}