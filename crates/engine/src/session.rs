// SPDX-License-Identifier: AGPL-3.0-or-later
//! Historial de sesiones, estadísticas por caso y exportación CSV.
//!
//! El historial se guarda como líneas versionadas, una por sesión, y se limita a las
//! últimas [`MAX_SESSIONS`]. No hay dependencias de formato ni de fecha: la marca de
//! tiempo es Unix en milisegundos y el resto son enteros pequeños.

use crate::pedagogy::PedagogicalCase;

/// Máximo de sesiones conservadas.
pub const MAX_SESSIONS: usize = 100;

/// Una sesión completada: un drill respondido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionRecord {
    /// Marca de tiempo Unix en milisegundos.
    pub timestamp_ms: u64,
    /// Nivel de progresión al empezar.
    pub level: u8,
    /// Dígitos por término.
    pub digits: u8,
    /// Cantidad de términos.
    pub terms: u8,
    /// Si la respuesta fue correcta.
    pub correct: bool,
    /// Tiempo desde el inicio del drill hasta enviar la respuesta.
    pub elapsed_ms: u32,
    /// Caso pedagógico dominante del drill.
    pub case: PedagogicalCase,
}

impl SessionRecord {
    /// Serializa la sesión en una línea.
    #[must_use]
    pub fn encode(self) -> String {
        format!(
            "v1,{},{},{},{},{},{},{}",
            self.timestamp_ms,
            self.level,
            self.digits,
            self.terms,
            u8::from(self.correct),
            self.elapsed_ms,
            self.case.code()
        )
    }

    /// Reconstruye una sesión desde [`Self::encode`].
    #[must_use]
    pub fn decode(raw: &str) -> Option<Self> {
        let mut parts = raw.split(',');
        if parts.next() != Some("v1") {
            return None;
        }
        let timestamp_ms = parts.next()?.parse::<u64>().ok()?;
        let level = parts.next()?.parse::<u8>().ok()?;
        let digits = parts.next()?.parse::<u8>().ok()?;
        let terms = parts.next()?.parse::<u8>().ok()?;
        let correct = match parts.next()? {
            "0" => false,
            "1" => true,
            _ => return None,
        };
        let elapsed_ms = parts.next()?.parse::<u32>().ok()?;
        let case = PedagogicalCase::from_code(parts.next()?)?;
        if parts.next().is_some()
            || !(1..=6).contains(&level)
            || !(1..=6).contains(&digits)
            || !(2..=20).contains(&terms)
        {
            return None;
        }
        Some(Self {
            timestamp_ms,
            level,
            digits,
            terms,
            correct,
            elapsed_ms,
            case,
        })
    }
}

/// Serializa un historial, conservando las últimas [`MAX_SESSIONS`].
#[must_use]
pub fn encode_log(sessions: &[SessionRecord]) -> String {
    let start = sessions.len().saturating_sub(MAX_SESSIONS);
    sessions[start..]
        .iter()
        .map(|session| session.encode())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Reconstruye un historial; las líneas corruptas se descartan.
#[must_use]
pub fn decode_log(raw: &str) -> Vec<SessionRecord> {
    let mut sessions: Vec<SessionRecord> = raw
        .lines()
        .filter_map(|line| SessionRecord::decode(line.trim()))
        .collect();
    if sessions.len() > MAX_SESSIONS {
        sessions.drain(..sessions.len() - MAX_SESSIONS);
    }
    sessions
}

/// CSV con cabecera; campos numéricos o códigos estables, sin comillas necesarias.
#[must_use]
pub fn to_csv(sessions: &[SessionRecord]) -> String {
    let mut csv = String::from("timestamp_unix_ms,level,digits,terms,correct,elapsed_ms,case\n");
    for session in sessions {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            session.timestamp_ms,
            session.level,
            session.digits,
            session.terms,
            u8::from(session.correct),
            session.elapsed_ms,
            session.case.code()
        ));
    }
    csv
}

/// Estadísticas acumuladas de un caso pedagógico.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CaseStat {
    /// Caso al que pertenecen los datos.
    pub case: PedagogicalCase,
    /// Sesiones atribuidas al caso.
    pub attempts: u32,
    /// Sesiones correctas.
    pub correct: u32,
}

impl CaseStat {
    /// Precisión en `0..=1`; 0 si no hay intentos.
    #[must_use]
    pub fn accuracy(self) -> f32 {
        if self.attempts == 0 {
            0.0
        } else {
            self.correct as f32 / self.attempts as f32
        }
    }
}

