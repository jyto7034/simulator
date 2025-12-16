use std::{
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

use bevy_ecs::world::World;
use uuid::Uuid;

use crate::{
    ecs::resources::{Field, Position},
    game::{
        ability::AbilityId,
        behavior::GameError,
        data::GameDataBase,
        enums::{Side, Tier},
        stats::{Effect, TriggerType, UnitStats},
    },
};

use self::{
    ability_executor::{AbilityExecutor, AbilityRequest, UnitSnapshot},
    damage::{
        apply_damage_to_unit, calculate_damage, BattleCommand, DamageContext, DamageRequest,
        DamageSource,
    },
    death::{DeadUnit, DeathHandler},
    enums::BattleEvent,
};

pub mod ability_executor;
pub mod damage;
pub mod death;
pub mod enums;

#[derive(Clone)]
pub struct PlayerDeckInfo {
    pub units: Vec<OwnedUnit>,
    pub artifacts: Vec<OwnedArtifact>,
    pub positions: HashMap<Uuid, Position>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleWinner {
    Player,
    Opponent,
    Draw,
}

pub struct Timeline {}

pub struct BattleResult {
    pub winner: BattleWinner,
    pub timeline: Timeline,
}

pub struct Event {}

#[derive(Debug, Clone)]
pub struct OwnedArtifact {
    pub base_uuid: Uuid,
}

#[derive(Debug, Clone)]
pub struct OwnedItem {
    pub base_uuid: Uuid,
}

#[derive(Debug, Clone)]
pub struct OwnedUnit {
    pub base_uuid: Uuid,
    pub level: Tier,
    pub growth_stacks: GrowthStack,
    pub equipped_items: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrowthId {
    KillStack,
    PveWinStack,
    QuestRewardStack,
}

#[derive(Debug, Clone, Default)]
pub struct GrowthStack {
    pub stacks: HashMap<GrowthId, i32>,
}

impl GrowthStack {
    pub fn new() -> Self {
        Self {
            stacks: HashMap::new(),
        }
    }
}

impl OwnedUnit {
    pub fn effective_stats(
        &self,
        game_data: &GameDataBase,
        artifacts: &[Uuid],
    ) -> Result<UnitStats, GameError> {
        let origin = game_data
            .abnormality_data
            .get_by_uuid(&self.base_uuid)
            .ok_or(GameError::MissingResource("AbnormalityMetadata"))?;

        if origin.attack_interval_ms == 0 {
            return Err(GameError::InvalidUnitStats(
                "attack_interval_ms must be > 0",
            ));
        }

        let mut stats = UnitStats::with_values(
            origin.max_health,
            origin.max_health,
            origin.attack,
            origin.defense,
            origin.attack_interval_ms,
        );

        for (stat_id, value) in &self.growth_stacks.stacks {
            match stat_id {
                GrowthId::KillStack => {
                    stats.add_attack(*value);
                }
                GrowthId::PveWinStack => {}
                GrowthId::QuestRewardStack => {}
            }
        }

        for item_uuid in &self.equipped_items {
            let origin_item = game_data
                .equipment_data
                .get_by_uuid(item_uuid)
                .ok_or(GameError::MissingResource(""))?;

            stats.apply_permanent_effects(&origin_item.triggered_effects);
        }

        for artifact_uuid in artifacts {
            let origin_artifact = game_data
                .artifact_data
                .get_by_uuid(artifact_uuid)
                .ok_or(GameError::MissingResource(""))?;

            stats.apply_permanent_effects(&origin_artifact.triggered_effects);
        }

        Ok(stats)
    }
}

/// 전투 중 사용되는 아티팩트 런타임 표현
#[derive(Debug, Clone)]
pub struct RuntimeArtifact {
    pub instance_id: Uuid,
    pub owner: Side,
    pub base_uuid: Uuid,
}

/// 전투 중 사용되는 장비 런타임 표현
#[derive(Debug, Clone)]
pub struct RuntimeItem {
    pub instance_id: Uuid,
    pub owner: Side,
    pub owner_unit_instance: Uuid,
    pub base_uuid: Uuid,
}

/// 트리거 수집 시 소스 구분
#[derive(Debug, Clone, Copy)]
pub enum TriggerSource {
    Artifact { side: Side },
    Item { unit_instance_id: Uuid },
}

pub struct RuntimeUnit {
    pub instance_id: Uuid,
    pub owner: Side,
    pub base_uuid: Uuid,
    pub stats: UnitStats,
    pub position: Position,
    pub current_target: Option<Uuid>,
}

impl RuntimeUnit {
    pub fn set_target(&mut self, target: Uuid) {
        self.current_target = Some(target);
    }

    /// UnitSnapshot 생성
    pub fn to_snapshot(&self) -> UnitSnapshot {
        UnitSnapshot {
            id: self.instance_id,
            owner: self.owner,
            position: self.position,
            stats: self.stats,
        }
    }
}

pub struct BattleCore {
    event_queue: BinaryHeap<BattleEvent>,

    player_info: PlayerDeckInfo,
    opponent_info: PlayerDeckInfo,

    units: HashMap<Uuid, RuntimeUnit>,
    artifacts: HashMap<Uuid, RuntimeArtifact>,
    items: HashMap<Uuid, RuntimeItem>,
    /// 전투 도중 제거된 유닛의 마지막 스냅샷 (OnDeath 등 후처리용)
    graveyard: HashMap<Uuid, UnitSnapshot>,

    runtime_field: Field,

    game_data: Arc<GameDataBase>,

    // 새로운 모듈들
    death_handler: DeathHandler,
    ability_executor: AbilityExecutor,
}

impl BattleCore {
    fn side_tag(side: Side) -> u8 {
        match side {
            Side::Player => 1,
            Side::Opponent => 2,
        }
    }

    fn make_instance_id(base_uuid: Uuid, side: Side, salt: u32) -> Uuid {
        let mut bytes = *base_uuid.as_bytes();
        bytes[0] ^= Self::side_tag(side);
        bytes[1] ^= (salt & 0xFF) as u8;
        bytes[2] ^= ((salt >> 8) & 0xFF) as u8;
        bytes[3] ^= ((salt >> 16) & 0xFF) as u8;
        bytes[4] ^= ((salt >> 24) & 0xFF) as u8;
        Uuid::from_bytes(bytes)
    }

    fn make_item_instance_id(
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

    pub fn new(
        player: &PlayerDeckInfo,
        opponent: &PlayerDeckInfo,
        game_data: Arc<GameDataBase>,
        field_size: (u8, u8),
    ) -> Self {
        Self {
            event_queue: BinaryHeap::new(),
            player_info: player.clone(),
            opponent_info: opponent.clone(),
            units: HashMap::new(),
            artifacts: HashMap::new(),
            items: HashMap::new(),
            graveyard: HashMap::new(),
            runtime_field: Field::new(field_size.0, field_size.1),
            game_data,
            death_handler: DeathHandler::new(),
            ability_executor: AbilityExecutor::new(),
        }
    }

    pub fn build_runtime_units_from_decks(&mut self, side: Side) -> Result<(), GameError> {
        let deck = match side {
            Side::Player => &self.player_info,
            Side::Opponent => &self.opponent_info,
        };

        // 덱에 포함된 아티팩트만 해당 사이드에 적용
        let artifact_base_uuids: Vec<Uuid> = deck.artifacts.iter().map(|a| a.base_uuid).collect();
        for (index, artifact) in deck.artifacts.iter().enumerate() {
            let instance_id = Self::make_instance_id(artifact.base_uuid, side, index as u32);
            self.artifacts.insert(
                instance_id,
                RuntimeArtifact {
                    instance_id,
                    owner: side,
                    base_uuid: artifact.base_uuid,
                },
            );
        }

        for unit in &deck.units {
            let stats = unit.effective_stats(&self.game_data, artifact_base_uuids.as_slice())?;

            let position = deck
                .positions
                .get(&unit.base_uuid)
                .copied()
                .ok_or(GameError::UnitNotFound)?;

            let unit_instance_id = Self::make_instance_id(unit.base_uuid, side, 0);

            self.units.insert(
                unit_instance_id,
                RuntimeUnit {
                    instance_id: unit_instance_id,
                    owner: side,
                    base_uuid: unit.base_uuid,
                    stats,
                    position,
                    current_target: None,
                },
            );

            for (index, equipment_uuid) in unit.equipped_items.iter().enumerate() {
                let item_instance_id = Self::make_item_instance_id(
                    *equipment_uuid,
                    side,
                    unit_instance_id,
                    index as u32,
                );
                self.items.insert(
                    item_instance_id,
                    RuntimeItem {
                        instance_id: item_instance_id,
                        owner: side,
                        owner_unit_instance: unit_instance_id,
                        base_uuid: *equipment_uuid,
                    },
                );
            }
        }

        Ok(())
    }

    fn build_runtime_field(&mut self) -> Result<(), GameError> {
        self.runtime_field = Field::new(self.runtime_field.width, self.runtime_field.height);

        for unit in self.units.values() {
            self.runtime_field
                .place(unit.instance_id, unit.owner, unit.position)?;
        }

        Ok(())
    }

    pub fn run_battle(&mut self, _world: &mut World) -> Result<BattleResult, GameError> {
        // 재사용 안전성
        self.units.clear();
        self.artifacts.clear();
        self.items.clear();
        self.graveyard.clear();
        self.death_handler.reset();
        self.ability_executor.reset_cooldowns();

        self.build_runtime_units_from_decks(Side::Player)?;
        self.build_runtime_units_from_decks(Side::Opponent)?;
        self.build_runtime_field()?;

        self.event_queue.clear();
        self.init_initial_events();

        const MAX_BATTLE_TIME_MS: u64 = 60_000;

        while let Some(event) = self.event_queue.pop() {
            let current_time_ms = event.time_ms();

            if current_time_ms > MAX_BATTLE_TIME_MS {
                return Ok(BattleResult {
                    winner: BattleWinner::Draw,
                    timeline: Timeline {},
                });
            }

            self.process_event(event, current_time_ms)?;

            let player_alive = self.units.values().any(|u| u.owner == Side::Player);
            let opponent_alive = self.units.values().any(|u| u.owner == Side::Opponent);

            let winner = match (player_alive, opponent_alive) {
                (true, true) => None,
                (true, false) => Some(BattleWinner::Player),
                (false, true) => Some(BattleWinner::Opponent),
                (false, false) => Some(BattleWinner::Draw),
            };

            if let Some(winner) = winner {
                return Ok(BattleResult {
                    winner,
                    timeline: Timeline {},
                });
            }
        }

        Ok(BattleResult {
            winner: BattleWinner::Draw,
            timeline: Timeline {},
        })
    }

    fn init_initial_events(&mut self) {
        for unit in self.units.values() {
            self.event_queue.push(BattleEvent::Attack {
                time_ms: unit.stats.attack_interval_ms,
                attacker_instance_id: unit.instance_id,
            });
        }
    }

    /// 트리거 효과 수집
    fn collect_triggers(&self, source: TriggerSource, trigger: TriggerType) -> Vec<Effect> {
        let mut effects = Vec::new();

        match source {
            TriggerSource::Artifact { side } => {
                let mut artifacts: Vec<&RuntimeArtifact> = self
                    .artifacts
                    .values()
                    .filter(|a| a.owner == side)
                    .collect();
                artifacts.sort_by(|a, b| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()));

                for artifact in artifacts {
                    if let Some(metadata) = self
                        .game_data
                        .artifact_data
                        .get_by_uuid(&artifact.base_uuid)
                    {
                        if let Some(triggered) = metadata.triggered_effects.get(&trigger) {
                            effects.extend(triggered.iter().cloned());
                        }
                    }
                }
            }
            TriggerSource::Item { unit_instance_id } => {
                let mut items: Vec<&RuntimeItem> = self
                    .items
                    .values()
                    .filter(|i| i.owner_unit_instance == unit_instance_id)
                    .collect();
                items.sort_by(|a, b| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()));

                for item in items {
                    if let Some(metadata) =
                        self.game_data.equipment_data.get_by_uuid(&item.base_uuid)
                    {
                        if let Some(triggered) = metadata.triggered_effects.get(&trigger) {
                            effects.extend(triggered.iter().cloned());
                        }
                    }
                }
            }
        }

        effects
    }

    /// 유닛의 모든 트리거 효과 수집 (아티팩트 + 장비)
    fn collect_all_triggers(&self, unit_instance_id: Uuid, trigger: TriggerType) -> Vec<Effect> {
        let Some(unit) = self.units.get(&unit_instance_id) else {
            return Vec::new();
        };

        let mut effects =
            self.collect_triggers(TriggerSource::Artifact { side: unit.owner }, trigger);
        effects.extend(self.collect_triggers(TriggerSource::Item { unit_instance_id }, trigger));

        effects
    }

    /// 공격 처리 (리팩토링된 버전)
    fn apply_attack(&mut self, attacker_instance_id: Uuid, current_time_ms: u64) {
        // 1. 공격자/타겟 정보 확인
        let Some(attacker) = self.units.get(&attacker_instance_id) else {
            return;
        };

        let Some(target_id) = attacker.current_target else {
            return;
        };

        let Some(target) = self.units.get(&target_id) else {
            return;
        };

        // 2. 트리거 수집
        let on_attack_effects =
            self.collect_all_triggers(attacker_instance_id, TriggerType::OnAttack);
        let on_hit_effects = self.collect_all_triggers(target_id, TriggerType::OnHit);

        // 3. DamageContext 구성
        let ctx = DamageContext {
            attacker_side: attacker.owner,
            target_side: target.owner,
            attacker_attack: attacker.stats.attack,
            target_defense: target.stats.defense,
            target_current_hp: target.stats.current_health,
            target_max_hp: target.stats.max_health,
            on_attack_effects: &on_attack_effects,
            on_hit_effects: &on_hit_effects,
        };

        // 4. 데미지 계산
        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            attacker_id: attacker_instance_id,
            target_id,
            base_damage: attacker.stats.attack,
            time_ms: current_time_ms,
        };

        let result = calculate_damage(&request, &ctx);

        // 5. 데미지 적용
        if let Some(target) = self.units.get_mut(&target_id) {
            apply_damage_to_unit(&mut target.stats, result.final_damage);
        }

        // 6. 커맨드 처리
        self.process_commands(result.triggered_commands, current_time_ms);
    }

    /// 커맨드 처리
    fn process_commands(&mut self, commands: Vec<BattleCommand>, current_time_ms: u64) {
        for command in commands {
            match command {
                BattleCommand::UnitDied { unit_id, killer_id } => {
                    if let Some(unit) = self.units.get(&unit_id) {
                        self.death_handler.enqueue_death(DeadUnit {
                            unit_id,
                            killer_id,
                            owner: unit.owner,
                        });
                    }
                }
                BattleCommand::ExecuteAbility {
                    ability_id,
                    caster_id,
                    target_id,
                } => {
                    self.execute_ability_via_executor(
                        ability_id,
                        caster_id,
                        target_id,
                        current_time_ms,
                    );
                }
                BattleCommand::ApplyModifier {
                    target_id,
                    modifier,
                } => {
                    if let Some(unit) = self.units.get_mut(&target_id) {
                        unit.stats.apply_modifier(modifier);
                    }
                }
                BattleCommand::ApplyHeal {
                    target_id,
                    flat,
                    percent: _,
                    source_id,
                } => {
                    if let Some(unit) = self.units.get_mut(&target_id) {
                        let owner = unit.owner;
                        if flat >= 0 {
                            unit.stats.current_health =
                                (unit.stats.current_health as i32 + flat).max(0) as u32;
                            unit.stats.current_health =
                                unit.stats.current_health.min(unit.stats.max_health);
                        } else {
                            // 음수 힐 = 데미지
                            unit.stats.current_health = unit
                                .stats
                                .current_health
                                .saturating_sub(flat.unsigned_abs());

                            // 사망 체크
                            if unit.stats.current_health == 0 {
                                self.death_handler.enqueue_death(DeadUnit {
                                    unit_id: target_id,
                                    killer_id: source_id,
                                    owner,
                                });
                            }
                        }
                    }
                }
                BattleCommand::ScheduleAttack {
                    attacker_id,
                    time_ms,
                } => {
                    self.event_queue.push(BattleEvent::Attack {
                        time_ms: current_time_ms + time_ms,
                        attacker_instance_id: attacker_id,
                    });
                }
            }
        }

        // 사망 처리
        self.process_pending_deaths(current_time_ms);
    }

    /// AbilityExecutor를 통한 어빌리티 실행
    fn execute_ability_via_executor(
        &mut self,
        ability_id: AbilityId,
        caster_id: Uuid,
        target_id: Option<Uuid>,
        current_time_ms: u64,
    ) {
        let caster_snapshot = self
            .units
            .get(&caster_id)
            .map(|c| c.to_snapshot())
            .or_else(|| self.graveyard.get(&caster_id).cloned());

        let Some(caster_snapshot) = caster_snapshot else {
            return;
        };

        let mut unit_snapshots: Vec<UnitSnapshot> =
            self.units.values().map(|u| u.to_snapshot()).collect();
        unit_snapshots.sort_by(|a, b| a.id.as_bytes().cmp(b.id.as_bytes()));

        let request = AbilityRequest {
            ability_id,
            caster_id,
            target_id,
            time_ms: current_time_ms,
        };

        let result = self
            .ability_executor
            .execute(&request, &caster_snapshot, &unit_snapshots);

        self.process_commands(result.commands, current_time_ms);
    }

    /// 대기 중인 사망 처리
    fn process_pending_deaths(&mut self, current_time_ms: u64) {
        // borrow checker 문제 해결을 위해 필요한 데이터를 미리 수집

        // 1. 대기 중인 사망 유닛들의 트리거 효과 미리 수집
        let pending_unit_ids: Vec<Uuid> = self
            .death_handler
            .pending_deaths
            .iter()
            .map(|d| d.unit_id)
            .collect();

        // OnDeath 효과 수집
        let on_death_effects: HashMap<Uuid, Vec<Effect>> = pending_unit_ids
            .iter()
            .map(|&id| (id, self.collect_all_triggers(id, TriggerType::OnDeath)))
            .collect();

        // OnKill 효과 수집 (킬러 기준)
        let killer_ids: Vec<Uuid> = self
            .death_handler
            .pending_deaths
            .iter()
            .filter_map(|d| d.killer_id)
            .collect();
        let on_kill_effects: HashMap<Uuid, Vec<Effect>> = killer_ids
            .iter()
            .map(|&id| (id, self.collect_all_triggers(id, TriggerType::OnKill)))
            .collect();

        // OnAllyDeath 효과 수집 (모든 유닛)
        let all_unit_ids: Vec<Uuid> = self.units.keys().copied().collect();
        let on_ally_death_effects: HashMap<Uuid, Vec<Effect>> = all_unit_ids
            .iter()
            .map(|&id| (id, self.collect_all_triggers(id, TriggerType::OnAllyDeath)))
            .collect();

        // 유닛 정보 수집
        let unit_info: HashMap<Uuid, (Side, Position)> = self
            .units
            .iter()
            .map(|(id, u)| (*id, (u.owner, u.position)))
            .collect();

        // 2. process_all_deaths 호출 (클로저가 수집된 데이터만 참조)
        let result = self.death_handler.process_all_deaths(
            |unit_id| on_death_effects.get(&unit_id).cloned().unwrap_or_default(),
            |unit_id| on_kill_effects.get(&unit_id).cloned().unwrap_or_default(),
            |unit_id| {
                on_ally_death_effects
                    .get(&unit_id)
                    .cloned()
                    .unwrap_or_default()
            },
            |dead_unit_id, dead_unit_side| {
                let mut allies: Vec<Uuid> = unit_info
                    .iter()
                    .filter(|(id, (owner, _))| **id != dead_unit_id && *owner == dead_unit_side)
                    .map(|(id, _)| *id)
                    .collect();
                allies.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
                allies
            },
        );

        // 3. 유닛 제거
        for unit_id in &result.units_to_remove {
            if let Some(unit) = self.units.get(unit_id) {
                self.graveyard.insert(*unit_id, unit.to_snapshot());
            }
            self.units.remove(unit_id);
            self.runtime_field.remove(*unit_id);

            // 해당 유닛을 타겟으로 삼고 있던 유닛들의 타겟 초기화
            for unit in self.units.values_mut() {
                if unit.current_target == Some(*unit_id) {
                    unit.current_target = None;
                }
            }
        }

        // 4. 추가 커맨드 처리 (연쇄 효과)
        if !result.commands.is_empty() {
            self.process_commands(result.commands, current_time_ms);
        }
    }

    /// 단일 BattleEvent 처리
    fn process_event(
        &mut self,
        event: BattleEvent,
        _current_time_ms: u64,
    ) -> Result<(), GameError> {
        match event {
            BattleEvent::Attack {
                attacker_instance_id,
                time_ms,
            } => {
                // 1. 공격자 정보 추출
                let attacker_info = self
                    .units
                    .get(&attacker_instance_id)
                    .map(|a| (a.owner, a.stats.attack_interval_ms, a.current_target));

                let Some((owner, interval_ms, current_target)) = attacker_info else {
                    return Ok(());
                };

                // 2. 타겟 선정
                let target = if current_target.is_none() {
                    self.runtime_field
                        .find_nearest_enemy(attacker_instance_id, owner)
                } else {
                    current_target
                };

                // 3. 타겟 설정
                if let Some(target_id) = target {
                    if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                        attacker.current_target = Some(target_id);
                    }
                }

                // 4. 공격 적용
                self.apply_attack(attacker_instance_id, time_ms);

                // 5. 다음 공격 예약
                let interval_ms = interval_ms.max(1);
                self.event_queue.push(BattleEvent::Attack {
                    time_ms: time_ms.saturating_add(interval_ms),
                    attacker_instance_id,
                });

                Ok(())
            }
            BattleEvent::ApplyBuff { .. } => {
                // TODO: 버프 적용 및 상태 업데이트
                Ok(())
            }
            BattleEvent::BuffTick { .. } => {
                // TODO: 버프/디버프 틱 처리
                Ok(())
            }
            BattleEvent::BuffExpire { .. } => {
                // TODO: 버프 만료 처리
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bevy_ecs::world::World;
    use uuid::Uuid;

    use crate::{
        ecs::resources::{Inventory, Position},
        game::{
            battle::{BattleCore, OwnedUnit, PlayerDeckInfo},
            data::{
                abnormality_data::{AbnormalityDatabase, AbnormalityMetadata},
                artifact_data::ArtifactDatabase,
                bonus_data::BonusDatabase,
                equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType},
                event_pools::{EventPhasePool, EventPoolConfig},
                pve_data::PveEncounterDatabase,
                random_event_data::RandomEventDatabase,
                shop_data::ShopDatabase,
                GameDataBase,
            },
            enums::{RiskLevel, Side, Tier},
            stats::{
                Effect, StatId, StatModifier, StatModifierKind, TriggerType, TriggeredEffects,
                UnitStats,
            },
        },
    };

    fn empty_event_pools() -> EventPoolConfig {
        let empty = EventPhasePool {
            shops: vec![],
            bonuses: vec![],
            random_events: vec![],
        };
        EventPoolConfig {
            dawn: empty.clone(),
            noon: empty.clone(),
            dusk: empty.clone(),
            midnight: empty.clone(),
            white: empty,
        }
    }

    fn minimal_game_data(
        abnormalities: Vec<AbnormalityMetadata>,
        equipments: Vec<EquipmentMetadata>,
    ) -> Arc<GameDataBase> {
        let abnormality_data = Arc::new(AbnormalityDatabase::new(abnormalities));
        let artifact_data = Arc::new(ArtifactDatabase::new(vec![]));
        let equipment_data = Arc::new(EquipmentDatabase::new(equipments));
        let shop_data = Arc::new(ShopDatabase::new(vec![]));
        let bonus_data = Arc::new(BonusDatabase::new(vec![]));
        let random_event_data = Arc::new(RandomEventDatabase::new(vec![]));
        let pve_data = Arc::new(PveEncounterDatabase::new(vec![]));
        let event_pools = empty_event_pools();

        Arc::new(GameDataBase::new(
            abnormality_data,
            artifact_data,
            equipment_data,
            shop_data,
            bonus_data,
            random_event_data,
            pve_data,
            event_pools,
        ))
    }

    #[test]
    fn battle_does_not_use_world_inventory_for_artifacts() {
        // 월드 인벤토리에 아티팩트가 있어도, BattleCore는 덱의 artifacts만 사용해야 한다.
        let mut world = World::new();
        let mut inventory = Inventory::new();

        // (이 아티팩트는 덱에 넣지 않는다)
        let artifact = Arc::new(crate::game::data::artifact_data::ArtifactMetadata {
            id: "a".to_string(),
            uuid: Uuid::from_u128(1),
            name: "a".to_string(),
            description: "a".to_string(),
            rarity: RiskLevel::ZAYIN,
            price: 0,
            triggered_effects: TriggeredEffects::default(),
        });
        inventory.artifacts.add_item(artifact).unwrap();
        world.insert_resource(inventory);

        let player_uuid = Uuid::from_u128(10);
        let opponent_uuid = Uuid::from_u128(11);

        let game_data = minimal_game_data(
            vec![
                AbnormalityMetadata {
                    id: "p".to_string(),
                    uuid: player_uuid,
                    name: "p".to_string(),
                    risk_level: RiskLevel::ZAYIN,
                    price: 0,
                    max_health: 10,
                    attack: 1,
                    defense: 0,
                    attack_interval_ms: 1000,
                    abilities: vec![],
                },
                AbnormalityMetadata {
                    id: "o".to_string(),
                    uuid: opponent_uuid,
                    name: "o".to_string(),
                    risk_level: RiskLevel::ZAYIN,
                    price: 0,
                    max_health: 10,
                    attack: 1,
                    defense: 0,
                    attack_interval_ms: 1000,
                    abilities: vec![],
                },
            ],
            vec![],
        );

        let player = PlayerDeckInfo {
            units: vec![OwnedUnit {
                base_uuid: player_uuid,
                level: Tier::I,
                growth_stacks: Default::default(),
                equipped_items: vec![],
            }],
            artifacts: vec![],
            positions: [(player_uuid, Position::new(0, 0))].into_iter().collect(),
        };
        let opponent = PlayerDeckInfo {
            units: vec![OwnedUnit {
                base_uuid: opponent_uuid,
                level: Tier::I,
                growth_stacks: Default::default(),
                equipped_items: vec![],
            }],
            artifacts: vec![],
            positions: [(opponent_uuid, Position::new(1, 0))].into_iter().collect(),
        };

        let mut battle = BattleCore::new(&player, &opponent, game_data, (3, 3));
        let _ = battle.run_battle(&mut world).unwrap();

        assert!(battle.artifacts.is_empty());
    }

    #[test]
    fn ability_can_execute_with_caster_in_graveyard() {
        let caster_id = Uuid::from_u128(100);
        let target_id = Uuid::from_u128(200);

        let game_data = minimal_game_data(
            vec![
                AbnormalityMetadata {
                    id: "c".to_string(),
                    uuid: Uuid::from_u128(1),
                    name: "c".to_string(),
                    risk_level: RiskLevel::ZAYIN,
                    price: 0,
                    max_health: 10,
                    attack: 1,
                    defense: 0,
                    attack_interval_ms: 1000,
                    abilities: vec![],
                },
                AbnormalityMetadata {
                    id: "t".to_string(),
                    uuid: Uuid::from_u128(2),
                    name: "t".to_string(),
                    risk_level: RiskLevel::ZAYIN,
                    price: 0,
                    max_health: 10,
                    attack: 1,
                    defense: 0,
                    attack_interval_ms: 1000,
                    abilities: vec![],
                },
            ],
            vec![],
        );

        let empty_deck = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: Default::default(),
        };
        let mut battle = BattleCore::new(&empty_deck, &empty_deck, game_data, (3, 3));

        // 타겟 유닛 (한 방에 사망하도록)
        battle.units.insert(
            target_id,
            super::RuntimeUnit {
                instance_id: target_id,
                owner: Side::Opponent,
                base_uuid: Uuid::from_u128(2),
                stats: UnitStats::with_values(10, 10, 1, 0, 1000),
                position: Position::new(1, 0),
                current_target: None,
            },
        );

        // 캐스터가 이미 제거된 상태(= units에 없음)여도, graveyard 스냅샷으로 실행되어야 함
        battle.graveyard.insert(
            caster_id,
            super::ability_executor::UnitSnapshot {
                id: caster_id,
                owner: Side::Player,
                position: Position::new(0, 0),
                stats: UnitStats::with_values(10, 0, 1, 0, 1000),
            },
        );

        battle.execute_ability_via_executor(
            crate::game::ability::AbilityId::UnknownDistortionStrike,
            caster_id,
            None,
            0,
        );

        assert!(!battle.units.contains_key(&target_id));
    }

    #[test]
    fn ability_kill_credits_killer_for_on_kill_triggers() {
        let caster_id = Uuid::from_u128(101);
        let target_id = Uuid::from_u128(201);
        let equipment_uuid = Uuid::from_u128(301);

        // OnKill 시 공격력 +5 (킬 크레딧 검증용)
        let mut triggered_effects = TriggeredEffects::default();
        triggered_effects.insert(
            TriggerType::OnKill,
            vec![Effect::Modifier(StatModifier {
                stat: StatId::Attack,
                kind: StatModifierKind::Flat,
                value: 5,
            })],
        );

        let game_data = minimal_game_data(
            vec![
                AbnormalityMetadata {
                    id: "c".to_string(),
                    uuid: Uuid::from_u128(1),
                    name: "c".to_string(),
                    risk_level: RiskLevel::ZAYIN,
                    price: 0,
                    max_health: 10,
                    attack: 1,
                    defense: 0,
                    attack_interval_ms: 1000,
                    abilities: vec![],
                },
                AbnormalityMetadata {
                    id: "t".to_string(),
                    uuid: Uuid::from_u128(2),
                    name: "t".to_string(),
                    risk_level: RiskLevel::ZAYIN,
                    price: 0,
                    max_health: 10,
                    attack: 1,
                    defense: 0,
                    attack_interval_ms: 1000,
                    abilities: vec![],
                },
            ],
            vec![EquipmentMetadata {
                id: "e".to_string(),
                uuid: equipment_uuid,
                name: "e".to_string(),
                equipment_type: EquipmentType::Weapon,
                rarity: RiskLevel::ZAYIN,
                price: 0,
                triggered_effects,
            }],
        );

        let empty_deck = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: Default::default(),
        };
        let mut battle = BattleCore::new(&empty_deck, &empty_deck, game_data, (3, 3));

        battle.units.insert(
            caster_id,
            super::RuntimeUnit {
                instance_id: caster_id,
                owner: Side::Player,
                base_uuid: Uuid::from_u128(1),
                stats: UnitStats::with_values(10, 10, 1, 0, 1000),
                position: Position::new(0, 0),
                current_target: None,
            },
        );
        battle.items.insert(
            Uuid::from_u128(999),
            super::RuntimeItem {
                instance_id: Uuid::from_u128(999),
                owner: Side::Player,
                owner_unit_instance: caster_id,
                base_uuid: equipment_uuid,
            },
        );
        battle.units.insert(
            target_id,
            super::RuntimeUnit {
                instance_id: target_id,
                owner: Side::Opponent,
                base_uuid: Uuid::from_u128(2),
                stats: UnitStats::with_values(10, 10, 1, 0, 1000),
                position: Position::new(1, 0),
                current_target: None,
            },
        );

        battle.execute_ability_via_executor(
            crate::game::ability::AbilityId::UnknownDistortionStrike,
            caster_id,
            None,
            0,
        );

        assert!(!battle.units.contains_key(&target_id));
        assert_eq!(battle.units.get(&caster_id).unwrap().stats.attack, 6);
    }

    #[test]
    fn basic_attack_kills_and_player_wins() {
        let mut world = World::new();

        let player_base_uuid = Uuid::from_u128(10);
        let opponent_base_uuid = Uuid::from_u128(11);

        let game_data = minimal_game_data(
            vec![
                AbnormalityMetadata {
                    id: "player".to_string(),
                    uuid: player_base_uuid,
                    name: "player".to_string(),
                    risk_level: RiskLevel::ZAYIN,
                    price: 0,
                    max_health: 10,
                    attack: 100,
                    defense: 0,
                    attack_interval_ms: 1,
                    abilities: vec![],
                },
                AbnormalityMetadata {
                    id: "opponent".to_string(),
                    uuid: opponent_base_uuid,
                    name: "opponent".to_string(),
                    risk_level: RiskLevel::ZAYIN,
                    price: 0,
                    max_health: 50,
                    attack: 0,
                    defense: 0,
                    attack_interval_ms: 1000,
                    abilities: vec![],
                },
            ],
            vec![],
        );

        let player = PlayerDeckInfo {
            units: vec![OwnedUnit {
                base_uuid: player_base_uuid,
                level: Tier::I,
                growth_stacks: Default::default(),
                equipped_items: vec![],
            }],
            artifacts: vec![],
            positions: [(player_base_uuid, Position::new(0, 0))]
                .into_iter()
                .collect(),
        };
        let opponent = PlayerDeckInfo {
            units: vec![OwnedUnit {
                base_uuid: opponent_base_uuid,
                level: Tier::I,
                growth_stacks: Default::default(),
                equipped_items: vec![],
            }],
            artifacts: vec![],
            positions: [(opponent_base_uuid, Position::new(1, 0))]
                .into_iter()
                .collect(),
        };

        let mut battle = BattleCore::new(&player, &opponent, game_data, (3, 3));
        let result = battle.run_battle(&mut world).unwrap();

        assert_eq!(result.winner, super::BattleWinner::Player);

        let opponent_instance_id = BattleCore::make_instance_id(opponent_base_uuid, Side::Opponent, 0);
        let player_instance_id = BattleCore::make_instance_id(player_base_uuid, Side::Player, 0);

        assert!(battle.units.contains_key(&player_instance_id));
        assert!(!battle.units.contains_key(&opponent_instance_id));
        assert!(battle.graveyard.contains_key(&opponent_instance_id));
    }

    #[test]
    fn on_attack_skill_triggers_and_changes_outcome() {
        let mut world = World::new();

        let player_base_uuid = Uuid::from_u128(20);
        let opponent_base_uuid = Uuid::from_u128(21);
        let equipment_uuid = Uuid::from_u128(22);

        let mut triggered_effects = TriggeredEffects::default();
        triggered_effects.insert(
            TriggerType::OnAttack,
            vec![Effect::Ability(crate::game::ability::AbilityId::UnknownDistortionStrike)],
        );
        triggered_effects.insert(
            TriggerType::OnKill,
            vec![Effect::Modifier(StatModifier {
                stat: StatId::Attack,
                kind: StatModifierKind::Flat,
                value: 5,
            })],
        );

        let game_data = minimal_game_data(
            vec![
                AbnormalityMetadata {
                    id: "player".to_string(),
                    uuid: player_base_uuid,
                    name: "player".to_string(),
                    risk_level: RiskLevel::ZAYIN,
                    price: 0,
                    max_health: 10,
                    attack: 1,
                    defense: 0,
                    attack_interval_ms: 5000,
                    abilities: vec![],
                },
                AbnormalityMetadata {
                    id: "opponent".to_string(),
                    uuid: opponent_base_uuid,
                    name: "opponent".to_string(),
                    risk_level: RiskLevel::ZAYIN,
                    price: 0,
                    max_health: 20,
                    attack: 1,
                    defense: 0,
                    attack_interval_ms: 1000,
                    abilities: vec![],
                },
            ],
            vec![EquipmentMetadata {
                id: "skill_item".to_string(),
                uuid: equipment_uuid,
                name: "skill_item".to_string(),
                equipment_type: EquipmentType::Weapon,
                rarity: RiskLevel::ZAYIN,
                price: 0,
                triggered_effects,
            }],
        );

        let player = PlayerDeckInfo {
            units: vec![OwnedUnit {
                base_uuid: player_base_uuid,
                level: Tier::I,
                growth_stacks: Default::default(),
                equipped_items: vec![equipment_uuid],
            }],
            artifacts: vec![],
            positions: [(player_base_uuid, Position::new(0, 0))]
                .into_iter()
                .collect(),
        };
        let opponent = PlayerDeckInfo {
            units: vec![OwnedUnit {
                base_uuid: opponent_base_uuid,
                level: Tier::I,
                growth_stacks: Default::default(),
                equipped_items: vec![],
            }],
            artifacts: vec![],
            positions: [(opponent_base_uuid, Position::new(1, 0))]
                .into_iter()
                .collect(),
        };

        let mut battle = BattleCore::new(&player, &opponent, game_data, (3, 3));
        let result = battle.run_battle(&mut world).unwrap();

        assert_eq!(result.winner, super::BattleWinner::Player);

        let player_instance_id = BattleCore::make_instance_id(player_base_uuid, Side::Player, 0);
        let opponent_instance_id = BattleCore::make_instance_id(opponent_base_uuid, Side::Opponent, 0);

        assert!(battle.units.contains_key(&player_instance_id));
        assert!(!battle.units.contains_key(&opponent_instance_id));
        assert!(battle.graveyard.contains_key(&opponent_instance_id));

        // 스킬로 적을 처치하면 OnKill 트리거가 발동되어 공격력이 증가해야 한다.
        assert_eq!(battle.units.get(&player_instance_id).unwrap().stats.attack, 6);
    }
}
