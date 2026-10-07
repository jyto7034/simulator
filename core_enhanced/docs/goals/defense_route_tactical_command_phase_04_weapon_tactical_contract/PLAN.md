# DefenseRoute 전술 지휘 Phase 4 — 무기 전술 계약

Status: complete

Implementation Complete: true

Goal Type: phase

Execution Mode: implement_and_verify

Production Mutation Authorized: yes — 이 Phase의 `DR-10`, `DR-11` 범위에 한정

Parent Master: [defense_route_tactical_command_master](../defense_route_tactical_command_master/PLAN.md)

Predecessor Audit:
[Phase 3 Complete / High](../defense_route_tactical_command_phase_03_tactical_resource_authority/COMPLETION_AUDIT.md)

Phase: 4

Goal Name: `defense_route_tactical_command_phase_04_weapon_tactical_contract`

Target Repository: `/mnt/f/work/simulator`

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Finding ID: 없음

## 목표

30개 first-live weapon 각각이 required tactical tile affinity, base block, Ground DefenseFocus 또는 Platform
Rest 수치를 RON에서 명시하도록 strict data contract를 확장한다. Starter 6명에게 일반 reward pool 밖
`standard_training_sword`를 지급하고, 무기 없는 직원은 safe-zone roster/equipment에서는 유지하되 battle
gameplay source가 되지 못하도록 condition-priority `weapon_required` capability와 strict active-run version
cutover를 Core/game_server/canonical까지 완결한다.

## 승인된 정책과 근거

