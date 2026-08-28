// Dibuja un minimapa en la esquina superior derecha de la pantalla

use crate::map::Map;
use crate::player::Player;

const CELL_PX: usize = 6;
const MARGIN: usize = 12;

/// Pinta un pixel del framebuffer si esta dentro de los limites.
fn put_pixel(buffer: &mut [u32], screen_w: usize, screen_h: usize, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 || x as usize >= screen_w || y as usize >= screen_h {
        return;
    }
    buffer[y as usize * screen_w + x as usize] = color;
}

/// Dibuja el minimapa completo
pub fn draw_minimap(
    buffer: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    map: &Map,
    player: &Player,
    goal: (f32, f32),
    time: f32,
) {
    let map_px_w = map.width * CELL_PX;
    let map_px_h = map.height * CELL_PX;

    let origin_x = screen_w.saturating_sub(map_px_w + MARGIN) as i32;
    let origin_y = MARGIN as i32;

    for py in 0..map_px_h {
        for px in 0..map_px_w {
            put_pixel(buffer, screen_w, screen_h, origin_x + px as i32, origin_y + py as i32, 0x101018);
        }
    }

    for cy in 0..map.height {
        for cx in 0..map.width {
            let cell = map.get(cx as i32, cy as i32);
            if cell == 0 {
                continue;
            }
            let color = match cell {
                1 => 0xFF5544,
                2 => 0x33DD55,
                3 => 0x2299FF,
                4 => 0xBB44DD,
                5 => 0xEEDD22,
                6 => 0x55EEEE,
                _ => 0xFFFFFF,
            };
            for py in 0..CELL_PX {
                for px in 0..CELL_PX {
                    put_pixel(
                        buffer,
                        screen_w,
                        screen_h,
                        origin_x + (cx * CELL_PX + px) as i32,
                        origin_y + (cy * CELL_PX + py) as i32,
                        color,
                    );
                }
            }
        }
    }

    // Marcador de META
    let gx = origin_x + (goal.0 * CELL_PX as f32) as i32;
    let gy = origin_y + (goal.1 * CELL_PX as f32) as i32;
    let pulse = ((time * 4.0).sin() * 0.5 + 0.5) * 3.0;
    let goal_radius = 3 + pulse as i32;
    for dy in -goal_radius..=goal_radius {
        for dx in -goal_radius..=goal_radius {
            if dx * dx + dy * dy <= goal_radius * goal_radius {
                put_pixel(buffer, screen_w, screen_h, gx + dx, gy + dy, 0xFFEE00);
            }
        }
    }

    // Marcador del jugador
    let px = origin_x + (player.x * CELL_PX as f32) as i32;
    let py = origin_y + (player.y * CELL_PX as f32) as i32;
    for dy in -2..=2 {
        for dx in -2..=2 {
            put_pixel(buffer, screen_w, screen_h, px + dx, py + dy, 0xFFFFFF);
        }
    }

    // y una linea corta que indica hacia donde mira
    let (dx, dy) = player.dir();
    for i in 0..8 {
        let lx = px + (dx * i as f32) as i32;
        let ly = py + (dy * i as f32) as i32;
        put_pixel(buffer, screen_w, screen_h, lx, ly, 0xFF00FF);
    }

    // Borde del minimapa para separarlo visualmente del resto de la escena.
    for px in 0..map_px_w {
        put_pixel(buffer, screen_w, screen_h, origin_x + px as i32, origin_y - 1, 0xFFFFFF);
        put_pixel(buffer, screen_w, screen_h, origin_x + px as i32, origin_y + map_px_h as i32, 0xFFFFFF);
    }
    for py in 0..map_px_h {
        put_pixel(buffer, screen_w, screen_h, origin_x - 1, origin_y + py as i32, 0xFFFFFF);
        put_pixel(buffer, screen_w, screen_h, origin_x + map_px_w as i32, origin_y + py as i32, 0xFFFFFF);
    }
}