# P02 Validated Data Execution Plan

상태: `GEX-20 attempt 1 Not Complete / correction GEX-10 exit conditions met / replacement target pending`

## Current Execution Identity

- Handoff / root target: `HA-CRAR-P2G60-002` / full `GS-CRAR-P2G50-002` goal-run
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P02-VALIDATED-DATA)`
- Phase / node: `PHASE-CRAR-C2-02-VALIDATED-DATA` / `WP-CRAR-C2-04`
- Current pass: correction `GEX-10 exit conditions met`; GEX-20 attempt 1 found and returned one in-scope defect
- Goal Quality Contract: `2`
- Authorization: Core-only selected-Goal mutation and test execution under the active goal-run. game_server/Unity and
  external consumer/admin compatibility are excluded.

## Execution Document Set

- Directory: `docs/goals/current_runtime_architecture_remediation/execution/p02_validated_data`
- Plan / Experiments / Notes / Audit:
  `PLAN.md` / `EXPERIMENTS.md` / `EXPERIMENT_NOTES.md` / `COMPLETION_AUDIT.md` under that directory
- Existing exact-identity execution history: none; this is the sole canonical `002` P02 set.

## Exact Basis and Direct Dependency

- Freeze / Plan / Receipt / Goal Set:
  `FPF-CRAR-P2G30-002` / `PP-CRAR-P2G40-002` / `PVR-CRAR-P2G40-002` /
  `GS-CRAR-P2G50-002`
- Direct predecessor: P01 `Complete / High` on `ATR-CRAR-C2-P01-GEX10-001`, currentness
  `current / unchanged` at its GEX-20 exit.
- Applicable integration seam: `IVA-CRAR-C2-04`.

## Objective and Boundary

- Objective: authoring input → recoverable validation → opaque `ValidatedGameData` publication → Core construction을
  유일 production path로 만들고 fixture/direct mutation capability를 production에서 격리한다.
- In scope: Core loader/validation/publication/factory, Rust visibility/feature gating, typed Core admin command,
  `docs/data_loading_contract.md`, actual Core constructors/call sites/live RON tests.
- Out of scope: gameplay balance/data content meaning, server admin API, transaction/lifecycle/checkpoint/protocol,
  game_server/Unity/external consumers.
- Existing dirty work remains user-owned. Overlapping production hunk는 pre-edit identity와 exact attribution이
  가능한 경우에만 수정한다.

## Completion Contract

- actual valid RON loads with identical semantics through one opaque publication path
- invalid malformed/reference/range input returns recoverable pre-authority failure with partial publication 0
- repeated identical valid/invalid load has stable identity and authority delta 0
- production bare database/builder/direct setter/fixture/default fallback paths 0
- fixture/test/admin capability is production-inaccessible; operating mutation uses typed Core command
- current loader → publication → actual Core construction producer-consumer closure
- `success`, `invalid_input`, `failure_rollback`, `retry_idempotency`, `boundary_empty_max`, `live_data`,
  `producer_consumer`, `legacy_fallback_absence` direct evidence
- `ordering_concurrency` and `disconnect_restart_persistence` N/A must remain valid in current actual
- canonical `docs/data_loading_contract.md` matches actual boundary

## Stop Conditions

- unchecked production dependency requires a bypass/compatibility API rather than the approved opaque boundary
- hot reload/concurrent publication or save/session lifetime becomes applicable, invalidating current N/A rows
- policy/Goal/Phase/node/dependency meaning must change
- exact predecessor audit becomes stale without an allowed expected-delta re-audit path
- dirty overlap cannot be isolated safely

## Current Baseline

- Current live path already performs `parse_gameplay_ron_sources → into_validated →
  from_validated_live_catalogs → generated-preview validation` and returns recoverable `LiveRonError` before publish.
- Missing opaque boundary: `ValidatedGameData` type does not exist. `GameDataBase::load_live_embedded()` returns
  `Arc<GameDataBase>` directly.
- Current bypass/API surface: public `GameDataBaseParts`, public `GameDataBuilder` with public setters/builders,
  public `GameDataBase::new(parts)`, and public `BattleCore.game_data: Arc<GameDataBase>`.
- `GameDataBuilder` appears in 28 Core source files (predominantly `#[cfg(test)]` modules) and five integration/support
  files. Production live construction call sites use the embedded loader.
- Approved implementation: `ValidatedGameData` owns the opaque publication token; production Core constructors consume
  it. Bare parts/builder/new are gated to `test | debug_assertions | non-default test-fixtures`, with explicit release
  compile-fail API evidence. The P01 release audit tool opts into the test capability explicitly.
- Startup publication has no hot reload owner; `ordering_concurrency` remains N/A. P02 changes no session/save lifetime;
  `disconnect_restart_persistence` remains N/A.
- Pre-edit identities: `Cargo.toml b7fabaf4…5bab`, `data/mod.rs 24ccad25…4f66`,
  `live_ron.rs 7e6fb60f…0e5`, `world.rs f293592e…e134`, `battle/core/mod.rs d0163df2…e77`,
  `events/combat.rs 7f808b8a…95ae`, data contract `58c636ce…c7b`.
- All relevant production/document/test paths are pre-existing dirty. The change is limited to exact publication,
  visibility/gating, constructor and documentation hunks; broad unrelated bytes remain preserved.
- P01 currentness at intake: `Complete / High`, `current / unchanged`; P02 has not yet changed its manifest/oracle.

## Implemented Current Boundary

- `ValidatedGameData` is the public opaque publication token. The complete live candidate is parsed, semantically
  validated, privately assembled, generated-preview validated and only then wrapped.
- Production `GameCore` and public `BattleCore` construction consume `impl Into<ValidatedGameData>`. The only bare
  database battle constructor and held database field are Core-private.
- `GameDataBuilder` and fixture-owned Arc conversion are absent from default release unless the explicit non-default
  `test-fixtures` capability is selected; bare parts/new remain private. Four fixture binaries require that feature.
- Independent release probes prove the official loader→publication→Core route compiles/runs and default-release imports
  of `GameDataBuilder`/`GameDataBaseParts` fail with `E0432`/`E0603`.
- The canonical data loading contract now names the opaque Core boundary, 24-source inventory and exact fixture gate;
  no external startup/consumer claim remains.
- Fresh valid/invalid retry coverage proves full publication semantic identity, identical recoverable error and no
  failed-candidate effect on a later official publication.
- P02 changed the P01 support consumer and receipt source hash. Fresh read-only P01 re-audit
  `ATR-CRAR-C2-P01-GEX20-002` restored predecessor currentness as `Complete / High`.
- GEX-20 attempt 1 found a public `CombatExecutor` helper that still accepted bare `Arc<GameDataBase>`. Correction
  removed that default-release method entirely, made its preview-only helpers test-only, and extended the invalid
  release probe to require `E0277` for bare Arc conversion plus `E0599` for the removed helper.
