// Estado del jugador (posicion, angulo de vista) y la logica de movimiento
// con deteccion de colisiones para que jamas atraviese una pared.

use crate::map::Map;

/// Radio de colision del jugador 
const PLAYER_RADIUS: f32 = 0.2;

pub struct Player {
    pub x: f32,
    pub y: f32,
    /// Angulo de vista en radianes. 0 = mirando hacia +x.
    pub angle: f32,
    /// Campo de vision (FOV) en radianes.
    pub fov: f32,
}

impl Player {
    /// Crea un jugador nuevo parado en el punto de spawn del mapa.
    pub fn new(map: &Map) -> Self {
        let (sx, sy) = map.spawn_point();
        Player {
            x: sx,
            y: sy,
            angle: 0.0,
            fov: std::f32::consts::FRAC_PI_3, // 60 grados
        }
    }

    /// Gira al jugador `delta` radianes (usado por teclado y por el mouse).
    pub fn rotate(&mut self, delta: f32) {
        self.angle += delta;
        // Se normaliza el angulo para que sea entre 0 y 2pi
        let two_pi = std::f32::consts::TAU;
        self.angle = ((self.angle % two_pi) + two_pi) % two_pi;
    }

    /// Intenta mover al jugador por (dx, dy) en coordenadas de mundo.
    /// Se resuelve eje por eje (X y luego Y) para permitir "deslizarse"
    /// sobre las paredes en vez de quedar pegado si el movimiento no es
    /// perfectamente perpendicular.
    pub fn try_move(&mut self, dx: f32, dy: f32, map: &Map) {
        let new_x = self.x + dx;
        if !Self::collides(new_x, self.y, map) {
            self.x = new_x;
        }

        let new_y = self.y + dy;
        if !Self::collides(self.x, new_y, map) {
            self.y = new_y;
        }
    }

    /// Revisa si un circulo de radio PLAYER_RADIUS centrado en (x, y)
    /// se solapa con alguna celda de pared, muestreando 4 puntos
    /// cardinales alrededor del jugador.
    fn collides(x: f32, y: f32, map: &Map) -> bool {
        let offsets = [
            (PLAYER_RADIUS, 0.0),
            (-PLAYER_RADIUS, 0.0),
            (0.0, PLAYER_RADIUS),
            (0.0, -PLAYER_RADIUS),
        ];
        for (ox, oy) in offsets {
            if map.is_wall_at(x + ox, y + oy) {
                return true;
            }
        }
        false
    }

    /// Vector de direccion (forward) segun el angulo actual.
    pub fn dir(&self) -> (f32, f32) {
        (self.angle.cos(), self.angle.sin())
    }
}
