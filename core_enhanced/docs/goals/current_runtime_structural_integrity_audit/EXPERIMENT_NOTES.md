# Experiment Notes

## 초기화 기록

- 초기화 날짜: 2026-08-21
- timezone: Asia/Seoul
- Goal 경로: `docs/goals/current_runtime_structural_integrity_audit`
- Goal 경로 충돌: 없음
- 초기 상태: 필수 Goal 문서와 `FINDINGS.md`가 모두 존재하지 않았음
- repository root: `/mnt/f/work/simulator`
- runtime target: `/mnt/f/work/simulator/core_enhanced`
- runtime 조사: 시작하지 않음
- baseline test: 실행하지 않음
- Goal 실행 prompt: `GOAL_PROMPT.md`로 작성함

초기화 전 허용 범위 안에서 다음 명령 성격의 확인만 수행했다.

- Goal 디렉터리와 필수 문서 존재 여부 확인
- repository root 확인
- `git status --short`

## 초기 Dirty Worktree

Goal 초기화 시 repository는 대규모 dirty 상태였다.

확인된 범주:

- `core_enhanced`의 `Cargo.toml`, runtime source, test와 canonical 문서에 다수의 수정·삭제가 존재함
- `core_enhanced/src/game/**`에 다수의 수정 파일과 untracked 신규 runtime 파일이 존재함
- 인접 `game_resources` live data에 수정과 untracked 항목이 존재함
- 인접 `game_server` source, config, 문서와 persistence 파일에 수정·삭제·untracked 항목이 존재함
- 과거 여러 Goal 디렉터리가 untracked 상태임
- 새 감사 기준 문서 `docs/audit/current_runtime_structural_integrity_audit_criteria.ko.md`가 untracked 상태임

이 변경은 모두 사용자 소유로 간주한다. 감사자는 기존 변경을 정리, revert, stash, 포맷 또는 삭제하지
않는다.

현재 commit만으로 감사 기준선을 식별할 수 없다. 실제 감사 시작 시 audited runtime file content manifest와
전체 dirty 상태를 다시 기록해야 한다.

## 구조적 판단

아직 runtime 구조를 조사하지 않았다.

## 감사 기준선 B-001

- 감사 시작: `2026-08-21T23:03:28+09:00 KST`
- HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- 대상: `../Cargo.lock`, `Cargo.toml`, `src/lib.rs`, `src/game/**`, `tests/**`,
  `../game_resources/data/**`의 현재 regular file 210개
- content identifier:
  `e30995cec4249a92c1d3981d4e7b5e64c153dac4b6e5df90d4f7d9fbb510fb61`
- toolchain: rustc 1.91.1, cargo 1.91.1, x86_64 WSL2 Linux
- baseline feature/target: 최초 sweep에서 `--all-targets --all-features`
- Cargo/Rust 관련 환경 변수: 명시 설정 없음
- 전체 dirty status와 재현 명령: `BASELINE.md`

이 식별자는 runtime/test/dependency lock와 이후 C07 조사에 필요한 live-data를 포함한다. Goal 문서 변경은
감사 산출물이며 identifier 범위 밖이다. 포함 표면이 외부에서 변경되면 즉시 중단하고 기존 기준선 종료와
새 기준선 재시작 중 사용자 결정을 받는다.

## 최초 전체 test baseline 판정

- experiment: `E-002`
- 명령: `cargo test --all-targets --all-features`
- 결과: 10개 runner binary, 안전하게 합산 가능한 1131 tests, 1131 pass, 0 fail, 0 ignored
- 판정: all-pass baseline
- 증거 한계: suite 통과는 현재 구현과 기존 test 사이의 일관성만 보이며 상태 소유권, 원자성, 결정론,
  persistence 행동 동등성, 외부 경계, 성능 상한 또는 AI 누적 부채를 자동으로 합격시키지 않는다.
- 이후 실행 정책: audited production/test/live data/dependency/environment가 유지되는 한 동일 full suite를
  재실행하지 않는다. 추가 probe가 필요하면 실행 전 가설과 비중복 증거를 이 문서에 먼저 기록한다.