/// Agrega el historial por caso dominante.
#[must_use]
pub fn case_stats(sessions: &[SessionRecord]) -> [CaseStat; 4] {
    let mut stats = [
        CaseStat {
            case: PedagogicalCase::Direct,
            ..CaseStat::default()
        },
        CaseStat {
            case: PedagogicalCase::Complement5,
            ..CaseStat::default()
        },
        CaseStat {
            case: PedagogicalCase::Complement10,
            ..CaseStat::default()
        },
        CaseStat {
            case: PedagogicalCase::Carry,
            ..CaseStat::default()
        },
    ];
    for session in sessions {
        if let Some(stat) = stats.iter_mut().find(|stat| stat.case == session.case) {
            stat.attempts = stat.attempts.saturating_add(1);
            if session.correct {
                stat.correct = stat.correct.saturating_add(1);
            }
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const CASES: u32 = 10_000;

    fn sample() -> SessionRecord {
        SessionRecord {
            timestamp_ms: 1_700_000_000_000,
            level: 2,
            digits: 1,
            terms: 5,
            correct: true,
            elapsed_ms: 1_234,
            case: PedagogicalCase::Complement5,
        }
    }

    #[test]
    fn record_round_trips() {
        assert_eq!(SessionRecord::decode(&sample().encode()), Some(sample()));
    }

    #[test]
    fn record_rejects_invalid_values() {
        assert_eq!(SessionRecord::decode("v2,0,1,1,5,1,0,direct"), None);
        assert_eq!(SessionRecord::decode("v1,0,0,1,5,1,0,direct"), None);
        assert_eq!(SessionRecord::decode("v1,0,1,7,5,1,0,direct"), None);
        assert_eq!(SessionRecord::decode("v1,0,1,1,1,1,0,direct"), None);
        assert_eq!(SessionRecord::decode("v1,0,1,1,5,2,0,direct"), None);
        assert_eq!(SessionRecord::decode("v1,0,1,1,5,1,0,nope"), None);
        assert_eq!(SessionRecord::decode(""), None);
    }

    #[test]
    fn log_round_trips_and_keeps_order() {
        let mut sessions = Vec::new();
        for index in 0..3u64 {
            sessions.push(SessionRecord {
                timestamp_ms: 1_700_000_000_000 + index,
                ..sample()
            });
        }
        let decoded = decode_log(&encode_log(&sessions));
        assert_eq!(decoded, sessions);
    }

    #[test]
    fn corrupt_lines_are_skipped() {
        let raw = format!("{}\nbasura\n", sample().encode());
        assert_eq!(decode_log(&raw), vec![sample()]);
    }

    #[test]
    fn log_is_capped_to_the_last_sessions() {
        let sessions: Vec<SessionRecord> = (0..(MAX_SESSIONS as u64 + 5))
            .map(|index| SessionRecord {
                timestamp_ms: index,
                ..sample()
            })
            .collect();
        let decoded = decode_log(&encode_log(&sessions));
        assert_eq!(decoded.len(), MAX_SESSIONS);
        assert_eq!(decoded.first().map(|session| session.timestamp_ms), Some(5));
    }

    #[test]
    fn csv_has_header_and_one_row_per_session() {
        let csv = to_csv(&[sample(), sample()]);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines[0],
            "timestamp_unix_ms,level,digits,terms,correct,elapsed_ms,case"
        );
        assert!(lines[1].contains(",c5"));
    }

    #[test]
    fn stats_are_grouped_by_case() {
        let sessions = vec![
            sample(),
            SessionRecord {
                correct: false,
                ..sample()
            },
            SessionRecord {
                case: PedagogicalCase::Carry,
                ..sample()
            },
        ];
        let stats = case_stats(&sessions);
        assert_eq!(stats[0].attempts, 0);
        assert_eq!(stats[1].attempts, 2);
        assert_eq!(stats[1].correct, 1);
        assert_eq!(stats[3].attempts, 1);
        assert_eq!(stats[3].correct, 1);
        assert!((stats[1].accuracy() - 0.5).abs() < 1e-6);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: CASES, ..ProptestConfig::default() })]
        #[test]
        fn encoding_round_trips(
            timestamp_ms in any::<u64>(),
            level in 1u8..=6,
            digits in 1u8..=6,
            terms in 2u8..=20,
            correct in any::<bool>(),
            elapsed_ms in any::<u32>(),
            case in prop::sample::select(PedagogicalCase::ALL.to_vec()),
        ) {
            let record = SessionRecord { timestamp_ms, level, digits, terms, correct, elapsed_ms, case };
            prop_assert_eq!(SessionRecord::decode(&record.encode()), Some(record));
        }

        #[test]
        fn stats_never_exceed_attempts(records in prop::collection::vec(any::<bool>(), 0..50)) {
            let sessions: Vec<SessionRecord> = records
                .into_iter()
                .map(|correct| SessionRecord { correct, ..sample() })
                .collect();
            let stats = case_stats(&sessions);
            let attempts: u32 = stats.iter().map(|stat| stat.attempts).sum();
            prop_assert_eq!(attempts as usize, sessions.len());
            for stat in stats {
                prop_assert!(stat.correct <= stat.attempts);
                prop_assert!((0.0..=1.0).contains(&stat.accuracy()));
            }
        }
    }
}
