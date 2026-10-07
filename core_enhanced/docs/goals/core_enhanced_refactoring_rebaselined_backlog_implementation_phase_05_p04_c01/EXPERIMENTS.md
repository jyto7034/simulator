# Phase 05 CRA-P04-C01 Experiments

Status: complete
Implementation Complete: true

## 2026-08-09 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 04 CRA-P03-C02 has a current `Complete / High` audit with improvement class `none`.
- Phase 05 is next in the exact registry; Phases 06~44 remain pending and unauthorized.
- CRA-POL-006 is the direct and global compatibility policy for this candidate.

### Evidence Carried Into Initialization

- CRA-P04-C01 remains a correctness correction with priority `1/2/1/1/2/2/unknown = 9/18`.
- Historical defect: typed publication permits repeat policies whose required battle/node identity is unavailable for
  the trigger source; runtime alone rejects the unusable data. Confirmed minimum example is
  `MapNodeEntered + OncePerBattle`.
- Required first proof: exact structured negatives for every current runtime-rejected source/repeat pair through
  typed and official-bundle publication, with the live bundle remaining valid.
- Preservation boundary: currently executable pairs (including accepted battle `OncePerNode`), trigger/runtime
  history and ordering, RON/checkpoint/DTO/wire/public shape, live content and adjacent VN lifecycle work.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 05 coordination changes from `ready` to `active`; no other Child is active.
- Phase 05 production status/source/data investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents completely, capture expanded dirty status and prospective target hashes, then map the
current trigger publication/official loader/runtime key matrix and establish a nonzero VN baseline.

## 2026-08-09 — E01 Current Authority, Matrix, Dirty Baseline And Focused Baseline

### Current Authority And Exact Matrix

- `data/visual_novel_trigger_data.rs::validate_trigger` owns typed local validation but checks only IDs and
  source-local fields. It does not inspect `repeat` against `source`.
- `world/visual_novel_triggers.rs::consumed_key_for_trigger` is the first current compatibility enforcement:
  `OncePerBattle` extracts a UUID only from `BattleStarted`/`BattleTimeReached`; `OncePerNode` extracts a node from
  `MapNodeEntered` or deliberately maps either battle UUID to a node scope.
- The exact 4 × 4 matrix therefore has three rejected pairs:
  `RunStart + OncePerBattle`, `RunStart + OncePerNode`, and `MapNodeEntered + OncePerBattle`. The other 13 pairs are
  runtime-keyable, including both battle sources with `OncePerNode`.
- `ParsedGameplayRon::validate_domain_catalogs` routes the official embedded source set through the same typed
  `validate_contract`, maps its `DataValidationError` to the trigger source/domain, then applies the narrower live
  RunStart policy. Runtime defensive key derivation must remain.

### Live Data And Canonical

- Production `visual_novel_triggers/base.ron` is clean at hash
  `4216706ac783ee5d11662c616360cd0054564c3297db6953245d61ca2c038875`; its two rows are valid blocking
  `RunStart + OncePerRun + PersistAcrossCheckpointLoad` triggers.
- Production Visual Novel data is clean at hash
  `0c37bba3a0659fa1960e44a4e8f7dca9c92399eb9f1c9fda9b588bae6338b55f`.
- `core_runtime_contract.md` is already dirty for broad unrelated runtime work at hash
  `9a654ddee0903d58a2f0086d49c22c163dfd22daacac9b02622bed7ba583cb18`; its VN trigger section is the nearest
  canonical and must receive only a separable publication-matrix sentence after implementation.

### Dirty Baseline And Overlap

- Commit remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded post-initialization status: 394 entries; SHA-256
  `982d24af9381ec28e7a6f9f06430017bf74ff0d3881d2b28c12ed57d470672ca`.
- Clean prospective authority/runtime files and hashes:
  - trigger data `f622f5c039cb3f83c15fc5ba038df97671596e1e49605fc414f64f14c5648798`;
  - runtime triggers `52a24ae63cb7d9d43182d3f2c294d35ac438375ad363845e3c60a2552c2c7819`.
- Official bundle validator/test owner `data/live_ron.rs` is pre-existing dirty, 384+/75-, hash
  `a0402c6e7a188817223910309f4aff9bfba4a41a0f292fce5fb1daeb0b16f03c`, for Boss runtime compilation and a
  macro-generated exact embedded-source inventory. The existing typed VN bundle tests are a separable insertion
  point; do not alter those user hunks.
- `tests/ron_loading.rs` is pre-existing dirty, 444+/7-, hash
  `75558ff28be83cafcfe535252f552903b2077249c2cd81ef00b41d63195854b3`, for Boss/tactical weapon/field-work
  live assertions. It need not be edited because official bundle mutation tests belong beside loader validation.
- `world/visual_novel_flow.rs`, `data/mod.rs` and `data/validation.rs` have unrelated user changes and require no
  Phase mutation. Their prehashes are recorded in command evidence; runtime source remains read-only.

### Baseline Commands And Results

