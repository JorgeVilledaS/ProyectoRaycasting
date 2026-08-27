// raycaster.rs
// Implementacion del algoritmo de raycasting estilo Wolfenstein 3D usando
// DDA (Digital Differential Analysis) para encontrar, por cada columna de
// pantalla, en que punto del mapa un rayo choca contra una pared.

use crate::map::Map;
use crate::player::Player;

/// Resultado de lanzar un rayo: toda la info que necesita el renderer
/// para pintar la columna de pantalla correspondiente.
pub struct RayHit {
    /// Distancia perpendicular a la pared (ya corregida de "ojo de pez").
    pub perp_dist: f32,
    /// Tipo de pared golpeada (1..=6), usado para elegir la textura.
    pub wall_type: u8,
    /// Coordenada horizontal dentro de la textura de la pared (0.0 a 1.0).
    pub wall_x: f32,
    /// True si el rayo golpeo una pared orientada en Y (para sombreado).
    pub side_is_y: bool,
}

/// Lanza un unico rayo desde el jugador con un angulo relativo `ray_angle`
/// (en radianes, ya absoluto respecto al mundo) y devuelve donde impacta.
pub fn cast_ray(player: &Player, ray_angle: f32, map: &Map) -> RayHit {
    let ray_dir_x = ray_angle.cos();
    let ray_dir_y = ray_angle.sin();

    let mut map_x = player.x.floor() as i32;
    let mut map_y = player.y.floor() as i32;

    // Distancia que hay que avanzar en el rayo para cruzar una celda completa
    // en cada eje. Se evita division por cero con un numero muy grande.
    let delta_dist_x = if ray_dir_x.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir_x).abs() };
    let delta_dist_y = if ray_dir_y.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir_y).abs() };

    let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
        (-1, (player.x - map_x as f32) * delta_dist_x)
    } else {
        (1, (map_x as f32 + 1.0 - player.x) * delta_dist_x)
    };

    let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
        (-1, (player.y - map_y as f32) * delta_dist_y)
    } else {
        (1, (map_y as f32 + 1.0 - player.y) * delta_dist_y)
    };

    let mut side_is_y = false;
    let mut wall_type = 0u8;

    // Avanza celda por celda (DDA) hasta encontrar una pared.
    // El limite de 256 pasos evita cualquier bucle infinito si algo
    // saliera mal (p.ej. jugador fuera del mapa), protegiendo contra crashes.
    for _ in 0..256 {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side_is_y = false;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side_is_y = true;
        }

        let cell = map.get(map_x, map_y);
        if cell != 0 {
            wall_type = cell;
            break;
        }
    }

    // Distancia perpendicular (no la distancia euclidiana real) para
    // evitar la distorsion de "ojo de pez" en los bordes de la pantalla.
    let perp_dist = if !side_is_y {
        (map_x as f32 - player.x + (1 - step_x) as f32 / 2.0) / ray_dir_x
    } else {
        (map_y as f32 - player.y + (1 - step_y) as f32 / 2.0) / ray_dir_y
    }
    .abs()
    .max(1e-4);

    // Punto exacto de impacto, usado para calcular la coordenada de textura.
    let wall_x_raw = if !side_is_y {
        player.y + perp_dist * ray_dir_y
    } else {
        player.x + perp_dist * ray_dir_x
    };
    let wall_x = wall_x_raw - wall_x_raw.floor();

    RayHit { perp_dist, wall_type, wall_x, side_is_y }
}
