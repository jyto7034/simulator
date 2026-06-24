# 컴포넌트 설계 검토 (리팩토링 Goal 도출용)

이 문서는 `core_enhanced`를 컴포넌트별로 분리해 설계 구조를 검토한 결과다. 목적은 리팩토링 goal 문서를 작성하기 위한 후보 식별이며, 모든 판단은 `refactor_preparation_plan.md`의 기준(source of truth 축소, 열어야 하는 파일 수 축소, 레거시 제거, 과도한 추상화 방지)을 따른다.

작성일: 2026-06-12. 코드가 바뀌면 이 문서의 근거 라인도 낡는다. goal 착수 시점에 근거를 재확인한다.

## 판정 기준 요약

각 문제 후보는 다음 중 하나로 판정한다.

- **goal 후보**: 실측된 고통(churn, source of truth 중복, 산탄 수술)이 확인됨. 별도 goal 문서로 분리할 가치가 있음.
- **보류**: 문제는 맞지만 지금 고치는 비용 대비 이득이 불명확하거나, 진행 중인 다른 goal과 겹침.
- **하지 않음**: 문제처럼 보이지만 고치면 안 되는 것(클라이언트 계약, 정당한 구조, 확장성 예감 기반 제안).

churn 참고치 (최근 3개월, `git log` 기준): `world.rs`, `data/mod.rs`, `behavior.rs`, `battle/types.rs`, `movement/engine.rs`가 각 5회로 최다. battle/core 전반(sim, commands, mod, build, triggers, movement)이 각 4회.

---

## 1. 전투 코어 (battle/core)

### 책임 지도

- `core/sim.rs` (3,018줄): 이벤트 루프, `start/step/finalize_battle_execution`, `apply_live_command`, 기본 공격 스케줄링, 스킬 스텝 실행 진입점.
- `core/mod.rs` (2,998줄): `BattleCore` 상태 저장소(units, buffs, timeline, event_queue), 레조넌스/자동캐스트 스케줄링, 테스트.
- `core/commands.rs` (2,601줄): `process_commands` 명령 처리, 피해/치유/버프 적용, 기본 공격 해석(`resolve_basic_attack`), 기본 공격 투사체 시뮬레이션.
- `core/build.rs`: 시나리오 → 런타임 변환, 라이브 배치(`deploy_player_unit`)/철수.
- `recording.rs` + `timeline.rs`: append-only event log. 정책 문서와 일치함(replay source 아님).

### 문제 후보

**1-A. 빈 `battle/replay/` 디렉토리 — goal 후보 (즉시 정리 가능)**

`src/game/battle/replay/`는 파일이 0개인 빈 디렉토리이며 코드 참조도 없다(확인됨). 정책상 replay-only 경로는 레거시다. 디렉토리 삭제만으로 끝나는 최소 비용 정리.

**1-B. 기본 공격 정책의 3분할 — goal 후보**

"기본 공격을 언제 시작하고 언제 명중시키는가"가 세 곳에 흩어져 있다.

- 스케줄링: `sim.rs:1606` `try_start_pending_basic_attacks`
- 타겟 선택: `sim.rs:1558` `select_basic_attack_target`
- 해석/피해/투사체: `commands.rs:1064` `resolve_basic_attack`, `commands.rs:384` `spawn_basic_attack_projectile`, `commands.rs:742` `advance_basic_attack_projectile`

기본 공격 한 정책을 바꾸려면 sim.rs와 commands.rs를 동시에 열어야 한다. battle/core는 churn 상위권이므로 이 산탄은 반복 비용이다. 단, sim/commands/mod 3파일 전체 재구성은 한 goal 범위를 초과한다. "기본 공격 수명주기 한 곳으로 모으기" 정도로 좁게 잡는다.

**1-C. sim.rs / mod.rs / commands.rs 경계 모호 — 보류**

