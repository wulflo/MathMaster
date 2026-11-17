# MEDICIONES — MathMaster

Toda afirmación de rendimiento del proyecto lleva aquí su número, su método y su fecha.
Los datos crudos viven en `spikes/results/` (spike) o `measurements/` (producto).

## Plantilla

```
### [fecha] [máquina] [escenario]
- Método:
- N:
- Resultado: p50 / p95 / máx, % dentro de tolerancia
- Entorno: SO, compositor, tasa de refresco medida, build (debug/release)
- Limitaciones / NO VERIFICADO:
```

---

## Spike de stack — Fase 1.1 (2026-11-13)

Objetivo: decidir el shell de UI (ADR-001). La prioridad es la precisión temporal del
estímulo visual; la unidad de planificación es el frame (ADR-002). Todos los arneses
piden **1000 estímulos de 100 ms** y **300 de 200 ms**, sin blanco entre estímulos
(cambio directo entre dos capas preasignadas), y registran la duración observada de cada
estímulo en el callback de `requestAnimationFrame`.

### Entorno común

- Máquina: desktop, GNOME/Wayland. Sesión forzada a X11 en XWayland (`GDK_BACKEND=x11`)
  porque WebKitGTK suspende `requestAnimationFrame` en ventanas sin foco.
- Ventana activada con `xdotool windowactivate --sync` antes de medir.
- Pantalla primaria: HDMI-1 AOC 1600x900 @ 60.000 Hz; eDP-1 1920x1080 @ 60.000 Hz, escala 1.25.
- Compilación `--release`; rustc 1.99.1; webkit2gtk 4.1 (2.54.2); GTK 3.24.52.
- Audio: PipeWire/ALSA 48 kHz, sink analógico ALC285.
- Resolución de `performance.now()` medida en la webview: **1 ms** (ambos spikes).
- La cadencia real se calcula como `wall_ms / (total_frames - 1)`, no con el `span_ms` de
  calibración: la calibración de 240 frames puede caer en un tramo transitorio y mentir
  (ver B corrida 1, donde calibró 16.10 ms y luego presentó a 16.66 ms).

### Spike A — Dioxus 0.7.10 desktop (webview wry/WebKitGTK)

Arnés: `spikes/spike-a-dioxus/`. Plan en frames; presentador JS de dos capas `#d0/#d1`;
telemetría por estímulo. Audio y reloj en el proceso Rust.

| Corrida | Calibración span | Frame real | 100 ms: media | Drift total | Err abs p50/p95/máx | Dentro ±1 frame | Gaps >1.5 frame |
|---|---|---|---|---|---|---|---|
| A-1 `1794593162` | 16.1004 ms | 16.1012 ms | 96.607 ms | −3393 ms (−3.39 %) | — (agregado con bug de unidades) | — | 0 |
| A-2 `1794595336` | 16.0879 ms | 16.6385 ms | 99.831 ms | −169 ms (−0.17 %) | 0 / 3 / 33 ms | 99.8 % | 2 |
| A-3 `1794595602` | 16.0879 ms | 16.0997 ms | 96.598 ms | −3402 ms (−3.40 %) | 3 / 4 / 11 ms | 100 % | 0 |
| A-4 `1794596499` | 16.1046 ms | 16.0928 ms | 96.557 ms | −3443 ms (−3.44 %) | 3 / 4 / 10 ms | 100 % | 0 |

- **Lectura:** en 3 de 4 corridas el `rAF` de WebKitGTK corre a ~62.1 Hz (16.09–16.10 ms),
  no a los 60 Hz del panel. Con 6 frames por estímulo eso entrega 96.56 ms cuando se piden
  100 ms: el error es de cadencia, no de jitter (100 % dentro de ±1 frame *respecto de la
  cadencia medida*). En A-2 la cadencia real fue 60.1 Hz y el resultado se acercó a 100 ms,
  pero con 2 gaps de hasta 33 ms. La cadencia de A **no está clavada al vsync**.
- **200 ms (A-4):** media 198.66 ms; drift −1338 ms (−2.23 %); err p50/p95 7/7 ms, máx 18 ms;
  100 % dentro de ±1 frame; 0 gaps. Frame real en esa fase: 16.295 ms.
