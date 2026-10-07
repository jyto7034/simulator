# Phase 36 CRA-P05-C05 Completion Audit

Audit Mode: `phase_exit` (restarted after pre-audit documentation normalization)
Audit Target: `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_36_p05_c05`
Audit Date: 2026-08-11 (`Asia/Seoul`)
Current Status: `active`
Implementation Complete Claim: `false`
Candidate / Work Type: CRA-P05-C05 / cleanup
Direct Policy: CRA-POL-006
Policy And Finding Sources: original/post-Boss P05 C05, CRA-TC-02/03/04, rebaselined exact item 36
Goal Documents: current Child `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`; parent Master `PLAN.md`
Repository / Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Audit Scope And Restart

This is a separate Phase-exit review under the complete 874-line authoring/execution guide and complete 567-line
completion-review guide. It reread the corrected current Child, Master linear registry, Phase-35 direct audit and
Phase-27 semantic audit, then independently rechecked current source, callers, assertions, live-data/canonical
fingerprints, discovery delta, dirty state, format/diff gates and fresh nonzero tests.

The first review readback found two PLAN sentences that still called current caller/test discovery pending. E08
updated only those documentation statements before any audit artifact or verdict existed. This audit restarted from
the corrected 273-line PLAN, 283-line experiments and 93-line notes. No production/test/data/canonical correction
was made and no result from the partial review is used as a verdict.

## Approved Condition Restatement

The approved cleanup may replace the elite/ordinary Combat preference branch only because both outputs are exactly
`[Defense]`. It may remove two no-op mission tests, one strict-subset Defense scenario test and one duplicate reward
test only after every unique assertion is present in named survivors. Boss mapping, every map-category result,
SplitRoom's exact failure, the valid and invalid mission pairs, the complete default Defense scenario, empty and
featured/forbidden/Boss reward behavior must remain exact.

CRA-POL-006 separately forbids public Rust removal from negative workspace search. Current actual evidence shows
both candidate helpers are public. Therefore the caller-free archetype wrapper must remain with its exact typed
signature and behavior; this Phase is complete by retaining that public compatibility surface, not by deleting it.
No DTO, RON, save, server, Unity, canonical or deployment migration is approved.

Current actual behavior now equals the approved target: category matching is direct, the public wrapper remains,
and only redundant tests are gone after assertion transfer. Current canonical documents correctly remain unchanged
because gameplay/runtime/data/wire behavior did not change. The historical/final audit records define the approved
cleanup target but are not treated as current runtime authority.

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy/finding basis | Current runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P36-C01 | current helper/test/caller graph is exact | P05 C05; CRA-TC-02 | public module exports both helpers; preference has one production caller; wrapper has no non-test workspace caller; four candidate bodies and survivors inventoried | independent current reference/visibility/name search | pass |
| P36-C02 | dirty targets and concurrent work are preserved | Master dirty invariant; CRA-POL-006 | only `reward_policy.rs` is a new status path beyond activation; exact candidate patches on already-dirty mission/event files; all non-target fingerprints and marker exact | 551 activation rows → 552 pre-audit rows at `90fdf6dc...`; full `git diff --check` | pass |
| P36-C03 | candidate and survivor assertions are fully characterized first | CRA-TC-02/03/04 | E04 assertion table identifies every unique/strict-subset predicate; transfer was applied before removals | old production: mission 5, reward 4, stronger Defense 1, subset Defense 1, RON 25; intermediate survivors 4/4 | pass |
| P36-C04 | no-op mission branch is removed with exact policy parity | P05 C05 | one match now owns Boss→Boss, Combat→Defense, all others→empty; no `kind_id.contains("elite")` remains in node-type preference | category/SplitRoom/live survivor suite 3/3 before and after; Core debug/release 1,073/0/0 | pass |
| P36-C05 | wrapper disposition satisfies CRA-POL-006 | CRA-POL-006 | `fallback_mission_variant_for_archetype` remains public with identical arguments/result and node-type-default implementation; unknown deployed Rust consumers remain explicit | survivor checks Defense and Boss across all eight archetypes; audit refresh mission 3/3 | pass |
| P36-C06 | only redundant tests are removed after transfer | CRA-TC-02/03/04 | two mission, one subset scenario and one duplicate reward body are absent; no other test name changed | current 1,073-name hash `391152c0...`; adding only four names reconstructs prior 1,077-name hash `be0d705e...` | pass |
| P36-C07 | full default Defense scenario remains exact | CRA-TC-03 | survivor pins exact objective/unit ref, fixed player plan, route, enemy/wave movement, object source/position and win condition | baseline, post-change and audit-refresh exact test each 1/1 | pass |
| P36-C08 | reward policy remains exact | CRA-TC-04 | survivor pins empty success and exact full Defense error; separate forbidden and Boss-positive cases remain | reward module baseline 4/4; post-change and audit refresh 3/3; RON 25/25 | pass |
| P36-C09 | DTO/data/save/public/external contracts are unchanged | CRA-POL-006; no-migration boundary | public helper retained; manifests, live data and three canonicals retain activation hashes; no DTO/save/server/Unity path edited | workspace checks both profiles; live RON debug/release; prospective hash audit | pass |
| P36-C10 | focused, broad and static validation is green | Child validation contract | current source hashes equal validated implementation; no later production change | Core debug/release 1,073/0/0; audit refresh 1,073/0/0; checks/fmt/diff/Clippy pass | pass |
| P36-C11 | independent completion audit and next boundary are current | review guide; Master linear gate | Phase 36 is sole active row; Phase 35 and semantic Phase 27 are Complete/High; Phase 37 is pending and absent | this restarted current review returns `Complete / High` | pass |

