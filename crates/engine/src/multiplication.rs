// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multiplicación por 空盘前乘法 (multiplicación frontal con ábaco vacío).
//!
//! El modelo no dibuja el ábaco: produce la secuencia de productos parciales de un
//! dígito, la varilla donde cae cada uno y el acumulado tras cada paso. La UI sólo
//! reproduce esa secuencia y compara la respuesta.
//!
//! Reglas fijadas en `docs/METODO-MULTIPLICACION.md`:
//! - Se recorre el **multiplicador** de izquierda a derecha; por cada cifra suya se
//!   recorren las del **multiplicando**, también de izquierda a derecha.
//! - La varilla 0 son las unidades; cada producto de un dígito se registra siempre con
//!   dos cifras (decenas y unidades), aunque las decenas sean 0.
//! - Si `m` y `n` son las cifras de multiplicando y multiplicador, el producto de la
//!   cifra de lugar `p` del multiplicando por la de lugar `q` del multiplicador ocupa
//!   sus unidades en la varilla `p + q` y sus decenas en `p + q + 1`.

/// Dígitos máximos por factor.
pub const MAX_FACTOR_DIGITS: u8 = 3;
/// Varillas máximas del producto (`m + n`, con `m, n <= 3`).
pub const MAX_RODS: usize = 6;
/// Nivel mínimo de multiplicación.
pub const MIN_MUL_LEVEL: u8 = 1;
/// Nivel máximo de multiplicación.
pub const MAX_MUL_LEVEL: u8 = 7;

/// Potencias de diez usadas para extraer cifras sin asignar memoria.
const POW10: [u32; MAX_RODS] = [1, 10, 100, 1_000, 10_000, 100_000];

/// Cifras de un factor, de la más significativa a la menos significativa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Digits {
    values: [u8; MAX_FACTOR_DIGITS as usize],
    len: u8,
}

impl Digits {
    /// Extrae las cifras de `value`, que debe estar validado en `0..=999`.
    fn of(value: u32) -> Self {
        let len = digit_count(value);
        let mut values = [0u8; MAX_FACTOR_DIGITS as usize];
        let mut index = 0usize;
        let mut place = len;
        while place > 0 {
            place -= 1;
            if let Some(slot) = values.get_mut(index) {
                *slot = digit_at_place(value, place);
            }
            index += 1;
        }
        Self { values, len }
    }

    /// Cifras en orden de lectura.
    fn as_slice(&self) -> &[u8] {
        self.values.get(..usize::from(self.len)).unwrap_or(&[])
    }
}

/// Cantidad de cifras de un valor en `0..=999` (el cero tiene una cifra).
const fn digit_count(value: u32) -> u8 {
    if value >= 100 {
        3
    } else if value >= 10 {
        2
    } else {
        1
    }
}

/// Cifra de `value` en el lugar `place` (0 = unidades).
const fn digit_at_place(value: u32, place: u8) -> u8 {
    let divisor = if place < POW10.len() as u8 {
        POW10[place as usize]
    } else {
        100_000
    };
    ((value / divisor) % 10) as u8
}

/// Error de construcción de una multiplicación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MultiplicationError {
    /// Alguno de los factores no está en `0..=999`.
    #[error("factor fuera de rango: {0} (se esperaba 0..=999)")]
    FactorOutOfRange(u32),
}

/// Producto parcial de un dígito por un dígito.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PartialProduct {
    /// Cifra del multiplicando que se multiplica.
    pub multiplicand_digit: u8,
    /// Cifra del multiplicador que fija el paso.
    pub multiplier_digit: u8,
    /// Producto de las dos cifras (`0..=81`).
    pub value: u8,
    /// Cifra de las decenas; siempre se registra aunque sea 0.
    pub tens: u8,
    /// Cifra de las unidades.
    pub units: u8,
    /// Varilla de las unidades del producto (0 = unidades del resultado).
    pub units_rod: u8,
    /// Varilla de las decenas (`units_rod + 1`).
    pub tens_rod: u8,
    /// Total acumulado en el ábaco después de sumar este producto.
    pub running_total: u64,
}

impl PartialProduct {
    /// Aporte de este producto al resultado, ya colocado por valor de lugar.
    #[must_use]
    pub fn contribution(self) -> u64 {
        let place = usize::from(self.units_rod);
        let factor = u64::from(POW10.get(place).copied().unwrap_or(100_000));
        u64::from(self.value) * factor
    }

