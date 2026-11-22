# CHECKLIST — MathMaster

> **Rediseño integral (UX, pedagogía y entrenamiento):** el plan maestro con estado por
> subtarea está en [`CHECKLIST-REDESENO.md`](CHECKLIST-REDESENO.md). Léase primero al
> reanudar; este archivo conserva el histórico de fases 1–M y 5.

Fuente única de verdad del proyecto. Estados:
`[ ]` pendiente · `[~]` en curso · `[x]` hecha (hash de commit) · `[!]` bloqueada · `[-]` descartada
`[NET]` = requiere red. `[HW]` = requiere hardware real (audio/háptico/pantalla), no verificable en CI.

## PUNTO DE REANUDACIÓN

> **Rediseño integral (UX/pedagogía/entrenamiento):** completado el 100 % de sus 6 etapas
> (`R1-01`…`R6-03`); detalle y verificación en `docs/CHECKLIST-REDESENO.md` y `docs/LOG.md`.
> Cambios en local, sin publicar.
>
> **Fase 4 — estado:** cerrada técnicamente y **publicada en GitHub Pages** (P4-001…P4-007);
> sin release de bundles. Informe en `docs/INFORME-FASE-4.md`; empaquetado en
> `docs/EMPAQUETADO.md`; mediciones en `docs/MEDICIONES.md`. Host Tauri con audio cpal,
> persistencia y bundles AppImage/`.deb`; `CI OK` con 111 tests. Firma de bundles, audio
> acústico y otras plataformas: **NO VERIFICADO**.
> **Fase 3:** cerrada técnicamente y publicada (P3-001…P3-009). Informe final en
> `docs/INFORME-FASE-3.md`; detalle de la 3a en `docs/INFORME-FASE-3A.md`.
> **Fase 2:** cerrada y publicada (P2-001…P2-010); informe en `docs/INFORME-FASE-2.md`.
> **Fase 1:** cerrada y publicada (P1-001…P1-027); informe en `docs/INFORME-FASE-1.md`.
> **Superprompt:** `docs/SUPERPROMPT.md` reconstruido desde el repositorio y de solo lectura.
> **Web:** <https://wulflo.github.io/MathMaster/> sirve la **Fase M (multiplicación)**, con
> service worker `anzanlab-v9` activo y precacheado; carga offline real **NO VERIFICADA**.
> El rediseño visual `6770eed` está sólo en local.
> **Stack:** **B — Tauri 2.12.5 + Leptos 0.8.23**; licencia **AGPL-3.0-or-later**.
> **Última tarea cerrada:** M-007 (`5b5f157`) y cierre de la Fase M (`60640f2`).
> `origin/main` = `aef0ca6` (publicación web de la Fase M, SW `anzanlab-v9`); CI y Pages
> verdes. El rediseño visual `6770eed` está en local.
> **Remoto:** `origin` = <https://github.com/wulflo/MathMaster>.
> **Siguiente paso:** compuerta de la **Fase 5 con alcance reducido** (ADR-007):
> accesibilidad avanzada, fotosensibilidad, web móvil y rendimiento, **sin Android ni
> háptico**. Plan y tareas P5-001…P5-005 en `docs/PLAN-FASE-5.md`. La Fase M quedó cerrada
> y **publicada** (`aef0ca6`, SW `anzanlab-v9`); el rediseño visual `6770eed` espera
> autorización para publicarse.
> **No iniciar la Fase 5 ni publicar el rediseño sin el «sí» explícito.**
> **No publicar una release de bundles sin el «sí» explícito.**

---

## FASE 1 — Setup + Core del Soroban + Fading

### 1.0 Protocolo de continuidad documental

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P1-000 | Guardar el superprompt textual en `docs/SUPERPROMPT.md` (solo lectura) y crear `docs/RESUME.md` con qué es el proyecto, comandos exactos, cómo leer el checklist y el prompt de reanudación textual | Ambos archivos existen; `SUPERPROMPT.md` reproduce las instrucciones vigentes; `RESUME.md` copia el prompt de reanudación tal cual; commit atómico | [x] superprompt reconstruido desde el repo; `SUPERPROMPT.md` con role/context/architecture/step_by_step_plan/rules `f354469` |

