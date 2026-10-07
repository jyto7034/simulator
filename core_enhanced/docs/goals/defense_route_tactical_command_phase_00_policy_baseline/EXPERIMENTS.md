# DefenseRoute 전술 지휘 Phase 0 — Experiments

Status: complete

이 문서는 Phase 0에서 실제로 수행한 조사, 변경과 검증만 시간순으로 기록한다.

## 2026-08-06 — Phase Goal 초기화 전 경로·dirty 상태 확인

### 검증하려던 가설

`defense_route_tactical_command_phase_00_policy_baseline` 경로가 기존 Goal과 충돌하지 않으며,
Master의 Phase 0만 활성화한 상태에서 필수 문서 3개를 안전하게 초기화할 수 있다.

### 수행한 확인

- Phase Goal 디렉터리와 필수 문서 3개의 존재 여부 확인
- repository root, 시작 commit과 현재 dirty 상태 확인
- Master Phase registry에서 Phase 0이 `ready`이고 다른 활성 Phase가 없음을 확인

### 변경한 내용

- Phase Goal 디렉터리와 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 생성했다.
- Master에서 Phase 0을 `active`로 전환하고 child Goal 링크를 기록했다.
- Production code, live data, game_server, tests와 gameplay canonical은 아직 변경하지 않았다.

### 실행한 정확한 명령

```bash
for p in core_enhanced/docs/goals/defense_route_tactical_command_phase_00_policy_baseline core_enhanced/docs/goals/defense_route_tactical_command_phase_00_policy_baseline/PLAN.md core_enhanced/docs/goals/defense_route_tactical_command_phase_00_policy_baseline/EXPERIMENTS.md core_enhanced/docs/goals/defense_route_tactical_command_phase_00_policy_baseline/EXPERIMENT_NOTES.md; do if [ -e "$p" ]; then echo "EXISTS $p"; else echo "MISSING $p"; fi; done
git rev-parse HEAD
git status --short
```

### 실제 test 수

경로·상태 확인이며 test runner는 실행하지 않았다. 해당 없음.

### Pass / Fail / Skip

해당 없음.

### 실제 결과

- Phase Goal 디렉터리와 필수 문서 3개는 모두 존재하지 않았다.
- 시작 commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Master Phase 0은 `ready`, Phase 1~9는 `pending`이었고 다른 활성 Phase는 없었다.
- 현재 dirty 항목은 `EXPERIMENT_NOTES.md`의 초기 dirty worktree 절에 기록했다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

Phase 0을 활성화하고 필수 문서를 먼저 초기화한다. 이후에만 정책 문서와 관련 current runtime을
조사한다.

## 2026-08-06 — 확정 정책 추출과 current baseline 조사

### 검증하려던 가설

임시 문서의 번호별 확정 의미와 tuning을 역사적 조사에서 분리할 수 있고, 현재 tree에서 각 Phase의
구현 전 seam/gap과 version owner를 다시 식별할 수 있다.

### 수행한 확인

- 4,705행 임시 문서의 `DR-01`~`DR-17` 확정 정책과 최종 tuning 절을 모두 대조했다.
- 현재 `WinCondition`/winner, playback, revision, command capability와 server actor ordering 경로를 읽었다.
- Future Phase의 현재 state/data/DTO symbol 존재/부재를 production Rust/RON에서 검색했다.
- Live RON, active-run, setup과 event-log version constant를 확인했다.
- Current gameplay/runtime/server canonical의 책임과 실제 문구를 대조했다.

### 변경한 내용

조사 시점에는 변경하지 않았다.

### 실행한 정확한 명령

