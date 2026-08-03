# Failure, Checkpoint, And Active-Run Persistence Experiments

Status: complete

Implementation complete: true

Created: 2026-07-27

## 2026-07-27 — Child Goal 초기화

### 확인

- 신규 경로가 존재하지 않아 충돌이 없음을 확인했다.
- P00 completion review는 `Complete / High`이며 Master registry에서 P01이 active다.
- 저장소와 외부 Unity worktree는 기존 대규모 dirty 상태이므로 대상별 scoped diff gate가 필요하다.

### 생성한 문서

- `PLAN.md`
- `EXPERIMENTS.md`
- `EXPERIMENT_NOTES.md`

### Production 변경

- 없음. P01 child 문서만 생성했다.

### 다음 작업

- 필수 child 문서 3개와 P00 completion review, Master P01 및 승인 decision 재확인
- scoped current-state/dirty audit와 baseline

## 2026-07-27 — 재개 선독과 current-state/dirty audit

### 선독

- Master 필수 3문서, README/workflow/completion guide, 승인 decision 5개를 완독했다.
- 활성 P01 필수 3문서, 직접 선행 P00 `COMPLETION_REVIEW.md`, Core durable canonical 3개와 외부
  Unity `unity_core_contract.md`/`unity_client_implementation_goal.md`를 완독했다.
- P00은 `Complete / High`, Master registry와 P01 Status는 `active`로 일치한다.

### 실제 current-state 발견

- P01-A/B 일부 command/state/evaluator/Treatment shape가 dirty worktree에 이미 존재한다.
- P01-C durable active-run host persistence는 아직 없다.
- Treatment usefulness 분리와 모든 no-deployable 경로의 공통 evaluator 사용에 명시적 gap이 있다.
- Core/server/Unity 대상 파일 모두 기존 누적 변경과 겹치므로 current 파일 위에 국소 병합한다.

### Production 변경

- 이 기록 전까지 없음.

### 다음 검증

- `cargo check -p game_core`
- `cargo check -p game_server`
- P01 관련 focused Core test filter와 현재 test count 확인

## 2026-07-27 — 최초 baseline compile/focused test

### 실행한 정확한 명령과 결과

| 명령 | 실제 실행 test 수 | Pass | Fail | Skip | 결과 |
| --- | ---: | ---: | ---: | ---: | --- |
| `cargo check -p game_core` | 0 | 해당 없음 | compile 1 | 0 | 삭제된 `qualified_recovery_node_ids_for_completion` 호출 2곳 때문에 실패 |
| `cargo check -p game_core` (국소 수정 후) | 0 | 해당 없음 | 0 | 0 | 성공; 기존 unused warning 2건 |
| `cargo check -p game_server` | 0 | 해당 없음 | 0 | 0 | 성공; Core warning 2건 |
| `cargo test -p game_core no_deployable -- --nocapture` | 4 | 1 | 3 | 0 | 구 recovery restriction/checkpoint 기대와 현재 target code 충돌 |
| `cargo test -p game_core physical_treatment -- --nocapture` | 3 | 3 | 0 | 0 | 기존 damage > 0/Dead/amount validation 범위 통과 |
| `cargo test -p game_core run_failure -- --nocapture` | 0 | 0 | 0 | 0 | 이름 filter와 일치하는 test 없음 |

### 실패 원인과 수정

- Partial P01 refactor가 recovery evaluator를 `(allowed_node_ids, path_exists)`로 교체했지만 전투 결과
  두 경로는 삭제된 구 helper 이름을 호출했다.
- 두 경로를 새 evaluator에 연결하고 treatment-only 경로처럼 허용 node가 0개여도
  `path_exists == true`인 경우 restriction을 유지하도록 수정했다.
- `no_deployable` 실패 3건 중 checkpoint test는 target 정책과 반대인 즉시 RunFailed를 기대한다.
  나머지 2건은 SavePoint가 restriction에서 허용되지 않는 구 기대를 사용한다. Production current
  evaluator는 승인 정책대로 SavePoint/Gate를 허용 action에 포함하므로 test migration이 필요하다.
- `physical_treatment` 3건은 physical damage 0에서의 구조적 사용 허용을 검증하지 않아 target gap을
  가리지 못한다.

### 변경/되돌림

- `world/combat.rs`의 끊긴 evaluator 호출과 path/node 구분만 국소 수정했다.
- 기존 변경이나 관련 없는 파일을 되돌리지 않았다.

## 2026-07-27 — P01-A/B recovery·failure·Treatment 수렴

### 구현

- 모든 전투 진입/선택 경로가 같은 recovery evaluator를 사용하도록 통합했다. evaluator는 현재 Floor의
  즉시 선택 가능한 SavePoint/Gate, 실제 HQ recruitment와 inventory의 물리 피해 회복 consumable을
  조합하고 `allowed_node_ids`와 node가 없어도 가능한 treatment-only path를 분리한다.
- 출전 불가지만 회복 경로가 있으면 `NoDeployableEmployees` restriction을 유지하고 consumable,
  Physical Treatment, 허용 node 이동 뒤 동일 evaluator를 재실행한다. loadout 변경은 restriction에서
  허용하지 않는다.
- 모든 gameplay failure 후보는 loadable checkpoint가 있으면 먼저
  `AwaitingRunFailureDecision`으로 전환한다. 이 상태에서는 `LoadRunCheckpoint`와
  `ConfirmRunFailure`만 허용하며 BossOmen 전용 post-terminal 예외는 제거했다.
