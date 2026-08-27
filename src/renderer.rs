// renderer.rs
// Convierte los resultados del raycaster en pixeles reales dentro del
// framebuffer: una columna vertical por cada rayo lanzado, mas piso, techo,
// y sprites tipo billboard (la diana) con oclusion correcta contra paredes.

use crate::map::Map;
use crate::player::Player;
use crate::raycaster::cast_ray;
use crate::sprite::Target;
use crate::textures;

/// Aplica un sombreado simple (mas oscuro) a un color, usado para dar
/// sensacion de profundidad segun la orientacion de la pared golpeada.
fn shade(color: u32, factor: f32) -> u32 {
    let r = ((color >> 16) & 0xFF) as f32 * factor;
    let g = ((color >> 8) & 0xFF) as f32 * factor;
    let b = (color & 0xFF) as f32 * factor;
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Renderiza la escena 3D completa (piso, techo, y paredes con textura
/// procedural) para el frame actual, escribiendo directamente en `buffer`.
/// Tambien llena `zbuffer` con la distancia perpendicular de la pared en
/// cada columna, para que los sprites puedan ocluirse correctamente detras
/// de paredes mas cercanas.
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

/// Dibuja la diana como un sprite billboard: siempre de frente a la camara,
/// escalado por distancia y ocluido por paredes mas cercanas usando el
/// zbuffer que dejo `render_scene`. El angulo se calcula igual que el de
/// los rayos de pared, para que la proyeccion sea consistente (sin efectos
/// de fisheye distintos entre paredes y sprite).
pub fn draw_sprite(
    buffer: &mut [u32],
    zbuffer: &[f32],
    screen_w: usize,
    screen_h: usize,
    player: &Player,
    target: &Target,
    time: f32,
) {
    let rel_x = target.x - player.x;
    let rel_y = target.y - player.y;
    let dist = (rel_x * rel_x + rel_y * rel_y).sqrt();
    if dist < 1e-4 {
        return;
    }

    let angle_to_sprite = rel_y.atan2(rel_x);
    let mut angle_diff = angle_to_sprite - player.angle;
    // Normaliza a [-PI, PI] para tomar el camino angular mas corto.
    while angle_diff > std::f32::consts::PI {
        angle_diff -= std::f32::consts::TAU;
    }
    while angle_diff < -std::f32::consts::PI {
        angle_diff += std::f32::consts::TAU;
    }

    // Si esta muy fuera del FOV (con margen extra para que no "aparezca"
    // de golpe en el borde), no vale la pena procesar la columna.
    if angle_diff.abs() > player.fov {
        return;
    }

    // Distancia "perpendicular" equivalente a la usada en paredes, para que
    // el tamaño en pantalla y el orden de oclusion sean consistentes.
    let perp_dist = (dist * angle_diff.cos()).max(1e-4);

    let screen_x_center = (screen_w as f32 / 2.0) * (1.0 + angle_diff / (player.fov / 2.0));
    let sprite_size = (screen_h as f32 / perp_dist).min(1e5);

    // El flash de impacto agranda levemente la diana como feedback extra.
    let size_boost = if target.is_flashing() { 1.15 } else { 1.0 };
    let sprite_size = sprite_size * size_boost;

    let half_size = sprite_size / 2.0;
    let x_start = (screen_x_center - half_size).max(0.0) as i32;
    let x_end = (screen_x_center + half_size).min(screen_w as f32 - 1.0) as i32;
    let y_start = (screen_h as f32 / 2.0 - half_size).max(0.0) as i32;
    let y_end = (screen_h as f32 / 2.0 + half_size).min(screen_h as f32 - 1.0) as i32;

    for x in x_start..=x_end {
        if x < 0 || x as usize >= screen_w {
            continue;
        }
        // Oclusion: si hay una pared mas cerca que la diana en esta columna, se salta.
        if perp_dist >= zbuffer[x as usize] {
            continue;
        }
        let u = (x as f32 - (screen_x_center - half_size)) / sprite_size.max(1.0);
        for y in y_start..=y_end {
            if y < 0 || y as usize >= screen_h {
                continue;
            }
            let v = (y as f32 - (screen_h as f32 / 2.0 - half_size)) / sprite_size.max(1.0);
            if let Some(color) = target.sample(u, v, time) {
                buffer[y as usize * screen_w + x as usize] = color;
            }
        }
    }
}
