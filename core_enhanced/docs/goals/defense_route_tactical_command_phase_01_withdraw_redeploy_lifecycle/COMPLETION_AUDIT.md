# DefenseRoute 전술 지휘 Phase 1 Completion Audit

Audit Mode: phase_exit

Audit Target: `defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle`

Current Status At Audit Start: active

Implementation Complete Claim At Audit Start: false

Policy Source:
[DefenseRoute 전술 지휘 목표 정책 결정 — DR-03](../../decisions/defense_route_tactical_command_policy_2026-08-06.md#dr-03--기존-회수와-재배치)

Goal Documents: [PLAN.md](PLAN.md), [EXPERIMENTS.md](EXPERIMENTS.md),
[EXPERIMENT_NOTES.md](EXPERIMENT_NOTES.md)

Repository/Commit/Dirty State: `/mnt/f/work/simulator`, start commit
`f318414fb0156a67e04c5479000c2a2683fc0c09`, initial and final dirty state recorded in the Goal evidence

## Completion Evidence Matrix

| ID | 완료 조건 또는 불변식 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P1-01 | Withdraw eligibility와 실패 무변경 | `DR-03` | `BattleCore::withdraw_unit`, world transaction | status/invalid/revision regression | pass |
| P1-02 | Unreleased basic/skill cancellation | `DR-03` | basic resolve lifecycle, active-cast `SkillStep` gate | instant attack, pending/active/queued skill regression | pass |
| P1-03 | Released autonomous delivery 유지/source-bound 종료 | `DR-03` | projectile snapshots, private area source context, tracking cleanup | basic/skill projectile와 세 area tracking regression | pass |
| P1-04 | HP/condition/identity/transient lifecycle | `DR-03` | HP policy, attempt ledger, redeploy draft | voluntary/defeated redeploy regression | pass |
| P1-05 | 공통 ceiling 비용과 Pause/reconnect | `DR-03` | `LiveBattleDeploymentState::redeploy_cost`, lock/checkpoint | odd/overflow/reduction/Pause/resync regression | pass |
| P1-06 | DTO/data/save/server/canonical 일치 | Master Phase 1 gate | shape/version/RON 무변경, current canonical 두 곳 | server 54, RON 23, canonical audit | pass |
| P1-07 | Legacy/compatibility/dirty 보존과 broad 검증 | Master 공통 gate | old floor/delete pattern 0, scoped diff | Core 855/server 54/fmt/diff audit | pass |

## Policy Restatement

자발적 철수는 Active Player runtime instance에만 성공하며 비용을 소비하거나 이미 지불한 비용을
환불하지 않는다. Release 전 action과 caster-owned transient state는 종료한다. 이미 독립한 projectile과
GroundFixed delivery는 생성 시점 source context로 계속 실행하고, FollowCaster/FollowTarget처럼 source에
묶인 runtime은 종료한다.

자발적 철수는 현재 HP의 30%를 내림한 값으로 새 deployment lock을 만들고 30초 뒤 같은 employee를 새
runtime instance로 재배치할 수 있다. 전투불능은 최신 condition과 instance별 penalty를 반영하고 90초
lock을 만든다. 두 경로 모두 비용 `ceil(base * 150 / 100)`을 사용하며 Pause 중 battle time은 진행하지
않는다.

## Runtime And State Evidence

- `BattleCore::withdraw_unit`은 Active lifecycle 확인, target/engagement/buff와 pending/active cast cleanup을
  한 transaction에서 수행한다. Dead와 이미 Withdrawn request는 state/event/revision을 바꾸지 않는다.
- Basic resolve는 source가 Withdrawn이면 release 전 event를 no-op하지만 Dead source의 이미 committed된
  instant behavior는 기존 의미를 유지한다.
- Active cast가 사라진 뒤 도착한 `BattleEvent::SkillStep`은 no-op한다. Withdraw cleanup은 released skill
  projectile을 삭제하지 않는다.
- Persistent area 생성은 private `AreaRuntime`에 source snapshot/facing을 저장한다. GroundFixed는 source
  철수 뒤에도 이 context와 area/time-specific roll identity를 사용한다. FollowCaster/FollowTarget는
  withdraw에서 제거된다.
- `LiveBattleDeploymentState::redeploy_cost()`가 자발적 철수와 defeated reconciliation 모두의
  overflow-safe ceiling 비용을 소유한다.
- Redeploy draft는 같은 employee identity의 새 unit ID로 initial resonance/cooldown/target/cast state를
  재구성하고 persistent condition/loadout만 반영한다.

## Data Evidence

- Production RON migration은 없다.
- `game_resources/data/run/policy.ron`의 redeploy 30초/90초/150%와 기존 deployment tuning을 유지했다.
- DeployCostReduction은 기존 effective base cost 계산 뒤 공통 ceiling redeploy helper에 들어간다.
- `cargo test -p game_core --test ron_loading` 23개가 별도 재실행으로 모두 통과했다.

## DTO/Transport/Persistence Evidence

- DTO migration: 없음.
- Save/checkpoint migration: 없음.
- Battle setup, event log, Core active-run payload와 server envelope version 변경: 없음.
- Source snapshot/facing 추가는 serialized DTO가 아닌 private in-memory `AreaRuntime`에 한정된다.
- Existing checkpoint의 redeploy lock/candidate와 Core battle update를 game_server가 기존 generic mapping으로
  전달한다.

## Consumer Evidence

- Confirmed workspace consumer: `game_server`; 전체 54 tests가 통과했다.
- Explicitly excluded consumer: Unity runtime, contract, UI와 tests.
- Inaccessible/unknown consumer: 별도로 발견하지 못했다.

Phase 1 완료는 Unity consumer 구현·검증 완료를 뜻하지 않는다.

## Legacy/Compatibility Audit

- 이전 중복 floor 비용식과 cast-wide projectile/area deletion pattern을 production source에서 검색해 0건을
  확인했다.
- 새 fallback, legacy path, compatibility alias, dual lifecycle, TODO/FIXME/placeholder를 production diff에
  추가하지 않았다.
- Production RON, DTO, checkpoint/save와 version shape를 그대로 사용했다.
- Future Phase 2~9 production symbol이나 authoring option을 선행 추가하지 않았다.

## Scoped Diff And Dirty Preservation

Phase 1이 직접 변경한 production/canonical 경로:

- `core_enhanced/src/game/battle/core/basic_attack.rs`
- `core_enhanced/src/game/battle/core/build.rs`
- `core_enhanced/src/game/battle/core/mod.rs`
- `core_enhanced/src/game/battle/core/sim.rs`
- `core_enhanced/src/game/battle/core/skill_runtime/area.rs`
- `core_enhanced/src/game/battle/core/types.rs`
- `core_enhanced/src/game/world/combat.rs`
- `core_enhanced/src/game/world/state.rs`
- `core_enhanced/src/game/world/tests/combat.rs`
- `core_enhanced/docs/game_rulebook.md`
- `core_enhanced/docs/core_runtime_contract.md`

Phase 0에서 이미 변경된 canonical/world test와 server files는 predecessor output으로 보존했다. 시작 시
존재하던 README/workflow/audit guide/skill notes, Goal authoring guide와 `docs/tmp`를 삭제·이동·되돌리지
않았다. Unity 경로는 Phase 1에서 수정하지 않았다. `git diff --check`가 통과했다.

## Canonical Alignment

- `game_rulebook.md`: 철수 eligibility, release 경계, delivery tracking, HP/condition/비용/시간의 player-facing
  current rule
- `core_runtime_contract.md`: source exit와 detached delivery, stale event, private runtime context, new-instance
  redeploy의 authoritative contract
- Target decision record: `DR-03`의 확정 정책

Runtime/code 문장과 대조해 material drift를 발견하지 않았다.

## Validation Results

### Baseline

- `cargo test -p game_core withdraw -- --nocapture`: 9 passed, 0 failed, 0 ignored

### Focused after correction

- `cargo test -p game_core withdraw -- --nocapture`: 16 passed
- `redeploy_cost_uses_ceiling_for_odd_base_and_saturates`: 1 passed
- `deploy_cost_reduction_consumable_reduces_live_deployment_cost_for_employee`: 1 passed
- `live_deployment_reconciles_defeated_player_unit_during_authoritative_advance`: 1 passed

중복을 제외한 최종 focused 합계: 19 passed, 0 failed, 0 ignored.

### Broad

- `cargo fmt --all -- --check`: pass; test runner 아님
- `cargo test -p game_core`: 855 passed, 0 failed, 0 ignored
- `cargo test -p game_server`: 54 passed, 0 failed, 0 ignored
- `cargo test -p game_core --test ron_loading`: 23 passed, 0 failed, 0 ignored; broad Core에 포함된 suite 재실행
- Old-pattern, compatibility, placeholder와 `git diff --check` static gate: pass

## Baseline And Correction Failures

Baseline failure는 없다. 첫 새 projectile regression 실행에서 두 fixture가
`speed_units_per_ms = 1_000`을 사용해 1ms에 1 world tile을 이동하지 못하여 2 tests가 실패했다. Runtime
결함이 아니라 `1 world = 1_000_000 data units` scale의 fixture 오류였고 speed를 `1_000_000`으로 수정한
뒤 같은 focused suite가 통과했다.

## Cross-Component Findings

- Material unresolved finding 없음.
- Core의 redeploy lock/candidate projection과 server mapping은 shape 변경 없이 일치한다.
- Private area context 덕분에 released delivery 정책을 save/wire duplicate authority 없이 구현했다.
- Unity는 명시적 비범위이므로 검증한 것으로 표현하지 않는다.

## Long-Term Direction Review

- Fit: High
- Improvement class: none
- Reason: action release와 delivery ownership을 기존 Core lifecycle seam에 명시했고, 비용 중복을 공통
  authority로 제거했다. 새 compatibility, duplicate authority, serialized migration과 future placeholder가
  없다.

## Primary Verdict

Complete

모든 Phase 1 완료 조건과 Master 추가 gate가 실제 evidence에 연결됐고 material gap이 없다.

## Status Transition

- from: active / `Implementation Complete: false`
- to: complete / `Implementation Complete: true`
- next Phase activation: allowed; Master가 Phase 1을 complete로 기록한 뒤 Phase 2 child Goal을 초기화할 수 있음

## Remaining Risk And Required Action

- 이후 stance/FieldWork가 새 source-bound runtime을 만들면 같은 delivery ownership matrix를 해당 Phase에서
  다시 감사해야 한다.
- Future serialized delivery가 필요하면 source snapshot migration을 그 Phase에서 명시적으로 결정해야 한다.
- Unity consumer는 승인 비범위이며 Master 완료 시에도 별도 위험으로 남는다.
- 위 항목은 Phase 1의 material 미완료가 아니며 Phase 2에서 선행 구현하지 않는다.
