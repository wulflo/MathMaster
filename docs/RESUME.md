# RESUME — MathMaster

Documento de reanudación. Se actualiza **al cerrar cada fase** y **ante cada tarea marcada
`[!]`** en `docs/CHECKLIST.md`.

## Qué es el proyecto

**MathMaster** es un entrenador progresivo de cálculo mental con **soroban virtual**, inspirado
en el *Flash Anzan*. Escritorio primero, después Android; también se publica como web estática
(PWA offline) en GitHub Pages.

- **100 % Rust**: UI Leptos/wasm, host Tauri, core/engine nativos.
- **Offline** por diseño: service worker en la web; el escritorio no necesita red.
- **Precisión temporal como prioridad 1**: planes en frames, telemetría real, sin maquillar.
- **Accesibilidad y seguridad fotosensible**: claro/oscuro, contraste reducido, sin destellos
  de pantalla completa.
- **Español e inglés**, sin strings hardcodeados.
- **Copyleft**: código AGPL-3.0-or-later; contenido CC BY-SA 4.0; sonidos y diccionarios
  originales CC0 1.0.

Stack decidido en STOP-1 (ADR-001): **Tauri 2.12.5 + Leptos 0.8.23 (CSR/wasm)**.
Estado de tareas y siguiente paso: `docs/CHECKLIST.md`.

## Comandos exactos

Todos los comandos se ejecutan desde la raíz del repositorio
(`/home/alonso/Documentos/Repositorios/MathMaster`).

### 1. Instalar requisitos

El repo fija el toolchain en `rust-toolchain.toml` (Rust **1.99.1**, componentes `rustfmt` y
`clippy`, target `wasm32-unknown-unknown`). Con `rustup`:

```bash
rustup toolchain install 1.99.1 -c rustfmt -c clippy -t wasm32-unknown-unknown
```

Dependencias del sistema (Arch Linux; webview + audio):

```bash
sudo pacman -S --needed base-devel webkit2gtk-4.1 gtk3 alsa-lib
```

Para construir la web hace falta además **wasm-bindgen CLI 0.2.130**:

```bash
cargo install wasm-bindgen-cli --version 0.2.130 --locked
```

En esta máquina no hay `rustup` ni target wasm32 de sistema; `./ci.sh` y
`./scripts/build-web.sh` recurren al sysroot local `/tmp/opencode/sysroot` y a
`/tmp/opencode/wb/wasm-bindgen-0.2.130-x86_64-unknown-linux-musl/wasm-bindgen` si existen.
No instalar el paquete `rust-wasm` del mirror actual (`-4`): da `E0514` con rustc 1.99.1.
Detalle en `spikes/README.md`.

### 2. Compilar

```bash
cargo build --workspace            # debug (host)
cargo build --workspace --release  # release (host)
```

Sitio web estático (wasm + wasm-bindgen + assets → `dist/`):

```bash
./scripts/build-web.sh
```

Host de escritorio (Tauri, embebe el mismo `dist/`):

```bash
./scripts/build-web.sh && cargo build -p anzan-desktop          # debug
./scripts/build-web.sh && cargo build -p anzan-desktop --release
```

Para ejecutarlo en esta máquina (XWayland): `GDK_BACKEND=x11 target/release/anzan-desktop`.
Modos de prueba: `--audio-selftest` (8 clics), `--ui-selftest` (drill completo en la UI
real, informa `ANZAN_SELFTEST`) y `--measure` (arnés del spike B dentro de la ventana;
`ANZAN_MEASURE_OUT`, `ANZAN_MEASURE_REPS100/200`, `ANZAN_MEASURE_HIDE_UI`,
`ANZAN_MEASURE_NO_AUDIO`). El host necesita `webkit2gtk-4.1` y `alsa-lib`.

### 3. CI local

```bash
./ci.sh
```

Ejecuta, en orden: `cargo fmt --all -- --check`, `scripts/check-forbidden.sh`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`scripts/check-deps.sh`, `cargo deny check` y `cargo check` de `anzan-ui`/`anzan-app` para
`wasm32-unknown-unknown` (si el target está disponible).

### 4. Tests

```bash
cargo test --workspace             # toda la suite
cargo test -p anzan-core           # un crate
cargo test --workspace -- --nocapture
```

`cargo test` debe quedar verde y sin `unwrap`/`expect`/`panic` fuera de tests (lo vigila
`scripts/check-forbidden.sh`).