## Runtime And State Evidence

### Mission preference and callers

- `CombatMissionPolicy::preferred_node_types_for_map_node` remains the explicit policy source. Its current match is
  exhaustive over `MapNodeCategory`: Boss returns `[Boss]`, Combat returns `[Defense]`, every other category returns
  an empty list.
- `world/map_encounters.rs::encounter_candidates_for_map_node` remains the sole production workspace caller. It
  uses the returned list only to filter encounter node types. The same caller still uses `kind_id` separately to
  resolve Normal versus Elite encounter class; that distinct `kind_id.contains("elite")` is not the removed
  identical node-type branch and is correctly retained.
- The test-first category table covers ordinary and elite Combat, Boss and all eight non-encounter categories. It
  passed against old production before the match was simplified and against current production afterwards.

### Mission pairs and public wrapper

- `starts_as_live_battle` still accepts only Defense/Defense and Boss/Boss. Both cross-pairs remain false, and
  production world start/resume callers remain unchanged.
- `try_fallback_node_type_for_archetype` still rejects Combat SplitRoom with the exact existing
  `InvalidStaticData` message. The separate Boss/category and archetype behavior was not changed.
- Public `fallback_mission_variant_for_archetype` still returns `default_for_node_type` and retains its ignored
  archetype argument. The survivor now checks both node types across all eight archetypes, so compatibility is
  explicit without a shim or public narrowing.

### Defense scenario and rewards

- The removed short scenario asserted only generic `ProtectUnit`, `FixedDefense` and `ProtectUnit` shapes. The
  retained default black-box test asserts those predicates more strictly and additionally pins the exact object
  ref/group/source/route position, tactical route, wave movement and win-condition identity.
- `CombatRewardPolicy::validate_rewards` is production-unchanged. Empty pools remain valid; a nonempty Defense pool
  without a featured kind returns the exact full message; disallowed kinds remain rejected; the Boss-positive pool
  still accepts a skill-fragment candidate.
- No command mutation, state transition, timing/order, rollback, retry, checkpoint or determinism path was changed.
  The affected executable contract is pure policy selection plus existing scenario/reward construction.

## Data Evidence

No production RON, schema, loader or validation file changed. Audit-time hashes equal activation for:

- live PVE encounters `0b3f03b7e68c3aff5db6270e9eaa83eacb4eb2da188e55058899db22f0a3ee1c`;
- battlefield templates `1f51d0e4cc1a784c684e60cf4bef9ee4b4e943aa776cd7db781ad7078144223f`;
- battlefield archetypes `c8251aab51532ee5f045fe182999e0f627e89c7e8ca936f2714bf165fc043dd9`;
- node definitions `64cc593e09712d6d619306679a97642635fad9deec6d9ad8108dcf4b0ebaa2e7`;
- run policy `83278e068302506f683eb593260ff3fe1024f71c30236c3a4b46c3e47ef7fe55`.

The 25-case `ron_loading` integration passed before mutation, after mutation, in both broad profiles and again in
the audit refresh. It exercises live encounter references, authored scenario preview data and live reward contracts.
No fixture-only policy, silent normalization or data fallback was introduced.

## DTO, Transport And Persistence Evidence

