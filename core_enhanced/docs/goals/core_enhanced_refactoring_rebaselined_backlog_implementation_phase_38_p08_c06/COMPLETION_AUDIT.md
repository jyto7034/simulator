# Phase 38 CRA-P08-C06 Completion Audit

Audit Mode: `phase_exit`
Audit Target: `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_38_p08_c06`
Audit Date: 2026-08-11 (`Asia/Seoul`)
Current Status: `active`
Implementation Complete Claim: `false`
Candidate / Work Type: CRA-P08-C06 / cleanup
Direct Policy: CRA-POL-006
Approval Source: Master PLAN Phase 38 row 235 — `사용자답변: 승인`
Finding Sources: original/post-Boss P08-C06, final test catalog, rebaselined exact item 38
Goal Documents: current Child `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`; parent Master `PLAN.md`
Repository / Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Audit Scope And Independence

This is the separate Phase-exit review required by the Child and Master contracts. It treats the implementation
narrative as a pointer rather than proof and independently reviews the current deletion state, real production
assertion, authority graph, public-consumer decision, exact test-list delta, package/data/canonical boundaries,
dirty preservation and validation results.

After implementation documentation was updated, the audit reread the actual production assertion body, reran the
complete Core debug all-target suite, repeated current legacy searches, verified the deleted path and sole active
Phase, and refreshed status/path/marker hashes plus whole-worktree diff syntax. Release all-target, workspace
debug/release checks and Clippy remain valid because no production, test, manifest, data or dependency source
changed after those commands; only Goal evidence documents changed.

## Approved Condition Restatement

CRA-P08-C06 must eliminate the disconnected public `battle::death` model only after its useful duplicate-death and
deterministic unique ally-order intent executes through the real `BattleCore` death lifecycle. The target authority
is `core::death_runtime::finalize_unit_death` plus the typed occurrence dispatcher; no queue/removal compatibility
model, shim or private replacement may survive.

CRA-POL-006 initially stopped removal because local negative searches could not exclude standalone/deployed Rust
consumers. Master row 235 now records the user's candidate-scoped approval. This authorizes the exact breaking
removal of `DeadUnit`, `DeathProcessResult`, `DeathHandler`, the `battle::death` export and their sole self-test. It
does not authorize production death/damage/event changes, data/DTO/save/wire changes or Phase-39 work.

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy/finding basis | Current runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P38-C01 | current legacy graph is exact | P08-C06 | pre-cleanup one 244-line file, one export, three public types and one self-test enumerated | baseline 1/1 legacy test; complete local consumer searches | pass |
| P38-C02 | production death authority graph is exact | P08-C01/C02/C05 | `finalize_unit_death` retains Dead units and owns cleanup/graveyard/events; typed dispatcher owns death occurrences | production assertion body and dependency audits | pass |
| P38-C03 | dirty targets and concurrent work are preserved | Master dirty invariant | only clean `death.rs` adds a status row; pre-dirty `mod.rs` and `commands.rs` changes remain attributable | 571 activation rows → 572 pre-audit rows; exact marker/hash checks | pass |
| P38-C04 | legacy assertion meaning is characterized | final test catalog | duplicate queue/removal and reverse ally input classified separately from false partial Modifier model | old legacy test 1/1 and assertion-disposition matrix E03 | pass |
| P38-C05 | real production death is characterized | P08-C01/C02/C05 | actual damage handoff, lifecycle, graveyard, UnitDied and occurrence order are directly observable | exact current production test 1/1; commands 43/43 | pass |
| P38-C06 | useful assertion moves before deletion | exact item 38 | two lethal inputs yield one damage/kill/death and exact unique sorted ally plans | assertion passed before cleanup and remains current | pass |
| P38-C07 | public consumer/removal gate is exact | CRA-POL-006; Master row 235 | no local consumer outside old self-test; user gives candidate-scoped breaking approval | zero current source/manifests references; workspace Core/server compile | pass |
| P38-C08 | only approved false authority is removed | Master row 235 | exact file/types/export/self-test absent; no shim, alias or parallel model | zero legacy search; exact one-row discovery reconstruction | pass |
| P38-C09 | runtime lifecycle/order/data behavior remains exact | P08 boundary | death runtime hash exact; production assertion carrier is the validated test-only transfer | death 15, item 31, skill 106, world 245; Core both profiles 1,067 | pass |
| P38-C10 | DTO/RON/save/server/Unity compatibility is exact | Phase boundary | manifests, lockfile, lib root, live Equipment/Skill/Buff and canonicals retain fingerprints | workspace debug/release checks; 25-case RON integration in all-target suites | pass |
| P38-C11 | focused, broad and static validation is green | Child validation contract | tested source hashes equal current implementation | debug/release all-target, workspace checks, format/diff and normal Clippy pass | pass |
| P38-C12 | independent audit and next boundary are current | linear gate | Phase 38 is sole active Child; Phase 39 is pending and absent during audit | this review returns `Complete / High` | pass |