- A-1 se conserva como dato crudo, pero sus agregados de error usaron unidades
  inconsistentes (`planned_total_ms = 1000000`); sólo se usa su `wall_ms`.

### Spike B — Tauri 2.12.5 + Leptos 0.8.23 CSR (wasm)

Arnés: `spikes/spike-b-tauri-leptos/`. Mismo presentador JS y mismo plan; UI Leptos
compilada a wasm; host Tauri en Rust; IPC `invoke` para ping y para entregar el reporte.

| Corrida | Calibración span | Frame real | 100 ms: media | Drift total | Err abs p50/p95/máx | Dentro ±1 frame | Gaps >1.5 frame |
|---|---|---|---|---|---|---|---|
| B-1 `1794596250` | 16.1004 ms | 16.6605 ms | 99.963 ms | −37 ms (−0.037 %) | 0 / 1 / 4 ms | 100 % | 0 |
| B-2 `1794596754` | 16.6192 ms | 16.6668 ms | 100.001 ms | +1 ms (+0.001 %) | 0 / 1 / 1 ms | 100 % | 0 |

- **Lectura:** en ambas corridas la presentación quedó clavada a ~16.66 ms (60.0 Hz),
  aunque la calibración previa mintiera en B-1. El plan de 6 frames entrega 100 ms.
- **200 ms:** B-1 media 200.007 ms, drift +2 ms, err máx 1 ms; B-2 media 200.003 ms,
  drift +1 ms, err máx 1 ms. Frame real 16.667 ms. Cero gaps en ambas.
- **Estabilidad:** 2/2 corridas a 60.0 Hz, contra 1/4 en A. La diferencia observada es
  consistente con que Tauri/wry (0.57.2) presente en vsync y Dioxus/wry (0.53.5) use un
  temporizador interno de ~16.1 ms; la causa exacta a nivel de compositor **NO VERIFICADA**
  (no se pudo capturar el scanout físico).

### Comparación directa

| Criterio | A · Dioxus 0.7.10 | B · Tauri 2.12.5 + Leptos 0.8.23 |
|---|---|---|
| Cadencia de presentación | 62.1 Hz en 3/4 corridas; 60.1 Hz en 1/4 | 60.0 Hz en 2/2 corridas (vsync) |
| 100 ms × 1000 (error de duración) | −3.4 % (96.56 ms) o −0.17 % (99.83 ms) | −0.037 % / +0.001 % (99.96–100.00 ms) |
| Error por estímulo p50/p95/máx | 3 / 4 / 10–11 ms | 0 / 1 / 1–4 ms |
| Plan en frames vs realidad | Dentro de ±1 frame, pero el frame no es el del panel | Dentro de ±1 frame y el frame es el del panel |
| IPC JS→Rust por frame | **0/600** mensajes intermedios (`document::eval`) | **p50 1 ms, p95 1 ms, máx 4–7 ms** (`invoke`, 600 pings a 60 Hz) |
| Supervisión frame a frame desde Rust | No viable con el canal actual | Viable con `invoke` a 60 Hz |
| Audio en proceso Rust | cpal directo (medido) | Mismo host Rust; no se repitió la sonda en B |
| Toolchain | Cargo nativo, sin wasm | Requiere target wasm32 + wasm-bindgen |
| Dependencia | dioxus 0.7.10 estable | tauri 2.12.5 + leptos 0.8.23 estables |
| Madurez del renderer | wry/webkit2gtk estable | wry/webkit2gtk estable |

### Latencia de audio (sólo Spike A, cpal 0.18.2 sobre PipeWire)

Latencia **software**: trigger → callback que consume el primer sample. No incluye DAC,
amplificador ni altavoz. `playback_offset` es el offset declarado por el backend.

| Buffer | p50 | p95 | máx | `playback_offset` | Callbacks tardíos | Gap máx callback |
|---|---|---|---|---|---|---|
| `Fixed(256)` | 2.595 ms | 2.644 ms | 2.700 ms | 10.667 ms | 0 / 2803 | 5.893 ms |
| Default | 7.920 ms | 7.985 ms | 10.348 ms | 85.333 ms | 366 / 367 | 42.943 ms |

- `BufferSize::Fixed(256)` sí se respeta en el host PipeWire. El buffer por defecto da un
  offset de 85 ms, incompatible con feedback rítmico.