3,000줄급 파일 3개가 같은 `BattleCore`에 impl을 나눠 갖고 있고, `process_event`의 소속이 한눈에 안 보인다. 그러나 순환 의존은 없고 이벤트 루프 자체는 결정론적으로 잘 동작한다. 파일을 더 쪼개는 것은 "파일은 쪼갰지만 책임은 그대로 섞이는" 실패 패턴 위험이 크다. 1-B 같은 정책 단위 응집을 먼저 하고, 그 결과로 파일 경계가 자연스럽게 드러나면 그때 분리한다.

**1-D. `Timeline` 이름의 이중 의미 — 하지 않음**

`record_timeline` 같은 내부 이름은 역사적 명명으로 남아 있지만, Unity-facing live 전송은 `battle_update.events_delta`/`battle_update.checkpoint` 계약으로 정리됐다. 새 코드나 문서는 구형 `timeline_delta`/`battle_delta` 전송을 다시 만들지 않는다.

### 건강한 부분

- 이벤트 큐(BinaryHeap) 기반 결정론적 시뮬레이션 루프, 라이브 명령이 같은 이벤트 큐로 합류하는 설계.
- `recording.rs`의 인과 추적(`with_recording_context`) — append API가 단순하고 delta 추출(`entries_after_seq`)이 용이.
- 라이브 배치가 시나리오 스폰과 같은 경로(`deploy_player_unit` → `ScenarioSpawnGroup` 추가 → `spawn_scenario_group`)를 재사용하는 점 (`build.rs:226-267`).

---

## 2. 이동 시스템 (battle/core/movement)

### 책임 지도

- `engine.rs` (1,203줄): `MovementEngine` trait, 입출력 타입, `ContinuousMovementBackend` enum(Direct/Rapier), `DirectContinuousMovement` 구현, sim.rs 연결점.
- `rapier_backend.rs` (1,134줄): Rapier2D 물리 래핑, 유닛/장애물 동기화, `MovementEngine` 구현.
- `planner.rs`: 이동 목표 생성. `steering.rs`: 순수 기하 계산. `blocking.rs`: 블로킹 상태. `lifecycle.rs`: 이동 중단 기록.

### 문제 후보

**2-A. Direct/Rapier 두 백엔드의 틱 로직 중복 — goal 후보 (단, 방향 주의)**

두 `tick()` 구현이 거의 동일한 제어 흐름(죽은 유닛 필터링 → 목표 도달 확인 → 목표 위치 계산 → 스티어링/경계 고정 → 장애물 처리 → 위치 갱신)을 반복한다 (`engine.rs:230-338` vs `rapier_backend.rs:476-610`, 중복 ~150-200줄). Direct는 프로덕션에서 사용되지 않고 테스트 전용이다(기본값은 Rapier, `engine.rs:152`; Direct 생성은 `engine.rs:439` `#[cfg(test)]` 경로).

이동 정책 하나를 바꾸면 두 파일을 동시에 수정해야 하며, `movement/engine.rs`는 churn 최상위권(5회)이므로 실측된 고통이다.

방향은 두 가지이며 goal 착수 시 먼저 선택한다.

- (a) **틱 루프 공통화**: 필터링/목표/스티어링을 공통 루프로 모으고 백엔드는 "충돌 해석" 한 책임만 갖게 좁힌다. Direct를 빠른 결정론적 테스트 하니스로 유지할 가치가 있을 때 택한다.
- (b) **Direct 백엔드 제거**: Rapier만 남기고 기존 Direct 기반 테스트를 Rapier 기반으로 강화한다. source of truth 축소 폭은 더 크지만, 테스트 속도/결정성에서 Direct가 주던 이점을 먼저 측정해야 한다.

어느 쪽이든 백엔드를 늘리는 식의 추상화 강화는 하지 않는다 — 실제 변형은 2개이고 그중 1개는 테스트 전용이다.

**2-B. 타일 좌표 레거시 혼재 — 보류**

`movement/types.rs:3-8`의 `TILE_UNITS_PER_TILE` 등 타일 상수는 런타임 이동(순수 월드 좌표)에서는 쓰이지 않지만, 주석대로 포메이션 입력 등에서 타일 좌표가 여전히 유용하다. `tile_range.rs`는 스킬 범위의 source of truth(`defense_tile_range`)로 live 계약이다. 좌표계 정리는 스킬 시스템과 얽혀 있으므로 단독 goal로 잡지 않는다.

