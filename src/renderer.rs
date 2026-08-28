// renderer.rs
// Convierte los resultados del raycaster en pixeles reales dentro del
// framebuffer: una columna vertical por cada rayo lanzado, mas piso, techo,
// y sprites tipo billboard (hongo y baliza).

use crate::map::Map;
use crate::player::Player;
use crate::raycaster::cast_ray;
use crate::sprite::Billboard;
use crate::textures;

fn shade(color: u32, factor: f32) -> u32 {
    let r = ((color >> 16) & 0xFF) as f32 * factor;
    let g = ((color >> 8) & 0xFF) as f32 * factor;
    let b = (color & 0xFF) as f32 * factor;
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Renderiza la escena 3D completa (piso, techo, y paredes con textura
/// procedural) para el frame actual, escribiendo directamente en `buffer`.

pub fn render_scene(
    buffer: &mut [u32],
    zbuffer: &mut [f32],
    screen_w: usize,
    screen_h: usize,
    player: &Player,
    map: &Map,
    time: f32,
) {
    let half_h = screen_h as f32 / 2.0;

    // --- Piso y techo ---
    for y in 0..screen_h {
        let is_floor = y as f32 > half_h;
        let color = if is_floor {
            let v = (y as f32 - half_h) / half_h;
            textures::floor_color(time * 0.05, v, time)
        } else {
            textures::ceiling_color(time)
        };
        for x in 0..screen_w {
            buffer[y * screen_w + x] = color;
        }
    }

    // --- Paredes: un rayo por columna de pantalla ---
    for x in 0..screen_w {
        let camera_x = (2.0 * x as f32 / screen_w as f32) - 1.0;
        let ray_angle = player.angle + camera_x * (player.fov / 2.0);

        let hit = cast_ray(player, ray_angle, map);
        zbuffer[x] = hit.perp_dist;

        let line_height = (screen_h as f32 / hit.perp_dist).min(1e5);
        let draw_start = (half_h - line_height / 2.0).max(0.0) as usize;
        let draw_end = (half_h + line_height / 2.0).min(screen_h as f32 - 1.0) as usize;

        let shade_factor = if hit.side_is_y { 0.7 } else { 1.0 };

        for y in draw_start..=draw_end {
            let v = (y as f32 - draw_start as f32) / (line_height.max(1.0));
            let color = textures::sample_wall(hit.wall_type, hit.wall_x, v, time);
            buffer[y * screen_w + x] = shade(color, shade_factor);
        }
    }
}

/// Dibuja cualquier sprite billboard (hongo, baliza, etc.)
pub fn draw_billboard(
    buffer: &mut [u32],
    zbuffer: &[f32],
    screen_w: usize,
    screen_h: usize,
    player: &Player,
    sprite: &dyn Billboard,
    time: f32,
    size_boost: f32,
) {
    let rel_x = sprite.x() - player.x;
    let rel_y = sprite.y() - player.y;
    let dist = (rel_x * rel_x + rel_y * rel_y).sqrt();
    if dist < 1e-4 {
        return;
    }

    let angle_to_sprite = rel_y.atan2(rel_x);
    let mut angle_diff = angle_to_sprite - player.angle;
    while angle_diff > std::f32::consts::PI {
        angle_diff -= std::f32::consts::TAU;
    }
    while angle_diff < -std::f32::consts::PI {
        angle_diff += std::f32::consts::TAU;
    }

    if angle_diff.abs() > player.fov {
        return;
    }

    let perp_dist = (dist * angle_diff.cos()).max(1e-4);

    let screen_x_center = (screen_w as f32 / 2.0) * (1.0 + angle_diff / (player.fov / 2.0));
    let sprite_size = ((screen_h as f32 / perp_dist).min(1e5)) * size_boost;

    let half_size = sprite_size / 2.0;
    let x_start = (screen_x_center - half_size).max(0.0) as i32;
    let x_end = (screen_x_center + half_size).min(screen_w as f32 - 1.0) as i32;
    let y_start = (screen_h as f32 / 2.0 - half_size).max(0.0) as i32;
    let y_end = (screen_h as f32 / 2.0 + half_size).min(screen_h as f32 - 1.0) as i32;

    for x in x_start..=x_end {
        if x < 0 || x as usize >= screen_w {
            continue;
        }
        if perp_dist >= zbuffer[x as usize] {
            continue;
        }
        let u = (x as f32 - (screen_x_center - half_size)) / sprite_size.max(1.0);
        for y in y_start..=y_end {
            if y < 0 || y as usize >= screen_h {
                continue;
            }
            let v = (y as f32 - (screen_h as f32 / 2.0 - half_size)) / sprite_size.max(1.0);
            if let Some(color) = sprite.sample(u, v, time) {
                buffer[y as usize * screen_w + x as usize] = color;
            }
        }
    }
}

/// Invierte los colores de TODO el framebuffer (efecto "mundo invertido"
/// tras dispararle al hongo). 
pub fn invert_colors(buffer: &mut [u32]) {
    for pixel in buffer.iter_mut() {
        let r = 255 - ((*pixel >> 16) & 0xFF);
        let g = 255 - ((*pixel >> 8) & 0xFF);
        let b = 255 - (*pixel & 0xFF);
        *pixel = (r << 16) | (g << 8) | b;
    }
}