- **NO VERIFICADO:** latencia acústica real (DAC → altavoz → oído), underruns reales del
  dispositivo (no hay contador de xruns; `misses` sólo cuenta fallos de la sonda) y
  comportamiento en Android.

### Spike C — dioxus-native 0.8.0-alpha.3

Arnés mínimo en `spikes/spike-c-dioxus-native/`: ventana + `<h1>` + `<p>`, sin lógica.

- **Compila:** sí. `cargo build --release` limpio en 2 min 57 s (primer build, sin caché).
  `target/` ocupa 1.9 GB; binario de 41 MB.
- **Corre:** sí. Permaneció 12 s sin crash ni error (sólo un warning de
  `xdg_toplevel_icon_manager_v1`). Ventana X11 de 800x600 titulada “Dioxus App”.
- **Renderiza:** sí. Captura de la ventana con `import -window <WID>`: 800x600, media RGB
  246.7, desviación 43.8, 256 colores; se lee el texto “AnzanLab · spike C” y el subtítulo.
  La captura de la raíz X11 (`ffmpeg x11grab`) sale negra y la API de GNOME Shell deniega
  la captura (`Screenshot is not allowed`); la ventana X11 sí se pudo capturar.
- **Fricción:** por defecto crea ventana Wayland (invisible para `xdotool`); para inspección
  hubo que forzar `WAYLAND_DISPLAY= WINIT_UNIX_BACKEND=x11`. Reportó `scale factor: 2`, por
  lo que el texto se ve al doble de tamaño; el manejo de escalado queda por verificar.
- **Timing:** **NO MEDIDO**. `dioxus-native` no expone un `requestAnimationFrame` de
  plataforma; medir la cadencia exigiría escribir un bucle de animación con winit/WGPU y
  separar presentación de composición. Al ser una alpha, no se invirtió ese esfuerzo en
  Fase 1.1. Por tanto C no compite en la comparación temporal: sólo se acredita que
  compila, corre y dibuja.

### Limitaciones transversales / NO VERIFICADO

- **Scanout físico:** no se pudo usar captura de pantalla como ground truth (la raíz X11
  sale negra por la ventana XWayland fuera del origen). Todo lo anterior mide el timestamp
  de `rAF` y la duración entre callbacks, no el instante en que el píxel llega al panel.
  Verificación pendiente con cámara lenta en Fase 2.
- **Fotometría/contraste real del panel** y comportamiento del compositor con la ventana
  a pantalla completa: no medido.
- **Audio real** (DAC/altavoz/auriculares), háptico y móviles: no medidos.
- **Carga del sistema:** las corridas se hicieron con el escritorio en uso; no se aisló CPU
  ni se fijó afinidad. Los gaps de A-2 (33 ms) son probablemente contención del sistema.
- Una sola máquina y una sola GPU; no se generaliza a otros equipos.

---

## Producto — Flash Anzan web (Fase 2, 2026-11-13)

Primera medición del modo Flash Anzan en la cápsula web. No sustituye a las mediciones de
Tauri: la web tiene su propio `rAF` y su propio audio.

### [2026-11-13] [desktop Linux, Chromium/Electron del entorno] [5 términos × 36 frames]

- **Método:** `engine::schedule::FrameTelemetry` dentro del callback de
  `requestAnimationFrame`. Registra el delta entre callbacks y, al cambiar de estímulo, el
  error absoluto entre el tiempo transcurrido desde el primer callback y
  `frame_planificado × cadencia media medida`. Plan: 30 frames de estímulo + 6 de hueco.
- **N:** 1 corrida, 180 frames, 5 apariciones de estímulo.
- **Resultado:** frame p95 **16,8 ms**; error de aparición p95 **0,0 ms**. Sin errores de
  consola.
- **Entorno:** servidor local `127.0.0.1:8768`; build wasm release (877.327 B, 203.248 B
  gzip); `requestAnimationFrame` del navegador; sin ventana a pantalla completa.
- **Limitaciones / NO VERIFICADO:** una sola corrida y una sola máquina; no mide scanout
  físico ni composición. Firefox, Safari, Android y móviles no se probaron. El audio web se
  crea sin errores, pero **no se verificó** que suene ni su latencia acústica. La pestaña en
  segundo plano suspende `rAF` y no se midió una pausa explícita.