    /// Producto escrito con dos cifras, como se registra en el ábaco.
    #[must_use]
    pub const fn recorded(self) -> (u8, u8) {
        (self.tens, self.units)
    }
}

/// Estado de las varillas tras un paso, con la cifra más significativa primero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RodState {
    digits: [u8; MAX_RODS],
    rods: u8,
}

impl RodState {
    /// Estado vacío con `rods` varillas.
    #[must_use]
    pub const fn empty(rods: u8) -> Self {
        Self {
            digits: [0; MAX_RODS],
            rods: if rods > MAX_RODS as u8 {
                MAX_RODS as u8
            } else {
                rods
            },
        }
    }

    /// Estado que representa `total` en `rods` varillas.
    #[must_use]
    pub fn from_total(total: u64, rods: u8) -> Self {
        let mut state = Self::empty(rods);
        let count = usize::from(state.rods);
        let mut value = total;
        let mut index = count;
        while index > 0 {
            index -= 1;
            if let Some(slot) = state.digits.get_mut(index) {
                *slot = (value % 10) as u8;
            }
            value /= 10;
        }
        state
    }

    /// Cantidad de varillas visibles.
    #[must_use]
    pub const fn rods(&self) -> usize {
        self.rods as usize
    }

    /// Cifras de la más significativa a la menos significativa.
    #[must_use]
    pub fn digits(&self) -> &[u8] {
        self.digits.get(..usize::from(self.rods)).unwrap_or(&[])
    }

    /// Cifra en una varilla (0 = unidades), o `None` si la varilla no existe.
    #[must_use]
    pub fn digit_at(&self, rod: u8) -> Option<u8> {
        if rod >= self.rods {
            return None;
        }
        let index = usize::from(self.rods) - 1 - usize::from(rod);
        self.digits.get(index).copied()
    }

    /// Valor numérico leído en las varillas.
    #[must_use]
    pub fn value(&self) -> u64 {
        let mut total = 0u64;
        for digit in self.digits() {
            total = total * 10 + u64::from(*digit);
        }
        total
    }
}

/// Multiplicación resuelta paso a paso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Multiplication {
    multiplicand: u32,
    multiplier: u32,
    multiplicand_digits: Digits,
    multiplier_digits: Digits,
    rods: u8,
    steps: Vec<PartialProduct>,
    product: u64,
}

impl Multiplication {
    /// Construye la multiplicación con el orden pedido: `multiplicand × multiplier`.
    ///
    /// El multiplicador es el factor cuyas cifras se recorren en el bucle exterior,
    /// como en la 空盘前乘法.
    ///
    /// # Errors
    /// Devuelve [`MultiplicationError::FactorOutOfRange`] si algún factor supera 999.
    pub fn new(multiplicand: u32, multiplier: u32) -> Result<Self, MultiplicationError> {
        if multiplicand > max_factor() {
            return Err(MultiplicationError::FactorOutOfRange(multiplicand));
        }
        if multiplier > max_factor() {
            return Err(MultiplicationError::FactorOutOfRange(multiplier));
        }
        let a_digits = Digits::of(multiplicand);
        let b_digits = Digits::of(multiplier);
        let rods = a_digits.len.saturating_add(b_digits.len);
        let mut steps = Vec::with_capacity(usize::from(a_digits.len) * usize::from(b_digits.len));
        let mut running = 0u64;
        let b_values = b_digits.as_slice();
        let a_values = a_digits.as_slice();
        for (j, b_digit) in b_values.iter().enumerate() {
            let b_place = b_digits.len - 1 - j as u8;
            for (i, a_digit) in a_values.iter().enumerate() {
                let a_place = a_digits.len - 1 - i as u8;
                let units_rod = a_place.saturating_add(b_place);
                let value = a_digit.saturating_mul(*b_digit);
                running += u64::from(value) * u64::from(POW10[usize::from(units_rod)]);
                steps.push(PartialProduct {
                    multiplicand_digit: *a_digit,
                    multiplier_digit: *b_digit,
                    value,
                    tens: value / 10,
                    units: value % 10,
                    units_rod,
                    tens_rod: units_rod.saturating_add(1),
                    running_total: running,
                });
            }
        }
        Ok(Self {
            multiplicand,
            multiplier,
            multiplicand_digits: a_digits,
            multiplier_digits: b_digits,
            rods,
            steps,
            product: u64::from(multiplicand) * u64::from(multiplier),
        })
    }