```bash
rg -n '^#{1,6} |확정 정책|tuning|첫 live' core_enhanced/docs/tmp/defense_route_tactical_command_design_notes_2026-08-04.ko.md
sed -n '<각 DR 확정 정책 및 최종 tuning 범위>' core_enhanced/docs/tmp/defense_route_tactical_command_design_notes_2026-08-04.ko.md
rg -n 'DefenseRoute|WinCondition|survive_timer|Pause|paused|SetBattleSpeed|simulation_delta_for_tick|advance_active_battle_by|ResumeBattle|RetreatBattle' core_enhanced/docs core_enhanced/src game_server/docs game_server/src -g '*.md' -g '*.rs'
rg -n 'ActiveRunCorePayloadV|ACTIVE_RUN_VERSION|BATTLE_EVENT_LOG_VERSION|SETUP_VERSION|LIVE_RON_SCHEMA_VERSION' core_enhanced/src game_server/src
rg -n 'EngagementStance|TacticalRelocateUnit|SwapUnitPositions|SkillCastSelection|FieldWorkObjectDef|FieldWorking|tactical_resource' core_enhanced/src game_server/src game_resources -g '*.rs' -g '*.ron'
```

`sed`는 각 번호의 heading부터 다음 코드 대조 절까지 여러 번 실행했다. 긴 source dump는 이 문서에
복제하지 않고 decision record와 `EXPERIMENT_NOTES.md`의 baseline matrix에 결과를 남겼다.

### 실제 test 수

Read-only source 조사이며 test runner는 실행하지 않았다. 해당 없음.

### Pass / Fail / Skip

해당 없음.

### 실제 결과

- `DR-01`~`DR-17`의 승인 의미와 tuning에 알려진 충돌 또는 미결정 항목이 없었다.
- `DR-01`은 current runtime/canonical과 일치했다.
- `DR-15` clock/revision/actor 기반도 current runtime과 일치했으나 실제 paused gameplay mutation sequence
  focused test가 없었다.
- Future policy production symbol은 없거나 각 Phase가 교체할 legacy seam만 존재했다.
- Current versions는 live RON `1`, Core active-run `2`, server envelope `3`, battle setup `1`, event log
  `27`이다.

### 실패 원인

없음. 첫 version 검색에 존재하지 않는 `core_enhanced/game_resources` 경로가 포함되어 `rg` 경고가
한 번 발생했다. 실제 production data root인 `game_resources`로 재실행해 결과를 확인했다.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

새 decision record가 승인 목표 정책을 소유하고 임시 문서는 historical evidence로 동결한다. Phase 0은
future state/schema를 추가하지 않고 `DR-15`의 current behavior test gap만 보강한다.

## 2026-08-06 — Production 변경 전 focused baseline

### 검증하려던 가설

현재 dirty tree의 `DR-01`과 `DR-15` 기반 test가 변경 전 통과하며, 이후 correction으로 발생한 새 실패를
기존 실패와 구분할 수 있다.

### 수행한 확인

- `DR-01` protected-unit survival/winner와 setup projection
- `DR-15` pause clock, stale authority, revision overflow와 setup recovery
- Server pause tick과 stale error→resync ordering

### 변경한 내용

없음.

### 실행한 정확한 명령

```bash
set -euo pipefail
cargo test -p game_core protect_unit_until
cargo test -p game_core battle_setup_snapshot_exposes_survival_timer_without_encirclement_variant
cargo test -p game_core live_defense_playback_pause_freezes_server_tick_and_resume_advances
cargo test -p game_core supplied_stale_battle_authority_rejects_before_playback_mutation
cargo test -p game_core battle_revision_overflow_rejects_mutation_without_changing_playback
cargo test -p game_core live_defense_setup_loss_recovery_preserves_battle_revision_and_state
cargo test -p game_server paused_live_battle_tick_does_not_push_delta_until_resumed
cargo test -p game_server stale_battle_authority_emits_error_then_current_resync_without_mutation
```

### 실제 test 수

- Core: 7 tests
- game_server: 2 tests
- 합계: 9 tests

각 filtered command가 실행한 0-test binary는 합계에서 제외했다.

### Pass / Fail / Skip