---

## Producto — app de escritorio real (Fase 4, 2026-11-14 UTC)

Medición de la app Tauri completa (ventana real, UI Leptos, audio cpal, persistencia) con
**el mismo arnés y el mismo plan que el spike B**, para que la comparación sea directa.
Datos crudos: `measurements/desktop-20261114T013306Z-*.json`.

### [2026-11-14] [AMD Ryzen 5 4600H · GNOME/Wayland sobre XWayland] [ventana 960×820, dpr 2]

- **Método:** `target/release/anzan-desktop --measure` (commit de P4-006). El script
  (`scripts/measure-desktop.sh`) completa primero un **drill real** (audio nativo,
  progreso, sesión persistida), espera foco (`xdotool windowactivate --sync`), calibra
  **240 frames** y después presenta **1000 estímulos de 100 ms** y **300 de 200 ms** sobre
  dos capas preasignadas, alternando `visibility` sin crear nodos ni cambiar layout. Mide
  la duración observada por estímulo con `requestAnimationFrame`, igual que el spike B.
  Cinco corridas consecutivas; una sola sesión de escritorio en uso, sin aislar CPU.
- **N:** 5 corridas × (1000 + 300) = **5000 estímulos de 100 ms** y **1500 de 200 ms**.
- **Build:** release; binario `d64d57fd…`, wasm `bc4222ba…`; tauri 2.12.5, wry 0.57.2,
  tao 0.37.2, webkit2gtk 2.54.2, GTK 3.24.52; rustc 1.99.1.
- **Pantalla:** HDMI-1 3200×1800 declarada a 59,96 Hz por `xrandr`; resolución efectiva de
  `performance.now()` = 1 ms.

| Corrida | Frame real | 100 ms: media | Drift | Err p50/p95/máx | Dentro ±1 frame | Gaps | 200 ms: media | Err p95/máx | Jitter p95/máx | Bloques tardíos |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 16,100 ms | 96,599 ms | −3,401 % | 3 / 4 / 4 ms | 100 % | 0 | 193,230 ms | 7 / 8 ms | 50 / 5149 µs | 1 |
| 2 | 16,093 ms | 96,557 ms | −3,443 % | 3 / 4 / 4 ms | 100 % | 0 | 193,083 ms | 7 / 8 ms | 50 / 151 µs | 0 |
| 3 | 16,099 ms | 96,597 ms | −3,403 % | 3 / 4 / 4 ms | 100 % | 0 | 193,083 ms | 7 / 8 ms | 50 / 158 µs | 0 |
| 4 | 16,099 ms | 96,595 ms | −3,405 % | 3 / 4 / 4 ms | 100 % | 0 | 193,207 ms | 7 / 7 ms | 50 / 5179 µs | 1 |
| 5 | 16,099 ms | 96,591 ms | −3,409 % | 3 / 4 / 4 ms | 100 % | 0 | 193,183 ms | 7 / 8 ms | 50 / 101 µs | 0 |

- Calibración: span 16,167–16,502 ms; mediana 16 ms; p95 18 ms; p99 19 ms; máx 24 ms.
- Telemetría propia de la UI en las 5 corridas: **frame p95 17,0 ms · aparición p95
  0,0 ms**; drill de 2187–2217 ms; 3/3 clics nativos por corrida.
- Audio agregado: 151 204 callbacks, 38 708 224 frames, **15/15 clics**, 0 colas llenas,
  **0 errores**, **2 bloques tardíos** (0,0013 %). El jitter p95 se mantuvo en 50 µs; dos
  corridas tuvieron un bloque aislado de ~5,1 ms (máximo 5149/5179 µs) que no movió el p95.

### Comparación con el spike B

El mismo día y en la misma sesión se volvió a ejecutar el binario del spike B (commit de
Fase 1, sin cambios) dos veces. La cadencia de la ventana cambió entre esas dos corridas,
y con ella el resultado, **en la app y en el spike por igual**:

| Escenario | Frame real | 100 ms: media | Err p95 | Gaps | Cuándo |
|---|---|---|---|---|---|
| App real, 5 corridas | 16,093–16,100 ms | 96,557–96,599 ms | 4 ms | 0 | 01:33–01:41 UTC |
| Spike B re-ejecutado (`1794621399`) | 16,108 ms | 96,650 ms | 4 ms | 0 | ~01:56 UTC |
| Spike B re-ejecutado (`1794618708`) | 16,643 ms | 99,857 ms | 1 ms | 0 | ~01:11 UTC |
| Spike B original B-1/B-2 (Fase 1) | 16,660–16,667 ms | 99,963–100,001 ms | 1 ms | 0 | 2026-11-13 |

- **Lectura:** cuando la ventana presenta a 60,0 Hz, 6 frames = 100 ms y el error p95 es
  1 ms (spike B original y re-ejecución de 01:11). Cuando la ventana presenta a 62,1 Hz,
  6 frames = 96,6 ms (−3,4 %) y el error p95 sube a 4 ms; la app real y el spike B dan
  **el mismo resultado dentro de 0,05 ms**. El comportamiento de la app coincide con el
  del spike B en la misma condición; el régimen de cadencia (60,0 vs 62,1 Hz) es del
  entorno, no del código de la app.
- **Controles:** con la UI oculta y fondo plano (16,095 ms), sin audio (16,099 ms) y con la
  ventana redimensionada/arriba (16,088 ms) la cadencia siguió en 62,1 Hz. No es carga de
  pintura, ni el stream de audio, ni el tamaño de la ventana. La causa última del régimen
  **NO VERIFICADA** (compositor/XWayland/scanout no instrumentados).

### Limitaciones / NO VERIFICADO

- Una sola máquina, una sola sesión y 5 corridas; no se generaliza. El régimen de cadencia
  puede cambiar entre corridas (se observó 60,0 y 62,1 Hz en la misma sesión).
- Se mide la duración entre callbacks de `rAF` y el cambio de `visibility`; **no** se
  verificó el scanout físico ni la persistencia de retina (haría falta cámara lenta).
- Audio: **NO VERIFICADO** acústicamente. No se comprobó que el clic suene, su volumen, ni
  la latencia DAC → altavoz → oído; sólo contadores, reloj de muestras y jitter del
  callback.
- `performance.now()` tiene resolución de 1 ms en esta webview; los valores sub-milisegundo
  son promedios, no resoluciones.
- La ventana se midió a dpr 2 (960×820 CSS = 1920×1640 físicos); a otros dpr o a pantalla
  completa no se midió.
- El escritorio estaba en uso; no se fijó afinidad de CPU ni se aislaron interrupciones.

---

## Producto — build web de Fase 4 (2026-11-14 UTC)

- **Método:** `./scripts/build-web.sh` (wasm32 release + wasm-bindgen 0.2.130) y descarga
  del artefacto publicado en Pages; tamaño con `stat` y `gzip -9`.
- **N:** 1 build local y 1 build del runner de GitHub Actions.
- **Resultado:** wasm local **1 228 666 B**; wasm del runner **1 230 769 B** (≈**263,8 KB**
  gzip, 13 % del presupuesto de 2 MB). `service-worker.js` v8. La UI monta en Chrome
  headless contra la URL publicada, en `es` y `en`.
- **Entorno:** CI con Rust 1.99.1, wasm32-unknown-unknown y wasm-bindgen 0.2.130.
- **Limitaciones / NO VERIFICADO:** los dos builds **no coinciden byte a byte** (contenido
  funcional equivalente); no se midió primera carga útil en 4G ni tiempo hasta interactivo,
  sólo tamaño. La carga offline real sigue **NO VERIFICADA**.

---

## Producto — build web con la sección de multiplicación (2026-11-14 UTC)

- **Método:** `./scripts/build-web.sh` (wasm32 release + wasm-bindgen 0.2.130) en local;
  tamaño con `stat` y `gzip -9`. Verificación funcional en Chrome (headless y con las
  herramientas de navegador) contra `dist/` servido en local.
- **N:** 1 build local. wasm **1 513 582 B** (**316 331 B** gzip), 15 % del presupuesto de
  2 MB; crece 284 916 B respecto del build local de Fase 4 (1 228 666 B) por el contenido
  de multiplicación y la sección nueva.
- **Verificación:** sección «Aprender a multiplicar» con 4 lecciones; pasos comprobados
  (2×4→08, 7×8→56, 28×3→84 en dos pasos, 7×34→238 en dos pasos), avance/retroceso con
  ratón y teclado (Enter conserva el foco) y textos es/en; sin errores de consola.