`docs/RESUME.md` se actualiza al cerrar cada fase y ante cada tarea marcada `[!]`.

### 1.1 Spike de stack (ADR-001) — compuerta STOP-1

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P1-001 | Protocolo de continuidad: `git init`, `docs/`, `.gitignore` | Existen CHECKLIST/LOG/DECISIONES/MEDICIONES y repo git inicializado | [x] `e60cb87` |
| P1-002 | [NET] Reverificar versiones (dioxus, tauri, leptos, cpal, dioxus-native, rtrb, rand_chacha) en crates.io/docs.rs y registrarlas | Tabla de versiones + fecha en `DECISIONES.md` | [x] `e60cb87` (ADR-000) |
| P1-003 | Spike A: Dioxus 0.7 webview. Arnés de timing (1000 estímulos de 100 ms en frames) + cpal | JSON de telemetría crudo guardado en `spikes/results/`; corre sin intervención manual | [x] `86cb2df` |
| P1-004 | Spike B: Tauri 2 + Leptos. Mismo arnés y mismas condiciones | JSON de telemetría crudo en `spikes/results/` | [x] `a8f0c65` |
| P1-005 | Spike C: `dioxus-native` 0.8 alpha. Intento de compilación y arnés mínimo | Compila y corre, o registro literal del fallo | [x] `d45fd0a` (compila, corre y dibuja; timing no medido) |
| P1-006 | Tabla comparativa + ADR-001 + **STOP-1** | `docs/MEDICIONES.md` con números reales, ADR escrito, recomendación argumentada | [x] STOP-1 resuelto: B + AGPL-3.0-or-later |

**Criterio de aceptación del spike:** reportar error de duración real (no objetivo) de 100 ms × 1000 reps medido en frames, % dentro de ±1 frame, p50/p95/máx en ms, tasa de refresco medida, y latencia software entrada→primer sample de audio. Todo dato no medido se marca **NO VERIFICADO**.

### 1.2 Workspace y CI local (bloqueada por STOP-1)

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P1-007 | Workspace de 5 crates + `rust-toolchain.toml` fijado + target wasm32 | `cargo metadata` resuelve; `rustc --version` coincide con el pin; el crate `ui` compila a wasm32 | [x] `62bf826` |
| P1-008 | `[workspace.lints]` + `clippy.toml` + `deny.toml` | `cargo clippy --workspace --all-targets -- -D warnings` limpio en esqueleto | [x] `62bf826` |
| P1-009 | `./ci.sh` (fmt + clippy + test + deny) | Corre local y falla ante un `unwrap` de prueba | [x] `62bf826` (probado: exit 1 con `unwrap`) |
| P1-010 | Regla de dependencias aplicada (ui→engine→core; content→core) | `cargo tree` lo demuestra; core no depende de engine/ui | [x] `62bf826` |

### 1.3 Core del soroban

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P1-011 | `Rod`, `Slot`, `Touch`, errores, `from_digit`/`digit`/`apply` | Biyección 0..=9 ↔ 10 estados; earth<=4 impuesto por API | [x] `d110290` |
| P1-012 | `Soroban`: `new`, `value`, `set_value`, `touch` | Property test ≥10.000 casos contra modelo u128 | [x] `a7c98a0` |
| P1-013 | `plan_add`/`plan_sub`: directo, complemento a 5, complemento a 10, acarreo/préstamo en cadena | Casos 999+1, 1000-1, 5+5, 4+1, 9+1, 10-1, cadenas aleatorias | [x] `bbb69e8` |
| P1-014 | Fuzz de secuencias de `touch` | Ninguna secuencia produce estado inválido (proptest) | [x] `5b5402f` |
| P1-015 | Regresión de fallos proptest | Semillas fallidas commiteadas como casos fijos | [x] `7d3458c` |

### 1.4 Fading y geometría

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P1-016 | `FadeStage`, `FadeProfile`, `for_stage`, `lerp` con clamp | Test de monotonía de opacidades por etapa | [x] `d84afe0` |
| P1-017 | `SorobanGeometry`: `hit_test`, `slot_rect` normalizados | `hit_test` idéntico en las 5 etapas, incluida Blank | [x] `5c68325` |
| P1-018 | Test de independencia opacidad/hit-testing | Mismo `Touch` para la misma coordenada en Full/Ghost/Rods/UnitMarks/Blank | [x] `09aa0d9` |

