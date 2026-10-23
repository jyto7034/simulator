use std::cmp::Ordering;

use uuid::Uuid;

use crate::game::ability::SkillId;

use super::buffs::BuffId;
use super::core::movement::types::WorldVec2;
use super::damage::DamageSourceSnapshot;
use super::event_log::{AttackDelivery, AttackKind, BattleEventCause, SkillCastTarget};
use super::ids::UnitInstanceId;
use super::scenario::FieldWorkObjectId;
use super::scenario::ScenarioGroupId;

/// 전투 이벤트
///
/// 모든 unit/caster/target ID는 `instance_id`를 참조합니다.
/// `base_uuid`(메타데이터 참조용)와 혼동하지 않도록 주의하세요.
#[derive(Debug, Clone, PartialEq)]
pub enum BattleEvent {
    /// Private causal program and independent periodic producer wakes.
    CausalInvoke {
        time_ms: u64,
    },
    CausalPeriodic {
        time_ms: u64,
    },
    SpawnGroup {
        time_ms: u64,
        group_id: ScenarioGroupId,
        cause: BattleEventCause,
    },
    AttackStart {
        time_ms: u64,
        attacker_instance_id: UnitInstanceId,
        /// Attack 이벤트에 타겟 힌트를 줄 때 사용 (예: ExtraAttack).
        target_instance_id: Option<UnitInstanceId>,
        /// 자동 공격(반복 스케줄) 여부. false면 1회성 공격으로 처리.
        schedule_next: bool,
        cause: BattleEventCause,
    },
    AttackResolve {
        time_ms: u64,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        attack_epoch: u64,
        source_snapshot: DamageSourceSnapshot,
        kind: AttackKind,
        delivery: AttackDelivery,
        cause: BattleEventCause,
    },
    BasicHealResolve {
        time_ms: u64,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        attack_epoch: u64,
        kind: AttackKind,
        cause: BattleEventCause,
    },
    BasicAttackProjectileAdvance {
        time_ms: u64,
        projectile_id: Uuid,
        cause: BattleEventCause,
    },
    SkillProjectileImpact {
        time_ms: u64,
        delivery_id: Uuid,
        cast_seq: u64,
        step_index: usize,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        source_snapshot: DamageSourceSnapshot,
        impact_position: WorldVec2,
        first_hit_unit_id: Option<UnitInstanceId>,
        impact_vfx_id: Option<String>,
        terminal: bool,
        cause: BattleEventCause,
    },
    SkillProjectileAdvance {
        time_ms: u64,
        delivery_id: Uuid,
        cause: BattleEventCause,
    },
    SkillAreaTick {
        time_ms: u64,
        area_id: Uuid,
        cast_seq: u64,
        step_index: usize,
        skill_id: SkillId,
        step_id: String,
        center: WorldVec2,
        cause: BattleEventCause,
    },
    SkillAreaExpire {
        time_ms: u64,
        area_id: Uuid,
        cast_seq: u64,
        step_index: usize,
        skill_id: SkillId,
        step_id: String,
        cause: BattleEventCause,
    },
    /// 공명(=마나) 만땅 시 자동 시전 시작
    TileBurstWave { time_ms: u64, id: u64 },
    CounterGain { time_ms: u64, source: Uuid, generation: u64 },
    ShotStockTick { time_ms:u64, source:Uuid },
    HostileCohortTick { time_ms:u64, source:Uuid },
    ShotFlightTick { time_ms:u64 },
    StatusMobileTick { time_ms:u64 },
    MobileEffectsTick { time_ms: u64 },
    PairedAreaTick {time_ms:u64,id:u64},
    PreparedVolleyTick {time_ms:u64,source:Uuid,generation:u64},
    VolleyFlightTick {time_ms:u64},
    RelationGain { time_ms: u64, source: Uuid, generation: u64 },
    RelationActionTick { time_ms: u64, source: Uuid, generation: u64 },
    CounterChannelTick { time_ms: u64, source: Uuid, generation: u64 },
    PresenceTick { time_ms: u64, source: Uuid, host: UnitInstanceId, generation: u64 },
    SustainedExpiry {
        time_ms: u64,
        activation: u64,
    },
    SkillFlowWake {
        time_ms: u64,
        cast_seq: u64,
        generation: u64,
    },
    CorrosionBoundary {
        time_ms: u64,
    },
    ResonancePulse {
        time_ms: u64,
        unit_instance_id: UnitInstanceId,
        spawn_order: u64,
    },
    SkillStartGate {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        cause: BattleEventCause,
    },
    AutoCastStart {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        cause: BattleEventCause,
    },
    /// 자동 시전 종료 훅 (공명 리셋/락 적용)
    AutoCastEnd {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        cause: BattleEventCause,
    },
    /// 플레이어 명령으로 시작한 수동 스킬 시전 종료 훅
    ManualCastEnd {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        cause: BattleEventCause,
    },
    RestTick {
        time_ms: u64,
        unit_instance_id: UnitInstanceId,
        rest_epoch: u64,
    },
    IndependentObjectExpiry {
        time_ms: u64,
        object_id: UnitInstanceId,
        epoch: u64,
    },
    ForcedDefenseEnd {
        time_ms: u64,
        unit_instance_id: UnitInstanceId,
        epoch: u64,
    },
    FieldWorkCompletion {
        time_ms: u64,
        object_id: FieldWorkObjectId,
        generation: u64,
    },
    SkillStep {
        time_ms: u64,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        skill_id: SkillId,
        step_id: String,
        cast_target: Option<SkillCastTarget>,
        cause: BattleEventCause,
    },
    ApplyBuff {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        duration_ms: u64,
        cause: BattleEventCause,
    },
    BuffTick {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        cause: BattleEventCause,
    },
    BuffExpire {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        cause: BattleEventCause,
    },
    /// Advance the continuous movement engine by one fixed tick.
    ContinuousMovementTick {
        time_ms: u64,
    },
    /// Internal battle-time wake for a Boss Task activation.
    ///
    /// This is not projected into the public battle event log. The activation
    /// identity makes an event left behind by State/Pattern exit a stale no-op.
    BossRuntimeWake {
        time_ms: u64,
        actor_slot_index: usize,
        task_activation_id: u64,
    },
    /// Internal battle-time wake for a committed summon batch lifetime.
    BossSummonWake {
        time_ms: u64,
        batch_uuid: Uuid,
    },
    /// Internal absolute-time/deadline wake for the post-bucket Wave evaluator.
    WaveTriggerWake {
        time_ms: u64,
        trigger_id: String,
    },
}