- **Limitaciones / NO VERIFICADO:** un solo navegador (Chromium) en escritorio; no se midió
  primera carga en 4G, móvil real ni lector de pantalla. La cache offline del service worker
  no se re-midió con esta build.

## Producto — práctica de multiplicación (2026-11-14 UTC)

- **Método:** `./scripts/build-web.sh` en local; tamaño con `stat` y `gzip -9`.
- **N:** 1 build local. wasm **1 666 369 B** (**343 336 B gzip**), 17 % del presupuesto de
  2 MB comprimido. El salto frente al build anterior (1 509 606 B / 321 482 B) se debe al
  generador de ejercicios (`MultiplicationDrill`) y a `js_sys::Date` para el tiempo.
- **Verificación en Chrome:** niveles M1–M7 con sus cifras; respuesta correcta (M1 2×9=18,
  M4 65×27=1755, M7 30×28=840) y incorrecta (M6 930×118 muestra «era 109740»); «Otro
  ejercicio» reinicia el campo y el feedback; aciertos/intentos y ms avanzan; Enter envía y
  conserva el foco en el campo de sólo lectura; inglés verificado; sin errores de consola.
- **Limitaciones / NO VERIFICADO:** un solo navegador (Chromium) en escritorio; móvil real,
  lector de pantalla y tiempos bajo carga **NO VERIFICADOS**.

## Producto — cierre de la Fase M (2026-11-16 UTC)

- **Método:** `./scripts/build-web.sh` en local; tamaño con `stat` y `gzip -9`. Pruebas en
  Chromium de escritorio contra `dist/` servido en 127.0.0.1:8790.
- **N:** 1 build local. wasm **1 708 695 B** (**347 160 B gzip**), 17 % del presupuesto de
  2 MB comprimido. Incluye el motor, las 12 lecciones es/en, el reproductor, la práctica y
  la persistencia de multiplicación.
- **Rendimiento de la sección:** reproductor con **1.000 transiciones en 24,8 ms**
  (≈ 0,025 ms por transición, 100 vueltas completas a los 9 pasos de M6 más reinicio).
  Práctica: **50 rondas completas en 1,675 s** incluyendo 150 esperas de
  `requestAnimationFrame`; 50/50 respuestas correctas y contador de 1/2 a 51/52.
- **Accesibilidad:** contraste del error ≈ 8,9:1 en tema oscuro (`#ff9d97`) y ≈ 6,5:1 en
  claro (`#a3221c`); árbol de accesibilidad con etiquetas «M0: …» separadas, `aria-live`
  en narración y pregunta, `role="status"` en feedback y foco conservado con Enter.
- **Limitaciones / NO VERIFICADO:** sólo Chromium de escritorio; móvil real, lector de
  pantalla, Firefox/Safari, perfil de CPU/memoria y batería **NO VERIFICADOS**. La eficacia
  pedagógica no se midió con estudiantes. La sección no está publicada.

## Producto — ajuste visual y build de publicación (2026-11-16 UTC)

- **Método:** `./scripts/build-web.sh` en local tras quitar el estado de fase y dar un
  acento por sección; tamaño con `stat` y `gzip -9`.
- **N:** 1 build local. wasm **1 625 355 B** (**343 400 B gzip**), 17 % del presupuesto de
  2 MB comprimido. El service worker del `dist/` es **`anzanlab-v9`**.
- **Acentos por sección** (fondo azul sin cambios): soroban teal `#0f766e` / `#5eead4`;
  multiplicación violeta `#6d28d9` / `#c4b5fd`; Flash Anzan ámbar `#b45309` / `#fbbf24`.
  Cada uno lleva su `--anzan-on-accent` para conservar el contraste del texto.
- **Verificación en Chrome es/en:** no queda el texto «Fase 4 completa»/«Phase 4», el
  kicker menciona la multiplicación, los tres acentos se aplican y no hay errores de
  consola.

## Producto — rediseño de la sección de multiplicación (2026-11-16 UTC)

- **Método:** `./scripts/build-web.sh` en local; tamaño con `stat` y `gzip -9`. Pruebas en
  Chromium de escritorio contra `dist/` servido en 127.0.0.1:8792.
