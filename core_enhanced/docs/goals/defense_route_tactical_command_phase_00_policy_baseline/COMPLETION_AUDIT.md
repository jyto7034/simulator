# DefenseRoute 전술 지휘 Phase 0 Completion Audit

Audit Mode: phase_exit

Audit Target: `defense_route_tactical_command_phase_00_policy_baseline`

Current Status At Audit Start: active

Implementation Complete Claim At Audit Start: false

Policy Source:
[DefenseRoute 전술 지휘 목표 정책 결정 — 2026-08-06](../../decisions/defense_route_tactical_command_policy_2026-08-06.md)

Goal Documents: [PLAN.md](PLAN.md), [EXPERIMENTS.md](EXPERIMENTS.md),
[EXPERIMENT_NOTES.md](EXPERIMENT_NOTES.md)

Repository/Commit/Dirty State: `/mnt/f/work/simulator`, start commit
`f318414fb0156a67e04c5479000c2a2683fc0c09`, initial and final dirty state recorded below

## Completion Evidence Matrix

| ID | 완료 조건 또는 불변식 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P0-01 | 확정 정책 decision record와 전체 coverage | Master Phase 0, `DR-01`~`DR-17` | Decision record의 17 policy heading/17 coverage row | ID별 count 1, 전체 count 17 | pass |
| P0-02 | 임시 조사 문서 동결과 superseding 링크 | Master Phase 0 | 임시 문서 상단과 `문서 지위` | historical/superseded marker 검색 | pass |
| P0-03 | 현재/target/canonical 책임 분리와 Phase/version baseline | Master source-of-truth 규칙 | Decision 책임 절, Phase notes baseline/version matrix | source/version search와 current canonical 대조 | pass |
| P0-04 | `DR-01` 현재 구현 정합성 | Decision `DR-01` | `WinCondition`, `compute_winner`, live rulebook | focused 3 passed, broad Core 포함 pass | pass |
| P0-05 | `DR-15` Pause 명령/시간 정합성 | Decision `DR-15` | playback clock, Core revision transaction, server actor ordering, 3 canonical | baseline 6 + 새 focused 3 passed, broad Core/server pass | pass |
| P0-06 | 범위·legacy·compatibility·dirty 보존 | Master 공통 불변식 | Future production symbol 0건, DTO/data/save version 무변경, scoped diff | placeholder search, `git diff --check`, status/diff audit | pass |
| P0-07 | focused/broad 검증과 완료 감사 | Master Phase gate | 이 audit와 experiments의 exact command/results | Core 847, server 54, RON 23 재실행 모두 pass | pass |

## Policy Restatement

### DR-01

공식 전투는 계속 `DefenseRoute`이며 전술 command는 목표 달성 수단일 뿐 사용 여부가 승패 조건이
아니다. Required enemy group과 protected target을 사용하는 기존 승패를 유지하고 survival encounter는
authored timer 도달 시 남은 적과 무관하게 승리한다.

### DR-15

Pause는 authoritative battle clock만 정지한다. Mutation/query pipeline은 frozen latest state에서 계속
동작하고 mutation은 한 번에 하나씩 독립 commit/reject된다. 성공은 immediate state/event와 revision 1을
commit하고 실패는 무변경이며 뒤 실패가 앞 성공을 rollback하지 않는다. Scheduled simulation과
battle-time cooldown/recovery는 멈추고 catch-up하지 않는다. Paused SetSpeed는 stored speed만 바꾸며
request ordering은 server actor/revision/event seq가 소유한다.

## Runtime And State Evidence

### DR-01 command-to-authority path

- `core_enhanced/src/game/battle/scenario.rs::WinCondition`이 required groups, protected unit과 timer 조건을
  typed state로 소유한다.
- `core_enhanced/src/game/battle/core/sim.rs::compute_winner`는 active combatant lifecycle, authored unit ref,
  required group spawn/death와 current battle time만 사용한다.
- 배치·철수·Pause·skill 사용 여부는 winner 입력이 아니다.
- `ProtectUnitUntil`은 objective death를 먼저 패배로 처리하고 `current_time_ms >= time_ms`면 남은 적과
  무관하게 Player 승리를 반환한다.

