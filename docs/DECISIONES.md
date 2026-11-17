# DECISIONES — MathMaster

ADRs breves. Cada uno: contexto, alternativas, decisión, consecuencias, estado.

---

## ADR-000 · Registro de versiones verificadas

Fecha: 2026-11-13. Comando usado: `cargo info <crate>` contra el índice de crates.io (red disponible).

| Crate | Versión estable / usada | Notas |
|---|---|---|
| rustc | 1.99.1 (2026-10-13) | Toolchain del sistema Arch; se fijará en `rust-toolchain.toml` (P1-007) |
| dioxus | 0.7.10 | Línea estable 0.7.x confirmada. `0.8.0-alpha.3` existe |
| dioxus-desktop | 0.7.10 / 0.8.0-alpha.3 | Renderer webview (wry/webkit2gtk en Linux) |
| dioxus-native | 0.8.0-alpha.3 | Sólo en la línea 0.8 alpha (blitz). No hay 0.7 estable |
| tauri | 2.12.5 | Línea 2 estable confirmada; 3.0.0-alpha.6 existe |
| leptos | 0.8.23 | Línea 0.8 estable; 0.9.0-rc1 existe |
| cpal | 0.18.2 | Features relevantes: `realtime`, `pipewire`, `pulseaudio`, `jack` |
| rtrb | 0.4.1 | Ring buffer SPSC sin locks; el spike A usó la línea 0.3 (resuelta por Cargo) |
| rand_chacha | 0.10.0 | Fuente determinista para property tests/fuzz |
| webkit2gtk (sistema) | 4.1 (2.54.2) | Requerido por dioxus-desktop y Tauri/wry en Linux |
| GTK sistema | 3.24.52 | — |

Pendiente de verificar cuando se agreguen: `directories`, `tempfile`, `proptest`, `criterion`, `fsrs`.

---

## ADR-001 · Shell de UI y renderer de estímulos

Estado: **ACEPTADO** (STOP-1 resuelto el 2026-11-13: el usuario eligió B).

### Contexto
Prioridad 1 es la precisión temporal de los estímulos visuales (objetivo ±1 frame a 60 Hz),
prioridad 2 la latencia entrada→sonido/háptico. La UI nunca calcula; sólo dibuja un
`StimulusPlan` precomputado. El shell debe poder reemplazarse sin tocar core/engine.

### Alternativas evaluadas
- **A. Dioxus 0.7.10 desktop (webview wry/webkit2gtk).** VirtualDom en Rust, mutaciones DOM
  enviadas a la webview por IPC.
- **B. Tauri 2.12.5 + Leptos 0.8.23 (CSR/wasm en la webview).** UI reactiva dentro de la
  webview; audio en el proceso Rust vía IPC, o audio wasm.
- **C. dioxus-native 0.8.0-alpha.3 (blitz/WGPU, sin webview).** Renderer nativo, dependencia
  de una línea alpha.

### Medición
Ver `docs/MEDICIONES.md` §Spike de stack y los JSON crudos en `spikes/results/`.
Resumen (1000 × 100 ms, ventana activa, X11/XWayland, release):

| | A · Dioxus 0.7 | B · Tauri 2 + Leptos |
|---|---|---|
| Cadencia real observada | 62.1 Hz (3/4) / 60.1 Hz (1/4) | 60.0 Hz (2/2) |
| Duración media a 100 ms | 96.56 ms (−3.4 %) / 99.83 ms | 99.96–100.00 ms |
| Error abs p95 / máx | 4 / 10–33 ms | 1 / 1–4 ms |
| IPC JS→Rust por frame | 0/600 mensajes | p50 1 ms, p95 1 ms, máx 4–7 ms |
| Supervisión desde Rust | no viable vía `eval` | viable vía `invoke` |

La causa física de que A corra a 62.1 Hz (¿timer interno de wry 0.53.5 en vez de vsync?)
no se verificó a nivel de compositor/scanout; se descarta como afirmación y se trata como
diferencia medida y reproducible en esta máquina.

### Decisión propuesta
**Adoptar B: Tauri 2.12.5 + Leptos 0.8.23 (CSR/wasm), con host Rust.**

