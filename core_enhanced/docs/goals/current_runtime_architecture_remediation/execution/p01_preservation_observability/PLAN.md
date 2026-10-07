# P01 Preservation and Observability Execution Plan

상태: `GEX-00 passed / GEX-10 exit conditions met / GEX-20 pending / current`

## Current Execution Identity

- Handoff: `HA-CRAR-P2G60-002` / `current`
- Root target: `GS-CRAR-P2G50-002` 전체를 terminal result까지 수행하는 `goal-run`
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY)`
- Phase / node: `PHASE-CRAR-C2-01-PRESERVATION-OBSERVABILITY` / `WP-CRAR-C2-01`
- Current pass: `GEX-10 closed`; active goal-run의 다음 분리 pass는 fresh read-only `GEX-20`
- Authorization: 사용자가 8개 Phase의 순차 GEX-00/GEX-10/GEX-20과, 전 Phase가 `Complete / High`일 때
  GEX-30까지 승인했다. Mutation·test 권한은 selected Goal의 Core-only 범위에 한정된다.
- Direct predecessor dependency: `none`
- Goal Quality Contract: `2`

## Execution Document Set

- Directory: `docs/goals/current_runtime_architecture_remediation/execution/p01_preservation_observability`
- Plan: `docs/goals/current_runtime_architecture_remediation/execution/p01_preservation_observability/PLAN.md`
- Experiments: `docs/goals/current_runtime_architecture_remediation/execution/p01_preservation_observability/EXPERIMENTS.md`
- Notes: `docs/goals/current_runtime_architecture_remediation/execution/p01_preservation_observability/EXPERIMENT_NOTES.md`
- Audit: `docs/goals/current_runtime_architecture_remediation/execution/p01_preservation_observability/COMPLETION_AUDIT.md`

`GS-CRAR-P2G50-001`에 결박된 `execution/p01_preservation_baseline`은 stopped historical record다. 그 기록의
candidate는 current actual에서 재검증할 수 있지만 prior success나 completion evidence는 자동 재사용하지 않는다.

## Exact Basis and Scope

- Freeze / Plan / Receipt / Goal Set:
  `FPF-CRAR-P2G30-002` / `PP-CRAR-P2G40-002` / `PVR-CRAR-P2G40-002` /
  `GS-CRAR-P2G50-002`
- Objective: current Rulebook/Core behavior를 stable manifest, slice differential oracle와 reproducible pre-change
  performance observation으로 고정한다.
- In scope: `core_enhanced`의 docs/test manifest, Core fixtures, audit/support tool, current Core route/load/checkpoint/
  projection oracle와 event-heavy/64-session baseline.
- Out of scope: production gameplay behavior 변경, architecture defect 구현, optimization/performance PASS,
  `game_server`, Unity, external compatibility/consumer, deployment/release.
- Boundary: production runtime algorithm/data mutation 없음. Existing dirty work는 사용자 작업으로 보존하며
  task-owned support/document delta와 분리한다.

## Current Baseline

- Repository revision / branch:
  `f318414fb0156a67e04c5479000c2a2683fc0c09` / `cleanup/simulator-dirty-worktree-20260803`
- Core-only dirty baseline after GEX-00 document initialization: modified 134, deleted 23, untracked 3,132,
  total 3,289; porcelain SHA-256 `1358edd332bdf6f4cf5c6f2a92ace757ecfeea98cc002b93e09d12c7a09b4a85`.
  생성된 support-tool `target/`과 광범위한 pre-existing tree를 포함하므로 completion attribution에는 path별
  identity를 별도로 사용한다.
- P01 candidate paths: untracked `docs/architecture/runtime_feature_manifest.v1.json`, untracked
  `tools/runtime_architecture_preservation`, modified `docs/game_rulebook.md`; 이들은 historical `001` pass의
  candidate이며 current `002` GEX-10에서 변경 전 identity와 scoped hunk를 다시 고정한다.
- Candidate manifest SHA-256: `c81f7ffcab5bc9eb47ed3e9cded0d64d8f26a14a743a27c6fab00a09e32d1321`;
  exact P2G reviewed identity와 일치.
- Candidate tool hashes: Cargo manifest `928a1a70…7bee`, baseline source `1e9b01ac…c9de`, verifier
  `5a2f55aa…993f`.
- Durable receipt `evidence/P01_PERFORMANCE_BASELINE.json`은 현재 없음. GEX-10 output이다.
- Runner: cargo `1.91.1`, rustc `1.91.1`; standalone tool workspace가 Core package를 path dependency로 사용한다.
- Upstream Freeze/Plan/Receipt/Goal Set digests는 Handoff와 exact match했다.
- Dirty isolation disposition: P01은 production algorithm/data를 수정하지 않고 candidate support/doc seam만
  current Goal에 귀속한다. 이 범위에서는 existing dirty work와 안전하게 분리 가능하다.

## Validation and Completion Contract

- 57 features의 Rulebook → actual Core route → executable test/data reverse coverage 100%; unknown/unmapped/duplicate 0
- deterministic command/rejection/checkpoint/projection corpus의 non-zero current execution과 exact identity
- event-heavy/64-session latency·encoded-size 및 direct heap/allocation의 repeatable raw samples
- all ten required Validation Dimensions에 current direct evidence
- production behavior delta 0; oracle-only production branch/fallback 0
- IVA-C2-01/09의 durable baseline seam과 IVA-C2-12 auxiliary observation locator가 재현 가능
- optimization 또는 performance PASS claim 0

## Stop Conditions

- `GAP-CRAR-P2G20-002` direct heap/allocation baseline seam을 current pre-change target에서 재현할 수 없음
- policy/acceptance 의미, Goal/Phase/node/dependency/mapping 또는 execution boundary 변경 필요
- upstream basis의 stale/contradiction
- existing dirty work와 selected/remaining Goal mutation을 안전하게 분리할 수 없음
- Core-only completion을 external consumer/schema 추정으로 채워야 함

## GEX-10 Closure

- Implementation delta는 standalone support tool의 direct heap/allocation 계측, manifest-receipt currentness
  verification과 durable Core-only receipt에 한정된다. Production runtime algorithm/data delta는 0이다.
- 57/57 feature inventory, 42-path locator content closure, ten Validation Dimensions와 10 legacy removal-target
  locators가 current manifest verifier에서 통과했다.
- Current `cargo test -p game_core`는 1,131 tests passed, failed/ignored 0이다.
- Release baseline과 replay는 deterministic output/event/encoded size/allocation count/deallocation count/
  reallocation count/retained delta identity가 exact 일치했다. P01은 optimization/performance PASS를 주장하지 않는다.
- `GAP-CRAR-P2G20-002`의 pre-change direct observation seam은 current receipt에서 닫혔다. Final comparison과
  blocking performance PASS는 P08/`WP-CRAR-C2-09` 소유로 남는다.
- Audit Target Record: `ATR-CRAR-C2-P01-GEX10-001`; repository revision, task-owned content hashes, 42-path
  locator composite, validation identities와 unrelated dirty fingerprint를
  `evidence/P01_AUDIT_TARGET.json`에 고정한다.
- GEX-10은 Phase 완료 verdict를 선반영하지 않는다.
