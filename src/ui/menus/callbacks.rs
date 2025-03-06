use std::rc::Rc;
use std::cell::RefCell;

use crate::game::{GameState, GamePhase};
use crate::ui::UiManager;
use crate::ui::menus::MenuFactory;

/// Manages callbacks for menu interactions
pub struct MenuCallbacks {
    game_state: Rc<RefCell<GameState>>,
    ui_manager: Rc<RefCell<UiManager>>,
}

impl MenuCallbacks {
    /// Create a new MenuCallbacks instance
    pub fn new(game_state: &mut GameState, ui_manager: &mut UiManager) -> Self {
        Self {
            game_state: Rc::new(RefCell::new(game_state.clone())),
            ui_manager: Rc::new(RefCell::new(ui_manager.clone())),
        }
    }

    /// Attach callbacks to UI elements
    pub fn attach_callbacks(&self, ui_manager: &mut UiManager) {
        // Main Menu Callbacks
        self.attach_main_menu_callbacks(ui_manager);
        
        // Multiplayer Menu Callbacks
        self.attach_multiplayer_menu_callbacks(ui_manager);
        
        // Settings Menu Callbacks
        self.attach_settings_menu_callbacks(ui_manager);
        
        // Game Setup Callbacks
        self.attach_game_setup_callbacks(ui_manager);
        
        // Pause Menu Callbacks
        self.attach_pause_menu_callbacks(ui_manager);
        
        // Game Over Menu Callbacks
        self.attach_game_over_menu_callbacks(ui_manager);
    }

    /// Attach main menu button callbacks
    fn attach_main_menu_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = Rc::clone(&self.game_state);
        let ui_manager_clone = Rc::clone(&self.ui_manager);

        // Play button
        if let Some(play_button) = ui_manager.get_element_mut("main_menu_play_button") {
            play_button.set_on_click(move || {
                ui_manager_clone.borrow_mut().set_active_screen("game_setup");
                true
            });
        }

        // Multiplayer button
        if let Some(multiplayer_button) = ui_manager.get_element_mut("main_menu_multiplayer_button") {
            multiplayer_button.set_on_click(move || {
                ui_manager_clone.borrow_mut().set_active_screen("multiplayer");
                true
            });
        }

        // Settings button
        if let Some(settings_button) = ui_manager.get_element_mut("main_menu_settings_button") {
            settings_button.set_on_click(move || {
                ui_manager_clone.borrow_mut().set_active_screen("settings");
                true
            });
        }

        // Exit button
        if let Some(exit_button) = ui_manager.get_element_mut("main_menu_exit_button") {
            exit_button.set_on_click(|| {
                std::process::exit(0);
            });
        }
    }
    /// Attach multiplayer menu button callbacks
    fn attach_multiplayer_menu_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = Rc::clone(&self.game_state);
        let ui_manager_clone = Rc::clone(&self.ui_manager);

        // Host game button
        if let Some(host_button) = ui_manager.get_element_mut("multiplayer_host_button") {
            host_button.set_on_click(move || {
                // Implement host game logic
                ui_manager_clone.borrow_mut().set_active_screen("game_setup");
                true
            });
        }

        // Join game button
        if let Some(join_button) = ui_manager.get_element_mut("multiplayer_join_button") {
            join_button.set_on_click(move || {
                // Implement join game logic
                ui_manager_clone.borrow_mut().set_active_screen("game_setup");
                true
            });
        }

        // Back button
        if let Some(back_button) = ui_manager.get_element_mut("multiplayer_back_button") {
            back_button.set_on_click(move || {
                ui_manager_clone.borrow_mut().set_active_screen("main_menu");
                true
            });
        }
    }

    /// Attach settings menu button callbacks
    fn attach_settings_menu_callbacks(&self, ui_manager: &mut UiManager) {
        let game_state_clone = Rc::clone(&self.game_state);
        let ui_manager_clone = Rc::clone(&self.ui_manager);

        // Music volume slider
        if let Some(music_volume) = ui_manager.get_element_mut("settings_music_volume") {
            music_volume.set_on_change(move |volume| {
                // Update audio settings
                println!("Music volume: {}", volume);
                true
            });
        }

        // Sound effects volume slider
        if let Some(sfx_volume) = ui_manager.get_element_mut("settings_sfx_volume") {
            sfx_volume.set_on_change(move |volume| {
                // Update audio settings
                println!("SFX volume: {}", volume);
                true
            });
        }

        // Fullscreen checkbox
        if let Some(fullscreen) = ui_manager.get_element_mut("settings_fullscreen") {
            fullscreen.set_on_change(move |checked| {
                // Toggle fullscreen
                println!("Fullscreen: {}", checked);
                true
            });
        }

        // Save button
        if let Some(save_button) = ui_manager.get_element_mut("settings_save_button") {
            save_button.set_on_click(move || {
                // Save settings
                println!("Settings saved");
                true
            });
        }

        // Back button
        if let Some(back_button) = ui_manager.get_element_mut("settings_back_button") {
            let ui_manager_clone = Rc::clone(&self.ui_manager);
            back_button.set_on_click(move || {
                ui_manager_clone.borrow_mut().set_active_screen("main_menu");
                true
            });
        }
    }
