// font.rs
// Fuente de mapa de bits, muy simple, de 5 columnas x 7 filas por caracter.
// Existe para poder dibujar texto legible (titulos, instrucciones) sin
// depender de una libreria externa de fuentes/TTF, mantiendo el proyecto
// simple como pide el enunciado.
//
// Solo cubre mayusculas, digitos basicos y espacio: lo suficiente para los
// textos de la interfaz de este proyecto.

/// Devuelve el patron de 7 filas (cada fila es un &str de 5 caracteres,
/// '#' = pixel encendido, '.' = apagado) para una letra dada.
/// Caracteres no reconocidos se devuelven como espacio en blanco.
fn glyph(c: char) -> [&'static str; 7] {
    match c.to_ascii_uppercase() {
        'A' => [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
        'C' => [".####", "#....", "#....", "#....", "#....", "#....", ".####"],
        'D' => ["####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####."],
        'E' => ["#####", "#....", "#....", "####.", "#....", "#....", "#####"],
        'F' => ["#####", "#....", "#....", "####.", "#....", "#....", "#...."],
        'G' => [".####", "#....", "#....", "#.###", "#...#", "#...#", ".####"],
        'I' => ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "#####"],
        'J' => ["..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##.."],
        'L' => ["#....", "#....", "#....", "#....", "#....", "#....", "#####"],
        'M' => ["#...#", "##.##", "#.#.#", "#...#", "#...#", "#...#", "#...#"],
        'N' => ["#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#"],
        'O' => [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
        'P' => ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."],
        'R' => ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"],
        'S' => [".####", "#....", "#....", ".###.", "....#", "....#", "####."],
        'T' => ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."],
        'U' => ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
        'V' => ["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."],
        'W' => ["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "##.##", "#...#"],
        'Y' => ["#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."],
        '0' => [".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."],
        '1' => ["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", ".###."],
        '2' => [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####"],
        '3' => ["####.", "....#", "....#", "..##.", "....#", "....#", "####."],
        '4' => ["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#."],
        '5' => ["#####", "#....", "####.", "....#", "....#", "#...#", ".###."],
        '6' => [".###.", "#....", "#....", "####.", "#...#", "#...#", ".###."],
        '7' => ["#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#..."],
        '8' => [".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###."],
        '9' => [".###.", "#...#", "#...#", ".####", "....#", "....#", ".###."],
        ':' => [".....", "..#..", ".....", ".....", "..#..", ".....", "....."],
        '+' => [".....", "..#..", "..#..", "#####", "..#..", "..#..", "....."],
        '-' => [".....", ".....", ".....", "#####", ".....", ".....", "....."],
        _ => [".....", ".....", ".....", ".....", ".....", ".....", "....."],
    }
}

/// Dibuja `text` en el framebuffer empezando en (x0, y0), con cada pixel de
/// la fuente escalado a `scale` pixeles reales, en el color `color`.
/// Se salta a la siguiente letra dejando 1 columna de espacio entre ellas.
pub fn draw_text(
    buffer: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    text: &str,
    x0: i32,
    y0: i32,
    scale: i32,
    color: u32,
) {
    let mut cursor_x = x0;
    for ch in text.chars() {
        let rows = glyph(ch);
        for (row_idx, row) in rows.iter().enumerate() {
            for (col_idx, pixel) in row.chars().enumerate() {
                if pixel != '#' {
                    continue;
                }
                let px = cursor_x + col_idx as i32 * scale;
                let py = y0 + row_idx as i32 * scale;
                for sy in 0..scale {
                    for sx in 0..scale {
                        let fx = px + sx;
                        let fy = py + sy;
                        if fx < 0 || fy < 0 || fx as usize >= screen_w || fy as usize >= screen_h {
                            continue;
                        }
                        buffer[fy as usize * screen_w + fx as usize] = color;
                    }
                }
            }
        }
        cursor_x += (5 + 1) * scale; // 5 columnas de letra + 1 de espacio
    }
}

/// Ancho en pixeles que ocuparia `text` dibujado con `draw_text` a esta escala.
/// Util para centrar texto horizontalmente.
pub fn text_width(text: &str, scale: i32) -> i32 {
    text.chars().count() as i32 * (5 + 1) * scale
}
