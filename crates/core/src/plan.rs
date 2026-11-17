// SPDX-License-Identifier: AGPL-3.0-or-later
//! Planes de suma y resta sobre el soroban.
//!
//! Un plan es una secuencia de [`Touch`] ya calculada: la UI sólo la reproduce. El plan
//! distingue los casos pedagógicos del ábaco: movimiento directo, complemento a 5,
//! complemento a 10 y acarreo/préstamo en cadena.

use crate::{Soroban, Touch};

/// Clasificación pedagógica de un paso del plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepKind {
    /// Movimiento directo: no cambia el lado celestial ni cruza la decena.
    Direct,
    /// Complemento a 5: se enciende o apaga la ficha celestial y se compensa con terrestres.
    FiveComplement,
    /// Complemento a 10: la varilla cruza la decena (acarreo o préstamo).
    TenComplement,
    /// Propagación de un acarreo desde una varilla de menor peso.
    Carry,
    /// Propagación de un préstamo desde una varilla de menor peso.
    Borrow,
}

/// Un paso del plan: un toque y su clasificación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Step {
    /// Toque que debe aplicarse.
    pub touch: Touch,
    /// Clasificación pedagógica.
    pub kind: StepKind,
}

/// Plan de toques para una operación.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Plan {
    steps: Vec<Step>,
}

impl Plan {
    /// Pasos en orden de aplicación.
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Cantidad de pasos.
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Indica si el plan no tiene pasos.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// Error al planificar una operación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PlanError {
    /// El resultado supera la capacidad del soroban.
    #[error("la suma desborda la capacidad del soroban")]
    Overflow,
    /// El sustraendo es mayor que el valor actual.
    #[error("la resta daría un valor negativo")]
    Underflow,
}

impl Soroban {
    /// Planifica la suma de `addend` al valor actual.
    ///
    /// # Errors
    /// Devuelve [`PlanError::Overflow`] si el resultado no cabe en las varillas actuales.
    pub fn plan_add(&self, addend: u128) -> Result<Plan, PlanError> {
        let value = self.value();
        let limit = 10u128.pow(self.rods() as u32);
        let target = value.checked_add(addend).ok_or(PlanError::Overflow)?;
        if target >= limit {
            return Err(PlanError::Overflow);
        }

        let mut steps = Vec::new();
        let mut carry = 0u8;
        for rod in 0..self.rods() {
            let current = self.rod(rod).map_or(0, |rod| rod.value());
            let operand = digit_at(addend, rod);
            let sum = current + operand + carry;
            let result = sum % 10;
            let carry_out = sum / 10;

            let base = if carry > 0 {
                StepKind::Carry
            } else {
                classify_add(current, result, sum)
            };
            push_transition(&mut steps, rod, current, result, base);
            carry = carry_out;
        }

        Ok(Plan { steps })
    }

    /// Planifica la resta de `subtrahend` al valor actual.
    ///
    /// # Errors
    /// Devuelve [`PlanError::Underflow`] si el resultado sería negativo.
    pub fn plan_sub(&self, subtrahend: u128) -> Result<Plan, PlanError> {
        let value = self.value();
        if subtrahend > value {
            return Err(PlanError::Underflow);
        }

        let mut steps = Vec::new();
        let mut borrow = 0u8;
        for rod in 0..self.rods() {
            let current = self.rod(rod).map_or(0, |rod| rod.value());
            let operand = digit_at(subtrahend, rod);
            let subtotal = operand + borrow;
            let (result, borrow_out) = if current >= subtotal {
                (current - subtotal, 0)
            } else {
                (current + 10 - subtotal, 1)
            };

            let base = if borrow > 0 {
                StepKind::Borrow
            } else {
                classify_sub(current, result, borrow_out)
            };
            push_transition(&mut steps, rod, current, result, base);
            borrow = borrow_out;
        }

        Ok(Plan { steps })
    }
}