/// Attach game setup menu button callbacks
fn attach_game_setup_callbacks(&self, ui_manager: &mut UiManager) {
    let game_state_clone = Rc::clone(&self.game_state);
    let ui_manager_clone = Rc::clone(&self.ui_manager);

    // Start game button
    if let Some(start_button) = ui_manager.get_element_mut("game_setup_start_button") {
        start_button.set_on_click(move || {
            // Get selected options from dropdowns
            let mut game_state = game_state_clone.borrow_mut();
            
            // Set game phase to playing
            game_state.phase = GamePhase::Playing;

            // Start the game
            game_state.start_game(
                false, // Single player by default
                2,     // Default 2 players
                12345  // Default seed
            );

            ui_manager_clone.borrow_mut().set_active_screen("game");
            true
        });
    }

    // Back button
    if let Some(back_button) = ui_manager.get_element_mut("game_setup_back_button") {
        back_button.set_on_click(move || {
            ui_manager_clone.borrow_mut().set_active_screen("main_menu");
            true
        });
    }
}

/// Attach pause menu button callbacks
fn attach_pause_menu_callbacks(&self, ui_manager: &mut UiManager) {
    let game_state_clone = Rc::clone(&self.game_state);
    let ui_manager_clone = Rc::clone(&self.ui_manager);

    // Resume button
    if let Some(resume_button) = ui_manager.get_element_mut("pause_resume_button") {
        resume_button.set_on_click(move || {
            let mut game_state = game_state_clone.borrow_mut();
            game_state.resume();
            ui_manager_clone.borrow_mut().set_active_screen("game");
            true
        });
    }

    // Settings button
    if let Some(settings_button) = ui_manager.get_element_mut("pause_settings_button") {
        settings_button.set_on_click(move || {
            ui_manager_clone.borrow_mut().set_active_screen("settings");
            true
        });
    }

    // Quit to main menu button
    if let Some(quit_button) = ui_manager.get_element_mut("pause_quit_button") {
        let ui_manager_clone = Rc::clone(&self.ui_manager);
        let game_state_clone = Rc::clone(&self.game_state);
        
        quit_button.set_on_click(move || {
            let mut game_state = game_state_clone.borrow_mut();
            game_state.phase = GamePhase::MainMenu;
            ui_manager_clone.borrow_mut().set_active_screen("main_menu");
            true
        });
    }
}

/// Attach game over menu button callbacks
fn attach_game_over_menu_callbacks(&self, ui_manager: &mut UiManager) {
    let game_state_clone = Rc::clone(&self.game_state);
    let ui_manager_clone = Rc::clone(&self.ui_manager);

    // Replay button
    if let Some(replay_button) = ui_manager.get_element_mut("game_over_replay_button") {
        replay_button.set_on_click(move || {
            // Open replay viewer
            println!("View replay clicked");
            true
        });
    }

    // Rematch button
    if let Some(rematch_button) = ui_manager.get_element_mut("game_over_rematch_button") {
        let ui_manager_clone = Rc::clone(&self.ui_manager);
        let game_state_clone = Rc::clone(&self.game_state);

        rematch_button.set_on_click(move || {
            let mut game_state = game_state_clone.borrow_mut();
            
            // Reset game state and start a new game with similar parameters
            game_state.phase = GamePhase::Playing;
            game_state.start_game(
                game_state.is_multiplayer, 
                game_state.player_count, 
                // Generate a new seed for variety
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            );

            ui_manager_clone.borrow_mut().set_active_screen("game");
            true
        });
    }

    // Main menu button
    if let Some(main_menu_button) = ui_manager.get_element_mut("game_over_main_menu_button") {
        let ui_manager_clone = Rc::clone(&self.ui_manager);
        let game_state_clone = Rc::clone(&self.game_state);

        main_menu_button.set_on_click(move || {
            let mut game_state = game_state_clone.borrow_mut();
            game_state.phase = GamePhase::MainMenu;
            ui_manager_clone.borrow_mut().set_active_screen("main_menu");
            true
        });
    }
}

// Utility methods

/// Validate game setup options
fn validate_game_setup(&self, ui_manager: &UiManager) -> bool {
    // Example validation logic
    let map_dropdown = ui_manager.get_element("game_setup_map_dropdown")
        .and_then(|e| e.as_any().downcast_ref::<Dropdown>());
    
    let player_dropdown = ui_manager.get_element("game_setup_player_dropdown")
        .and_then(|e| e.as_any().downcast_ref::<Dropdown>());
    
    let ai_dropdown = ui_manager.get_element("game_setup_ai_dropdown")
        .and_then(|e| e.as_any().downcast_ref::<Dropdown>());
    
    // Ensure all dropdowns have valid selections
    match (map_dropdown, player_dropdown, ai_dropdown) {
        (Some(map), Some(players), Some(ai)) => {
            map.get_selected_index() < 3 &&  // Validate map selection
            players.get_selected_index() < 3 &&  // Validate player count
            ai.get_selected_index() < 4  // Validate AI count
        },
        _ => false
    }
}

