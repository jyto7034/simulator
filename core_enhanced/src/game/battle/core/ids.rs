use uuid::Uuid;

use crate::game::battle::ids::UnitInstanceId;
use crate::game::determinism;
use crate::game::enums::Side;

use super::BattleCore;

impl BattleCore {
    const UNIT_INSTANCE_NS: u64 = 0x554E_4954_494E_5354; // "UNITINST"
    const ARTIFACT_INSTANCE_NS: u64 = 0x4152_5449_4E53_5431; // "ARTINST1"
    const ITEM_INSTANCE_NS: u64 = 0x4954_454D_494E_5354; // "ITEMINST"

    fn side_tag(side: Side) -> u8 {
        match side {
            Side::Player => 1,
            Side::Opponent => 2,
        }
    }

    fn stable_seed(bytes: &[u8]) -> u64 {
        const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
        const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;

        let mut hash = FNV_OFFSET;
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }

    pub(super) fn make_instance_id(base_uuid: Uuid, side: Side, salt: u32) -> UnitInstanceId {
        let mut material = Vec::with_capacity(16 + 1 + 4);
        material.extend_from_slice(base_uuid.as_bytes());
        material.push(Self::side_tag(side));
        material.extend_from_slice(&salt.to_be_bytes());
        let seed = Self::stable_seed(&material);
        UnitInstanceId(determinism::uuid_v4_from_seed(
            seed,
            Self::UNIT_INSTANCE_NS,
            0,
        ))
    }

    pub(super) fn make_artifact_instance_id(base_uuid: Uuid, side: Side, salt: u32) -> Uuid {
        let mut material = Vec::with_capacity(16 + 1 + 4 + 1);
        material.extend_from_slice(base_uuid.as_bytes());
        material.push(Self::side_tag(side));
        material.extend_from_slice(&salt.to_be_bytes());
        material.push(0xA1);
        let seed = Self::stable_seed(&material);
        determinism::uuid_v4_from_seed(seed, Self::ARTIFACT_INSTANCE_NS, 0)
    }

    pub(super) fn make_item_instance_id(
        equipment_uuid: Uuid,
        side: Side,
        owner_unit_instance: UnitInstanceId,
        salt: u32,
    ) -> Uuid {
        let mut material = Vec::with_capacity(16 + 16 + 1 + 4);
        material.extend_from_slice(equipment_uuid.as_bytes());
        material.extend_from_slice(owner_unit_instance.as_bytes());
        material.push(Self::side_tag(side));
        material.extend_from_slice(&salt.to_be_bytes());
        let seed = Self::stable_seed(&material);
        determinism::uuid_v4_from_seed(seed, Self::ITEM_INSTANCE_NS, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_unit_instance_ids_change_with_side_and_salt() {
        let base = Uuid::from_u128(0xABCD);

        let player_0 = BattleCore::make_instance_id(base, Side::Player, 0);
        let player_1 = BattleCore::make_instance_id(base, Side::Player, 1);
        let opponent_0 = BattleCore::make_instance_id(base, Side::Opponent, 0);

        assert_ne!(player_0, player_1);
        assert_ne!(player_0, opponent_0);
        assert_eq!(
            player_0,
            BattleCore::make_instance_id(base, Side::Player, 0)
        );
    }

    #[test]
    fn deterministic_item_instance_ids_change_with_owner_and_salt() {
        let equipment = Uuid::from_u128(0xE001);
        let owner_a = UnitInstanceId(Uuid::from_u128(0xAA));
        let owner_b = UnitInstanceId(Uuid::from_u128(0xBB));

        let a0 = BattleCore::make_item_instance_id(equipment, Side::Player, owner_a, 0);
        let a1 = BattleCore::make_item_instance_id(equipment, Side::Player, owner_a, 1);
        let b0 = BattleCore::make_item_instance_id(equipment, Side::Player, owner_b, 0);

        assert_ne!(a0, a1);
        assert_ne!(a0, b0);
        assert_eq!(
            a0,
            BattleCore::make_item_instance_id(equipment, Side::Player, owner_a, 0)
        );
    }
}