### 건강한 부분

- `MovementTickInput`/`MovementOutput`의 명확한 입출력 계약 (`engine.rs:74-128`).
- planner(목표 결정) / steering(기하 계산) 분리 — 독립 진화 가능.
- 풍부한 결정론적 테스트(engine 27개, rapier 40+개, planner 46개). 2-A 공통화 시 이 테스트들이 행동 보존의 안전망이 된다.

---

## 3. 스킬 시스템 (skill_runtime, ability, skill_data)

### 책임 지도

- `ability.rs` (929줄): 스킬 정의 타입(SkillDef, SkillStepDef, DeliveryDef, SkillTarget 등). 순수 선언, 부작용 없음.
- `skill_runtime/cast.rs`: 시전 수명주기(스텝 시작/결과 기록/완료/연기, damage gate).
- `skill_runtime/projectile.rs`: 발사체 물리(고정/유도), 충돌 판정.
- `skill_runtime/area.rs`: 타일 범위 영역(앵커/추적/틱 정책/만료).
- `data/skill_data.rs`: RON → SkillDatabase 변환, preset 해석, 로드 시 검증.
- `skill_target_contract.md`: `defense_tile_range`를 단일 source of truth로 선언한 계약 문서.

### 문제 후보

**3-A. `StepTargetingMode`가 live 정책인데 계약 문서에 없음 — goal 후보 (문서화)**

`SkillStepDef`에 `target: SkillTarget`과 `targeting: StepTargetingMode`(`ability.rs:346, 483`)가 공존한다. 확인 결과 이는 레거시가 아니라 live 정책이다: `game_resources/data/skills/base.ron`에 `targeting: RetargetOnStep`이 다수 사용 중이고(246, 330, 420, 596, 821행 등), `sim.rs:692`가 `step.targeting`으로 ReuseCastTarget/RetargetOnStep을 실제 분기한다. 코드 제거 대상이 아니라 **`skill_target_contract.md`에 두 모드의 의미와 `target`과의 관계를 명문화**하는 문서 goal이다. 계약 문서가 live 스키마 필드를 누락하고 있는 상태가 문제의 본질이다.

**3-B. 범위 기하와 전달 정책의 이중 참조 — 보류**

`TileArea` 실행 시 `SkillStepDef.defense_tile_range`(기하)와 `DeliveryDef::TileArea`의 `SkillTileAreaDeliveryDef`(앵커/추적)를 함께 읽는다 (`area.rs:247` 부근). 다만 이는 "어디를 맞출지(제약)"와 "어디서 투영할지(보조정책)"라는 서로 다른 축이며, 계약 문서가 이미 이 구분을 설명한다. 합치면 오히려 계약이 흐려질 수 있다. 실제로 두 곳을 동시에 잘못 수정한 버그가 발생하면 그때 재검토한다.

**3-C. 타겟팅 규칙 추가 시 다중 파일 수정 — 하지 않음**

`UnitTargetRule` 변형 추가 시 ability.rs(enum) + cast.rs(match) + 계약 문서를 수정해야 하지만, 이는 Rust enum 기반 설계의 자연스러운 비용이며 컴파일러가 누락을 잡아준다. 동적 dispatch나 registry로 바꾸는 것은 정책 문서가 경계하는 과도한 추상화다.

**3-D. (정정) `ProjectileRecord`는 죽은 코드가 아님 — 하지 않음**

분석 과정에서 레거시 의심이 제기됐으나 교차 검증 결과 `commands.rs:409` 등에서 기본 공격 투사체용으로 live 사용 중이다(`core/mod.rs:56` 필드 보유). 스킬 발사체(`ActiveProjectileRuntime`)와 기본 공격 발사체(`ProjectileRecord`)는 별개 런타임이며, 이 분리가 적절한지는 1-B(기본 공격 수명주기 goal) 범위에서 함께 본다.

### 건강한 부분

