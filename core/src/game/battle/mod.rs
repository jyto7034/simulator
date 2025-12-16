use std::{
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

use bevy_ecs::world::World;
use uuid::Uuid;

use crate::{
    ecs::resources::{Field, Inventory, Position},
    game::{
        ability::AbilityId,
        behavior::GameError,
        data::GameDataBase,
        enums::{Side, Tier},
        stats::{Effect, TriggerType, UnitStats},
    },
};

use self::enums::BattleEvent;

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

// 아티팩트는 그럴 일 없겠지만, Runtime 때 수치 변경 기능 확장을 위해 Owned Layer 유지
#[derive(Debug, Clone)]
pub struct OwnedArtifact {
    pub base_uuid: Uuid,
}

// Runtime 때 수치 변경 기능 확장을 위해 Owned Layer 유지
#[derive(Debug, Clone)]
pub struct OwnedItem {
    pub base_uuid: Uuid,
}

#[derive(Debug, Clone)]
pub struct OwnedUnit {
    pub base_uuid: Uuid,
    pub level: Tier,
    pub growth_stacks: GrowthStack, // 영구 성장형 스택
    pub equipped_items: Vec<Uuid>,  // 장비 uuid
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
        // 1) base stats (AbnormalityMetadata)
        let origin = game_data
            .abnormality_data
            .get_by_uuid(&self.base_uuid)
            .ok_or(GameError::MissingResource("AbnormalityMetadata"))?;

        // Abnormality 메타데이터에서 기본 전투 스탯 구성
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

        // TODO: 수치 연산 순서는 매우 중요함.
        // 최종 데미지 증가의 경우 모든 수치가 더해진 마지막에 계산되어야 하는데
        // 중간에 더해지면 안되는 것 처럼.

        // 2) growth/level 스택
        for (stat_id, value) in &self.growth_stacks.stacks {
            match stat_id {
                GrowthId::KillStack => {
                    stats.add_attack(*value);
                }
                GrowthId::PveWinStack => {
                    // TODO: PvE 승리 스택 반영
                }
                GrowthId::QuestRewardStack => {
                    // TODO: 퀘스트 보상 스택 반영
                }
            }
        }

        // 3) 장비 효과
        // 장비 효과에서 퍼센테이지 증가가 존재함.
        // base attack 에서 증가시키는지, 아니면 최종 attack 에서 증가시키는지 명세 작성해야함.
        for item_uuid in &self.equipped_items {
            let origin_item = game_data
                .equipment_data
                .get_by_uuid(item_uuid)
                .ok_or(GameError::MissingResource(""))?;

            stats.apply_permanent_effects(&origin_item.triggered_effects);
        }

        // 4) 덱 전체 아티팩트의 상시 패시브
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
    /// 특정 Side의 아티팩트
    Artifact { side: Side },
    /// 특정 유닛의 장비
    Item { unit_instance_id: Uuid },
}

pub struct RuntimeUnit {
    pub instance_id: Uuid,
    pub owner: Side,
    pub base_uuid: Uuid,
    pub stats: UnitStats,
    pub position: Position,
    /// 현재 집중해서 공격 중인 대상 기물 instance_id (살아 있는 동안 유지)
    pub current_target: Option<Uuid>,
}

impl RuntimeUnit {
    pub fn set_target(&mut self, target: Uuid) {
        self.current_target = Some(target);
    }
}

pub struct BattleCore {
    event_queue: BinaryHeap<BattleEvent>,

    player_info: PlayerDeckInfo,
    opponent_info: PlayerDeckInfo,

    units: HashMap<Uuid, RuntimeUnit>,
    artifacts: HashMap<Uuid, RuntimeArtifact>,
    items: HashMap<Uuid, RuntimeItem>,

    runtime_field: Field,

    game_data: Arc<GameDataBase>,
}

impl BattleCore {
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
            runtime_field: Field::new(field_size.0, field_size.1),
            game_data,
        }
    }

    pub fn build_runtime_units_from_decks(
        &mut self,
        side: Side,
        world: &mut World,
    ) -> Result<(), GameError> {
        // 1. 인벤토리에서 아티팩트 런타임 상태 구성
        let inventory = world
            .get_resource::<Inventory>()
            .ok_or(GameError::MissingResource("Inventory"))?;

        let mut artifact_base_uuids: Vec<Uuid> = Vec::new();

        for artifact in inventory.artifacts.get_all_items() {
            let instance_id = Uuid::new_v4();
            artifact_base_uuids.push(artifact.uuid);
            self.artifacts.insert(
                instance_id,
                RuntimeArtifact {
                    instance_id,
                    owner: side,
                    base_uuid: artifact.uuid,
                },
            );
        }

        // 2. 빌드 대상 덱 선택
        let deck = match side {
            Side::Player => &self.player_info,
            Side::Opponent => &self.opponent_info,
        };

        // 3. 각 OwnedUnit을 RuntimeUnit / RuntimeItem으로 변환
        for unit in &deck.units {
            let stats = unit.effective_stats(&self.game_data, artifact_base_uuids.as_slice())?;

            let position = deck
                .positions
                .get(&unit.base_uuid)
                .copied()
                .ok_or(GameError::UnitNotFound)?;

            let unit_instance_id = Uuid::new_v4();

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

            for equipment_uuid in &unit.equipped_items {
                let item_instance_id = Uuid::new_v4();
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

    pub fn run_battle(&mut self, world: &mut World) -> Result<BattleResult, GameError> {
        // 1. 덱 → Runtime* 변환 (유닛/장비/아티팩트)
        self.build_runtime_units_from_decks(Side::Player, world)?;
        self.build_runtime_units_from_decks(Side::Opponent, world)?;

        // 2. RuntimeField 구성
        self.build_runtime_field()?;

        // 3. 이벤트 큐 초기화 및 첫 공격 이벤트 등록
        self.event_queue.clear();
        self.init_initial_events();

        // 전투 시간 상한 (스켈레톤: 60초)
        const MAX_BATTLE_TIME_MS: u64 = 60_000;
        let mut current_time_ms = 0u64;

        // 3. 이벤트 루프 – 가장 이른 이벤트부터 처리
        while let Some(event) = self.event_queue.pop() {
            current_time_ms = event.time_ms();

            if current_time_ms > MAX_BATTLE_TIME_MS {
                return Ok(BattleResult {
                    winner: BattleWinner::Draw,
                    timeline: Timeline {},
                });
            }

            self.process_event(event)?;

            // 승패 체크: Side별 유닛 수 확인
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

        // 이벤트가 더 이상 없으면 무승부로 처리
        Ok(BattleResult {
            winner: BattleWinner::Draw,
            timeline: Timeline {},
        })
    }

    /// 현재 RuntimeUnit 들을 기준으로 초기 공격 이벤트를 등록한다.
    fn init_initial_events(&mut self) {
        for unit in self.units.values() {
            self.event_queue.push(BattleEvent::Attack {
                time_ms: unit.stats.attack_interval_ms,
                attacker_instance_id: unit.instance_id,
            });
        }
    }

    fn apply_attack(&mut self, attacker_instance_id: Uuid) {
        // 1. 공격자/타겟 정보 확인
        let Some(attacker) = self.units.get(&attacker_instance_id) else {
            return;
        };

        let Some(target_id) = attacker.current_target else {
            return;
        };

        let attacker_attack = attacker.stats.attack;
        let _attacker_owner = attacker.owner;

        let Some(target) = self.units.get(&target_id) else {
            return;
        };

        let target_defense = target.stats.defense;

        // 2. OnAttack 트리거 수집 (공격자 측)
        let on_attack_effects =
            self.collect_all_triggers(attacker_instance_id, TriggerType::OnAttack);

        // 3. 기본 데미지 계산 (attack - defense, 최소 1)
        let base_damage = attacker_attack.saturating_sub(target_defense).max(1);

        // 4. OnAttack 효과 적용
        let mut damage = base_damage as i32;
        let mut on_attack_abilities = Vec::new();

        for effect in &on_attack_effects {
            match effect {
                Effect::BonusDamage { flat, percent } => {
                    damage += flat;
                    damage += damage * percent / 100;
                }
                Effect::Ability(ability_id) => {
                    on_attack_abilities.push(*ability_id);
                }
                _ => {}
            }
        }

        // OnAttack 어빌리티 실행
        for ability_id in on_attack_abilities {
            self.execute_ability(ability_id, attacker_instance_id, Some(target_id));
        }

        // 5. OnHit 트리거 수집 (피격자 측)
        let on_hit_effects = self.collect_all_triggers(target_id, TriggerType::OnHit);

        // 6. OnHit 효과 적용
        let mut on_hit_abilities = Vec::new();

        for effect in &on_hit_effects {
            match effect {
                Effect::BonusDamage { flat, percent } => {
                    damage += flat;
                    damage += damage * percent / 100;
                }
                Effect::Heal { flat, percent: _ } => {
                    // 피격 시 회복
                    if let Some(target) = self.units.get_mut(&target_id) {
                        target.stats.current_health =
                            (target.stats.current_health as i32 + flat).max(0) as u32;
                        target.stats.current_health =
                            target.stats.current_health.min(target.stats.max_health);
                    }
                }
                Effect::Ability(ability_id) => {
                    on_hit_abilities.push(*ability_id);
                }
                _ => {}
            }
        }

        // OnHit 어빌리티 실행
        for ability_id in on_hit_abilities {
            self.execute_ability(ability_id, target_id, Some(attacker_instance_id));
        }

        // 7. 최종 데미지 적용 (최소 0)
        let final_damage = damage.max(0) as u32;

        let target = self.units.get_mut(&target_id).unwrap();
        target.stats.current_health = target.stats.current_health.saturating_sub(final_damage);

        let target_died = target.stats.current_health == 0;
        let target_owner = target.owner;

        // 8. 사망 처리
        if target_died {
            // OnDeath 트리거 (피격자) - 유닛 제거 전에 실행
            let on_death_effects = self.collect_all_triggers(target_id, TriggerType::OnDeath);

            for effect in on_death_effects {
                match effect {
                    Effect::Ability(ability_id) => {
                        self.execute_ability(ability_id, target_id, None);
                    }
                    Effect::Modifier(_modifier) => {
                        // OnDeath Modifier는 보통 의미 없지만 일단 처리
                    }
                    _ => {}
                }
            }

            // OnAllyDeath 트리거 (같은 편 유닛들)
            let ally_ids: Vec<Uuid> = self
                .units
                .iter()
                .filter(|(id, u)| **id != target_id && u.owner == target_owner)
                .map(|(id, _)| *id)
                .collect();

            for ally_id in ally_ids {
                let on_ally_death_effects =
                    self.collect_all_triggers(ally_id, TriggerType::OnAllyDeath);

                for effect in on_ally_death_effects {
                    match effect {
                        Effect::Ability(ability_id) => {
                            self.execute_ability(ability_id, ally_id, None);
                        }
                        Effect::Modifier(modifier) => {
                            if let Some(ally) = self.units.get_mut(&ally_id) {
                                ally.stats.apply_modifier(modifier);
                            }
                        }
                        _ => {}
                    }
                }
            }

            // OnKill 트리거 (공격자)
            let on_kill_effects =
                self.collect_all_triggers(attacker_instance_id, TriggerType::OnKill);

            for effect in on_kill_effects {
                match effect {
                    Effect::Modifier(modifier) => {
                        if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                            attacker.stats.apply_modifier(modifier);
                        }
                    }
                    Effect::Ability(ability_id) => {
                        self.execute_ability(ability_id, attacker_instance_id, Some(target_id));
                    }
                    _ => {}
                }
            }

            // 유닛 제거 및 필드에서 제거
            self.units.remove(&target_id);
            self.runtime_field.remove(target_id);

            // 공격자 타겟 초기화
            if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                attacker.current_target = None;
            }
        }
    }

    /// 트리거 효과 수집
    fn collect_triggers(&self, source: TriggerSource, trigger: TriggerType) -> Vec<Effect> {
        let mut effects = Vec::new();

        match source {
            TriggerSource::Artifact { side } => {
                for artifact in self.artifacts.values().filter(|a| a.owner == side) {
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
                for item in self
                    .items
                    .values()
                    .filter(|i| i.owner_unit_instance == unit_instance_id)
                {
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

    /// 어빌리티 실행
    fn execute_ability(&mut self, ability_id: AbilityId, caster_id: Uuid, target_id: Option<Uuid>) {
        match ability_id {
            AbilityId::ScorchedExplosion => {
                // 사망 시 주변 적에게 데미지
                if let Some(caster) = self.units.get(&caster_id) {
                    let caster_pos = caster.position;
                    let caster_owner = caster.owner;

                    let nearby_enemies: Vec<Uuid> = self
                        .units
                        .iter()
                        .filter(|(_, u)| {
                            u.owner != caster_owner && caster_pos.manhattan(&u.position) <= 1
                        })
                        .map(|(id, _)| *id)
                        .collect();

                    for enemy_id in nearby_enemies {
                        if let Some(enemy) = self.units.get_mut(&enemy_id) {
                            let damage = 30u32; // 폭발 데미지
                            enemy.stats.current_health =
                                enemy.stats.current_health.saturating_sub(damage);
                        }
                    }
                }
            }
            AbilityId::PlagueMassHeal => {
                // 아군 전체 회복
                if let Some(caster) = self.units.get(&caster_id) {
                    let caster_owner = caster.owner;
                    let heal_amount = 20u32;

                    let ally_ids: Vec<Uuid> = self
                        .units
                        .iter()
                        .filter(|(_, u)| u.owner == caster_owner)
                        .map(|(id, _)| *id)
                        .collect();

                    for ally_id in ally_ids {
                        if let Some(ally) = self.units.get_mut(&ally_id) {
                            ally.stats.current_health = (ally.stats.current_health + heal_amount)
                                .min(ally.stats.max_health);
                        }
                    }
                }
            }
            AbilityId::RedShoesBerserk => {
                // 추가 공격 (현재 타겟에게)
                if let Some(target) = target_id {
                    if let Some(target_unit) = self.units.get_mut(&target) {
                        let bonus_damage = 15u32;
                        target_unit.stats.current_health = target_unit
                            .stats
                            .current_health
                            .saturating_sub(bonus_damage);
                    }
                }
            }
            AbilityId::FragmentOfUniverseNova => {
                // 전체 적에게 데미지
                if let Some(caster) = self.units.get(&caster_id) {
                    let caster_owner = caster.owner;
                    let damage = 25u32;

                    let enemy_ids: Vec<Uuid> = self
                        .units
                        .iter()
                        .filter(|(_, u)| u.owner != caster_owner)
                        .map(|(id, _)| *id)
                        .collect();

                    for enemy_id in enemy_ids {
                        if let Some(enemy) = self.units.get_mut(&enemy_id) {
                            enemy.stats.current_health =
                                enemy.stats.current_health.saturating_sub(damage);
                        }
                    }
                }
            }
            AbilityId::SpiderBudPoisonStack => {
                // TODO: 독 스택 적용 (버프 시스템 필요)
            }
            AbilityId::FairyFestivalBlessing => {
                // 아군 전체 공격력 버프
                if let Some(caster) = self.units.get(&caster_id) {
                    let caster_owner = caster.owner;

                    let ally_ids: Vec<Uuid> = self
                        .units
                        .iter()
                        .filter(|(_, u)| u.owner == caster_owner)
                        .map(|(id, _)| *id)
                        .collect();

                    for ally_id in ally_ids {
                        if let Some(ally) = self.units.get_mut(&ally_id) {
                            ally.stats.attack = ally.stats.attack.saturating_add(5);
                        }
                    }
                }
            }
            AbilityId::UnknownDistortionStrike => {
                // 단일 타겟 강력 공격
                if let Some(target) = target_id {
                    if let Some(target_unit) = self.units.get_mut(&target) {
                        let damage = 50u32;
                        target_unit.stats.current_health =
                            target_unit.stats.current_health.saturating_sub(damage);
                    }
                }
            }
        }
    }

    /// 단일 BattleEvent 처리 스켈레톤.
    /// 실제 데미지/타게팅/버프 로직은 추후 구현한다.
    fn process_event(&mut self, event: BattleEvent) -> Result<(), GameError> {
        let event_time_ms = event.time_ms();

        match event {
            BattleEvent::Attack {
                attacker_instance_id,
                ..
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
                self.apply_attack(attacker_instance_id);

                // 5. 다음 공격 예약
                self.event_queue.push(BattleEvent::Attack {
                    time_ms: event_time_ms.saturating_add(interval_ms),
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
