# Spikes de stack — Fase 1.1

Código **desechable** de medición. No es producto ni sigue las reglas de producción
(se permite `unwrap`/`expect` en arneses). Cada spike escribe JSON crudo en `results/`.

Los números y su interpretación están en `docs/MEDICIONES.md`; la decisión, en
`docs/DECISIONES.md` (ADR-001).

## Condiciones de medición (obligatorias)

Sin foco, WebKitGTK suspende `requestAnimationFrame` y el watchdog aborta con
`raf_stalled`. Por eso todos los lanzamientos usan:

```bash
GDK_BACKEND=x11 ...                     # evitar el camino Wayland nativo
xdotool windowactivate --sync <WID>     # dar foco antes de medir
```

`xdotool` no está instalado en el sistema; se usó el binario de
`/tmp/opencode/xdotool-3.20160805.1/xdotool` con `LD_LIBRARY_PATH` apuntando a
`/tmp/opencode` (que contiene `libxdo.so.3`).

## Spike A — Dioxus 0.7.10 desktop

```bash
cd spike-a-dioxus
# Esta máquina no tiene libxdo.so.3 de sistema (dioxus-desktop → muda la enlaza).
# Se compiló desde fuente en /tmp/opencode; crear los symlinks locales una vez:
mkdir -p .libs
ln -sf /tmp/opencode/libxdo.so.3 .libs/libxdo.so.3
ln -sf libxdo.so.3 .libs/libxdo.so
RUSTFLAGS="-L native=$PWD/.libs" cargo build --release
./run_measure.sh
```

- `run_measure.sh`: lanza la app, activa la ventana y espera el reporte.
- Modos: `SPIKE_STAGE=calib|capture|ipc`, `SPIKE_QUICK=1`, `SPIKE_REPS=<n>`.
- Audio: cpal 0.18.2 con features `realtime,pipewire`, buffer `Fixed(256)` y default.
- Salida: `results/spike-a-<stamp>.json` + líneas `SPIKE_A_*` en el log.

## Spike B — Tauri 2.12.5 + Leptos 0.8.23 (CSR/wasm)

El frontend se compila a wasm y se sirve desde `frontend/dist` (configurado en
`tauri.conf.json` como `frontendDist`). El host Rust sólo implementa `ping`, `report`
y `fail`.

```bash
# 1) frontend
cd spike-b-tauri-leptos/frontend
RUSTFLAGS="--sysroot=/tmp/opencode/sysroot" \
  cargo build --release --target wasm32-unknown-unknown --lib
/tmp/opencode/wb/wasm-bindgen-0.2.130-x86_64-unknown-linux-musl/wasm-bindgen \
  target/wasm32-unknown-unknown/release/spike_b_frontend.wasm \
  --target web --out-dir dist --out-name spike_b_frontend

# 2) host
cd ..
RUSTFLAGS="-L native=$PWD/.libs" cargo build --release
./run_measure.sh
```

- Salida: `results/spike-b-<stamp>.json`.
- `frontend/dist/` se commitea (200 KB) con el wasm/js exactos que se midieron, para poder
  reproducir la corrida sin montar el toolchain wasm32. Si se cambia el frontend, se
  regenera con los comandos de arriba.
- El toolchain wasm32 se resolvió **sin sudo**: sysroot local en `/tmp/opencode/sysroot`
  (`rust-wasm-1:1.99.1-2` desde archive.archlinux.org) y wasm-bindgen CLI 0.2.130.
  El paquete `rust-wasm` del mirror actual es `-4` y da `E0514` (incompatible con rustc
  1.99.1); no usarlo.

## Spike C — dioxus-native 0.8.0-alpha.3

```bash
cd spike-c-dioxus-native
cargo build --release
# En esta máquina winit elige Wayland y xdotool no ve la ventana; para inspeccionarla:
WAYLAND_DISPLAY= WINIT_UNIX_BACKEND=x11 cargo run --release
```

Intento mínimo (ventana + texto). Resultado: compiló, corrió y dibujó; captura de la
ventana con `import -window <WID>`. La cadencia no se mide (no hay `rAF` de plataforma).
Detalle en `docs/MEDICIONES.md`.

## Resultados crudos

`results/` contiene los JSON tal como los escribió cada arnés. No se editan a mano; si se
reprocesan, se hace en un script aparte y se documenta.