- `ability.rs`의 데이터 정의 계층: serde 전용, 다른 game 모듈로 의존하지 않음. 손대지 않는다.
- 레거시 geometric shape(`DeliveryDef::Area`, Circle/Cone 등)는 schema/runtime에서 이미 제거 완료 — 계약 문서 선언과 코드가 일치.
- cast.rs의 수명주기 메서드들이 각각 한 가지 일만 함. projectile의 sweep 충돌 수학은 spatial.rs로 외부화되어 있고 테스트 존재.
- skill_data.rs의 preset 해석 + 로드 시 panic 검증(조기 실패).

---

## 4. 게임 흐름 (world, map, resources, managers)

### 책임 지도

- `GameState` 정의: `resources/state.rs:6-52`. 전이 실행: `world/helpers.rs:616` `transition_to()`가 상태와 allowed_actions를 원자적으로 갱신.
- `world/node_flow.rs`, `map_content.rs`: 노드 진입/세션 생성. `world/{shop,reward,support,headquarters,maintenance,combat}.rs`: 도메인별 행동 처리.
- `map/progression.rs`: 노드 그래프 진행 상태. `map/executor.rs`: 노드 진입 시 일회성 데이터.
- `managers/uuid_manager.rs`: run_seed 기반 결정적 UUID.
- 순환 의존 없음: world → map/resources/managers 단방향.

### 문제 후보

**4-A. allowed_actions 결정 로직의 2분할 — goal 후보**

기본 목록은 `managers/action_scheduler.rs:11-92`(GameState → 액션 목록 순수 함수), 보정은 `world/helpers.rs:104-130` `allowed_actions_for_state_context()`(InReward의 ExitReward 조건부 제거, Maintenance 노드의 장비/스킬 행동 추가). "이 상태에서 무엇이 가능한가"라는 한 정책이 두 파일에 있고, 특히 Maintenance 보정은 `active_node_content` 존재 여부에 암묵적으로 의존한다. 한 곳으로 모으는 goal 가치가 있다.

**4-B. `resources/`의 낮은 응집 — 보류**

`resources/`에 게임 흐름 상태(state, action, selection)와 무관한 도메인(economy의 Enkephalin, inventory, item_slot, board의 RosterOrder)이 혼재한다. 다만 이는 "읽기 불편함"이지 source of truth 중복이나 산탄 수술이 아니다. 파일 이동은 diff 노이즈가 크므로, 다른 goal이 해당 파일을 크게 수정할 때 같이 옮기는 기회비용 0의 방식을 우선한다.

**4-C. `ActionScheduler` 이름 — 하지 않음 (단독으로는)**

schedule을 하지 않는 순수 조회 함수라 이름이 과대하지만, 이름 변경만을 위한 goal은 만들지 않는다. 4-A 진행 시 자연스럽게 함께 정리한다.

### 건강한 부분

- `transition_to()`의 원자적 상태+액션 갱신 (`world/state.rs:419-422`) — 동기화 불변식이 구조적으로 보장됨.
- `ActiveNodeContent` enum의 type-safe 세션 컨테이너(`as_shop()` 등 안전 캐스팅).
- `map/progression.rs`의 노드 진행이 선형적이고 사이클 방지가 명확.
- 결정적 UUID(`uuid_manager.rs:29-52`) — 재현성의 기반. 손대지 않는다.

---

## 5. 아이템 / 장비 / 경제

### 책임 지도

- 정의: `data/{equipment,consumable,artifact,reward,shop}_data.rs` (메타데이터 + 인덱스 + 검증).
- 보유: `resources/inventory.rs` (Equipment/Consumable/Artifact/Material 인벤토리), `resources/item_slot.rs` (직원 장착 슬롯).
- 거래: `world/shop.rs` (구매/판매). 보상: `reward.rs` (효과 실행) + `reward_policy.rs` (태그 검증) + `world/reward.rs` (세션).
- 장착: `world/maintenance.rs` (착용/해제/조합).

### 문제 후보

**5-A. 아티팩트 제거 미구현 (TODO 방치) — goal 후보 (소형)**

