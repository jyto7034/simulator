//! Immutable owner identity captured at observation boundaries, not authority.
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct MeasurementIdentity {
    /// None explicitly identifies a publication without a valid SHA256 digest.
    pub content_sha256: Option<[u8; 32]>,
    pub raw_profile_sha256: Option<[u8; 32]>,
    pub effective_profile_sha256: Option<[u8; 32]>,
    pub run_seed: u64,
    pub profile_revision: Option<u32>,
    pub profile_warning_bytes: u64,
    /// Selected profile cap; a particular native owner can retain a smaller cap.
    pub selected_profile_hard_bytes: u64,
}

pub(crate) const IDENTITY_WORDS: usize = 20;

pub(crate) fn digest_bytes(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let nibble = |v: u8| match v {
        b'0'..=b'9' => Some(v - b'0'),
        b'a'..=b'f' => Some(v - b'a' + 10),
        _ => None,
    };
    let mut result = [0; 32];
    for (out, pair) in result.iter_mut().zip(value.as_bytes().chunks_exact(2)) {
        *out = nibble(pair[0])?
            .checked_mul(16)?
            .checked_add(nibble(pair[1])?)?;
    }
    Some(result)
}

pub(crate) fn encode_identity(value: Option<MeasurementIdentity>) -> [u64; IDENTITY_WORDS] {
    let mut words = [0; IDENTITY_WORDS];
    let Some(value) = value else {
        return words;
    };
    words[19] = 1;
    for (offset, digest) in [
        (0, value.content_sha256),
        (5, value.raw_profile_sha256),
        (10, value.effective_profile_sha256),
    ] {
        if let Some(digest) = digest {
            words[offset] = 1;
            for (destination, bytes) in words[offset + 1..offset + 5]
                .iter_mut()
                .zip(digest.chunks_exact(8))
            {
                *destination = u64::from_le_bytes(bytes.try_into().expect("eight digest bytes"));
            }
        }
    }
    words[15] = value.run_seed;
    words[16] = value.profile_warning_bytes;
    words[17] = value.selected_profile_hard_bytes;
    words[18] = value
        .profile_revision
        .map_or(0, |revision| u64::from(revision) + 1);
    words
}

