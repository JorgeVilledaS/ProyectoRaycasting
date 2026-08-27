// screens.rs
// Pantallas de la maquina de estados

use crate::font;
use crate::textures;

/// Dibuja el fondo animado de la pantalla de bienvenida
pub fn draw_welcome_background(buffer: &mut [u32], screen_w: usize, screen_h: usize, time: f32) {
    for y in 0..screen_h {
        let v = y as f32 / screen_h as f32;
        for x in 0..screen_w {
            let u = x as f32 / screen_w as f32;
            buffer[y * screen_w + x] = textures::tex_type_6(u, v, time);
        }
    }
}

/// Dibuja un rectangulo lleno de un color solido para darle fondo al texto.
fn fill_rect(buffer: &mut [u32], screen_w: usize, screen_h: usize, x0: i32, y0: i32, w: i32, h: i32, color: u32) {
    for y in y0..(y0 + h) {
        if y < 0 || y as usize >= screen_h {
            continue;
        }
        for x in x0..(x0 + w) {
            if x < 0 || x as usize >= screen_w {
                continue;
            }
            buffer[y as usize * screen_w + x as usize] = color;
        }
    }
}

/// Dibuja centrado, en la fila `y`, un texto a la escala dada.
fn draw_centered(buffer: &mut [u32], screen_w: usize, screen_h: usize, text: &str, y: i32, scale: i32, color: u32) {
    let w = font::text_width(text, scale);
    let x = (screen_w as i32 - w) / 2;
    font::draw_text(buffer, screen_w, screen_h, text, x, y, scale, color);
}

/// Dibuja el panel completo de la pantalla de bienvenida: titulo del juego,
/// instrucciones de control, y un mensaje "presiona ENTER" parpadeante.
pub fn draw_welcome_panel(buffer: &mut [u32], screen_w: usize, screen_h: usize, blink_on: bool) {
    let panel_w = (screen_w as f32 * 0.72) as i32;
    let panel_h = (screen_h as f32 * 0.46) as i32;
    let panel_x = (screen_w as i32 - panel_w) / 2;
    let panel_y = (screen_h as i32 - panel_h) / 2;

    // Fondo solido detras del texto para que se lea sobre el plasma animado.
    fill_rect(buffer, screen_w, screen_h, panel_x, panel_y, panel_w, panel_h, 0x000000);
    fill_rect(buffer, screen_w, screen_h, panel_x + 4, panel_y + 4, panel_w - 8, panel_h - 8, 0x1a0033);

    let mut y = panel_y + 26;
    draw_centered(buffer, screen_w, screen_h, "RAYCASTER PSICODELICO", y, 4, 0xFF33CC);
    y += 4 * 9 + 18;

    draw_centered(buffer, screen_w, screen_h, "WASD Y MOUSE PARA MOVERTE", y, 2, 0xFFFFFF);
    y += 2 * 9 + 10;

    draw_centered(buffer, screen_w, screen_h, "ENTER PARA JUGAR", y, 2, 0x55EEEE);
    y += 2 * 9 + 18;

    if blink_on {
        draw_centered(buffer, screen_w, screen_h, "PULSA ENTER", y, 3, 0x33FF99);
    }
}

/// Dibuja el overlay de "nivel completado" cuando el jugador llega al punto
/// final del mapa: un panel con el mensaje y la cantidad de impactos hechos
/// en la diana durante la partida, mas instrucciones para reiniciar.
pub fn draw_level_complete(buffer: &mut [u32], screen_w: usize, screen_h: usize, hits: u32) {
    let panel_w = (screen_w as f32 * 0.6) as i32;
    let panel_h = (screen_h as f32 * 0.34) as i32;
    let panel_x = (screen_w as i32 - panel_w) / 2;
    let panel_y = (screen_h as i32 - panel_h) / 2;

    fill_rect(buffer, screen_w, screen_h, panel_x, panel_y, panel_w, panel_h, 0x000000);
    fill_rect(buffer, screen_w, screen_h, panel_x + 4, panel_y + 4, panel_w - 8, panel_h - 8, 0x0d2b0d);

    let mut y = panel_y + 24;
    draw_centered(buffer, screen_w, screen_h, "NIVEL COMPLETADO", y, 3, 0x33FF99);
    y += 3 * 9 + 20;

    let hits_text = format!("IMPACTOS EN LA DIANA: {}", hits);
    draw_centered(buffer, screen_w, screen_h, &hits_text, y, 2, 0xFFFFFF);
    y += 2 * 9 + 14;

    draw_centered(buffer, screen_w, screen_h, "PULSA R PARA REINICIAR", y, 2, 0x55EEEE);
}