`inventory.rs:311-314`에 "ArtifactSlots에 remove_by_uuid 추가 필요" TODO가 있고 `remove_item()`이 아티팩트에 대해 조용히 `None`을 반환한다. 아티팩트가 정책상 영구 귀속이라면 그것을 명시적 에러로 표현하고 TODO를 제거해야 하고, 제거 가능해야 한다면 구현해야 한다. 현재는 정책이 코드에 표현되지 않은 상태다. 정책 확인이 필요하므로 착수 전 사용자 확인 1건 포함.

**5-B. `Item::Abnormality`의 전면 거부 패턴 — goal 후보 (소형)**

`inventory.rs:277,322`, `world/shop.rs:97` 등 여러 곳에서 `Item::Abnormality`를 개별적으로 거부한다. 주석(`inventory.rs:251-254`)대로 환상체가 더 이상 플레이어 소유물이 아니라면, Item enum에서 해당 변형을 제거하거나 인벤토리 진입 불가를 타입으로 표현하는 것이 거부 코드 산재보다 낫다. 단 Item enum은 ItemRegistry/RON과 얽혀 있으므로 영향 범위 조사가 선행되어야 한다.

**5-C. 보유 표현의 3가지 패턴 (OwnedEquipment / OwnedConsumable / Arc 직접) — 하지 않음**

장비는 `equipped_to`가 필요하고 소모품은 즉시 소비되며 아티팩트는 귀속이라 수명주기가 실제로 다르다. "일관성을 위해" 통일하는 것은 미래 확장성 예감 기반 추상화다. 각 타입의 현재 요구에 맞는 표현이면 충분하다.

**5-D. 인벤토리 검증 중복 — 보류**

용량/중복/착용 검증이 `inventory.rs`의 `can_add_item()`과 호출처(shop.rs, reward.rs, helpers.rs)에서 반복된다. 다만 호출처 검증은 "사전 검증으로 더 친절한 에러를 주기 위한" 것일 수 있어, 단순 제거가 아니라 검증 결과 타입을 공유하는 설계가 필요하다. 5-A/5-B 진행 중에 같이 보는 정도로 둔다.

### 건강한 부분

- 소유(instance_uuid) / 메타(base_uuid) 분리, OnceLock 인덱스 + `validate_indexes()` 패턴의 일관성.
- `ItemSlot`의 슬롯 레이아웃 추상화(ByType/Any3) — 실제 두 변형이 존재하는 정당한 추상화.
- 결정적 UUID로 소유 아이템 생성(재현성).

---

## 6. 효과 / 버프 / 스탯

### 책임 지도

- `stats.rs`: UnitStats, StatModifier(Flat/Percent), TriggeredEffect(Permanent/OnAttack/...). 수정 로직은 `apply_modifier` 단일 구현.
- `battle/buffs.rs`: BuffDatabase(전투 상태이상: PeriodicDamage/Stun/Freeze/Silence), ms 단위 수명.
- `employee.rs:67-94`: ActiveConsumableModifier, 노드 단위 수명.
- `battle/damage.rs`: 데미지 공식, DamageModifiers.

### 문제 후보

**6-A. 최종 스탯 합산 경로의 3분할과 암묵적 순서 — goal 후보**

전투 진입 시 최종 스탯이 세 단계에서 누적 수정된다.

1. `battle/types.rs:338-424` `effective_stats()`: 기본 + 성장 스택 + 장비 Permanent/강화 + 아티팩트 Permanent
2. `skill_fragment.rs:814-815`: 스킬프래그먼트의 attack 수정
3. `employee.rs:247-284` `apply_consumable_battle_profile_effects()`: 소모품의 stats/damage_reduction 수정

호출 순서가 `employee.rs:218-224`에 암묵적으로 코딩되어 있다(프래그먼트 → HP 보정 → 소모품). "스탯이 어떻게 결정되는가"는 게임 룰의 핵심인데 single source가 없다. 합산 파이프라인을 한 함수로 모으고 순서를 명시하는 goal 가치가 있다. 단, 버프와 소모품 runtime을 합치는 것은 정책상 금지(수명주기가 다름)이므로 범위에서 제외한다.

**6-B. BuffDatabase → GameDataBase 통합 — 진행 중 goal (이 문서 범위 밖)**