pub(crate) fn decode_identity(words: [u64; IDENTITY_WORDS]) -> Option<MeasurementIdentity> {
    if words[19] == 0 {
        return None;
    }
    let digest = |offset: usize| {
        if words[offset] == 0 {
            return None;
        }
        let mut value = [0; 32];
        for (out, word) in value
            .chunks_exact_mut(8)
            .zip(&words[offset + 1..offset + 5])
        {
            out.copy_from_slice(&word.to_le_bytes());
        }
        Some(value)
    };
    Some(MeasurementIdentity {
        content_sha256: digest(0),
        raw_profile_sha256: digest(5),
        effective_profile_sha256: digest(10),
        run_seed: words[15],
        profile_warning_bytes: words[16],
        selected_profile_hard_bytes: words[17],
        profile_revision: words[18].checked_sub(1).and_then(|v| u32::try_from(v).ok()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cmrp_identity_fixed_digest_and_atomic_word_roundtrip_preserve_missing() {
        let settings = crate::runtime_safety::RuntimeSafetySettings::embedded();
        let digest = digest_bytes(settings.effective_sha256()).unwrap();
        let value = MeasurementIdentity {
            content_sha256: None,
            raw_profile_sha256: Some(digest),
            effective_profile_sha256: Some(digest),
            run_seed: u64::MAX,
            profile_revision: Some(u32::MAX),
            profile_warning_bytes: 8 * 1024 * 1024,
            selected_profile_hard_bytes: 32 * 1024 * 1024,
        };
        assert_eq!(decode_identity(encode_identity(Some(value))), Some(value));
        assert_eq!(decode_identity(encode_identity(None)), None);
        assert!(digest_bytes("invalid").is_none());
        assert!(digest_bytes(&"z".repeat(64)).is_none());
        assert_eq!(digest_bytes(&"ff".repeat(32)), Some([255; 32]));
    }
}

/// Exact scope of the already-computed native owner extent. These owners are
/// separate ledgers; their bytes are never summed into process memory.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeOwnerExtent {
    #[default]
    Unmeasured,
    NoNativeOwner,
    Battle {
        bytes: u64,
    },
    PendingResources {
        bytes: u64,
    },
    BattleAndPendingResources {
        battle_bytes: u64,
        pending_bytes: u64,
    },
    Relics {
        bytes: u64,
    },
    BattleAndRelics {
        battle_bytes: u64,
        relic_bytes: u64,
    },
    PendingResourcesAndRelics {
        pending_bytes: u64,
        relic_bytes: u64,
    },
    BattleAndPendingResourcesAndRelics {
        battle_bytes: u64,
        pending_bytes: u64,
        relic_bytes: u64,
    },
}
impl NativeOwnerExtent {
    pub(crate) fn from_scans(battle: Option<usize>, pending: Option<usize>) -> Self {
        match (battle, pending) {
            (None, None) => Self::NoNativeOwner,
            (Some(bytes), None) => Self::Battle {
                bytes: bytes as u64,
            },
            (None, Some(bytes)) => Self::PendingResources {
                bytes: bytes as u64,
            },
            (Some(battle_bytes), Some(pending_bytes)) => Self::BattleAndPendingResources {
                battle_bytes: battle_bytes as u64,
                pending_bytes: pending_bytes as u64,
            },
        }
    }
    pub(crate) fn with_relic_owner(self, relic: Option<usize>) -> Self {
        let Some(bytes) = relic else {
            return self;
        };
        let relic_bytes = bytes as u64;
        match self {
            Self::NoNativeOwner => Self::Relics { bytes: relic_bytes },
            Self::Battle { bytes } => Self::BattleAndRelics {
                battle_bytes: bytes,
                relic_bytes,
            },
            Self::PendingResources { bytes } => Self::PendingResourcesAndRelics {
                pending_bytes: bytes,
                relic_bytes,
            },
            Self::BattleAndPendingResources {
                battle_bytes,
                pending_bytes,
            } => Self::BattleAndPendingResourcesAndRelics {
                battle_bytes,
                pending_bytes,
                relic_bytes,
            },
            _ => self,
        }
    }
    pub(crate) fn encode(self) -> [u64; 4] {
        match self {
            Self::Unmeasured => [0, 0, 0, 0],
            Self::NoNativeOwner => [1, 0, 0, 0],
            Self::Battle { bytes } => [2, bytes, 0, 0],
            Self::PendingResources { bytes } => [3, 0, bytes, 0],
            Self::BattleAndPendingResources {
                battle_bytes,
                pending_bytes,
            } => [4, battle_bytes, pending_bytes, 0],
            Self::Relics { bytes } => [5, 0, 0, bytes],
            Self::BattleAndRelics {
                battle_bytes,
                relic_bytes,
            } => [6, battle_bytes, 0, relic_bytes],
            Self::PendingResourcesAndRelics {
                pending_bytes,
                relic_bytes,
            } => [7, 0, pending_bytes, relic_bytes],
            Self::BattleAndPendingResourcesAndRelics {
                battle_bytes,
                pending_bytes,
                relic_bytes,
            } => [8, battle_bytes, pending_bytes, relic_bytes],
        }
    }
    pub(crate) fn decode(words: [u64; 4]) -> Self {
        match words[0] {
            1 => Self::NoNativeOwner,
            2 => Self::Battle { bytes: words[1] },
            3 => Self::PendingResources { bytes: words[2] },
            4 => Self::BattleAndPendingResources {
                battle_bytes: words[1],
                pending_bytes: words[2],
            },
            5 => Self::Relics { bytes: words[3] },
            6 => Self::BattleAndRelics {
                battle_bytes: words[1],
                relic_bytes: words[3],
            },
            7 => Self::PendingResourcesAndRelics {
                pending_bytes: words[2],
                relic_bytes: words[3],
            },
            8 => Self::BattleAndPendingResourcesAndRelics {
                battle_bytes: words[1],
                pending_bytes: words[2],
                relic_bytes: words[3],
            },
            _ => Self::Unmeasured,
        }
    }
}

#[cfg(test)]
mod relic_tests {
    use super::*;
    #[test]
    fn relic_owner_extent_roundtrip_preserves_all_presence_combinations_and_legacy_json() {
        for battle in [None, Some(0), Some(17)] {
            for pending in [None, Some(0), Some(31)] {
                for relic in [None, Some(0), Some(47)] {
                    let base = NativeOwnerExtent::from_scans(battle, pending);
                    let combined = base.with_relic_owner(relic);
                    assert_eq!(NativeOwnerExtent::decode(combined.encode()), combined);
                    if relic.is_none() {
                        assert_eq!(combined, base);
                    } else {
                        assert_ne!(combined, base);
                    }
                }
            }
        }
        assert_eq!(
            serde_json::to_value(NativeOwnerExtent::NoNativeOwner).unwrap(),
            serde_json::json!("no_native_owner")
        );
        assert_eq!(
            serde_json::to_value(NativeOwnerExtent::Battle { bytes: 0 }).unwrap(),
            serde_json::json!({"battle":{"bytes":0}})
        );
        assert_eq!(
            serde_json::to_value(NativeOwnerExtent::PendingResources { bytes: 0 }).unwrap(),
            serde_json::json!({"pending_resources":{"bytes":0}})
        );
        assert_eq!(
            serde_json::to_value(NativeOwnerExtent::BattleAndPendingResources {
                battle_bytes: 1,
                pending_bytes: 0
            })
            .unwrap(),
            serde_json::json!({"battle_and_pending_resources":{"battle_bytes":1,"pending_bytes":0}})
        );
        assert_eq!(
            serde_json::to_value(NativeOwnerExtent::Unmeasured).unwrap(),
            serde_json::json!("unmeasured")
        );
        println!(
            "relic observation layout enum={} committed={} operation={} session={} encode={}",
            std::mem::size_of::<NativeOwnerExtent>(),
            std::mem::size_of::<crate::runtime_observation::CommittedSnapshot>(),
            std::mem::size_of::<crate::runtime_observation::OperationSummary>(),
            std::mem::size_of::<crate::runtime_observation::SessionState>(),
            std::mem::size_of::<[u64; 4]>()
        );
    }
}