Razones, en orden de prioridad del producto:
1. **Precisión temporal:** B quedó clavado al vsync de 60 Hz y entregó 100 ms con error
   p95 de 1 ms. A planifica en frames, pero su frame no es el del panel y entrega 96.6 ms
   cuando se piden 100 ms (3.4 % corto) en 3 de 4 corridas.
2. **Supervisión y audio:** el `invoke` de Tauri permite que el host Rust (dueño de core,
   planes, audio y reloj) supervise o gobierne la presentación frame a frame con 1 ms de
   RTT. `document::eval` de Dioxus no entregó **ningún** mensaje intermedio (0/600).
3. **Madurez:** ambas opciones usan wry/WebKitGTK estable; B no depende de una alpha.
4. **100 % Rust:** el frontend Leptos se compila desde Rust a wasm; no se introduce JS de
   producto (el JS del presentador es un arnés de medición y, en producto, sería una capa
   mínima generada/revisada, no lógica de negocio).
5. **GitHub Pages:** el requisito de publicar en Pages refuerza B: Leptos CSR produce un
   sitio estático wasm que se sirve tal cual, y la misma UI se embebe en Tauri. A también
   compila a web, pero su déficit temporal en escritorio sigue siendo el criterio decisivo.
   La web tendrá su propia medición (rAF del navegador, WebAudio) y no heredará estos
   números.

Coste aceptado: toolchain wasm32 + wasm-bindgen (se resolvió sin `sudo` con un sysroot
local) y dos artefactos de build (host + frontend). Se documentará en P1-007/009.

Alternativa de reserva: A (Dioxus) si el coste de toolchain pesa más que el 3.4 % de error
de duración; no se recomienda para el modo Flash Anzan, donde la duración del estímulo es
el parámetro pedagógico. C (`dioxus-native` 0.8 alpha) **compiló, corrió y dibujó** en el
intento mínimo, pero no se midió su cadencia (no expone un `rAF` de plataforma) y depende
de una alpha; no se adopta como shell principal. Podría reconsiderarse en el futuro si
publica una versión estable y demuestra vsync.

### Consecuencias
- El workspace de P1-007 tendrá: `core`, `engine`, `content`, `ui` (Leptos/wasm) y
  `app`/host Tauri, respetando `ui→engine→core` y `content→core`.
- El bucle crítico de presentación vive en la capa de presentación, alimentado por un
  `StimulusPlan` precomputado en Rust. En producto se implementa en Rust/wasm con
  `web-sys` (callback de `rAF`), sin lógica de negocio en JS; el presentador JS de dos
  capas es sólo el arnés de medición.
- El audio se queda en el proceso Rust (cpal 0.18, `BufferSize::Fixed(256)`; offset medido
  10.7 ms). La UI nunca calcula ni sintetiza en el camino crítico.
- Se debe añadir al CI local el target wasm32 y la versión de wasm-bindgen CLI fijada.
- Riesgo pendiente: verificar que la cadencia de 60 Hz se mantiene en la app real (con
  ventana a pantalla completa y carga de UI) y en Android. **NO VERIFICADO** hasta entonces.

Estado: **ACEPTADO**. STOP-1 cerrado el 2026-11-13: el usuario confirmó B y AGPL-3.0-or-later.
El workspace de P1-007 y el primer despliegue en Pages se construyeron sobre esta decisión.

---

## ADR-002 · Unidad de planificación temporal: frames, no milisegundos

Estado: aceptado (por especificación del producto).

- Los planes se expresan en frames enteros a la tasa medida; el objetivo es ±1 frame.
- Se mide y reporta el error real. Nunca se convierte un objetivo de ms a frames y se
  declara cumplido sin telemetría.
- A 60 Hz, 100 ms = 6 frames exactos (que es suerte aritmética; 90 ms y 110 ms no existen).

---

## ADR-003 · Multi-target: escritorio (Tauri) + web estática (GitHub Pages)

Estado: **ACEPTADO** (STOP-1, 2026-11-13).

