# LOG — MathMaster

Entradas append-only. Formato: fecha · ID · acción · resultado · errores literales · próximo paso.

## 2026-11-13 · P1-001 · Protocolo de continuidad
- Acción: inspección del directorio de trabajo (vacío, sin repo git), creación de `docs/` y protocolo.
- Resultado: entorno verificado: rustc 1.99.1 (2026-10-13, Arch), GNOME/Wayland, webkit2gtk 4.1, ALSA con salida analógica (card 1).
- Errores: ninguno.
- Próximo paso: P1-002 reverificación de versiones.

## 2026-11-13 · P1-002 · Reverificación de versiones
- Acción: `cargo info` contra crates.io para dioxus, dioxus-desktop, dioxus-native, tauri,
  leptos y cpal; consulta de las versiones de webkit2gtk/GTK del sistema.
- Resultado: dioxus 0.7.10 (estable), dioxus-native 0.8.0-alpha.3 (sólo alpha),
  tauri 2.12.5, leptos 0.8.23, cpal 0.18.2, rtrb 0.4.1, rand_chacha 0.10.0;
  webkit2gtk 4.1 (2.54.2), GTK 3.24.52. Registrado en ADR-000 (`docs/DECISIONES.md`).
- Errores: ninguno.
- Próximo paso: P1-003, arnés del spike A.

## 2026-11-13 · P1-003 · Spike A: Dioxus 0.7.10 webview
- Acción: arnés de calibración rAF, presentador JS de dos capas preasignadas, telemetría
  por estímulo, ping IPC y sonda cpal (buffer 256 vs default).
- Resultado: 4 corridas completas de 1000×100 ms y 300×200 ms. Cadencia 62.1 Hz en 3/4
  corridas (entrega 96.56 ms al pedir 100 ms, −3.4 %); 60.1 Hz en 1/4 (99.83 ms, 2 gaps).
  Error abs p50/p95 3/4 ms, máx 10–11 ms; 100 % dentro de ±1 frame de la cadencia medida.
  IPC `document::eval`: 0/600 mensajes intermedios JS→Rust. Audio buffer 256:
  trigger→primer sample p50 2.60 ms, offset 10.67 ms; default: p50 7.92 ms, offset 85.3 ms.
  JSON en `spikes/results/spike-a-*.json`.
- Errores: (a) la primera corrida agregó drift en unidades inconsistentes (descartada
  salvo `wall_ms`); (b) sin foco, WebKit suspende rAF y el watchdog aborta con
  `raf_stalled`; se resolvió con `GDK_BACKEND=x11` + `xdotool windowactivate`.
- Próximo paso: P1-004, mismo arnés sobre Tauri + Leptos.

## 2026-11-13 · P1-004 · Spike B: Tauri 2.12.5 + Leptos 0.8.23 CSR
- Acción: host Tauri con comandos `ping`/`report`/`fail`, frontend Leptos compilado a wasm
  y el mismo presentador JS del spike A.
- Resultado: 2 corridas completas. Presentación clavada a 60.0 Hz (16.66 ms/frame);
  100 ms → media 99.963/100.001 ms, drift −37/+1 ms, error abs p95 1 ms, máx 1–4 ms,
  0 gaps. 200 ms → 200.007/200.003 ms. IPC `invoke`: p50 1 ms, p95 1 ms, máx 4–7 ms en
  600 pings a 60 Hz. JSON en `spikes/results/spike-b-*.json`.
- Errores: el toolchain wasm32 no estaba instalado y no hay `sudo`; se usó un sysroot local
  (`/tmp/opencode/sysroot`, paquete `rust-wasm-1:1.99.1-2`) y wasm-bindgen CLI 0.2.130.
  El paquete `rust-wasm` del mirror actual (`-4`) falla con `E0514`.
- Próximo paso: P1-005, intento de dioxus-native 0.8 alpha.

## 2026-11-13 · P1-005 · Spike C: dioxus-native 0.8.0-alpha.3
- Acción: crate mínimo con `dioxus = "=0.8.0-alpha.3"` y feature `native`; ventana + texto.
- Resultado: compila limpio (2m57s, target 1.9 GB, binario 41 MB) y corre 12 s sin crash.
  Ventana X11 800x600 “Dioxus App”; render confirmado capturando la ventana con
  `import -window` (texto legible). No se midió cadencia: no hay `rAF` de plataforma.
- Errores: (a) por defecto abre ventana Wayland, invisible para xdotool; se forzó
  `WAYLAND_DISPLAY= WINIT_UNIX_BACKEND=x11`; (b) reporta `scale factor: 2` y dibuja al doble;
  (c) `ffmpeg x11grab` de la raíz X11 sale negro y GNOME Shell deniega la captura
  (`Screenshot is not allowed`), pero la captura por ventana X11 sí funciona.
- Próximo paso: P1-006, tabla comparativa y ADR-001.

## 2026-11-13 · P1-006 · Tabla comparativa + ADR-001 + STOP-1
- Acción: consolidar A/B/C en `docs/MEDICIONES.md`, completar ADR-001 en
  `docs/DECISIONES.md` y preparar la compuerta STOP-1.
- Resultado: recomendación fundada en medición: **B (Tauri 2.12.5 + Leptos 0.8.23)** por
  cadencia vsync estable a 60.0 Hz, 100 ms con error p95 de 1 ms e IPC `invoke` de 1 ms;
  A entrega 96.56 ms (−3.4 %) en 3/4 corridas y no entrega mensajes intermedios por
  `document::eval` (0/600). C compila/dibuja pero es alpha y no tiene timing medido.
- Errores: ninguno nuevo; se documentó que la calibración rAF puede mentir y que el
  scanout físico sigue NO VERIFICADO.
- Próximo paso: **STOP-1** — esperar decisión del usuario sobre el stack antes de P1-007.

## 2026-11-13 · P1-006 (ampliación) · Requisitos transversales y remoto
- Acción: incorporar GitHub Pages, licencia copyleft, Material Aero claro/oscuro e i18n
  es/en; conectar el remoto `https://github.com/wulflo/MathMaster` (público, vacío).
- Resultado: ADR-003 (multi-target web/escritorio), ADR-004 (AGPL-3.0-or-later + CC BY-SA
  4.0 + CC0), ADR-005 (Material Aero + temas), ADR-006 (i18n RON). Tareas P1-024..P1-027
  añadidas. `LICENSE`, `LICENSE-CONTENT`, `README.md` y `CREDITS.md` creados.