- Physical Treatment는 살아 있는 직원이면 현재 물리 피해가 0이어도 구조적으로 사용할 수 있다.
  지속 modifier 교체는 command commit 안에서 원자적으로 수행하고 별도 overwrite modal을 제거했다.
- Unity는 recovery restriction, 공통 failure modal, loadable checkpoint와 정확한 위치를 Core
  projection에서 읽으며 Run 포기는 별도 확인을 요구한다.

### 검증한 핵심 경계

- recovery node/consumable/HQ/treatment-only positive·negative 경로와 action 재평가
- damage 0 Treatment eligibility, Dead rejection과 기존 modifier 원자 교체
- invalid/non-loadable checkpoint에서 load가 횟수나 state를 소비하지 않고 failure confirm은 가능한 경로
- final boss를 포함한 공통 failure 사전 선택과 terminal 이후 load 불가

## 2026-07-27 — P01-C durable active-run lifecycle

### Core envelope와 round-trip

- Core에 versioned active-run export/import 경계를 추가했다. payload는 checkpoint-owned map,
  recruitment, inventory, roster, research, visual-novel과 run state를 round-trip한다.
- transport `player_id`와 local encounter compendium profile은 payload에서 제외하고 continue 시 현재
  host identity/profile을 주입한다.
- successful checkpoint load만 `loads_used`를 증가시키며 import/continue와 process restart가 이
  monotonic metadata를 초기화하지 않는다.
- import validation/restore failure는 whole-state rollback으로 partial mutation을 남기지 않는다.

### Host/server storage

- `game_server::persistence::ActiveRunStore`를 추가했다. 명시적 root 아래 version 1 JSON envelope,
  Base64 Core payload, SHA-256 checksum, current/rolling backup/temp/tombstone와 lifetime file lock을
  사용한다.
- temp write·file fsync·재검증·rename publish·directory fsync 순서를 구현하고 current 손상 시 valid
  backup을 복구한다. current/backup 모두 손상, unsupported version과 checksum mismatch는 명시적
  오류다.
- backup fallback에서도 `loads_used`가 감소하지 않으며 terminal invalidation 뒤 이전 run backup이
  부활하지 않는다.
- active-run store는 player actor별 map이 아니라 load-balance가 공유하는 local-install singleton이다.
  RunComplete, confirmed RunFailed, AbandonRun, 명시적 새 run 대체에서 무효화한다.
- `ContinueRun`, `ResetActiveRun`과 `StartNewGame { replace_active_run }` host action을 추가했다.
  손상 파일은 reset 전까지 차단하고 backup recovery는 최초 snapshot/continue 상태로 투영한다.

### Unity lifecycle

- 메인 메뉴에 Core/server capability 기반 Continue/Reset을 추가하고 새 run 대체 확인을 명시적으로
  보낸다.
- checkpoint `exists`/`loadable`/count/location을 strict reader로 소비하며 failure modal은 load 가능
  여부와 무관하게 failure 확정 action을 유지한다.
- modifier overwrite modal과 stale consumable pending state를 제거하고 consumable command를 즉시
  전송한다.

## 2026-07-27 — 최종 검증과 환경 분리

### Rust

| 명령 | 실제 실행 test 수 | Pass | Fail | Skip | 결과 |
| --- | ---: | ---: | ---: | ---: | --- |
| `cargo test -p game_core --lib` | 762 | 762 | 0 | 0 | 성공 |
| `cargo test -p game_server --lib` | 40 | 40 | 0 | 0 | 성공 |
| `cargo test --workspace --all-targets` | Core library 762 + integration 3/4/23/13/4 + server 40 | 전부 | 0 | 0 | 성공 |

Workspace integration group은 `live_item_activation` 3, `live_skill_catalog_audit` 4,
`ron_loading` 23, `skill_runtime_contract` 13, `skill_test_suite` 4이며 example/main target은 test 0건으로
성공했다.

### Unity

- 임시 프로젝트 복사본의 batchmode EditMode: 246/246 성공.
- 임시 복사본의 P01 SavePoint focused PlayMode: 2/2 성공.
- batchmode broad PlayMode는 58 pass, 3 fail, 4 skip이었다. 두 failure는 batchmode에서 지원하지 않는
  `WaitForEndOfFrame`, 한 failure는 비P01 preparation pan 임계값이었다. 이 결과를 제품 통과로
  오인하지 않고 원본 non-batch Editor MCP에서 다시 실행했다.
- 원본 `F:/unity projects/ark`, Unity `6000.3.4f1` MCP EditMode job
  `cca002a0078845c4bfb52f4708eb122f`: 246/246 성공.
- 같은 원본 Editor MCP PlayMode job `8d8d9140e04a48238d25ce48202eabf8`: 총 65,
  61 pass, 0 fail, live server가 필요한 explicit 4 skip으로 성공.

### Legacy/contract 감사

- official Core/server/Unity current contract에서 `NoPhysicalDamage`, modifier overwrite confirmation과
  BossOmen post-`RunFailed` load exception을 제거했다.
- active-run과 local profile은 별도 aggregate이며 active-run payload/envelope에 compendium이나
  transport player identity를 넣지 않았다.
- canonical `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`와 Unity
  `unity_core_contract.md`, `unity_client_implementation_goal.md`, 관련 UI/probe contract를 runtime에
  맞춰 갱신했다.

## 2026-07-27 — Completion review

- 현재 runtime/data/server/Unity/canonical을 Goal 주장과 독립적으로 다시 대조했다.
- `COMPLETION_REVIEW.md` verdict는 `Complete / High`다.
- 정책 질문과 immediate correction 후보는 없다.
