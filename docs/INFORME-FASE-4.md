# INFORME — Fase 4: host de escritorio Tauri y audio nativo

Fecha: 2026-11-14 UTC. Estado: **cerrada técnicamente y publicada en GitHub Pages**; sin
release de bundles. Alcance: P4-001 … P4-007 de `docs/CHECKLIST.md`.

## Alcance entregado

| Tarea | Qué se hizo |
|---|---|
| P4-001 | `crates/desktop`: binario Tauri 2.12.5 + tauri-build 2.7.4 que embebe el mismo `dist/` de Leptos y abre la ventana real; workspace y regla de dependencias ampliados; `ci.sh` excluye el host con aviso cuando faltan `dist/` o webkit2gtk. |
| P4-002 | Audio nativo cpal 0.18 en `BufferSize::Fixed(256)`, 48 kHz estéreo; hilo dedicado dueño del `Stream`; cola SPSC `rtrb`; clic precalculado por `engine::audio`; callback sin locks ni asignaciones. |
| P4-003 | Reloj de muestras monótono, histograma fijo de jitter (32 bins de 50 µs), contadores de callbacks/frames/clics/cola llena/bloques tardíos/errores; comandos `audio_start`, `audio_click`, `audio_stats`, `audio_clock_ms`; telemetría visible en la UI. |
| P4-004 | `store.json` en el directorio de datos; snapshot inyectado como `window.__ANZAN_STORE__`; escritura atómica tmp+rename; comandos `store_get/set/remove`; `localStorage` sigue siendo el respaldo web. |
| P4-005 | Iconos RGBA, script `scripts/package-desktop.sh` y bundles Linux: AppImage 110,27 MiB y `.deb` 3,24 MiB; `docs/EMPAQUETADO.md` con contenido, hashes y firma documentada. |
| P4-006 | Modo `--measure` con el arnés del spike B dentro de la ventana real; 5 corridas × 1000 estímulos de 100 ms + 300 de 200 ms; crudos en `measurements/`; comparación en `docs/MEDICIONES.md`. |
| P4-007 | Rebuild y reverificación de bundles con el código final, `./ci.sh` verde, este informe y actualización de RESUME/LOG/CHECKLIST. |

## Verificación real

- `./ci.sh` → **CI OK**: fmt, construcciones prohibidas, clippy `-D warnings`, tests,
  regla de dependencias, `cargo deny` y check wasm32.
- **111 tests** verdes: 45 core + 1 presupuesto + 58 engine + 2 content + 3 ui + 2 desktop.
- **Ventana real:** abierta en X11 (`GDK_BACKEND=x11`), capturada con ImageMagick; la UI de
  Fase 3 se ve completa a 960×820 CSS y dpr 2.
- **Audio nativo:** dispositivo por defecto, 48 000 Hz, 2 canales, 256 frames; 3/3 clics
  por drill (15/15 en la medición de P4-006), 0 colas llenas y 0 errores; jitter p95
  **50 µs** en todas las corridas, máximos de 101–5179 µs y **2 bloques tardíos** de
  151 204 callbacks (0,0013 %).
- **Persistencia:** tres ejecuciones reales acumularon 1, 2 y 3 sesiones en
  `~/.local/share/dev.anzanlab.desktop/store.json`, con progreso y repaso; los bundles
  también leen y escriben ese archivo.
- **Empaquetado:** AppImage ejecutado con la UI real y `.deb` extraído con `ar`+`tar` (sin
  instalar); ambos pasan `--ui-selftest` (`ok: true`, frame p95 17,0 ms, aparición p95
  0,0 ms, 3/3 clics, 0 bloques tardíos). `desktop-file-validate` acepta el `.desktop`.
- **Medición (P4-006):** 5000 estímulos de 100 ms y 1500 de 200 ms. Frame real
  16,093–16,100 ms; media 96,557–96,599 ms (−3,4 %); error p50/p95/máx **3/4/4 ms**; 100 %
  dentro de ±1 frame; **0 gaps**. En 200 ms, media 193,08–193,23 ms y error p95 7 ms.
  Telemetría propia de la UI: frame p95 17,0 ms y aparición p95 0,0 ms en 5/5.
- **Comparación con el spike B:** el mismo binario del spike se re-ejecutó en la misma
  sesión. A 62,1 Hz dio 16,108 ms/96,650 ms (idéntico a la app); a 60,0 Hz dio
  16,643 ms/99,857 ms. El régimen de cadencia es del entorno y afecta por igual a ambos.

## Diseño y decisiones locales

- **El host no reimplementa la UI.** Tauri embebe `dist/`, el mismo wasm de la web. La
  lógica nueva vive en Rust (`audio`, `store`, `selftest`, `measure`) y la UI la consume
  por `invoke`; la web sigue funcionando sin Tauri.
- **Un solo dueño del `Stream`.** cpal no garantiza `Send` del stream en todas las
  plataformas; el hilo de audio lo crea, lo reproduce y lo libera. La UI sólo empuja
  unidades a una cola SPSC; el callback sólo lee y escribe el buffer.
- **Jitter en bins de 50 µs.** El histograma de 32 bins da una cota superior del p95 (50 µs
  cuando casi todos los bloques caen en el primer bin) sin ordenar muestras en el callback.
- **Persistencia por snapshot.** El store se inyecta antes de cargar la página para que la
  UI lea de forma síncrona; las escrituras son atómicas. El formato es el mismo que
  `localStorage`, lo que permite conservar ambas rutas sin migración.
