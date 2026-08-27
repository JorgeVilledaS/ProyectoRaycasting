// main.rs
// Punto de entrada del proyecto. Arma la ventana, arranca el audio de
// fondo, y corre el game loop: procesa input -> actualiza estado -> dibuja.
//
// Proyecto: Ray Caster psicodelico - Curso de Graficas por Computadora.

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
use sprite::Target;
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
const GOAL_RADIUS: f32 = 0.6;

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

    // Limita el loop a ~60 FPS para que el juego no corra a velocidad
    // distinta segun la maquina y para no saturar la CPU innecesariamente.
    window.set_target_fps(60);

    let map = Map::brain_level();
    let mut player = Player::new(&map);
    let (goal_x, goal_y) = map.goal_point();
    let (target_x, target_y) = map.target_point();
    let mut target = Target::new(target_x, target_y);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut zbuffer: Vec<f32> = vec![1e30; WIDTH];

    // Si no hay dispositivo de audio (por ejemplo corriendo en un servidor
    // sin sonido) el juego debe seguir funcionando igual, por eso es Option.
    let audio_system = audio::AudioSystem::start();

    let mut state = GameState::Welcome;
    let start_time = Instant::now();
    let mut last_frame = Instant::now();

    let mut last_mouse_x: Option<f32> = None;
    let mut mouse_was_down = false;
    let mut fps_smoothed: f32 = 60.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(0.05); // clamp evita saltos si hay lag
        last_frame = now;
        let time = start_time.elapsed().as_secs_f32();

        // FPS suavizado con media movil exponencial, para que el numero no
        // salte de forma ilegible frame a frame.
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
                handle_input(&window, &mut player, &map, dt, &mut last_mouse_x);
                target.update(dt);

                // Disparo: flanco de subida del boton izquierdo del mouse o barra espaciadora.
                let mouse_down = window.get_mouse_down(MouseButton::Left);
                let shoot_pressed = (mouse_down && !mouse_was_down) || window.is_key_pressed(Key::F, minifb::KeyRepeat::No);
                mouse_was_down = mouse_down;
                if shoot_pressed {
                    try_shoot(&player, &map, &mut target);
                }

                renderer::render_scene(&mut buffer, &mut zbuffer, WIDTH, HEIGHT, &player, &map, time);
                renderer::draw_sprite(&mut buffer, &zbuffer, WIDTH, HEIGHT, &player, &target, time);
                minimap::draw_minimap(&mut buffer, WIDTH, HEIGHT, &map, &player);
                draw_crosshair(&mut buffer, WIDTH, HEIGHT);
                draw_hud(&mut buffer, WIDTH, HEIGHT, fps_smoothed, target.hits);

                let dgx = player.x - goal_x;
                let dgy = player.y - goal_y;
                if (dgx * dgx + dgy * dgy).sqrt() < GOAL_RADIUS {
                    state = GameState::LevelComplete;
                }
            }
            GameState::LevelComplete => {
                renderer::render_scene(&mut buffer, &mut zbuffer, WIDTH, HEIGHT, &player, &map, time);
                minimap::draw_minimap(&mut buffer, WIDTH, HEIGHT, &map, &player);
                draw_hud(&mut buffer, WIDTH, HEIGHT, fps_smoothed, target.hits);
                screens::draw_level_complete(&mut buffer, WIDTH, HEIGHT, target.hits);

                // R reinicia el nivel desde el spawn original.
                if window.is_key_down(Key::R) {
                    let (sx, sy) = map.spawn_point();
                    player.x = sx;
                    player.y = sy;
                    player.angle = 0.0;
                    target.hits = 0;
                    state = GameState::Playing;
                }
            }
        }

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("Fallo al actualizar el framebuffer de la ventana");
    }
}

/// Lee teclado y mouse y aplica movimiento/rotacion al jugador.
/// Toda colision pasa por `Player::try_move`, que jamas permite atravesar
/// una pared (ver player.rs), cumpliendo el requisito de no atravesar muros.
fn handle_input(window: &Window, player: &mut Player, map: &Map, dt: f32, last_mouse_x: &mut Option<f32>) {
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

    let len = (move_x * move_x + move_y * move_y).sqrt();
    if len > 1e-5 {
        move_x = move_x / len * MOVE_SPEED * dt;
        move_y = move_y / len * MOVE_SPEED * dt;
        player.try_move(move_x, move_y, map);
    }

    if window.is_key_down(Key::Left) {
        player.rotate(-TURN_SPEED * dt);
    }
    if window.is_key_down(Key::Right) {
        player.rotate(TURN_SPEED * dt);
    }

    if let Some((mx, _my)) = window.get_mouse_pos(MouseMode::Pass) {
        if let Some(prev_x) = *last_mouse_x {
            let delta = mx - prev_x;
            player.rotate(delta * MOUSE_SENSITIVITY);
        }
        *last_mouse_x = Some(mx);
    }
}

/// Disparo hitscan: es "lo opuesto" a los rayos de renderizado, en vez de
/// pintar una columna de pantalla, se lanza UN rayo desde el centro de la
/// mira para ver si intersecta a la diana antes que a cualquier pared.
/// No hay proyectil ni fisica de bala: el impacto es instantaneo (hitscan).
fn try_shoot(player: &Player, map: &Map, target: &mut Target) -> bool {
    let (dir_x, dir_y) = player.dir();
    let to_target_x = target.x - player.x;
    let to_target_y = target.y - player.y;

    // Distancia del jugador a la diana proyectada sobre la direccion de vista.
    let proj = to_target_x * dir_x + to_target_y * dir_y;
    if proj <= 0.0 {
        return false; // la diana esta detras del jugador
    }

    // Distancia perpendicular de la diana a la linea de disparo (para dar
    // algo de tolerancia de puntería, como un area de impacto real).
    let perp = (to_target_x * dir_y - to_target_y * dir_x).abs();
    if perp > sprite::TARGET_HIT_RADIUS {
        return false; // el disparo pasa de largo
    }

    // Se verifica que ninguna pared este mas cerca que la diana en esa direccion.
    let wall_hit = cast_ray(player, player.angle, map);
    if proj < wall_hit.perp_dist {
        target.register_hit();
        return true;
    }
    false
}

/// Dibuja una pequeña mira (crosshair) en el centro de la pantalla, para
/// tener referencia de hacia donde se dispara.
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

/// Dibuja el HUD superior-izquierdo: FPS actuales y cantidad de impactos
/// registrados en la diana, usando la mini fuente de pixeles.
fn draw_hud(buffer: &mut [u32], screen_w: usize, screen_h: usize, fps: f32, hits: u32) {
    let fps_text = format!("FPS: {}", fps.round() as i32);
    font::draw_text(buffer, screen_w, screen_h, &fps_text, 10, 10, 2, 0x00FF00);

    let hits_text = format!("IMPACTOS: {}", hits);
    font::draw_text(buffer, screen_w, screen_h, &hits_text, 10, 30, 2, 0xFFFF00);
}