- Errores: ninguno. El stack sigue sin decidir; no se implementó UI ni build web.
- Próximo paso: confirmar STOP-1 (stack) y la licencia; luego P1-007 y el workflow de Pages.


## 2026-11-13 · P1-000 · Protocolo documental (bloqueada)
- Acción: crear `docs/RESUME.md` con qué es el proyecto, comandos exactos (instalación,
  build, CI, tests, benchmarks de timing), cómo leer el checklist y el prompt de
  reanudación textual; agregar P1-000 al checklist.
- Resultado: `docs/RESUME.md` creado y verificado contra el repo. `docs/SUPERPROMPT.md`
  **no existe**: el superprompt original no está en el contexto ni en el repositorio.
- Errores: bloqueo `[!]` por falta del texto original; no se inventa ni se resume.
- Próximo paso: el usuario aporta el superprompt; guardarlo verbatim y cerrar P1-000.

## 2026-11-13 · P1-007..P1-010 · Recuperación de workspace y CI
- Acción: revisar el árbol tras el corte; el workspace de 5 crates, el pin 1.99.1, los
  lints, `clippy.toml`, `deny.toml`, `./ci.sh` y los scripts de reglas ya estaban escritos
  pero sin commit.
- Resultado: `cargo metadata` resuelve; `rustc 1.99.1` coincide con el pin; `./ci.sh`
  termina en `CI OK` (fmt, prohibidas, clippy `-D warnings`, tests, dependencias, deny y
  check wasm). `cargo tree` confirma core sin dependencias internas.
- Errores: ninguno; el corte sólo dejó los cambios sin commitear.
- Próximo paso: commit de recuperación y continuar con P1-011.

## 2026-11-13 · P1-024..P1-027 · Recuperación de web, licencias, diseño e i18n
- Acción: revisar `web/`, workflows de CI/Pages, `crates/content` (RON es/en),
  `crates/ui/assets/style.css`, `CREDITS.md`, `deny.toml` y completar `docs/DISENO.md`.
- Resultado: P1-025 y P1-027 verificadas (`cargo deny check` limpio; ambos locales
  parsean). P1-026 cerrada con `docs/DISENO.md` y contrastes AA calculados (texto
  15.39:1/17.51:1; muted 6.70:1/10.21:1). P1-024 queda `[~]`: el workflow está listo pero
  `https://wulflo.github.io/MathMaster/` devuelve **404** y el remoto sigue en `cdcc49d`;
  no se hizo push.
- Errores: la nota previa «Web desplegada en Pages» era incorrecta; se corrigió en el
  checklist y en `RESUME.md`.
- Próximo paso: P1-011; P1-024 espera autorización de push.

## 2026-11-13 · P1-011 · Rod, Slot, Touch y errores
- Acción: implementar `crates/core/src/rod.rs`: `Rod` (1 celestial + 4 terrestres),
  `Slot` validado (1..=4), `Touch { rod, slot }`, `RodError`, `from_digit`, `digit`,
  `from_parts`, `set_earth` y `apply`.
- Resultado: biyección 0..=9 ↔ 10 estados; `earth <= 4` impuesto en `from_parts`,
  `set_earth` y `Slot::earth`; `apply` nunca produce un estado inválido. 10 tests verdes
  en `anzan-core` y clippy `-D warnings` limpio.
- Errores: un test esperaba mal el efecto de tocar una ficha terrestre activa; se corrigió
  la expectativa (tocar la ficha i deja i-1 activas).
- Próximo paso: P1-012, `Soroban` y property tests contra modelo u128.

## 2026-11-13 · P1-012 · Soroban y property tests u128
- Acción: implementar `crates/core/src/soroban.rs` con `new`, `from_value`, `value`,
  `set_value`, `set_digit`, `touch`, `reset` y `digits`; límite de 38 varillas para que
  `value()` nunca desborde `u128`.
- Resultado: property tests con proptest 1.12.0 y 10.000 casos por propiedad: ida y vuelta
  de `set_value`, rechazo de overflow, secuencias de toques contra un modelo u8
  independiente y validez de todas las varillas. 18 tests verdes; clippy limpio.
- Errores: dos préstamos mutables/immutables y un shadowing en `set_value`; corregidos.
- Próximo paso: P1-013, planes de suma y resta.

## 2026-11-13 · P1-013 · Planes de suma y resta
- Acción: implementar `crates/core/src/plan.rs` (`plan_add`/`plan_sub`, `Plan`, `Step`,
  `StepKind`) con transiciones por varilla: movimiento directo, complemento a 5,
  complemento a 10 y acarreo/préstamo en cadena.
- Resultado: casos 999+1, 1000-1, 5+5, 4+1, 9+1 y 10-1 verificados; property tests de
  10.000 casos para sumas y restas aleatorias reproducen el resultado. Clippy limpio.
- Errores: (a) el test de cadena miraba todos los pasos, incluidos los de la varilla de
  unidades; se filtró por `touch.rod > 0`; (b) el property test permitía sumas ≥ 10.000
  (overflow correcto) y las trataba como fallo; se corrigió la aserción. Proptest guardó
  la semilla `021da9b0…` en `crates/core/proptest-regressions/plan.txt`.
- Próximo paso: P1-014, fuzz de secuencias de `touch`.

## 2026-11-13 · P1-014 · Fuzz de secuencias de touch
- Acción: agregar property test de secuencias largas (hasta 120 toques) sobre 1..=38
  varillas, con valores iniciales válidos y toques dentro y fuera de rango; incluye
  `Slot::Earth(0)` y `Slot::Earth(5)` construidos fuera de la API.
- Resultado: ninguna secuencia corrompe el estado; cada varilla conserva `earth <= 4`,
  `value()` queda bajo el límite y los toques fuera de rango devuelven error sin mutar.
  `Rod::apply` ahora ignora índices terrestres inválidos en vez de desbordar.
- Errores: `RangeInclusive::contains` no es const en Rust 1.99; se reemplazó por
  comparaciones explícitas.
- Próximo paso: P1-015, regresión de fallos proptest.

## 2026-11-13 · P1-015 · Regresión de fallos proptest
- Acción: conservar `crates/core/proptest-regressions/plan.txt` (semilla
  `021da9b0530352e97c70f2ef172ad7958f8804c75aa4b19c321beebbc1047511`, caso
  5361+4639) y fijarla como test explícito de overflow.
