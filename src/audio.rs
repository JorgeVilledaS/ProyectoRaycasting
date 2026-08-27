// audio.rs
// Carga y reproduce en loop un archivo .wav externo como musica de fondo.
// archivo en `assets/background_music.wav`.

use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Ruta donde el juego busca la musica de fondo.
const MUSIC_PATH: &str = "assets/background_music.wav";

/// Mantiene vivos el stream y el sink de audio 
pub struct AudioSystem {
    _stream: OutputStream,
    sink: Sink,
}

impl AudioSystem {
    /// Intenta abrir el dispositivo de audio y cargar `assets/background_music.wav`
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

    /// Pausa/reanuda la musica 
    pub fn set_playing(&self, playing: bool) {
        if playing {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }
}