### Contexto
El producto debe estar disponible en escritorio y en GitHub Pages, y seguir funcionando
offline. GitHub Pages sólo sirve archivos estáticos: no hay servidor, cabeceras propias ni
`SharedArrayBuffer` (no se pueden usar hilos wasm). La prioridad 1 sigue siendo la
precisión temporal; la web es un canal de distribución, no el patrón de referencia.

### Decisión propuesta
Un solo código de UI en **Leptos CSR** compilado a wasm, compartido por ambas cápsulas:

```
core  ← engine ← ui ← app-web   (wasm, GitHub Pages)
content ← core      ↖ app-desktop (Tauri, host Rust)
```

- `core`, `engine` y `content` se compilan también a `wasm32-unknown-unknown`: sin hilos,
  sin `std::time::Instant` directo y sin APIs de sistema en el camino común.
- `engine::platform` abstrae reloj, audio, almacenamiento, háptico y locale:
  - reloj: `performance.now()` (web) / `Instant` (escritorio);
  - audio: WebAudio (web) / cpal con `Fixed(256)` (escritorio);
  - almacenamiento: `localStorage`/IndexedDB (web) / archivos (escritorio);
  - háptico: Vibration API si existe (web) / plataforma (escritorio);
  - locale: `navigator.language` (web) / `sys-locale` (escritorio).
- La web se construye como sitio estático (`trunk`/wasm-bindgen) con `--public-url`
  configurable para el subpath `/MathMaster/`; se añade PWA con service worker cache-first
  para el requisito offline.
- Sin rutas de servidor: rutas por hash o `404.html` que sirve `index.html`.
- El workflow de GitHub Actions compila, ejecuta tests y publica en Pages. Nada de secretos
  en el frontend.

### Consecuencias y límites
- La latencia de audio web (WebAudio) y la cadencia de `rAF` del navegador se miden aparte;
  no se heredan los números de Tauri. Se declararán **NO VERIFICADO** hasta medirlas en
  Chrome/Firefox/Safari de escritorio y Android.
- Sin `SharedArrayBuffer`, el motor web es monohilo; los planes deben ser lo bastante
  baratos para no bloquear el frame.
- El escritorio Tauri sigue siendo la referencia de timing y audio; la web puede degradar
  con dignidad (mensaje claro, sin prometer precisión que no puede dar).
- Presupuesto inicial: bundle wasm comprimido < 2 MB y primera carga útil < 3 s en 4G.

---

## ADR-004 · Licencia copyleft y licencias de contenido

Estado: **ACEPTADO** (2026-11-13: AGPL-3.0-or-later confirmada).

- **Código:** AGPL-3.0-or-later. Es copyleft fuerte y cubre el uso en red, relevante porque
  la app se publica en GitHub Pages. Si se prefiere GPL-3.0-or-later, el cambio es de una
  línea, pero se pierde la cláusula de red.
- **Contenido pedagógico** (textos, ejercicios, datos RON): CC BY-SA 4.0.
- **Sonidos y diccionarios originales/generados:** CC0 1.0, como exige el producto.
- **Fuentes e iconos:** sólo SIL OFL / Apache-2.0 / MIT compatibles, con atribución en
  `CREDITS.md`.
- **Dependencias:** `cargo deny check licenses` con allowlist; prohibidas licencias
  incompatibles con AGPL. Se revisa también la licencia de los assets.
- Archivos: `LICENSE`, `LICENSE-CONTENT`, `CREDITS.md`, cabeceras SPDX donde sea práctico.
- La app muestra un enlace “Código fuente” y la licencia en “Acerca de”.

---

## ADR-005 · Sistema de diseño «Material Aero» (claro/oscuro)

Estado: **ACEPTADO** (2026-11-13). Implementación inicial en `crates/ui/assets/style.css`.

### Definición
“Material Aero” = tokens de **Material 3** (roles de color, tipografía, forma, elevación,
state layers) + una capa **Aero** de superficies translúcidas y brillo sutil:

- superficies con `backdrop-filter: blur(20px) saturate(140%)`, gradientes suaves, borde
  claro de 1 px y sombra difusa; sin biseles ni reflejos que reduzcan la legibilidad;
- tipografía Inter/Roboto Flex para UI y números tabulares (`font-variant-numeric:
  tabular-nums`) para el soroban;
- iconografía Material Symbols (Apache-2.0).