### 1.5 UI mínima

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P1-019 | Soroban interactivo (mouse/touch), control de fading, lectura opcional de valor, reinicio | Interacción funcional; sin asignaciones por frame en el camino de dibujo | [x] `b9ac34a` |
| P1-020 | Respuesta visual inmediata (feedback de toque) | Toque resalta ficha antes del siguiente frame | [x] `b9ac34a` |

### 1.6 Transversales (requisitos añadidos 2026-11-13)

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P1-024 | [NET] Publicación en GitHub Pages: build wasm, base path, workflow de Actions, PWA offline | URL de Pages carga la app y funciona sin red tras la primera visita | [x] `0c17d0b` (URL y precache verificados; offline real no forzado) · docs `88a58a1` |
| P1-025 | Licencia copyleft + licencias de contenido y dependencias | `LICENSE` (AGPL-3.0-or-later), `LICENSE-CONTENT` (CC BY-SA 4.0), `CREDITS`, `deny.toml`; `cargo deny check licenses` limpio | [x] `62bf826` |
| P1-026 | Sistema de diseño «Material Aero»: tokens M3 + capa Aero, claro/oscuro/auto | `docs/DISENO.md`; contraste AA; `prefers-reduced-transparency`/`-motion`; sin destellos de pantalla completa | [x] `62bf826` |
| P1-027 | i18n español/inglés sin strings hardcodeados | Diccionarios RON validados, paridad de claves, selector persistido, `<html lang>` correcto | [x] `62bf826` |

### Cierre de Fase 1

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P1-021 | Property tests ≥10.000 casos (varilla válida, `value()` vs referencia, planes reproducen el resultado) | `cargo test` verde y contador de casos visible | [x] `64a6144` |
| P1-022 | fmt + clippy `-D warnings` + sin `unwrap/expect/panic` fuera de tests | `./ci.sh` verde | [x] `b606beb` |
| P1-023 | INFORME DE FASE + pregunta de autorización Fase 2 | Informe entregado con números reales | [x] `8d42b44` (informe en `docs/INFORME-FASE-1.md`) |

---

## FASE 2 — Motor temporal, audio y drills Flash Anzan

### 2.1 Motor

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P2-001 | `engine::schedule`: `FramePlan` precomputado (estímulo/hueco), consulta O(log n) sin asignar; telemetría de frames y aparición con ventana fija (p50/p95/máx) | Tests de bordes, plan contiguo, 6 frames = 100 ms a 60 Hz y telemetría sin heap | [x] `71285f0` |
| P2-002 | `engine::drill`: generador determinista Flash Anzan (dígitos, términos, semilla), respuesta y verificación | Property tests: reproducibilidad, rangos de términos, suma y respuesta exacta | [x] `71285f0` |
| P2-003 | `engine::audio`: clic sintetizado con envolvente, mapeo frame→muestra y offsets por estímulo | Longitud, amplitud acotada, envolvente y offsets verificados | [x] `71285f0` |

### 2.2 UI y contenido

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P2-004 | UI `FlashAnzan`: reproducir plan con `requestAnimationFrame`, DOM directo sin asignaciones por frame, respuesta y feedback | Funciona en la web local; callback sin `format!` ni `Vec` por frame | [x] `3b26451` |
| P2-005 | Audio web (WebAudio) por estímulo, con mute; sin bloquear el frame | Un nodo por estímulo, ninguno por frame; sin errores de consola; audición real **NO VERIFICADA** | [x] `3b26451` |
| P2-006 | i18n es/en del modo y ajustes con paridad de claves | Ambos RON parsean; textos verificados en `?lang=en` | [x] `3b26451` |
| P2-007 | Persistencia de ajustes (dígitos, términos, sonido) | Recarga conserva `3,7,1` en `localStorage` (verificado en navegador) | [x] `3b26451` |

### 2.3 Cierre

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P2-008 | Tests, `./ci.sh` y build web verdes | 75 tests, 150.000 casos de propiedades, `CI OK`, wasm 877 KB (203 KB gzip) | [x] `f354469` |
| P2-009 | INFORME DE FASE 2 + RESUME/LOG/CHECKLIST actualizados | Informe con números reales y límites **NO VERIFICADO** | [x] `f354469` |
| P2-010 | [NET] Publicar Fase 2 en GitHub Pages | URL sirve el modo Flash Anzan y el SW v6; requiere autorización de push | [x] `6943d86`; Pages run 43533448602 en verde |

