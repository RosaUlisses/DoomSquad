#![allow(dead_code)]

mod audio;
mod game;
mod menu;
mod parser;
mod render;

use crate::audio::audio_handler::Audio;
use crate::menu::menu_handler::{AppState, Menu};
use crate::render::{RendererData, render_init};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::f64::consts::PI;
use std::time::Instant;
use crate::game::key_bind_config::{KeyBindConfig, string_to_key};
use std::path::Path;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 600;
const TARGET_FPS: usize = 30;
const HORIZONTAL_FOV: f64 = PI / 2.0;
const BACKGROUND_COLOR: u32 = 0x444444;
const DISTANCE_DARKNESS_COEFFICIENT: f64 = 0.005;
const WALL_DEFAULT_COLOR: u32 = 0x00ff00;
const BLOCK_DEFAULT_COLOR: u32 = 0x0000ff;
const SURFACE_DEFAULT_COLOR: u32 = 0xffff00;

const KEY_BINDINGS_CONFIG_FILE_PATH: &str = "assets/keybindings.json";

fn save_key_bindigs_config(key_bind_config: KeyBindConfig) -> anyhow::Result<()> {
    let json = key_bind_config.to_json()?;
    std::fs::write(KEY_BINDINGS_CONFIG_FILE_PATH, json)?;
    Ok(())
}

fn read_key_binding_config() -> KeyBindConfig {
    let contents =
        std::fs::read_to_string(KEY_BINDINGS_CONFIG_FILE_PATH).unwrap();

    let json: serde_json::Value =
        serde_json::from_str(&contents).unwrap();

    let get_mapped_key = |field: &str| {
        string_to_key(json[field].as_str().unwrap())
    };

    KeyBindConfig {
        forwards: get_mapped_key("forwards"),
        backwards: get_mapped_key("backwards"),
        left: get_mapped_key("left"),
        right: get_mapped_key("right"),
        sprint: get_mapped_key("sprint"),
        jump: get_mapped_key("jump"),
        crouch: get_mapped_key("crouch"),
        rocket_jump: get_mapped_key("rocket_jump"),
        interact: get_mapped_key("interact"),
        move_crosshair_up: get_mapped_key("move_crosshair_up"),
        move_crosshair_down: get_mapped_key("move_crosshair_down"),
        turn_left: get_mapped_key("turn_left"),
        turn_right: get_mapped_key("turn_right"),
        toggle_fine_aiming: get_mapped_key("toggle_fine_aiming"),
        fire_bullet: get_mapped_key("fire_bullet"),
        fire_arrow: get_mapped_key("fire_arrow"),
        enter_godmode: get_mapped_key("enter_godmode"),
        lock_enemies: get_mapped_key("lock_enemies"),
        exit_to_main_menu: get_mapped_key("exit_to_main_menu"),
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut last_time = Instant::now();
    let mut frame_count = 0;
    let mut fps_value;

    let mut window = match Window::new(
        "DoomSquad",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            eprint!("failed to create Window");
            return Err(Box::new(e));
        }
    };

    window.set_target_fps(TARGET_FPS);
    let mut buffer: Vec<u32> = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];

    let renderer_data: RendererData = render_init(
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        HORIZONTAL_FOV,
        BACKGROUND_COLOR,
        DISTANCE_DARKNESS_COEFFICIENT,
        WALL_DEFAULT_COLOR,
        BLOCK_DEFAULT_COLOR,
        SURFACE_DEFAULT_COLOR,
    );

    let mut key_bindings =
        if Path::new(KEY_BINDINGS_CONFIG_FILE_PATH).is_file() {
            read_key_binding_config()
        } else {
            KeyBindConfig::default()
        };

    let mut game = game::Game::new_game(&renderer_data, key_bindings).unwrap();
    let mut audio = Audio::init();
    let mut menu = Menu::new();

    let mut app_state = AppState::StartScreen;

    while window.is_open() {
        let escape_clicked = window.is_key_pressed(key_bindings.exit_to_main_menu, KeyRepeat::No);

        match app_state {
            AppState::StartScreen => {
                window.set_cursor_visibility(true);

                if escape_clicked {
                    break;
                } else {
                    app_state = menu.update_and_draw_start_menu(
                        &mut window,
                        &mut buffer,
                        &mut game,
                        &renderer_data,
                        &mut audio,
                        &mut key_bindings,
                    );
                }
            }

            AppState::KeyBindingsMenu => {
                window.set_cursor_visibility(true);
                app_state = menu.update_and_draw_key_bindings_menu(
                    &mut window,
                    &mut buffer,
                    &mut game,
                    &mut key_bindings,
                );
            }

            AppState::Playing => {
                window.set_cursor_visibility(false);

                if escape_clicked {
                    app_state = AppState::GamePaused;
                } else if game.player.hp <= 0.0 {
                    app_state = AppState::GameOver;
                } else {
                    game.update(&window, &renderer_data, &mut audio);
                    render::draw_screen(&mut buffer, &renderer_data, &game);

                    frame_count += 1;
                    let elapsed = last_time.elapsed().as_secs_f32();
                    if elapsed >= 1.0 {
                        fps_value = frame_count as f32 / elapsed;
                        frame_count = 0;
                        last_time = Instant::now();
                        window.set_title(&format!("DoomSquad | FPS: {:.1}", fps_value));
                    }
                }
            }

            AppState::GamePaused => {
                window.set_cursor_visibility(true);

                if escape_clicked {
                    app_state = AppState::StartScreen;
                } else {
                    app_state = menu.update_and_draw_game_paused(
                        &mut window,
                        &mut buffer,
                        &mut game,
                        &mut audio,
                    );
                }
            }

            AppState::GameOver => {
                window.set_cursor_visibility(true);

                if escape_clicked {
                    app_state = AppState::StartScreen;
                } else {
                    app_state = menu.update_and_draw_game_over(&mut window, &mut buffer, &mut game);
                }
            }

            AppState::Quit => {
                break;
            }
        }

        save_key_bindigs_config(key_bindings)?;

        if let Err(e) = window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT) {
            eprintln!("failed to update the window: {e}");
            return Err(Box::new(e));
        }
    }
    Ok(())
}