crate::authoring::clone_shape::enum_shape!(BattleEvent {
    units: [],
    tuples: [],
    records: [
        CausalInvoke { time_ms },
        CausalPeriodic { time_ms },
        SpawnGroup {
            time_ms,
            group_id,
            cause
        },
        AttackStart {
            time_ms,
            attacker_instance_id,
            target_instance_id,
            schedule_next,
            cause
        },
        AttackResolve {
            time_ms,
            attacker_instance_id,
            target_instance_id,
            attack_epoch,
            source_snapshot,
            kind,
            delivery,
            cause
        },
        BasicHealResolve {
            time_ms,
            attacker_instance_id,
            target_instance_id,
            attack_epoch,
            kind,
            cause
        },
        BasicAttackProjectileAdvance {
            time_ms,
            projectile_id,
            cause
        },
        SkillProjectileImpact {
            time_ms,
            delivery_id,
            cast_seq,
            step_index,
            skill_id,
            step_id,
            caster_instance_id,
            source_snapshot,
            impact_position,
            first_hit_unit_id,
            impact_vfx_id,
            terminal,
            cause
        },
        SkillProjectileAdvance {
            time_ms,
            delivery_id,
            cause
        },
        SkillAreaTick {
            time_ms,
            area_id,
            cast_seq,
            step_index,
            skill_id,
            step_id,
            center,
            cause
        },
        SkillAreaExpire {
            time_ms,
            area_id,
            cast_seq,
            step_index,
            skill_id,
            step_id,
            cause
        },
        TileBurstWave { time_ms, id },
        CounterGain { time_ms, source, generation },
        ShotStockTick { time_ms, source },
        HostileCohortTick { time_ms, source },
        ShotFlightTick { time_ms },
        StatusMobileTick {time_ms},
        MobileEffectsTick {time_ms},
        PairedAreaTick {time_ms,id},
        PreparedVolleyTick {time_ms,source,generation},
        VolleyFlightTick {time_ms},
        RelationGain { time_ms, source, generation },
        RelationActionTick { time_ms, source, generation },
        CounterChannelTick { time_ms, source, generation },
        PresenceTick { time_ms, source, host, generation },
        SustainedExpiry {
            time_ms,
            activation
        },
        SkillFlowWake {
            time_ms,
            cast_seq,
            generation
        },
        CorrosionBoundary { time_ms },
        ResonancePulse {
            time_ms,
            unit_instance_id,
            spawn_order
        },
        SkillStartGate {
            time_ms,
            caster_instance_id,
            cause
        },
        AutoCastStart {
            time_ms,
            caster_instance_id,
            cause
        },
        AutoCastEnd {
            time_ms,
            caster_instance_id,
            cause
        },
        ManualCastEnd {
            time_ms,
            caster_instance_id,
            cause
        },
        RestTick {
            time_ms,
            unit_instance_id,
            rest_epoch
        },
        IndependentObjectExpiry {
            time_ms,
            object_id,
            epoch
        },
        ForcedDefenseEnd {
            time_ms,
            unit_instance_id,
            epoch
        },
        FieldWorkCompletion {
            time_ms,
            object_id,
            generation
        },
        SkillStep {
            time_ms,
            cast_seq,
            step_index,
            caster_instance_id,
            skill_id,
            step_id,
            cast_target,
            cause
        },
        ApplyBuff {
            time_ms,
            caster_instance_id,
            target_instance_id,
            buff_id,
            duration_ms,
            cause
        },
        BuffTick {
            time_ms,
            caster_instance_id,
            target_instance_id,
            buff_id,
            cause
        },
        BuffExpire {
            time_ms,
            caster_instance_id,
            target_instance_id,
            buff_id,
            cause
        },
        ContinuousMovementTick { time_ms },
        BossRuntimeWake {
            time_ms,
            actor_slot_index,
            task_activation_id
        },
        BossSummonWake {
            time_ms,
            batch_uuid
        },
        WaveTriggerWake {
            time_ms,
            trigger_id
        }
    ]
});

// `BattleEvent` needs `Eq` for `Ord`/`BinaryHeap`. Float-bearing payloads use
// `WorldVec2`, whose raw public construction and deserialization reject
// non-finite components before an event can reach the private kernel queue.
impl Eq for BattleEvent {}

impl BattleEvent {
    /// 이벤트 발생 시간(ms)
    pub fn time_ms(&self) -> u64 {
        match self {
            BattleEvent::CausalInvoke { time_ms }
            | BattleEvent::CausalPeriodic { time_ms }
            | BattleEvent::AttackStart { time_ms, .. }
            | BattleEvent::SpawnGroup { time_ms, .. }
            | BattleEvent::AttackResolve { time_ms, .. }
            | BattleEvent::BasicHealResolve { time_ms, .. }
            | BattleEvent::BasicAttackProjectileAdvance { time_ms, .. }
            | BattleEvent::SkillProjectileAdvance { time_ms, .. }
            | BattleEvent::SkillProjectileImpact { time_ms, .. }
            | BattleEvent::SkillAreaTick { time_ms, .. }
            | BattleEvent::SkillAreaExpire { time_ms, .. }
            | BattleEvent::ShotStockTick {time_ms,..} | BattleEvent::HostileCohortTick {time_ms,..} | BattleEvent::ShotFlightTick {time_ms} | BattleEvent::StatusMobileTick {time_ms} | BattleEvent::MobileEffectsTick {time_ms} | BattleEvent::PairedAreaTick {time_ms,..} | BattleEvent::PreparedVolleyTick {time_ms,..} | BattleEvent::VolleyFlightTick {time_ms}
            | BattleEvent::RelationGain { time_ms, .. }
            | BattleEvent::RelationActionTick { time_ms, .. }
            | BattleEvent::CounterGain { time_ms, .. }
            | BattleEvent::CounterChannelTick { time_ms, .. }
            | BattleEvent::TileBurstWave { time_ms, .. }
            | BattleEvent::PresenceTick { time_ms, .. }
            | BattleEvent::SustainedExpiry { time_ms, .. }
            | BattleEvent::SkillFlowWake { time_ms, .. }
            | BattleEvent::CorrosionBoundary { time_ms }
            | BattleEvent::ResonancePulse { time_ms, .. }
            | BattleEvent::SkillStartGate { time_ms, .. }
            | BattleEvent::AutoCastStart { time_ms, .. }
            | BattleEvent::AutoCastEnd { time_ms, .. }
            | BattleEvent::ManualCastEnd { time_ms, .. }
            | BattleEvent::RestTick { time_ms, .. }
            | BattleEvent::IndependentObjectExpiry { time_ms, .. }
            | BattleEvent::ForcedDefenseEnd { time_ms, .. }
            | BattleEvent::FieldWorkCompletion { time_ms, .. }
            | BattleEvent::SkillStep { time_ms, .. }
            | BattleEvent::ApplyBuff { time_ms, .. }
            | BattleEvent::BuffTick { time_ms, .. }
            | BattleEvent::BuffExpire { time_ms, .. }
            | BattleEvent::ContinuousMovementTick { time_ms }
            | BattleEvent::BossRuntimeWake { time_ms, .. }
            | BattleEvent::BossSummonWake { time_ms, .. }
            | BattleEvent::WaveTriggerWake { time_ms, .. } => *time_ms,
        }
    }

