# P01 Preservation Baseline Experiments

## 2026-08-22 — GEX-00 Intake and Baseline

### Request and Routing

- Request: `P2G_60_HANDOFF를 인계 입력으로 사용해서 GEX-00을 수행해줘`
- Routing: explicit single GEX stage → `stage-run`.
- Root target: Handoff가 인계한 sole current Goal Set `GS-CRAR-P2G50-001`.
- Eligible selection: sequential Phase relation과 predecessor requirement를 대조해 predecessor가 없는
  `GOAL-CRAR-P01-PRESERVATION-BASELINE` 하나만 eligible함을 확인했다.
- Authorization: GEX-00 read-only intake와 execution document 초기화만 허용. GEX-10 mutation/test execution 미승인.

### Upstream Integrity Observation

Command/method: current Handoff가 선택한 5개 materialization에 `sha256sum`을 실행하고 Handoff registry와 비교.

| Materialization | Result |
| --- | --- |
| `P2G_30_FINAL_POLICY_FREEZE.ko.md` | `2a0f70f8…f96bfa3`, exact match |
| `P2G_40_PHASE_PLAN.ko.md` | `ba2f1b20…58c5cd81`, exact match |
| `P2G_40_PHASE_PLAN_VALIDATION_RECEIPT.json` | `06628506…aa4298a`, exact match |
| `P2G_50_GOAL_SET.ko.md` | `5c0c7272…cfa7f5c6`, exact match |
| `P2G_60_HANDOFF.ko.md` | `c13c4dce…8c38d0c0`, exact match |

P2G-20 `RSB-CRAR-P2G20-001` Full Reviewed Content Hash Registry의 exact 38개 path에 `sha256sum`을 실행해
table의 full hash와 비교했다.

- Matched: 38
- Changed: 0
- Missing: 0
- Simulator HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Unity HEAD: `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`

### Existing Execution History Gate

`docs/goals/current_runtime_architecture_remediation` 아래에는 P2G artifacts만 있었고 exact P01을 가리키는
`PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, `COMPLETION_AUDIT.md` 또는 다른 execution history는 없었다.
따라서 신규 execution directory를 선택했다.

### Repository and Dirty Baseline

Commands/methods:

- simulator: `git rev-parse --show-toplevel`, `git rev-parse HEAD`, `git branch --show-current`,
  `git status --short --untracked-files=all`
- Unity: 같은 revision/branch/status 명령을 `git -C '/mnt/f/unity projects/ark'`에서 실행
- status identity: 각 status output의 SHA-256

Observed pre-document result:

- simulator workspace: 862 entries — M 162, D 24, untracked 676; fingerprint
  `ef3fbce3698f49691b4b742f08f07fcb03934faf26daa69d22daea68cd506fd7`
- cwd-relative `core_enhanced`: 829 entries
- cwd-relative `game_server`: 17 entries — M 15, D 1, untracked 1
- Unity: 2,126 entries — M 104, D 28, untracked 1,992, other 2; fingerprint
  `131cc115c0d1c31d78837b13b2c49720ea7a59f8dc82d0707d6fcc79bcb1ff91`
- execution document 3개 생성 뒤 simulator status: 865 entries — M 162, D 24, untracked 679; fingerprint
  `f3687bbe5aa50aedeafc728cad5830bc32e6fb71839f918b852a9da66b619536`. Pre-document status와의 exact member delta는
  현재 P01 directory의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`뿐이다.

Broad production/test/fixture dirty overlap이 존재한다. P01은 production behavior delta를 금지하므로 current files를
read-only input으로 유지하고 신규 support seam을 우선하는 isolation disposition을 선택했다. 기존 dirty file을
써야 할 경우에는 GEX-10 pre-edit path hash와 hunk attribution이 mandatory다.

### Runner and Seam Readiness

Commands/methods and observed results:

- `cargo --version` → `cargo 1.91.1 (ea2d97820 2025-10-10)`
- `rustc --version` → `rustc 1.91.1 (ed61e7d7e 2025-11-07)`
- `cargo metadata --no-deps --format-version 1` → exit 0; `game_core`, `game_server` packages identified
- Unity file locator search →
  `/mnt/c/Program Files/Unity/Hub/Editor/6000.3.4f1/Editor/Unity.exe`
- repository search for stable manifest/event-heavy/64-session harness → current repository-owned implementation 없음;
  previous audits와 performance budget의 gap statements만 발견
- `/tmp/core_arch_perf` source/Cargo hashes → P2G-20 reviewed hashes와 exact match; audit-local supplemental seam만 가능