No serialized type, tag, field, nullability, server adapter, message order, request correlation, V3 checkpoint,
reader/writer, restore/reconnect or Unity consumer path is part of this private policy/test cleanup. Root/Core
manifests, lockfile, Core library root, public module root, `combat_preview/mod.rs` and `world/combat.rs` retain their
activation hashes. The public Rust wrapper is retained, so no versioning, deprecation, alias, dual path or coordinated
rollout is required.

## Consumer Evidence

- Confirmed workspace production consumer: `world/map_encounters.rs` consumes the preference helper; world combat
  consumes the unchanged live-pair policy; preview generation consumes the unchanged archetype fallback; reward
  construction consumes the unchanged featured-kind policy.
- Confirmed workspace test consumers: the three mission survivors, three reward survivors, full default Defense
  scenario and live-RON/all-target suites.
- Canonical external consumers: none directly consume these private implementation tests; game_server/Unity consume
  unchanged serialized Core results rather than these Rust helpers.
- Inaccessible or unknown consumers: deployed Rust consumers of the public mission-policy module cannot be ruled
  out. CRA-POL-006 is satisfied by retaining both public methods and their typed behavior.
- Explicitly excluded: live server/WebSocket, save restore and Unity runtime builds are non-applicable because no
  wire, persistence or client contract changed. Workspace compilation includes `game_server` in both profiles.

## Legacy And Compatibility Audit

- The four approved redundant test names have zero current source matches.
- The identical elite branch has zero current match in the node-type preference function. The remaining elite
  substring in `map_encounters.rs` selects encounter class and remains a distinct live authority.
- No ignored/disabled replacement, test-only production branch, deprecated wrapper, new alias, silent fallback,
  dual path/schema, placeholder or Phase-37 validator code was added.
- The public archetype wrapper is deliberately retained compatibility, not a newly introduced shim. Its continued
  support is the required CRA-POL-006 disposition for unknown consumers.

## Scoped Diff And Dirty Preservation

- Start HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Pre-Child expanded status was 548 rows at
  `9c03cb1c82f8acdd41686da5291ac30222aa155aa810d28ed7c389b3f4690172`.
- Sole activation added only three Child files: 551 rows at
  `88a595dd4801e12528349a8220a28318d6133219aa3ee7af74343d2526dade4a`.
- Before this audit artifact, the implementation-complete tree has 552 rows at
  `90fdf6dcef93f88cd0b2d097bcfdc4ff9a1250d6b7eb71dcecbb6c298551316c`. The only new status path beyond
  activation is formerly-clean `reward_policy.rs`; mission policy and event combat were already dirty.
- Exact current candidate hashes are mission policy `87465db2...`, event combat `7b25d74a...` and reward policy
  `b2b4b5f7...`. Apply-patch history and scoped diff show only the direct match/test assertions, four body removals
  and two final whitespace lines. The pre-existing `field_work_objects` change and every other existing event hunk
  remain untouched.
- Every non-target prospective source/data/canonical hash equals activation. Whole-worktree `git diff --check` and
  format check pass.
- The protected marker remains exactly 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`; mtime and ctime are both
  `2026-08-10 22:55:07.178423600 +0900`.
- Phase 36 is the only active Master row. Phase 37's directory is absent during the audit.

## Canonical Alignment

The cleanup changes no durable gameplay meaning: elite and ordinary already selected the same Defense node type,
all policy results/errors remain exact, and production reward/scenario logic is unchanged. Audit-time hashes equal
activation for `game_rulebook.md` (`6c53ecd5...`), `core_runtime_contract.md` (`ab7d2f98...`) and
`data_loading_contract.md` (`30ed5fd5...`). Leaving them unchanged is correct; adding helper/test layout to a
durable canonical would create the wrong authority. Goal/history documents remain evidence records only.

## Validation Results

All commands ran in the current intentionally dirty workspace.

### Old-production baseline and test-first gate

- `cargo test -p game_core --lib game::combat_setup::mission_policy::tests::`: 5 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --lib game::reward_policy::tests::`: 4 passed, 0 failed, 0 ignored.
- each exact default/subset Defense scenario filter: 1 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --test ron_loading`: 25 passed, 0 failed, 0 ignored.
- four exact transferred survivors on still-old production: 4 passed, 0 failed, 0 ignored.

### Post-change focused and broad

- mission policy: 3 passed, 0 failed, 0 ignored; reward policy: 3/0/0; stronger Defense: 1/0/0; live RON: 25/0/0.
- `cargo test -p game_core --all-targets`: 1,073 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --release --all-targets`: 1,073 passed, 0 failed, 0 ignored.
- Each broad distribution is library 1,022 plus integrations 4 + 4 + 25 + 14 + 4. The retained default main and
  five fixture binaries each report 0/0/0 and are not counted as behavioral proof.