- Passed: 9
- Failed: 0
- Skipped/ignored: 0

### 실제 결과

모든 baseline이 통과했다. 기존 실패는 없다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

Runtime rewrite는 필요하지 않다. Missing coverage를 테스트와 current canonical correction으로 보강한 뒤
같은 baseline과 broad suite를 재실행한다.

## 2026-08-06 — Decision record, Pause regression과 current canonical 보강

### 검증하려던 가설

미구현 target policy를 current canonical과 분리하면서 current `DR-15` semantics를 runtime 변경 없이
direct regression으로 보호할 수 있다.

### 수행한 확인

- Target decision record의 17개 정책 heading과 Phase coverage
- Pause 중 deploy/invalid deploy/withdraw/invalid withdraw의 frozen time, revision과 resource 원자성
- Pause 중 manual skill start, no tick, stored speed 변경과 Resume 뒤 delivery
- 동일 revision 두 server mutation의 first commit/second stale error→resync

### 변경한 내용

- `core_enhanced/docs/decisions/defense_route_tactical_command_policy_2026-08-06.md` 생성
- 임시 조사 문서 historical freeze/superseding 링크 추가
- `game_rulebook.md`, `core_runtime_contract.md`, `game_server/docs/unity_bridge_protocol.md`에 current Pause
  clock/revision/ordering 계약 동기화
- Core paused deploy/withdraw test 추가, 기존 manual skill test에 paused activation/stored-speed 경로 추가
- Server stale test를 same-revision sequential mutation 계약으로 강화
- `cargo fmt --all`로 변경한 Rust 파일 formatting 적용

### 실행한 정확한 명령

```bash
cargo fmt --all
cargo test -p game_core paused_live_battle_commits_deploy_and_withdraw_without_advancing_time
cargo test -p game_core activate_skill_uses_equipped_manual_fragment_in_live_defense
cargo test -p game_server second_mutation_using_same_revision_is_stale_after_first_commit
```

### 실제 test 수

- Core: 2 tests
- game_server: 1 test
- 합계: 3 tests

### Pass / Fail / Skip

- Passed: 3
- Failed: 0
- Skipped/ignored: 0

### 실제 결과

- Paused deploy/withdraw 성공은 time `0`을 유지하며 command마다 revision을 1 증가시켰다.
- 반복 invalid deploy/withdraw는 reconnect projection을 변경하지 않았고 withdrawal은 비용을 환불하지
  않았다.
- Manual cast start와 stored X2 speed는 paused time에 즉시 commit됐고 server tick은 Resume 전 진행하지
  않았다. Resume 뒤 raw 1ms tick이 X2 stored speed로 battle time 2ms를 진행해 ability를 방출했다.
- 같은 revision을 사용한 두 actor mutation은 첫 Pause가 commit된 뒤 둘째 Resume가 stale로 거부되고
  current paused resync를 반환했다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

Focused gap은 닫혔다. Broad validation과 독립 완료 감사를 통과하기 전까지 Status는 `active`로 유지한다.

## 2026-08-06 — Broad validation

### 검증하려던 가설

Decision/canonical/test 변경이 Core와 game_server 전체 package에 새 회귀를 만들지 않았고 production live
RON을 현재 strict schema로 계속 읽을 수 있다.

### 수행한 확인

- Rust formatting
- game_core unit/integration/live-data/skill suites 전체
- game_server unit/actor/persistence suites 전체
- Production RON loading suite 별도 재실행

### 변경한 내용

없음.

### 실행한 정확한 명령

```bash
set -euo pipefail
cargo fmt --all -- --check
cargo test -p game_core
cargo test -p game_server
cargo test -p game_core --test ron_loading
```

### 실제 test 수

- `cargo fmt --all -- --check`: test runner 아님
- `cargo test -p game_core`: 847 tests
  - lib: 800
  - `live_item_skill_activation`: 3
  - `live_skill_catalog_audit`: 4
  - `ron_loading`: 23
  - `skill_runtime_contract`: 13
  - `skill_test_suite`: 4
