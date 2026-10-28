//! The Menu and User Interface system.
//!
//! This module manages all the non-gameplay screens, such as the Main Menu,
//! the Pause Screen, and the Game Over screen. It handles drawing buttons and text,
//! processing mouse clicks, and taking care of saving and loading the player's progress.

use font8x8::{BASIC_FONTS, UnicodeFonts};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};
use std::fs;

use crate::audio::audio_handler::Audio;
use crate::game::gamestate::Game;
use crate::game::key_bind_config::KeyBindConfig;
use crate::render::RendererData;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 450;

const COL_BG_MAIN: u32 = 0x111111;
const COL_BG_GAMEOVER: u32 = 0x330000;
const COL_BG_PAUSED: u32 = 0x002244;
const COL_TITLE_RED: u32 = 0xCC0000;
const COL_TITLE_BLUE: u32 = 0x44AAFF;
const COL_BTN_NORMAL: u32 = 0x444444;
const COL_BTN_HOVER: u32 = 0xAA0000;
const COL_TEXT_WHITE: u32 = 0xFFFFFF;

/// Dictates the high-level routing in the `main.rs` loop.
pub enum AppState {
    StartScreen,
    Playing,
    GameOver,
    GamePaused,
    KeyBindingsMenu,
    Quit,
}

/// Simple layout container for button definitions.
struct ButtonDef {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    text: String,
}