- Primary policies: `DR-10`, `DR-11`
- Target source:
  [DefenseRoute 전술 지휘 목표 정책 결정](../../decisions/defense_route_tactical_command_policy_2026-08-06.md#dr-10--무기-전술-데이터-소유권)
- Master boundary:
  [Phase 4](../defense_route_tactical_command_master/PLAN.md#phase-4--무기-전술-계약과-weapon-required-gate)
- Predecessor resource contract:
  [Phase 3 Complete / High](../defense_route_tactical_command_phase_03_tactical_resource_authority/COMPLETION_AUDIT.md)

## 현재 runtime 및 live data 근거

Production 조사와 구현으로 다음 authority를 확정했다.

- `WeaponCombatProfile`이 per-weapon affinity, base block, Ground DefenseFocus/Platform Rest authoring을
  소유하며 `EquipmentDatabase` validation이 tile/profile 조합을 검증한다.
- `BattleUnitDraft::combat_profile`은 employee battle source에 실제 weapon을 요구하고 weapon profile만
  `UnitCombatProfile`에 적용한다.
- `Employee::battle_deployment_disabled_reason`은 condition reason 뒤에 `WeaponRequired`를 계산한다.
  Safe-zone snapshot/fragment compatibility는 별도 non-battle helper로 비무장 직원을 유지한다.
- Starter grant는 unified employee definition의 initial loadout을 실제 owned equipment instance로 만들며,
  reward pool filter는 explicit exclusion IDs를 소유한다.
- Active-run host envelope는 V3, Core payload도 strict V3다. Checkpoint inventory가 embedded equipment
  metadata를 소유하므로 Core V2는 migration 없이 reject한다.
- Phase 4 변경 전 broad baseline은 Core 872, server 55, live RON 23였고 관련 focused baseline은 모두
  green이었다.

## 유지할 불변식

- Tactical profile은 개별 weapon RON만 source이며 archetype/stat/default/effective block에서 보충하지 않는다.
- `GroundOnly`는 DefenseFocus만, `PlatformOnly`는 Rest만, `Any`는 둘 다 required로 명시한다.
- Ground 가능한 weapon의 base block 0은 유효하고 PlatformOnly base block은 0이다.
- First live catalog는 Staff 16, Bow 1, Gun 1, Sword 9, Spear 1, Shield 1, training sword 1의 exact 30개다.
- Staff는 PlatformOnly/Rest 3·5, Bow/Gun은 GroundOnly/base 0/DefenseFocus +0·10%, Sword/Spear는
  GroundOnly/base 1/+1·10%, Shield는 base 3/+1·10%, training sword는 approved Sword baseline이다.
- Fragment `block_capacity_min`과 certified combat power는 base block만 사용한다.
- Weaponless employee는 roster/safe-zone equipment target으로 유지하고 condition상 가능할 때만
  `weapon_required`를 표시한다.
- Battle deploy/redeploy, future movement/swap/stance/range/manual/field work가 fallback profile을 만들 수 없다.
- Starter-eligible 6명은 training sword를 가지고 새 run이 진행 가능해야 하며 training sword는 일반 reward
  pool에 들어가지 않는다.
- Active-run old/new Core schema를 병행하거나 missing tactical profile을 보완하지 않는다.
- Phase 5 stance/effect state, timer, command와 Phase 6~9 placeholder를 추가하지 않는다.
- Phase 0~3 output과 기존 사용자 dirty 파일을 보존한다.

## 범위

- Required weapon tactical profile Rust schema와 strict validation
- Exact 30 weapon live RON migration 및 training sword authoring
- Existing affinity/block/fragment/power consumer를 new single authority에 연결
- Starter 6명 training sword grant/validation과 reward exclusion
- Weapon-required typed eligibility/reason priority와 battle gameplay gates
- Active-run strict Core payload version cutover와 game_server persistence regression
- Checkpoint/root/server projection에 필요한 current reason mapping
- Gameplay/data/runtime canonical 갱신과 Complete / High audit

## 명시적 비범위

- Phase 5 Attack/Defense state, DefenseFocus/Rest runtime effect와 cadence
- Phase 6 relocation, Phase 7 swap, Phase 8 manual target shape, Phase 9 FieldWork command 구현
- `Any` first-live weapon content
- Weaponless fallback combat profile
- Unity runtime/contract/UI/tests
- 승인되지 않은 save/DTO compatibility

## 미결정 정책

현재 알려진 사용자-visible 미결정 항목은 없다. Current schema 조사에서 30개 catalog 구성 또는 persisted
version ownership이 target policy와 충돌하거나 새로운 tuning 선택이 필요하면 영향 변경을 중단하고
`policy_decision_required`로 전환한다.

## 구현 계획

1. Equipment schema→consumer→starter/reward→weaponless gates→active-run/server path를 읽는다.
2. 수정 대상 시작 diff와 focused baseline을 production 변경 전에 확보한다.
3. Required tactical profile type/validation과 exact live weapon RON 30개를 strict migration한다.
4. Existing affinity/base-block/fragment/power consumers를 single authored profile에 연결한다.
5. Training sword와 starter/reward exclusion을 구현한다.
6. Weapon-required reason priority와 current battle entry/deploy/redeploy/range/manual gates를 완결한다.
7. Active-run Core version과 server persistence를 strict cutover한다.
8. Focused/broad/static/canonical과 completion audit을 수행한다.

## 완료 조건

- `DR-10`, `DR-11`의 exact 30 weapon data와 weaponless meaning이 구현된다.
- 모든 live weapon이 usable tile에 맞는 required profile을 명시하고 invalid/missing/mismatched schema가 거부된다.
- Existing block/affinity/fragment/power consumer가 base authored values만 사용한다.
- Starter 6명이 training sword로 진행 가능하고 reward pool에는 노출되지 않는다.
- Weaponless safe-zone access와 condition-priority `weapon_required` battle rejection이 direct regression으로
  증명된다.
- Active-run old Core version/missing profile이 strict reject되고 current save/restore가 round-trip한다.
- Core/server/live RON/canonical/static/broad gate와 dirty preservation이 통과한다.
- Phase exit audit가 `Complete / High`이고 Master가 Phase 5만 ready로 전환한다.

## DTO migration

기존 typed capability/error projection을 재사용하고 `ActionDisabledReasonCode::WeaponRequired`와 lower
`GameError::WeaponRequired`를 추가했다. Root snapshot과 game_server는 stable `weapon_required` code를 같은
release에 투영한다. Employee combat profile DTO에서는 무기와 중복 authority였던 base
`deployment_affinity`를 제거하고 optional `effective_deployment_affinity`만 유지했다. Unity는 비범위다.

## Data/live RON migration

완료. 모든 weapon RON에 required tactical profile을 명시하고 training sword를 추가했다. Old missing profile,
legacy default/alias와 invalid tile-profile 조합은 수용하지 않는다. General reward pool은 explicit exclusion을
사용한다.

## Save/checkpoint migration

완료. Active-run Core payload를 V3로 strict cutover했다. Header V2는 current body parse 전에 reject하고,
current V3의 missing embedded weapon tactical field도 reject하며 V3 round-trip은 monotonic metadata와 함께
검증된다. Host envelope는 기존 strict V3를 유지한다.

## 필수 focused 검증

- Exact 30 catalog/count/archetype/affinity/base block/profile tuning
- Missing/unknown/mismatched profile strict rejection과 explicit zero acceptance
- Base block-only fragment/power behavior와 Platform effective block 0
- Starter 6 weapon grant, new-run progress와 reward exclusion
- Weaponless safe-zone/equipment allowance, condition-priority reason와 deploy/redeploy/range/manual rejection immutability
- Current active-run round-trip와 old Core version/missing profile rejection
- Checkpoint/root/server typed reason serialization

정확한 test name과 baseline 수는 discovery 뒤 기록한다.

## 필수 broad 검증

- `cargo test -p game_core`
- `cargo test -p game_server`
- `cargo test -p game_core --test ron_loading`
- `cargo fmt --all -- --check`

## 중단 조건

- Exact catalog/tuning에 승인 정책과 다른 current data choice가 필요함
- Active-run version owner를 strict하게 cutover할 수 없고 compatibility 승인이 필요함
- 필수 Core/server/live data/test runner에 접근할 수 없음
- 선행 Phase contract 또는 user dirty와 안전하게 병합할 수 없음

## Compatibility

승인된 compatibility는 없다. Required profile/data/reason/save version을 strict cutover하며 alias, fallback,
dual-read/write/schema와 implicit migration을 추가하지 않는다.

## 완료 감사 Evidence Matrix

| ID | 완료 조건 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P4-01 | Required per-weapon tactical profile와 exact 30 catalog | `DR-10` | required schema/validator와 exact live RON | exact catalog/tuning + invalid/zero tests | pass |
| P4-02 | Affinity/base block/fragment/power single authority | `DR-10` | weapon apply, base-only fragment/power consumers | archetype/block/certification regressions | pass |
| P4-03 | Training sword starter grant/reward exclusion | `DR-10`, `DR-11` | 6 initial loadouts, grant validation, pool exclusion | live RON + new-run grant tests | pass |
| P4-04 | Weaponless safe-zone와 typed gameplay rejection priority | `DR-11` | safe-zone helper, armed battle gate, reason mapping | visibility/rearm + command immutability tests | pass |
| P4-05 | Strict active-run/data/DTO/server cutover | Master Phase 4 gate | Core V3/header reject, DTO/server reason | round-trip/V2/missing-field/server tests | pass |
| P4-06 | Canonical/legacy/compatibility/dirty/broad | Master 공통 gate | gameplay/runtime/data canonical | Core 880/server 55/RON 24/static/fmt/diff | pass |

## 현재 진행 상태

- Predecessor audit와 child path/dirty 확인: 완료
- Production discovery와 변경 전 focused/broad baseline: 완료
- Required schema, exact 30 live data, starter/reward, weaponless gate와 Core V3 구현: 완료
- Focused/broad/static/canonical 검증: 완료
- Completion audit: [Complete / High](COMPLETION_AUDIT.md)
