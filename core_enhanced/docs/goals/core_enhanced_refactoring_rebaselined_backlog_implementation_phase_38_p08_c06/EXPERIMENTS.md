# Phase 38 CRA-P08-C06 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E00 Goal Initialization

### Hypothesis

The disconnected public death model can be investigated as one cleanup boundary only after the real production
death assertions and public-consumer gate are made explicit. Initializing the required Child documents before any
current Phase-38 production inspection preserves the Master one-active-Phase and dirty-worktree contracts.

### Performed checks

- Read the resumed-execution attachment completely (198 lines).
- Read all three Master documents completely; Master is active, Phase 37 is `complete — Complete / High`, Phase 38
  is `ready`, and Phases 39~44 are pending.
- Read the original and post-Boss P08-C06 candidate/test/boundary evidence and the Final catalog, exact-order,
  test-rebaseline, wave/gate and CRA-POL-006 records relevant to this candidate.
- Read the complete Phase 11 P08-C01, Phase 12 P08-C02, Phase 28 P08-C05 and Phase 37 P06-C04 completion audits.
- Verified the reserved Child path and Phase-39 path were absent and that no implementation Child was active.
- Recorded HEAD, expanded status count/hash and protected-marker identity before creating this Child.
- No Phase-38 current production content, caller, test, data, canonical or external-consumer detail was inspected.

### Exact commands

```text
wc -l -c /home/acr0tic/.codex/attachments/2a19e75e-a8d8-49cc-82d0-ee216a4f3dfd/pasted-text-1.txt
sed -n '1,220p' /home/acr0tic/.codex/attachments/2a19e75e-a8d8-49cc-82d0-ee216a4f3dfd/pasted-text-1.txt
wc -l <Master PLAN/EXPERIMENTS/EXPERIMENT_NOTES>
sed -n <bounded complete reads of all three Master documents>
sed/rg <candidate-scoped original, post-Boss and Final evidence>
sed -n <complete Phase 11/12/28/37 completion audits>
git rev-parse HEAD
test ! -e <Phase-38 Child path>
test ! -e <Phase-39 Child path>
rg -l '^Status: active$' <implementation phase PLAN glob> | wc -l
git status --porcelain=v1 --untracked-files=all | wc -l
git status --porcelain=v1 --untracked-files=all | LC_ALL=C sort | sha256sum
stat -c '%n %s %y %z' game_server/logs/app.log.2026-08-09
sha256sum game_server/logs/app.log.2026-08-09
```

### Actual result

- Repository/HEAD: `/mnt/f/work/simulator` /
  `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Direct gates: Phase 37 linear predecessor and P08-C01/C02/C05 technical predecessors are all independently
  `Complete / High`.
- Candidate: CRA-P08-C06, cleanup, Tier D, `3/1/2/1/2/1/unknown = 10`.
- First proof: production death must own unique sorted/deduped ally occurrence before legacy removal.
- Public gate: CRA-POL-006 requires affirmative inventory/migration or candidate-scoped breaking approval; the
  Phase-37 approval is not generalized.
- Pre-Child status: 568 rows at canonical C-sort SHA-256
  `8e45c51357bed53aecdd7748743e298af9f53c107fcb52ce30a4bb936f99a5c9`.
- Active implementation Child count: 0. Phase 38/39 paths: absent.
- Protected marker: 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, timestamps unchanged.
- Production/data/test/canonical mutation and test execution: none.

### Decision

Initialize only the three required Child documents. Register and activate Phase 38 before any current production
content, caller, test or consumer inspection. Phase 39 remains pending and absent.

## 2026-08-11 — E01 Sole Activation

### Gate check

- The initialized Child contains exactly the required `PLAN.md`, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md`.
- Adding those three files changes the expanded status set exactly from 568 to 571 rows at canonical C-sort
  SHA-256 `68785da9ec0b529b278f26f052882f771309fe4443f87e909de3312283c799a4`.
