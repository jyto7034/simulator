# Compendium Transaction Rollback Integrity Correction Experiments

Status: complete

Implementation complete: true

Created: 2026-07-28

## 2026-07-28 — Child Goal 초기화

### Dependency와 activation

- Correction Master status: `active`, implementation false.
- Master Phase P00 hard prerequisite: 없음.
- Child 경로는 생성 전에 존재하지 않았다.
- 승인된 policy question: 없음. 재감사 F-01과 R-01이 rollback/profile lifetime을 닫고 있다.

### 필수 선독

- Correction Master 필수 문서 3개를 끝까지 읽었다.
- `docs/README.md`, `engineering_workflow.md`, `goal_completion_review_guide.md`를 끝까지 읽었다.
- 기존 implementation Master `PLAN.md`, P00~P05/P07의 PLAN/EXPERIMENTS/EXPERIMENT_NOTES/
  COMPLETION_REVIEW와 2026-07-28 재감사 메모를 끝까지 읽었다.
- 기존 P00 기록 header 불일치 D-01은 final correction P04 범위이며 현재 P00 production scope가 아니다.

### Path/dirty gate

- Core/workspace Git root: `/mnt/f/work/simulator`.
- Unity Git root: `/mnt/f/unity projects/ark`.
- 두 저장소 모두 광범위한 기존 tracked/untracked 변경이 있다.
- Core P00 후보인 `world/state.rs`, `world/node_flow.rs`, `world/active_run.rs`, compendium 및 server actor/
  persistence 관련 경로도 현재 campaign 누적 변경과 겹칠 수 있어 production 수정 전 scoped diff가 필요하다.
- 기존 변경은 사용자 변경으로 보존하며 전체 reset/restore/stash/clean/format을 하지 않는다.

### Production 변경과 test

- 이 기록 시점 production code/data/server/Unity/canonical 변경 없음.
- Test runner를 실행하지 않았다.
- 실제 test 수와 pass/fail/skip: 해당 없음.

### 다음 작업

- P00 관련 current runtime/state mutation과 scoped dirty diff 조사
- focused Core/server baseline 및 실제 test count 확보
- 조사 근거를 기록한 뒤 production correction 구현

## 2026-07-28 — Current-state, dirty overlap와 focused baseline

### 실제 current-state evidence

- `GameCoreState::clone_non_battle_transaction_state()`는 rollback copy의
  `encounter_compendium`을 default로 둔다.
- `GameCore::with_state_transaction()`은 operation 실패 시 현재 live profile을 보존하지 않고 그 copy를
  `self.state`에 직접 대입한다. 따라서 transaction 전 knowledge와 transaction 중 commit된 knowledge를
  모두 잃을 수 있다.
- `restore_active_run_transaction_rollback()`만 restore 직전 live compendium을 보존하고 restore 뒤
  `merge()`한다. 공통 transaction과 active-run transaction이 서로 다른 profile lifetime을 사용한다.
- `PlayerGameActor::handle(AttachSession)`은 `build_state_snapshot()`을 먼저 호출한다. Durable profile sync는
  일반 `ExecutePlayerBehavior` 시작 전에만 수행되므로 같은 actor의 Core profile이 stale/empty인 경우
  reconnect snapshot이 durable singleton union보다 작을 수 있다.
- Persistence `Fatal` 상태에서는 AttachSession이 error status와 Retry/Reset capability를 투영해야 하므로
  attach-time sync는 readable profile이 있을 때만 merge하고 fatal recovery UX를 막지 않아야 한다.

### Scoped dirty gate 판정

- 관련 tracked 파일에는 기존 P00~P07 campaign의 대규모 누적 diff가 있고 compendium/active-run 신규
  modules와 server persistence directory는 untracked다.
- 현재 target 함수와 기존 completion tests는 같은 승인된 monotonic profile 방향이다. 필요한 변경은
  공통 restore 한 곳, AttachSession ordering과 직접 회귀 test에 국한할 수 있어 기존 의미를 보존하며
  안전하게 병합 가능하다고 판정했다.
- Profile schema/store/Unity 파일은 변경할 필요가 없다.

### 수정 전 focused baseline

| 명령 | Pass | Fail | Ignored/filtered | 결과 |
| --- | ---: | ---: | ---: | --- |
| `cargo test -p game_core encounter_compendium --lib -- --nocapture` | 5 | 0 | 782 filtered | 성공 |
| `cargo test -p game_server encounter_compendium --lib -- --nocapture` | 5 | 0 | 47 filtered | 성공 |
| `cargo test -p game_core active_run_transaction_rollback_does_not_rewind_monotonic_profile_or_power_authority --lib -- --nocapture` | 1 | 0 | 786 filtered | 성공 |
| `cargo test -p game_server active_run_write_failure_does_not_rollback_or_skip_compendium_publish --lib -- --nocapture` | 1 | 0 | 51 filtered | 성공 |
| 잘못 추정한 filter `backup_recovery_notification_is_consumed_only_once_across_reconnects` | 0 | 0 | 52 filtered | 실행 test 0; baseline 근거로 사용하지 않음 |

### Baseline 판단

- 기존 focused suite는 profile/store/active-run 전용 경계는 통과하지만 common transaction failure와
  AttachSession-before-snapshot sync를 직접 assert하지 않아 F-01을 검출하지 못한다.
- 다음 단계는 두 직접 회귀 test를 먼저 추가해 현재 실패를 재현한 뒤 production correction을 적용한다.