---

## FASE 3 — Progresión, sesiones y estadísticas

La Fase 3 se dividió en **3a** (progresión y estadísticas, cerrada) y **3b** (sesiones,
exportación, errores por caso pedagógico y repaso espaciado, pendiente).

### 3a Progresión y estadísticas

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P3-001 | `engine::progress`: niveles 1..=6 con `LevelSpec`, promoción tras 2 aciertos y descenso tras 2 fallos, precisión, mejor racha y `encode`/`decode` v1 validado | Tests de promoción/descenso, límites, codificación y 4 property tests de 10.000 casos | [x] `5819483` |
| P3-002 | UI del panel de progresión: nivel, acierto, racha, intentos y mejor racha; modo automático/práctica libre; reinicio de progreso | La UI refleja el estado; el modo automático fija dígitos y términos del nivel y deshabilita sus campos | [x] `5819483` |
| P3-003 | Persistencia `anzan.progress` y `anzan.progressive`, con validación al cargar | Recarga conserva `v1,2,0,0,2,2,2`; un valor corrupto vuelve al estado por defecto | [x] `5819483` |
| P3-004 | i18n es/en de las 12 claves nuevas y estilos del panel | Ambos RON parsean con paridad; verificado en `?lang=en` | [x] `5819483` |
| P3-005 | Tests, `./ci.sh`, build web, INFORME DE FASE 3A y actualización de LOG/RESUME/CHECKLIST | 86 tests, 190.000 casos de propiedades, `CI OK`, wasm 1.005.246 B (219.812 B gzip) | [x] `3e4badc` |

### 3b Pendiente

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P3-006 | Historial de sesiones (fecha, nivel, aciertos, tiempo) y exportación CSV | `SessionRecord` v1, historial de 100 y CSV verificado en el navegador | [x] `10255b6` |
| P3-007 | Errores por caso pedagógico (movimiento directo, complemento a 5, complemento a 10, acarreo/préstamo) | Clasificación por suma parcial y estadísticas por caso dominante; tests y property tests | [x] `10255b6` |
| P3-008 | Repaso espaciado tipo FSRS con datos propios, sin dependencias opacas | Planificador tipo SM-2, serializado por nivel, con intervalos acotados y property tests | [x] `10255b6` |
| P3-009 | Cierre de Fase 3 completa: informe final, RESUME/LOG/MEDICIONES y compuerta | 109 tests, 240.000 casos, `CI OK`, wasm 1.187.902 B (259.425 B gzip), informe final | [x] `bda99db` |

## FASE 4 — Host de escritorio Tauri y audio nativo

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P4-001 | [HW] Crate/binario Tauri de producción que carga la misma UI Leptos compilada | `crates/desktop` compila y abre una ventana Tauri con el `dist/` real; `./ci.sh` sigue verde y la regla de dependencias incluye el host | [x] `546d133` |
| P4-002 | [HW] Audio nativo cpal con `Fixed(256)` y callback en tiempo real | El callback no asigna ni bloquea (cola SPSC `rtrb`), reproduce el clic precalculado y expone comandos de arranque/parada | [x] `753ed23` |
| P4-003 | [HW] Reloj de audio y supervisión nativa frame a frame | Reloj de muestras monótono, jitter de callback en histograma fijo, contadores de clics y bloques; visible en la UI | [x] `753ed23` |
| P4-004 | Persistencia en disco con comandos Tauri | Snapshot inicial inyectado y escritura atómica de `store.json`; la web sigue usando `localStorage` | [x] `d79856d` |
| P4-005 | [HW] Empaquetado de escritorio y firma documentada | Bundle Linux generado o limitación declarada; firma real **NO VERIFICADA** sin claves | [x] `2aef0f3` |
| P4-006 | [HW] Medición de la app real (ventana completa) y comparación con el spike B | `docs/MEDICIONES.md` con método, N, percentiles, entorno y límites; audio acústico **NO VERIFICADO** | [x] `2c404d3` |
| P4-007 | Cierre de Fase 4: tests, CI, informe y compuerta | `./ci.sh` verde, informe con números reales y límites, RESUME/LOG/CHECKLIST al día | [x] `e5d901a` |