- Resultado: la semilla se replaya antes de generar casos nuevos y el caso fijo pasa.
- Errores: ninguno; el «fallo» original era una expectativa incorrecta del test, no un
  defecto del planificador.
- Próximo paso: P1-016, fading y geometría.

## 2026-11-13 · P1-016 · Fading por etapas
- Acción: implementar `crates/core/src/fade.rs`: `FadeStage` (Full, Ghost, Rods,
  UnitMarks, Blank), `FadeLayer`, `FadeProfile::for_stage`, `opacity`, `lerp` con clamp y
  `lerp_profile`.
- Resultado: perfiles discretos con opacidades 0..=1 y monotonía no creciente por capa;
  interpolación acotada incluso con `t<0` o `t>1`. 6 tests verdes, clippy limpio.
- Errores: ninguno.
- Próximo paso: P1-017, geometría normalizada y `hit_test`.

## 2026-11-13 · P1-017 · Geometría normalizada y hit-testing
- Acción: implementar `crates/core/src/geometry.rs` con `Rect`, `SorobanGeometry`,
  `rod_rect`, `slot_rect` y `hit_test`, en coordenadas 0..1; la varilla interna 0
  (unidades) queda a la derecha.
- Resultado: 7 tests verdes: rectángulos dentro de 0..1, centros que devuelven su propia
  ficha, orden derecha→izquierda, huecos y bordes sin impacto, y `hit_test` idéntico en
  las 5 etapas de fading. Clippy limpio.
- Errores: ninguno.
- Próximo paso: P1-018, prueba explícita de independencia opacidad/impacto.

## 2026-11-13 · P1-018 · Independencia opacidad / hit-testing
- Acción: agregar test que recorre las 5 fichas de cada varilla, toma centro y dos puntos
  internos de cada rectángulo y compara `hit_test` en Full/Ghost/Rods/UnitMarks/Blank.
- Resultado: mismo `Touch` en todas las etapas, incluso con `beads = 0.0`; 45 puntos por
  corrida verificados. Clippy limpio.
- Errores: ninguno.
- Próximo paso: P1-019/P1-020, UI interactiva del soroban.

## 2026-11-13 · P1-019/P1-020 · UI interactiva del soroban y feedback
- Acción: `crates/ui/src/soroban.rs` con `SorobanBoard`: 8 varillas, un botón por ficha
  (5 por varilla), lectura de valor con mostrar/ocultar, reinicio y las 5 etapas de
  fading. Posiciones desde `SorobanGeometry`; fichas cuadradas centradas en su hueco,
  varillas finas y travesaño. Textos nuevos en es/en (sin cadenas visibles hardcodeadas).
- Resultado (Chromium local sobre `dist/`, wasm release): 40 fichas; 5+3 = 8; reinicio a
  0; ocultar valor; Ghost 0.20/0.85/1, Rods 0/0.85/1, Blank 0/0/0; en Blank las fichas
  siguen aceptando toques (valor 5), como exige P1-018; Enter con foco también toca;
  cambio es/en con `<html lang>` correcto. No hay bucle de dibujo ni asignaciones por
  frame: el DOM se actualiza sólo al cambiar estado o etapa.
- Feedback de toque: `pointerdown` aplica `is-active`/`is-pressed` en el microtask
  posterior al evento, antes de que el navegador pinte el frame siguiente (verificado
  leyendo el DOM tras `Promise.resolve()`). El tiempo de pintura real no se midió:
  **NO VERIFICADO** con telemetría de frames.
- Errores: (a) `class:is-pressed` se perdía al combinarlo con `class=closure`; se unificó
  en una sola closure. (b) El service worker servía HTML viejo (cache-first) y sin red
  el proxy devolvía 502; ahora la navegación es network-first con respaldo en caché y
  `status >= 500` también cae a caché. Verificado apagando el servidor: la app carga y
  monta las 40 fichas offline.
- Próximo paso: P1-021, contadores visibles de propiedades.

## 2026-11-13 · P1-021 · Contadores de propiedades ≥10.000
- Acción: `crates/core/tests/property_budget.rs` recorre 10.000 casos deterministas por
  categoría (LCG propio, sin dependencias nuevas): `value()` vs referencia y varillas
  válidas, secuencias de 10 toques, sumas y restas que reproducen el resultado.
- Resultado: `cargo test -p anzan-core --test property_budget -- --nocapture` imprime
  `P1-021: 10000 valores, 10000 secuencias de toques, 10000 sumas, 10000 restas (40000 casos)`
  y termina verde. Los `proptest` de `soroban` y `plan` siguen con 10.000 casos cada uno.
- Errores: el rango inicial del sumando permitía `valor + addend = 10000` (overflow
  correcto); se ajustó a `below(LIMIT - value)`.
- Próximo paso: P1-022, CI completo limpio.

## 2026-11-13 · P1-022 · CI completo limpio
- Acción: ejecutar `./ci.sh` después de la UI y del test de propiedades.
- Resultado: `fmt` limpio; `check-forbidden` sin hallazgos (incluye tests); `clippy
  --workspace --all-targets -- -D warnings` limpio; tests verdes (40.000 casos de
  propiedades incluidos); regla de dependencias OK; `cargo deny check` con advisories,
  bans, licenses y sources OK; `cargo check` wasm32 OK. Salida final: `CI OK`.
- Errores: ninguno.
- Próximo paso: P1-023, informe de fase y compuerta a Fase 2.

## 2026-11-13 · P1-023 · Informe de Fase 1 y compuerta a Fase 2
- Acción: escribir `docs/INFORME-FASE-1.md` con el estado por bloque, los números reales de
  CI y tests, lo verificado en la web, lo NO VERIFICADO y los bloqueos; actualizar
  `docs/RESUME.md` y el PUNTO DE REANUDACIÓN.
- Resultado: Fase 1 cerrada técnicamente (P1-001…P1-023). Números: 52 tests, 110.000 casos
  de propiedades, `./ci.sh` → CI OK, wasm release ≈ 622 KB, PWA offline verificada en local.
  Bloqueos: P1-000 `[!]` (falta el superprompt) y P1-024 `[~]` (Pages 404, falta push).
- Errores: ninguno.
- Próximo paso: esperar autorización para push de P1-024 y para definir/iniciar Fase 2 con
  el superprompt original. No se inicia Fase 2 sin el «sí» explícito.

