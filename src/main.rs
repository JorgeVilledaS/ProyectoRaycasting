

mod audio;
mod font;
mod map;
mod minimap;
mod player;
mod raycaster;
mod renderer;
mod screens;
mod sprite;
mod textures;

use map::Map;
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use player::Player;
use raycaster::cast_ray;
use sprite::{Beacon, Mushroom};
use std::time::Instant;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

/// Velocidad de movimiento del jugador, en unidades de mundo por segundo.
const MOVE_SPEED: f32 = 3.0;
/// Velocidad de rotacion por teclado, en radianes por segundo.
const TURN_SPEED: f32 = 2.5;
/// Sensibilidad del mouse (radianes girados por pixel de movimiento horizontal).
const MOUSE_SENSITIVITY: f32 = 0.0025;
/// Distancia a la que se considera que el jugador "llego" al punto final del nivel.
/// Se agrando para que sea mas facil de activar ahora que hay una baliza visible.
const GOAL_RADIUS: f32 = 1.0;

/// Estados posibles de la maquina de estados del juego.
#[derive(PartialEq)]
enum GameState {
    Welcome,
    Playing,
    LevelComplete,
}

fn main() {
    let mut window = Window::new(
        "Ray Caster Psicodelico - Graficas por Computadora",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("No se pudo crear la ventana");

    window.set_target_fps(60);

    let map = Map::brain_level();
    let mut player = Player::new(&map);
    let goal_point = map.goal_point();
    let (target_x, target_y) = map.target_point();
    let mut mushroom = Mushroom::new(target_x, target_y);
    let beacon = Beacon::new(goal_point.0, goal_point.1);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut zbuffer: Vec<f32> = vec![1e30; WIDTH];

    let audio_system = audio::AudioSystem::start();

    let mut state = GameState::Welcome;
    let start_time = Instant::now();
    let mut last_frame = Instant::now();

    let mut last_mouse_x: Option<f32> = None;
    let mut mouse_was_down = false;
    let mut fps_smoothed: f32 = 60.0;

    // Al dispararle al hongo se activa/desactiva el "mundo invertido":
    // izquierda<->derecha, adelante<->atras, y todos los colores en negativo.
    let mut inverted = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(0.05);
        last_frame = now;
        let time = start_time.elapsed().as_secs_f32();

        if dt > 0.0 {
            let instant_fps = 1.0 / dt;
            fps_smoothed = fps_smoothed * 0.9 + instant_fps * 0.1;
        }

        match state {
            GameState::Welcome => {
                screens::draw_welcome_background(&mut buffer, WIDTH, HEIGHT, time);
                let blink_on = (time * 2.0).sin() > 0.0;
                screens::draw_welcome_panel(&mut buffer, WIDTH, HEIGHT, blink_on);

                if let Some(audio) = &audio_system {
                    audio.set_playing(true);
                }

                if window.is_key_down(Key::Enter) || window.is_key_down(Key::Space) {
                    state = GameState::Playing;
                    last_mouse_x = None;
                }
            }
            GameState::Playing => {
                handle_input(&window, &mut player, &map, dt, &mut last_mouse_x, inverted);
                mushroom.update(dt);

                let mouse_down = window.get_mouse_down(MouseButton::Left);
                let shoot_pressed = (mouse_down && !mouse_was_down) || window.is_key_pressed(Key::F, minifb::KeyRepeat::No);
                mouse_was_down = mouse_down;
                if shoot_pressed && try_shoot(&player, &map, &mut mushroom) {
                    // Cada impacto alterna el efecto de mundo invertido.
                    inverted = !inverted;
                }

                renderer::render_scene(&mut buffer, &mut zbuffer, WIDTH, HEIGHT, &player, &map, time);
                let mushroom_boost = if mushroom.is_flashing() { 1.15 } else { 1.0 };
                renderer::draw_billboard(&mut buffer, &zbuffer, WIDTH, HEIGHT, &player, &mushroom, time, mushroom_boost);
                renderer::draw_billboard(&mut buffer, &zbuffer, WIDTH, HEIGHT, &player, &beacon, time, 1.0);
                minimap::draw_minimap(&mut buffer, WIDTH, HEIGHT, &map, &player, goal_point, time);
                draw_crosshair(&mut buffer, WIDTH, HEIGHT);
                draw_hud(&mut buffer, WIDTH, HEIGHT, fps_smoothed, mushroom.hits, inverted);

                if inverted {
                    renderer::invert_colors(&mut buffer);
                }

                let dgx = player.x - goal_point.0;
                let dgy = player.y - goal_point.1;
                if (dgx * dgx + dgy * dgy).sqrt() < GOAL_RADIUS {
                    state = GameState::LevelComplete;
                }
            }
            GameState::LevelComplete => {
                renderer::render_scene(&mut buffer, &mut zbuffer, WIDTH, HEIGHT, &player, &map, time);
                minimap::draw_minimap(&mut buffer, WIDTH, HEIGHT, &map, &player, goal_point, time);
                draw_hud(&mut buffer, WIDTH, HEIGHT, fps_smoothed, mushroom.hits, inverted);
                screens::draw_level_complete(&mut buffer, WIDTH, HEIGHT, mushroom.hits);
                if inverted {
                    renderer::invert_colors(&mut buffer);
                }

                if window.is_key_down(Key::R) {
                    let (sx, sy) = map.spawn_point();
                    player.x = sx;
                    player.y = sy;
                    player.angle = 0.0;
                    mushroom.hits = 0;
                    inverted = false;
                    state = GameState::Playing;
                }
            }
        }

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("Fallo al actualizar el framebuffer de la ventana");
    }
}