/// Holds the minimal state required for the IMGUI.
/// Since `minifb` natively handles keyboard debouncing via `KeyRepeat::No`,
/// we only need to manually track the mouse state for edge detection (clicks).
pub struct Menu {
    was_mouse_down: bool,
    waiting_for_key: Option<usize>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            was_mouse_down: false,
            waiting_for_key: None,
        }
    }

    /// Renders the main menu and handles user interactions.
    pub fn update_and_draw_start_menu(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
        renderer_data: &RendererData,
        audio: &mut Audio,
        key_bindings: &mut KeyBindConfig
    ) -> AppState {
        // Clear the buffer from the previous frame
        buffer.fill(COL_BG_MAIN);

        // Fetch user inputs
        let (mx, my, clicked) = self.get_mouse_state(window);
        let click1 = window.is_key_pressed(Key::Key1, KeyRepeat::No);
        let click2 = window.is_key_pressed(Key::Key2, KeyRepeat::No);
        let click3 = window.is_key_pressed(Key::Key3, KeyRepeat::No);
        let click4 = window.is_key_pressed(Key::Key4, KeyRepeat::No);

        // Render titles
        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "DOOMSQUAD",
            50,
            6,
            COL_TITLE_RED,
        );
        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "MAIN MENU",
            105,
            2,
            COL_TEXT_WHITE,
        );

        // Define button layouts
        let btn_start = ButtonDef {
            x: 230,
            y: 160,
            w: 340,
            h: 50,
            text: "[1] NEW GAME".to_string(),
        };
        let btn_load = ButtonDef {
            x: 230,
            y: 230,
            w: 340,
            h: 50,
            text: "[2] LOAD MAP".to_string(),
        };

        let audio_text = if !audio.is_muted {
            "[3] SOUND: ON"
        } else {
            "[3] SOUND: OFF"
        };
        let btn_audio = ButtonDef {
            x: 230,
            y: 300,
            w: 340,
            h: 50,
            text: audio_text.to_string(),
        };

        let btn_key_bindings = ButtonDef {
            x: 230,
            y: 370,
            w: 340,
            h: 30,
            text: "[4] CHANGE KEY BINDINGS".to_string(),
        };

        let btn_quit = ButtonDef {
            x: 230,
            y: 420,
            w: 340,
            h: 30,
            text: "[ESC] QUIT GAME".to_string(),
        };

        // Interaction processing
        if self.draw_flat_button(buffer, &btn_start, mx, my) && clicked || click1 {
            *game = Game::new_game(renderer_data, key_bindings.clone()).unwrap(); // ! this unwrap is intentionally accepted
            game.map_swap(renderer_data, 1); // Jump straight to map 1 (map 0 is reserved for testing)
            return AppState::Playing;
        }

        #[allow(clippy::collapsible_if)]
        if self.draw_flat_button(buffer, &btn_load, mx, my) && clicked || click2 {
            // Read saved map index from disk and safely parse it
            if let Ok(content) = fs::read_to_string("savegame.txt")
                && let Ok(index) = content.trim().parse::<usize>()
            {
                *game = Game::new_game(renderer_data, key_bindings.clone()).unwrap(); // ! this unwrap is intentionally accepted
                game.map_swap(renderer_data, index);
                return AppState::Playing;
            }
        }

        if self.draw_flat_button(buffer, &btn_audio, mx, my) && clicked || click3 {
            audio.set_muted(!audio.is_muted);
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_key_bindings, mx, my) && clicked || click4 {
            return AppState::KeyBindingsMenu;
        }

        if self.draw_flat_button(buffer, &btn_quit, mx, my) && clicked {
            return AppState::Quit;
        }

        
        // Keep looping in the menu until an action changes the state
        AppState::StartScreen
    }

    pub fn update_and_draw_key_bindings_menu(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
        key_bindings: &mut KeyBindConfig,
    ) -> AppState {
        buffer.fill(COL_BG_MAIN);

        let (mx, my, clicked) = self.get_mouse_state(window);

        // Only capture keyboard input if we were already waiting
        // before this frame's mouse click.
        let pending = self.waiting_for_key;

        let pressed_key = if pending.is_some() {
            window
                .get_keys_pressed(KeyRepeat::No)
                .into_iter()
                .next()
        } else {
            None
        };

        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "KEY BINDINGS",
            20,
            3,
            COL_TITLE_RED,
        );

        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "Click a binding, then press its replacement key.",
            58,
            1,
            COL_TEXT_WHITE,
        );

        let back_button = ButtonDef {
            x: 210,
            y: 400,
            w: 180,
            h: 32,
            text: "BACK".to_string(),
        };

        if self.draw_flat_button(buffer, &back_button, mx, my) && clicked {
            self.waiting_for_key = None;
            return AppState::StartScreen;
        }

        if pending.is_some() {
            let cancel_button = ButtonDef {
                x: 410,
                y: 400,
                w: 180,
                h: 32,
                text: "CANCEL".to_string(),
            };

            if self.draw_flat_button(buffer, &cancel_button, mx, my) && clicked {
                self.waiting_for_key = None;
                return AppState::KeyBindingsMenu;
            }
        }

        let rows = [
            ("Move forwards", &mut key_bindings.forwards),
            ("Move backwards", &mut key_bindings.backwards),
            ("Move left", &mut key_bindings.left),
            ("Move right", &mut key_bindings.right),
            ("Sprint", &mut key_bindings.sprint),
            ("Jump", &mut key_bindings.jump),
            ("Crouch", &mut key_bindings.crouch),
            ("Rocket jump", &mut key_bindings.rocket_jump),
            ("Interact", &mut key_bindings.interact),
            ("Crosshair up", &mut key_bindings.move_crosshair_up),
            ("Crosshair down", &mut key_bindings.move_crosshair_down),
            ("Turn left", &mut key_bindings.turn_left),
            ("Turn right", &mut key_bindings.turn_right),
            ("Toggle fine aiming", &mut key_bindings.toggle_fine_aiming),
            ("Fire bullet", &mut key_bindings.fire_bullet),
            ("Fire arrow", &mut key_bindings.fire_arrow),
            ("God mode", &mut key_bindings.enter_godmode),
            ("Lock enemies", &mut key_bindings.lock_enemies),
            ("Pause / menu", &mut key_bindings.exit_to_main_menu),
        ];

        for (index, (label, binding)) in rows.into_iter().enumerate() {
            if pending == Some(index) {
                if let Some(key) = pressed_key {
                    *binding = key;
                    self.waiting_for_key = None;
                }
            }

            let column = index / 10;
            let row = index % 10;

            let x = 20 + column * 390;
            let y = 86 + row * 28;

            let selected = self.waiting_for_key == Some(index);

            Self::draw_text(
                buffer,
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                label,
                x,
                y + 8,
                1,
                if selected {
                    COL_TITLE_BLUE
                } else {
                    COL_TEXT_WHITE
                },
            );

            let button = ButtonDef {
                x: x + 180,
                y,
                w: 180,
                h: 24,
                text: if selected {
                    "PRESS KEY".to_string()
                } else {
                    format!("{:?}", *binding)
                },
            };

            if self.draw_flat_button(buffer, &button, mx, my) && clicked {
                self.waiting_for_key = Some(index);
            }
        }

        game.key_bindings = *key_bindings;

        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "Changes apply immediately after clicking on the back button",
            375,
            1,
            COL_TEXT_WHITE,
        );

        AppState::KeyBindingsMenu
    }

    /// Renders the death screen and allows the user to save their current level progress.
    pub fn update_and_draw_game_over(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
    ) -> AppState {
        buffer.fill(COL_BG_GAMEOVER);

        let (mx, my, clicked) = self.get_mouse_state(window);
        let click1 = window.is_key_pressed(Key::Key1, KeyRepeat::No);

        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "YOU DIED",
            80,
            6,
            COL_TITLE_RED,
        );

        let btn_save = ButtonDef {
            x: 230,
            y: 220,
            w: 340,
            h: 60,
            text: "[1] SAVE MAP INDEX".to_string(),
        };
        let btn_menu = ButtonDef {
            x: 230,
            y: 300,
            w: 340,
            h: 60,
            text: "[ESC] MAIN MENU".to_string(),
        };

        if self.draw_flat_button(buffer, &btn_save, mx, my) && clicked || click1 {
            // Persist the current level index to disk so it can be loaded later
            let _ = fs::write("savegame.txt", game.map_index.to_string());
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_menu, mx, my) && clicked {
            return AppState::StartScreen;
        }

        AppState::GameOver
    }

    /// Renders the pause menu (triggered by pressing ESC during gameplay).
    pub fn update_and_draw_game_paused(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
        audio: &mut Audio,
    ) -> AppState {
        buffer.fill(COL_BG_PAUSED);

        let (mx, my, clicked) = self.get_mouse_state(window);

        let click1 = window.is_key_pressed(Key::Key1, KeyRepeat::No);
        let click2 = window.is_key_pressed(Key::Key2, KeyRepeat::No);
        let click3 = window.is_key_pressed(Key::Key3, KeyRepeat::No);

        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "GAME PAUSED",
            60,
            6,
            COL_TITLE_BLUE,
        );

        let audio_text = if !audio.is_muted {
            "[3] SOUND: ON"
        } else {
            "[3] SOUND: OFF"
        };

        let btn_resume = ButtonDef {
            x: 230,
            y: 150,
            w: 340,
            h: 50,
            text: "[1] RESUME GAME".to_string(),
        };
        let btn_save = ButtonDef {
            x: 230,
            y: 210,
            w: 340,
            h: 50,
            text: "[2] SAVE & EXIT".to_string(),
        };
        let btn_audio = ButtonDef {
            x: 230,
            y: 270,
            w: 340,
            h: 50,
            text: audio_text.to_string(),
        };
        let btn_menu = ButtonDef {
            x: 230,
            y: 330,
            w: 340,
            h: 50,
            text: "[ESC] MAIN MENU".to_string(),
        };

        if self.draw_flat_button(buffer, &btn_resume, mx, my) && clicked || click1 {
            // Simply return to the Playing state without touching the game object to resume seamlessly
            return AppState::Playing;
        }

        if self.draw_flat_button(buffer, &btn_save, mx, my) && clicked || click2 {
            let _ = std::fs::write("savegame.txt", game.map_index.to_string());
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_audio, mx, my) && clicked || click3 {
            audio.set_muted(!audio.is_muted);
            return AppState::GamePaused;
        }

        if self.draw_flat_button(buffer, &btn_menu, mx, my) && clicked {
            return AppState::StartScreen;
        }

        AppState::GamePaused
    }

    /// Fetches mouse coordinates and performs edge detection (debouncing) for left-clicks.
    fn get_mouse_state(&mut self, window: &Window) -> (usize, usize, bool) {
        // Clamp prevents out-of-bounds coordinates if the mouse leaves the window frame
        let mouse_pos = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
        let is_mouse_down = window.get_mouse_down(MouseButton::Left);

        let clicked = is_mouse_down && !self.was_mouse_down;
        self.was_mouse_down = is_mouse_down;

        (mouse_pos.0 as usize, mouse_pos.1 as usize, clicked)
    }

    /// Draws a rectangular button and its text onto the buffer.
    /// Returns `true` if the mouse cursor is currently hovering over it.
    fn draw_flat_button(&self, buffer: &mut [u32], btn: &ButtonDef, mx: usize, my: usize) -> bool {
        // Simple check for hover detection
        let hover = mx >= btn.x && mx <= (btn.x + btn.w) && my >= btn.y && my <= (btn.y + btn.h);
        let bg_color = if hover { COL_BTN_HOVER } else { COL_BTN_NORMAL };

        // Rasterize the rectangle into the 1D pixel buffer
        for row in btn.y..(btn.y + btn.h) {
            for col in btn.x..(btn.x + btn.w) {
                // Safety check to prevent panic from out-of-bounds memory writes
                if row < SCREEN_HEIGHT && col < SCREEN_WIDTH {
                    buffer[row * SCREEN_WIDTH + col] = bg_color;
                }
            }
        }

        // Center the text horizontally and vertically inside the button bounds
        let text_width = btn.text.len() * 8 * 2;
        let text_x = btn.x + btn.w.saturating_sub(text_width) / 2;
        let text_y = btn.y + btn.h.saturating_sub(16) / 2;

        Self::draw_text(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &btn.text,
            text_x,
            text_y,
            2,
            COL_TEXT_WHITE,
        );

        hover
    }

    /// Helper to align a given text string exactly in the horizontal center of the screen.
    fn draw_text_centered(
        buffer: &mut [u32],
        screen_width: usize,
        screen_height: usize,
        text: &str,
        y: usize,
        scale: usize,
        color: u32,
    ) {
        // Each character in the font is 8 pixels wide, multiplied by the scaling factor
        let text_width = text.len() * 8 * scale;
        let x = screen_width.saturating_sub(text_width) / 2;

        Self::draw_text(
            buffer,
            screen_width,
            screen_height,
            text,
            x,
            y,
            scale,
            color,
        );
    }

    /// Software text rasterizer using the `font8x8` library.
    /// It translates strings into raw pixel data and pushes them to the 1D buffer.
    #[allow(clippy::too_many_arguments)]
    fn draw_text(
        buffer: &mut [u32],
        screen_width: usize,
        screen_height: usize,
        text: &str,
        start_x: usize,
        start_y: usize,
        scale: usize,
        color: u32,
    ) {
        let mut cursor_x = start_x;

        for c in text.chars() {
            if let Some(glyph) = BASIC_FONTS.get(c) {
                #[allow(clippy::needless_range_loop)]
                for y in 0..8 {
                    for x in 0..8 {
                        // Bitwise check: if the specific bit in the glyph byte is a 1, a pixel needs to be drawn
                        if (glyph[y] & (1 << x)) != 0 {
                            // Nearest-neighbor scaling (draws larger 2D blocks for each active pixel based on the `scale` factor)
                            for dy in 0..scale {
                                for dx in 0..scale {
                                    let px = cursor_x + x * scale + dx;
                                    let py = start_y + y * scale + dy;

                                    if px < screen_width && py < screen_height {
                                        // Flatten 2D coordinate into a 1D array index
                                        buffer[py * screen_width + px] = color;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Advance the cursor by the width of the character to prepare for the next letter
            cursor_x += 8 * scale;
        }
    }
}
