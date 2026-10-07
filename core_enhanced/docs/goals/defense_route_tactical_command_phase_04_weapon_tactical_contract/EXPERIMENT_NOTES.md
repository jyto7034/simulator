# DefenseRoute 전술 지휘 Phase 4 — Experiment Notes

Status: active

## 초기 구조 판단

- Child 이름은 Master Phase 4의 weapon tactical profile/data/weapon-required gate 범위를 표현하는
  `defense_route_tactical_command_phase_04_weapon_tactical_contract`로 정했다.
- Phase 4는 `DR-10`, `DR-11`의 data contract, existing consumer, starter, reason과 active-run cutover만 소유한다.
- Phase 5 DefenseFocus/Rest 실행과 stance state, Phase 6~9 command를 선행하지 않는다.

## 초기 dirty worktree

- Phase 0~3의 verified production/data/server/canonical/test/Goal output이 dirty 상태로 함께 존재한다.
- 기존 사용자 변경인 README/workflow/audit guide/abnormality skill 문서와 Unity protocol dirty를 보존한다.
- Phase 4 production 수정 전 대상 파일별 시작 diff를 읽어 선행 변경과 겹치는 hunk를 구분한다.

## 경로 충돌

- Phase 4 후보 directory는 초기화 전에 존재하지 않았다.
- 생성한 path는 기존 Goal과 충돌하지 않는다.

## Source-of-truth 판단

- 목표 정책은 decision record `DR-10`, `DR-11`이다.
- Current weapon schema/data/consumer, starter grant, reason와 save version owner는 production discovery 전까지
  확정하지 않는다.
- Phase 3 resource payment와 Phase 1 deploy/redeploy transaction은 weapon-required failure의 immutability
  predecessor constraint다.

## 외부 소비자와 비범위

- game_server persistence/transport는 접근 가능한 consumer로 검증한다.
- Unity runtime/contract/tests는 사용자 승인 비범위라 접근하지 않는다.

## 남은 위험

- Equipment metadata가 active-run roster/inventory 안에 직렬화되면 30개 RON migration과 Core payload version이
  결합될 수 있다.
- Current block/affinity 값이 여러 fields/archetype defaults에 분산되어 single authority cutover 범위가 넓을 수
  있다.
- Weaponless condition priority와 battle candidate filtering이 서로 다른 helpers를 사용할 수 있다.

## 다음 안전한 작업

1. Master Phase 4 activation을 기록한다.
2. Equipment schema→consumer→starter/reward→weaponless gate→active-run/server path를 읽는다.
3. 수정 대상 diff와 focused baseline을 확보한다.

## 확정된 구현 판단

- Tactical authoring authority는 `WeaponCombatProfile` 하나다. `UnitCombatProfile::employee_default`의 내부
  값은 non-weapon unit construction default일 뿐 employee battle fallback으로 사용할 수 없다.
- Required Option key는 explicit `None`과 missing을 구분해야 하므로 custom deserializer를 사용한다.
- `ground_defense_focus`와 `platform_rest` 수치는 Phase 4에서 strict authoring/serialization만 소유한다.
  효과, stance와 tick cadence는 Phase 5가 구현한다.
- Safe-zone compatibility profile은 roster/equipment/fragment 표시용이며 battle draft source가 아니다.
  Weaponless employee의 effective weapon/affinity는 `None`으로 투영한다.
- Condition reason이 weapon requirement보다 우선한다. condition상 가능한 경우에만 `weapon_required`를
  표시하며 lower battle source error도 같은 stable code로 server가 mapping한다.
- Certified membership과 no-deployable 판정은 armed eligibility를 사용한다. Fragment/certified power의
  block 숫자는 authored base block만 사용한다.
- HQ recruitment는 현재 candidate definition의 equipment IDs를 owned/equipped instance로 grant하지 않는다.
  따라서 weaponless roster의 guaranteed recovery path가 아니며 recovery capability로 추정하지 않는다.
- Active-run host envelope version 3과 Core payload version 3은 서로 다른 version owner다. Host envelope는
  유지하고 Core만 V2→V3 strict cutover했다.

## Final source-of-truth 경로

- Schema/validation: `core_enhanced/src/game/data/equipment_data.rs`, `validation.rs`
- Live data: `game_resources/data/equipments/base.ron`, `employees/definitions.ron`,
  `events/rewards/base.ron`
- Battle authority: `battle/types.rs`, `combat_setup/player_spawns.rs`, `world/combat.rs`
- Safe-zone/reason projection: `employee.rs`, `world/helpers.rs`, `world/snapshot.rs`, `behavior.rs`
- Persistence: `world/active_run.rs`, `world/tests/facility.rs`
- Server mapping/fixtures: `game_server/src/game/player_game_actor/state.rs`, `handlers.rs`
- Canonical: `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`

## Compatibility와 legacy 분류

- Core payload V2 reader/migration, missing field fill, weapon archetype inference, weaponless battle fallback,
  dual affinity DTO와 reward alias는 없다.
- Rust `WeaponCombatProfile::default`는 unit/test fixture construction convenience이며 live RON required fields를
  default하지 않는다.
- Corroded employee와 defense object affinity는 non-player unit source이며 employee weapon authority와
  충돌하지 않는다.
- `effective_deployment_affinity`는 equipped weapon에서 계산한 projection이라 중복 base authority가 아니다.

## 비범위와 잔여 위험

- Unity runtime/contract/UI/tests는 읽거나 변경하지 않았다. 기존 Unity protocol dirty는 보존했다.
- Phase 5 Attack/Defense state, Ground DefenseFocus/Platform Rest effect/timer와 tactical resource producer는
  추가하지 않았다.
- 현재 Phase 4 범위의 알려진 correctness gap은 없다. Phase 5는 authored Option profile을 current tile과
  stance에 따라 적용하되 fragment/certified base block 의미를 변경하면 안 된다.
