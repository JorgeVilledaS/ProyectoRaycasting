// La "diana" (target) es un sprite tipo billboard: siempre mira hacia la
// camara sin importar el angulo del jugador. Sirve para probar el disparo
// hitscan (ver `try_shoot` en main.rs) y cumple el requisito de tener al
// menos una animacion en pantalla.

/// Radio de colision (en unidades de mundo) usado para saber si un disparo
/// pasa lo bastante cerca del centro de la diana como para contar como impacto.
pub const TARGET_HIT_RADIUS: f32 = 0.45;

pub struct Target {
    pub x: f32,
    pub y: f32,
    pub hits: u32,
    /// Cuenta regresiva (segundos) del "flash" de impacto: mientras es > 0,
    /// la diana se dibuja mas brillante/mas grande, dando feedback visual claro.
    hit_flash: f32,
}

impl Target {
    /// Crea una diana nueva en la posicion dada, sin impactos todavia.
    pub fn new(x: f32, y: f32) -> Self {
        Target { x, y, hits: 0, hit_flash: 0.0 }
    }

    /// Avanza el temporizador de animacion de impacto. Se llama una vez por frame.
    pub fn update(&mut self, dt: f32) {
        if self.hit_flash > 0.0 {
            self.hit_flash = (self.hit_flash - dt).max(0.0);
        }
    }

    /// Registra un impacto: suma al contador y dispara la animacion de flash.
    pub fn register_hit(&mut self) {
        self.hits += 1;
        self.hit_flash = 0.35;
    }

    /// True mientras dura la animacion de "acabo de recibir un impacto".
    pub fn is_flashing(&self) -> bool {
        self.hit_flash > 0.0
    }

    /// Textura de la diana en coordenadas locales (u, v) en [0,1]x[0,1],
    /// con el centro del billboard en (0.5, 0.5). Devuelve `None` fuera del
    /// circulo (para que el resto del cuadrado sea transparente).
    ///
    /// La ANIMACION consiste en: los anillos rotan continuamente con el
    /// tiempo (efecto hipnotico/psicodelico) y, al recibir un impacto, la
    /// diana entera destella en blanco por una fraccion de segundo.
    pub fn sample(&self, u: f32, v: f32, time: f32) -> Option<u32> {
        let dx = u - 0.5;
        let dy = v - 0.5;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > 0.5 {
            return None; // fuera del circulo: transparente
        }

        if self.hit_flash > 0.0 {
            // Destello blanco/amarillo que se apaga a medida que pasa el timer.
            let t = self.hit_flash / 0.35;
            let r = 255u8;
            let g = (200.0 + 55.0 * t) as u8;
            let b = (80.0 * t) as u8;
            return Some(((r as u32) << 16) | ((g as u32) << 8) | b as u32);
        }

        // Anillos concentricos tipo dartboard, con rotacion continua dada
        // por el angulo polar del punto mas un offset dependiente del tiempo.
        let angle = dy.atan2(dx) + time * 1.5;
        let ring = (dist * 10.0).floor() as i32;
        let stripe = ((angle * 3.0).sin() > 0.0) as i32;

        let color = if ring % 2 == (stripe % 2) {
            0xFF2244u32 // rojo psicodelico
        } else {
            0xFFFFFFu32 // blanco
        };
        Some(color)
    }
}