- **N:** 1 build local. wasm **1 937 324 B** (**378 391 B gzip**), 19 % del presupuesto de
  2 MB comprimido. El crecimiento frente al cierre (347 160 B gzip) viene del ábaco, la
  tabla 大九九 de 81 celdas y las tablas de pasos.
- **Verificación:** M0 `2×4` resalta `08` y el paso 1 resalta `4×2`; M6 `213×324` dibuja
  seis varillas, nueve pasos y el acumulado `67900` tras el paso 4; la práctica M4 `65×27`
  responde «Correcto» con medidor al 100 %; 12 lecciones y 7 niveles en una fila; sin
  desbordamiento horizontal (897 px de documento en 912 px de ventana) y sin errores de
  consola. Inglés y tema claro revisados.
- **Publicación:** la fase M está en Pages desde `aef0ca6` (SW `anzanlab-v9`, wasm servido
  1 707 369 B). El rediseño `6770eed` **no está publicado**.
- **Limitaciones / NO VERIFICADO:** móvil real, lector de pantalla, Firefox/Safari, perfil de
  CPU/memoria y batería; el tema claro se simuló con tokens, no con emulación real.

## Producto — paleta de 10 acentos y renombrado (2026-11-16 UTC)

- **Método:** `./scripts/build-web.sh` en local; tamaño con `stat` y `gzip -9`. Pruebas en
  Chromium de escritorio contra `dist/` servido en 127.0.0.1:8793.
- **N:** 1 build local. wasm **1 937 324 B** (**378 391 B gzip**), 19 % del presupuesto de
  2 MB comprimido. Service worker **`mathmaster-v10`**; la caché local contiene sólo
  `mathmaster-v10` tras activarse.
- **Paleta:** diez acentos con hues distintos. En claro: `#1f5fd6`, `#0f766e`, `#6d28d9`,
  `#b45309`, `#be123c`, `#4d7c0f`, `#0e7490`, `#a21caf`, `#c2410c`, `#4338ca`. En oscuro se
  usan variantes claras y texto oscuro sobre acento.
- **Verificación:** título y cabecera **MathMaster** en es/en; tres secciones con acentos
  teal/violeta/ámbar; botones activos, tablas y medidor del mismo color que su sección;
  fondo con los diez acentos; contraste blanco sobre violeta **7,10:1** en los controles
  principales del tema claro; sin errores de consola.
- **Limitaciones / NO VERIFICADO:** móvil real, lector de pantalla, Firefox/Safari y perfil
  de CPU/memoria; el tema claro se simuló con los tokens, no con emulación real.

## Producto — acento único, tema claro/oscuro y sin telemetría (2026-11-18 UTC)

- **Método:** `./scripts/build-web.sh` en local; tamaño con `stat` y `gzip -9`. Pruebas en
  Chromium de escritorio contra `dist/` servido en 127.0.0.1:8793.
- **N:** 1 build local. wasm **1 850 383 B** (**373 012 B gzip**), 18 % del presupuesto de
  2 MB comprimido. Service worker **`mathmaster-v11`**.
- **Acento:** diez muestras en la cabecera; la elegida se aplica en `.anzan-root` y cambia
  a la vez cabecera, botones, tarjetas activas, tablas, ábaco, medidor y fondo. Fondo con
  dos resplandores del acento y mezcla del 16 % (claro) / 26 % (oscuro).
- **Tema:** interruptor claro/oscuro junto a las muestras, `role="switch"`, `data-theme` en
  `<html>` y persistencia en `anzan.theme`; el acento se guarda en `anzan.accent`. Ambos
  valores sobreviven a la recarga.
- **Contraste:** blanco sobre acento en tema claro entre **4,99:1** (oliva) y **7,90:1**
  (índigo); tinta oscura sobre acento en tema oscuro entre **8,63:1** y **14,33:1**.
- **Verificación:** es/en, diez muestras, toggle en la misma fila que las muestras, cambio
  global de acento y fondo, sin «Telemetría»/«Telemetry» en el texto visible, sin errores
  de consola. 138 tests y `CI OK`.
- **Limitaciones / NO VERIFICADO:** móvil real, lector de pantalla, Firefox/Safari, perfil
  de CPU/memoria y batería; el cambio de tema del sistema en caliente no se propaga hasta
  recargar (el valor guardado manda).