- The protected marker remains exact at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`.
- Phase 37 is complete, Phase 38 has all dependency gates satisfied, Phase 39 remains pending and absent, and there
  was no active implementation Child before this transition.

### Result

- All three Child headers changed from `initialized` to `active`; `Implementation Complete` remains `false`.
- Master Phase 38 changed from `ready` to `active`. It is the sole active Child.
- Phase 39 and all later Phases remain pending and unauthorized for production inspection.
- Current Phase-38 production content/callers/tests/consumers, production mutation and test execution: none.

### Next gate

Fully reread all three active Child documents and direct completion reviews, then record filename-only prospective
fingerprints before opening current production content.

## 2026-08-11 — E02 Active Reread And Prospective Fingerprints

### Procedure

- Fully reread all three active Child documents.
- Fully reread the current Master PLAN, Phase-37 linear completion audit and Phase-11/12/28 technical dependency
  completion audits after activation.
- Reread the candidate-scoped original/post-Boss and Final catalog/order/test/gate/policy sections.
- Used filename-only discovery to locate prospective death, occurrence, command, world, manifest, data and canonical
  paths. Current production content and symbol bodies remained unopened.
- Recorded scoped status, line counts and SHA-256 fingerprints before Phase-38 current discovery.

### Activation state

- Expanded status remains 571 rows at canonical C-sort SHA-256
  `68785da9ec0b529b278f26f052882f771309fe4443f87e909de3312283c799a4`.
- `core_enhanced/src/game/battle/death.rs` is clean, 244 lines, SHA-256
  `f36b55cbea0c9d895fee134556539517d6e08f224ab175ace48e490907269647`.
- `battle/mod.rs` is already dirty from Phase 37, 15 lines, SHA-256
  `2ced98c31319969aa964cd5de95970c49798b9438a4520365420436dd083f6e7`.
- `core/death_runtime.rs` is already dirty, 210 lines, SHA-256
  `096a68265a37d6af46cf7c3eaaae10bdb4a632c4323ee17f4b0062f2384fc6d9`.
- `core/mod.rs` is already dirty, 8,523 lines, SHA-256
  `822bf01e7b0db9638ff64eff54bf7e8bf1ea61118906d820080897c3cc09cd2f`.
- `core/triggers.rs`, `core/commands.rs` and `core/damage_runtime.rs` are already dirty at respectively
  `567bc636...`, `e97e30ed...` and `7b4d4379...`; they are dependency authorities, not presumptive Phase edits.
- `world/tests/mod.rs` is already dirty at `c8d52c36...` and is a prospective world-result test consumer only.
- Root/Core/server manifests, lockfile and Core library root are clean at `98c9cf4d...`, `40d8a9ec...`,
  `19b41cd1...`, `4223318b...` and `3fbfdca2...`.
- Equipment/Skill/Buff live data retain dependency-audit hashes `322d7455...`, `4087df05...`, `e7cc335f...`.
- `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md` retain hashes `6c53ecd5...`,
  `ab7d2f98...`, `30ed5fd5...`; they are already dirty from earlier campaign work.
- Protected marker remains exact at `614d2fbd...`.

### Decision

The clean disconnected `death.rs` can be isolated from earlier work. Any assertion transfer into the already-dirty
production test carrier must be appended as a narrow, independently attributable hunk while preserving its
activation fingerprint context. Current content/caller/test/consumer discovery may now begin.

## 2026-08-11 — E03 Current Authority, Assertion And Consumer Inventory

### Current legacy graph

- `battle/death.rs` is one 244-line tracked clean file. It publicly defines `DeadUnit`, `DeathProcessResult` and
  `DeathHandler`, exposes the pending queue and methods, removes processed units from a parallel result model, and
  handles only Modifier subsets.
- `battle/mod.rs` publicly exports that module; `lib.rs` and `game/mod.rs` make the path reachable as
  `game_core::game::battle::death`.
- Its only current code consumer is its colocated test
  `death_handler_deduplicates_deaths_and_emits_effects_in_sorted_ally_order`.
- Complete workspace and accessible `/mnt/f/work` Rust/TOML/source searches find no other constructor, type or module
  consumer. Historical/Goal references are evidence only.

### Current production authority

- `core/death_runtime.rs::finalize_unit_death` retains units as Dead, settles/interruption state, clears buffs,
  snapshots graveyard state, records one UnitDied per valid handoff, dispatches death occurrences under the death
  event cause, reconciles field work and schedules movement reevaluation.
- `death_trigger_commands` dispatches OnDeath, then causal OnKill, then every distinct active same-owner unit from
  the runtime unit map after UUID-byte sorting. The map key makes the ally set unique; inactive/dead units are
  excluded.
- The existing production test
  `death_item_occurrences_execute_dead_caster_and_sorted_ally_plans` enters through real `ApplyDamage`, proves dead-
  caster OnDeath, OnKill and two UUID-sorted OnAllyDeath effect/activation plans, and compares exact cast and
  StatChanged target vectors. Any repeated ally occurrence changes those exact vectors and fails.
- `committed_damage_uses_actual_overkill_delta_and_finalizes_one_causal_death`, lethal MaxHealth and buff cleanup
  controls already prove one causal production finalization, graveyard/event lineage and cleanup seams.

### Legacy assertion disposition

| Legacy assertion | Production disposition |
| --- | --- |
| duplicate enqueue does not double-remove | no production queue/removal model exists; strengthen the real damage handoff with duplicate lethal inputs and assert one summary/death/occurrence set |
| reverse ally input becomes UUID sorted | already directly stronger in the production death test's exact effect/activation vectors |
| killer Modifier command exists | already stronger as executed OnKill effect plus activation and final state |
| ally Modifier commands exist once each | already stronger as executed state/events plus exact unique vectors |

### Consumer and compatibility inventory

- Cargo workspace inverse dependency: only in-workspace `game_server` depends on `game_core`; it has no death-model
  reference.
- `game_core` manifest does not set `publish = false`; the public module is reachable through public crate modules.
- Git origin is the public-looking `https://github.com/jyto7034/simulator` URL.
- Accessible workspace and `/mnt/f/work` searches find no standalone checkout or tool consumer, but cannot prove the
  absence of downloaded/published/deployed Rust revisions.