- **Medición con el arnés del spike B.** No se inventó un protocolo nuevo: mismas capas
  preasignadas, mismos 6 y 12 frames, mismo cálculo de error. Así la comparación con B-1/B-2
  es directa y la diferencia de cadencia se puede atribuir al entorno con datos pareados.
- **Sin firma.** No hay claves; los bundles no están firmados y se documenta como
  **NO VERIFICADO**. La integridad disponible es el SHA-256.

## Límites y riesgos

- **La cadencia no está garantizada.** La app planifica en frames (ADR-002); si la ventana
  presenta a 62,1 Hz, 6 frames son 96,6 ms y no 100 ms. Ocurrió en 5/5 corridas de la
  medición, igual que en el spike B re-ejecutado en ese momento. La causa última no se
  determinó; los controles descartan carga de UI, audio y tamaño de ventana. Un plan por
  tiempo o un anclaje explícito a vsync sería trabajo de otra fase.
- **El clic nativo no se escuchó.** La verificación es por contadores/reloj; latencia
  acústica, volumen y dispositivos alternativos quedan **NO VERIFICADO**.
- **Empaquetado de una sola plataforma.** Linux amd64, X11/XWayland. Windows, macOS,
  Android y Wayland nativo no se probaron.
- **Instalación real no probada.** No se ejecutó `dpkg -i` (requiere root) ni un arranque
  sin FUSE; sólo el AppImage con FUSE y el binario extraído.
- **CI en GitHub sin host.** Si el runner no tiene `dist/` o webkit2gtk, `ci.sh` omite
  `anzan-desktop` con un AVISO. En ese entorno el host queda **NO VERIFICADO**; aquí sí se
  compiló y ejecutó.
- **Dependencias del escritorio.** `cargo deny` ignora `RUSTSEC-2024-0370`
  (`proc-macro-error` unmaintained, transitivo de GTK, sin CVE) y mantiene el ignore
  preexistente de `paste`.

## NO VERIFICADO

- Audio acústico (DAC → altavoz → oído), latencia audible y clic audible.
- Scanout físico y persistencia de retina; la medición usa callbacks de `rAF`.
- Causa del régimen de 60,0 vs 62,1 Hz en esta sesión de XWayland.
- Firma de los bundles, updater y `latest.json` (no existen claves ni pipeline de release).
- Instalación con `dpkg -i`/apt, arranque sin FUSE y otras distribuciones.
- Windows, macOS, Android, Wayland nativo y otros dispositivos de audio.
- Publicación: los commits de Fase 4 están pusheados y GitHub Pages sirve el build de la
  fase (ver «Publicación web»). Los bundles AppImage/`.deb` **no** se publicaron como
  release y siguen sin firma.

## Publicación web (GitHub Pages, 2026-11-14)

- Push de `c57dde4..d550894` a `main`; workflows **CI** (`43547444902`) y **Pages**
  (`43547444914`) completados con éxito (53 s y 1 min 36 s).
- El sitio publicado sirve `service-worker.js` con `anzanlab-v8`, el estado nuevo
  («Fase 4 completa · escritorio con audio nativo y medición real» / «Phase 4 complete…»)
  y un wasm construido por el runner de **1 230 769 B** (≈263,8 KB gzip), que contiene las
  cadenas de la fase.
- Verificado con Chrome headless contra la URL real en `es` y `?lang=en`: la UI monta
  (210 nodos/atributos `anzan-*`) y muestra el estado de Fase 4; sin errores de consola.
- El wasm del runner **no coincide byte a byte** con el build local (`56fa7bbe…`,
  1 228 666 B): la compilación no es reproducible byte a byte (toolchain/entorno). El
  contenido funcional sí coincide (mismas cadenas y UI).
- **Alcance de la publicación:** sólo el sitio web/PWA. No hay GitHub Release, tag ni
  subida de los bundles AppImage/`.deb`; siguen sin firma.

## Estado del repositorio

- `main` local y `origin/main`: `d550894` (publicación web de Fase 4); `main` local puede ir
  por delante con los commits de documentación posteriores.
- Commits de la fase: `546d133` (P4-001), `753ed23` (P4-002/003), `d79856d` (P4-004),
  `50c8e9f` (docs), `2aef0f3` (P4-005), `04b5058` (docs), `2c404d3` (P4-006),
  `55c0579` (docs), `e5d901a`/`f35ef03` (cierre) y `d550894` (web).
- `./ci.sh` verde en esta máquina; `dist/` y bundles en `target/` (ignorados por git).

## Compuerta de Fase 5

La Fase 4 queda cerrada técnicamente y su build web está publicado en Pages (autorizado el
2026-11-14). **La Fase 5 no se inicia sin autorización explícita.** Por decisión del usuario
(ADR-007), la fase **no incluye app Android ni háptico**: se limita a accesibilidad
avanzada, fotosensibilidad, web móvil digna y presupuesto de rendimiento. Plan y tareas en
`docs/PLAN-FASE-5.md` y en la sección FASE 5 de `docs/CHECKLIST.md`.

Queda pendiente decidir:

1. ¿Se autoriza **iniciar la Fase 5** con el alcance reducido?
2. ¿Se autoriza en el futuro una **release** con los bundles AppImage/`.deb`? Hoy no hay
   firma ni claves; no se subió ningún bundle.
