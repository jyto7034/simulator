use crate::game::battle::{core::BattleCore, ids::UnitInstanceId};

impl BattleCore {
    // Basic attacks grant resonance separately for release, dealt damage, and received damage.
    pub(in crate::game::battle::core) fn grant_basic_attack_release_resonance(
        &mut self,
        attacker_id: UnitInstanceId,
        time_ms: u64,
    ) {
        self.add_resonance(attacker_id, 10, time_ms, true);
    }

    pub(in crate::game::battle::core) fn grant_basic_attack_damage_dealt_resonance(
        &mut self,
        attacker_id: UnitInstanceId,
        damage_dealt: u32,
        time_ms: u64,
    ) {
        let gained = damage_dealt / 10;
        if gained > 0 {
            self.add_resonance(attacker_id, gained, time_ms, true);
        }
    }

    pub(in crate::game::battle::core) fn grant_basic_attack_damage_received_resonance(
        &mut self,
        target_id: UnitInstanceId,
        damage_taken: u32,
        time_ms: u64,
        target_survived: bool,
    ) {
        let gained = damage_taken / 10;
        if gained > 0 {
            self.add_resonance(target_id, gained, time_ms, target_survived);
        }
    }
}