- Consumer class is therefore `workspace_confirmed: none outside self-test`, `unknown_deployment: unresolved`.
  CRA-POL-006 does not authorize public removal from that negative evidence.

### Non-mutating baseline results

- Legacy exact test: 1 passed, 0 failed, 0 ignored, 1,016 filtered.
- `cargo test -p game_core --lib death -- --nocapture`: 16 passed, 0 failed, 0 ignored, 1,001 filtered. This includes
  the one legacy test and 15 real death-related controls.
- Exact production sorted occurrence test: 1 passed, 0 failed, 0 ignored, 1,016 filtered.
- Complete commands module: 43 passed, 0 failed, 0 ignored, 974 filtered.
- Baseline failures and zero-selected filters: none.

### Decision

Assertion transfer may proceed independently of the external gate. Strengthen the existing real production death
test with two same-time lethal inputs and exact one-kill/one-UnitDied/one sorted unique occurrence expectations, then
rerun it and the affected family. Do not delete `death.rs` or its export unless candidate-scoped breaking approval
or a supported migration is recorded.

## 2026-08-11 — E04 Production Assertion Transfer

### Change

Strengthened only the existing production test
`death_item_occurrences_execute_dead_caster_and_sorted_ally_plans` in the already-dirty `core/commands.rs`:

- submit two same-time lethal `ApplyDamage` commands for the same active victim;
- assert exactly one actual-damage target and one killed target in the command summary;
- assert the retained runtime unit is Dead, the graveyard contains it and exactly one `UnitDied` event exists;
- retain the preexisting exact final state and ordered event vectors proving one OnDeath, one OnKill and one
  OnAllyDeath effect+activation plan for each UUID-sorted unique active ally.

No runtime, legacy module/export, data, manifest, canonical, server or Unity source changed. This translates the
legacy duplicate-enqueue intent to the actual committed-damage/death authority instead of preserving the queue or
unit-removal model.

### Results