- `cargo test -p game_server`: 54 tests
- `cargo test -p game_core --test ron_loading`: 23 tests 재실행

### Pass / Fail / Skip

- game_core: 847 passed, 0 failed, 0 ignored
- game_server: 54 passed, 0 failed, 0 ignored
- ron_loading 재실행: 23 passed, 0 failed, 0 ignored
- Rust format check: pass

### 실제 결과

모든 broad validation이 통과했다. 별도 실행한 live RON 23개도 모두 통과했다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

Broad regression과 production data loading gate가 통과했다. Phase exit audit에서 coverage, legacy,
canonical, scoped diff와 dirty 보존을 별도로 판정한다.

## 2026-08-06 — Phase exit static audit checks

### 검증하려던 가설

Decision record가 각 `DR-01`~`DR-17`을 정확히 한 번 소유하고, future placeholder/compatibility가 production
source에 없으며 문서·Rust diff가 깨끗하고 기존 dirty 변경이 보존됐다.

### 수행한 확인

- Policy heading과 Phase coverage row 수를 ID별/전체로 검사
- 임시 문서 freeze/superseding marker 검사
- Future production symbol 부재 검색
- 새 regression test 존재 확인
- whitespace diff, 최종 status와 changed path 확인

### 변경한 내용

첫 policy 재서술 대조에서 `DR-02`의 authored weapon/encounter current effect와 `DR-13`의 최대 Hard CC
expiry 문구가 decision record에서 지나치게 압축된 것을 발견해 승인 의미를 추가했다. Runtime/data는
변경하지 않았다.

### 실행한 정확한 명령

```bash
for n in $(seq -w 1 17); do
  rg -c "^### DR-${n} —" core_enhanced/docs/decisions/defense_route_tactical_command_policy_2026-08-06.md
  rg -c "^\\| `DR-${n}` \\|" core_enhanced/docs/decisions/defense_route_tactical_command_policy_2026-08-06.md
done
rg -n '목표 정책 source|superseded|historical' core_enhanced/docs/tmp/defense_route_tactical_command_design_notes_2026-08-04.ko.md
rg -n 'EngagementStance|TacticalRelocateUnit|SwapUnitPositions|SkillCastSelection|FieldWorkObjectDef|FieldWorking|tactical_resource' core_enhanced/src game_server/src game_resources -g '*.rs' -g '*.ron'
rg -n 'paused_live_battle_commits_deploy_and_withdraw_without_advancing_time|activate_skill_uses_equipped_manual_fragment_in_live_defense|second_mutation_using_same_revision_is_stale_after_first_commit' core_enhanced/src/game/world/tests/combat.rs game_server/src/game/player_game_actor/handlers.rs
git diff --check
git status --short
git diff --name-status
```

### 실제 test 수

Static search/diff 검사이며 test runner는 실행하지 않았다. 해당 없음.

### Pass / Fail / Skip

해당 없음. 모든 assertion/search gate는 성공했다.

### 실제 결과

- 17개 policy heading과 17개 coverage row가 존재하며 각 ID count는 1이다.
- 임시 문서는 historical/superseded 상태와 target decision link를 가진다.
- Future policy production symbol 검색은 0건이다.
- 새 test 세 경로가 존재하고 `git diff --check`가 통과했다.
- 초기 사용자 dirty 파일은 status에 그대로 남아 있고 삭제/이동된 경로는 없다.

### 실패 원인

첫 전체 count command의 table-row 정규식이 shell quoting과 맞지 않아 heading별 출력 뒤 중단됐다.
정책 파일 문제가 아니라 audit command 문제였으며 단순 row regex로 재실행해 heading/coverage 각각
17임을 확인했다.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

Static audit gate는 통과했다. Detailed evidence와 verdict는 `COMPLETION_AUDIT.md`가 소유한다.