## C00 formatter/lint 판단

- E-004: `cargo fmt --all -- --check` pass
- E-005: all-target/all-feature clippy with `-D warnings` fails (lib 82, lib-test 133 diagnostics before abort)
- 해석: build/test 가능성과 formatting은 확인됐지만 warning-free lint gate는 성립하지 않는다. Clippy
  진단 개수는 스타일과 test 진단이 섞인 signal이므로 구조 결함 수로 환산하지 않는다.
- feature: 유일한 opt-in feature `event_log_checks`는 E-002/E-005의 `--all-features`에 포함되었다. Default
  feature build는 all-features 성공에 의해 compile subset으로 간접 지원되지만 별도 runner를 반복하지 않는다.
- warning suppression: `Cargo.toml`에는 lint table이나 rustflags가 없고 삭제된 local `.cargo/config.toml`은
  현재 파일 시스템에 존재하지 않는다. E-020 found only localized `too_many_arguments`/`dead_code` allows; E-005
  still exposes the full failing `-D warnings` gate rather than a crate-wide suppression.

## Source-of-Truth 충돌

사용자에 따르면 현재 rulebook 정책과 runtime code는 의도적으로 불일치한다. 따라서 policy 문서와 다른
구현을 이 감사의 결함으로 판정하지 않는다.

과거 Goal과 과거 감사는 현재 코드의 품질 증거로 사용하지 않는다.

## 기록 오염 방지 원칙

이 Goal에서는 raw observation과 최종 판단 사이의 시간을 최소화한다.

- 코드 위치와 호출 경로를 확인하면 즉시 기록한다.
- candidate finding은 확정될 때까지 보류하지 않는다.
- 새로운 증거는 기존 관찰을 덮어쓰지 않고 시간순으로 추가한다.
- 잘못된 가설은 삭제하지 않고 반증 근거와 함께 보존한다.
- 최종 취합은 ID 연결과 root-cause 중복 제거만 수행한다.
- 최종 취합 단계에서 없던 관찰이나 실행 결과를 새로 만들어내지 않는다.

## 정책 질문

현재 없음.

정책에 의존하는 gameplay 의미를 만나면 구조적 사실과 정책 판단을 분리하고, 이 감사에서 의미를
결정하지 않는다.

## 외부 시스템과의 직접 의존성

완료된 제한 범위는 embedded `game_resources` source inventory, direct active-run persistence, direct behavior
transport adapter and tick/event-cursor reference다. 각 접근 근거와 snapshot hash는 E-012/E-014/E-016/E-018 및
아래 사전 기록에 있다. 오래된/현재 Unity 구현과 server-wide behavior는 비범위로 접근하지 않았다.

### 2026-08-21 23:37 KST — game_server active-run persistence 접근 승인 근거

- criterion: C08 (save/checkpoint/restore), 제한적으로 C06/C10
- necessity: core는 versioned bytes와 two sideband values만 반환하고 filesystem/database atomicity, corruption,
  concurrency and reconnect load semantics를 구현하지 않는다. C08 완료 조건의 actual durable write/restore boundary는
  direct consumer를 보지 않고 판정할 수 없다.
- narrow read scope: `../game_server/src/persistence/active_run_store.rs`, persistence module wiring, 그리고 이 store의
  direct save/load/delete call sites만. Server 전체 구조/transport/Unity compatibility는 조사하지 않는다.
- baseline separation: server files are B-001 content digest 대상이 아니므로 읽기 시각의 `git status`와 inspected
  files aggregate hash를 별도 adjacent identifier로 기록한다. Core finding과 server-wide assessment로 확대하지 않는다.

### 2026-08-21 23:39 KST — adjacent persistence 조사 결과

- identifier: `ADJ-PERSIST-001`; six exact file hashes/status are E-012.
- scope used: active store, persistence replacement/sync helper, actor direct transaction ranges, standard-completion recovery,
  startup wiring only.
