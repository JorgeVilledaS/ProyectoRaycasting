// audio.rs
// Carga y reproduce en loop un archivo .wav externo como musica de fondo.
// El archivo NO se genera por codigo: el usuario debe colocar su propio
// archivo en `assets/background_music.wav` (ver README).

use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Ruta donde el juego busca la musica de fondo. Se puede reemplazar por
/// cualquier archivo .wav propio; basta con sobrescribir este archivo.
const MUSIC_PATH: &str = "assets/background_music.wav";

/// Mantiene vivos el stream y el sink de audio mientras dura la partida
/// (si se dropean, el sonido se corta). Es Option a nivel de main porque
/// el juego debe poder seguir funcionando aunque no haya audio disponible
/// (sin tarjeta de sonido, o sin el archivo .wav puesto).
pub struct AudioSystem {
    _stream: OutputStream,
    sink: Sink,
}

impl AudioSystem {
    /// Intenta abrir el dispositivo de audio y cargar `assets/background_music.wav`
    /// en loop infinito. Si algo falla (no hay tarjeta de sonido, o no existe el
    /// archivo), se imprime un aviso en consola y se devuelve `None`: el juego
    /// continua sin musica en vez de crashear.
    pub fn start() -> Option<Self> {
        let (stream, handle) = match OutputStream::try_default() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[audio] No se encontro dispositivo de sonido ({e}); el juego continua sin musica.");
                return None;
            }
        };

        let sink = match Sink::try_new(&handle) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[audio] No se pudo crear el reproductor de audio ({e}).");
                return None;
            }
        };

        if !Path::new(MUSIC_PATH).exists() {
            eprintln!(
                "[audio] No se encontro '{MUSIC_PATH}'. Coloca ahi tu archivo .wav para tener musica de fondo (ver README). El juego continua sin musica."
            );
            return None;
        }

        let file = match File::open(MUSIC_PATH) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[audio] No se pudo abrir '{MUSIC_PATH}': {e}");
                return None;
            }
        };

        let source = match Decoder::new(BufReader::new(file)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[audio] No se pudo decodificar '{MUSIC_PATH}' (¿es un .wav valido?): {e}");
                return None;
            }
        };

        // `repeat_infinite` hace que la pista vuelva a sonar en loop sin cortes.
        sink.append(source.repeat_infinite());
        sink.set_volume(0.5);
        Some(AudioSystem { _stream: stream, sink })
    }

    /// Pausa/reanuda la musica (por ejemplo al entrar/salir de la pantalla de bienvenida).
    pub fn set_playing(&self, playing: bool) {
        if playing {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }
}
