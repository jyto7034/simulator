use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustBand {
    Distrust,
    Uneasy,
    Neutral,
    Trusting,
    Devoted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PsychologyEventKind {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PsychologySubject {
    EmployeeSelf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PsychologyEventSource {
    NodeCompletion { node_id: Uuid },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PsychologyMemory {
    pub kind: PsychologyEventKind,
    pub subject: PsychologySubject,
    pub occurrence_count: u32,
    pub intensity: u8,
    pub first_source: PsychologyEventSource,
    pub last_source: PsychologyEventSource,
    pub remaining_nodes: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PsychologyRecentEvent {
    pub kind: PsychologyEventKind,
    pub source: PsychologyEventSource,
    pub trust_delta: i16,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmployeePsychology {
    pub trust_score: i16,
    pub memories: Vec<PsychologyMemory>,
    pub recent_events: Vec<PsychologyRecentEvent>,
}

impl EmployeePsychology {
    pub const MIN_TRUST_SCORE: i16 = -100;
    pub const MAX_TRUST_SCORE: i16 = 100;
    pub fn trust_band(&self) -> TrustBand {
        match self.trust_score {
            i16::MIN..=-51 => TrustBand::Distrust,
            -50..=-11 => TrustBand::Uneasy,
            -10..=25 => TrustBand::Neutral,
            26..=70 => TrustBand::Trusting,
            71..=i16::MAX => TrustBand::Devoted,
        }
    }

    pub fn advance_node(&mut self) {
        for memory in &mut self.memories {
            if let Some(remaining_nodes) = &mut memory.remaining_nodes {
                *remaining_nodes = remaining_nodes.saturating_sub(1);
            }
        }
        self.memories
            .retain(|memory| memory.remaining_nodes != Some(0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_band_boundaries_are_exact() {
        let cases = [
            (-100, TrustBand::Distrust),
            (-51, TrustBand::Distrust),
            (-50, TrustBand::Uneasy),
            (-11, TrustBand::Uneasy),
            (-10, TrustBand::Neutral),
            (25, TrustBand::Neutral),
            (26, TrustBand::Trusting),
            (70, TrustBand::Trusting),
            (71, TrustBand::Devoted),
            (100, TrustBand::Devoted),
        ];
        for (trust_score, expected) in cases {
            let psychology = EmployeePsychology {
                trust_score,
                ..EmployeePsychology::default()
            };
            assert_eq!(psychology.trust_band(), expected);
        }
    }
}