/// Lee teclado y mouse y aplica movimiento/rotacion al jugador. Aquí invierto los controles cuando le das al hongo.
fn handle_input(
    window: &Window,
    player: &mut Player,
    map: &Map,
    dt: f32,
    last_mouse_x: &mut Option<f32>,
    inverted: bool,
) {
    let (dir_x, dir_y) = player.dir();
    let (strafe_x, strafe_y) = (-dir_y, dir_x);

    let mut move_x = 0.0;
    let mut move_y = 0.0;

    if window.is_key_down(Key::W) || window.is_key_down(Key::Up) {
        move_x += dir_x;
        move_y += dir_y;
    }
    if window.is_key_down(Key::S) || window.is_key_down(Key::Down) {
        move_x -= dir_x;
        move_y -= dir_y;
    }
    if window.is_key_down(Key::A) {
        move_x -= strafe_x;
        move_y -= strafe_y;
    }
    if window.is_key_down(Key::D) {
        move_x += strafe_x;
        move_y += strafe_y;
    }

    if inverted {
        move_x = -move_x;
        move_y = -move_y;
    }

    let len = (move_x * move_x + move_y * move_y).sqrt();
    if len > 1e-5 {
        move_x = move_x / len * MOVE_SPEED * dt;
        move_y = move_y / len * MOVE_SPEED * dt;
        player.try_move(move_x, move_y, map);
    }

    let turn_sign = if inverted { -1.0 } else { 1.0 };
    if window.is_key_down(Key::Left) {
        player.rotate(-TURN_SPEED * dt * turn_sign);
    }
    if window.is_key_down(Key::Right) {
        player.rotate(TURN_SPEED * dt * turn_sign);
    }

    if let Some((mx, _my)) = window.get_mouse_pos(MouseMode::Pass) {
        if let Some(prev_x) = *last_mouse_x {
            let delta = mx - prev_x;
            player.rotate(delta * MOUSE_SENSITIVITY);
        }
        *last_mouse_x = Some(mx);
    }
}

/// Disparo hitscan
fn try_shoot(player: &Player, map: &Map, mushroom: &mut Mushroom) -> bool {
    let (dir_x, dir_y) = player.dir();
    let to_target_x = mushroom.x - player.x;
    let to_target_y = mushroom.y - player.y;

    let proj = to_target_x * dir_x + to_target_y * dir_y;
    if proj <= 0.0 {
        return false;
    }

    let perp = (to_target_x * dir_y - to_target_y * dir_x).abs();
    if perp > sprite::TARGET_HIT_RADIUS {
        return false;
    }

    let wall_hit = cast_ray(player, player.angle, map);
    if proj < wall_hit.perp_dist {
        mushroom.register_hit();
        return true;
    }
    false
}

/// Dibuja una pequeña mira en el centro de la pantalla.
fn draw_crosshair(buffer: &mut [u32], screen_w: usize, screen_h: usize) {
    let cx = (screen_w / 2) as i32;
    let cy = (screen_h / 2) as i32;
    let color = 0xFFFFFFu32;
    for d in -6..=6 {
        put_pixel(buffer, screen_w, screen_h, cx + d, cy, color);
        put_pixel(buffer, screen_w, screen_h, cx, cy + d, color);
    }
}

fn put_pixel(buffer: &mut [u32], screen_w: usize, screen_h: usize, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 || x as usize >= screen_w || y as usize >= screen_h {
        return;
    }
    buffer[y as usize * screen_w + x as usize] = color;
}

/// Dibuja el HUD superior-izquierdo: FPS, impactos, y un aviso cuando el
/// mundo esta invertido
fn draw_hud(buffer: &mut [u32], screen_w: usize, screen_h: usize, fps: f32, hits: u32, inverted: bool) {
    let fps_text = format!("FPS: {}", fps.round() as i32);
    font::draw_text(buffer, screen_w, screen_h, &fps_text, 10, 10, 2, 0x00FF00);

    let hits_text = format!("IMPACTOS: {}", hits);
    font::draw_text(buffer, screen_w, screen_h, &hits_text, 10, 30, 2, 0xFFFF00);

    if inverted {
        font::draw_text(buffer, screen_w, screen_h, "MUNDO INVERTIDO", 10, 50, 2, 0xFF00FF);
    }
}