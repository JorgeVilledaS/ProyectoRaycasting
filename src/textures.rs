// textures.rs
// En vez de cargar archivos de imagen, cada tipo de pared se pinta con una
// textura PROCEDURAL (generada por formulas matematicas / ruido de senos).
// Esto encaja con la tematica psicodelica/surrealista pedida (patrones tipo
// "plasma") y evita depender de assets externos.
//
// Cada funcion recibe:
//   u, v  -> coordenadas dentro de la pared (0.0 a 1.0)
//   t     -> tiempo total transcurrido (segundos), para animar el patron
// y devuelve un color en formato 0x00RRGGBB.

/// Empaqueta 3 canales de color (0-255) en un solo u32, formato usado por minifb.
fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Convierte un valor en [-1, 1] (tipico de sin/cos) a un byte de color [0, 255].
fn wave_to_byte(w: f32) -> u8 {
    (((w + 1.0) * 0.5).clamp(0.0, 1.0) * 255.0) as u8
}

/// Patron "plasma" clasico: suma de senos en distintas frecuencias y fases,
/// desplazado con el tiempo para que el color fluya sobre la pared.
fn plasma(u: f32, v: f32, t: f32, hue_shift: f32) -> u32 {
    let s1 = (u * 10.0 + t).sin();
    let s2 = (v * 10.0 - t * 1.3).sin();
    let s3 = ((u + v) * 8.0 + t * 0.7).sin();
    let s4 = ((u * u + v * v).sqrt() * 12.0 - t).sin();

    let r = wave_to_byte((s1 + s3 + hue_shift).sin());
    let g = wave_to_byte((s2 + s4 + hue_shift * 1.5).sin());
    let b = wave_to_byte((s1 - s2 + s3 * s4 + hue_shift * 0.5).sin());
    rgb(r, g, b)
}

/// Textura tipo 1: espiral roja/naranja pulsante ("neurona encendida").
pub fn tex_type_1(u: f32, v: f32, t: f32) -> u32 {
    let base = plasma(u, v, t, 0.0);
    // Se mezcla con un tinte rojo dominante para diferenciarla claramente.
    let r = ((base >> 16 & 0xFF) as u32 + 120).min(255) as u8;
    rgb(r, ((base >> 8) & 0xFF) as u8 / 2, (base & 0xFF) as u8 / 3)
}

/// Textura tipo 2: rejilla acida verde con lineas que laten.
pub fn tex_type_2(u: f32, v: f32, t: f32) -> u32 {
    let grid = ((u * 8.0).fract() < 0.08) || ((v * 8.0).fract() < 0.08);
    let pulse = ((t * 3.0).sin() * 0.5 + 0.5) * 255.0;
    if grid {
        rgb(20, 220, 40)
    } else {
        rgb(10, pulse as u8, 40)
    }
}

/// Textura tipo 3: ondas azules concentricas tipo "portal liquido".
pub fn tex_type_3(u: f32, v: f32, t: f32) -> u32 {
    let dx = u - 0.5;
    let dy = v - 0.5;
    let d = (dx * dx + dy * dy).sqrt();
    let ring = ((d * 30.0 - t * 2.0).sin() + 1.0) * 0.5;
    rgb((ring * 60.0) as u8, (ring * 140.0) as u8, (150.0 + ring * 105.0) as u8)
}

/// Textura tipo 4: bandas moradas/magenta tipo lava lamp.
pub fn tex_type_4(u: f32, v: f32, t: f32) -> u32 {
    let w = (u * 4.0 + (v * 6.0 + t).sin()).sin();
    let intensity = (w + 1.0) * 0.5;
    rgb((120.0 + intensity * 135.0) as u8, (intensity * 60.0) as u8, (140.0 + intensity * 115.0) as u8)
}

/// Textura tipo 5: cuadros amarillos electricos parpadeantes (tipo panal).
pub fn tex_type_5(u: f32, v: f32, t: f32) -> u32 {
    let cell = (((u * 6.0).floor() as i32) + ((v * 6.0).floor() as i32)) % 2 == 0;
    let flicker = ((t * 5.0 + u * 10.0).sin() * 0.5 + 0.5) * 60.0;
    if cell {
        rgb(230, (200.0 + flicker) as u8, 20)
    } else {
        rgb(60, 50, 5)
    }
}

/// Textura tipo 6: "espejo"/portal celeste con plasma completo, la mas
/// caotica visualmente, para las camaras centrales del cerebro.
pub fn tex_type_6(u: f32, v: f32, t: f32) -> u32 {
    plasma(u, v, t, 2.0)
}

/// Devuelve el color de la textura correspondiente a un tipo de pared (1..=6)
/// en la coordenada (u, v) de esa pared, en el instante t.
/// Si el tipo no se reconoce, cae a un magenta de "textura faltante" bien visible.
pub fn sample_wall(wall_type: u8, u: f32, v: f32, t: f32) -> u32 {
    match wall_type {
        1 => tex_type_1(u, v, t),
        2 => tex_type_2(u, v, t),
        3 => tex_type_3(u, v, t),
        4 => tex_type_4(u, v, t),
        5 => tex_type_5(u, v, t),
        6 => tex_type_6(u, v, t),
        _ => rgb(255, 0, 255),
    }
}

/// Genera un color de piso animado (para reforzar el look psicodelico
/// tambien por debajo de los pies del jugador).
pub fn floor_color(u: f32, v: f32, t: f32) -> u32 {
    let base = plasma(u * 0.3, v * 0.3, t * 0.4, 1.2);
    // Se oscurece para que no compita visualmente con las paredes.
    let r = ((base >> 16) & 0xFF) as u32 * 30 / 100;
    let g = ((base >> 8) & 0xFF) as u32 * 30 / 100;
    let b = (base & 0xFF) as u32 * 30 / 100;
    rgb(r as u8, g as u8, b as u8)
}

/// Genera un color de techo animado, distinto y mas oscuro que el piso.
pub fn ceiling_color(t: f32) -> u32 {
    let pulse = ((t * 0.6).sin() * 0.5 + 0.5) * 40.0;
    rgb(10 + pulse as u8, 5, 20 + pulse as u8)
}
