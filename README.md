# Ray Caster Psicodélico

Proyecto de Gráficas por Computadora sobre Ray Caster simple en Rust, con temática
psicodélica/surrealista inspirada en corrientes lostmedia de internet, con texturas proceduralmente
animadas, un piso abierto tipo arena con bloques de colores flotando, disparo
hitscan contra una diana animada, y un punto final del nivel.

El "plotwist" o lo que hace interesante el nivel, es que al darle un tiro a la diana (Con forma de hongo) los controles se invierten, así como los colores de todo el mapa, para darle una sensación aún más extraña al nivel.

## Video de funcionamiento

> https://youtu.be/viJFVpklRt4

## Cómo correrlo

Requiere Rust y Cargo (`rustup` recomendado, o `apt install cargo rustc` en Ubuntu).

En Linux, `minifb` (la librería de ventana) necesita las librerías de desarrollo de
X11/Wayland y ALSA para el audio:

```bash
sudo apt-get install -y libxkbcommon-dev libwayland-dev libxcb1-dev libx11-dev \
    libxrandr-dev libxi-dev libgl1-mesa-dev pkg-config libasound2-dev
```

### Música de fondo

Lel juego carga un archivo `.wav` real que se debe colocar en

```
assets/background_music.wav
```

Luego:

```bash
cargo run --release
```

## Controles

| Tecla / acción         | Efecto                                             |
|------------------------|------------------------------------------------------|
| `W` / `↑`              | Avanzar                                             |
| `S` / `↓`              | Retroceder                                          |
| `A`                    | Strafe (moverse) a la izquierda                     |
| `D`                    | Strafe (moverse) a la derecha                       |
| `←` / `→`              | Girar la cámara (alternativa sin mouse)             |
| Mover el mouse          | Girar la cámara horizontalmente                     |
| Clic izquierdo / `F`   | Disparar (hitscan instantáneo, sin proyectil)       |
| `Enter` / `Espacio`    | Iniciar el juego desde la pantalla de bienvenida    |
| `R`                    | Reiniciar el nivel (en la pantalla de nivel completado) |
| `Esc`                  | Salir                                               |

## Objetivo del nivel

El mapa es una arena abierta con bloques de colores flotando (cada bloque es un
tipo de pared distinto, con su propia textura animada). En el centro hay una
diana psicodélica con forma de hongo que gira sobre sí misma. Al llegar a la
esquina inferior derecha del mapa se activa la pantalla de nivel completado,
que muestra cuántos impactos hiciste y te deja reiniciar con `R`.

El "plotwist" o lo que hace interesante el nivel, es que al darle un tiro a la diana (Con forma de hongo) los controles se invierten, así como los colores de todo el mapa, para darle una sensación aún más extraña al nivel.

El contador de FPS actual se muestra en la esquina superior izquierda durante
la partida.

## Estructura del proyecto

```
src/
  main.rs        Punto de entrada: ventana, game loop, máquina de estados, input, disparo
  map.rs         Grilla del nivel, colisión, spawn, punto de meta y posición de la diana
  player.rs       Posición, rotación y movimiento del jugador (con colisión)
  raycaster.rs    Algoritmo DDA: lanza un rayo por columna de pantalla
  renderer.rs     Pinta paredes/piso/techo y el sprite billboard de la diana (con z-buffer)
  sprite.rs      La diana: animación de rotación continua + flash al recibir un impacto
  textures.rs    Texturas 100% procedurales (funciones matemáticas), una por tipo de pared
  minimap.rs      Dibuja el minimapa en la esquina superior derecha
  audio.rs        Carga y reproduce en loop el archivo assets/background_music.wav
  font.rs         Mini fuente de píxeles 5x7, para dibujar texto real en pantalla
  screens.rs     Pantalla de bienvenida y pantalla de nivel completado
assets/
  background_music.wav   
```

## Objetivos del proyecto cubiertos

- [x] Ray caster simple en Rust, nivel completo y jugable, con un punto final
- [x] El jugador se puede desplazar libremente por todo el piso abierto del mapa
      (verificado por BFS: el 100% del piso es una sola región conectada)
- [x] El jugador no puede atravesar paredes (colisión por eje, con deslizamiento)
      ni hace crashear el programa (probado con estrés de input automatizado:
      chocar contra bloques, girar con el mouse y disparar simultáneamente)
- [x] 6 tipos de pared distintos, cada uno con su propia textura procedural animada
- [x] Rotación de cámara con el mouse (horizontal)
- [x] Disparo hitscan ("lo opuesto" al ray casting de render: un solo rayo desde
      el centro de la mira, sin proyectil) contra una diana de práctica
- [x] Animación de sprite: la diana rota continuamente y destella/crece al recibir
      un impacto
- [x] Minimapa en la esquina superior derecha (no al lado del mapa 3D)
- [x] Contador de FPS visible en pantalla
- [x] Música de fondo (cargada desde `assets/background_music.wav`, puesto por el usuario)
- [x] Pantalla de bienvenida con fondo psicodélico animado y texto real
- [x] Pantalla de nivel completado al llegar al punto final del mapa

## Notas de diseño

- **Sin texturas de imagen**: cada pared usa una función matemática (senos
  combinados tipo "plasma", ondas concéntricas, rejillas parpadeantes, etc.) que
  se anima con el tiempo.
- **Nivel**: piso abierto de 26x18 celdas con 8 bloques de colores flotando (6
  tipos de pared distintos). Se eligió un piso abierto en vez de pasillos
  angostos para garantizar que el jugador SIEMPRE tenga espacio para moverse y
  nunca quede atascado; la conectividad total se verificó programáticamente
  (BFS) antes de escribir el mapa en Rust.
- **Colisión**: se resuelve por eje (mover en X, revisar colisión; mover en Y,
  revisar colisión) y se muestrea un pequeño círculo alrededor del jugador, así
  nunca queda atascado ni atraviesa esquinas de pared.
- **Disparo**: es hitscan puro (sin física de proyectil). Se lanza un rayo desde
  el jugador en la dirección exacta de la cámara; si pasa lo bastante cerca del
  centro de la diana Y ninguna pared está más cerca en esa dirección, cuenta
  como impacto. Esto reutiliza el mismo algoritmo de intersección rayo-pared
  del renderer (`cast_ray`), por eso el disparo respeta la oclusión de paredes.
- **Sprite billboard**: la diana siempre mira de frente a la cámara sin importar
  el ángulo del jugador, y se dibuja usando un z-buffer para quedar correctamente oculta detrás de paredes
  más cercanas.