## 2026-11-13 · P1-024 · Publicación en GitHub Pages y PWA
- Acción: push autorizado de `main` (`cdcc49d..0c17d0b`) y verificación del despliegue.
  Correcciones necesarias: (a) token `--anzan-on-accent` para el contraste AA del botón de
  etapa activo (`8821289`); (b) service worker red-primero con respaldo en caché
  (`09a1881`) y revalidación `no-cache`/`reload` para no precachear un wasm viejo por el
  `max-age` de Pages (`0c17d0b`).
- Resultado: workflows CI y Pages en verde (runs 43528152607 y 43528152665; después
  43528347287/43528347148). `https://wulflo.github.io/MathMaster/` carga la app con 40
  fichas y textos es/en; el service worker v5 queda controlando y precachea
  `index.html`, `anzan_app.js`, `anzan_app_bg.wasm`, manifest e icono. Lighthouse sobre la
  URL real: **Accesibilidad 1.00, Best Practices 1.00, SEO 1.00**, sin fallos (antes 0.95
  por contraste).
- Offline: verificado en local con el mismo build y el servidor apagado (la app monta y
  conserva las 40 fichas). En la URL real no se pudo forzar la ausencia de red con las
  herramientas disponibles: **NO VERIFICADO** el corte de red sobre Pages, aunque el
  precache v5 está completo.
- Errores: la primera publicación seguía mostrando el wasm anterior por la caché HTTP de
  Pages; se corrigió con revalidación explícita y se volvió a desplegar.
- Próximo paso: Fase 2 pendiente del superprompt original (P1-000 `[!]`).

## 2026-11-13 · P1-000 · Superprompt reconstruido
- Acción: `docs/SUPERPROMPT.md` reconstruido desde README, ADR-000…006, MEDICIONES,
  DISEÑO, CHECKLIST y el código, con las secciones `<role>`, `<context>`,
  `<architecture>`, `<step_by_step_plan>` y `<rules>`. Queda de solo lectura.
- Resultado: el proyecto ya no depende de un prompt externo para reanudarse; el documento
  fija visión, capas, fases 1–6, protocolo de tareas, reglas de código y compuertas.
- Errores: ninguno.
- Próximo paso: definir y ejecutar la Fase 2.

## 2026-11-13 · P2-001/P2-002/P2-003 · Motor temporal, drills y audio
- Acción: `engine::schedule` (FramePlan, builder validado, consulta binaria y
  `FrameTelemetry` de ventanas fijas), `engine::drill` (xorshift64*, configuración validada,
  respuesta y verificación) y `engine::audio` (envolvente, buffer, frame→muestra y offsets).
- Resultado: 25 tests de engine verdes; 40.000 casos nuevos de propiedades; ninguna
  asignación ni lock en las estructuras que consume el bucle de presentación.
- Errores: (a) `TelemetryError` no puede derivar `Eq` por el `f64`; se dejó en `PartialEq`.
  (b) La prueba de envolvente comparaba muestras con fase cero; se corrigió el índice.
  (c) Clippy rechazó variantes `Invalid*` redundantes; renombradas a `SampleRate`, `Tone` y
  `Frame`.
- Próximo paso: UI del modo Flash Anzan.

## 2026-11-13 · P2-004/P2-005/P2-006/P2-007 · UI Flash Anzan, audio, i18n y ajustes
- Acción: componente `FlashAnzan` con plan en frames, callback de `rAF` sin asignaciones por
  frame, campo de respuesta, feedback y telemetría; AudioContext con un nodo por estímulo;
  17 claves i18n nuevas; persistencia de ajustes en `localStorage`; service worker v6.
- Resultado: verificado en Chromium local: secuencia y respuesta funcionan, «Incorrecto» y
  «Respuesta correcta: 189» visibles, `?lang=en` completo, ajustes 3/7 persistidos tras
  recarga. Medición: frame p95 16,8 ms; aparición p95 0,0 ms (1 corrida, 180 frames).
- Errores: (a) `wasm_bindgen::Closure` no está en la raíz del crate; se usa
  `wasm_bindgen::closure::Closure`. (b) `[hidden]` no ocultaba los paneles porque el CSS de
  autor definía `display`; se añadió la regla explícita. (c) La primera medición de
  aparición daba ~10 ms porque el reloj de inicio se fijaba antes del primer callback; se
  movió al primer frame.
- Próximo paso: cierre de Fase 2 y compuerta.

## 2026-11-13 · P2-008/P2-009 · CI, build y cierre de Fase 2
- Acción: `./ci.sh` y `./scripts/build-web.sh`; `docs/INFORME-FASE-2.md`,
  `docs/MEDICIONES.md`, checklist y RESUME actualizados.
- Resultado: **CI OK**; 75 tests; 150.000 casos de propiedades; wasm 877.327 B (203.248 B
  gzip); sin errores de consola en el navegador. Fase 2 cerrada técnicamente.
- Errores: ninguno.
- Próximo paso: autorización para publicar P2-010 y para iniciar la Fase 3 (progresión,
  sesiones y estadísticas).

## 2026-11-13 · P3-001/P3-002/P3-003/P3-004 · Progresión y estadísticas
- Acción: `engine::progress` con 6 niveles (`LEVELS`, `level_spec`), `ProgressState`
  (promoción tras 2 aciertos, descenso tras 2 fallos, precisión, mejor racha) y
  `encode`/`decode` v1 validado; 4 property tests de 10.000 casos. UI: panel de progresión,
  modo automático/práctica libre, reinicio de progreso y aviso de cambio de nivel.
  Persistencia `anzan.progress` y `anzan.progressive`; 12 claves i18n nuevas y estilos.
- Resultado: 86 tests verdes; 190.000 casos de propiedades; verificado en navegador:
  dos aciertos → «Level up 2» con nivel 2 (1 dígito, 5 términos), persistencia
  `v1,2,0,0,2,2,2` tras recarga y «Free practice» con los campos habilitados.
- Errores: (a) El generador `any_progress` producía estados imposibles (racha > intentos)
  y hacía fallar `decode`; se acotaron racha y fallos a los intentos. (b) El test UI
  `proportional_levels_change_the_plan` asumía que el nivel 6 dura más frames; en realidad
  dura 168 vs 135 porque acorta los estímulos. Se reescribió para comprobar dificultad
  (más dígitos, estímulo más corto) y que los planes difieren. (c) Variables de etiqueta
  movidas a closures: se clonaron antes de `view!`.