    /// Multiplicando original.
    #[must_use]
    pub const fn multiplicand(&self) -> u32 {
        self.multiplicand
    }

    /// Multiplicador original (el que se recorre primero).
    #[must_use]
    pub const fn multiplier(&self) -> u32 {
        self.multiplier
    }

    /// Resultado exacto.
    #[must_use]
    pub const fn product(&self) -> u64 {
        self.product
    }

    /// Cifras del multiplicando, de izquierda a derecha.
    #[must_use]
    pub fn multiplicand_digits(&self) -> &[u8] {
        self.multiplicand_digits.as_slice()
    }

    /// Cifras del multiplicador, de izquierda a derecha.
    #[must_use]
    pub fn multiplier_digits(&self) -> &[u8] {
        self.multiplier_digits.as_slice()
    }

    /// Cantidad de varillas necesarias para el resultado.
    #[must_use]
    pub const fn rods(&self) -> usize {
        self.rods as usize
    }

    /// Productos parciales en el orden en que se registran.
    #[must_use]
    pub fn steps(&self) -> &[PartialProduct] {
        &self.steps
    }

    /// Estado del ábaco tras registrar `done` productos parciales.
    ///
    /// `done = 0` devuelve el ábaco vacío; un valor mayor que el número de pasos
    /// equivale a haber registrado todos.
    #[must_use]
    pub fn state_after(&self, done: usize) -> RodState {
        if done == 0 {
            return RodState::empty(self.rods);
        }
        let index = done.min(self.steps.len()) - 1;
        match self.steps.get(index) {
            Some(step) => RodState::from_total(step.running_total, self.rods),
            None => RodState::empty(self.rods),
        }
    }

    /// Comprueba la respuesta del usuario.
    #[must_use]
    pub const fn is_correct(&self, answer: u64) -> bool {
        answer == self.product
    }
}

/// Mayor factor admitido.
#[must_use]
pub const fn max_factor() -> u32 {
    POW10[MAX_FACTOR_DIGITS as usize] - 1
}

/// Parámetros pedagógicos de un nivel de multiplicación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MultiplicationLevel {
    /// Cifras del multiplicando.
    pub multiplicand_digits: u8,
    /// Cifras del multiplicador.
    pub multiplier_digits: u8,
    /// Si el ejercicio se plantea para resolverlo sin ábaco visible.
    pub mental: bool,
}

/// Tabla de niveles M1…M7.
pub const MUL_LEVELS: [MultiplicationLevel; 7] = [
    MultiplicationLevel {
        multiplicand_digits: 1,
        multiplier_digits: 1,
        mental: false,
    },
    MultiplicationLevel {
        multiplicand_digits: 2,
        multiplier_digits: 1,
        mental: false,
    },
    MultiplicationLevel {
        multiplicand_digits: 1,
        multiplier_digits: 2,
        mental: false,
    },
    MultiplicationLevel {
        multiplicand_digits: 2,
        multiplier_digits: 2,
        mental: false,
    },
    MultiplicationLevel {
        multiplicand_digits: 3,
        multiplier_digits: 2,
        mental: false,
    },
    MultiplicationLevel {
        multiplicand_digits: 3,
        multiplier_digits: 3,
        mental: false,
    },
    MultiplicationLevel {
        multiplicand_digits: 2,
        multiplier_digits: 2,
        mental: true,
    },
];

/// Nivel de multiplicación con saturación en 1..=7.
#[must_use]
pub const fn multiplication_level(level: u8) -> MultiplicationLevel {
    let index = if level < MIN_MUL_LEVEL {
        0
    } else if level > MAX_MUL_LEVEL {
        MUL_LEVELS.len() - 1
    } else {
        level as usize - 1
    };
    MUL_LEVELS[index]
}

/// Ejercicio de multiplicación generado de forma determinista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiplicationDrill {
    level: u8,
    seed: u64,
    task: Multiplication,
}