```text
cargo test -p game_core game::data::visual_novel_trigger_data::tests --lib -q
cargo test -p game_core bundle_validation_returns_typed_visual_novel_trigger --lib -q
cargo test -p game_core live_game_data_loads_run_start_visual_novel_triggers --lib -q
```

- Typed trigger data passed 2/2 with 935 filtered.
- Existing official-bundle typed trigger/error-reference cases passed 2/2 with 935 filtered.
- Live embedded RunStart trigger load passed 1/1 with 936 filtered.
- These protect local/reference/live success, but none exercises repeat/source compatibility.

### Next Experiment

Add a complete three-pair typed error table, all 13 accepted typed pairs and a three-pair official embedded-bundle
provenance table. Run them before production validation mutation to capture the fail-open behavior.

## 2026-08-09 — E02 Test-First Compatibility Matrix

### Added Proofs

- One typed table records all three rejected pairs and requires exact trigger ID, `triggers[0].repeat` path and
  pair-specific cause.
- One positive cross-product proves all 13 runtime-keyable pairs remain accepted, including both battle sources with
  `OncePerNode`.
- One official embedded-bundle table reparses the real source set per case, mutates only its parsed trigger database
  and requires exact source file/domain/object/path/cause for all three pairs.

### Pre-Implementation Results

```text
cargo test -p game_core runtime_rejected_repeat_source_pairs_fail_typed_publication --lib -- --nocapture
cargo test -p game_core every_runtime_keyable_repeat_source_pair_remains_typed_publication_valid --lib -- --nocapture
cargo test -p game_core bundle_validation_rejects_every_runtime_invalid_trigger_pair_with_exact_provenance --lib -- --nocapture
```

- Typed negative failed 0/1 with 939 filtered and reported `[None, None, None]`: all three runtime-invalid pairs
  publish successfully through the typed database.
- Accepted matrix passed 1/1 with 939 filtered and asserted exactly 13 accepted pairs.
- Official-bundle negative failed 0/1 with 939 filtered: both invalid RunStart cases reached the later generic
  `source.RunStart.policy` rejection, while `MapNodeEntered + OncePerBattle` returned no error. None produced the
  required typed compatibility provenance.
- Initial fixture compile correction changed only the new test from nonexistent `Normal/Elimination` variants to
  the current `Defense/Defense` pair. It occurred before behavior evidence and is not counted as a product result.

### Implementation Decision

Extend only `validate_trigger` with the runtime-derived repeat/source matrix after source-local validation. Use the
existing `trigger_error` and `triggers[index].repeat` path with exact stable causes. Do not change enums, RON,
runtime key extraction, live RunStart policy or accepted matrix.

## 2026-08-09 — E03 Publication Correction, Canonical And Broad Validation

### Implementation

- `validate_trigger` now rejects `OncePerBattle` for RunStart/MapNodeEntered and `OncePerNode` for RunStart after
  existing source-local validation. It uses the existing trigger error type, object ID and indexed `.repeat` path.
- No enum, schema, RON, runtime matcher/key derivation, consumed history, queue/session or live RunStart policy
  changed. The runtime continues to reject impossible bypassed state defensively.
- The runtime canonical now records the exact three rejected pairs, accepted battle `OncePerNode` UUID mapping and
  defensive runtime boundary.

### Focused And Broad Results

- New typed negative passed 1/1; accepted matrix passed 1/1 and asserted 13; official-bundle negative passed 1/1.
- Trigger-data module passed 4/4; official live-RON validation module passed 62/62; VN family passed 26/26;
  facility/VN runtime passed 11/11; production RON loading passed 25/25.
- Full Core package passed 989/989 physical tests across nonzero binaries
  (`940 + 3 + 4 + 25 + 13 + 4`), 0 failed/ignored.
- Core and server all-target checks, formatting and `git diff --check` pass.

### Dirty And Compatibility Result

- Expanded status is 395 entries with SHA-256
  `6c8f7f8480ae2915a703db42c72d97362a8cdb6e05552bf8b594465d893cbffc`; the one-entry increase from activation
  is the formerly clean typed trigger-data file.
- `live_ron.rs` moved from 384+/75- to 468+/75- solely by the adjacent 84-line bundle matrix; its existing Boss and
  source-inventory hunks remain. `ron_loading.rs` hash remains unchanged.
- Both live VN RON hashes and the read-only runtime trigger hash remain unchanged.

### Next Experiment

Reread the required completion-review guide, audit every matrix/compatibility/consumer/dirty condition against the
current tree and issue the Phase verdict. Do not activate Phase 06 unless it is `Complete / High`.

## 2026-08-09 — E04 Phase Exit

- Required completion review independently checked the typed/official/runtime/data/canonical/consumer paths, all
  matrix rows, compatibility, scoped diff and current validation.
- Verdict: `Complete / High`; improvement class `none`; no material finding or open policy question.
- Phase 05 is complete and may release CRA-P05-C01 to readiness. Phase 06 production remains gated by its own three
  required documents and full read.