    /// 같은 시각에 여러 이벤트가 있을 때 우선순위
    fn priority(&self) -> u8 {
        match self {
            // Scenario spawns must materialize before same-timestamp attacks,
            // casts, or movement ticks can observe the board.
            BattleEvent::PairedAreaTick {..} | BattleEvent::PreparedVolleyTick {..} => 1,
            BattleEvent::VolleyFlightTick {..} | BattleEvent::ShotFlightTick {..} => 2,
            BattleEvent::ShotStockTick {..} => 6,
            BattleEvent::HostileCohortTick {..} => 22,
            BattleEvent::StatusMobileTick {..} => 25,
            BattleEvent::MobileEffectsTick {..} => 24,
            BattleEvent::CausalInvoke { .. } => 20,
            BattleEvent::CausalPeriodic { .. } => 21,
            BattleEvent::SpawnGroup { .. } => 0,
            // Projectile advances first so collision impacts are resolved before new actions.
            BattleEvent::BasicAttackProjectileAdvance { .. } => 1,
            BattleEvent::SkillProjectileAdvance { .. } => 2,
            BattleEvent::SkillProjectileImpact { .. } => 3,
            BattleEvent::SkillAreaTick { .. } => 4,
            // 버프 틱/적용을 먼저 처리하고, 시전 종료, 공격, 시전 시작, 만료 순으로 처리
            BattleEvent::ApplyBuff { .. } => 5,
            BattleEvent::BuffTick { .. } => 6,
            BattleEvent::AutoCastEnd { .. } | BattleEvent::ManualCastEnd { .. } => 7,
            BattleEvent::SkillStep { .. } => 8,
            BattleEvent::AttackStart { .. } => 9,
            BattleEvent::AttackResolve { .. } => 10,
            BattleEvent::BasicHealResolve { .. } => 11,
            // Rest observes all same-timestamp committed damage before healing.
            BattleEvent::FieldWorkCompletion { .. } => 11,
            BattleEvent::RestTick { .. } => 12,
            BattleEvent::ForcedDefenseEnd { .. } => 0,
            BattleEvent::IndependentObjectExpiry { .. } => 0,
            BattleEvent::SkillStartGate { .. } | BattleEvent::AutoCastStart { .. } => 13,
            BattleEvent::BuffExpire { .. } => 14,
            BattleEvent::SkillAreaExpire { .. } => 15,
            BattleEvent::ContinuousMovementTick { .. } => 16,
            // Leaf/domain completion at a timestamp must be visible before a
            // Boss Task checks the same absolute timeout boundary.
            BattleEvent::BossRuntimeWake { .. } => 17,
            BattleEvent::BossSummonWake { .. } => 18,
            BattleEvent::RelationGain { .. } | BattleEvent::RelationActionTick { .. } | BattleEvent::CounterGain { .. } | BattleEvent::CounterChannelTick { .. } | BattleEvent::TileBurstWave { .. } | BattleEvent::PresenceTick { .. } => 6,
            BattleEvent::SustainedExpiry { .. } => 0,
            BattleEvent::SkillFlowWake { .. } => 16,
            BattleEvent::CorrosionBoundary { .. } => 19,
            BattleEvent::WaveTriggerWake { .. } => 19,
            // Observe releases and expiry at this battle-second boundary first.
            BattleEvent::ResonancePulse { .. } => 20,
        }
    }

    /// Stable discriminator used only after time and the preserved semantic priority band.
    ///
    /// AutoCastEnd intentionally precedes ManualCastEnd inside their shared priority-8 band.
    fn variant_discriminator(&self) -> u8 {
        match self {
            BattleEvent::SpawnGroup { .. } => 0,
            BattleEvent::BasicAttackProjectileAdvance { .. } => 1,
            BattleEvent::SkillProjectileAdvance { .. } => 2,
            BattleEvent::SkillProjectileImpact { .. } => 3,
            BattleEvent::SkillAreaTick { .. } => 4,
            BattleEvent::ApplyBuff { .. } => 5,
            BattleEvent::BuffTick { .. } => 6,
            BattleEvent::AutoCastEnd { .. } => 7,
            BattleEvent::ManualCastEnd { .. } => 8,
            BattleEvent::SkillStep { .. } => 9,
            BattleEvent::AttackStart { .. } => 10,
            BattleEvent::AttackResolve { .. } => 11,
            BattleEvent::BasicHealResolve { .. } => 26,
            BattleEvent::FieldWorkCompletion { .. } => 12,
            BattleEvent::RestTick { .. } => 13,
            BattleEvent::ForcedDefenseEnd { .. } => 27,
            BattleEvent::IndependentObjectExpiry { .. } => 28,
            BattleEvent::StatusMobileTick {..} | BattleEvent::MobileEffectsTick {..} | BattleEvent::PairedAreaTick {..} | BattleEvent::PreparedVolleyTick {..} | BattleEvent::VolleyFlightTick {..} | BattleEvent::ShotFlightTick {..} | BattleEvent::ShotStockTick {..} | BattleEvent::HostileCohortTick {..} | BattleEvent::RelationGain { .. } | BattleEvent::RelationActionTick { .. } | BattleEvent::CounterGain { .. } | BattleEvent::CounterChannelTick { .. } | BattleEvent::TileBurstWave { .. } | BattleEvent::PresenceTick { .. } => 32,
            BattleEvent::SustainedExpiry { .. } => 24,
            BattleEvent::SkillFlowWake { .. } => 23,
            BattleEvent::CorrosionBoundary { .. } => 22,
            BattleEvent::ResonancePulse { .. } => 25,
            BattleEvent::SkillStartGate { .. } => 21,
            BattleEvent::AutoCastStart { .. } => 14,
            BattleEvent::BuffExpire { .. } => 15,
            BattleEvent::SkillAreaExpire { .. } => 16,
            BattleEvent::ContinuousMovementTick { .. } => 17,
            BattleEvent::BossRuntimeWake { .. } => 18,
            BattleEvent::BossSummonWake { .. } => 19,
            BattleEvent::WaveTriggerWake { .. } => 20,
            BattleEvent::CausalInvoke { .. } => 29,
            BattleEvent::CausalPeriodic { .. } => 30,
        }
    }