### 5. Benchmarks de timing (spikes A/B)

Los benchmarks de timing son los arneses desechables de `spikes/`. Escriben JSON crudo en
`spikes/results/`; los números interpretados están en `docs/MEDICIONES.md`.

**Spike B — Tauri 2.12.5 + Leptos 0.8.23 (stack elegido):**

```bash
cd spikes/spike-b-tauri-leptos/frontend
RUSTFLAGS="--sysroot=/tmp/opencode/sysroot" \
  cargo build --release --target wasm32-unknown-unknown --lib
/tmp/opencode/wb/wasm-bindgen-0.2.130-x86_64-unknown-linux-musl/wasm-bindgen \
  target/wasm32-unknown-unknown/release/spike_b_frontend.wasm \
  --target web --out-dir dist --out-name spike_b_frontend
cd ..
RUSTFLAGS="-L native=$PWD/.libs" cargo build --release
./run_measure.sh
```

**Spike A — Dioxus 0.7.10 webview:**

```bash
cd spikes/spike-a-dioxus
mkdir -p .libs
ln -sf /tmp/opencode/libxdo.so.3 .libs/libxdo.so.3
ln -sf libxdo.so.3 .libs/libxdo.so
RUSTFLAGS="-L native=$PWD/.libs" cargo build --release
./run_measure.sh
```

**Spike C — `dioxus-native` 0.8.0-alpha.3:**

```bash
cd spikes/spike-c-dioxus-native
cargo build --release
WAYLAND_DISPLAY= WINIT_UNIX_BACKEND=x11 cargo run --release
```

Condiciones obligatorias de medición: `GDK_BACKEND=x11` y activar la ventana con `xdotool`
antes de medir; sin foco, WebKit suspende `requestAnimationFrame` y el watchdog aborta con
`raf_stalled`. `xdotool` se usa desde `/tmp/opencode/xdotool-3.20160805.1/` con
`LD_LIBRARY_PATH=/tmp/opencode`. Salida: `spikes/results/spike-a-<stamp>.json` y
`spikes/results/spike-b-<stamp>.json`.

### 6. Empaquetado Linux

```bash
./scripts/package-desktop.sh            # AppImage + .deb
./scripts/package-desktop.sh appimage   # sólo uno
```

Requiere `cargo-tauri` (tauri-cli 2.12.4). Deja los bundles en
`target/release/bundle/{appimage,deb}/`. Detalle, hashes y límites (firma **NO VERIFICADA**)
en `docs/EMPAQUETADO.md`.

### 7. Medición de la app real

```bash
RUNS=5 ./scripts/measure-desktop.sh     # SKIP_BUILD=1 para no recompilar
```

Ejecuta `--measure` cinco veces, activa la ventana con `xdotool` (si está) y deja crudos en
`measurements/`. Resultados en `docs/MEDICIONES.md`.

## Cómo leer el checklist

`docs/CHECKLIST.md` es la **fuente única de verdad**. Cada tarea tiene ID, criterio de
aceptación verificable y estado:

- `[ ]` pendiente · `[~]` en curso · `[x]` hecha (hash de commit) · `[!]` bloqueada ·
  `[-]` descartada.
- `[NET]` = requiere red. `[HW]` = requiere hardware real (audio/háptico/pantalla), no
  verificable en CI.
- El bloque **PUNTO DE REANUDACIÓN** al inicio indica la fase y tarea activas, el bloqueo
  vigente, el stack/licencia decididos, la última tarea cerrada y el remoto.
- Ante una tarea `[!]`, registrar el bloqueo en `docs/LOG.md` y actualizar este archivo y el
  punto de reanudación del checklist.
- Al cerrar una fase: actualizar el informe de fase, `docs/LOG.md`, `docs/CHECKLIST.md` y
  este `docs/RESUME.md`.

## Prompt de reanudación (copiado tal cual)