- Próximo paso: Fase 3b (historial/exportación, errores por caso pedagógico y repaso
  espaciado) y autorización para publicar P2-010.

## 2026-11-13 · P3-005 · CI, build y cierre de Fase 3a
- Acción: `./ci.sh` y `./scripts/build-web.sh`; `docs/INFORME-FASE-3A.md`,
  `docs/CHECKLIST.md`, `docs/RESUME.md` y README actualizados; Fase 3 dividida en 3a
  (cerrada) y 3b (P3-006…P3-009).
- Resultado: **CI OK**; 86 tests; 190.000 casos de propiedades; wasm 1.005.246 B
  (219.812 B gzip), dentro del presupuesto de < 2 MB comprimido. Sin errores de consola en
  el navegador. Fase 3a cerrada técnicamente.
- Errores: ninguno.
- Próximo paso: compuerta; autorización para P2-010 y para iniciar la Fase 3b.

## 2026-11-14 · P2-010 · Publicación de Fase 2 + 3a en GitHub Pages
- Acción: `git push origin main` del commit `6943d86` (Fase 2 + 3a) con autorización del
  usuario. Los workflows CI y Pages de GitHub Actions terminaron en verde (Pages run
  43533448602).
- Resultado: <https://wulflo.github.io/MathMaster/> sirve la Fase 2 + 3a con service worker
  controlando la página, el modo Flash Anzan y el panel de progresión.
- Errores: ninguno. La publicación de la 3b se hará con el push del cierre.
- Próximo paso: Fase 3b.

## 2026-11-14 · P3-006/P3-007/P3-008 · Historial, casos y repaso
- Acción: `engine::session` (registro v1, historial de 100, CSV y estadísticas por caso),
  `engine::pedagogy` (clasificación directo/complemento 5/complemento 10/acarreo en cadena
  y `Drill::cases`), `engine::review` (planificador tipo SM-2 con facilidad acotada e
  intervalos de 10 min a 1 año). UI: repaso próximo, panel por caso, últimas 6 sesiones y
  descarga CSV; al responder se guardan sesión, revisión y progreso. 17 claves i18n y
  service worker v7.
- Resultado: 109 tests verdes; 240.000 casos de propiedades. Verificado en navegador:
  sesión con fecha/tiempo/caso, estadísticas 1/1, repaso «En 24 h» tras acierto y
  «En 1 h» tras fallo, persistencia tras recarga, textos en inglés y CSV descargado con
  cabecera y una fila por sesión. Sin errores de consola.
- Errores: (a) El test de pedagogía esperaba `123+456` como directo; la unidad 1+4 cruza
  el 5 y requiere complemento a 5. Se corrigió la expectativa y se añadió `123+321` como
  caso directo. (b) El primer montaje movía `drill_correct`/`drill_wrong` antes de usarlos
  en el historial; se clonaron antes de las closures. (c) `session_case` quedó sin uso en
  la vista; se dejó fuera del destructuring.
- Próximo paso: cierre documental de Fase 3.

## 2026-11-14 · P3-009 · Cierre de Fase 3
- Acción: `./ci.sh`, `./scripts/build-web.sh`, `docs/INFORME-FASE-3.md`, actualización de
  CHECKLIST/RESUME/README/LOG y revisión del estado publicado.
- Resultado: **CI OK**; 109 tests; 240.000 casos de propiedades; wasm 1.187.902 B
  (259.425 B gzip), dentro del presupuesto. Fase 3 cerrada técnicamente y publicada.
- Errores: ninguno.
- Próximo paso: compuerta de Fase 4 (host Tauri y audio nativo).

## 2026-11-14 · P3-009 · Verificación post-despliegue de la Fase 3
- Acción: comprobar <https://wulflo.github.io/MathMaster/> tras el push `98a03fe` y el run
  de Pages 43534450991: versión del service worker, caché activa, control de página y
  contenido precacheado; registrar limpio (desregistrar, borrar cachés, cerrar pestaña y
  abrir de nuevo).
- Resultado: `service-worker.js` servido v7 (md5 `d4be61a3…`), caché `anzanlab-v7`, worker
  activado y controlando, seis entradas precacheadas y wasm cacheado idéntico al servido
  (SHA-256). Textos «Fase 3 completa…», sin errores de consola.
- Errores: una comprobación intermedia mostró la caché vacía porque el borrado manual
  compitió con la instalación; en el registro limpio el precache fue correcto. No se
  reproduce en condiciones normales.
- Límite: la carga realmente sin conexión sigue **NO VERIFICADA** (el navegador del entorno
  no emula red offline).
- Próximo paso: compuerta de Fase 4.

## 2026-11-14 · P4-001 · Host Tauri de escritorio
- Acción: `crates/desktop` con Tauri 2.12.5 + tauri-build 2.7.4, configuración con
  `frontendDist = ../../dist`, ventana de 960×820 y título desde `anzan-content`.
  Workspace y regla de dependencias ampliados; `ci.sh` omite el host cuando faltan
  `dist/` o webkit2gtk y lo declara; `deny.toml` ignora `RUSTSEC-2024-0370`
  (proc-macro-error unmaintained, transitivo de GTK, sin CVE).
- Resultado: compila y abre la UI de Fase 3 en X11 (`GDK_BACKEND=x11`), captura de
  ventana verificada con ImageMagick. `./ci.sh` → CI OK.
- Errores: ninguno; solo el aviso de que el runner sin webkit no compila el host.
- Próximo paso: audio nativo.

## 2026-11-14 · P4-002/P4-003 · Audio nativo, reloj y supervisión
- Acción: cpal 0.18 con `Fixed(256)` en un hilo dueño del `Stream`; cola SPSC `rtrb`
  para clics; clic precalculado por `engine::audio`; reloj de muestras atómico;
  histograma de jitter de 32 bins (50 µs), contadores de clics, cola llena, bloques
  tardíos y errores. Comandos `audio_start/click/stats/clock_ms`; UI que usa el audio
  nativo en Tauri y muestra jitter p95. Autoprueba `--ui-selftest` con evaluación JS.
- Resultado: 48 kHz, 2 canales, 256 frames; 3/3 clics por drill; jitter p95 50 µs, máx
  238 µs, 0 bloques tardíos; frame p95 17,0 ms; «Audio nativo · 48000 Hz · 256 bloques ·
  Jitter p95 50 µs» visible en la ventana real. `--audio-selftest`: 8/8 clics.