## Runtime And State Evidence

- `core_enhanced/src/game/battle/death.rs` is absent. `battle/mod.rs` no longer exports `death`; its only other
  deletion is the preserved Phase-37 `validation` export removal.
- `core/death_runtime.rs` retains SHA-256
  `096a68265a37d6af46cf7c3eaaae10bdb4a632c4323ee17f4b0062f2384fc6d9`; no runtime death code changed in this
  Phase.
- The production test carrier remains SHA-256
  `78ef19fe5119060aa350885825eb76403a9be481707b9d067e4071cbe678eb7a`. Its current body submits two same-time
  lethal `ApplyDamage` commands for one victim and asserts one actual-damage target, one killed target, Dead
  lifecycle, one graveyard entry and one `UnitDied`.
- The same assertion compares the exact occurrence output: OnDeath on the dead victim, OnKill on the killer, then
  one OnAllyDeath effect/activation for each UUID-sorted distinct live ally. Final attack values and ordered
  `StatChanged` targets also remain exact.
- No production queue, unit-removal result model, second dispatcher or Modifier-only death path was introduced.

## Consumer And Compatibility Evidence

- Before cleanup, only the colocated self-test constructed or named the three public legacy types. Workspace,
  server, tool and accessible `/mnt/f/work` searches found no other consumer.
- The crate remains publicly reachable, so negative local evidence alone was correctly treated as insufficient.
  Master row 235 now supplies the missing candidate-scoped breaking authority.
- Current Core/server source, test and manifest search has zero matches for `DeadUnit`, `DeathProcessResult`,
  `DeathHandler`, `battle::death`, `pub mod death` or the removed self-test name.
- No deprecation duplicate, alias, feature-gated copy, fallback, ignored test or placeholder exists. Historical Goal
  references remain evidence of the old state, not a live compatibility surface.

## Data, DTO, Transport And Persistence Evidence

- Live Equipment, Skill and Buff RON remain respectively
  `322d7455a256082f03804c0dcc7a77a4caa3a06919d5e34b667042b411bcd1a6`,
  `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770` and
  `e7cc335f4243b4fc0ac907a72356ba44a1eae2669aa340147a0fbbc14f9c448b`.
- Root/Core/server manifests, lockfile and Core library root retain activation hashes. Workspace checks compile
  `game_core` and `game_server` together in both profiles.
- No event carrier, serde shape, DTO, mapping, message order, save/checkpoint, loader, RON schema/value, server or
  Unity source changed. No coordinated rollout or migration is required beyond the explicitly approved Rust API
  break.
- The all-target distributions include the 25-case current RON loading integration suite in debug and release. The
  exact production occurrence proof itself constructs typed test inputs, so no extra live-data-only probe is needed.

## Scoped Diff And Dirty Preservation

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Sole activation baseline was 571 expanded rows at
  `68785da9ec0b529b278f26f052882f771309fe4443f87e909de3312283c799a4`.
- Before this audit artifact the implementation tree is 572 rows at
  `623dc1d8d4da946dd2dafff20eec53622c00c507ac1dfe0806addc4fa844586d`. The exact +1 path is the formerly-clean
  tracked deletion of `death.rs`; `battle/mod.rs` and the assertion carrier were already status rows.
- The approved source delta is 244 deleted file lines plus one export line. No unrelated path was cleaned, moved,
  restored or formatted.
- Current `battle/mod.rs` is
  `6100a99907befaa0c7556f8437a07400c07c8d17af6d8088deaed24ed5cb79a1`; the inherited validator removal and new
  death removal are both visible as exact one-line deletions.