`data/mod.rs:132`에 `buff_data: Arc<BuffDatabase>`로 이미 GameDataBase 필드로는 들어가 있으나, 정책 문서가 말하는 통합 goal은 별도로 진행 중이다. 이 문서에서는 중복 goal을 만들지 않는다.

### 건강한 부분

- `apply_modifier` 단일 구현 — Flat/Percent 계산이 중앙집중화. 손대지 않는다.
- 버프(ms, 전투 중) / 소모품(노드 단위) / Permanent(전투 전 고정)의 수명주기 분리가 코드에서 명확 — 정책 문서와 일치.
- 죽은 코드 미발견.

---

## 7. 데이터 계층 / 전투 셋업 (data, combat_setup, combat_preview)

### 책임 지도

- `data/mod.rs` (1,531줄): GameDataBase 조합, 제너릭 인덱스 빌더, Item/ItemRegistry, 교차 참조 검증 7종, 테스트 ~1,000줄.
- 각 `*_data.rs`: Database + Metadata + OnceLock 인덱스 패턴(대체로 일관).
- `combat_setup/`: mission_policy(307줄)만 실질 내용, 나머지(balance, rewards 등)는 ~70줄 이하 스텁.
- `combat_preview/`: threat/validation/template/types는 분리 완료, `mod.rs`가 여전히 2,016줄(전장 생성 + 침식 직원 웨이브 생성 + 테스트 ~1,000줄).

### 문제 후보

**7-A. `data/mod.rs`의 교차 참조 검증 전부 재분류 — goal 후보**

mod.rs 안의 교차 참조 검증 함수는 현재 7종이다: `validate_skill_fragment_skill_references`(669행), `validate_defense_route_skill_contract`(764행), `validate_unit_skill_references`(789행), `validate_reward_references`(829행), `validate_pve_enemy_references`(906행), `validate_combat_preview_threat_warning_contract`(1022행), `validate_shop_item_references`(1172행). `data/mod.rs`는 churn 최상위권(5회)이므로, 이들을 `data/validation.rs`로 옮기면 mod.rs는 조합과 조회만 남는다. goal 범위는 특정 함수 목록이 아니라 "mod.rs의 교차 참조 검증 전부 재분류"로 잡는다 — 함수가 추가/변경되어도 범위가 낡지 않는다. 행동 변경 없는 순수 이동이라 리스크가 낮다.

**7-B. `combat_preview/mod.rs`의 침식 웨이브 생성 분리 — goal 후보**

`generate_corroded_wave`(715행) 이후 ~200줄의 침식 직원 웨이브 생성이 전장 생성과 무관하게 mod.rs에 섞여 있다. threat/template/validation을 분리한 기존 패턴을 따라 한 파일 더 분리하는 동질적 작업이다.

**7-C. combat_setup 파일별 실사용 조사 — goal 후보 (조사 우선, 파일별 판정)**

파일 크기가 크게 갈린다: mission_policy.rs 307줄, enemy_spawns.rs 229줄, player_spawns.rs 190줄, defense_object.rs 113줄은 실질 내용이 있고, balance.rs 28줄, rewards.rs 41줄, scenario_groups.rs 43줄, battlefield_plan.rs 72줄은 스텁에 가깝다. "스텁 전체 삭제"가 아니라 **파일별로 live 참조를 조사해 삭제/유지를 개별 판정**한다. 스텁으로 확인된 것만 삭제하고, "미래에 채울 것"이라는 이유만으로 빈 파일을 유지하지 않는다(정책 문서가 경계하는 패턴).

**7-D. `behavior.rs`의 BehaviorResult가 전 계층 결합 — 하지 않음 (현재는)**

`BehaviorResult::BattleState`가 combat_preview, battle timeline, deployment를 모두 담는다. 그러나 이것은 Unity-facing 응답 DTO이며, 클라이언트가 한 응답으로 받아야 하는 데이터가 실제로 그만큼이다. 계약 변경 없이 내부만 쪼개면 어댑터 계층이 생길 뿐이다. 클라이언트 계약이 바뀌는 시점에 재검토한다.

