// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fading del soroban: opacidades por capa y etapa.
//!
//! El fading es una propiedad visual, no lógica: el estado del ábaco y la geometría de
//! impacto no cambian al pasar de una etapa a otra. La UI interpola entre perfiles con
//! [`lerp_profile`] o usa los valores discretos de [`FadeProfile::for_stage`].

/// Etapa de visibilidad del soroban, de más a menos información.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FadeStage {
    /// Todo visible: fichas, varillas y marcas de unidad.
    Full,
    /// Fichas «fantasma»: se intuye el estado sin mostrarlo con claridad.
    Ghost,
    /// Sólo varillas y marcas de unidad.
    Rods,
    /// Sólo marcas de unidad.
    UnitMarks,
    /// Nada visible.
    Blank,
}

impl FadeStage {
    /// Todas las etapas, en orden de desvanecimiento.
    pub const ALL: [Self; 5] = [
        Self::Full,
        Self::Ghost,
        Self::Rods,
        Self::UnitMarks,
        Self::Blank,
    ];

    /// Posición en [`FadeStage::ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Full => 0,
            Self::Ghost => 1,
            Self::Rods => 2,
            Self::UnitMarks => 3,
            Self::Blank => 4,
        }
    }

    /// Etapa siguiente; en `Blank` se queda en `Blank`.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Full => Self::Ghost,
            Self::Ghost => Self::Rods,
            Self::Rods => Self::UnitMarks,
            Self::UnitMarks | Self::Blank => Self::Blank,
        }
    }

    /// Etapa anterior; en `Full` se queda en `Full`.
    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Full | Self::Ghost => Self::Full,
            Self::Rods => Self::Ghost,
            Self::UnitMarks => Self::Rods,
            Self::Blank => Self::UnitMarks,
        }
    }

    /// Atajo booleano: ¿la etapa oculta las fichas?
    #[must_use]
    pub const fn hides_beads(self) -> bool {
        matches!(self, Self::Rods | Self::UnitMarks | Self::Blank)
    }
}

/// Capa visual del soroban.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FadeLayer {
    /// Fichas celestiales y terrestres.
    Beads,
    /// Varillas verticales.
    Rods,
    /// Marcas de unidad.
    UnitMarks,
}

/// Opacidades de las tres capas para una etapa.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadeProfile {
    /// Opacidad de las fichas (0..=1).
    pub beads: f32,
    /// Opacidad de las varillas (0..=1).
    pub rods: f32,
    /// Opacidad de las marcas de unidad (0..=1).
    pub unit_marks: f32,
}

impl FadeProfile {
    /// Perfil discreto de una etapa.
    #[must_use]
    pub const fn for_stage(stage: FadeStage) -> Self {
        match stage {
            FadeStage::Full => Self {
                beads: 1.0,
                rods: 1.0,
                unit_marks: 1.0,
            },
            FadeStage::Ghost => Self {
                beads: 0.20,
                rods: 0.85,
                unit_marks: 1.0,
            },
            FadeStage::Rods => Self {
                beads: 0.0,
                rods: 0.85,
                unit_marks: 1.0,
            },
            FadeStage::UnitMarks => Self {
                beads: 0.0,
                rods: 0.0,
                unit_marks: 1.0,
            },
            FadeStage::Blank => Self {
                beads: 0.0,
                rods: 0.0,
                unit_marks: 0.0,
            },
        }
    }

    /// Opacidad de una capa.
    #[must_use]
    pub const fn opacity(self, layer: FadeLayer) -> f32 {
        match layer {
            FadeLayer::Beads => self.beads,
            FadeLayer::Rods => self.rods,
            FadeLayer::UnitMarks => self.unit_marks,
        }
    }

    /// Interpola linealmente entre dos perfiles.
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            beads: lerp(self.beads, other.beads, t),
            rods: lerp(self.rods, other.rods, t),
            unit_marks: lerp(self.unit_marks, other.unit_marks, t),
        }
    }
}

/// Interpolación lineal con `t` acotado a 0..=1.
#[must_use]
pub fn lerp(from: f32, to: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    from + (to - from) * t
}

/// Interpola dos perfiles con `t` acotado.
#[must_use]
pub fn lerp_profile(from: FadeProfile, to: FadeProfile, t: f32) -> FadeProfile {
    from.lerp(to, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-6;

    #[test]
    fn every_stage_has_an_opacity_in_range() {
        for stage in FadeStage::ALL {
            let profile = FadeProfile::for_stage(stage);
            for layer in [FadeLayer::Beads, FadeLayer::Rods, FadeLayer::UnitMarks] {
                let opacity = profile.opacity(layer);
                assert!(
                    (0.0..=1.0).contains(&opacity),
                    "{stage:?}/{layer:?} fuera de rango: {opacity}"
                );
            }
        }
    }

    #[test]
    fn opacities_are_monotonic_across_stages() {
        for layer in [FadeLayer::Beads, FadeLayer::Rods, FadeLayer::UnitMarks] {
            let mut previous = 1.0;
            for stage in FadeStage::ALL {
                let opacity = FadeProfile::for_stage(stage).opacity(layer);
                assert!(
                    opacity <= previous + EPSILON,
                    "{layer:?} sube en {stage:?}: {opacity} > {previous}"
                );
                previous = opacity;
            }
        }
    }

    #[test]
    fn full_shows_everything_and_blank_hides_everything() {
        let full = FadeProfile::for_stage(FadeStage::Full);
        assert_eq!(full.beads, 1.0);
        assert_eq!(full.rods, 1.0);
        assert_eq!(full.unit_marks, 1.0);

        let blank = FadeProfile::for_stage(FadeStage::Blank);
        assert_eq!(blank.beads, 0.0);
        assert_eq!(blank.rods, 0.0);
        assert_eq!(blank.unit_marks, 0.0);
    }

    #[test]
    fn stage_navigation_saturates() {
        assert_eq!(FadeStage::Full.previous(), FadeStage::Full);
        assert_eq!(FadeStage::Blank.next(), FadeStage::Blank);
        assert_eq!(FadeStage::Full.next(), FadeStage::Ghost);
        assert_eq!(FadeStage::Blank.previous(), FadeStage::UnitMarks);
    }

    #[test]
    fn lerp_clamps_the_parameter() {
        assert!((lerp(2.0, 4.0, -1.0) - 2.0).abs() < EPSILON);
        assert!((lerp(2.0, 4.0, 0.0) - 2.0).abs() < EPSILON);
        assert!((lerp(2.0, 4.0, 0.5) - 3.0).abs() < EPSILON);
        assert!((lerp(2.0, 4.0, 1.0) - 4.0).abs() < EPSILON);
        assert!((lerp(2.0, 4.0, 2.0) - 4.0).abs() < EPSILON);
    }

    #[test]
    fn profile_lerp_interpolates_each_layer() {
        let from = FadeProfile::for_stage(FadeStage::Full);
        let to = FadeProfile::for_stage(FadeStage::Blank);
        let middle = lerp_profile(from, to, 0.5);
        assert!((middle.beads - 0.5).abs() < EPSILON);
        assert!((middle.rods - 0.5).abs() < EPSILON);
        assert!((middle.unit_marks - 0.5).abs() < EPSILON);
    }
}