## FASE 5 — Accesibilidad avanzada, web móvil y rendimiento

Alcance ajustado por el usuario (ADR-007): **sin Android ni háptico**. Detalle y riesgos en
`docs/PLAN-FASE-5.md`.

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| P5-001 | Auditoría de accesibilidad y correcciones | Teclado, foco, ARIA, contraste y `prefers-contrast` documentados y corregidos; lector de pantalla manual **NO VERIFICADO** con protocolo | [ ] |
| P5-002 | Fotosensibilidad del drill | Frecuencia y área de cambio medidas/documentadas; sin destellos >3 Hz ni pantalla completa | [ ] |
| P5-003 | Web móvil digna | Breakpoints, objetivos táctiles, viewport y aviso de límites verificados en viewports emulados; móvil real **NO VERIFICADO** | [ ] |
| P5-004 | Presupuesto de rendimiento | wasm <2 MB gzip, primera carga con throttling, CPU en reposo y registro en `docs/MEDICIONES.md`; batería móvil **NO VERIFICADA** | [ ] |
| P5-005 | Cierre de Fase 5: tests, CI, informe y compuerta | `./ci.sh` verde, informe con números y límites, RESUME/LOG/CHECKLIST al día | [ ] |

## FASE M — Multiplicación 空盘前乘法 (prioridad por ADR-008)

La multiplicación es **requisito central del producto** y se ejecuta antes de la Fase 5.
Método, ejemplos, progresión y verificación en `docs/METODO-MULTIPLICACION.md`.
Alcance: enteros hasta 3×3 cifras; atajos como unidad avanzada al final; sin decimales.

| ID | Tarea | Aceptación verificable | Estado |
|----|-------|------------------------|--------|
| M-001 | `engine::multiplication`: modelo de pasos y producto | API determinista, límites documentados y property tests ≥10 000 por categoría; sin `unwrap`/`expect` | [x] `c833b04` |
| M-002 | Contenido M0–M3 es/en (tabla, 1×1, 2×1, 1×2) | Lecciones con ejemplo resuelto y resultados comprobados por test | [x] `418533c` |
| M-003 | UI «Aprender»: lista, lección y visor de varillas | Reproductor accesible por teclado; estado tras cada paso | [x] `00c9361` |
| M-004 | Práctica M1–M6 con generación y corrección | Niveles correctos, feedback y telemetría propia; tests | [x] `cd1d36a` |
| M-005 | Contenido M4–M6, mental y atajos | 2×2, 3×2, 3×3, mental y atajos con condiciones y ejemplos | [x] `bcc8574` |
| M-006 | Persistencia y progreso de multiplicación | Claves propias, migración segura y recarga que conserva | [x] `71bb3fe` |
| M-007 | Accesibilidad, i18n, rendimiento, CI e informe | `./ci.sh` verde, sin regresiones y límites declarados | [x] `5b5f157` |

**Cierre de la Fase M (2026-11-16):** 135 tests, 280.000 casos de propiedades, `CI OK`,
wasm 1.708.695 B (347.160 B gzip) y verificación en Chromium de escritorio. Informe con
números, diseño, límites y lo **NO VERIFICADO** en `docs/INFORME-FASE-M.md`.
**Publicada** en Pages con `aef0ca6` (SW `anzanlab-v9`; wasm servido 1.707.369 B).
**Rediseño visual posterior** en `6770eed`: ábaco dibujado, tabla 大九九, tabla de productos
parciales, tarjetas y medidor). **Paleta y nombre** en `5f2383d`: la app pasa a
**MathMaster**, con diez acentos de hues distintos. **Acento único, tema y limpieza** en
`7a88b96`: una sola muestra activa cambia toda la app y el fondo, interruptor claro/oscuro
junto a las muestras, persistencia en `anzan.accent`/`anzan.theme`, telemetría visible
eliminada y service worker `mathmaster-v11`; wasm local 1.850.383 B (373.012 B gzip),
138 tests y `CI OK`. Rediseño, renombrado y ajuste están **sólo en local**, pendientes de
autorización para publicarlos.