    /// Collision-free for valid live events because derived Debug includes the complete enum payload.
    /// Equal values are handled before this key, normalizing equality-equivalent floats such as -0.0.
    /// NaN positions are invalid by the BattleEvent Eq invariant above.
    fn complete_payload_key(&self) -> String {
        format!("{self:?}")
    }
}

/// BinaryHeap에서 가장 이른 시간의 이벤트가 먼저 나오도록
/// Ord/PartialOrd를 커스텀 구현한다.
impl Ord for BattleEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }

        // BinaryHeap은 기본이 max-heap이므로, 더 작은 time_ms가
        // 먼저 나오게 하려면 순서를 뒤집어서 비교한다.
        other
            .time_ms()
            .cmp(&self.time_ms())
            .then_with(|| other.priority().cmp(&self.priority()))
            .then_with(|| {
                other
                    .variant_discriminator()
                    .cmp(&self.variant_discriminator())
            })
            .then_with(|| match (self, other) {
                (
                    BattleEvent::ResonancePulse {
                        unit_instance_id: a,
                        spawn_order: ag,
                        ..
                    },
                    BattleEvent::ResonancePulse {
                        unit_instance_id: b,
                        spawn_order: bg,
                        ..
                    },
                ) => b.cmp(a).then_with(|| bg.cmp(ag)),
                (
                    BattleEvent::SpawnGroup { group_id: a, .. },
                    BattleEvent::SpawnGroup { group_id: b, .. },
                ) => b.0.cmp(&a.0),
                (
                    BattleEvent::SkillProjectileAdvance {
                        delivery_id: a_d, ..
                    },
                    BattleEvent::SkillProjectileAdvance {
                        delivery_id: b_d, ..
                    },
                ) => b_d.as_bytes().cmp(a_d.as_bytes()),
                (
                    BattleEvent::SkillProjectileImpact {
                        delivery_id: a_d,
                        first_hit_unit_id: a_t,
                        ..
                    },
                    BattleEvent::SkillProjectileImpact {
                        delivery_id: b_d,
                        first_hit_unit_id: b_t,
                        ..
                    },
                ) => b_d.as_bytes().cmp(a_d.as_bytes()).then_with(|| {
                    b_t.map(|id| *id.as_bytes())
                        .cmp(&a_t.map(|id| *id.as_bytes()))
                }),
                (
                    BattleEvent::SkillAreaTick {
                        area_id: a_area,
                        step_index: a_step,
                        ..
                    },
                    BattleEvent::SkillAreaTick {
                        area_id: b_area,
                        step_index: b_step,
                        ..
                    },
                ) => b_area
                    .as_bytes()
                    .cmp(a_area.as_bytes())
                    .then_with(|| b_step.cmp(a_step)),
                (
                    BattleEvent::SkillAreaExpire {
                        area_id: a_area,
                        step_index: a_step,
                        ..
                    },
                    BattleEvent::SkillAreaExpire {
                        area_id: b_area,
                        step_index: b_step,
                        ..
                    },
                ) => b_area
                    .as_bytes()
                    .cmp(a_area.as_bytes())
                    .then_with(|| b_step.cmp(a_step)),
                (
                    BattleEvent::AutoCastStart {
                        caster_instance_id: a,
                        ..
                    },
                    BattleEvent::AutoCastStart {
                        caster_instance_id: b,
                        ..
                    },
                )
                | (
                    BattleEvent::AutoCastEnd {
                        caster_instance_id: a,
                        ..
                    },
                    BattleEvent::AutoCastEnd {
                        caster_instance_id: b,
                        ..
                    },
                ) => b.as_bytes().cmp(a.as_bytes()),
                (
                    BattleEvent::ManualCastEnd {
                        caster_instance_id: a,
                        ..
                    },
                    BattleEvent::ManualCastEnd {
                        caster_instance_id: b,
                        ..
                    },
                ) => b.as_bytes().cmp(a.as_bytes()),
                (
                    BattleEvent::FieldWorkCompletion {
                        object_id: a,
                        generation: a_generation,
                        ..
                    },
                    BattleEvent::FieldWorkCompletion {
                        object_id: b,
                        generation: b_generation,
                        ..
                    },
                ) => b.cmp(a).then_with(|| b_generation.cmp(a_generation)),
                (
                    BattleEvent::RestTick {
                        unit_instance_id: a,
                        rest_epoch: a_epoch,
                        ..
                    },
                    BattleEvent::RestTick {
                        unit_instance_id: b,
                        rest_epoch: b_epoch,
                        ..
                    },
                ) => b
                    .as_bytes()
                    .cmp(a.as_bytes())
                    .then_with(|| b_epoch.cmp(a_epoch)),
                (
                    BattleEvent::IndependentObjectExpiry {
                        object_id: a,
                        epoch: ae,
                        ..
                    },
                    BattleEvent::IndependentObjectExpiry {
                        object_id: b,
                        epoch: be,
                        ..
                    },
                ) => b.cmp(a).then_with(|| be.cmp(ae)),
                (
                    BattleEvent::ForcedDefenseEnd {
                        unit_instance_id: a,
                        epoch: a_epoch,
                        ..
                    },
                    BattleEvent::ForcedDefenseEnd {
                        unit_instance_id: b,
                        epoch: b_epoch,
                        ..
                    },
                ) => b
                    .as_bytes()
                    .cmp(a.as_bytes())
                    .then_with(|| b_epoch.cmp(a_epoch)),
                (
                    BattleEvent::SkillStep {
                        cast_seq: a_cast,
                        step_index: a_step,
                        caster_instance_id: a,
                        ..
                    },
                    BattleEvent::SkillStep {
                        cast_seq: b_cast,
                        step_index: b_step,
                        caster_instance_id: b,
                        ..
                    },
                ) => b
                    .as_bytes()
                    .cmp(a.as_bytes())
                    .then_with(|| b_cast.cmp(a_cast))
                    .then_with(|| b_step.cmp(a_step)),
                (
                    BattleEvent::AttackStart {
                        attacker_instance_id: a,
                        ..
                    },
                    BattleEvent::AttackStart {
                        attacker_instance_id: b,
                        ..
                    },
                ) => {
                    let a_t = match self {
                        BattleEvent::AttackStart {
                            target_instance_id, ..
                        } => target_instance_id.map(|id| *id.as_bytes()),
                        _ => None,
                    };
                    let b_t = match other {
                        BattleEvent::AttackStart {
                            target_instance_id, ..
                        } => target_instance_id.map(|id| *id.as_bytes()),
                        _ => None,
                    };
                    b.as_bytes().cmp(a.as_bytes()).then_with(|| b_t.cmp(&a_t))
                }
                (
                    BattleEvent::AttackResolve {
                        attacker_instance_id: a,
                        target_instance_id: a_t,
                        ..
                    },
                    BattleEvent::AttackResolve {
                        attacker_instance_id: b,
                        target_instance_id: b_t,
                        ..
                    },
                ) => b
                    .as_bytes()
                    .cmp(a.as_bytes())
                    .then_with(|| b_t.as_bytes().cmp(a_t.as_bytes())),
                (
                    BattleEvent::BasicAttackProjectileAdvance {
                        projectile_id: a_p, ..
                    },
                    BattleEvent::BasicAttackProjectileAdvance {
                        projectile_id: b_p, ..
                    },
                ) => b_p.as_bytes().cmp(a_p.as_bytes()),
                (
                    BattleEvent::ApplyBuff {
                        caster_instance_id: a_c,
                        target_instance_id: a_t,
                        buff_id: a_b,
                        duration_ms: a_d,
                        ..
                    },
                    BattleEvent::ApplyBuff {
                        caster_instance_id: b_c,
                        target_instance_id: b_t,
                        buff_id: b_b,
                        duration_ms: b_d,
                        ..
                    },
                ) => b_c
                    .as_bytes()
                    .cmp(a_c.as_bytes())
                    .then_with(|| b_t.as_bytes().cmp(a_t.as_bytes()))
                    .then_with(|| b_b.as_u64().cmp(&a_b.as_u64()))
                    .then_with(|| b_d.cmp(a_d)),
                (
                    BattleEvent::BuffTick {
                        caster_instance_id: a_c,
                        target_instance_id: a_t,
                        buff_id: a_b,
                        ..
                    },
                    BattleEvent::BuffTick {
                        caster_instance_id: b_c,
                        target_instance_id: b_t,
                        buff_id: b_b,
                        ..
                    },
                ) => b_c
                    .as_bytes()
                    .cmp(a_c.as_bytes())
                    .then_with(|| b_t.as_bytes().cmp(a_t.as_bytes()))
                    .then_with(|| b_b.as_u64().cmp(&a_b.as_u64())),
                (
                    BattleEvent::BuffExpire {
                        caster_instance_id: a_c,
                        target_instance_id: a_t,
                        buff_id: a_b,
                        ..
                    },
                    BattleEvent::BuffExpire {
                        caster_instance_id: b_c,
                        target_instance_id: b_t,
                        buff_id: b_b,
                        ..
                    },
                ) => b_c
                    .as_bytes()
                    .cmp(a_c.as_bytes())
                    .then_with(|| b_t.as_bytes().cmp(a_t.as_bytes()))
                    .then_with(|| b_b.as_u64().cmp(&a_b.as_u64())),
                (
                    BattleEvent::BossRuntimeWake {
                        actor_slot_index: a_slot,
                        task_activation_id: a_activation,
                        ..
                    },
                    BattleEvent::BossRuntimeWake {
                        actor_slot_index: b_slot,
                        task_activation_id: b_activation,
                        ..
                    },
                ) => b_slot
                    .cmp(a_slot)
                    .then_with(|| b_activation.cmp(a_activation)),
                (
                    BattleEvent::WaveTriggerWake { trigger_id: a, .. },
                    BattleEvent::WaveTriggerWake { trigger_id: b, .. },
                ) => b.cmp(a),
                _ => Ordering::Equal,
            })
            .then_with(|| {
                other
                    .complete_payload_key()
                    .cmp(&self.complete_payload_key())
            })
    }
}