- Errores: (a) `tauri_invoke` enviaba un `Array` vacío y Tauri espera un objeto; los
  clics nativos no llegaban y las estadísticas quedaban vacías. Se cambió a `Object`.
  (b) Clippy exigió colapsar dos `if`; corregido con `let`-chain.
- Próximo paso: persistencia en disco.

## 2026-11-14 · P4-004 · Persistencia en disco
- Acción: `store.json` en el directorio de datos, snapshot inyectado como
  `window.__ANZAN_STORE__` con `initialization_script`, escritura atómica tmp+rename y
  comandos `store_get/set/remove`; la UI lee del snapshot nativo y escribe en disco,
  con `localStorage` como respaldo.
- Resultado: tres ejecuciones reales de la app acumulan 1, 2 y 3 sesiones en
  `~/.local/share/dev.anzanlab.desktop/store.json`, con progreso y repaso.
  `./ci.sh` → CI OK (45 core + 1 presupuesto + 58 engine + 2 content + 3 ui + 2 desktop).
- Errores: ninguno.
- Próximo paso: empaquetado Linux.

## 2026-11-14 · P4-005 · Empaquetado Linux y firma documentada
- Acción: tauri-cli 2.12.4 (`cargo install tauri-cli --version "^2"`); iconos RGBA
  32/128/256 y `scripts/package-desktop.sh` (reconstruye `dist/` y ejecuta
  `cargo tauri build --bundles appimage,deb`). Documento `docs/EMPAQUETADO.md` con
  requisitos, artefactos, SHA-256, contenido del `.deb` y límites.
- Resultado: AppImage 115 599 864 B (`b1b80d79…`) y `.deb` 3 371 362 B (`5d7bf885…`).
  El AppImage se ejecuta con la UI real (ventana capturada a dpr 2) y pasa la autoprueba:
  frame p95 17,0 ms, 3/3 clics, 0 bloques tardíos. El `.deb` extraído con `ar`+`tar`
  (sin instalar) ejecuta el mismo binario con los mismos resultados (1 bloque tardío en
  una corrida). `desktop-file-validate` acepta el `.desktop`.
- Errores: el primer `tauri build` falló con «icon … is not RGBA» porque ImageMagick
  escribió PNG sin canal alfa; se regeneraron con `PNG32:`. El CLI reescribió
  `tauri-build` como `{ version = "=2.7.4", features = [] }` y se conservó.
- Firma: **NO VERIFICADA**; no hay claves, ni firma GPG ni updater. La integridad
  disponible es el SHA-256. La instalación con `dpkg -i` y el arranque sin FUSE quedan
  como **NO VERIFICADO**.
- Próximo paso: P4-006 (medición de la app real).

## 2026-11-14 · P4-006 · Medición de la app real (ventana completa)
- Acción: modo `--measure` en `crates/desktop/src/measure.rs` con el mismo arnés y plan
  del spike B (capas preasignadas, 240 frames de calibración, 1000×100 ms y 300×200 ms)
  dentro de la UI real, después de un drill con audio nativo y persistencia.
  `scripts/measure-desktop.sh` ejecuta 5 corridas, activa la ventana con xdotool, reintenta
  una corrida sin JSON y agrega el resumen; crudos en `measurements/`.
- Resultado: 5/5 corridas con frame 16,093–16,100 ms, media 96,557–96,599 ms (−3,4 %),
  err p50/p95/máx 3/4/4 ms, 100 % dentro de ±1 frame y 0 gaps; 200 ms media
  193,08–193,23 ms, err p95 7 ms, máx 8 ms. Audio: 15/15 clics, jitter p95 50 µs, máx
  5149 µs en dos corridas, 2 bloques tardíos de 151 204 callbacks, 0 errores. Telemetría
  de la UI: frame p95 17,0 ms y aparición p95 0,0 ms en las 5.
- Comparación: el spike B re-ejecutado en la misma sesión dio 16,108 ms/96,650 ms (mismo
  régimen de 62,1 Hz) y, minutos antes, 16,643 ms/99,857 ms (régimen de 60,0 Hz). Los
  controles (UI oculta, sin audio, ventana redimensionada) siguieron a 62,1 Hz: el régimen
  es del entorno, no de la app; su causa última queda **NO VERIFICADA**.
- Errores: (a) `window.eval` desde `setup` competía con la carga de la página; los scripts
  de prueba pasaron a `initialization_script`. (b) Dos corridas abortaron con estado 0 y
  sin JSON durante el ajuste del script; se añadió reintento (hasta 3) y la tanda final
  completó 5/5. La causa exacta de esos abortos queda **NO VERIFICADA**.
- Próximo paso: cierre de Fase 4 (P4-007).

## 2026-11-14 · P4-007 · Cierre de Fase 4
- Acción: bundles regenerados con el código final (`3afa1497…` AppImage 115 628 536 B y
  `374eab9f…` .deb 3 394 442 B) y reverificados: AppImage ejecutado y `.deb` extraído pasan
  `--ui-selftest` (`ok: true`, frame p95 17,0 ms, aparición p95 0,0 ms, 3/3 clics, jitter
  p95 50 µs, 0 bloques tardíos). `docs/EMPAQUETADO.md` con los hashes nuevos.
  `docs/INFORME-FASE-4.md` escrito con alcance, números, decisiones, límites y compuerta;
  `docs/RESUME.md` y el punto de reanudación del CHECKLIST actualizados.
- Resultado: `./ci.sh` → **CI OK**; **111 tests** (45 core + 1 presupuesto + 58 engine +
  2 content + 3 ui + 2 desktop). Repositorio limpio salvo este registro.
- Límites: Fase 4 queda cerrada **técnicamente y sin publicar**; no se hizo push, tag ni
  release. Firma de bundles, audio acústico, instalación real y otras plataformas siguen
  **NO VERIFICADO**.
- Próximo paso: compuerta de Fase 4. Se requiere autorización explícita para (a) publicar
  los commits/bundles y (b) iniciar la Fase 5 (Android/háptico, accesibilidad, rendimiento
  y degradación móvil).

## 2026-11-14 · Publicación web de Fase 4 en GitHub Pages
- Acción: actualización del estado visible a Fase 4 en es/en y service worker a
  `anzanlab-v8` (precache nuevo, borra el de Fase 3), rebuild web y push
  `c57dde4..d550894` a `main`.
