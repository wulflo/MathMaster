# DISEÑO — Material Aero de MathMaster

Sistema de diseño de la UI: tokens de **Material 3** + una capa **Aero** de superficies
translúcidas. Implementación: `crates/ui/assets/style.css`. Decisiones: ADR-005.

## Principios

1. **Legibilidad primero.** El brillo y el blur nunca reducen el contraste por debajo de AA.
2. **Sin destellos de pantalla completa.** No se anima la luminancia del fondo ni se hacen
   flashes; el fading del soroban ocurre sólo sobre las capas del ábaco.
3. **Temas claros y oscuros.** `auto` sigue `prefers-color-scheme`; el override manual se
   persiste y se aplica con `data-theme` en `<html>`.
4. **Accesibilidad.** Foco visible, navegación por teclado, etiquetas ARIA y respeto por
   `prefers-reduced-motion`, `prefers-reduced-transparency` y `prefers-contrast`.
5. **Rendimiento.** El blur se limita a pocas superficies y no se anima. Hay fallback opaco
   si `backdrop-filter` no está disponible.

## Tokens

### Color (tema claro)

| Token | Valor | Uso |
|---|---|---|
| `--anzan-bg-1` | `#eaf3ff` | Fondo base |
| `--anzan-bg-2` | `#dbe8ff` | Fondo medio del gradiente |
| `--anzan-bg-3` | `#f7fbff` | Fondo alto |
| `--anzan-text` | `#0b1b33` | Texto principal |
| `--anzan-muted` | `#43566f` | Texto secundario |
| `--anzan-surface` | `rgba(255,255,255,0.62)` | Superficie Aero |
| `--anzan-surface-strong` | `rgba(255,255,255,0.84)` | Superficie opaca/fallback |
| `--anzan-bead-top` | `#f9fcff` | Ficha inactiva (arriba) |
| `--anzan-bead-bottom` | `#c6d8f2` | Ficha inactiva (abajo) |
| `--anzan-bead-edge` | `rgba(11,27,51,0.22)` | Borde de ficha |
| `--anzan-border` | `color-mix(in srgb, var(--anzan-accent) 32%, transparent)` | Borde y separadores (tinte del acento) |
| `--anzan-accent` | `#1f5fd6` | Acento y enlaces |
| `--anzan-on-accent` | `#ffffff` | Texto sobre acento |
| `--anzan-glow` | `rgba(94,160,255,0.35)` | Halo del fondo |

### Color (tema oscuro)

| Token | Valor | Uso |
|---|---|---|
| `--anzan-bg-1` | `#050b14` | Fondo base |
| `--anzan-bg-2` | `#0a1830` | Fondo medio |
| `--anzan-bg-3` | `#101f3a` | Fondo alto |
| `--anzan-text` | `#eaf2ff` | Texto principal |
| `--anzan-muted` | `#a9bcd8` | Texto secundario |
| `--anzan-surface` | `rgba(16,30,52,0.62)` | Superficie Aero |
| `--anzan-surface-strong` | `rgba(18,34,58,0.84)` | Superficie opaca/fallback |
| `--anzan-bead-top` | `#5b769c` | Ficha inactiva (arriba) |
| `--anzan-bead-bottom` | `#29405f` | Ficha inactiva (abajo) |
| `--anzan-bead-edge` | `rgba(190,216,255,0.4)` | Borde de ficha |
| `--anzan-border` | `color-mix(in srgb, var(--anzan-accent) 26%, transparent)` | Borde y separadores (tinte del acento) |
| `--anzan-accent` | `#7db2ff` | Acento y enlaces |
| `--anzan-on-accent` | `#050b14` | Texto sobre acento |
| `--anzan-glow` | `rgba(64,130,255,0.28)` | Halo del fondo |

### Tipografía

