use uuid::Uuid;

use crate::game::battle::ids::UnitInstanceId;

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

pub fn seed_with_namespace(seed: u64, namespace: u64) -> u64 {
    splitmix64(seed ^ namespace.wrapping_mul(0x9E37_79B9_7F4A_7C15))
}

/// Derive a deterministic random stream seed from a UUID without assuming
/// entropy is concentrated in either half of the UUID.
pub fn seed_with_uuid(seed: u64, namespace: u64, uuid: Uuid) -> u64 {
    let bytes = uuid.as_bytes();
    let hi = u64::from_be_bytes(bytes[..8].try_into().expect("uuid high bytes"));
    let lo = u64::from_be_bytes(bytes[8..].try_into().expect("uuid low bytes"));
    let base = seed_with_namespace(seed, namespace);
    let uuid_mix =
        splitmix64(hi ^ namespace.rotate_left(13)) ^ splitmix64(lo ^ namespace.rotate_right(7));
    splitmix64(base ^ uuid_mix.rotate_left(31))
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
    fn uuid_v4_from_seed_is_deterministic() {
        let a = uuid_v4_from_seed(123, 0x5355_5052, 0);
        let b = uuid_v4_from_seed(123, 0x5355_5052, 0);
        let c = uuid_v4_from_seed(123, 0x5355_5052, 1);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn seed_with_uuid_uses_the_full_uuid() {
        let namespace = 0x4655_4C4C_5555_4944;
        let a = Uuid::from_u128(0x1111_2222_3333_4444_aaaa_bbbb_cccc_0001);
        let same_high_different_low = Uuid::from_u128(0x1111_2222_3333_4444_aaaa_bbbb_cccc_0002);
        let different_high_same_low = Uuid::from_u128(0x9999_2222_3333_4444_aaaa_bbbb_cccc_0001);

        assert_eq!(
            seed_with_uuid(123, namespace, a),
            seed_with_uuid(123, namespace, a)
        );
        assert_ne!(
            seed_with_uuid(123, namespace, a),
            seed_with_uuid(123, namespace, same_high_different_low)
        );
        assert_ne!(
            seed_with_uuid(123, namespace, a),
            seed_with_uuid(123, namespace, different_high_same_low)
        );
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