impl MultiplicationDrill {
    /// Genera un ejercicio del nivel pedido a partir de una semilla.
    ///
    /// El nivel se satura en `1..=7`. Cada factor tiene exactamente las cifras del
    /// nivel (sin ceros a la izquierda) y puede contener ceros intermedios.
    #[must_use]
    pub fn generate(level: u8, seed: u64) -> Self {
        let level = level.clamp(MIN_MUL_LEVEL, MAX_MUL_LEVEL);
        let spec = multiplication_level(level);
        let mut state = normalize_seed(seed);
        let multiplicand = factor_with_digits(&mut state, spec.multiplicand_digits);
        let multiplier = factor_with_digits(&mut state, spec.multiplier_digits);
        let task = Multiplication::new(multiplicand, multiplier)
            .unwrap_or_else(|_| fallback_task(spec.multiplicand_digits, spec.multiplier_digits));
        Self { level, seed, task }
    }

    /// Nivel efectivo (1..=7).
    #[must_use]
    pub const fn level(&self) -> u8 {
        self.level
    }

    /// Semilla original.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Multiplicación resuelta paso a paso.
    #[must_use]
    pub const fn task(&self) -> &Multiplication {
        &self.task
    }

    /// Comprueba la respuesta del usuario.
    #[must_use]
    pub const fn is_correct(&self, answer: u64) -> bool {
        self.task.is_correct(answer)
    }
}

/// Normaliza la semilla para que el generador nunca se quede en cero.
const fn normalize_seed(seed: u64) -> u64 {
    if seed == 0 {
        0x9E37_79B9_7F4A_7C15
    } else {
        seed
    }
}

/// Xorshift64*: rápido, determinista y suficiente para generar ejercicios.
const fn next_u64(state: &mut u64) -> u64 {
    let mut value = *state;
    value ^= value >> 12;
    value ^= value << 25;
    value ^= value >> 27;
    *state = value;
    value.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// Factor aleatorio con exactamente `digits` cifras (la primera nunca es 0).
fn factor_with_digits(state: &mut u64, digits: u8) -> u32 {
    let digits = digits.clamp(1, MAX_FACTOR_DIGITS);
    let min = POW10[usize::from(digits - 1)];
    let max = POW10[usize::from(digits)] - 1;
    let span = u64::from(max - min) + 1;
    min + (next_u64(state) % span) as u32
}

/// Ejercicio de respaldo si la construcción fallara; no debería ocurrir con factores
/// generados en rango.
fn fallback_task(multiplicand_digits: u8, multiplier_digits: u8) -> Multiplication {
    let a = POW10[usize::from(multiplicand_digits.clamp(1, MAX_FACTOR_DIGITS) - 1)];
    let b = POW10[usize::from(multiplier_digits.clamp(1, MAX_FACTOR_DIGITS) - 1)];
    match Multiplication::new(a, b) {
        Ok(task) => task,
        Err(_) => Multiplication {
            multiplicand: a,
            multiplier: b,
            multiplicand_digits: Digits::of(a),
            multiplier_digits: Digits::of(b),
            rods: 1,
            steps: Vec::new(),
            product: u64::from(a) * u64::from(b),
        },
    }
}

/// Progreso agregado de la práctica de multiplicación, por nivel.
///
/// Se guarda con un formato de texto versionado (`v1`) para poder migrarlo sin romper
/// partidas guardadas: una versión desconocida se rechaza y la UI vuelve a cero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MulProgress {
    correct: [u32; MAX_MUL_LEVEL as usize],
    attempts: [u32; MAX_MUL_LEVEL as usize],
}

