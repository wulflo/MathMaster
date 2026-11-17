// SPDX-License-Identifier: AGPL-3.0-or-later
//! P1-021: contadores visibles de la batería de propiedades del ábaco.
//!
//! Además de los `proptest` de los módulos, este test recorre 10.000 casos
//! deterministas por categoría con un generador congruencial lineal. Imprime el
//! conteo real para que quede visible con `cargo test -- --nocapture`.
//!
//! Categorías:
//! 1. `value()` contra el valor de referencia y varillas válidas.
//! 2. Secuencias de toques que preservan la invariante.
//! 3. Planes de suma que reproducen el resultado.
//! 4. Planes de resta que reproducen el resultado.

use anzan_core::{Plan, Slot, Soroban, SorobanError, Touch};

const CASES: usize = 10_000;
const RODS: usize = 4;
const LIMIT: u128 = 10_000;

struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn below(&mut self, limit: u64) -> u64 {
        self.next() % limit
    }

    fn slot(&mut self) -> Slot {
        match self.below(5) {
            0 => Slot::Heaven,
            other => Slot::Earth(other as u8),
        }
    }
}

fn apply(soroban: &Soroban, plan: &Plan) -> Result<Soroban, SorobanError> {
    let mut copy = soroban.clone();
    for step in plan.steps() {
        copy.touch(step.touch)?;
    }
    Ok(copy)
}

fn assert_invariants(soroban: &Soroban) {
    assert!(soroban.value() < LIMIT, "valor fuera de rango");
    for index in 0..RODS {
        let rod = soroban.rod(index);
        assert!(rod.is_some(), "falta la varilla {index}");
        if let Some(rod) = rod {
            assert!(rod.earth() <= 4, "más de cuatro fichas terrestres");
            assert_eq!(rod.value(), rod.digit().value());
        }
    }
}

#[test]
fn p1_021_ten_thousand_cases_per_category() -> Result<(), SorobanError> {
    let mut rng = Lcg::new(0x0210_2026_1113_0001);
    let mut values = 0usize;
    let mut touches = 0usize;
    let mut adds = 0usize;
    let mut subs = 0usize;

    for _ in 0..CASES {
        let value = u128::from(rng.below(LIMIT as u64));
        let soroban = Soroban::from_value(value, RODS)?;

        // 1. value() contra la referencia y varillas válidas.
        assert_eq!(soroban.value(), value);
        assert_invariants(&soroban);
        values += 1;

        // 2. Secuencias de toques.
        let mut touched = soroban.clone();
        for _ in 0..10 {
            let rod = rng.below(RODS as u64) as usize;
            let slot = rng.slot();
            touched.touch(Touch::new(rod, slot))?;
            assert_invariants(&touched);
        }
        touches += 1;

        // 3. Suma que cabe en 4 varillas.
        // `below(LIMIT - value)` deja el resultado en 0..=LIMIT-1.
        let addend = u128::from(rng.below((LIMIT - value) as u64));
        let plan = soroban.plan_add(addend);
        assert!(plan.is_ok(), "la suma debería caber");
        if let Ok(plan) = plan {
            let result = apply(&soroban, &plan)?;
            assert_eq!(result.value(), value + addend);
            adds += 1;
        }

        // 4. Resta no negativa.
        let subtrahend = u128::from(rng.below(value as u64 + 1));
        let plan = soroban.plan_sub(subtrahend);
        assert!(plan.is_ok(), "la resta debería ser no negativa");
        if let Ok(plan) = plan {
            let result = apply(&soroban, &plan)?;
            assert_eq!(result.value(), value - subtrahend);
            subs += 1;
        }
    }

    let total = values + touches + adds + subs;
    println!(
        "P1-021: {values} valores, {touches} secuencias de toques, {adds} sumas, {subs} restas ({total} casos)"
    );

    assert_eq!(values, CASES);
    assert_eq!(touches, CASES);
    assert_eq!(adds, CASES);
    assert_eq!(subs, CASES);
    assert_eq!(total, CASES * 4);
    Ok(())
}