impl PartialOrd for BattleEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::damage::{
        CombatRollIdentity, CombatRollKind, DamageModifiers, DamageSource, DamageSourceSnapshot,
        DamageType,
    };
    use crate::game::battle::event_log::{BattleEventCause, BattleEventRootCause};
    use crate::game::enums::Side;
    use crate::game::resources::Position;
    use std::collections::BinaryHeap;
    use uuid::Uuid;

    fn test_cause(kind: BattleEventRootCause) -> BattleEventCause {
        BattleEventCause::Root { kind }
    }

    fn test_damage_snapshot(source_id: UnitInstanceId) -> DamageSourceSnapshot {
        DamageSourceSnapshot {
            fact_source: Default::default(),
            skill_cast_source: None,
            value_expr_capture: None,
            prepared_basic: None,
            weapon_growth: None,
            presence_capture: None,
            reactive_capture: None,
            mobile_capture: None,
            weapon_mark_capture: None,
            source_id,
            source_side: Side::Player,
            source_attack: 10,
            source: DamageSource::BasicAttack,
            damage_type: DamageType::Physical,
            base_damage: 10,
            modifiers: DamageModifiers::default(),
            crit_roll_percent: Some(50),
            crit_roll_identity: CombatRollIdentity::new(
                CombatRollKind::BasicAttackCrit,
                Some(source_id),
                Uuid::from_u128(0xD001),
            ),
            minimum_damage: 1,
            committed_at_ms: 10,
            on_attack_modifiers: DamageModifiers::default(),
            on_attack_bonus_damage: Vec::new(),
            on_attack_modifier_sources: Vec::new(),
            basic_lifesteal: Vec::new(),
        }
    }

    fn unequal_same_variant_pairs() -> Vec<(BattleEvent, BattleEvent)> {
        let unit_a: UnitInstanceId = Uuid::from_u128(1).into();
        let unit_b: UnitInstanceId = Uuid::from_u128(2).into();
        let system = test_cause(BattleEventRootCause::System);
        let period = test_cause(BattleEventRootCause::Period);
        let snapshot = test_damage_snapshot(unit_a);

        vec![
            (
                BattleEvent::BasicHealResolve {
                    time_ms: 10,
                    attacker_instance_id: unit_a,
                    target_instance_id: unit_b,
                    attack_epoch: 1,
                    kind: AttackKind::Auto,
                    cause: system,
                },
                BattleEvent::BasicHealResolve {
                    time_ms: 10,
                    attacker_instance_id: unit_a,
                    target_instance_id: unit_b,
                    attack_epoch: 2,
                    kind: AttackKind::Auto,
                    cause: system,
                },
            ),
            (
                BattleEvent::SpawnGroup {
                    time_ms: 10,
                    group_id: ScenarioGroupId::new("group"),
                    cause: system,
                },
                BattleEvent::SpawnGroup {
                    time_ms: 10,
                    group_id: ScenarioGroupId::new("group"),
                    cause: period,
                },
            ),
            (
                BattleEvent::AttackStart {
                    time_ms: 10,
                    attacker_instance_id: unit_a,
                    target_instance_id: Some(unit_b),
                    schedule_next: false,
                    cause: system,
                },
                BattleEvent::AttackStart {
                    time_ms: 10,
                    attacker_instance_id: unit_a,
                    target_instance_id: Some(unit_b),
                    schedule_next: true,
                    cause: system,
                },
            ),
            (
                BattleEvent::AttackResolve {
                    time_ms: 10,
                    attacker_instance_id: unit_a,
                    target_instance_id: unit_b,
                    attack_epoch: 1,
                    source_snapshot: snapshot.clone(),
                    kind: AttackKind::Auto,
                    delivery: AttackDelivery::Instant,
                    cause: system,
                },
                BattleEvent::AttackResolve {
                    time_ms: 10,
                    attacker_instance_id: unit_a,
                    target_instance_id: unit_b,
                    attack_epoch: 2,
                    source_snapshot: snapshot.clone(),
                    kind: AttackKind::Auto,
                    delivery: AttackDelivery::Instant,
                    cause: system,
                },
            ),
            (
                BattleEvent::BasicAttackProjectileAdvance {
                    time_ms: 10,
                    projectile_id: Uuid::from_u128(3),
                    cause: system,
                },
                BattleEvent::BasicAttackProjectileAdvance {
                    time_ms: 10,
                    projectile_id: Uuid::from_u128(3),
                    cause: period,
                },
            ),
            (
                BattleEvent::SkillProjectileImpact {
                    time_ms: 10,
                    delivery_id: Uuid::from_u128(4),
                    cast_seq: 1,
                    step_index: 0,
                    skill_id: SkillId::from("skill"),
                    step_id: "step".to_string(),
                    caster_instance_id: unit_a,
                    source_snapshot: snapshot.clone(),
                    impact_position: WorldVec2::new(1.0, 2.0),
                    first_hit_unit_id: Some(unit_b),
                    impact_vfx_id: Some("impact".to_string()),
                    terminal: false,
                    cause: system,
                },
                BattleEvent::SkillProjectileImpact {
                    time_ms: 10,
                    delivery_id: Uuid::from_u128(4),
                    cast_seq: 2,
                    step_index: 0,
                    skill_id: SkillId::from("skill"),
                    step_id: "step".to_string(),
                    caster_instance_id: unit_a,
                    source_snapshot: snapshot.clone(),
                    impact_position: WorldVec2::new(1.0, 2.0),
                    first_hit_unit_id: Some(unit_b),
                    impact_vfx_id: Some("impact".to_string()),
                    terminal: false,
                    cause: system,
                },
            ),
            (
                BattleEvent::SkillProjectileAdvance {
                    time_ms: 10,
                    delivery_id: Uuid::from_u128(5),
                    cause: system,
                },
                BattleEvent::SkillProjectileAdvance {
                    time_ms: 10,
                    delivery_id: Uuid::from_u128(5),
                    cause: period,
                },
            ),
            (
                BattleEvent::SkillAreaTick {
                    time_ms: 10,
                    area_id: Uuid::from_u128(6),
                    cast_seq: 1,
                    step_index: 0,
                    skill_id: SkillId::from("skill"),
                    step_id: "step".to_string(),
                    center: WorldVec2::new(1.0, 2.0),
                    cause: system,
                },
                BattleEvent::SkillAreaTick {
                    time_ms: 10,
                    area_id: Uuid::from_u128(6),
                    cast_seq: 2,
                    step_index: 0,
                    skill_id: SkillId::from("skill"),
                    step_id: "step".to_string(),
                    center: WorldVec2::new(1.0, 2.0),
                    cause: system,
                },
            ),
            (
                BattleEvent::SkillAreaExpire {
                    time_ms: 10,
                    area_id: Uuid::from_u128(7),
                    cast_seq: 1,
                    step_index: 0,
                    skill_id: SkillId::from("skill"),
                    step_id: "step".to_string(),
                    cause: system,
                },
                BattleEvent::SkillAreaExpire {
                    time_ms: 10,
                    area_id: Uuid::from_u128(7),
                    cast_seq: 2,
                    step_index: 0,
                    skill_id: SkillId::from("skill"),
                    step_id: "step".to_string(),
                    cause: system,
                },
            ),
            (
                BattleEvent::AutoCastStart {
                    time_ms: 10,
                    caster_instance_id: unit_a,
                    cause: system,
                },
                BattleEvent::AutoCastStart {
                    time_ms: 10,
                    caster_instance_id: unit_a,
                    cause: period,
                },
            ),
            (
                BattleEvent::AutoCastEnd {
                    time_ms: 10,
                    caster_instance_id: unit_a,
                    cause: system,
                },
                BattleEvent::AutoCastEnd {
                    time_ms: 10,
                    caster_instance_id: unit_a,
                    cause: period,
                },
            ),
            (
                BattleEvent::ManualCastEnd {
                    time_ms: 10,
                    caster_instance_id: unit_a,
                    cause: system,
                },
                BattleEvent::ManualCastEnd {
                    time_ms: 10,
                    caster_instance_id: unit_b,
                    cause: system,
                },
            ),
            (
                BattleEvent::RestTick {
                    time_ms: 10,
                    unit_instance_id: unit_a,
                    rest_epoch: 1,
                },
                BattleEvent::RestTick {
                    time_ms: 10,
                    unit_instance_id: unit_a,
                    rest_epoch: 2,
                },
            ),
            (
                BattleEvent::IndependentObjectExpiry {
                    time_ms: 10,
                    object_id: unit_a,
                    epoch: 1,
                },
                BattleEvent::IndependentObjectExpiry {
                    time_ms: 10,
                    object_id: unit_a,
                    epoch: 2,
                },
            ),
            (
                BattleEvent::ForcedDefenseEnd {
                    time_ms: 10,
                    unit_instance_id: unit_a,
                    epoch: 1,
                },
                BattleEvent::ForcedDefenseEnd {
                    time_ms: 10,
                    unit_instance_id: unit_a,
                    epoch: 2,
                },
            ),
            (
                BattleEvent::FieldWorkCompletion {
                    time_ms: 10,
                    object_id: FieldWorkObjectId::new("object"),
                    generation: 1,
                },
                BattleEvent::FieldWorkCompletion {
                    time_ms: 10,
                    object_id: FieldWorkObjectId::new("object"),
                    generation: 2,
                },
            ),
            (
                BattleEvent::SkillStep {
                    time_ms: 10,
                    cast_seq: 1,
                    step_index: 0,
                    caster_instance_id: unit_a,
                    skill_id: SkillId::from("skill"),
                    step_id: "step_a".to_string(),
                    cast_target: Some(SkillCastTarget::Tile {
                        position: Position::new(1, 2),
                    }),
                    cause: system,
                },
                BattleEvent::SkillStep {
                    time_ms: 10,
                    cast_seq: 1,
                    step_index: 0,
                    caster_instance_id: unit_a,
                    skill_id: SkillId::from("skill"),
                    step_id: "step_b".to_string(),
                    cast_target: Some(SkillCastTarget::Tile {
                        position: Position::new(1, 2),
                    }),
                    cause: system,
                },
            ),
            (
                BattleEvent::ApplyBuff {
                    time_ms: 10,
                    caster_instance_id: unit_a,
                    target_instance_id: unit_b,
                    buff_id: BuffId::from_name("buff"),
                    duration_ms: 100,
                    cause: system,
                },
                BattleEvent::ApplyBuff {
                    time_ms: 10,
                    caster_instance_id: unit_a,
                    target_instance_id: unit_b,
                    buff_id: BuffId::from_name("buff"),
                    duration_ms: 100,
                    cause: period,
                },
            ),
            (
                BattleEvent::BuffTick {
                    time_ms: 10,
                    caster_instance_id: unit_a,
                    target_instance_id: unit_b,
                    buff_id: BuffId::from_name("buff"),
                    cause: system,
                },
                BattleEvent::BuffTick {
                    time_ms: 10,
                    caster_instance_id: unit_a,
                    target_instance_id: unit_b,
                    buff_id: BuffId::from_name("buff"),
                    cause: period,
                },
            ),
            (
                BattleEvent::BuffExpire {
                    time_ms: 10,
                    caster_instance_id: unit_a,
                    target_instance_id: unit_b,
                    buff_id: BuffId::from_name("buff"),
                    cause: system,
                },
                BattleEvent::BuffExpire {
                    time_ms: 10,
                    caster_instance_id: unit_a,
                    target_instance_id: unit_b,
                    buff_id: BuffId::from_name("buff"),
                    cause: period,
                },
            ),
            (
                BattleEvent::ContinuousMovementTick { time_ms: 10 },
                BattleEvent::ContinuousMovementTick { time_ms: 11 },
            ),
            (
                BattleEvent::BossRuntimeWake {
                    time_ms: 10,
                    actor_slot_index: 0,
                    task_activation_id: 1,
                },
                BattleEvent::BossRuntimeWake {
                    time_ms: 10,
                    actor_slot_index: 0,
                    task_activation_id: 2,
                },
            ),
            (
                BattleEvent::BossSummonWake {
                    time_ms: 10,
                    batch_uuid: Uuid::from_u128(8),
                },
                BattleEvent::BossSummonWake {
                    time_ms: 10,
                    batch_uuid: Uuid::from_u128(9),
                },
            ),
            (
                BattleEvent::WaveTriggerWake {
                    time_ms: 10,
                    trigger_id: "wave_a".to_string(),
                },
                BattleEvent::WaveTriggerWake {
                    time_ms: 10,
                    trigger_id: "wave_b".to_string(),
                },
            ),
        ]
    }

    #[test]
    fn battle_event_cmp_equal_iff_eq_for_every_live_variant() {
        let pairs = unequal_same_variant_pairs();
        assert_eq!(pairs.len(), 24, "fixture must cover every live variant");

        for (index, (left, right)) in pairs.iter().enumerate() {
            assert_ne!(left, right, "pair {index} must be unequal");
            assert_ne!(
                left.cmp(right),
                Ordering::Equal,
                "unequal pair {index} compared Equal: {left:?} vs {right:?}"
            );
            assert_eq!(
                left.cmp(right),
                right.cmp(left).reverse(),
                "pair {index} must be antisymmetric"
            );
            assert_eq!(left.cmp(left), Ordering::Equal);
            assert_eq!(right.cmp(right), Ordering::Equal);
        }

        let representatives = pairs.iter().map(|(event, _)| event).collect::<Vec<_>>();
        for (left_index, left) in representatives.iter().enumerate() {
            for (right_index, right) in representatives.iter().enumerate() {
                assert_eq!(
                    left.cmp(right) == Ordering::Equal,
                    left == right,
                    "cross-variant equality mismatch at {left_index}/{right_index}"
                );
                assert_eq!(
                    left.cmp(right),
                    right.cmp(left).reverse(),
                    "cross-variant antisymmetry mismatch at {left_index}/{right_index}"
                );
            }
        }
    }

    fn next_permutation(values: &mut [usize]) -> bool {
        let Some(pivot) = (0..values.len().saturating_sub(1))
            .rev()
            .find(|&index| values[index] < values[index + 1])
        else {
            return false;
        };
        let successor = (pivot + 1..values.len())
            .rev()
            .find(|&index| values[pivot] < values[index])
            .expect("pivot must have a successor");
        values.swap(pivot, successor);
        values[pivot + 1..].reverse();
        true
    }

    #[test]
    fn owned_clone_shape_queue_all_variants_preserve_native_order_and_payloads() {
        use crate::authoring::{
            clone_shape::CloneShape,
            control::{self, CancellationToken, CloneMeter, ExecutionLimits, ExecutionPhase},
        };
        let mut heap = BinaryHeap::with_capacity(8192);
        for (left, right) in unequal_same_variant_pairs() {
            heap.push(left);
            heap.push(right);
        }
        let original = heap.clone().into_vec();
        let (copied, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                heap.visit_clone_shape(&mut meter).unwrap();
                meter.finish("queue_storage", "queue_copy").unwrap();
                heap.clone()
            },
        )
        .unwrap();
        assert_eq!(copied.clone().into_vec(), original);
        assert_eq!(copied.into_sorted_vec(), heap.clone().into_sorted_vec());
        let mut copied = false;
        let failure = control::run(
            &ExecutionLimits {
                max_clone_slots: usage.clone_slots_peak - 1,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                heap.visit_clone_shape(&mut meter)?;
                meter.finish("queue_storage", "queue_copy")?;
                let _candidate = heap.clone();
                copied = true;
                Ok::<_, crate::game::behavior::GameError>(())
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_clone_limit");
        assert!(!copied);
        assert_eq!(heap.into_vec(), original);
    }

    fn drain_in_order(events: &[BattleEvent], insertion_order: &[usize]) -> Vec<BattleEvent> {
        let mut heap = BinaryHeap::new();
        for &index in insertion_order {
            heap.push(events[index].clone());
        }
        std::iter::from_fn(|| heap.pop()).collect()
    }

    #[test]
    fn battle_event_heap_is_identical_for_all_relevant_insert_permutations() {
        let pairs = unequal_same_variant_pairs();
        let events = vec![
            pairs[11].0.clone(),
            pairs[11].1.clone(),
            pairs[10].0.clone(),
            pairs[15].0.clone(),
            pairs[15].1.clone(),
            pairs[1].0.clone(),
        ];
        let mut insertion_order = (0..events.len()).collect::<Vec<_>>();
        let expected = drain_in_order(&events, &insertion_order);
        let mut checked = 0usize;

        loop {
            assert_eq!(
                drain_in_order(&events, &insertion_order),
                expected,
                "heap drain changed for insertion order {insertion_order:?}"
            );
            checked += 1;
            if !next_permutation(&mut insertion_order) {
                break;
            }
        }

        assert_eq!(checked, 720);
    }

    #[test]
    fn simultaneous_auto_and_manual_cast_end_uses_documented_variant_order() {
        let pairs = unequal_same_variant_pairs();
        let auto = pairs[10].0.clone();
        let manual_small = pairs[11].0.clone();
        let manual_large = pairs[11].1.clone();
        let mut heap = BinaryHeap::new();
        heap.push(manual_large.clone());
        heap.push(auto.clone());
        heap.push(manual_small.clone());

        assert_eq!(heap.pop(), Some(auto));
        assert_eq!(heap.pop(), Some(manual_small));
        assert_eq!(heap.pop(), Some(manual_large));
    }

    #[test]
    fn simultaneous_manual_cast_end_is_stable_across_insertion_orders() {
        let pairs = unequal_same_variant_pairs();
        let events = vec![pairs[11].0.clone(), pairs[11].1.clone()];

        assert_eq!(
            drain_in_order(&events, &[0, 1]),
            drain_in_order(&events, &[1, 0])
        );
        assert_eq!(drain_in_order(&events, &[0, 1]), events);
    }

    #[test]
    fn battle_event_heap_orders_by_time_then_priority_then_ids() {
        let cause = BattleEventCause::Root {
            kind: BattleEventRootCause::System,
        };

        let attacker_early: UnitInstanceId = Uuid::from_u128(1).into();
        let attacker_late: UnitInstanceId = Uuid::from_u128(2).into();

        let caster_small: UnitInstanceId = Uuid::from_u128(3).into();
        let caster_large: UnitInstanceId = Uuid::from_u128(4).into();
        assert!(caster_small.as_bytes() < caster_large.as_bytes());

        let mut heap = BinaryHeap::new();

        heap.push(BattleEvent::AttackStart {
            time_ms: 9,
            attacker_instance_id: attacker_early,
            target_instance_id: None,
            schedule_next: false,
            cause,
        });

        heap.push(BattleEvent::AttackStart {
            time_ms: 10,
            attacker_instance_id: attacker_late,
            target_instance_id: None,
            schedule_next: false,
            cause,
        });

        heap.push(BattleEvent::SpawnGroup {
            time_ms: 10,
            group_id: ScenarioGroupId::new("wave"),
            cause,
        });
        heap.push(BattleEvent::BasicAttackProjectileAdvance {
            time_ms: 10,
            projectile_id: Uuid::from_u128(10),
            cause,
        });
        heap.push(BattleEvent::SkillProjectileAdvance {
            time_ms: 10,
            delivery_id: Uuid::from_u128(11),
            cause,
        });

        heap.push(BattleEvent::AutoCastStart {
            time_ms: 10,
            caster_instance_id: caster_large,
            cause,
        });
        heap.push(BattleEvent::AutoCastStart {
            time_ms: 10,
            caster_instance_id: caster_small,
            cause,
        });

        let mut popped = Vec::new();
        while let Some(ev) = heap.pop() {
            popped.push(ev);
        }

        assert!(matches!(
            popped[0],
            BattleEvent::AttackStart { time_ms: 9, .. }
        ));
        assert!(matches!(popped[1], BattleEvent::SpawnGroup { .. }));
        assert!(matches!(
            popped[2],
            BattleEvent::BasicAttackProjectileAdvance { .. }
        ));
        assert!(matches!(
            popped[3],
            BattleEvent::SkillProjectileAdvance { .. }
        ));
        assert!(matches!(
            popped[4],
            BattleEvent::AttackStart { time_ms: 10, .. }
        ));
        assert!(matches!(
            popped[5],
            BattleEvent::AutoCastStart {
                caster_instance_id,
                ..
            } if caster_instance_id == caster_small
        ));
        assert!(matches!(
            popped[6],
            BattleEvent::AutoCastStart {
                caster_instance_id,
                ..
            } if caster_instance_id == caster_large
        ));
    }
}