### Temas
- `auto | light | dark`; `auto` sigue `prefers-color-scheme`.
- Override manual persistido; atributo `data-theme` en `<html>`.
- Paletas con contraste AA: texto 4.5:1, UI 3:1, incluso sobre superficies translúcidas.
- Modo de contraste reducido y fallback opaco si no hay `backdrop-filter`.

### Accesibilidad y fotosensibilidad
- Respetar `prefers-reduced-motion`, `prefers-reduced-transparency` y `prefers-contrast`.
- **Nunca** parpadear el fondo completo ni hacer transiciones de luminancia bruscas.
- Aviso previo y modo de contraste reducido antes de estímulos rápidos.
- Foco visible, navegación por teclado y etiquetas ARIA.

### Rendimiento
- El blur se limita a pocas superficies; no se anima `backdrop-filter`.
- Fallback para GPUs lentas y para la web móvil.
- Entregable: `docs/DISENO.md` con tokens, estados y capturas.

---

## ADR-006 · Internacionalización español/inglés

Estado: **ACEPTADO** (2026-11-13). Implementación inicial en `crates/content` y `crates/app/src/web.rs`.

- Locales: `es` (por defecto) y `en`. Ambos LTR.
- **Cero strings hardcodeados** en componentes: todo en `content/i18n/es.ron` y `en.ron`,
  con el mismo esquema tipado.
- Tests de validación: paridad exacta de claves, valores no vacíos, mismos placeholders,
  y detección de claves huérfanas. El build falla si falta una traducción.
- Detección: `navigator.language` / `sys-locale`, fallback `es`; selector manual persistido
  y override opcional `?lang=en`.
- Actualizar `<html lang>`, título y etiquetas ARIA al cambiar de idioma.
- Números: enteros con separador de miles según locale; decimales con coma/punto cuando
  aplique. Los dígitos del soroban son 0–9 (no se soportan dígitos arábigo-índicos en 1.0).
- El contenido pedagógico también se localiza; la lógica matemática no se duplica.


---

## ADR-007 · Fase 5 sin cápsula Android ni háptico

Estado: **ACEPTADO** (2026-11-14, decisión del usuario).

### Contexto
El superprompt de Fase 5 incluye «cápsula Android (Tauri) y háptico». El usuario aclaró que
el producto se entrega como **web (GitHub Pages/PWA) o instalable de escritorio**, no como
programa Android, y que **no quiere háptico por el momento**. En este entorno tampoco hay
SDK/NDK de Android ni teléfono, así que esos criterios no podrían verificarse de verdad.

### Decisión
- La Fase 5 **no** incluye app Android, toolchain Android, emulador ni APK.
- La Fase 5 **no** incluye háptico; queda diferido, no descartado para siempre.
- El alcance de la fase pasa a: accesibilidad avanzada, fotosensibilidad, web móvil digna y
  presupuesto de rendimiento (carga/CPU; batería móvil **NO VERIFICADA** sin dispositivo).
- `docs/PLAN-FASE-5.md` refleja este alcance y el desglose P5-001…P5-005.

### Consecuencias
- No se instalan SDK/NDK ni se añaden dependencias móviles.
- El superprompt no se modifica (solo lectura); esta ADR documenta la desviación.
- Queda pendiente para una fase futura, si el usuario lo pide, la cápsula Android con su
  háptico y las mediciones en dispositivo real.

---

## ADR-008 · La multiplicación es una sección central del producto

Estado: **ACEPTADO** (2026-11-14, requisito del usuario). Implementación pendiente.

### Contexto
El usuario aclaró el propósito real de la app: «La intención de esta app es enseñar a la
persona a multiplicar como lo hacen los prodigios chinos, así que es esencial que la app
incluya una sección donde enseña los métodos completos para esto». Hasta hoy la app sólo
entrena sumas (soroban libre y Flash Anzan); no existe multiplicación en `core`, `engine`,
`ui` ni en el contenido.

### Decisión
- La multiplicación deja de ser una mejora opcional: es **requisito del producto** y debe
  existir antes del release 1.0, con lecciones completas y práctica.