fn digit_at(value: u128, index: usize) -> u8 {
    let mut rest = value;
    for _ in 0..index {
        rest /= 10;
    }
    (rest % 10) as u8
}

fn classify_add(current: u8, result: u8, sum: u8) -> StepKind {
    if sum >= 10 {
        StepKind::TenComplement
    } else if current / 5 != result / 5 {
        StepKind::FiveComplement
    } else {
        StepKind::Direct
    }
}

fn classify_sub(current: u8, result: u8, borrow_out: u8) -> StepKind {
    if borrow_out > 0 {
        StepKind::TenComplement
    } else if current / 5 != result / 5 {
        StepKind::FiveComplement
    } else {
        StepKind::Direct
    }
}

fn push_transition(steps: &mut Vec<Step>, rod: usize, current: u8, result: u8, kind: StepKind) {
    let heaven_before = current >= 5;
    let earth_before = current % 5;
    let heaven_after = result >= 5;
    let earth_after = result % 5;

    let heaven = || Step {
        touch: Touch::new(rod, crate::Slot::Heaven),
        kind,
    };
    let earth = |index: u8| Step {
        touch: Touch::new(rod, crate::Slot::Earth(index)),
        kind,
    };

    if heaven_before != heaven_after {
        if heaven_after {
            steps.push(heaven());
            adjust_earth(steps, earth_before, earth_after, &earth);
        } else {
            adjust_earth(steps, earth_before, earth_after, &earth);
            steps.push(heaven());
        }
    } else {
        adjust_earth(steps, earth_before, earth_after, &earth);
    }
}

