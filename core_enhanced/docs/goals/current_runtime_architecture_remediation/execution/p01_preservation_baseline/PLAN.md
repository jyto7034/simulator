# P01 Preservation Baseline Execution Plan

상태: `GEX-10 stopped / run-global scope change / successor P2G basis required`

## Current Execution Identity

- Original root authorization: `HA-CRAR-P2G60-001`을 accepted basis로 삼아
  `GS-CRAR-P2G50-001` 전체를 수행하는 `goal-run`.
- Original selected member: `(GS-CRAR-P2G50-001, GOAL-CRAR-P01-PRESERVATION-BASELINE)` /
  `PHASE-CRAR-01-PRESERVATION-BASELINE` / `WP-CRAR-01`.
- Current pass: `P01 GEX-10`, implementation mutation을 멈춘 상태.
- Current user scope decision: `유니티는 검사하지마, core 리팩토링에만 집중해.`
- Scope decision timestamp: `2026-08-22`.
- Current disposition: accepted `001` basis는 아래 scope decision과 양립하지 않으므로 P01 GEX-10을 완료하거나
  GEX-20/P02로 자동 전환하지 않는다.

## Current Core-Only Scope

### Included

- `/mnt/f/work/simulator/core_enhanced`의 Rust Core production code, data publication boundary, Core-owned
  persistence representation, tests, benchmark/support tooling과 canonical Core documentation
- `docs/game_rulebook.md` 기능을 Core route와 executable Core test에 연결하는 preservation manifest
- Core rejection, rollback, retry/idempotency, ordering, persistence, live-data와 public projection oracle
- Little Red/projectile/effect event-heavy 및 64-session/8-way interleaved Core baseline
- 이후 successor Goal Set이 승인하는 Core architecture remediation

### Excluded

- Unity repository, source, document, fixture, test runner, editor state, MCP, revision, status와 content hash의 읽기·검사
- Unity compatibility, client protocol parity, client cutover와 client failure를 completion evidence로 사용하는 것
- `game_server` source 수정과 server architecture remediation
- server/Unity end-to-end suite, deployment, release와 production promotion
- EGO corrosion, normal boss egg, skill-fragment handoff/content와 run relic 임시 정책 구현

`game_server`와 Unity는 이 실행의 read-only consumer도 아니며 완료 증거·blocker·currentness registry member도 아니다.
Core 외부의 stale consumer에 맞추기 위한 shim, fallback 또는 Core contract 약화도 허용하지 않는다.

## Why the Accepted `001` Basis Cannot Continue

기존 artifact bytes는 역사 기록으로 보존하며 덮어쓰지 않는다. 그러나 다음 current requirements가 새 사용자
scope와 직접 충돌한다.

| Existing artifact requirement | Conflict |
| --- | --- |
| `FPF-CRAR-P2G30-001`의 `PER-CRAR-007` | server와 Unity의 same-version hard cutover를 acceptance로 요구 |
| `PP-CRAR-P2G40-001`의 `WP-CRAR-10`, `AG-CRAR-02`, Phase 7 | Unity consumer 구현·검사와 server/Unity coimplementation을 필수화 |
| `GS-CRAR-P2G50-001` P01 | Core/server/Unity route, fixture, behavior delta와 broad suite를 요구 |
| `GS-CRAR-P2G50-001` P07/P08 | Unity cutover와 full Core/server/Unity validation을 완료 조건으로 요구 |
| `HA-CRAR-P2G60-001` | P07/P08 Unity 실행을 mandatory completion evidence로 인계 |

이는 test selector 하나의 수정이 아니라 policy scope, Work Package, Atomic Group, Phase dependency, Goal mapping과
Master completion condition을 바꾸는 `run-global` 변경이다. GEX 문서가 `001` artifact를 직접 수정하거나 Unity
검증을 생략한 채 `Complete / High`를 주장할 수 없다.

## Required Successor Owner Path

가장 이른 owner는 `P2G-10`이다. 새 core-only scope를 current policy basis로 고정한 뒤 변경 영향에 맞춰 다음
successor artifact를 단계별 사용자 요청으로 재작성해야 한다.

1. `P2G-10`: core-only scope와 외부 consumer 비의존 acceptance 확정
2. `P2G-20`: Core source/boundary만 current review basis로 재검토
3. `P2G-30`: successor Final Policy Freeze
4. `P2G-40`: Unity/server work package와 atomic dependency를 제거하거나 Core-owned work로 재구성
5. `P2G-50`: successor core-only Goal Set
6. `P2G-60`: successor Handoff