impl MulProgress {
    /// Progreso vacío.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            correct: [0; MAX_MUL_LEVEL as usize],
            attempts: [0; MAX_MUL_LEVEL as usize],
        }
    }

    /// Anota un intento del nivel `level` (se satura en 1..=7).
    pub fn record(&mut self, level: u8, correct: bool) {
        let index = level_index(level);
        if let Some(slot) = self.attempts.get_mut(index) {
            *slot = slot.saturating_add(1);
        }
        if correct && let Some(slot) = self.correct.get_mut(index) {
            *slot = slot.saturating_add(1);
        }
    }

    /// Aciertos acumulados del nivel (0 si el nivel no existe).
    #[must_use]
    pub fn correct(&self, level: u8) -> u32 {
        self.correct.get(level_index(level)).copied().unwrap_or(0)
    }

    /// Intentos acumulados del nivel (0 si el nivel no existe).
    #[must_use]
    pub fn attempts(&self, level: u8) -> u32 {
        self.attempts.get(level_index(level)).copied().unwrap_or(0)
    }

    /// Aciertos de todos los niveles.
    #[must_use]
    pub fn total_correct(&self) -> u32 {
        self.correct.iter().copied().sum()
    }

    /// Intentos de todos los niveles.
    #[must_use]
    pub fn total_attempts(&self) -> u32 {
        self.attempts.iter().copied().sum()
    }

    /// Serializa el progreso a `v1;c:a;c:a;…` (siete pares).
    #[must_use]
    pub fn encode(&self) -> String {
        let mut out = String::from("v1");
        for index in 0..MAX_MUL_LEVEL as usize {
            let correct = self.correct.get(index).copied().unwrap_or(0);
            let attempts = self.attempts.get(index).copied().unwrap_or(0);
            out.push(';');
            out.push_str(&correct.to_string());
            out.push(':');
            out.push_str(&attempts.to_string());
        }
        out
    }

    /// Decodifica el formato de [`MulProgress::encode`].
    ///
    /// Devuelve `None` si la versión no es `v1`, si faltan o sobran pares o si algún
    /// número no es un entero sin signo. Un par con más aciertos que intentos se
    /// normaliza a intentos.
    #[must_use]
    pub fn decode(raw: &str) -> Option<Self> {
        let mut parts = raw.split(';');
        if parts.next() != Some("v1") {
            return None;
        }
        let mut progress = Self::new();
        for index in 0..MAX_MUL_LEVEL as usize {
            let pair = parts.next()?;
            let mut numbers = pair.split(':');
            let correct = numbers.next()?.parse::<u32>().ok()?;
            let attempts = numbers.next()?.parse::<u32>().ok()?;
            if numbers.next().is_some() {
                return None;
            }
            let slot_correct = progress.correct.get_mut(index)?;
            let slot_attempts = progress.attempts.get_mut(index)?;
            *slot_attempts = attempts;
            *slot_correct = correct.min(attempts);
        }
        if parts.next().is_some() {
            return None;
        }
        Some(progress)
    }
}

