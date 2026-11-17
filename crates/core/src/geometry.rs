// SPDX-License-Identifier: AGPL-3.0-or-later
//! Geometría normalizada del soroban.
//!
//! Las coordenadas van de 0 a 1 en ambos ejes. La geometría no depende de la etapa de
//! fading: la misma coordenada produce el mismo [`Touch`] con el ábaco visible o en blanco.

use crate::{FadeStage, Slot, Touch};

/// Error al construir una geometría.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GeometryError {
    /// La cantidad de varillas debe ser al menos 1.
    #[error("cantidad de varillas inválida: {0} (se esperaba >= 1)")]
    InvalidRodCount(usize),
}

/// Rectángulo normalizado.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Borde izquierdo (0..=1).
    pub x: f32,
    /// Borde superior (0..=1).
    pub y: f32,
    /// Ancho (0..=1).
    pub width: f32,
    /// Alto (0..=1).
    pub height: f32,
}

impl Rect {
    /// Centro del rectángulo.
    #[must_use]
    pub fn center(self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Indica si el punto cae dentro del rectángulo.
    #[must_use]
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

/// Márgenes horizontales de la geometría.
pub const PADDING_X: f32 = 0.06;
/// Márgenes verticales de la geometría.
pub const PADDING_Y: f32 = 0.06;
/// Separación entre varillas.
pub const ROD_GAP: f32 = 0.012;
/// Altura del travesaño (centro).
pub const BEAM_Y: f32 = 0.44;
/// Mitad del hueco del travesaño (sin fichas).
pub const BEAM_HALF_GAP: f32 = 0.012;

const EARTH_SLOTS: u8 = 4;

/// Geometría de un soroban de `n` varillas.
///
/// La columna visual 0 es la de mayor peso; la varilla interna 0 es la de menor peso
/// (unidades), igual que en [`crate::Soroban`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SorobanGeometry {
    rods: usize,
}

impl SorobanGeometry {
    /// Construye la geometría para `rods` varillas.
    ///
    /// # Errors
    /// Devuelve [`GeometryError::InvalidRodCount`] si `rods == 0`.
    pub const fn new(rods: usize) -> Result<Self, GeometryError> {
        if rods == 0 {
            Err(GeometryError::InvalidRodCount(rods))
        } else {
            Ok(Self { rods })
        }
    }

    /// Construye la geometría para `rods` varillas, con un mínimo de 1.
    ///
    /// Es la variante infalible que usa la UI cuando ya validó el número de varillas.
    #[must_use]
    pub const fn for_rods(rods: usize) -> Self {
        Self {
            rods: if rods == 0 { 1 } else { rods },
        }
    }

    /// Cantidad de varillas.
    #[must_use]
    pub const fn rods(self) -> usize {
        self.rods
    }

    fn rod_width(self) -> f32 {
        let area = 1.0 - 2.0 * PADDING_X;
        let gaps = ROD_GAP * (self.rods.saturating_sub(1)) as f32;
        (area - gaps) / self.rods as f32
    }

    fn step(self) -> f32 {
        self.rod_width() + ROD_GAP
    }

    fn column_x(self, column: usize) -> f32 {
        PADDING_X + column as f32 * self.step()
    }

    fn column_of(self, rod: usize) -> Option<usize> {
        if rod < self.rods {
            Some(self.rods - 1 - rod)
        } else {
            None
        }
    }

    /// Rectángulo completo de una varilla.
    #[must_use]
    pub fn rod_rect(self, rod: usize) -> Option<Rect> {
        let column = self.column_of(rod)?;
        Some(Rect {
            x: self.column_x(column),
            y: PADDING_Y,
            width: self.rod_width(),
            height: 1.0 - 2.0 * PADDING_Y,
        })
    }

    /// Rectángulo de una ficha.
    #[must_use]
    pub fn slot_rect(self, rod: usize, slot: Slot) -> Option<Rect> {
        let rod_rect = self.rod_rect(rod)?;
        match slot {
            Slot::Heaven => Some(Rect {
                x: rod_rect.x,
                y: PADDING_Y,
                width: rod_rect.width,
                height: BEAM_Y - BEAM_HALF_GAP - PADDING_Y,
            }),
            Slot::Earth(index) if (1..=EARTH_SLOTS).contains(&index) => {
                let earth_top = BEAM_Y + BEAM_HALF_GAP;
                let height = (1.0 - PADDING_Y - earth_top) / f32::from(EARTH_SLOTS);
                Some(Rect {
                    x: rod_rect.x,
                    y: earth_top + f32::from(index - 1) * height,
                    width: rod_rect.width,
                    height,
                })
            }
            Slot::Earth(_) => None,
        }
    }

    /// Convierte una coordenada normalizada en un toque.
    ///
    /// Devuelve `None` fuera del área, en el hueco del travesaño o en la separación entre
    /// varillas.
    #[must_use]
    pub fn hit_test(self, x: f32, y: f32) -> Option<Touch> {
        if !(PADDING_X..=1.0 - PADDING_X).contains(&x)
            || !(PADDING_Y..=1.0 - PADDING_Y).contains(&y)
        {
            return None;
        }

        let column = ((x - PADDING_X) / self.step()).floor() as usize;
        let column = column.min(self.rods - 1);
        let rod = self.rods - 1 - column;

        let slot = if y < BEAM_Y - BEAM_HALF_GAP {
            Slot::Heaven
        } else if y > BEAM_Y + BEAM_HALF_GAP {
            let earth_top = BEAM_Y + BEAM_HALF_GAP;
            let height = (1.0 - PADDING_Y - earth_top) / f32::from(EARTH_SLOTS);
            let index = ((y - earth_top) / height).floor() as u8 + 1;
            Slot::Earth(index.clamp(1, EARTH_SLOTS))
        } else {
            return None;
        };

        let rect = self.slot_rect(rod, slot)?;
        if rect.contains(x, y) {
            Some(Touch::new(rod, slot))
        } else {
            None
        }
    }