- El método enseñado es la **空盘前乘法** (multiplicación frontal con ábaco vacío):
  multiplicar desde la cifra más alta, tabla 大九九, cada producto de un dígito registrado
  con dos cifras y sumado por valor de lugar.
- Progresión M1…M7 (1×1, 2×1, 1×2, 2×2, 3×2, 3×3 y mental 2×2).
- Los atajos (十几乘十几, 头同尾合十, 尾同头合十, 补数乘法) son una unidad **avanzada**
  posterior al método general, con su condición de uso explícita.
- La especificación pedagógica, los ejemplos resueltos, la verificación y el desglose de
  tareas M-001…M-007 están en `docs/METODO-MULTIPLICACION.md`.
- La sección vive en la misma página Leptos, con modos **Aprender** y **Practicar**, visor
  de varillas de sólo lectura y progreso propio (sin mezclar con las sesiones de suma).

### Consecuencias
- La app necesita un motor de multiplicación en `engine` con property tests (≥10 000 casos)
  antes de cualquier UI.
- El contenido es/en crece con lecciones y ejemplos verificados por test, no a mano.
- Se añaden claves de persistencia versionadas para el progreso de multiplicación.
- El orden frente a la Fase 5 (accesibilidad/rendimiento) queda a decisión del usuario; la
  recomendación es hacer la multiplicación primero por ser el propósito central.

---

## ADR-009 · Un solo acento cambiable, tema claro/oscuro y nombre MathMaster

Estado: **ACEPTADO** (2026-11-16, requisito del usuario). Implementado en `5f2383d` y
revisado en `7a88b96` (acento único y tema).

### Contexto
La interfaz usaba un solo azul para el fondo y los controles. El usuario pidió «que los
botones y cosas sean del mismo color y el fondo y los botones dependan de una paleta de 10
accent color con diferencia de hue», que la app se llame **MathMaster** y no AnzanLab, y
más tarde aclaró que el acento debe ser **uno solo** y poder cambiarse, con el interruptor
de tema claro/oscuro junto a las muestras.

### Decisión
- Se define una paleta de **diez acentos** (`--anzan-accent-1`…`-10`) con hues distintos,
  cada uno con su variante clara para tema oscuro y un `--anzan-on-accent` que conserva el
  contraste.
- El acento es **global**: la cabecera muestra diez muestras y la elegida se inyecta como
  `--anzan-accent` en `.anzan-root`. Todos los controles (botones, tarjetas activas,
  tablas, medidor, campos, ábaco) y el fondo usan ese mismo acento; no hay overrides por
  sección.
- El fondo deriva del acento elegido con dos resplandores y una mezcla del 16 % en tema
  claro y del 26 % en oscuro, para que el cambio se perciba sin competir con el contenido.
- El acento se guarda en `anzan.accent` y se normaliza al cargar; un valor fuera de rango
  vuelve al primero.
- El tema claro/oscuro se elige con un interruptor (`role="switch"`) junto a las muestras,
  se guarda en `anzan.theme` y se aplica como `data-theme` en `<html>`; el CSS define los
  tokens en `:root[data-theme="dark"]` y `:root[data-theme="light"]`. Sin valor guardado se
  sigue la preferencia del sistema.
- El nombre visible pasa a **MathMaster** en textos, título, manifest, icono y host Tauri.
  Los nombres internos de crates (`anzan-*`) y las claves `anzan.*` se conservan para no
  romper la persistencia ni el empaquetado; son identificadores, no marca visible.
- No hay telemetría visible: el motor conserva `FrameTelemetry` para planificar frames,
  pero la UI no muestra etiquetas de frame, aparición ni audio nativo.

### Consecuencias
- Cambiar de acento o de tema es una sola señal y dos claves de persistencia.
- El contraste de texto sobre acento debe comprobarse al cambiar cualquier color; en tema
  claro el texto es blanco (mínimo medido 4,99:1 en los diez acentos) y en oscuro es tinta
  oscura (mínimo 8,63:1).
- El service worker sube a `mathmaster-v11`; las cachés anteriores se borran al activarse.
- La documentación vigente usa MathMaster; los informes y mediciones históricas conservan
  AnzanLab y los nombres de artefactos que realmente se generaron.