- not inspected: server transport routing as a whole, other stores, database/network behavior, Unity consumer.
- conclusion limit: C08의 direct filesystem boundary만 증명하며 `game_server` 전체 구조 합격을 뜻하지 않는다.
- no server tests run: E-002 is a core crate baseline and does not include adjacent store tests.

### 2026-08-21 23:42 KST — game_resources embedded source inventory 접근 승인 근거

- criterion: C07
- necessity: official loader uses `include_str!` from 24 direct resource files and a manifest. Code path alone cannot prove
  the B-001 files are present/non-empty or identify the exact embedded bytes audited, although E-002 proves they compile/load.
- narrow read scope: `../game_resources/data/bundle.ron` and the exact 24 paths listed by
  `LIVE_GAMEPLAY_RON_SOURCE_PATHS`; collect path/size/SHA-256 and read only the small manifest. Do not audit balance/content.
- baseline: these files are already included in B-001 aggregate; a path-level adjacent inventory strengthens C07 without
  creating a new data baseline.

### 2026-08-21 23:47 KST — behavior transport seam raw observation

- `PlayerBehavior` is the strict external input schema, but its fields still reuse domain identifiers/enums.
- `BehaviorResult` and `GameError` derive serde directly without the input enum's explicit stable tag/rename/unknown-field
  contract. `BehaviorResult` mixes DTO projections with multiple internal/domain structures and recursive results.
- The separate `GameError::code()` mapping is a stable adapter seam, but static core inspection alone does not prove the
  adjacent server uses that mapping rather than direct enum serialization.
- Record candidate RUNTIME-AUDIT-008 now; inspect only the direct server transport conversion/serialization caller after
  a separate scope note.

### 2026-08-21 23:48 KST — game_server direct behavior transport adapter 접근 승인 근거

- criterion: C06, 제한적으로 C05/C10/C13
- necessity: core type inspection proves potential wire coupling but cannot distinguish a public-Rust-only concern from
  the active transport contract. The severity of RUNTIME-AUDIT-008 requires the direct `BehaviorResult`/`GameError`
  conversion or serialization caller.
- narrow read scope: `../game_server/src` exact references to `BehaviorResult`, `GameError`, `serde_json::to_*`, and only
  the directly containing handler/response definitions. Do not inspect unrelated transport, authentication, database, or
  Unity compatibility behavior.
- baseline separation: record exact hashes/status for any adjacent files opened; do not make a server-wide claim.

### 2026-08-21 23:50 KST — adjacent transport 조사 결과

- identifier: `ADJ-TRANSPORT-001`; exact three-file status/hash is E-016.
- the active adapter uses central `command_result_projection` and `GameError::code()`; it does not directly send whole core
  enums. Battle updates use purpose-built DTO side messages.
- the envelope is unversioned and payload/snapshot are `Value`; central projection continues to serialize multiple internal
  domain structures directly. Core's exhaustive 47-case test intentionally compares the adapter payload to old direct
  enum serde, preserving compatibility but also preserving the coupling.
- conclusion limit: direct behavior transport only; no Unity consumer or server-wide protocol assessment.

### 2026-08-21 23:51 KST — adjacent tick/delta reference scope sequencing correction

- During E-018, one `rg` over `../game_server/src/game/player_game_actor` was issued to determine whether live delta
  extraction is a real per-tick consumer before this explicit note was appended. This was a process-order deviation.
- actual access remained read-only and narrow: symbol/line matches for tick delta and event cursor only; no new server file
  contents were opened and the matching handler/state files were already hashed by ADJ-PERSIST-001/ADJ-TRANSPORT-001.
- necessity: distinguishes a theoretical public-core O(N) query from a direct live-tick O(N) production call.
- scope limit: no server-wide performance verdict, configuration evaluation, network/load test, or Unity claim.

### 2026-08-21 23:59 KST — E-021 focused event-log scaling probe 사전 기록

- criterion: C11/C12; finding: RUNTIME-AUDIT-009.
- static evidence already proves O(total history) scanning, but C12 explicitly requires representative sizes and measured
  environment/input. A bounded focused runtime measurement therefore adds required magnitude evidence and is not a repeat
  of E-002.
