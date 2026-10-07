# Core Enhanced Refactoring Audit Phase 03 — Experiments

Status: complete

## 2026-08-07 — Phase initialization

### 검증하려던 가설

Phase 2가 `Complete / High`이고 Phase 3 예약 경로 충돌/다른 active child 없이 승인된 sequential
roster-inventory-economy investigation 문서를 초기화할 수 있다.

### 수행한 실험 또는 검증

- Parent Master와 Phase 2 completion audit 확인
- Phase 0 boundary proposal의 Phase 3 범위 확인
- Phase 3 경로 존재 여부와 active audit child 확인
- Phase 2 production-content/status hash 보존 결과 확인

Production roster/inventory/economy search, test runner와 formatter는 Phase 3 필수 문서 생성 전에 수행하지
않았다.

### 변경한 내용

- 이 Phase의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md` 생성
- Parent Master registry에서 Phase 2를 complete, Phase 3을 active로 전환

Production code, live data, test, canonical과 server code는 수정하지 않았다.

### 실행한 정확한 명령

```bash
if test -e core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy; then echo EXISTS; else echo MISSING; fi
find core_enhanced/docs/goals -maxdepth 2 -name PLAN.md -path '*core_enhanced_refactoring_audit_phase_*' -print | sort
rg -n '^Status: active' core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_*/*
```

### 실제 실행된 test 수

Phase initialization test runner는 실행하지 않았다. 해당 없음.

### Pass / Fail / Skip

Test 기준 해당 없음. Dependency/path/active-child gate는 통과했다.

### 실제 결과

- Phase 2 completion matrix 10/10 pass, verdict `Complete / High`.
- Phase 3 path는 생성 전에 존재하지 않았다.
- Phase 2 종료 뒤 active child는 없었다.
- Phase 2 initial/final production hashes 7개가 동일했다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음. 기존 dirty/untracked production 파일을 수정, 이동, 삭제, format, restore하지 않았다.

### 결과에 따른 판단

Phase 3을 `investigation_only`, production mutation `no`로 활성화할 수 있다. 실제 조사 전 필수 문서와
direct dependency를 다시 읽는다.

## 아직 수행하지 않은 실험과 검증

- exact roster/inventory/economy surface inventory
- authoritative ledger/identity/transaction trace
- focused test and live-data/canonical audit
- candidate/disposition and completion audit

위 항목은 `discovery pending`이다.

## 2026-08-07 — Roster/inventory/economy investigation

### 검증하려던 가설

현재 persistent asset authority가 root roster/inventory/fragment/currency ledger에 있고 reward/shop/event/combat
등의 grant caller는 이를 staged commit하는지, 그리고 정상 command가 유지하지만 V3 restore 또는 허용된
recipe shape가 깨뜨릴 수 있는 ownership/atomicity invariant가 있는지 검증했다.

### 수행한 실험 또는 검증

- 초기 dirty 목록과 다음 production content/status hash를 고정했다.
  - Core employee/resource domain: `d89334ff357733097d2969039075eeb3af477db7d06dfc9d9f4b5d1de47f7a49`
  - Core world economy seams: `8a19fe8687c2790b8547d031f9a6c0965e1a5fbf31c4ca2b3e2b3dc860f6222b`
  - typed data/validation: `003119cf844cbf1c0b812ff945b0521cd395bd96df5fbad7f2325678dcfa40dd`
  - focused source/integration tests: `521fbd4dca7969311563281b07a31146802e8bb8645a2c62f0bd01124bef8270`
  - relevant live RON: `4a951620d6238ea3626da1c1af53da0f363ceb35da1f8fae0f9bdd92a405ef57`
  - routed canonical: `87540e3a2a2be57a5382c303d064ca11e0757807ffcc320dbb4e1fa13cc0fc04`
  - direct Phase production status: `ad4ec371d2343ac34e605d62e9ab5b933167db661464004766a0348485c379f2`
- employee definition ID, run-local employee UUID, owned equipment instance UUID, equipment/consumable definition
  identity와 roster/inventory/loadout의 양방향 binding을 추적했다.
- starter/equip/move/replace/unequip/consumable/death/dismantle가 두 ledger를 어떻게 stage/commit하는지
  추적했다.
- skill fragment ownership/progress/pending/dust, active loadout와 abnormality research planning-return delivery를
  추적했다.
- `GrantExecutor`의 다섯 ledger와 reward/shop/event/combat/admin caller의 clone/commit을 모두 검색했다.
- shop buy/sell/refresh, headquarters recruitment/emergency rollback, maintenance dismantle/enhancement의
  validation→cost→grant→commit 순서를 추적했다.
- V3 checkpoint validator/restore와 battle setup/snapshot/fragment-profile consumer를 대조했다.
- live data 수와 recipe/policy 값을 확인하고 nearest canonical의 atomicity/identity 규칙과 대조했다.
- legacy/default/fallback/ignored test와 workspace direct public-API consumer를 검색했다.

### 변경한 내용

- Phase Goal 경로에 `BOUNDARY_AUDIT.md`, `CANDIDATE_CATALOG.md`와 이 조사 기록만 추가했다.
- production source/data/test/canonical/server/Unity 파일은 수정하지 않았다.

### 실행한 정확한 production baseline 명령

아래 명령은 repository root `/mnt/f/work/simulator`에서 실행했다. 마지막 status 명령은 동일한 direct
production path set의 initial/final 상태를 비교하는 데 사용했다.

```bash
sha256sum core_enhanced/src/game/{employee.rs,skill_fragment.rs,abnormality_research.rs,reward.rs,reward_policy.rs,recruitment.rs} core_enhanced/src/game/resources/{inventory.rs,item_slot.rs,economy.rs,state.rs,selection.rs} | sha256sum
sha256sum core_enhanced/src/game/world.rs core_enhanced/src/game/world/{helpers.rs,reward.rs,shop.rs,headquarters.rs,maintenance.rs,state.rs,node_flow.rs,combat.rs,event_node.rs,facility.rs,active_run.rs,admin/grants.rs} | sha256sum
sha256sum core_enhanced/src/game/data/{employee_data.rs,equipment_data.rs,consumable_data.rs,reward_data.rs,shop_data.rs,skill_fragment_data.rs,run_policy_data.rs} core_enhanced/src/game/data/validation.rs | sha256sum
sha256sum core_enhanced/src/game/world/tests/{equipment.rs,node_sessions.rs,facility.rs,map_flow.rs,snapshots_and_start.rs,combat.rs} core_enhanced/tests/{ron_loading.rs,live_item_skill_activation.rs} | sha256sum
sha256sum game_resources/data/{employees/definitions.ron,equipments/base.ron,consumables/base.ron,events/rewards/base.ron,events/shops/base.ron,skill_fragments/base.ron,run/policy.ron,abnormalities/base.ron,pve/encounters.ron} | sha256sum
sha256sum core_enhanced/docs/{README.md,core_runtime_contract.md,game_rulebook.md,data_loading_contract.md} | sha256sum
git status --short -- core_enhanced/src/game/{employee.rs,skill_fragment.rs,abnormality_research.rs,reward.rs,reward_policy.rs,recruitment.rs,resources,world.rs,world,data} core_enhanced/tests game_resources/data core_enhanced/docs/{README.md,core_runtime_contract.md,data_loading_contract.md,game_rulebook.md} | sha256sum
```

### 실행한 대표 source/data/canonical 명령

```bash
rg -n "struct GameCoreState|pub struct Inventory|pub struct OwnedEquipment|pub struct EmployeeRoster|pub struct EmployeeLoadout|pub struct SkillFragmentLoadout|pub struct RunCheckpointPayload|fn validate_run_checkpoint_payload|fn restore_run_checkpoint_payload|pub struct GrantExecutor|fn with_headquarters_transaction|fn with_state_transaction|validate_enhancement_recipe_references|validate_enhance_equipment_payload|handle_enhance_equipment|apply_to_profile" core_enhanced/src/game
rg -n "GrantExecutor::grant_effects_with_state|GrantExecutor::grant_reward_with_state" core_enhanced/src | sort
sed -n '520,610p' core_enhanced/src/game/data/equipment_data.rs
sed -n '550,610p' core_enhanced/src/game/world/helpers.rs
sed -n '450,535p' core_enhanced/src/game/world/maintenance.rs
sed -n '1270,1455p' core_enhanced/src/game/world/node_flow.rs
rg -n "employee_definition_id:" game_resources/data/employees/definitions.ron
rg -n "EquipmentMetadata\(" game_resources/data/equipments/base.ron
rg -n "ConsumableMetadata\(" game_resources/data/consumables/base.ron
rg -n "RewardMetadata\(" game_resources/data/events/rewards/base.ron
rg -n "SkillFragmentMetadata\(" game_resources/data/skill_fragments/base.ron
rg -n "enhancement_recipes|costs_per_level|equipment_dust|starter_enkephalin|emergency_enkephalin" game_resources/data
rg -n "atomic|checkpoint|GrantConsumable|fragment dust|equipment material|pending research|enhancement" core_enhanced/docs/{README.md,game_rulebook.md,core_runtime_contract.md,data_loading_contract.md}
rg -n "#\[ignore" core_enhanced/src/game/{employee.rs,skill_fragment.rs,abnormality_research.rs} core_enhanced/src/game/resources/{inventory.rs,item_slot.rs} core_enhanced/src/game/world/tests/{equipment.rs,node_sessions.rs,facility.rs,map_flow.rs,snapshots_and_start.rs,combat.rs} core_enhanced/tests/{ron_loading.rs,live_item_skill_activation.rs}
```

### 실행한 test와 retained evidence의 정확한 결과

| Command/filter | Passed | Failed | Ignored | Filtered |
| --- | ---: | ---: | ---: | ---: |
| `cargo test -p game_core --lib game::resources::inventory::tests -- --nocapture` | 5 | 0 | 0 | 857 |
| `cargo test -p game_core --lib game::resources::item_slot::tests -- --nocapture` | 4 | 0 | 0 | 858 |
| `cargo test -p game_core --lib game::employee::tests -- --nocapture` | 16 | 0 | 0 | 846 |
| `cargo test -p game_core --lib game::skill_fragment::tests -- --nocapture` | 13 | 0 | 0 | 849 |
| `cargo test -p game_core --lib game::abnormality_research::tests -- --nocapture` | 2 | 0 | 0 | 860 |
| `cargo test -p game_core --lib game::world::tests::equipment -- --nocapture` | 28 | 0 | 0 | 834 |
| `cargo test -p game_core --lib game::recruitment::tests -- --nocapture` | 3 | 0 | 0 | 859 |
| `cargo test -p game_core --lib game::reward::tests -- --nocapture` | 2 | 0 | 0 | 860 |
| `cargo test -p game_core --lib game::world::shop::tests -- --nocapture` | 6 | 0 | 0 | 856 |
| `cargo test -p game_core --lib game::world::tests::node_sessions -- --nocapture` | 18 | 0 | 0 | 844 |
| `cargo test -p game_core --lib game::world::tests::facility -- --nocapture` | 11 | 0 | 0 | 851 |
| `cargo test -p game_core --lib game::world::tests::snapshots_and_start -- --nocapture` | 23 | 0 | 0 | 839 |
| `cargo test -p game_core --test ron_loading -- --nocapture` | 24 | 0 | 0 | 0 |
| `cargo test -p game_core --test live_item_skill_activation -- --nocapture` | 3 | 0 | 0 | 0 |
| `cargo test -p game_core --lib game::world::tests::map_flow -- --nocapture` | 40 | 0 | 0 | 822 |
| `cargo test -p game_core --lib game::world::tests::combat::combat_result_completion_failure_does_not_partially_apply_rewards_on_retry -- --exact --nocapture` | 1 | 0 | 0 | 861 |
| `cargo test -p game_core --lib game::world::tests::combat::endless_combat_victory_adds_satisfied_bonus_objective_research -- --exact --nocapture` | 1 | 0 | 0 | 861 |
| `cargo test -p game_core --lib game::world::tests::combat::active_run_transaction_rollback_preserves_profile_and_peak_but_restores_floor_anchor -- --exact --nocapture` | 1 | 0 | 0 | 861 |

Retained evidence는 18 runner invocation, 201 pass, 0 fail, 0 ignored다. Filtered는 이름이 맞지 않아 실행하지
않은 test이며 skip/pass로 세지 않았다.

앞서 recruitment/reward/shop/node-session/facility/snapshot/RON/item-skill 8개 filter를 병렬 실행한 첫 batch의
aggregate output이 context 압축에서 잘려 per-filter 결과를 보존하지 못했다. 같은 8개를 즉시 재실행해 위
90/90 결과를 확보했다. 따라서 실제 physical execution은 26 invocation/291 tests이고, pass 근거에는
보존된 201/201만 사용한다.

### 실제 결과

- 정상 equip/consumable/reward/shop/research/checkpoint 흐름은 선택된 safety net을 통과했다.
- V3 import가 roster/inventory/loadout/fragment 내부 graph를 검증하지 않는 문제를 C01 correction으로
  분류했다.
- schema가 duplicate enhancement material row를 허용하고 handler가 live balance를 부분 소비하는 문제를
  C02 correction으로 분류했다.
- 양방향 equipment/loadout transition은 C03, 다섯 grant ledger staging은 C04, headquarters 수동 rollback
  field list는 C05 behavior-preserving refactor로 분류했다.
- live catalog count는 employee 12, equipment 88, consumable 8, reward 13, fragment 31, shop context 3,
  material/dismantle/enhancement recipe 각 1이었다.
- current enhancement recipe는 `standard_armor`, max level 5, `equipment_dust` 2 한 줄이라 C02를 현재 live
  content가 직접 발동하지는 않는다.
- legacy reward/shop/consumable vocabulary는 explicit rejection이며 fallback debt가 아니었다.

### 실패 원인

Test failure는 없다. 병렬 runner는 Cargo package/build lock을 기다린 뒤 모두 종료했다. 첫 8-filter
aggregate output은 test 실패가 아니라 evidence capture truncation이었고, 재실행으로 결과를 보존했다.

한 source-search 명령은 잘못된 `crates/game_core/...` 경로를 사용해 두 파일을 찾지 못했다. 즉시 실제
`src/game/world/tests/{combat,map_flow}.rs` 경로를 `rg --files`로 확인하고 같은 검색을 재실행했으며,
production/test 결과에는 영향이 없다.

### 되돌리거나 수정한 내용

없음. production code/data/test/canonical/server 파일을 restore/format/move/delete/edit하지 않았다.

### 결과에 따른 판단

C01/C02는 test-first correctness correction, C03~C05는 characterization 뒤 behavior-preserving refactor다.
Checkpoint metadata rebind, recipe/content rebalance, public API removal, Event/VN/combat/DTO 내부 변경은
deferred/unverified로 남겨 이 Phase가 정책을 선택하지 않는다.

## 아직 수행하지 않은 실험과 검증 — investigation 후

- C01 contradictory-current-V3 negative matrix와 valid full-economy round trip
- C02 duplicate-material aggregate insufficient/success test
- C03 full equip transition parity characterization
- C04 caller별 seed/UUID/diff/late-failure parity characterization
- C05 headquarters full-state/compendium rollback parity characterization
- Unity 및 other external Rust public API consumer inventory

위 항목은 production mutation이 금지된 read-only Phase의 누락이 아니라 candidate implementation 또는
deferred compatibility work다.

## 2026-08-07 — Final production-preservation verification

초기와 동일한 파일 순서/집합과 direct scoped-status 명령으로 다시 계산한 7개 hash는 모두 동일했다.

- Core employee/resource domain: `d89334ff357733097d2969039075eeb3af477db7d06dfc9d9f4b5d1de47f7a49`
- Core world economy seams: `8a19fe8687c2790b8547d031f9a6c0965e1a5fbf31c4ca2b3e2b3dc860f6222b`
- typed data/validation: `003119cf844cbf1c0b812ff945b0521cd395bd96df5fbad7f2325678dcfa40dd`
- focused source/integration tests: `521fbd4dca7969311563281b07a31146802e8bb8645a2c62f0bd01124bef8270`
- relevant live RON: `4a951620d6238ea3626da1c1af53da0f363ceb35da1f8fae0f9bdd92a405ef57`
- routed canonical: `87540e3a2a2be57a5382c303d064ca11e0757807ffcc320dbb4e1fa13cc0fc04`
- direct Phase production status: `ad4ec371d2343ac34e605d62e9ab5b933167db661464004766a0348485c379f2`

Phase completion matrix와 verdict는 별도 `COMPLETION_AUDIT.md`에서 재검토한다.
