# P02 Validated Data Experiments

## 2026-08-22 — GEX-00 Intake

- Run mode: full Goal Set `goal-run`.
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P02-VALIDATED-DATA)` / `WP-CRAR-C2-04`.
- Existing exact-identity execution documents: none; canonical directory `execution/p02_validated_data` selected.
- Direct predecessor P01 latest audit: `Complete / High`, `current / unchanged`.
- Upstream strict sequence and dependency graph make P02 the sole eligible Goal.
- Production mutation/test execution in this GEX-00 pass: none.

### Current Core-Only Baseline

- Loader pipeline is already fallible and validates complete typed/domain catalogs before constructing the published
  database; invalid equipment-before-publication coverage exists.
- Structural bypass remains: public parts/builder/new and a public battle-held database Arc; no `ValidatedGameData`.
- Current documentation still names the bare loader and external startup consumer and must be aligned to the Core-only
  opaque publication boundary.
- `GameDataBuilder`: 28 Core source files, five integration/support files. Inspection shows the Core uses are fixture
  modules while the P01 release support tool is an explicit nonproduction consumer.
- Relevant paths are dirty but exact GEX-10 hunks can be isolated; pre-edit hashes are recorded in `PLAN.md`.
- N/A recheck: no hot reload/concurrent publication owner; no P02 session/save lifetime change.

### GEX-00 Result

- Result: `passed`; production mutation/test execution 0.
- No new policy, Phase/Goal mapping, external consumer or dirty-isolation blocker.
- Next authorized pass: separated GEX-10.

## 2026-08-22 — GEX-10 Implementation and Validation

### Opaque Publication and Capability Isolation

- Added `ValidatedGameData(Arc<GameDataBase>)` with the sole production live entrypoint
  `ValidatedGameData::load_live_embedded()`.
- `GameCore` constructors and public `BattleCore::new_from_scenario` now consume the opaque token; Core-internal battle
  setup retains a crate-private bare-database constructor.
- `BattleCore.game_data` changed from public to `pub(crate)`. `GameDataBaseParts` and `GameDataBase::new` are private.
- `GameDataBuilder`, `From<Arc<GameDataBase>>` and fixture Arc extraction are gated behind test/debug fixture builds or
  the explicit non-default `test-fixtures` feature. All four contract-fixture binaries declare that required feature.
- The P01 release support tool explicitly selects `test-fixtures` and derives its fixture from a freshly validated
  publication; it does not recreate live catalogs through an empty/default fallback.

### Failed Compile Loops and Corrections

- Initial release compile found the Event fixture binary attempted `Arc::get_mut` on the new opaque token. The four
  fixture binaries were moved behind `required-features = ["test-fixtures"]`; the Event fixture uses the explicitly
  gated owned-Arc conversion. Default release then compiled.
- Initial test-target compile found three test helpers still returning bare Arcs from the token. Those exact fixture
  helpers now use the gated conversion; the next `cargo test --no-run` passed.
- A first focused command used `--exact` with an unqualified test name and ran 0 tests. It was rejected as evidence and
  rerun without `--exact`; each intended test then ran 1/1.

### Direct Validation Results

- `cargo fmt --all` and later format checks: exit 0.
- `cargo check -p game_core --release`: exit 0, warnings 0 after keeping private validation assembly in the production
  loader and gating fixture-only map caches.
- `cargo test -p game_core --no-run`: exit 0; all current lib/integration targets compiled.
- `cargo check -p game_core --release --features test-fixtures --bins`: exit 0; the explicit fixture capability and all
  four gated binaries compile.
- Valid release probe: `cargo run --manifest-path tools/validated_data_api_audit/valid/Cargo.toml --release` exited 0
  after actual embedded publication and `GameCore::new` construction.
- Invalid default-release probe exited non-zero as required: `GameDataBuilder` unresolved (`E0432`) and
  `GameDataBaseParts` private (`E0603`). This is the expected PASS condition, not a build failure.
- `repeated_valid_and_invalid_publication_attempts_are_idempotent`: 1/1 pass; two full publications are semantically
  equal across every catalog, identical invalid manifest attempts return equal `LiveRonError`, and a later official
  publication remains equal.
- `live_publication_preserves_every_parsed_catalog_and_derived_item_registry`: 1/1 pass.
- `invalid_equipment_source_is_rejected_before_database_publication`: 1/1 pass.
- `cargo test -p game_core --test ron_loading`: 25/25 pass.
- `cargo test -p game_core`: lib 1,081 + integrations 4 + 4 + 25 + 14 + 4 = 1,132 passed; failed/ignored 0.
- P01 currentness restoration: rebound release workload 5+5 repeatable, support fmt/clippy/tests pass, manifest verifier
  features 57 / locator contents 42 / errors 0; fresh P01 re-audit is `Complete / High` on
  `ATR-CRAR-C2-P01-GEX20-002`.

### Quality/Baseline Disposition

- Whole-package release Clippy is not a P02 gate and currently exits non-zero on a pre-existing denied
  `never_loop` in `src/game/data/boss_runtime_data.rs:1680`, with 81 broad warnings across unrelated dirty code.
  The failure is outside P02-owned hunks; it was not modified or hidden. The dedicated P01 support workspace passes
  `clippy --all-targets -- -D warnings`, while P02's release compile, focused matrix and full tests pass.
- `ordering_concurrency` remains N/A: the official API is one startup publication and introduces no hot-reload owner.
- `disconnect_restart_persistence` remains N/A: P02 changes no session/save document or lifetime.
- No policy, Goal/Phase/node change, unchecked production dependency, external evidence requirement or unsafe dirty
  overlap was found.

### GEX-10 Result

- Assigned node implementation and every required P02 validation dimension have current direct evidence.
- Predecessor stale-on-successor-delta was resolved by the fresh P01 read-only re-audit; no correction was required.
- Next action in this active goal-run: freeze `ATR-CRAR-C2-P02-GEX10-001`, stop implementation mutation and begin a
  separated fresh GEX-20.

## 2026-08-23 — GEX-20 Attempt 1 and Correction GEX-10

### Read-only Audit Finding

- `ATR-CRAR-C2-P02-GEX10-001` currentness passed: 34/34 members and unrelated dirty fingerprint exact.
- Fresh release check, valid probe and three critical publication tests passed; builder/parts probe still failed as
  expected.
- Source-level review then found `CombatExecutor::build_battle_with_combat_preview` remained public and accepted bare
  `Arc<GameDataBase>`, returning a constructed `BattleCore`. This left production bare construction count at one.
- GEX-20 attempt 1 recorded `Not Complete` in `COMPLETION_AUDIT.md` and returned to an in-scope correction GEX-10.

### Correction

- The unconsumed public helper is now private and compiled only under `cfg(test)`. Its bare-Arc handoff and the
  preview-only `BattleStartPlan::from_preview` / scenario builder are likewise absent from release.
- The normal live setup path remains crate-private and receives the database only from the owning `GameCore` token;
  gameplay algorithms were not changed.
- The invalid release probe now checks four independent negative surfaces:
  `GameDataBuilder` (`E0432`), parts (`E0603`), `Arc<GameDataBase>: Into<ValidatedGameData>` (`E0277`) and removed public
  combat helper (`E0599`). Expected exit 101 with all four errors was observed.

### Correction Validation

- `cargo check -p game_core --release`: exit 0, warnings 0.
- `cargo test -p game_core --no-run`: exit 0.
- `cargo test -p game_core --lib game::events::combat::tests`: 14/14 pass.
- `cargo check -p game_core --release --features test-fixtures --bins`: exit 0.
- Valid actual publication→`GameCore` release probe: exit 0.
- Full `cargo test -p game_core`: 1,132 passed, failed/ignored 0.
- P01 manifest verifier remains exact after correction: locator contents 42, composite `ecaee174…eab`, receipt current,
  errors 0. `ATR-CRAR-C2-P01-GEX20-002` therefore remains `Complete / High`; the test-only correction is a proven
  non-impacting delta to its unrelated dirty fingerprint, not a changed P01 completion seam.

### Correction GEX-10 Result

- Public/bare production construction paths now have direct source and four-way compile-fail evidence of count 0.
- Policy meaning, Goal/Phase mapping and N/A dispositions remain unchanged; no external dependency or dirty overlap
  blocker appeared.
- Freeze replacement target `ATR-CRAR-C2-P02-GEX10-002`; next pass is a new separated GEX-20. Attempt 1 remains an
  immutable `Not Complete` record.