### DR-15 command-to-authority path

- `ActiveBattleSession::simulation_delta_for_tick`은 paused면 `None`을 반환해 production server tick이
  BattleCore를 진행시키지 않는다.
- `handle_pause_battle`, `handle_resume_battle`, `handle_set_battle_speed`와 deploy/withdraw/skill handler는
  commit 가능 여부를 먼저 확인하고 성공 state 뒤 `commit_battle_projection`으로 revision을 한 번
  증가시킨다.
- `battle_capability`는 paused를 deploy/withdraw/skill의 공통 금지 조건으로 사용하지 않는다.
- `PlayerGameActor` mailbox는 request를 직렬 처리하고 supplied battle authority를 Core가 mutation 전에
  검증한다.
- 같은 revision 두 request test는 첫 Pause가 revision을 증가시킨 뒤 둘째 Resume를 stale로 거부하고
  error→현재 paused resync를 순서대로 전달함을 증명한다.

새 production runtime branch는 추가하지 않았다. 직접 behavior coverage와 canonical만 보강했다.

## Data Evidence

- Phase 0은 production RON을 수정하지 않았다.
- `game_resources/data/run/policy.ron`의 current `live_deployment` initial 20/max 99/1초당 1/base deploy 10과
  redeploy 30/90초·150%는 future Phase baseline으로만 기록했다.
- `cargo test -p game_core --test ron_loading` 23개가 별도 재실행으로 모두 통과했다.
- Live RON manifest schema version은 1을 유지한다.

## DTO/Transport/Persistence Evidence

- Phase 0 DTO migration: 없음.
- Data/live RON migration: 없음.
- Save/checkpoint migration: 없음.
- Battle setup version 1, event log version 27, Core active-run payload version 2와 server envelope version 3을
  변경하지 않았다.
- Core checkpoint는 paused/stored speed/battle time/revision/event cursor를 계속 생산하고 server는 같은
  typed update/resync를 재추론 없이 전달한다.
- Server protocol canonical은 actor serialization, same-revision stale와 Pause tick 의미를 current runtime에
  맞게 명시한다.

## Consumer Evidence

- Confirmed workspace consumer: `game_server`의 `PlayerGameActor`, command result/update/resync mapping과
  active-run persistence.
- Canonical external consumer: Unity가 존재하지만 사용자 지시로 이번 Master 전체에서 제외됐다.
- Inaccessible/unknown consumer: 별도로 발견하지 못했다.
- Explicitly excluded consumer: Unity runtime, contract, UI, store와 tests.

Phase 0 완료는 Unity consumer 구현·검증 완료를 뜻하지 않는다.

## Legacy/Compatibility Audit

- `EngagementStance`, `TacticalRelocateUnit`, `SwapUnitPositions`, `SkillCastSelection`,
  `FieldWorkObjectDef`, `FieldWorking`, `tactical_resource`를 production Rust/RON에서 검색해 0건을 확인했다.
  Future placeholder가 Phase 0에 선행되지 않았다.
- Pause 전용 runtime, command batch, `ResumeBattleWithPlan`, alias, dual-read/write와 fallback을 추가하지
  않았다.
- 기존 setup/event/save/live-RON version과 shape를 그대로 사용했다.
- 변경한 test에 ignored/disabled marker와 production special case가 없다.

## Scoped Diff And Dirty Preservation

Phase 0이 직접 변경한 production/canonical 경로:

- `core_enhanced/docs/game_rulebook.md`
- `core_enhanced/docs/core_runtime_contract.md`
- `core_enhanced/src/game/world/tests/combat.rs`
- `game_server/docs/unity_bridge_protocol.md`
- `game_server/src/game/player_game_actor/handlers.rs`

Phase 0 policy/evidence 경로:

- 새 target decision record
- 동결한 DefenseRoute 임시 조사 문서
- Master와 Phase 0 Goal 문서

