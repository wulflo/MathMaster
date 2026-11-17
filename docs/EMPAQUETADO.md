# Empaquetado de escritorio — Linux

Cómo se genera el host Tauri de MathMaster y qué se verificó realmente. Las cifras de
rendimiento viven en `docs/MEDICIONES.md`; aquí sólo hay empaquetado y distribución.

## Requisitos

| Componente | Versión usada | Nota |
|---|---|---|
| tauri-cli (`cargo-tauri`) | 2.12.4 | `cargo install tauri-cli --version "^2"` |
| tauri / tauri-build | 2.12.5 / 2.7.4 | fijadas en `crates/desktop/Cargo.toml` |
| webkit2gtk-4.1 | 2.54.2 | `pkg-config --exists webkit2gtk-4.1` |
| GTK 3 | 3.24.52 | dependencia de Tauri |
| ALSA | 1.2.16.1 | audio nativo cpal |
| wasm32-unknown-unknown + wasm-bindgen 0.2.130 | — | los usa `scripts/build-web.sh` |

No hace falta `dpkg-deb` ni `patchelf`: el bundler de Tauri arma el `.deb` y el AppImage
por su cuenta (descarga `linuxdeploy` y el runtime de AppImage la primera vez).

## Generación

```sh
./scripts/package-desktop.sh            # AppImage + deb
./scripts/package-desktop.sh appimage   # sólo uno
```

El script reconstruye `dist/` con la UI Leptos real, compila el binario release y deja:

- `target/release/bundle/appimage/AnzanLab_0.1.0_amd64.AppImage`
- `target/release/bundle/deb/AnzanLab_0.1.0_amd64.deb`

El binario suelto queda en `target/release/anzan-desktop`.

## Artefactos verificados (2026-11-14 UTC, build de cierre de Fase 4)

| Artefacto | Tamaño | SHA-256 |
|---|---|---|
| `AnzanLab_0.1.0_amd64.AppImage` | 115 628 536 B (110,27 MiB) | `3afa149798ff2ff7d7a80edba00f9756754ac40c83b0cfbee0226cfdbbd4706b` |
| `AnzanLab_0.1.0_amd64.deb` | 3 394 442 B (3,24 MiB) | `374eab9f4a3e8bf04db24f247f7f058b969d94bdbcfcdabc60139c5dd8b17442` |

Los hashes identifican **esta** compilación concreta. Reempaquetar cambia los bytes
(metadatos de tiempo y parcheo del binario), así que no son reproducibles byte a byte;
sí lo es el contenido funcional. Los artefactos de P4-005 (`b1b80d79…` / `5d7bf885…`)
quedaron obsoletos al añadir el modo de medición y se reemplazaron por estos, generados
con el código final de la fase.

El `.deb` (formato `ar` + `control.tar.gz` + `data.tar.gz`) declara:

- `Package: anzan-lab`, `Version: 0.1.0`, `Architecture: amd64`.
- `Depends: libwebkit2gtk-4.1-0, libgtk-3-0`.
- Instala `usr/bin/anzan-desktop`, iconos hicolor 32/128/256 y el `.desktop`.
- `desktop-file-validate` acepta el `.desktop` generado (categoría `Education`).

## Verificación realizada

Sobre los artefactos finales, en la sesión X11 local con audio PulseAudio/ALSA por
defecto:

1. **AppImage ejecutado directamente** (runtime tipo 2 con FUSE disponible):
   `--audio-selftest` abre el dispositivo y reproduce 8/8 clics; `--ui-selftest` arranca
   la UI empaquetada, completa un drill y reporta:
   `Frame p95 17,0 ms · Aparición p95 0,0 ms`, `Audio nativo · 48000 Hz · 256 bloques ·
   Jitter p95 50 µs`, 3/3 clics, 0 bloques tardíos, `ok: true`.
2. **Ventana real del AppImage** capturada con ImageMagick: se ve la UI de Fase 3
   completa (soroban, progresión, sesiones) a 960×820 CSS y `dpr = 2`.
3. **`.deb` extraído** con `ar` + `tar` (sin instalar en el sistema): el binario
   `usr/bin/anzan-desktop` pasa `--audio-selftest` y `--ui-selftest` con los mismos
   valores; en una de las corridas del `.deb` apareció 1 bloque tardío (el AppImage, 0).
4. La persistencia sigue funcionando dentro del bundle: el drill de la autoprueba
   acumuló una sesión más en `~/.local/share/dev.anzanlab.desktop/store.json`.

Los dos bundles se regeneraron con el código final de la fase y se volvieron a ejecutar:
AppImage y binario extraído del `.deb` pasan `--ui-selftest` (`ok: true`, frame p95
17,0 ms, aparición p95 0,0 ms, 3/3 clics, jitter p95 50 µs, 0 bloques tardíos).

## Firma — **NO VERIFICADA**

No hay claves de firma en el proyecto ni en esta máquina. En consecuencia:

- El AppImage y el `.deb` **no están firmados** y no hay firma GPG separada.
- No hay updater ni artefacto `latest.json`, así que no aplica la firma de Tauri.
- La verificación de integridad disponible hoy es el SHA-256 de la tabla anterior.
- Para una release real habrá que crear una clave (por ejemplo GPG para el `.deb` y
  `minisign`/GPG para el AppImage), publicar la huella y adjuntar las firmas; hasta
  entonces cualquier afirmación de "release firmada" es falsa.

## Límites

- **NO VERIFICADO**: instalación con `dpkg -i`/apt en el sistema (requiere root; aquí no
  se instaló nada). Sólo se ejecutó el binario extraído.
- **NO VERIFICADO**: arranque sin FUSE (`--appimage-extract` sí se usó para inspeccionar,
  pero la ejecución probada fue la del AppImage con FUSE).
- **NO VERIFICADO**: otras distribuciones, Wayland nativo, Windows y macOS.
- **NO VERIFICADO**: audio acústico; la comprobación es por contadores y reloj del
  callback (ver `docs/MEDICIONES.md`).
- El tamaño del AppImage (110 MiB) incluye WebKitGTK y GTK empaquetados por linuxdeploy;
  no es un binario liviano y no se optimizó.
