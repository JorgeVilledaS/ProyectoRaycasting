
// Sprites tipo billboard. Solo hay dos:
// - Mushroom: el objetivo de disparo. Al recibir un impacto, alterna el
//   efecto "mundo invertido" (ver main.rs) ademas de su propio flash visual.
// - Meta: la baliza de meta, una columna de luz pulsante facil de ver
//   desde lejos para que el punto final del nivel sea obvio.

/// Trait comun para poder reusar el mismo dibujado de billboard.
pub trait Billboard {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    /// Color en coordenadas locales (u, v) en [0,1]x[0,1]. `None` = transparente.
    fn sample(&self, u: f32, v: f32, time: f32) -> Option<u32>;
}

/// Radio de colision (en unidades de mundo) para saber si un disparo pasa
/// lo bastante cerca del hongo como para contar como impacto.
pub const TARGET_HIT_RADIUS: f32 = 0.45;

pub struct Mushroom {
    pub x: f32,
    pub y: f32,
    pub hits: u32,
    hit_flash: f32,
}

impl Mushroom {
    pub fn new(x: f32, y: f32) -> Self {
        Mushroom { x, y, hits: 0, hit_flash: 0.0 }
    }

    pub fn update(&mut self, dt: f32) {
        if self.hit_flash > 0.0 {
            self.hit_flash = (self.hit_flash - dt).max(0.0);
        }
    }

    /// Registra un impacto: suma al contador y dispara el flash visual. Toda la inversión de colores la hice en el main.
    pub fn register_hit(&mut self) {
        self.hits += 1;
        self.hit_flash = 0.35;
    }

    pub fn is_flashing(&self) -> bool {
        self.hit_flash > 0.0
    }
}

impl Billboard for Mushroom {
    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    /// Dibuja un hongo psicodelico: sombrero rojo con puntos blancos y un
    /// tallo claro debajo. Al recibir un impacto, destella en blanco/amarillo
    fn sample(&self, u: f32, v: f32, time: f32) -> Option<u32> {
        if self.hit_flash > 0.0 {
            let dx = u - 0.5;
            let dy = v - 0.45;
            if (dx * dx + dy * dy).sqrt() > 0.48 {
                return None;
            }
            let t = self.hit_flash / 0.35;
            let r = 255u8;
            let g = (220.0 + 35.0 * t) as u8;
            let b = (120.0 * t) as u8;
            return Some(((r as u32) << 16) | ((g as u32) << 8) | b as u32);
        }

        let cap_cx = 0.5;
        let cap_cy = 0.42;
        let dx = u - cap_cx;
        let dy = v - cap_cy;
        let cap_dist = (dx * dx + (dy * 1.3) * (dy * 1.3)).sqrt();
        if cap_dist < 0.42 && v < cap_cy + 0.06 {

            let dot_pattern = ((u * 14.0 + time * 0.3).sin() * (v * 14.0).cos()).abs();
            if dot_pattern > 0.82 {
                return Some(0xFFFFFFu32);
            }
            return Some(0xE81020u32);
        }

        if (0.40..=0.60).contains(&u) && (0.48..=0.85).contains(&v) {
            return Some(0xF2E6C9u32);
        }

        None
    }
}

/// Baliza de meta
pub struct Beacon {
    pub x: f32,
    pub y: f32,
}

impl Beacon {
    pub fn new(x: f32, y: f32) -> Self {
        Beacon { x, y }
    }
}

impl Billboard for Beacon {
    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    /// Columna vertical brillante y angosta
    fn sample(&self, u: f32, v: f32, time: f32) -> Option<u32> {
        let pulse = (time * 3.0).sin() * 0.5 + 0.5; // 0..1
        let dx = (u - 0.5).abs();

        // Haz de luz
        let beam_half_width = 0.05 + 0.03 * v;
        if dx < beam_half_width {
            let g = (200.0 + 55.0 * pulse) as u8;
            return Some(((255u32) << 16) | ((g as u32) << 8) | 40u32);
        }

        // Base brillante tipo estrella
        if v > 0.75 {
            let base_dist = (dx * dx + (v - 0.9) * (v - 0.9)).sqrt();
            if base_dist < 0.18 + 0.05 * pulse {
                return Some(0xFFFF66u32);
            }
        }

        None
    }
}