fn adjust_earth(steps: &mut Vec<Step>, before: u8, after: u8, earth: &impl Fn(u8) -> Step) {
    if after > before {
        for index in (before + 1)..=after {
            steps.push(earth(index));
        }
    } else if after < before {
        for index in ((after + 1)..=before).rev() {
            steps.push(earth(index));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Soroban;
    use proptest::prelude::*;

    const CASES: u32 = 10_000;

    fn soroban(value: u128, rods: usize) -> Result<Soroban, crate::SorobanError> {
        Soroban::from_value(value, rods)
    }

    fn add_plan(initial: u128, addend: u128) -> Option<Plan> {
        soroban(initial, 4).ok()?.plan_add(addend).ok()
    }

    fn sub_plan(initial: u128, subtrahend: u128) -> Option<Plan> {
        soroban(initial, 4).ok()?.plan_sub(subtrahend).ok()
    }

    fn apply(soroban: &Soroban, plan: &Plan) -> u128 {
        let mut copy = soroban.clone();
        for step in plan.steps() {
            let result = copy.touch(step.touch);
            assert!(result.is_ok(), "un paso del plan debe ser válido");
        }
        copy.value()
    }

    fn assert_plan(initial: u128, plan: &Plan, expected: u128) {
        let soroban = soroban(initial, 4);
        assert!(soroban.is_ok());
        if let Ok(soroban) = soroban {
            assert_eq!(apply(&soroban, plan), expected);
        }
    }

    #[test]
    fn case_999_plus_1() {
        let plan = add_plan(999, 1);
        assert!(plan.is_some());
        if let Some(plan) = plan {
            assert_plan(999, &plan, 1_000);
            assert_eq!(plan.steps()[0].kind, StepKind::TenComplement);
            assert!(
                plan.steps()
                    .iter()
                    .filter(|step| step.touch.rod > 0)
                    .all(|step| step.kind == StepKind::Carry)
            );
        }
    }

    #[test]
    fn case_1000_minus_1() {
        let plan = sub_plan(1_000, 1);
        assert!(plan.is_some());
        if let Some(plan) = plan {
            assert_plan(1_000, &plan, 999);
            assert_eq!(plan.steps()[0].kind, StepKind::TenComplement);
            assert!(
                plan.steps()
                    .iter()
                    .filter(|step| step.touch.rod > 0)
                    .all(|step| step.kind == StepKind::Borrow)
            );
        }
    }

    #[test]
    fn case_5_plus_5() {
        let plan = add_plan(5, 5);
        assert!(plan.is_some());
        if let Some(plan) = plan {
            assert_plan(5, &plan, 10);
            assert!(
                plan.steps()
                    .iter()
                    .any(|step| step.kind == StepKind::TenComplement)
            );
        }
    }

    #[test]
    fn case_4_plus_1_is_five_complement() {
        let plan = add_plan(4, 1);
        assert!(plan.is_some());
        if let Some(plan) = plan {
            assert_plan(4, &plan, 5);
            assert!(
                plan.steps()
                    .iter()
                    .all(|step| step.kind == StepKind::FiveComplement)
            );
        }
    }

    #[test]
    fn case_9_plus_1() {
        let plan = add_plan(9, 1);
        assert!(plan.is_some());
        if let Some(plan) = plan {
            assert_plan(9, &plan, 10);
        }
    }

    #[test]
    fn case_10_minus_1() {
        let plan = sub_plan(10, 1);
        assert!(plan.is_some());
        if let Some(plan) = plan {
            assert_plan(10, &plan, 9);
        }
    }

    #[test]
    fn adding_or_subtracting_zero_is_empty() {
        let soroban = soroban(123, 4);
        assert!(soroban.is_ok());
        if let Ok(soroban) = soroban {
            let add = soroban.plan_add(0);
            assert!(add.is_ok());
            if let Ok(add) = add {
                assert!(add.is_empty());
            }
            let sub = soroban.plan_sub(0);
            assert!(sub.is_ok());
            if let Ok(sub) = sub {
                assert!(sub.is_empty());
            }
        }
    }

    #[test]
    fn regression_seed_021da9b0_is_an_overflow() {
        // Semilla guardada por proptest en `proptest-regressions/plan.txt`.
        let soroban = soroban(5_361, 4);
        assert!(soroban.is_ok());
        if let Ok(soroban) = soroban {
            assert_eq!(soroban.plan_add(4_639), Err(PlanError::Overflow));
        }
    }

    #[test]
    fn overflow_and_underflow_are_rejected() {
        let maximum = soroban(9_999, 4);
        assert!(maximum.is_ok());
        if let Ok(maximum) = maximum {
            assert_eq!(maximum.plan_add(1), Err(PlanError::Overflow));
        }
        let zero = soroban(0, 4);
        assert!(zero.is_ok());
        if let Ok(zero) = zero {
            assert_eq!(zero.plan_sub(1), Err(PlanError::Underflow));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: CASES, ..ProptestConfig::default() })]

        #[test]
        fn random_additions_reproduce_the_result(initial in 0u128..10_000, addend in 0u128..10_000) {
            let soroban = soroban(initial, 4);
            prop_assert!(soroban.is_ok());
            if let Ok(soroban) = soroban {
                let plan = soroban.plan_add(addend);
                if initial + addend < 10_000 {
                    prop_assert!(plan.is_ok());
                    if let Ok(plan) = plan {
                        prop_assert_eq!(apply(&soroban, &plan), initial + addend);
                    }
                } else {
                    prop_assert_eq!(plan, Err(PlanError::Overflow));
                }
            }
        }

        #[test]
        fn random_subtractions_reproduce_the_result(initial in 0u128..10_000, subtrahend in 0u128..10_000) {
            let soroban = soroban(initial, 4);
            prop_assert!(soroban.is_ok());
            if let Ok(soroban) = soroban {
                let plan = soroban.plan_sub(subtrahend);
                if subtrahend <= initial {
                    prop_assert!(plan.is_ok());
                    if let Ok(plan) = plan {
                        prop_assert_eq!(apply(&soroban, &plan), initial - subtrahend);
                    }
                } else {
                    prop_assert_eq!(plan, Err(PlanError::Underflow));
                }
            }
        }
    }
}