`IVA-CRAR-01`, `IVA-CRAR-09`는 P01/`WP-CRAR-01` owner와 required input이 식별된다. durable seam 자체는 아직
없으며 GEX-10 output이다. `IVA-CRAR-12`는 available하지만 required baseline 또는 performance PASS를 대신하지 않는다.

### Validation Execution

- Cargo tests: not run
- Benchmarks: not run
- Unity EditMode/PlayMode: not run
- Reason: GEX-00 stage는 implementation/validation을 수행하지 않는 intake pass이며 direct GEX-10 authorization이 없다.
- Prior success reuse: not authorized

### GEX-00 Result

- Result: `passed`
- Exact selected Goal: `(GS-CRAR-P2G50-001, GOAL-CRAR-P01-PRESERVATION-BASELINE)`
- Phase / execution node: `PHASE-CRAR-01-PRESERVATION-BASELINE` / `WP-CRAR-01`
- Production mutation: none
- Created artifacts: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md` in the canonical P01 execution directory
- Stop boundary: stage-run ends here; GEX-10 needs a new user request

## 2026-08-22 — Goal-Run GEX-10 and Core-Only Scope Correction

### Goal-Run Authorization and Safe Candidate Work

사용자는 이후 `HA-CRAR-P2G60-001`/`GS-CRAR-P2G50-001` 전체 goal-run을 승인했다. P01에서 다음
preservation-only candidate를 작성했다.

- Rulebook heading 57개를 stable ID와 route/test bundle에 연결하는
  `docs/architecture/runtime_feature_manifest.v1.json`
- manifest completeness/locator verifier와 Core-only performance harness
- Little Red/projectile/effect event-heavy 및 64-session/8-way workload receipt
- `docs/game_rulebook.md`의 preservation locator 한 행

Core current validation:

- `cargo test -p game_core`: non-zero 1,131 passed, failed 0, ignored 0
- event-heavy receipt: 5/5 repeatable, 186 events/sample, serialized bytes 65,720/sample
- 64-session/8-way receipt: 5/5 repeatable, 11,840 events/sample, serialized bytes 3,881,088/sample

초기 manifest locator와 benchmark fixture data가 current source 계약에 맞지 않아 각각 verifier failure와 loader
failure를 한 차례 관찰했다. locator를 actual Core path로 고치고 dummy의 required response fragment를 live valid
definition으로 지정한 뒤 current target에서 재검증했다. production behavior 분기나 fallback은 추가하지 않았다.

### Pre-Correction External Validation — Excluded

core-only 지시 전 기존 `001` completion 문구를 따라 game_server/Unity test를 실행했다. Unity live 4건은 stale
client가 Core action capability `ResetPlayerProfile`을 알지 못해 동일 실패했다. 이 observation은 current Core
결함이나 compatibility requirement가 아니며 acceptance, blocker, baseline 또는 완료 증거에서 전부 제외한다.
해당 실패에 맞춘 Core/server 수정은 없었다.

### User Scope Correction

- Exact direction: `유니티는 검사하지마, core 리팩토링에만 집중해.`
- Current execution scope: `core_enhanced` only.
- Excluded from all future reads and validation: Unity repository/source/docs/fixtures/MCP/revision/status/hash/tests,
  game_server remediation and external client compatibility.
- Candidate manifest의 Unity/game_server locator를 Core-owned route/test로 교체했다.
- verifier에 external Unity/game_server/absolute locator rejection을 추가했다.
- Unity direct-batch attempt가 남긴 task-owned `evidence/UNITY_EDITMODE.log`는 current evidence set에서 제거한다.

### Run-Global Stop Disposition

`FPF-...-001`, `PP-...-001`, `GS-...-001`, `HA-...-001`은 P07 server/Unity atomic cutover와 P08 full
Core/server/Unity validation을 필수화한다. core-only 지시는 policy scope, Phase/Atomic Group와 Goal mapping을
바꾸므로 P01 GEX-10을 완료하지 않았다.

- GEX-20: not started
- P02~P08: not started
- First required owner: `P2G-10`
- Existing `001` artifact bytes: preserved as historical; not edited in place

### Core-Only Document Validation

- `cargo fmt --manifest-path tools/runtime_architecture_preservation/Cargo.toml --check` → exit 0
- `cargo run --manifest-path tools/runtime_architecture_preservation/Cargo.toml --bin
  verify_feature_manifest -- docs/architecture/runtime_feature_manifest.v1.json` → exit 0
- verifier result: feature 57, Rulebook heading 57, evidence bundle 14, validation dimension 10,
  legacy target 10, unknown/unmapped/duplicate 0
- current manifest SHA-256: `c81f7ffcab5bc9eb47ed3e9cded0d64d8f26a14a743a27c6fab00a09e32d1321`
- external Unity/game_server/absolute locator search in manifest/support source: 0