**7-E. 검증 시점 3분산 (로드/조합/생성 후) — 하지 않음**

Database 생성 시 / GameDataBase::new() / combat_preview 생성 후로 검증이 나뉘어 있지만, 이는 각 검증이 필요로 하는 컨텍스트가 다르기 때문이다(단일 DB ↔ 교차 참조 ↔ 생성 결과). 한 곳으로 강제 통합하면 오히려 컨텍스트를 끌어와야 한다. 7-A로 위치만 정리하면 충분하다.

### 건강한 부분

- 제너릭 인덱스 빌더(`build_unique_index` 등, `mod.rs:72-108`)와 Database 패턴의 일관성.
- GameDataBase 생성 실패 = 애플리케이션 시작 불가(빠른 실패).
- combat_preview의 기존 분리(threat/validation/template/types) — 7-B가 따라갈 선례.

---

## 종합: goal 후보 우선순위

판단 근거: (1) churn으로 실측된 고통, (2) source of truth 축소량, (3) 행동 보존 검증 수단의 존재, (4) 작업 크기(한 goal = 경계 하나).

| 순위 | goal 후보 | 근거 항목 | 크기 | 안전망 |
|---|---|---|---|---|
| 1 | data/mod.rs 검증 재분류 + 빈 battle/replay/ 삭제 | 7-A, 1-A | 소 | 순수 이동/삭제, cargo check + 기존 테스트 |
| 2 | 스탯 합산 파이프라인 단일화 | 6-A | 중 | world/tests의 전투/장비 테스트, 결정론 시드 비교 |
| 3 | allowed_actions 단일화 | 4-A | 소-중 | 상태 전이 테스트 |
| 4 | movement 틱 중복 정리 (Direct 유지/제거 먼저 판단) | 2-A | 중 | movement 테스트 100+개 |
| 5 | 기본 공격 수명주기 응집 | 1-B | 중 | battle core 테스트, event log 비교 |
| 6 | StepTargetingMode 계약 문서화 + 아이템 소형 정리 | 3-A, 5-A, 5-B | 소 | 문서 goal + 일부 사용자 확인 필요 |
| 7 | combat_setup 파일별 실사용 조사 후 삭제/유지 | 7-C | 소 | 조사 선행, 파일별 판정 |

1번을 먼저 두는 이유: 행동 변경이 전혀 없는 이동/삭제라 리스크가 0에 가깝고, churn 최상위 파일(`data/mod.rs`)의 후속 작업 비용을 즉시 낮춘다. combat_setup 정리(7-C)는 같은 "정리" 성격이지만 조사가 선행되어야 하므로 1번에 묶지 않고 분리했다. movement(4번)는 착수 전 Direct 백엔드 유지/제거 선택이 필요해 allowed_actions(3번)보다 뒤로 보냈다.

## 하지 않을 것 (명시적 제외)

- 구형 Unity-facing live 전투 전송명(`timeline_delta`, `battle_delta`) 복구 — 최신 클라이언트 계약과 충돌.
- 버프 runtime과 소모품 modifier runtime 통합 — 수명주기가 다름(정책 확정).
- 아이템 보유 표현(OwnedEquipment/OwnedConsumable/Arc) 통일 — 확장성 예감 기반.
- 타겟팅 규칙의 동적 dispatch/registry화 — enum + match가 적정 수준.
- sim/mod/commands 3파일의 일괄 재구성 — 정책 단위 응집(1-B 등)을 먼저, 파일 분리는 그 결과로.
- BehaviorResult 내부 분해 — 클라이언트 계약 변경 시점에 재검토.
- battle/core의 ECS 전환 등 아키텍처 리라이트.

## 미확정 사항 (goal 착수 전 사용자 확인 필요)

- 아티팩트는 영구 귀속(제거 불가)이 정책인가, 제거 미구현인가? (5-A)
- `Item::Abnormality`는 Item enum에서 제거해도 되는가? RON/레지스트리 영향 조사 후 결정. (5-B)
- movement Direct 백엔드를 결정론적 테스트 하니스로 유지할 것인가, 제거하고 Rapier 테스트로 대체할 것인가? (2-A)