- `git diff --check` and format check pass. The protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`; modify/change timestamps remain
  `2026-08-10 22:55:07.178423600 +0900`.
- After adding this sole audit artifact, expanded status is expected to be 573 rows at canonical C-sort SHA-256
  `a086d6ad75aae503d2da3649bc84bf8d38605e21b0466c9b5f15ed1c6251b149`.

## Canonical Alignment

`game_rulebook.md`, `core_runtime_contract.md` and `data_loading_contract.md` retain activation hashes
`6c53ecd5...`, `ab7d2f98...` and `30ed5fd5...`. The cleanup changes no gameplay, data, save, DTO or wire meaning, so
leaving these durable canonicals unchanged is correct. The approved public Rust break is recorded in Master/Child
execution evidence rather than promoted into gameplay authority.

## Validation Results

All commands ran in the current intentionally dirty workspace.

### Baseline And Assertion-Transfer Proof

- Removed legacy self-test before cleanup: 1 passed, 0 failed, 0 ignored, 1,016 filtered.
- Production occurrence assertion before transfer: 1/0/0; complete commands baseline: 43/0/0.
- After assertion transfer and before cleanup: exact production 1/0/0, commands 43/0/0 and death filter 16/0/0.

### Post-Cleanup Focused Validation

- A shortened `--exact` filter selected 0 tests and 1,016 filtered because it omitted the full module path. It was
  classified as a selection-command error and is not used as evidence.
- Correct full-path exact production assertion: 1 passed, 0 failed, 0 ignored, 1,015 filtered.
- Commands module: 43/0/0, 973 filtered. Death filter: 15/0/0, 1,001 filtered.
- Item filter: 31/0/0, 985 filtered. Skill filter: 106/0/0, 910 filtered. World filter: 245/0/0, 771 filtered.

### Post-Cleanup Broad And Static Validation

- `cargo test -q -p game_core --all-targets`: 1,067/0/0.
- `cargo test -q -p game_core --all-targets --release`: 1,067/0/0.
- Both distributions are library 1,016 plus integrations 4 + 4 + 25 + 14 + 4. Six executable/fixture targets
  select zero tests and are not counted as behavioral proof.
- `cargo check --workspace --all-targets` and release equivalent: exit 0 for both `game_core` and `game_server`.
- `cargo fmt --all -- --check` and whole-worktree `git diff --check`: exit 0.
- `cargo clippy -p game_core --all-targets`: exit 0 with inherited 82 library / 134 library-test warnings plus
  existing integration warnings; no removed-surface diagnostic appeared.
- Current sorted full discovery: 1,067 rows at
  `7c77e183a4c54e2a5f51d0bdfe35f80adae53128d006728fa55577646a760eac`. Adding only the removed full legacy-test row
  reconstructs the Phase-37 1,068-row hash exactly at
  `71ff93259eb70ba0948bd28a0e783416fc55ee1927b821ae49a77472a4086a1d`.

### Independent Audit Refresh

- `cargo test -q -p game_core --all-targets`: 1,067/0/0 with the same distribution.
- Current legacy search selects zero live source/test/manifest matches; deleted-path, sole-active-Phase,
  Phase-39-absence, 572-row pre-audit status, marker and diff syntax were independently reverified.
- Release, workspace and Clippy commands were not repeated during refresh because the audited source fingerprints
  equal the successful implementation-validation state; only Goal evidence documents changed afterward.

## Failures And Diagnostics

No baseline, implementation, focused, broad or audit-refresh behavioral test failed. The one zero-selected exact
command was a transparent filter error, corrected immediately with the full test path. Clippy warnings are inherited
and unchanged in classification; they do not originate from the two deletion-only production deltas.

## Cross-Component Findings

- One real production death authority remains and its observable duplicate/order safety net is stronger than the
  removed partial model.
- Public damage/event carriers retained by Phase 28 remain unchanged.
- No Phase-39 movement backend, Phase-41/42 event-log/file-I/O or Phase-44 package cleanup was pulled forward.
- Material cross-component findings: none.

## Long-Term Direction Review

| Candidate ID | Work type | Current→target authority result | Preserved invariant result | Removal/consolidation result | Validation basis | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P08-C06 | cleanup | disconnected queue/removal false authority → production `finalize_unit_death` only | exactly-once death, OnDeath→OnKill→unique UUID-sorted OnAllyDeath, cleanup/data boundaries exact | one export, 244-line module, three types and one self-test removed with no shim | real production assertion, exact discovery reconstruction, debug/release/workspace/static passes | pass |

- Fit: `High`.
- Improvement class: `none`.
- Reason: the false authority disappears without creating a replacement abstraction. Production-owned tests protect
  the only useful behavior, runtime and compatibility boundaries remain exact, and later cleanup remains linear.

## Primary Verdict

- Primary verdict: `Complete`.
- Long-term fit: `High`.
- Combined verdict: `Complete / High`.
- Confidence: High. Candidate-scoped approval, test-first assertion transfer, exact file/export/symbol removal,
  discovery reconstruction, both profiles, workspace compilation, static checks and current preservation hashes
  provide independent converging evidence.

## Status Transition

- Child: `active` / `Implementation Complete: false` → `complete` / `Implementation Complete: true`.
- Master Phase 38: `active` → `complete — Complete / High`; approved public break applied.
- Master Phase 39: `pending` → `ready` only after Child/Master exit records are updated.
- Next implementation activation remains gated by Phase-39 initialization, full required-document/dependency reads,
  sole activation and prospective fingerprints. This audit does not create or inspect a Phase-39 Child.

## Remaining Risk And Required Action

The public API removal is intentionally breaking and explicitly approved on the authoritative statement that no
external consumer exists. No Phase-38 correction remains. Preserve the protected marker and dirty work, apply the
record transitions above, make Phase 39 ready only, and leave Master final Stage-D validation/final audit for after
Phase 44.