```text
Estás retomando el proyecto "MathMaster" (Rust), que quedó interrumpido por un corte de sesión o de internet. No tenés el contexto previo: la fuente de verdad está en el repositorio. NO empieces a programar todavía.

PASO 1: Orientación (solo lectura)
1. Leé docs/SUPERPROMPT.md completo: son tus instrucciones originales y siguen vigentes. Si no existe, avisame y esperá que te lo pase.
2. Leé docs/RESUME.md, el bloque PUNTO DE REANUDACIÓN de docs/CHECKLIST.md, las últimas 50 líneas de docs/LOG.md y docs/DECISIONES.md (en especial ADR-001, el stack elegido).
3. Ejecutá `git status`, `git log --oneline -10` y `git stash list`.
4. Corré `./ci.sh` (fmt, clippy -D warnings, tests) y anotá el primer fallo. Si no hay red, usá solo las dependencias ya descargadas (`--offline`).

PASO 2: Diagnóstico (informámelo antes de tocar nada)
Presentame en pocas líneas:
- Fase actual y progreso (tareas [x] / total de la fase).
- Tarea en curso [~] o bloqueada [!], o la primera pendiente [ ].
- Si hay cambios sin commitear o en el stash, qué contienen y a qué tarea pertenecen.
- Resultado de ./ci.sh y, si algo está roto, la causa probable.
- Si la fase anterior quedó cerrada pero SIN mi autorización para la siguiente, decímelo y esperá mi respuesta.

PASO 3: Reanudación (solo después de mi OK al diagnóstico)
- Cambios sin commitear de la sesión cortada: no los descartes a ciegas. Revisalos con `git diff`; si son coherentes y los tests pasan, completá la tarea y commiteá; si son parciales o rotos, guardalos con `git stash push -m "WIP <ID>"` o descartalos con criterio, y anotalo en docs/LOG.md.
- Si una tarea figura [x] pero su verificación hoy falla, marcala [!], registrá el error literal en LOG.md y arreglala antes de seguir.
- Reanudá desde la tarea [~]/[!] o la primera [ ], aplicando el protocolo de continuidad de <rules>: [~] antes de empezar, commit atómico con el ID, [x] con el hash al terminar y entrada en LOG.md.
- Sin internet, seguí solo con tareas sin marca [NET] y dejá las demás [ ] con la nota "pendiente por falta de red".
- No rehagas tareas ya marcadas [x] y verificadas. No cambies decisiones de DECISIONES.md sin consultarme.

REGLAS QUE SIGUEN VIGENTES
- Compuertas entre fases: al cerrar una, entregá el INFORME DE FASE y esperá mi "sí" explícito antes de empezar la siguiente.
- Prohibido unwrap/expect/panic en código de producción; cero asignaciones y locks en el hot path de render y de audio; tests de propiedades para la lógica del ábaco.
- Lo que no puedas verificar en tu entorno (audio real, háptico, móviles) se declara NO VERIFICADO con pasos para que yo lo pruebe. Nunca lo marques como hecho.
- Sin push, tags ni releases remotos sin mi autorización.
```

## Estado vigente al reanudar

- **Fase 4 cerrada técnicamente y publicada en Pages; sin release de bundles:**
  P4-001…P4-007 `[x]`. Informe final en `docs/INFORME-FASE-4.md`; empaquetado en
  `docs/EMPAQUETADO.md`; mediciones en `docs/MEDICIONES.md`. El push `c57dde4..d550894`
  disparó CI y Pages, ambos verdes; el sitio sirve el build de Fase 4 con service worker
  `anzanlab-v8`. Los bundles AppImage/`.deb` no se publicaron.
- **Fase 4 — qué existe:** host Tauri 2.12.5 que embebe el mismo `dist/` web; audio cpal
  (`Fixed(256)`, 48 kHz, 2 canales) con hilo dueño del `Stream`, cola SPSC y callback sin
  locks/asignaciones; reloj de muestras y jitter en histograma fijo; persistencia atómica
  en `~/.local/share/dev.anzanlab.desktop/store.json`; bundles AppImage y `.deb`.
- **Fase 4 — números:** `CI OK` con **111 tests**; ventana real a dpr 2; 3/3 clics por
  drill (15/15 en la medición), jitter p95 50 µs, 2 bloques tardíos de 151 204 callbacks;
  medición de 5000 estímulos de 100 ms + 1500 de 200 ms con frame 16,093–16,100 ms,
  media 96,557–96,599 ms (−3,4 %), error p95 4 ms, 100 % dentro de ±1 frame y 0 gaps.
  El spike B re-ejecutado en la misma sesión dio lo mismo (16,108 ms/96,650 ms); el régimen
  de 60,0 vs 62,1 Hz es del entorno y su causa queda **NO VERIFICADA**.