- `cargo check --workspace --all-targets` and release equivalent: exit 0.
- `cargo fmt --all -- --check`: exit 0 after the two-line whitespace correction.
- whole-worktree `git diff --check`: exit 0.
- Core Clippy all-target debug and release: exit 0 with inherited 82 library / 134 library-test warnings.

### Independent audit refresh

- mission policy 3/0/0; reward policy 3/0/0; stronger Defense 1/0/0; live RON 25/0/0.
- `cargo test -q -p game_core --all-targets`: 1,073 passed, 0 failed, 0 ignored with the same harness distribution.
- current sorted discovery: 1,073 rows at `391152c0...`; adding only the four removed `: test` rows reconstructs
  the prior 1,077-row table exactly at `be0d705e...`.
- current prospective fingerprints, protected marker, sole-active row, Phase-37 absence, format and diff checks were
  all reverified after the E08 documentation correction.

Separate game_server runtime tests, WebSocket probes, Unity builds, save/restore and performance benchmarks are
omitted as non-applicable: there is no changed DTO, transport, persistence or hot algorithm. Both workspace check
profiles compile the Core/server boundary. Master final Stage-D validation remains a later required boundary.

## Baseline Failures

No baseline, focused, broad or audit-refresh test/check failed. The first post-change format check reported only one
extra blank line at each edited test-module end; both were removed and the check passed. The first final fingerprint
command used eight nonexistent shorthand paths, reported `No such file or directory`, then read-only filename
discovery resolved the actual paths and all hashes matched activation. Neither event was a production behavior
failure. Normal Clippy warning totals are inherited and unchanged.

## Cross-Component Findings

- No competing mission/reward authority or contract drift exists. Map encounter class selection continues to own
  Normal/Elite class, while mission preference directly owns the unchanged Defense/Boss node-type result.
- Combat preview fallback, world live-pair admission, scenario construction and reward validation remain separate
  existing responsibilities and pass together in live-RON and all-target runs.
- No Phase-37 event-log validator symbol, test or placeholder was touched. Phase 37 needs only the linear
  Complete/High exit gate and no new runtime seam from this cleanup.
- Material cross-component findings: none.

## Detailed Findings

- Material findings: none.
- Corrected before restarted verdict: two stale `discovery pending` PLAN sentences; documentation only.
- Policy questions: none.
- External blockers: none. Unknown deployed Rust callers are handled by exact public retention.

## Long-Term Direction Review

| Candidate ID | Work type | Current→target authority result | Preserved invariant result | Removal/consolidation result | Validation basis | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P05-C05 | cleanup | identical elite/ordinary node-type branch → one direct category match; public compatibility wrapper retained | exact category/Boss/SplitRoom/live pairs, Defense scenario, reward matrix and public results | four redundant bodies removed only after transfer; no shim, placeholder or adjacent validator cleanup | old-production characterization, exact discovery delta, focused plus debug/release broad runs | pass |

- Fit: `High`.
- Improvement class: `none`.
- Reason: the implementation removes a false policy distinction and assertion duplication without creating another
  abstraction or compatibility mechanism. Public compatibility is deliberate and exhaustively tested, authority
  boundaries remain domain-specific, and surviving tests protect outputs/errors/full scenario shape rather than
  private branch layout.

## Primary Verdict

- Primary verdict: `Complete`.
- Long-term fit: `High`.
- Combined verdict: `Complete / High`.
- Confidence: High. Old-production characterization, test-first transfer, exact four-name discovery reconstruction,
  fresh audit tests and activation-exact non-target fingerprints provide independent converging evidence.

## Status Transition

- Child: `active` / `Implementation Complete: false` → `complete` / `Implementation Complete: true`.
- Master Phase 36: `active` → `complete — Complete / High`.
- Master Phase 37: `pending` → `ready` only after the Child and Master exit records are updated.
- Next Phase implementation activation: allowed only through the full Master activation protocol. This audit did not
  create or inspect a Phase-37 Child.

## Remaining Risk And Required Action

No material Phase-36 correction remains. Unknown deployed Rust consumers remain a documented non-breaking risk
because the public wrapper is retained rather than depublished. Apply only the record transitions above, preserve
the protected marker and all dirty work, then make Phase 37 ready. Master final Stage-D validation and final
cross-Phase audit remain required before the 44-Phase campaign can complete.
