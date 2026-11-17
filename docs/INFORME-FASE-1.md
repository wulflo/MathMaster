# INFORME DE FASE 1 — Setup + Core del Soroban + Fading

- **Fecha:** 2026-11-13
- **Base verificada:** `b606beb` (P1-022)
- **Stack:** Tauri 2.12.5 + Leptos 0.8.23 (CSR/wasm) · licencia **AGPL-3.0-or-later**
- **Estado:** Fase 1 cerrada en lo técnico (P1-001…P1-023). Quedan dos puntos fuera del
  cierre: P1-000 `[!]` y P1-024 `[~]`.

## 1. Estado por bloque

| Bloque | Tareas | Estado |
|---|---|---|
| 1.0 Protocolo documental | P1-000 | `[!]` falta el superprompt original |
| 1.1 Spike de stack | P1-001…P1-006 | `[x]` |
| 1.2 Workspace y CI | P1-007…P1-010 | `[x]` `62bf826` |
| 1.3 Core del soroban | P1-011…P1-015 | `[x]` `d110290`…`7d3458c` |
| 1.4 Fading y geometría | P1-016…P1-018 | `[x]` `d84afe0`…`09aa0d9` |
| 1.5 UI mínima | P1-019…P1-020 | `[x]` `b9ac34a` |
| 1.6 Transversales | P1-024…P1-027 | `[x]` (P1-024 `0c17d0b`; el resto `62bf826`) |
| Cierre | P1-021…P1-023 | `[x]` `64a6144`, `[x]`, este informe |

## 2. Verificación ejecutada (números reales)

### CI

```text
./ci.sh
== fmt                          OK
== construcciones prohibidas    OK (incluye tests)
== clippy -D warnings           OK
== tests                        OK
== regla de dependencias        OK
== cargo deny check             advisories, bans, licenses y sources OK
== check wasm                   OK
CI OK
```

### Tests

`cargo test --workspace`: **52 tests, 0 fallos**.

| Crate | Tests |
|---|---:|
| `anzan-core` (unitarios) | 45 |
| `anzan-core` (`property_budget`) | 1 |
| `anzan-engine` | 4 |
| `anzan-content` | 2 |
| `anzan-ui` / `anzan-app` | 0 |

Casos de propiedades:

- 7 propiedades `proptest` × 10.000 casos = **70.000** casos generados
  (5 de `soroban`, 2 de `plan`).
- Test determinista P1-021: **40.000** casos (10.000 por categoría).
- Total: **110.000** casos.
- Contador visible con `cargo test -p anzan-core --test property_budget -- --nocapture`:

```text
P1-021: 10000 valores, 10000 secuencias de toques, 10000 sumas, 10000 restas (40000 casos)
```

### Web / wasm

- `cargo check -p anzan-ui -p anzan-app --target wasm32-unknown-unknown`: OK.
- `./scripts/build-web.sh`: OK; `anzan_app_bg.wasm` ≈ **622 KB** (release).
- Servido localmente en Chromium (navegador de la app) sobre `dist/`:
  - 40 fichas (8 varillas × 5) montadas;
  - tocar 5 + 3 ⇒ valor **8**; reinicio ⇒ **0**;
  - mostrar/ocultar valor;
  - etapas: Ghost `0.20/0.85/1`, Rods `0/0.85/1`, Blank `0/0/0`;
  - en Blank las fichas siguen aceptando toques (valor 5), conforme a P1-018;
  - teclado: foco + Enter toca la ficha;
  - cambio es/en con `<html lang="en">` y textos traducidos;
  - **offline verificado**: con el servidor apagado, el service worker sirve la app y
    monta las 40 fichas.
- Service worker corregido: navegación *network-first* con respaldo en caché y caída a
  caché también con `status >= 500`.

### Accesibilidad / diseño

- Contrastes AA calculados en `docs/DISENO.md` (texto 15.39:1 claro / 17.51:1 oscuro;
  muted 6.70:1 / 10.21:1; acento 5.12:1 / 9.10:1).
- `prefers-reduced-motion`, `prefers-reduced-transparency` y `:focus-visible` presentes.
- Sin destellos de pantalla completa: el fading sólo afecta a las capas del ábaco.

## 3. NO VERIFICADO

- Firefox, Safari y móviles reales (Android/iOS): **NO VERIFICADO**.
- Toque táctil real: sólo se probaron ratón y eventos de puntero sintéticos.
- Tiempo de pintura del feedback: la clase `is-active`/`is-pressed` se aplica en el
  microtask posterior a `pointerdown` (antes del siguiente pintado por especificación),
  pero **no se midió** con telemetría de frames.
- Rendimiento de la UI: no hay presupuesto numérico en Fase 1 ni medición de allocations
  con instrumentación; sólo se verifica que no existe bucle de dibujo.
- Audio/háptico: fuera del alcance de Fase 1.
- Corte de red real sobre GitHub Pages: no se pudo forzar con las herramientas
  disponibles. El precache v5 está completo y el offline se verificó en local con el
  mismo build.

## 4. Bloqueos y decisiones pendientes

1. **P1-000 `[!]`** — no existe `docs/SUPERPROMPT.md` porque el superprompt original no se
   aportó. Sin él no se conoce el plan de Fase 2. No se inventó ni se resumió.
2. **P1-024 `[x]`** — publicado y verificado el 2026-11-13 (ver sección 6).

## 5. Propuesta para continuar

1. Aportar el superprompt original para crear `docs/SUPERPROMPT.md` y definir las tareas
   de Fase 2.
2. No se inicia Fase 2 hasta recibir el «sí» explícito de la compuerta.

## 6. Publicación (P1-024)

- `main` publicado: `cdcc49d..0c17d0b`; CI y Pages en verde (runs `43528152607` y
  `43528152665`, y de nuevo `43528347287`/`43528347148` tras los arreglos).
- URL: <https://wulflo.github.io/MathMaster/> — 40 fichas, valor/reset/fading y es/en.
- Service worker **v5**: navegación y assets con red primero y respaldo en caché;
  `no-cache`/`reload` para evitar el wasm viejo por el `max-age` de Pages.
- Lighthouse sobre la URL real: **Accesibilidad 1.00 · Best Practices 1.00 · SEO 1.00**,
  sin fallos.
- Correcciones durante la publicación: `8821289` (contraste del botón de etapa activo),
  `09a1881` (red primero), `0c17d0b` (revalidación de assets).