초기 Phase dirty였던 `README.md`, `engineering_workflow.md`, `goal_completion_review_guide.md`,
`skills/abnormality_skill_design_notes.ko.md`, `goal_authoring_and_execution_guide.md`는 Phase 0 구현 범위에서
수정·삭제·이동하지 않았다. 관련 없는 untracked file 삭제와 repository-wide formatting은 없었다.
`cargo fmt --all`의 actual Rust diff는 변경한 두 test 파일에 한정됐다. `git diff --check`가 통과했다.

## Canonical Alignment

- `game_rulebook.md`: current `DefenseRoute` 승패와 Pause 중 현재 command의 player-facing 의미
- `core_runtime_contract.md`: pause-aware production tick, exact authority/revision, independent transaction,
  event sequence와 reconnect 불변식
- `game_server/docs/unity_bridge_protocol.md`: actor serialization, same-revision stale error→resync와 paused
  command delivery
- Target decision record: 아직 미구현인 DR-02~DR-14/16/17을 current contract와 분리

Runtime/code 문장과 비교해 material drift를 발견하지 않았다.

## Validation Results

### Baseline

- `cargo test -p game_core protect_unit_until`: 2 passed, 0 failed, 0 ignored
- `cargo test -p game_core battle_setup_snapshot_exposes_survival_timer_without_encirclement_variant`: 1 passed,
  0 failed, 0 ignored
- Core DR-15 baseline 네 command: 각 1 passed, 합계 4 passed, 0 failed, 0 ignored
- Server DR-15 baseline 두 command: 각 1 passed, 합계 2 passed, 0 failed, 0 ignored

Baseline 합계: 9 passed, 0 failed, 0 ignored.

### Focused after correction

- `cargo test -p game_core paused_live_battle_commits_deploy_and_withdraw_without_advancing_time`: 1 passed
- `cargo test -p game_core activate_skill_uses_equipped_manual_fragment_in_live_defense`: 1 passed
- `cargo test -p game_server second_mutation_using_same_revision_is_stale_after_first_commit`: 1 passed

Focused correction 합계: 3 passed, 0 failed, 0 ignored.

### Broad

- `cargo fmt --all -- --check`: pass; test runner 아님
- `cargo test -p game_core`: 847 passed, 0 failed, 0 ignored
- `cargo test -p game_server`: 54 passed, 0 failed, 0 ignored
- `cargo test -p game_core --test ron_loading`: 23 passed, 0 failed, 0 ignored; broad Core에 포함된 suite 재실행
- Policy/coverage/placeholder/diff static gate: pass; test runner 아님

## Baseline Failures

없음. 변경 전 focused 9개가 모두 통과했고 변경 후 새 실패도 없다.

## Cross-Component Findings

- Material finding 없음.
- Core paused mutation revision과 server same-revision stale/resync가 같은 authority를 사용한다.
- Future policy는 decision record에만 있으며 current Core/data/server shape와 경쟁하는 source of truth를
  만들지 않았다.
- Unity는 명시적 비범위이므로 검증한 것으로 표현하지 않는다.

## Long-Term Direction Review

- Fit: High
- Improvement class: none
- Reason: 목표 정책, current canonical과 historical evidence 책임을 분리했고 기존 playback/revision/actor
  seam을 직접 테스트했다. 새 runtime state, compatibility, duplicate authority와 future placeholder가
  없다.

## Primary Verdict

Complete

모든 Phase 0 완료 조건과 Master 추가 gate가 실제 evidence에 연결됐고 material gap이 없다.

## Status Transition

- from: active / `Implementation Complete: false`
- to: complete / `Implementation Complete: true`
- next Phase activation: allowed; Master가 Phase 0을 complete로 기록한 뒤 Phase 1만 ready로 전환 가능

## Remaining Risk And Required Action

- Unity consumer는 승인 비범위이며 Master 완료 시에도 별도 위험으로 남는다.
- Future command Phase는 각각 `DR-15` Pause consumer regression을 추가해야 한다.
- Phase 3/4와 6/9의 실제 version bump 결합 여부는 해당 배포 단위에서 다시 판정한다.
- 위 항목은 Phase 0의 material 미완료가 아니며 다음 Phase에서 선행 구현하지 않는다.
