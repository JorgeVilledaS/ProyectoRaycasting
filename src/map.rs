// map.rs
// Define el nivel jugable como una grilla 2D y expone utilidades de
// consulta (que celda hay en tal posicion, si es pared, etc).
// Cada numero distinto de la grilla representa un TIPO DE PARED distinto,
// y cada tipo de pared tiene su propia textura procedural psicodelica
// (ver textures.rs).

/// Ancho y alto de cada celda del mapa en "unidades de mundo".
pub const TILE_SIZE: f32 = 1.0;

/// Representa el nivel completo: una grilla rectangular de enteros.
/// 0 = piso libre (caminable). 1..=6 = distintos tipos de pared.
pub struct Map {
    pub width: usize,
    pub height: usize,
    cells: Vec<u8>,
}

impl Map {
    /// Construye el nivel "psychedelic arena" del proyecto: un piso abierto
    /// (para que el jugador SIEMPRE pueda desplazarse libremente, sin quedar
    /// encerrado) con bloques de colores flotando, cada uno de un tipo de
    /// pared distinto (textura distinta). Layout verificado por BFS para
    /// garantizar que el 100% del piso es una sola region conectada.
    pub fn brain_level() -> Self {
        let raw: Vec<&str> = vec![
            "11111111111111111111111111",
            "10000000000000000000000001",
            "10000000000000000000000001",
            "10022220003333000444400001",
            "10022220003333000444400001",
            "10022220003333000444400001",
            "10000000000000000000000001",
            "10000000000000000000000001",
            "10000000000000000000000001",
            "10000555500006666022220001",
            "10000555500006666022220001",
            "10000555500006666022220001",
            "10000000000000000000000001",
            "10000000033330044440000001",
            "10000000033330044440000001",
            "10000000000000000000000001",
            "10000000000000000000000001",
            "11111111111111111111111111",
        ];

        let height = raw.len();
        let width = raw[0].len();
        let mut cells = vec![0u8; width * height];

        for (y, row) in raw.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let v = ch.to_digit(10).unwrap_or(0) as u8;
                cells[y * width + x] = v;
            }
        }

        Map { width, height, cells }
    }

    /// Devuelve el tipo de celda (0 = vacio, 1..=6 = tipo de pared) en (x, y).
    /// Fuera de rango se trata como pared (evita que rayos/jugador "se salgan" del mundo).
    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return 1;
        }
        self.cells[y as usize * self.width + x as usize]
    }

    /// True si la celda que contiene el punto de mundo (wx, wy) es pared solida.
    pub fn is_wall_at(&self, wx: f32, wy: f32) -> bool {
        let cx = (wx / TILE_SIZE).floor() as i32;
        let cy = (wy / TILE_SIZE).floor() as i32;
        self.get(cx, cy) != 0
    }

    /// Punto de partida del jugador. Verificado a mano y por BFS (ver
    /// script de diseño del mapa) que cae en piso libre y con margen del
    /// borde, para que jamas empiece atascado dentro de una pared.
    pub fn spawn_point(&self) -> (f32, f32) {
        (3.5, 2.5)
    }

    /// Punto final del nivel: cuando el jugador se acerca lo suficiente a
    /// esta coordenada, se considera el nivel completado (ver main.rs).
    /// Tambien verificado por BFS como alcanzable desde el spawn.
    pub fn goal_point(&self) -> (f32, f32) {
        (22.5, 15.5)
    }

    /// Posicion sugerida para la diana/objetivo de práctica de disparo,
    /// ubicada en un area abierta central, visible desde varios angulos.
    pub fn target_point(&self) -> (f32, f32) {
        (13.5, 8.5)
    }
}