- Exact strengthened production test: 1 passed, 0 failed, 0 ignored, 1,016 filtered.
- Complete commands module: 43 passed, 0 failed, 0 ignored, 974 filtered.
- Death-filter family: 16 passed, 0 failed, 0 ignored, 1,001 filtered; it still includes the one legacy test pending
  the public gate.
- No baseline or introduced failure; parallel runs only reported transient Cargo file-lock waits.
- `commands.rs` post-transfer SHA-256 is
  `78ef19fe5119060aa350885825eb76403a9be481707b9d067e4071cbe678eb7a`.
- `death.rs`, `battle/mod.rs` and `core/death_runtime.rs` retain activation hashes. Expanded status remains the exact
  571-path activation set/hash because `commands.rs` was already dirty.
- Protected marker remains exact at `614d2fbd...`.

### Decision

The assertion-transfer gate is complete on the real production authority. Public cleanup remains unmodified and
blocked solely by the candidate-scoped CRA-POL-006 consumer/approval gate.

## 2026-08-11 — E05 Public-Compatibility Policy Gate

### Final policy-independent checks

- `cargo fmt --all -- --check`: exit 0.
- Whole-worktree `git diff --check`: exit 0.
- Expanded status remains the exact 571-path activation set at canonical C-sort SHA-256
  `68785da9ec0b529b278f26f052882f771309fe4443f87e909de3312283c799a4`.
- Protected marker remains 57 bytes at exact SHA-256 `614d2fbd...` with unchanged mtime/ctime.
- Legacy file/export hashes and runtime/data/canonical fingerprints remain unchanged; only the attributable
  production-test hunk changed inside the already-dirty commands carrier.

### Gate result

Workspace and accessible-directory evidence proves no current local consumer, but cannot exclude downloaded or
deployed Rust code using this reachable public API. Because the crate is not `publish = false` and the repository
has a public GitHub origin, the consumer remains `unknown_deployment`. CRA-POL-006 requires a deprecation/versioned
migration or explicit candidate-scoped breaking approval before removal.

### Status transition

- Child headers: `active` → `policy_decision_required`; `Implementation Complete` remains `false`.
- Master Phase 38: `active` → `policy_decision_required — assertion transfer complete; public removal approval
  required`.
- Phase 39 and later remain pending and absent/unauthorized.
- No legacy file/export removal, runtime/data/canonical change or broad post-cleanup claim occurred.

### Required decision

Recommended: if no external consumer exists, explicitly approve breaking removal of the exact public
`battle::death` module/types/export/self-test. Otherwise identify the consumer and approved deprecation/versioned
migration lifetime. No automatic continuation is allowed until one of those authorities is recorded.

## 2026-08-11 — E06 Breaking Approval And Resume

### Authority

- Master PLAN row 235 now ends with the user's explicit answer `사용자답변: 승인` on the Phase-38 public-removal
  gate.
- The approved break is exactly the disconnected public
  `battle::death::{DeadUnit, DeathProcessResult, DeathHandler}` surface, its module export and colocated self-test.
- Production death/damage/event carriers, runtime behavior, data/DTO/save/wire meaning and Phase 39 remain outside
  the approval.

### Resume checks

- Reread the 198-line execution attachment and all three Phase-38 Child documents before resuming.
- Reconfirmed Phase 11/12/28 and Phase 37 are complete, the Master registry retains Phase 39 pending, and the
  assertion-transfer target remains at SHA-256 `78ef19fe...`.
- Reconfirmed the legacy file/export/production authority hashes and the protected marker remain exact.
- `battle/mod.rs` has one preexisting Phase-37 hunk removing only `pub mod validation;`; the Phase-38 export removal
  can be applied as a distinct adjacent line deletion.

### Decision

Resolve CRA-POL-006 for this candidate, return Phase 38 and all three Child headers to `active`, and apply only the
approved cleanup. Phase 39 remains pending until validation and an independent `Complete / High` exit audit.

## 2026-08-11 — E07 Authorized Cleanup And Validation

### Exact production delta

- Deleted only the approved clean 244-line `core_enhanced/src/game/battle/death.rs`, which contained the three
  disconnected public types and their sole self-test.