현재 요청은 문서 scope 정정이며 위 P2G 단계들의 결합 실행을 승인하지 않는다. successor Handoff 전에는 기존
goal-run을 재개하지 않는다.

## Safe P01 Work Preserved Before the Stop

- `docs/architecture/runtime_feature_manifest.v1.json`
  - Rulebook H2~H4 57개 heading의 stable feature ID를 유지한다.
  - locator와 executable evidence를 Core-owned route/test로 한정했다.
  - scope payload는 `included = core_enhanced`, `excluded = game_server/Unity/external client compatibility`다.
- `tools/runtime_architecture_preservation`
  - manifest verifier는 Unity, game_server와 absolute external locator를 거부한다.
  - event-heavy와 64-session/8-way workload harness는 Core만 실행한다.
- `docs/game_rulebook.md`
  - gameplay 의미 변경 없이 preservation locator 행 하나만 추가했다.
- `evidence/P01_PERFORMANCE_BASELINE.json`
  - Core-only workload의 raw sample과 repeatability receipt다.
- Current Core validation already observed before the scope stop:
  `cargo test -p game_core`의 non-zero 1,131 tests passed, failed/ignored 0.

이 산출물은 successor P01의 candidate input일 뿐, stale `001` Goal의 completion이나 future evidence reuse 승인을
뜻하지 않는다. successor basis가 manifest 의미·workload를 채택한 뒤 currentness를 다시 확인해야 한다.

## Discarded Evidence

scope 정정 전에 수행한 game_server/Unity test와 live compatibility observation은 현 실행의 acceptance,
completion, blocker 또는 rebaseline 근거에서 제외한다. 해당 결과에 맞춰 Core/server behavior를 변경하지 않았고
앞으로 재실행하지 않는다. 역사적 시도 자체는 `EXPERIMENTS.md`에 최소 사실만 보존한다.

## Current Validation Contract for a Successor P01

| Dimension | Core-only direct evidence |
| --- | --- |
| `success` | Rulebook feature → Core route → Core executable test manifest |
| `invalid_input` | Core command/data rejection과 exact pre-state |
| `failure_rollback` | Core staged command failure와 state/event/revision equality |
| `retry_idempotency` | 동일 Core basis retry와 side-effect identity |
| `ordering_concurrency` | event-heavy same-bucket 및 64-session/8-way Core harness |
| `boundary_empty_max` | Core empty/min/max workload와 inventory coverage |
| `disconnect_restart_persistence` | Core V4/V5 continuation representation과 restart oracle; server store 제외 |
| `live_data` | Core production RON loader corpus |
| `producer_consumer` | Core command executor → Core result/public projection; external client 제외 |
| `legacy_fallback_absence` | Core public API, writer, fallback와 bypass inventory |

### Completion Conditions

- stable Core feature inventory 100%, unknown/unmapped/duplicate 0
- current Core behavior/rejection/continuation/public projection의 non-zero executable evidence
- event-heavy 및 64-session/8-way Core baseline의 exact input/toolchain/raw sample과 repeatability
- required validation dimensions의 Core-local direct mapping
- Core production behavior delta 0인 preservation baseline
- oracle-only production branch/fallback 0과 unrelated dirty work 보존
- successor P2G basis가 위 core-only scope와 completion contract를 명시적으로 채택

## Dirty Work and Mutation Boundary

- 광범위한 pre-existing dirty/untracked work는 사용자 작업으로 간주하고 되돌리거나 현재 Goal 성과로 귀속하지
  않는다.
- 현재 task-owned write surface는 위 preservation manifest/tool과 이 P01 execution document set뿐이다.
- existing dirty Core path가 필요하면 mutation 전 exact bytes와 intended hunk를 고정하고 user-owned change와
  분리할 수 있어야 한다.
- Unity와 game_server에 대한 추가 read/write/status/hash/test는 수행하지 않는다.

## Stop Conditions and Current Result

- Current result: `GEX-10 stopped / run-global scope change`.
- Production Core remediation: 시작하지 않음. P01은 preservation support candidate만 작성됨.
- GEX-20: 시작하지 않음.
- P02~P08: 시작하지 않음.
- Required next owner: `P2G-10`.
- Stop reason: current user-approved core-only execution scope와 accepted `001` Freeze/Plan/Goal Set/Handoff의
  Unity/server mandatory scope가 모순됨.