- Resultado: workflows **CI** (`43547444902`, 1 min 36 s) y **Pages** (`43547444914`, 53 s)
  verdes. El sitio sirve v8, el estado «Fase 4 completa…» en ambos idiomas y un wasm de
  **1 230 769 B** (≈263,8 KB gzip) con las cadenas de la fase; verificado con Chrome
  headless contra la URL real, 210 nodos/atributos `anzan-*` y sin errores de consola.
- Límites: el wasm del runner no coincide byte a byte con el local (`56fa7bbe…`,
  1 228 666 B); la compilación no es reproducible byte a byte. **No** se creó release, tag
  ni se subieron los bundles; la firma sigue **NO VERIFICADA**. La carga offline real no se
  re-midió en esta publicación.
- Extra: revisión de preparación de Fase 5 en `docs/PLAN-FASE-5.md` (alcance, entorno
  Android ausente, límites de verificación y desglose propuesto). No se inició la fase.
- Próximo paso: compuerta de Fase 5, pendiente de autorización explícita.

## 2026-11-14 · Alcance de Fase 5: sin Android ni háptico
- Decisión del usuario: el producto se entrega como web (GitHub Pages/PWA) o instalable de
  escritorio; no se hace cápsula Android y no se implementa háptico por el momento.
- Acción: ADR-007 en `docs/DECISIONES.md`; `docs/PLAN-FASE-5.md` reescrito con el alcance
  reducido y desglose P5-001…P5-005; sección FASE 5 dada de alta en `docs/CHECKLIST.md`
  (tareas `[ ]`); RESUME e informe de Fase 4 alineados. No se tocó código de producto.
- Consecuencia: quedan fuera SDK/NDK, emulador, APK, háptico y mediciones en dispositivo;
  lector de pantalla, móvil real y batería seguirán **NO VERIFICADO** con protocolo.
- Próximo paso: esperar el «sí» explícito para iniciar P5-001 (auditoría de accesibilidad).

## 2026-11-14 · M-001 — motor de multiplicación 空盘前乘法
- Decisión previa (ADR-008): la multiplicación es requisito central y se hace antes de la
  Fase 5. Alcance confirmado por el usuario: enteros hasta 3×3 y atajos como unidad
  avanzada; sin decimales.
- Acción: módulo `engine::multiplication` con `Multiplication` (factores, pasos, producto,
  `state_after`), `PartialProduct`, `RodState` y `MultiplicationDrill` (M1…M7).
- Método implementado: multiplicador por fuera y multiplicando por dentro, cada producto
  de una cifra con dos cifras y unidades en la varilla `p+q`, decenas en `p+q+1`.
- Verificación: 14 tests unitarios/de módulo y 7 property tests con **10 000 casos cada
  uno** (70 000 ejecuciones): producto exacto, reconstrucción, dos cifras, posiciones,
  estado por paso y conmutatividad. `anza-engine`: 72 tests verdes; `./ci.sh` → `CI OK`.
- Límites: 0…999 por factor (6 varillas); decimales y atajos fuera de este módulo. La
  calidad del cálculo mental real sigue **NO VERIFICADA**.
- Próximo paso: M-002 (contenido M0–M3 es/en).

## 2026-11-14 · M-002 y M-003 — contenido y sección «Aprender a multiplicar»
- M-002 (`418533c`): `content/multiplication/{es,en}.ron` con la sección y las lecciones
  M0–M3; modelo `MultiplicationContent`/`MultiplicationLesson`/`WorkedExample` con
  `deny_unknown_fields`; `load_multiplication(locale)`. Tests: paridad estructural es/en,
  ids `M0…M3` y producto exacto de cada ejemplo. UiStrings suma las 11 cadenas del
  reproductor en ambos idiomas.
- M-003 (`00c9361`): componente `MultiplicationLearn` con lista de lecciones, objetivo,
  párrafos, ejemplo resuelto, visor de varillas de sólo lectura y reproductor
  anterior/siguiente/inicio. El paso se anuncia en `aria-live` con producto, registro de
  dos cifras y acumulado; controles nativos con foco visible.
- Verificación en Chrome contra `dist/` local: M0 `2×4 = 08` (varillas 0,8); M2 `28×3` en
  dos pasos (06 → 60; 24 → 84); M3 `7×34` en dos pasos (21 → 210; 28 → 238); resultado final
  visible sólo al terminar; teclado con Enter mantiene el foco; inglés verificado en
  `?lang=en`. Sin errores de consola.
- Números: `cargo test --workspace` = **129 tests**; `./ci.sh` → `CI OK`. wasm local
  **1 513 582 B** (**316 331 B** gzip), dentro del presupuesto de 2 MB.
- Límites: sólo Chromium de escritorio; móvil real y lector de pantalla **NO VERIFICADOS**.
- Próximo paso: M-004 (práctica M1–M6).

## 2026-11-14 · M-004 — práctica de multiplicación M1–M7
- Acción: componente `MultiplicationPractice` dentro de la sección, con selector de niveles
  M1–M7 (M7 marcado como mental), ejercicio generado por `MultiplicationDrill` con semilla
  incremental, respuesta numérica y corrección contra el producto exacto.
- Feedback: «Correcto» o «Incorrecto; era {producto}», contador de aciertos/intentos y
  tiempo de respuesta (`js_sys::Date::now` en wasm, 0 fuera del navegador). El campo pasa a
  `readonly` tras comprobar: Enter envía y el foco permanece en el campo (probado).
- Cadenas: 11 nuevas en `UiStrings` y en ambos RON. Helpers puros `level_label` y
  `parse_answer` con dos tests (etiquetas y entradas vacías/no numéricas/negativas).
- Verificación en Chrome contra `dist/` local: M1 `2×9=18` correcto; M4 `65×27=1755`; M6
  `930×118` incorrecto con «era 109740»; M7 mental por Enter; «Otro ejercicio» limpia el
  estado; telemetría avanza. Sin errores de consola.
- Números: **131 tests** y `./ci.sh` → `CI OK`; wasm local **1 666 369 B** (**343 336 B
  gzip**), dentro del presupuesto de 2 MB comprimido.
- Límites: sólo Chromium de escritorio; móvil real y lector de pantalla **NO VERIFICADOS**.
- Próximo paso: M-006 (persistencia y progreso de multiplicación).

## 2026-11-16 · M-005, M-006 y M-007 — cierre de la Fase M
- M-005 (`bcc8574`): lecciones M4–M7 (23×45, 213×32, 213×324 y mental) y atajos S1–S4
  (12×14, 43×47, 34×74, 738×96) con su condición de uso; `show_steps` oculta el
  reproductor en mental y atajos. Verificado en Chrome: M6 recorre 9 pasos hasta 69012 con
  seis varillas; M7/S1 muestran sólo su conclusión.