- Removed only `pub mod death;` from `battle/mod.rs`; its preexisting Phase-37 `pub mod validation;` deletion remains
  intact and separately attributable.
- Added no shim, alias, private replacement, fallback or parallel death model. Current Core/server source and
  manifest search has zero matches for all three types, the old module/export and self-test name.
- The real production death runtime remains SHA-256 `096a6826...`; the transferred assertion carrier remains
  `78ef19fe...`. Final `battle/mod.rs` is `6100a999...`.

### Focused behavior

- Initial shortened exact-filter command selected 0 tests (`0/0/0`, 1,016 filtered) because Rust `--exact`
  requires the full module path. This is a command-selection error, not behavioral evidence.
- Corrected full-path exact production death assertion: 1 passed, 0 failed, 0 ignored, 1,015 filtered.
- Commands module: 43/0/0, 973 filtered. Death filter: 15/0/0, 1,001 filtered; the sole legacy test is the exact
  expected removal from the prior 16-test family.
- Item, Skill and World families: respectively 31/0/0, 106/0/0 and 245/0/0.

### Broad and static behavior

- Core all-target debug and release each pass 1,067/0/0: library 1,016 plus integrations 4 + 4 + 25 + 14 + 4;
  six executable/fixture targets select zero tests and are not behavioral evidence.
- Current sorted full discovery has 1,067 rows at SHA-256
  `7c77e183a4c54e2a5f51d0bdfe35f80adae53128d006728fa55577646a760eac`. Adding only the removed full legacy-test row
  reconstructs the Phase-37 1,068-row hash exactly at
  `71ff93259eb70ba0948bd28a0e783416fc55ee1927b821ae49a77472a4086a1d`.
- Workspace all-target debug/release checks compile both `game_core` and `game_server`; format and whole-worktree
  diff checks exit 0.
- Normal Core all-target Clippy exits 0 with the inherited 82 library / 134 library-test warnings plus existing
  integration warnings; no removed-surface diagnostic appears.
- Live Equipment/Skill/Buff RON, root/Core/server manifests, lockfile, Core lib root and three canonical documents
  retain their prospective hashes. The selected production proof constructs its occurrence inputs and does not
  consume live RON, so the predeclared live-RON rerun condition is false.

### Dirty and boundary result

- Expanded status is 572 rows at canonical C-sort SHA-256
  `623dc1d8d4da946dd2dafff20eec53622c00c507ac1dfe0806addc4fa844586d`; the sole new path relative to activation is
  the approved tracked deletion of `death.rs`.
- Protected marker remains 57 bytes at exact SHA-256 `614d2fbd...` and exact timestamps.
- Runtime/data/DTO/save/wire/canonical/server/Unity and Phase-39 production changes: none.

### Decision

Implementation and required validation are complete. Keep Phase 38 active and Phase 39 pending while a separate
completion audit tests every condition from current evidence.

## 2026-08-11 — E08 Completion Audit And Phase 39 Readiness

- The separate 212-line `COMPLETION_AUDIT.md` at SHA-256
  `b1063d8b6cc03c595e59f48f5f81e91a81d92e4860bd0bb6a129fc8cdb8e082b` independently returns
  `Complete / High`, fit `High`, improvement class `none`, with all twelve rows passing and no material finding.
- The audit reran current Core debug all-targets at 1,067/0/0 and reverified the deleted path, zero legacy search,
  exact assertion body, sole-active boundary, dirty/status hashes and protected marker.
- The audit artifact alone explains 572→573 expanded rows at canonical C-sort SHA-256
  `a086d6ad75aae503d2da3649bc84bf8d38605e21b0466c9b5f15ed1c6251b149`.
- Phase 38 transitions to `complete — Complete / High`; Master Phase 39 transitions from `pending` to `ready` only.
  No Phase-39 Child, source inspection or cleanup placeholder exists.

## Execution Log Contract

Future entries record meaningful current discovery, prospective fingerprints, baseline/first-proof behavior,
consumer classification, cleanup, validation and audit handoff. Every test entry includes exact selected/pass/fail/
ignored counts. Failed or zero-selected commands are retained and classified rather than silently replaced.
