use uuid::Uuid;

use crate::game::enums::Side;

use super::BattleCore;

impl BattleCore {
    fn side_tag(side: Side) -> u8 {
        match side {
            Side::Player => 1,
            Side::Opponent => 2,
        }
    }

    pub(super) fn make_instance_id(base_uuid: Uuid, side: Side, salt: u32) -> Uuid {
        let mut bytes = *base_uuid.as_bytes();
        bytes[0] ^= Self::side_tag(side);
        bytes[1] ^= (salt & 0xFF) as u8;
        bytes[2] ^= ((salt >> 8) & 0xFF) as u8;
        bytes[3] ^= ((salt >> 16) & 0xFF) as u8;
        bytes[4] ^= ((salt >> 24) & 0xFF) as u8;
        Uuid::from_bytes(bytes)
    }

    pub(super) fn make_artifact_instance_id(base_uuid: Uuid, side: Side, salt: u32) -> Uuid {
        let mut bytes = *base_uuid.as_bytes();
        bytes[0] ^= Self::side_tag(side) ^ 0x80;
        bytes[1] ^= (salt & 0xFF) as u8;
        bytes[2] ^= ((salt >> 8) & 0xFF) as u8;
        bytes[3] ^= ((salt >> 16) & 0xFF) as u8;
        bytes[4] ^= ((salt >> 24) & 0xFF) as u8;
        Uuid::from_bytes(bytes)
    }

    pub(super) fn make_item_instance_id(
        equipment_uuid: Uuid,
        side: Side,
        owner_unit_instance: Uuid,
        salt: u32,
    ) -> Uuid {
        let mut bytes = *equipment_uuid.as_bytes();
        let owner_bytes = owner_unit_instance.as_bytes();
        for (dst, src) in bytes.iter_mut().zip(owner_bytes.iter()) {
            *dst ^= *src;
        }
        bytes[0] ^= Self::side_tag(side);
        bytes[1] ^= (salt & 0xFF) as u8;
        bytes[2] ^= ((salt >> 8) & 0xFF) as u8;
        bytes[3] ^= ((salt >> 16) & 0xFF) as u8;
        bytes[4] ^= ((salt >> 24) & 0xFF) as u8;
        Uuid::from_bytes(bytes)
    }
}