- method: compile a `/tmp`-only Rust probe against the exact B-001 `game_core` rlib produced by E-002. Populate public
  `BattleEventLog` with simple fixed-size `BattleStart` entries at several N values; repeatedly request a constant two-entry
  tail; use `black_box`; report elapsed time and ns/call. Also measure whole-log extraction only at bounded sizes to expose
  clone/allocation cost.
- isolation: no source/test/live-data/adjacent-server modification; probe source and binary live under `/tmp`. Do not rerun
  full/focused cargo tests. Results are diagnostic, not a universal performance threshold.
- stop rule: if exact B-001 rlib cannot be identified or compilation fails, record failure and retain static complexity
  evidence rather than altering the crate.

## 범위 밖 후속 후보

- RUNTIME-AUDIT-005: six live command families를 cursor preflight/staged transaction으로 통일하고 GAP-001 회귀
  테스트를 추가하는 최우선 remediation.
- RUNTIME-AUDIT-006/007: validated data type-state와 typed `try_build`, StartNewGame transaction/equality test.
- RUNTIME-AUDIT-009: live delta index/retention/work budget and permanent scaling guard.
- RUNTIME-AUDIT-003/004/001: fixture/debug public authority 격리, finite WorldVec2/event boundary, test setter 비노출.
- RUNTIME-AUDIT-008: versioned explicit result/snapshot DTO and compatibility fixtures.
- GAP-003 plus adjacent store tests: original-vs-continued canonical command equivalence and dirty adjacent server test run.

이 후보들은 이 감사에서 구현하거나 새 Goal로 생성하지 않았다.

## 적용하지 않은 대안

### 기존 과거 감사 재사용

적용하지 않는다. 현재 tree와 정책 상태가 다르고 사용자가 과거 Goal을 참고하지 말라고 지정했다.

### 감사와 remediation 동시 수행

적용하지 않는다. Production 수정은 기준선을 오염시키고 finding 누락과 회귀 귀속 혼선을 만들 수 있다.

### criterion마다 test 재실행

적용하지 않는다. 최초 full-suite baseline이 all-pass이면 변경 없는 동일 suite의 반복은 새 증거를 제공하지
않는다.

## 접근하거나 검증하지 못한 소비자

- Unity consumer: 비범위, 접근·검증하지 않음
- game_server: direct persistence/transport/tick seam만 read-only로 확인; server 전체 auth/network/store/config behavior,
  load/performance and all tests는 접근·검증하지 않음
- separate-process/cross-platform Rapier/float determinism consumer: 실행하지 않음

## 생략한 검증과 근거

- E-002가 all-pass였고 production baseline이 E-022까지 동일하므로 full/focused cargo tests를 반복하지 않았다.
- adjacent game_server tests는 B-001 core baseline이 아니며 server-wide 확대를 피하기 위해 실행하지 않았다; C08을
  PARTIAL로 유지했다.
- cross-process/platform physics probe는 same-process insertion probe가 pass했고 public non-finite boundary alone already
  makes C04 FAIL, so 추가 비용 대비 verdict를 바꾸지 않는다.
- generic multi-domain original-vs-continued command equivalence는 기존 증거가 representative only라 GAP-003으로 남겼다.
- clippy는 E-005의 실제 failure를 보존하고 수정/재실행하지 않았다.
- only additional runtime probe was pre-noted E-021, because C12 required representative measured scale.

## 남은 위험

- B-001은 commit만으로 재현되지 않는 dirty baseline이므로 210-file aggregate가 필수다; E-022 exact match 확인.
- one S1, six S2, one S3 Active finding remain; production remediation is outside this Goal.
- C08 remains PARTIAL because mandatory continuation behavior equivalence is representative and adjacent tests unexecuted.
- Unity/server-wide/cross-platform consumers remain explicitly unassessed; no criterion is UNKNOWN because their exclusion
  does not prevent a core structural verdict.
