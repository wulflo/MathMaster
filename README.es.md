# MathMaster

[![CI](https://github.com/wulflo/MathMaster/actions/workflows/ci.yml/badge.svg)](https://github.com/wulflo/MathMaster/actions/workflows/ci.yml)
[![GitHub Pages](https://github.com/wulflo/MathMaster/actions/workflows/pages.yml/badge.svg)](https://wulflo.github.io/MathMaster/)
[![Licencia: AGPL-3.0-or-later](https://img.shields.io/badge/licencia-AGPL--3.0--or--later-blue.svg)](LICENSE)
[![Rust 1.99+](https://img.shields.io/badge/rust-1.99%2B-orange.svg)](https://www.rust-lang.org/)

[English](README.md) · **Español**

Entrenador progresivo de cálculo mental con **soroban virtual**, inspirado en el *Flash Anzan*. Escritorio primero; también disponible como web estática y offline.

**▶ Web: <https://wulflo.github.io/MathMaster/>** · **▶ Código: <https://github.com/wulflo/MathMaster>**

> El rediseño de pestañas horizontales descrito más abajo está construido y verificado localmente; la build publicada en GitHub Pages puede seguir mostrando la versión de una sola columna hasta que se republica.

## Tabla de contenidos

- [¿Por qué es diferente?](#por-qué-es-diferente)
- [Mapa de la aplicación](#mapa-de-la-aplicación)
- [Arquitectura](#arquitectura)
- [Inicio rápido](#inicio-rápido)
- [Documentación](#documentación)
- [Licencias](#licencias)

## ¿Por qué es diferente?

- **Pedagogía primero, velocidad después.** Seis capítulos estructurados cubren anatomía del soroban, digitación ortodoxa, amigos pequeños (complementos a 5), amigos grandes (complementos a 10), familias mixtas y el desvanecimiento progresivo hacia el cálculo mental Anzan. Las lecciones de multiplicación M0–M7 y los atajos S1–S4 incluyen la base matemática, posicionamiento de varas *Teii-hō*, procedimiento detallado, errores comunes y un ejemplo anotado.
- **Temporización por frames.** Los estímulos se planifican en frames, nunca en milisegundos aproximados. Los controles de exposición y separación muestran su equivalente en frames y los términos/segundo resultantes en tiempo real.
- **Reconocimiento inmediato de la respuesta.** Cada modo de práctica enfoca el campo de respuesta y comprueba el valor en cada pulsación. Un valor correcto se acepta al instante y, con el avance automático activado por defecto, el siguiente ejercicio comienza de inmediato. Un error revela el valor esperado y ofrece un recorrido paso a paso en el soroban con un clic.
- **Un campo de entrenamiento completo.** Flash Anzan (sólo sumas o sumas/restas mezcladas, con cuenta atrás de fijación 3·2·1), columnas estilo examen Mitori Anzan, multiplicación por niveles, ejercicios dirigidos a debilidades, repaso espaciado, diagnóstico por regla y exportación CSV.
- **Material Aero.** Tokens Material 3, superficies translúcidas Aero, diez acentos seleccionables, temas claro/oscuro, soporte para movimiento reducido y transparencia reducida, y sin destellos de pantalla completa.

## Mapa de la aplicación

| Pestaña | Subsecciones |
|---------|--------------|
| **Learn** | Fundamentos del soroban · Multiplicación frontal (空盘前乘法) · Atajos mentales · Tabla 大九九 |
| **Free soroban** | Soroban interactivo (4–13 varas, cinco etapas de desvanecimiento) · Demostrador de movimiento |
| **Train** (por defecto) | Flash Anzan · Mitori Anzan · Multiplicación por niveles · Entrenamiento dirigido |
| **Progress** | Panel de dominio · Repaso espaciado · Diagnóstico por regla e historial de sesiones |

La pestaña activa por defecto es **Train → Flash Anzan**, lo que mantiene los arneses de automatización de la UI (`--ui-selftest` y `--measure`) funcionando sin navegación extra.

Teclado: `←`/`→`/`Home`/`End` se mueven entre pestañas, `0`–`9` y `Backspace` editan la respuesta, `Enter` confirma o continúa, y `Space` inicia o repite una secuencia.

## Arquitectura

| Crate | Rol |
|-------|-----|
| `anzan-core` | Estado del soroban, varas y ranuras válidas, `plan_add`/`plan_sub`, desvanecimiento y geometría. Sin dependencias internas; `forbid(unsafe_code)`. |
| `anzan-engine` | Planes por frame y telemetría, ejercicios deterministas (Flash, mixtos, dirigidos, Mitori, multiplicación), conversión de velocidad, progresión, sesiones, repasos y clasificación pedagógica. |
| `anzan-content` | Todo el texto visible y todo el material pedagógico en diccionarios RON validados con paridad exacta de claves es/en; `forbid(unsafe_code)`. |
| `anzan-ui` | Componentes Leptos: pestañas, vistas de aprendizaje, laboratorio, entrenamiento, campo de respuesta y paneles de progreso. |
| `anzan-app` | Bootstrap web: detección de idioma, `<html lang>` y montaje WASM. |
| `anzan-desktop` | Host Tauri 2: audio nativo con cpal, persistencia en disco y arneses de automatización. |

La dirección de dependencias se impone con `scripts/check-deps.sh`:

`ui → engine → core`, `content → core`, `app → ui/engine/content/core`, `desktop → engine/content`.

Ninguna cadena visible vive en Rust: la copia de interfaz y el contenido del curso se cargan desde `crates/content/content/i18n/{es,en}.ron` y `crates/content/content/multiplication/{es,en}.ron`, y los cargadores rechazan claves desconocidas.

## Inicio rápido

```bash
# Toolchain: Rust 1.99.x con el objetivo wasm32-unknown-unknown
cargo test --workspace
./ci.sh                    # fmt, construcciones prohibidas, clippy -D warnings, tests,
                           # reglas de dependencias, cargo deny y comprobación wasm32
./scripts/build-web.sh     # build estático en dist/
cargo tauri dev            # host de escritorio (necesita webkit2gtk y dist/)
cargo tauri build --bundles appimage,deb
```

Los modos de automatización de escritorio son `cargo tauri dev -- --ui-selftest` y `cargo tauri dev -- --measure` (véase `scripts/measure-desktop.sh` para una ejecución completa).

## Documentación

- [`docs/CHECKLIST-REDESENO.md`](docs/CHECKLIST-REDESENO.md) — plan maestro de este rediseño.
- [`docs/CHECKLIST.md`](docs/CHECKLIST.md) — checklist histórico por fases y punto de reanudación.
- [`docs/LOG.md`](docs/LOG.md) — bitácora append-only.
- [`docs/DISENO.md`](docs/DISENO.md) — sistema de diseño Material Aero.
- [`docs/METODO-MULTIPLICACION.md`](docs/METODO-MULTIPLICACION.md) — método de multiplicación.
- [`docs/DECISIONES.md`](docs/DECISIONES.md) — registros de decisiones de arquitectura.
- [`docs/MEDICIONES.md`](docs/MEDICIONES.md) — mediciones reales de tiempo y límites.
- [`docs/EMPAQUETADO.md`](docs/EMPAQUETADO.md) — bundles para Linux.

## Licencias

### Este proyecto

- Código: [AGPL-3.0-or-later](LICENSE).
- Contenido pedagógico: [CC BY-SA 4.0](LICENSE-CONTENT).
- Sonidos y diccionarios originales: CC0 1.0 (ver [`CREDITS.md`](CREDITS.md)).

### Dependencias de Rust

Las versiones están fijadas en `Cargo.toml`/`Cargo.lock`; `cargo deny check` verifica el grafo completo.

| Crate | Versión | Licencia |
|-------|---------|----------|
| serde | 1.0.229 | MIT OR Apache-2.0 |
| ron | 0.12.3 | MIT OR Apache-2.0 |
| thiserror | 2.0.21 | MIT OR Apache-2.0 |
| leptos | 0.8.23 | MIT |
| wasm-bindgen | 0.2.130 | MIT OR Apache-2.0 |
| wasm-bindgen-futures | 0.4.80 | MIT OR Apache-2.0 |
| js-sys / web-sys | 0.3.107 | MIT OR Apache-2.0 |
| console_error_panic_hook | 0.1.7 | Apache-2.0 OR MIT |
| proptest (dev) | 1.12.0 | MIT OR Apache-2.0 |
| tauri / tauri-build | 2.12.5 / 2.7.4 | Apache-2.0 OR MIT |
| cpal | 0.18.2 | Apache-2.0 |
| rtrb | 0.4.1 | MIT OR Apache-2.0 |
| serde_json | 1.0.150 | MIT OR Apache-2.0 |

### Herramientas y componentes de la plataforma

| Componente | Versión usada | Licencia |
|------------|---------------|----------|
| Toolchain de Rust (rustc, cargo) | 1.99.1 | MIT OR Apache-2.0 |
| wasm-bindgen CLI | 0.2.130 | MIT OR Apache-2.0 |
| Tauri CLI | 2.12.4 | Apache-2.0 OR MIT |
| cargo-deny | 0.22.0 | MIT OR Apache-2.0 |
| WebKit2GTK | 4.1 (sistema) | LGPL-2.1-or-later |
| GTK | 3.x (sistema) | LGPL-2.1-or-later |
| ALSA (backend de audio de escritorio) | sistema | LGPL-2.1-or-later |
| Web Audio API | implementación del navegador | Especificación W3C libre de regalías |

Las dependencias transitivas, incluidas sus licencias, se listan con `cargo deny list`; `deny.toml` documenta el pequeño conjunto de crates transitivos sin mantenimiento que actualmente se ignoran con una razón.

---

Hecho con ❤️ y Rust · [wulflo/MathMaster](https://github.com/wulflo/MathMaster)
