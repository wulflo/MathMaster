# PLAN — Fase 5: accesibilidad, web móvil y rendimiento

Revisión hecha el **2026-11-14** y ajustada el mismo día por decisión del usuario:
**sin cápsula Android y sin háptico por el momento**. La Fase 5 se concentra en la web
publicada (GitHub Pages/PWA) y en el instalable de escritorio ya existente.

> **Nota de alcance.** El superprompt lista «cápsula Android (Tauri) y háptico» en Fase 5.
> El usuario decidió el 2026-11-14 no incluirlos: el producto se entrega como web o como
> instalable, no como app Android. La decisión queda registrada en `docs/DECISIONES.md`
> (ADR-007); el superprompt no se modifica por ser de solo lectura.

## Alcance vigente

1. **Auditoría de accesibilidad** de la UI compartida: teclado, foco, ARIA, contraste,
   `prefers-reduced-motion`/`transparency`/`contrast` y fotosensibilidad.
2. **Web móvil digna:** viewport, breakpoints, objetivos táctiles, aviso claro de límites y
   degradación sin prometer precisión que el navegador no puede dar.
3. **Presupuesto de rendimiento:** tamaño del wasm, primera carga con red limitada y CPU en
   reposo del escritorio. Batería en móvil real queda **NO VERIFICADA** sin dispositivo.

## Qué ya existe

| Área | Estado real hoy |
|---|---|
| UI compartida | Un solo wasm Leptos para web y escritorio; la web publicada sirve el build de Fase 4. |
| Accesibilidad | `aria-live` en valor/estado, `aria-pressed` por ficha, `role="group"`, foco visible, `prefers-reduced-motion`/`transparency`, claro/oscuro y tabla de contraste AA en `docs/DISENO.md`. |
| Fotosensibilidad | Sin flashes de pantalla completa; el fading afecta sólo a las capas del ábaco. Frecuencia y área de cambio no medidas formalmente. |
| Rendimiento web | wasm de Fase 4: **1 230 769 B** en el runner, **263 794 B gzip** (13 % del presupuesto de 2 MB). Primera carga en 4G y CPU no medidas. |
| Responsive | Sin `@media` por ancho; sólo preferencias del sistema. No probado en viewport móvil ni con puntero táctil. |
| Android / háptico | **Fuera de alcance** por decisión del usuario. No se instala SDK/NDK ni se implementa háptico. |

## Evaluación

### Accesibilidad — buena base, faltan pruebas y un modo

- **Cubierto:** foco visible, etiquetas, regiones vivas, operación por botones, movimiento y
  transparencia reducidos, contraste AA calculado.
- **Falta:** `prefers-contrast` (el superprompt pide «modo de contraste reducido» y hoy no
  existe), auditoría con teclado real (orden de tabulación, foco atrapado o perdido, atajos),
  prueba con lector de pantalla y comprobación automatizada de etiquetas.
- **Lector de pantalla:** no hay `orca` instalado y no se instalará sin autorización; la
  prueba manual queda **NO VERIFICADA** con instrucciones para el usuario.
- **Fotosensibilidad:** hay que medir frecuencia y área de cambio de los dígitos del drill y
  documentarlo; no se espera riesgo, pero no está demostrado.

### Web móvil — trabajo real por hacer

- No hay breakpoints por ancho ni adaptación a puntero grueso; el ábaco y los paneles pueden
  quedar apretados en pantallas pequeñas.
- Hay que definir qué se degrada (tamaño del ábaco, densidad, animaciones, audio) y mostrar
  un aviso cuando la precisión temporal no esté garantizada.
- La verificación se hará con viewports emulados en Chrome/Firefox de escritorio; un móvil
  real sigue **NO VERIFICADO**.

### Rendimiento — medible a medias

- **Se puede medir:** tamaño del wasm, primera carga con throttling de red, tiempo hasta
  interactivo local, CPU en reposo del escritorio, tamaño del DOM y coste de render.
- **No se puede medir aquí:** batería y CPU en un móvil real; latencia táctil y audio móvil.
- El wasm ya cumple el presupuesto de 2 MB gzip; falta fijar el presupuesto de primera carga
  y dejar registro para detectar crecimientos.

## Riesgos y decisiones

1. **Lector de pantalla:** sin `orca`/NVDA no se puede certificar la experiencia real; se
   entregará protocolo de prueba y se declarará **NO VERIFICADO**.
2. **Móvil real:** no hay dispositivo; las pruebas responsive serán emuladas y así se
   declarará.
3. **Batería:** no se puede medir sin móvil; se mide CPU en escritorio y se documenta el
   límite.
4. **Publicación:** la release de bundles y la firma siguen sin autorización.

## Desglose propuesto (a registrar en el CHECKLIST al aprobar)

| ID | Tarea | Aceptación verificable |
|---|---|---|
| P5-001 | Auditoría de accesibilidad y correcciones | Teclado, foco, ARIA, contraste y `prefers-contrast` documentados y corregidos; lector de pantalla manual **NO VERIFICADO** con protocolo |
| P5-002 | Fotosensibilidad del drill | Frecuencia y área de cambio medidas/documentadas; sin destellos >3 Hz ni pantalla completa |
| P5-003 | Web móvil digna | Breakpoints, objetivos táctiles, viewport y aviso de límites verificados en viewports emulados; móvil real **NO VERIFICADO** |
| P5-004 | Presupuesto de rendimiento | wasm <2 MB gzip, primera carga medida con throttling, CPU en reposo y registro en `docs/MEDICIONES.md`; batería móvil **NO VERIFICADA** |
| P5-005 | Cierre de Fase 5: tests, CI, informe y compuerta | `./ci.sh` verde, informe con números y límites, RESUME/LOG/CHECKLIST al día |

## Recomendación

Fase más corta y completamente ejecutable en este entorno, salvo las pruebas que exigen
hardware (lector de pantalla, móvil real, batería), que se entregarán como **NO VERIFICADO**
con pasos concretos. Si se autoriza, el primer paso es P5-001 (auditoría de accesibilidad).
