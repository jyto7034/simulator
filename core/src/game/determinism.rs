use uuid::Uuid;

use crate::game::{
    battle::ids::UnitInstanceId,
    enums::{OrdealType, PhaseType},
};

const REPATH_JITTER_MOD_MS: u64 = 17;

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub fn repath_jitter_ms(run_seed: u64, unit_id: UnitInstanceId, repath_counter: u32) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&unit_id.as_bytes()[..8]);
    let unit_tag = u64::from_be_bytes(b);

    let x = run_seed
        ^ unit_tag.rotate_left(17)
        ^ (repath_counter as u64).wrapping_mul(0xD1B5_4A32_D192_ED03);

    splitmix64(x) % REPATH_JITTER_MOD_MS
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

pub fn seed_for_phase_roll(run_seed: u64, ordeal: OrdealType, phase: PhaseType, roll: u64) -> u64 {
    let phase_seed = seed_for_phase(run_seed, ordeal, phase);
    splitmix64(phase_seed ^ roll.wrapping_mul(0x94D0_49BB_1331_11EB))
}

pub fn seed_with_namespace(seed: u64, namespace: u64) -> u64 {
    splitmix64(seed ^ namespace.wrapping_mul(0x9E37_79B9_7F4A_7C15))
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
    fn seed_for_phase_roll_changes_across_rolls() {
        let s1 = seed_for_phase_roll(123, OrdealType::Dawn, PhaseType::I, 0);
        let s2 = seed_for_phase_roll(123, OrdealType::Dawn, PhaseType::I, 1);
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

    #[test]
    fn repath_jitter_ms_is_deterministic_and_bounded() {
        let unit_id: UnitInstanceId = Uuid::from_u128(1).into();
        let a = repath_jitter_ms(123, unit_id, 0);
        let b = repath_jitter_ms(123, unit_id, 0);
        let c = repath_jitter_ms(123, unit_id, 1);

        assert_eq!(a, b);
        assert!(a < super::REPATH_JITTER_MOD_MS);
        assert!(c < super::REPATH_JITTER_MOD_MS);
    }
}