/// Apply settings from settings menu
fn apply_settings(&self, ui_manager: &UiManager) {
    // Music volume slider
    let music_volume = ui_manager.get_element("settings_music_volume")
        .and_then(|e| e.as_any().downcast_ref::<Slider>());
    
    // SFX volume slider  
    let sfx_volume = ui_manager.get_element("settings_sfx_volume")
        .and_then(|e| e.as_any().downcast_ref::<Slider>());
    
    // Fullscreen checkbox
    let fullscreen = ui_manager.get_element("settings_fullscreen")
        .and_then(|e| e.as_any().downcast_ref::<Checkbox>());
    
    // Apply audio settings
    if let Some(music_vol) = music_volume {
        // TODO: Integrate with audio system
        println!("Setting music volume to {}", music_vol.get_value());
    }
    
    if let Some(sfx_vol) = sfx_volume {
        // TODO: Integrate with audio system
        println!("Setting SFX volume to {}", sfx_vol.get_value());
    }
    
    // Apply display settings
    if let Some(fs) = fullscreen {
        // TODO: Integrate with window/display system
        println!("Fullscreen: {}", fs.is_checked());
    }
/// Defines common interactions for menu elements
pub trait MenuInteraction {
    /// Handle navigation between menu screens
    fn navigate(&mut self, from: MenuScreen, to: MenuScreen) -> bool;
    
    /// Handle dropdown selection
    fn select_dropdown(&mut self, dropdown: MenuScreen, index: usize) -> bool;
    
    /// Handle checkbox toggle
    fn toggle_checkbox(&mut self, checkbox: MenuScreen, checked: bool) -> bool;
    
    /// Handle slider adjustment
    fn adjust_slider(&mut self, slider: MenuScreen, value: f32) -> bool;
    
    /// Handle button click
    fn click_button(&mut self, button: MenuScreen) -> bool;
}

/// Represents different menu screens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuScreen {
    MainMenu,
    Multiplayer,
    Settings,
    GameSetup,
    Pause,
    GameOver,
}

/// Default implementation for menu interactions
pub struct DefaultMenuInteractions {
    current_screen: MenuScreen,
}

impl DefaultMenuInteractions {
    /// Create a new menu interaction handler
    pub fn new() -> Self {
        Self {
            current_screen: MenuScreen::MainMenu,
        }
    }
}

impl MenuInteraction for DefaultMenuInteractions {
    fn navigate(&mut self, from: MenuScreen, to: MenuScreen) -> bool {
        println!("Navigating from {:?} to {:?}", from, to);
        self.current_screen = to;
        true
    }
    
    fn select_dropdown(&mut self, dropdown: MenuScreen, index: usize) -> bool {
        println!("Dropdown {:?} selected index {}", dropdown, index);
        true
    }
    
    fn toggle_checkbox(&mut self, checkbox: MenuScreen, checked: bool) -> bool {
        println!("Checkbox {:?} toggled to {}", checkbox, checked);
        true
    }
    
    fn adjust_slider(&mut self, slider: MenuScreen, value: f32) -> bool {
        println!("Slider {:?} adjusted to {}", slider, value);
        true
    }
    
    fn click_button(&mut self, button: MenuScreen) -> bool {
        println!("Button {:?} clicked", button);
        match button {
            MenuScreen::MainMenu => {
                // Specific logic for main menu button
                true
            }
            MenuScreen::Multiplayer => {
                // Specific logic for multiplayer menu button
                true
            }
            _ => true
        }
    }
}

/// Error type for menu interactions
#[derive(Debug)]
pub enum MenuInteractionError {
    InvalidScreen,
    InvalidSelection,
    NavigationFailed,
}

/// Extension trait for additional menu interaction utilities
pub trait MenuInteractionExt {
    /// Safely navigate between screens
    fn safe_navigate(&mut self, from: MenuScreen, to: MenuScreen) 
        -> Result<(), MenuInteractionError>;
    
    /// Get current active screen
    fn current_screen(&self) -> MenuScreen;
}

impl MenuInteractionExt for DefaultMenuInteractions {
    fn safe_navigate(&mut self, from: MenuScreen, to: MenuScreen) 
        -> Result<(), MenuInteractionError> {
        // Example validation logic
        match (from, to) {
            (MenuScreen::MainMenu, MenuScreen::Multiplayer) => {
                self.current_screen = to;
                Ok(())
            }
            (MenuScreen::MainMenu, MenuScreen::Settings) => {
                self.current_screen = to;
                Ok(())
            }
            (MenuScreen::Multiplayer, MenuScreen::GameSetup) => {
                self.current_screen = to;
                Ok(())
            }
            _ => Err(MenuInteractionError::NavigationFailed)
        }
    }
    
    fn current_screen(&self) -> MenuScreen {
        self.current_screen
    }
}
}
}