## 2026-07-28 — 직접 회귀 test의 수정 전 실패 재현

### 추가한 test

- Core `failed_common_transaction_preserves_profile_knowledge_from_before_and_during_operation`
- Server `attach_session_merges_durable_compendium_before_building_snapshot`

### 최초 실행과 test 자체 수정

- Core 첫 실행은 `GameError`에 `PartialEq`가 없어 `assert_eq!`가 compile error 1건을 냈다. Production
  결함 결과가 아니므로 test를 `matches!` assertion으로 고치고 재실행했다.
- 이 test compile 수정은 production type에 derive나 compatibility를 추가하지 않았다.

### 수정 전 실제 결과

| 명령 | Pass | Fail | Filtered | 확인한 결함 |
| --- | ---: | ---: | ---: | --- |
| `cargo test -p game_core failed_common_transaction_preserves_profile_knowledge_from_before_and_during_operation --lib -- --nocapture` | 0 | 1 | 787 | 실패 rollback 뒤 discovered set이 기대 2건 대신 `{}` |
| `cargo test -p game_server attach_session_merges_durable_compendium_before_building_snapshot --lib -- --nocapture` | 0 | 1 | 52 | durable discovered ID가 attach snapshot에서 `false` |

두 회귀가 F-01의 Core rollback과 reconnect projection을 각각 직접 재현했다. 이제 기록된 restore/attach
ordering correction을 적용한다.

## 2026-07-28 — Production correction과 focused 재검증

### 구현

- `GameCore::with_state_transaction()`은 failure restore 직전 live compendium을 clone하고 rollback state를
  복원한 뒤 Core compendium `merge()`로 union한다. Snapshot은 계속 profile copy를 소유하지 않는다.
- `PlayerGameActor::handle(AttachSession)`은 root snapshot 전 readable shared profile을 Core에 merge한다.
- Attach-time persistence가 `Fatal`이면 merge를 생략하고 기존 error status/Retry/Reset snapshot을
  제공한다. 일반 gameplay command의 fatal sync error 의미는 바꾸지 않았다.
- Profile envelope/store/schema, DTO shape와 Unity consumer는 변경하지 않았다.
- `game_rulebook.md`, `core_runtime_contract.md`, server bridge/canonical에 common rollback과 attach ordering을
  current contract로 반영했다.

### 수정 후 focused 결과

| 명령 | Pass | Fail | Filtered | 결과 |
| --- | ---: | ---: | ---: | --- |
| Core 새 common-transaction regression | 1 | 0 | 787 | 성공 |
| Server 새 AttachSession regression 재수집 | 1 | 0 | 52 | 성공 |
| Server fatal bootstrap regression | 1 | 0 | 52 | 성공 |
| Core 기존 active-run rollback regression | 1 | 0 | 787 | 성공 |
| Server 기존 active-run publish failure regression | 1 | 0 | 52 | 성공 |
| Core 기존 `encounter_compendium` filter | 5 | 0 | 783 | 성공 |
| Server 기존 `encounter_compendium` filter | 5 | 0 | 48 | 성공 |

첫 묶음 실행에서 Server 새 AttachSession test는 compile이 30초를 넘어 command 결과 수집이
불완전했으므로, build 완료 뒤 같은 exact test를 재실행해 1/1 pass를 확정했다.

### 다음 작업

- Rust formatting/check와 Core/server broad 검증
- profile dirty/fatal/reset 및 complete workspace 영향 검증
- scoped diff/legacy/canonical audit 후 독립 completion review

## 2026-07-28 — Broad validation과 completion review

### Formatting/check

- 첫 `cargo fmt --all -- --check`는 이번 P00 코드의 formatting diff 3개를 보고했다.
- 해당 줄만 rustfmt 제안 형태로 고치고 재실행한 `cargo fmt --all -- --check`는 성공했다.
- `cargo check -p game_core`: 성공, compile failure 0.
- `cargo check -p game_server`: 성공, compile failure 0.

### Broad validation

| 명령 | Pass | Fail | Skip/ignored | 결과 |
| --- | ---: | ---: | ---: | --- |
| `cargo test -p game_core --lib --quiet` | 788 | 0 | 0 | 성공 |
| `cargo test -p game_server --quiet` | 53 | 0 | 0 | 성공; 추가 0-test targets 성공 |
| `cargo test --workspace --all-targets --quiet` Core lib | 788 | 0 | 0 | 성공 |
| 같은 workspace integration groups | 3/4/23/13/4 = 47 | 0 | 0 | 성공 |
| 같은 workspace server lib | 53 | 0 | 0 | 성공 |

### Scoped audit

- 관련 tracked diff `git diff --check`: 오류 0.
- Child/Master Goal 문서 trailing whitespace: 0건.
- Common/active-run rollback snapshot은 profile copy를 소유하지 않고 restore path만 union한다.
- AttachSession과 command execution은 같은 shared profile authority를 사용하며 fatal recovery snapshot은
  유지된다.
- Profile JSON envelope, store version, DTO field와 Unity reader/presentation은 변경하지 않았다.
- Unity test는 wire/consumer 변경이 없고 P00 Master exit gate가 Core/server focused+broad를 요구하므로
  실행하지 않았다. 실제 server AttachSession snapshot JSON 회귀가 consumer-facing shape/value를 직접
  검증한다.

### Completion review

- `COMPLETION_REVIEW.md`: `Complete / High`.
- 정책 질문, blocker, immediate correction 후보 없음.