/// Índice de un nivel saturado en `0..7`.
const fn level_index(level: u8) -> usize {
    if level < MIN_MUL_LEVEL {
        0
    } else if level > MAX_MUL_LEVEL {
        MAX_MUL_LEVEL as usize - 1
    } else {
        level as usize - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const CASES: u32 = 10_000;

    fn any_factor() -> impl Strategy<Value = u32> {
        0u32..=max_factor()
    }

    fn task(a: u32, b: u32) -> Result<Multiplication, TestCaseError> {
        Multiplication::new(a, b).map_err(|error| TestCaseError::fail(error.to_string()))
    }

    #[test]
    fn rejects_out_of_range_factors() {
        assert_eq!(
            Multiplication::new(1000, 1).err(),
            Some(MultiplicationError::FactorOutOfRange(1000))
        );
        assert_eq!(
            Multiplication::new(1, 4000).err(),
            Some(MultiplicationError::FactorOutOfRange(4000))
        );
    }

    #[test]
    fn canonical_example_23_by_45_follows_the_method() {
        let result = Multiplication::new(23, 45);
        assert!(result.is_ok());
        if let Ok(task) = result {
            assert_eq!(task.product(), 1035);
            let steps = task.steps();
            assert_eq!(steps.len(), 4);
            let recorded: Vec<(u8, u8, u8, u64)> = steps
                .iter()
                .map(|step| (step.value, step.tens, step.units, step.running_total))
                .collect();
            assert_eq!(
                recorded,
                vec![
                    (8, 0, 8, 800),
                    (12, 1, 2, 920),
                    (10, 1, 0, 1020),
                    (15, 1, 5, 1035)
                ]
            );
            assert_eq!(steps[0].units_rod, 2);
            assert_eq!(steps[0].tens_rod, 3);
            assert_eq!(steps[3].units_rod, 0);
            let final_state = task.state_after(usize::MAX);
            assert_eq!(final_state.digits(), &[1, 0, 3, 5]);
            assert_eq!(final_state.value(), 1035);
        }
    }

    #[test]
    fn two_by_one_example_28_by_3() {
        let result = Multiplication::new(28, 3);
        assert!(result.is_ok());
        if let Ok(task) = result {
            assert_eq!(task.product(), 84);
            assert_eq!(task.steps().len(), 2);
            assert_eq!(task.steps()[0].running_total, 60);
            assert_eq!(task.steps()[1].running_total, 84);
            assert_eq!(task.state_after(0).digits(), &[0, 0, 0]);
            assert_eq!(task.state_after(0).value(), 0);
        }
    }

    #[test]
    fn zero_digit_is_skipped_without_shifting() {
        let result = Multiplication::new(405, 6);
        assert!(result.is_ok());
        if let Ok(task) = result {
            assert_eq!(task.product(), 2430);
            let zero_steps: Vec<&PartialProduct> = task
                .steps()
                .iter()
                .filter(|step| step.multiplicand_digit == 0)
                .collect();
            assert_eq!(zero_steps.len(), 1);
            assert_eq!(zero_steps[0].value, 0);
            assert_eq!(zero_steps[0].recorded(), (0, 0));
            assert_eq!(task.state_after(task.steps().len()).value(), 2430);
        }
    }

    #[test]
    fn biggest_three_by_three_fits_in_six_rods() {
        let result = Multiplication::new(999, 999);
        assert!(result.is_ok());
        if let Ok(task) = result {
            assert_eq!(task.product(), 998_001);
            assert_eq!(task.rods(), 6);
            assert_eq!(task.steps().len(), 9);
            assert_eq!(task.state_after(9).digits(), &[9, 9, 8, 0, 0, 1]);
        }
    }

    #[test]
    fn drill_is_deterministic_and_respects_the_level() {
        for level in MIN_MUL_LEVEL..=MAX_MUL_LEVEL {
            let spec = multiplication_level(level);
            let first = MultiplicationDrill::generate(level, 2026);
            let second = MultiplicationDrill::generate(level, 2026);
            assert_eq!(first, second);
            assert_eq!(first.level(), level);
            assert_eq!(
                first.task().multiplicand_digits().len(),
                usize::from(spec.multiplicand_digits)
            );
            assert_eq!(
                first.task().multiplier_digits().len(),
                usize::from(spec.multiplier_digits)
            );
            assert!(first.is_correct(first.task().product()));
        }
    }

    #[test]
    fn level_saturates_outside_the_range() {
        assert_eq!(multiplication_level(0), MUL_LEVELS[0]);
        assert_eq!(multiplication_level(99), MUL_LEVELS[6]);
        assert_eq!(MultiplicationDrill::generate(0, 1).level(), MIN_MUL_LEVEL);
        assert_eq!(MultiplicationDrill::generate(200, 1).level(), MAX_MUL_LEVEL);
    }

    #[test]
    fn progress_starts_empty_and_records_by_level() {
        let mut progress = MulProgress::new();
        assert_eq!(progress.total_attempts(), 0);
        progress.record(1, true);
        progress.record(1, false);
        progress.record(4, true);
        progress.record(4, true);
        progress.record(9, true);
        assert_eq!(progress.correct(1), 1);
        assert_eq!(progress.attempts(1), 2);
        assert_eq!(progress.correct(4), 2);
        assert_eq!(progress.attempts(4), 2);
        assert_eq!(progress.correct(7), 1);
        assert_eq!(progress.attempts(7), 1);
        assert_eq!(progress.correct(0), 1);
        assert_eq!(progress.total_correct(), 4);
        assert_eq!(progress.total_attempts(), 5);
    }

    #[test]
    fn progress_decode_rejects_bad_formats_and_normalizes() {
        let mut progress = MulProgress::new();
        progress.record(2, true);
        let encoded = progress.encode();
        assert_eq!(MulProgress::decode(&encoded), Some(progress));

        assert_eq!(MulProgress::decode(""), None);
        assert_eq!(MulProgress::decode("v2;0:0;0:0;0:0;0:0;0:0;0:0;0:0"), None);
        assert_eq!(MulProgress::decode("v1;0:0"), None);
        assert_eq!(
            MulProgress::decode("v1;0:0;0:0;0:0;0:0;0:0;0:0;0:0;0:0"),
            None
        );
        assert_eq!(MulProgress::decode("v1;0:0;0:0;0:0;0:0;0:0;0:0;x:0"), None);

        let normalized = MulProgress::decode("v1;5:3;0:0;0:0;0:0;0:0;0:0;0:0");
        assert!(normalized.is_some());
        if let Some(normalized) = normalized {
            assert_eq!(normalized.correct(1), 3);
            assert_eq!(normalized.attempts(1), 3);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: CASES, ..ProptestConfig::default() })]

        #[test]
        fn product_matches_direct_multiplication(a in any_factor(), b in any_factor()) {
            let task = task(a, b)?;
            prop_assert_eq!(task.product(), u64::from(a) * u64::from(b));
        }

        #[test]
        fn steps_reconstruct_the_product(a in any_factor(), b in any_factor()) {
            let task = task(a, b)?;
            let mut total = 0u64;
            for step in task.steps() {
                total += step.contribution();
            }
            prop_assert_eq!(total, task.product());
            if let Some(last) = task.steps().last() {
                prop_assert_eq!(last.running_total, task.product());
            } else {
                prop_assert_eq!(task.product(), 0);
            }
        }

        #[test]
        fn every_partial_product_records_two_digits(a in any_factor(), b in any_factor()) {
            let task = task(a, b)?;
            for step in task.steps() {
                prop_assert_eq!(step.value, step.multiplicand_digit * step.multiplier_digit);
                prop_assert_eq!(step.tens * 10 + step.units, step.value);
                prop_assert!(step.tens <= 8);
                prop_assert!(step.units <= 9);
                prop_assert_eq!(step.recorded(), (step.tens, step.units));
            }
        }

        #[test]
        fn rod_positions_follow_place_value(a in any_factor(), b in any_factor()) {
            let task = task(a, b)?;
            let expected_steps =
                task.multiplicand_digits().len() * task.multiplier_digits().len();
            prop_assert_eq!(task.steps().len(), expected_steps);
            for step in task.steps() {
                prop_assert_eq!(step.tens_rod, step.units_rod + 1);
                prop_assert!(usize::from(step.tens_rod) < task.rods());
            }
        }

        #[test]
        fn state_after_matches_the_running_total(a in any_factor(), b in any_factor()) {
            let task = task(a, b)?;
            prop_assert_eq!(task.state_after(0).value(), 0);
            prop_assert_eq!(task.state_after(usize::MAX).value(), task.product());
            for done in 1..=task.steps().len() {
                let state = task.state_after(done);
                prop_assert_eq!(state.digits().len(), task.rods());
                prop_assert!(state.digits().iter().all(|digit| *digit <= 9));
                prop_assert_eq!(state.value(), task.steps()[done - 1].running_total);
            }
        }

        #[test]
        fn multiplication_is_commutative(a in any_factor(), b in any_factor()) {
            let forward = task(a, b)?;
            let backward = task(b, a)?;
            prop_assert_eq!(forward.product(), backward.product());
        }

        #[test]
        fn progress_roundtrips_through_text(events in prop::collection::vec((MIN_MUL_LEVEL..=MAX_MUL_LEVEL, any::<bool>()), 0..200)) {
            let mut progress = MulProgress::new();
            for (level, correct) in &events {
                progress.record(*level, *correct);
            }
            prop_assert_eq!(MulProgress::decode(&progress.encode()), Some(progress));
            prop_assert_eq!(progress.total_attempts(), events.len() as u32);
            prop_assert!(progress.total_correct() <= progress.total_attempts());
            for level in MIN_MUL_LEVEL..=MAX_MUL_LEVEL {
                prop_assert!(progress.correct(level) <= progress.attempts(level));
            }
        }

        #[test]
        fn drills_only_use_their_level_digits(level in MIN_MUL_LEVEL..=MAX_MUL_LEVEL, seed in any::<u64>()) {
            let spec = multiplication_level(level);
            let first = MultiplicationDrill::generate(level, seed);
            let second = MultiplicationDrill::generate(level, seed);
            prop_assert_eq!(first.level(), level);
            prop_assert_eq!(first.task(), second.task());
            prop_assert_eq!(first.task().multiplicand_digits().len(), usize::from(spec.multiplicand_digits));
            prop_assert_eq!(first.task().multiplier_digits().len(), usize::from(spec.multiplier_digits));
            prop_assert_eq!(first.task().product(), u64::from(first.task().multiplicand()) * u64::from(first.task().multiplier()));
            prop_assert!(first.is_correct(first.task().product()));
            prop_assert!(!first.is_correct(first.task().product() + 1));
        }
    }
}