- UI: `Inter, "Segoe UI", system-ui, -apple-system, sans-serif`.
- Números del soroban: `font-variant-numeric: tabular-nums`.
- Títulos: `clamp(2.6rem, 7vw, 4.5rem)`, `letter-spacing: -0.04em`.

### Forma y elevación

- Radios: tarjeta `2rem`, controles `0.75rem`, píldoras `999px`.
- Sombra de tarjeta: `0 24px 80px` con tinte azul (claro) o negro (oscuro).
- Borde interior claro de 1 px para el efecto Aero.

### Movimiento

- Transiciones cortas (`120–180 ms`) y sólo en color/opacidad.
- `prefers-reduced-motion: reduce` desactiva animaciones y transiciones.
- El fading del ábaco usa opacidades discretas por etapa; no hay animación de fondo.

## Capas del soroban

| Etapa | Fichas | Varillas | Marcas de unidad |
|---|---|---|---|
| `Full` | 1.00 | 1.00 | 1.00 |
| `Ghost` | 0.20 | 0.85 | 1.00 |
| `Rods` | 0.00 | 0.85 | 1.00 |
| `UnitMarks` | 0.00 | 0.00 | 1.00 |
| `Blank` | 0.00 | 0.00 | 0.00 |

La geometría de impacto (`hit_test`) es **independiente de la etapa**: la misma coordenada
produce el mismo `Touch` en las cinco etapas, incluida `Blank`.

## Contraste verificado (WCAG 2.1)

Calculado sobre los colores sólidos de referencia (el peor caso de las superficies
translúcidas es el fondo; las superficies claras/opacas sólo mejoran el contraste):

| Combinación | Ratio | AA texto (4.5:1) | AA UI (3:1) |
|---|---|---|---|
| Claro: texto `#0b1b33` / fondo `#eaf3ff` | 15.39:1 | Sí | Sí |
| Claro: texto `#0b1b33` / blanco | 17.23:1 | Sí | Sí |
| Claro: muted `#43566f` / `#eaf3ff` | 6.70:1 | Sí | Sí |
| Claro: acento `#1f5fd6` / `#eaf3ff` | 5.12:1 | Sí | Sí |
| Oscuro: texto `#eaf2ff` / `#050b14` | 17.51:1 | Sí | Sí |
| Oscuro: muted `#a9bcd8` / `#050b14` | 10.21:1 | Sí | Sí |
| Oscuro: acento `#7db2ff` / `#050b14` | 9.10:1 | Sí | Sí |
| Claro: texto sobre acento `#ffffff` / `#1f5fd6` | 5.73:1 | Sí | Sí |
| Oscuro: texto sobre acento `#050b14` / `#7db2ff` | 9.10:1 | Sí | Sí |

## Accesibilidad y fotosensibilidad

- `:focus-visible` con contorno de 2 px y offset de 3 px en enlaces y controles.
- El soroban se opera con botones (teclado y puntero), no sólo con gestos.
- `aria-pressed` refleja el estado de cada ficha; el valor se anuncia en una región
  `aria-live="polite"` cuando la lectura está activa.
- `prefers-reduced-transparency` elimina `backdrop-filter` y usa superficie opaca.
- `prefers-reduced-motion` elimina animaciones y transiciones.
- No hay flashes de pantalla completa: el fading sólo afecta a las capas del ábaco.

## Rendimiento

- Una sola tarjeta con blur; los controles no usan blur.
- Sin animación de `backdrop-filter`.
- El render del soroban es estático por estado: se actualizan clases/atributos al tocar,
  no se asignan buffers por frame ni se ejecuta un bucle de dibujo continuo.

## Pendiente / NO VERIFICADO

- Capturas en navegadores reales (Chrome/Firefox/Safari) y en Android: **NO VERIFICADO**.
- Contraste real sobre superficies translúcidas compuestas: se usa el peor caso sólido;
  falta medición con herramienta sobre captura.
- Comportamiento de `prefers-reduced-transparency` en Safari: **NO VERIFICADO**.