- M-006 (`71bb3fe`): `MulProgress` por nivel con `encode`/`decode` versionado `v1` y
  property test de ida y vuelta (10.000 casos); `crates/ui/src/storage.rs` centraliza
  `read_key`/`write_key` y los comandos Tauri que vivían en `flash.rs`. Claves propias
  `anzan.mul.level` y `anzan.mul.progress`, separadas de las de suma. Verificado: tras
  recargar se conservan M4 y 1/1; con `v9;basura` y nivel `99` arranca en M7 y 0/0 y
  reescribe valores válidos.
- M-007 (`5b5f157`): `aria-label` «M0: título» en lecciones, `aria-pressed` en lecciones y
  niveles, `role="status"` en feedback, `readonly` tras comprobar para conservar el foco y
  `--anzan-danger` por tema (contraste ≈ 8,9:1 oscuro y ≈ 6,5:1 claro). Rendimiento:
  1.000 transiciones del reproductor en 24,8 ms; la sección no usa `rAF` ni temporizadores.
- Números: **135 tests** (46 core, 75 engine, 5 content, 7 ui, 2 app), **280.000** casos de
  propiedades (28 × 10.000), `./ci.sh` → `CI OK`; wasm local **1 708 695 B** (**347 160 B
  gzip**).
- Límites: sólo Chromium de escritorio; móvil real, lector de pantalla, Firefox/Safari,
  CPU/memoria y batería **NO VERIFICADOS**; eficacia pedagógica no medida. Informe en
  `docs/INFORME-FASE-M.md`. La sección **no está publicada** (SW `anzanlab-v8`).
- Próximo paso: compuerta de la Fase 5 reducida (ADR-007) y, aparte, autorización para
  publicar la Fase M en Pages con SW `v9`.

## 2026-11-16 · Ajuste visual previo a la publicación
- A petición del usuario se elimina la cadena `status` («Fase 4 completa…») de `UiStrings`
  y de ambos RON; la cabecera ya no muestra estado de fase.
- El kicker pasa a «soroban · multiplicación · cálculo mental» / «soroban · multiplication ·
  mental math».
- Cada sección tiene su acento dentro de la paleta sin cambiar el fondo azul: soroban teal,
  multiplicación violeta y Flash Anzan ámbar, con `--anzan-on-accent` propio por tema.
- wasm local **1 625 355 B** (**343 400 B gzip**); service worker subido a **`anzanlab-v9`**
  para publicar la fase M. Verificado en Chrome es/en sin errores de consola.

## 2026-11-16 · Rediseño visual de la multiplicación
- `6770eed`: ábaco dibujado (varilla, bola superior, beam y cuatro inferiores), tabla
  大九九 9×9 con resaltado que sigue al paso, tabla de productos parciales con fila actual,
  tarjetas de lección y de nivel, panel de fórmula y medidor de aciertos.
- La tarjeta sube a 62 rem y la sección usa dos columnas; el soroban se limita a 42 rem.
  Ocho cadenas nuevas es/en.
- Verificado en Chrome es/en y tema claro: M0 `2×4` resalta `08`, M6 `213×324` con seis
  varillas y acumulado `67900` tras el paso 4, práctica M4 `65×27` correcta al 100 %, sin
  errores de consola ni desbordamiento horizontal.
- wasm local **1 935 340 B** (**378 080 B gzip**), `CI OK` y 135 tests.
- La fase M publicada sigue siendo `aef0ca6` (SW `anzanlab-v9`); el rediseño queda en local
  a la espera de autorización para publicarlo.

## 2026-11-16 · Paleta de 10 acentos y nombre MathMaster
- `5f2383d`: la app se llama **MathMaster** en textos es/en, título, manifest, icono, host
  Tauri y documentación vigente; el service worker sube a `mathmaster-v10`.
- El CSS define `--anzan-accent-1`…`-10` con diez hues distintos y variantes para tema
  oscuro. Soroban usa accent-2 (teal), multiplicación accent-3 (violeta), Flash Anzan
  accent-4 (ámbar) y cabecera/pie accent-1 (azul); todos los controles de una sección
  comparten su acento.
- El fondo mezcla los diez acentos (9 % claro, 14 % oscuro); botones secundarios y campos
  se tiñen con el acento de su sección.
- Verificado en Chrome es/en: título y cabecera MathMaster, diez tokens distintos, tres
  acentos de sección, controles coherentes, caché `mathmaster-v10` y sin errores de
  consola. Contraste blanco/violeta 7,10:1 en tema claro.
- wasm local **1 937 324 B** (**378 391 B gzip**), 135 tests y `CI OK`. Sigue **sin
  publicar**: `origin/main` continúa en `aef0ca6` (fase M con SW `anzanlab-v9`).

## 2026-11-18 · Acento único, tema claro/oscuro y sin telemetría visible
- `7a88b96`: el acento pasa a ser **global y cambiable**. La cabecera muestra diez
  muestras; la elegida se inyecta en `.anzan-root` y cambia a la vez cabecera, botones,
  tarjetas activas, tablas, ábaco, medidor y fondo. Se eliminan los overrides de acento por
  sección.
- El fondo deriva del acento con dos resplandores y una mezcla del 16 % en claro y del
  26 % en oscuro.
- Nuevo interruptor claro/oscuro (`role="switch"`) junto a las muestras, con `data-theme`
  en `<html>`; sin valor guardado se sigue la preferencia del sistema.
- Persistencia: `anzan.accent` (0..9, normalizado) y `anzan.theme` (`light`/`dark`).
- Se elimina la telemetría visible: footer del drill, frame/aparición p95, estadísticas de
  audio nativo y sus cadenas es/en; `FrameTelemetry` sigue en el motor para planificar.
- Verificado en Chrome es/en: diez muestras, cambio global de acento y fondo, toggle en la
  misma fila, persistencia tras recargar, sin «Telemetría»/«Telemetry» y sin errores de
  consola. Contraste 4,99:1–7,90:1 en claro y 8,63:1–14,33:1 en oscuro.
- wasm local **1 850 383 B** (**373 012 B gzip**), 138 tests y `CI OK`; service worker
  `mathmaster-v11`. Sigue **sin publicar**: `origin/main` continúa en `aef0ca6`.