    /// Igual que [`Self::hit_test`], pero explícito en que la etapa no altera el impacto.
    #[must_use]
    pub fn hit_test_for_stage(self, stage: FadeStage, x: f32, y: f32) -> Option<Touch> {
        let _ = stage;
        self.hit_test(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RODS: usize = 4;

    fn geometry() -> SorobanGeometry {
        SorobanGeometry::new(RODS).unwrap_or(SorobanGeometry { rods: 1 })
    }

    fn sample_slots() -> [Slot; 5] {
        [
            Slot::Heaven,
            Slot::Earth(1),
            Slot::Earth(2),
            Slot::Earth(3),
            Slot::Earth(4),
        ]
    }

    #[test]
    fn rejects_zero_rods() {
        assert_eq!(
            SorobanGeometry::new(0),
            Err(GeometryError::InvalidRodCount(0))
        );
    }

    #[test]
    fn every_rect_is_normalized() {
        for rods in 1..=10 {
            let geometry = SorobanGeometry::new(rods);
            assert!(geometry.is_ok());
            if let Ok(geometry) = geometry {
                for rod in 0..rods {
                    for slot in sample_slots() {
                        let rect = geometry.slot_rect(rod, slot);
                        assert!(rect.is_some(), "falta el rect de {rod}/{slot:?}");
                        if let Some(rect) = rect {
                            assert!(rect.x >= 0.0 && rect.y >= 0.0);
                            assert!(rect.x + rect.width <= 1.0 + 1e-6);
                            assert!(rect.y + rect.height <= 1.0 + 1e-6);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn slot_centers_hit_their_own_slot() {
        let geometry = geometry();
        for rod in 0..RODS {
            for slot in sample_slots() {
                let rect = geometry.slot_rect(rod, slot);
                assert!(rect.is_some());
                if let Some(rect) = rect {
                    let (x, y) = rect.center();
                    assert_eq!(
                        geometry.hit_test(x, y),
                        Some(Touch::new(rod, slot)),
                        "centro de {rod}/{slot:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn hit_test_is_identical_in_all_fade_stages() {
        let geometry = geometry();
        let points = [
            (0.10, 0.10),
            (0.10, 0.30),
            (0.10, 0.60),
            (0.10, 0.90),
            (0.50, 0.10),
            (0.50, 0.60),
            (0.90, 0.30),
            (0.90, 0.90),
        ];

        for (x, y) in points {
            let expected = geometry.hit_test(x, y);
            for stage in FadeStage::ALL {
                assert_eq!(
                    geometry.hit_test_for_stage(stage, x, y),
                    expected,
                    "la etapa {stage:?} cambió el impacto en ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn outside_points_and_beam_gap_return_none() {
        let geometry = geometry();
        assert_eq!(geometry.hit_test(-0.01, 0.5), None);
        assert_eq!(geometry.hit_test(1.01, 0.5), None);
        assert_eq!(geometry.hit_test(0.5, -0.01), None);
        assert_eq!(geometry.hit_test(0.5, 1.01), None);
        assert_eq!(geometry.hit_test(0.5, BEAM_Y), None);
    }

    #[test]
    fn rod_order_is_right_to_left_units() {
        let geometry = geometry();
        let leftmost = geometry.slot_rect(RODS - 1, Slot::Heaven);
        let rightmost = geometry.slot_rect(0, Slot::Heaven);
        assert!(leftmost.is_some() && rightmost.is_some());
        if let (Some(left), Some(right)) = (leftmost, rightmost) {
            assert!(
                left.x < right.x,
                "la varilla de mayor peso va a la izquierda"
            );
            let (x, y) = left.center();
            assert_eq!(
                geometry.hit_test(x, y),
                Some(Touch::new(RODS - 1, Slot::Heaven))
            );
            let (x, y) = right.center();
            assert_eq!(geometry.hit_test(x, y), Some(Touch::new(0, Slot::Heaven)));
        }
    }

    #[test]
    fn invalid_slots_have_no_rect() {
        let geometry = geometry();
        assert_eq!(geometry.slot_rect(0, Slot::Earth(0)), None);
        assert_eq!(geometry.slot_rect(0, Slot::Earth(5)), None);
        assert_eq!(geometry.slot_rect(RODS, Slot::Heaven), None);
    }

    #[test]
    fn opacity_does_not_change_hit_testing() {
        use crate::FadeProfile;

        let geometry = geometry();
        let mut checked = 0;
        for rod in 0..RODS {
            for slot in sample_slots() {
                let rect = geometry.slot_rect(rod, slot);
                assert!(rect.is_some());
                if let Some(rect) = rect {
                    let points = [
                        rect.center(),
                        (rect.x + rect.width * 0.25, rect.y + rect.height * 0.25),
                        (rect.x + rect.width * 0.75, rect.y + rect.height * 0.75),
                    ];
                    for (x, y) in points {
                        let expected = geometry.hit_test(x, y);
                        for stage in FadeStage::ALL {
                            let profile = FadeProfile::for_stage(stage);
                            let _ = profile.beads;
                            assert_eq!(
                                geometry.hit_test_for_stage(stage, x, y),
                                expected,
                                "la opacidad de {stage:?} alteró el impacto"
                            );
                        }
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked >= RODS * 5 * 3);
    }
}
