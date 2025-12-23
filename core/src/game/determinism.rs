use uuid::Uuid;

use crate::game::enums::{OrdealType, PhaseType};

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn ordeal_tag(ordeal: OrdealType) -> u64 {
    match ordeal {
        OrdealType::Dawn => 1,
        OrdealType::Noon => 2,
        OrdealType::Dusk => 3,
        OrdealType::Midnight => 4,
        OrdealType::White => 5,
    }
}

fn phase_tag(phase: PhaseType) -> u64 {
    phase.value() as u64
}

pub fn seed_for_phase(run_seed: u64, ordeal: OrdealType, phase: PhaseType) -> u64 {
    // Mix run_seed with stable tags so each phase has an independent deterministic stream.
    let tag = (ordeal_tag(ordeal) << 8) | phase_tag(phase);
    splitmix64(run_seed ^ tag.wrapping_mul(0xD1B5_4A32_D192_ED03))
}

pub fn uuid_v4_from_seed(seed: u64, namespace: u64, index: u64) -> Uuid {
    let hi = splitmix64(seed ^ namespace);
    let lo = splitmix64(seed ^ namespace.rotate_left(17) ^ index);
    let mut bytes = (((hi as u128) << 64) | (lo as u128)).to_be_bytes();

    // Set RFC4122 variant and v4 version bits.
    bytes[6] = (bytes[6] & 0x0F) | 0x40;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;

    Uuid::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_for_phase_changes_across_phases() {
        let s1 = seed_for_phase(123, OrdealType::Dawn, PhaseType::I);
        let s2 = seed_for_phase(123, OrdealType::Dawn, PhaseType::II);
        assert_ne!(s1, s2);
    }

    #[test]
    fn uuid_v4_from_seed_is_deterministic() {
        let a = uuid_v4_from_seed(123, 0x5355_5052, 0);
        let b = uuid_v4_from_seed(123, 0x5355_5052, 0);
        let c = uuid_v4_from_seed(123, 0x5355_5052, 1);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