- **Fase 4 — NO VERIFICADO:** audio acústico (DAC/altavoz/oído), firma de bundles, updater,
  instalación con `dpkg -i`, arranque sin FUSE, Windows/macOS/Android/Wayland nativo y el
  host en el CI de GitHub (se omite si falta `dist/` o webkit2gtk).
- **Fase 3 cerrada técnicamente y publicada:** P3-001…P3-009 `[x]`. Informe final en
  `docs/INFORME-FASE-3.md`; detalle de la 3a en `docs/INFORME-FASE-3A.md`.
- **Fase 2 cerrada y publicada:** P2-001…P2-010 `[x]`. Informe en
  `docs/INFORME-FASE-2.md`.
- **Fase 1 cerrada y publicada:** P1-001…P1-027 `[x]`; informe en
  `docs/INFORME-FASE-1.md`.
- **Web publicada:** <https://wulflo.github.io/MathMaster/> sirve la Fase 4 (service worker
  `anzanlab-v8`) tras el push `d550894`; verificado en `es` y `?lang=en` con Chrome headless.
  wasm del runner 1 230 769 B (≈263,8 KB gzip), dentro del presupuesto de 2 MB.
- **Medición web de Fase 2:** frame p95 16,8 ms; aparición p95 0,0 ms (1 corrida, 180
  frames, Chromium local). Audio acústico, otros navegadores y móviles: **NO VERIFICADO**.
- **Verificación Fase 3:** 109 tests, 240.000 casos de propiedades, wasm 1.187.902 B
  (259.425 B gzip), `CI OK`; en navegador: sesiones, CSV, estadísticas por caso, repaso
  «En 24 h»/«En 1 h» y persistencia tras recarga.
- **FASE M cerrada técnicamente (2026-11-16):** M-001 `[x]` (`c833b04`) motor;
  M-002 `[x]` (`418533c`) contenido M0–M3; M-003 `[x]` (`00c9361`) sección «Aprender»;
  M-004 `[x]` (`cd1d36a`) práctica M1–M7; M-005 `[x]` (`bcc8574`) M4–M7, mental y atajos;
  M-006 `[x]` (`71bb3fe`) persistencia con claves `anzan.mul.*`; M-007 `[x]` (`5b5f157`)
  accesibilidad, i18n y rendimiento. **135 tests**, **280.000** casos de propiedades,
  `CI OK`; wasm local **1.708.695 B** (**347.160 B gzip**). Informe con números, diseño,
  límites y lo **NO VERIFICADO** en `docs/INFORME-FASE-M.md`.
  **Publicada:** `aef0ca6` en `origin/main`; Pages sirve SW **`anzanlab-v9`** y wasm
  1.707.369 B (build de CI). Verificado en la web publicada: kicker de multiplicación, sin
  «Fase 4», tres acentos y 12 lecciones.
- **Rediseño visual de la multiplicación:** `6770eed` (ábaco dibujado, tabla 大九九, tabla
  de productos parciales, tarjetas, dos columnas y medidor).
- **Paleta y nombre:** `5f2383d` renombra la app a **MathMaster** y define diez acentos con
  hues distintos.
- **Acento único, tema y limpieza:** `7a88b96` deja **un solo acento** elegible entre diez
  muestras; la elección cambia cabecera, botones, tarjetas, tablas, ábaco, medidor y fondo
  a la vez y se guarda en `anzan.accent`. Añade un interruptor claro/oscuro junto a las
  muestras (`anzan.theme`) y elimina toda la telemetría visible. Service worker
  `mathmaster-v11`; wasm local **1.850.383 B** (**373.012 B gzip**), 138 tests y `CI OK`;
  verificado en Chrome es/en. Los tres commits están **sólo en local**, pendientes de
  autorización para publicarlos.
  **Siguiente paso:** compuerta de la Fase 5 reducida por ADR-007 (accesibilidad avanzada,
  fotosensibilidad, web móvil y rendimiento; **sin Android ni háptico**), plan en
  `docs/PLAN-FASE-5.md`. **No iniciar la Fase 5 sin el «sí» explícito.** Publicar el
  rediseño o una release de bundles sigue sin autorización.

## Superprompt

`docs/SUPERPROMPT.md` existe y es de **solo lectura**. Se reconstruyó el 2026-11-13 desde
README, ADR-000…006, MEDICIONES, DISEÑO, CHECKLIST y el código, con las secciones
`<role>`, `<context>`, `<architecture>`, `<step_by_step_plan>` y `<rules>`. No se modifica
salvo pedido explícito del usuario.
