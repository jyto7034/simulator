use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BattleEvent {
    Attack {
        time_ms: u64,
        attacker_id: Uuid,
    },
    ApplyBuff {
        time_ms: u64,
        caster_id: Uuid,
        target_id: Uuid,
        buff_id: Uuid,
    },
    BuffTick {
        time_ms: u64,
        caster_id: Uuid,
        target_id: Uuid,
        buff_id: Uuid,
    },
    BuffExpire {
        time_ms: u64,
        caster_id: Uuid,
        target_id: Uuid,
        buff_id: Uuid,
    },
}
