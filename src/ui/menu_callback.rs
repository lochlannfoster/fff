use std::rc::Rc;
use std::cell::RefCell;

use crate::game::{GameState, GamePhase};
use crate::ui::UiManager;

/// Menu callback handler for the UI system
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
    
    /// Attach callbacks to all menu elements
    pub fn attach_callbacks(&self, ui_manager: &mut UiManager) {
        // Main menu callbacks
        self.setup_main_menu_callbacks(ui_manager);
        
        // Settings menu callbacks
        self.setup_settings_menu_callbacks(ui_manager);
        
        // Game setup menu callbacks
        self.setup_game_setup_callbacks(ui_manager);
        
        // Pause menu callbacks
        self.setup_pause_menu_callbacks(ui_manager);
        
        // Game over menu callbacks
        self.setup_game_over_callbacks(ui_manager);
    }
    
    /// Set up main menu button callbacks
    fn setup_main_menu_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        
        // Play button
        if let Some(element) = ui_manager.get_element_mut("main_menu_play_button") {
            element.set_on_click(move || {
                // Show game setup screen
                ui_manager_clone.borrow_mut().set_active_screen("setup");
                true
            });
        }
        
        // Multiplayer button
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        if let Some(element) = ui_manager.get_element_mut("main_menu_multiplayer_button") {
            element.set_on_click(move || {
                // Show multiplayer setup screen
                ui_manager_clone.borrow_mut().set_active_screen("multiplayer");
                true
            });
        }
        
        // Settings button
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        if let Some(element) = ui_manager.get_element_mut("main_menu_settings_button") {
            element.set_on_click(move || {
                // Show settings screen
                ui_manager_clone.borrow_mut().set_active_screen("settings");
                true
            });
        }
        
        // Exit button
        if let Some(element) = ui_manager.get_element_mut("main_menu_exit_button") {
            element.set_on_click(move || {
                // Exit game
                std::process::exit(0);
            });
        }
    }
    
    /// Set up settings menu callbacks
    fn setup_settings_menu_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        
        // Music volume slider
        if let Some(element) = ui_manager.get_element_mut("settings_music_volume") {
            element.set_on_change(move |value| {
                // Set music volume
                // In a real implementation, this would update the audio system
                true
            });
        }
        
        // Sound volume slider
        let game_state_clone = self.game_state.clone();
        if let Some(element) = ui_manager.get_element_mut("settings_sfx_volume") {
            element.set_on_change(move |value| {
                // Set sound volume
                // In a real implementation, this would update the audio system
                true
            });
        }
        
        // Fullscreen checkbox
        if let Some(element) = ui_manager.get_element_mut("settings_fullscreen") {
            element.set_on_change(move |checked| {
                // Toggle fullscreen
                // In a real implementation, this would update the window
                true
            });
        }
        
        // Show FPS checkbox
        let game_state_clone = self.game_state.clone();
        if let Some(element) = ui_manager.get_element_mut("settings_show_fps") {
            element.set_on_change(move |checked| {
                // Toggle FPS display
                game_state_clone.borrow_mut().settings.show_fps = checked;
                true
            });
        }
        
        // Fog of War checkbox
        let game_state_clone = self.game_state.clone();
        if let Some(element) = ui_manager.get_element_mut("settings_fog_of_war") {
            element.set_on_change(move |checked| {
                // Toggle fog of war
                game_state_clone.borrow_mut().settings.fog_of_war_enabled = checked;
                true
            });
        }
        
        // Game speed slider
        let game_state_clone = self.game_state.clone();
        if let Some(element) = ui_manager.get_element_mut("settings_game_speed") {
            element.set_on_change(move |value| {
                // Set game speed
                game_state_clone.borrow_mut().settings.game_speed = value;
                true
            });
        }
        
        // Back button
        let ui_manager_clone = self.ui_manager.clone();
        if let Some(element) = ui_manager.get_element_mut("settings_back_button") {
            element.set_on_click(move || {
                // Go back to previous screen
                ui_manager_clone.borrow_mut().set_active_screen("main_menu");
                true
            });
        }
    }
    
    /// Set up game setup menu callbacks
    fn setup_game_setup_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        
        // Map dropdown
        if let Some(element) = ui_manager.get_element_mut("setup_map_dropdown") {
            element.set_on_select(move |index, name| {
                // Set map
                // In a real implementation, this would select the map
                true
            });
        }
        
        // Players dropdown
        let game_state_clone = self.game_state.clone();
        if let Some(element) = ui_manager.get_element_mut("setup_players_dropdown") {
            element.set_on_select(move |index, name| {
                // Set player count
                game_state_clone.borrow_mut().player_count = (index as u8) + 2; // 2-4 players
                true
            });
        }
        
        // AI dropdown
        if let Some(element) = ui_manager.get_element_mut("setup_ai_dropdown") {
            element.set_on_select(move |index, name| {
                // Set AI count
                // In a real implementation, this would set AI players
                true
            });
        }
        
        // Difficulty dropdown
        if let Some(element) = ui_manager.get_element_mut("setup_difficulty_dropdown") {
            element.set_on_select(move |index, name| {
                // Set AI difficulty
                // In a real implementation, this would set AI difficulty
                true
            });
        }
        
        // Start button
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        if let Some(element) = ui_manager.get_element_mut("setup_start_button") {
            element.set_on_click(move || {
                // Start the game
                game_state_clone.borrow_mut().phase = GamePhase::Playing;
                ui_manager_clone.borrow_mut().set_active_screen("game");
                
                // In a real implementation, this would also initialize the game map, etc.
                let mut gs = game_state_clone.borrow_mut();
                gs.start_game(false, gs.player_count, rand::random());
                
                true
            });
        }
        
        // Back button
        let ui_manager_clone = self.ui_manager.clone();
        if let Some(element) = ui_manager.get_element_mut("setup_back_button") {
            element.set_on_click(move || {
                // Go back to main menu
                ui_manager_clone.borrow_mut().set_active_screen("main_menu");
                true
            });
        }
    }
    
    /// Set up pause menu callbacks
    fn setup_pause_menu_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        
        // Resume button
        if let Some(element) = ui_manager.get_element_mut("pause_resume_button") {
            element.set_on_click(move || {
                // Resume game
                game_state_clone.borrow_mut().phase = GamePhase::Playing;
                ui_manager_clone.borrow_mut().set_active_screen("game");
                true
            });
        }
        
        // Settings button
        let ui_manager_clone = self.ui_manager.clone();
        if let Some(element) = ui_manager.get_element_mut("pause_settings_button") {
            element.set_on_click(move || {
                // Show settings screen
                ui_manager_clone.borrow_mut().set_active_screen("settings");
                true
            });
        }
        
        // Save button
        if let Some(element) = ui_manager.get_element_mut("pause_save_button") {
            element.set_on_click(move || {
                // Save game
                // In a real implementation, this would save the game state
                true
            });
        }
        
        // Load button
        if let Some(element) = ui_manager.get_element_mut("pause_load_button") {
            element.set_on_click(move || {
                // Load game
                // In a real implementation, this would load a saved game
                true
            });
        }
        
        // Quit button
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        if let Some(element) = ui_manager.get_element_mut("pause_quit_button") {
            element.set_on_click(move || {
                // Quit to main menu
                game_state_clone.borrow_mut().phase = GamePhase::MainMenu;
                ui_manager_clone.borrow_mut().set_active_screen("main_menu");
                true
            });
        }
    }
    
    /// Set up game over menu callbacks
    fn setup_game_over_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        
        // Play again button
        if let Some(element) = ui_manager.get_element_mut("game_over_play_again_button") {
            element.set_on_click(move || {
                // Start a new game with same settings
                ui_manager_clone.borrow_mut().set_active_screen("setup");
                true
            });
        }
        
        // View replay button
        if let Some(element) = ui_manager.get_element_mut("game_over_replay_button") {
            element.set_on_click(move || {
                // Show replay
                // In a real implementation, this would load the replay
                true
            });
        }
        
        // Main menu button
        let game_state_clone = self.game_state.clone();
        let ui_manager_clone = self.ui_manager.clone();
        if let Some(element) = ui_manager.get_element_mut("game_over_main_menu_button") {
            element.set_on_click(move || {
                // Return to main menu
                game_state_clone.borrow_mut().phase = GamePhase::MainMenu;
                ui_manager_clone.borrow_mut().set_active_screen("main_menu");
                true
            });
        }
    }
}