# Game Rulebook Runtime Alignment Audit — Completion Review

Review mode: `audit_only` review of an `investigation_only` Goal  
Production mutation authorized: no  
Audit target: current working tree rulebook SHA-256
`c11a09402febbb6b16dcc98a11dd1364bac8dfe53645627f1670e6fbd7380f1e`

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy basis | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| `CA-01` | Every rulebook gameplay section and normative claim is inventoried with stable IDs. | PLAN Completion Conditions | `RULEBOOK_CLAIM_MATRIX.md` coverage scheme and 58 rows | Range verifier: 58 contiguous rows, next line 1211; source counts: 58 headings, 365 list items | pass |
| `CA-02` | Current, approved-pending, delegated and future/dormant meanings are separated. | PLAN taxonomy/invariants | Matrix lifecycle column; catalog classification and exclusions | Manual lifecycle pass plus hidden-boss/exact-authoring boundary recheck | pass |
| `CA-03` | Current claims are judged from reachable runtime and production live data, not symbol presence. | `GRB-AUD-SCOPE-001` | Facility, result, pool, research, BossOmen, Event, Monolith, reward and loader traces in catalog/experiments | 1,063 fresh Core tests plus static command-to-authority traces | pass |
| `CA-04` | Serialization/server/access­ible consumer boundaries are checked without overstating access. | PLAN Source Of Truth | Server mappings/persistence and readable Unity readers/command bus/canonicals | 63 fresh server tests; Unity source/assertions inspected; runner absence disclosed | pass |
| `CA-05` | Final catalog contains mismatches and required evidence gaps only; aligned claims stay in matrix. | PLAN Completion Conditions | 15 catalog entries; matrix contains aligned/delegated coverage | Catalog heading/summary count both 15; matrix↔catalog ID set has no difference | pass |
| `CA-06` | Every mismatch has both rulebook and actual-authority evidence plus impact/direction/confidence. | PLAN Finding record minimum fields | `MISMATCH_CATALOG.md` detailed records | Manual field-by-field review of `GRB-M-001` through `GRB-M-015` | pass |
| `CA-07` | High-risk legacy/fallback paths receive an independent pass. | PLAN Broad Validation | Research/delivery, old weighting, RunComplete, transport, `ChooseSupport`, sweep, BossOmen and persistence searches | server rejects removed wire paths; catalog separates live legacy from future/dormant | pass |
| `CA-08` | No correction, migration, compatibility, production or canonical mutation occurs. | `GRB-AUD-SCOPE-002` | Only Goal-directory Markdown was patched; test target is `/tmp/grb_audit_target` | final scoped status/diff inspection and unchanged rulebook hash | pass |
| `CA-09` | Existing dirty user work remains preserved. | PLAN Invariants | baseline/current commit and broad dirty status recorded | no reset/restore/stash/formatter/destructive command; production status remains user-owned | pass |
| `CA-10` | An independent completeness pass and explicit verdict exist. | PLAN Completion Conditions; completion-review guide | this document plus artifact cross-checks | requirements audited one row at a time below | pass |

## Runtime And State Evidence

The audit traced the high-risk rules from their actual entrypoints to authoritative state:

- `generate_current_floor_map` through facility assignment, difficulty manifestation and encounter class;
- combat result planning/commit through research, rewards, attempts, node completion and terminal states;
- `MonolithRunState` natural outcome, purchase/use, remanifestation victory/failure and checkpoint fields;
- `BossOmenRunState` provisional/active/placed/forced transitions, selection, source consumption and
  capstone resolution;
- Event choice → live battle → common attempt/result completion;
- persistent global compendium state versus run/checkpoint-scoped state.

This proves both aligned rules and the contradictions in the catalog. The passing test suite is not used
as blanket rulebook proof; named expectations that protect an older contract are identified as such.

## Data Evidence

Production data was distinguished from schema/test fixtures. The audit inspected and connected:

- `map/node_assignment_profiles.ron` to the production facility schedule and generator;
- `run/policy.ron` to research, repeat weighting, Monolith and difficulty authority;
- `abnormalities/base.ron` to response-complete fragment research;
- `events/rewards/base.ron.default_treasures` to live Reward-node content;
- `pve/encounters.ron` to encounter-class selection;
- `boss_omen/chains.ron` to the active chain schema/selector;
- `skill_fragments/base.ron` to the 29 acknowledged direct-modifier placeholders;
- `employees/definitions.ron` and `equipments/base.ron` to starter/equipment claims.

Schema-only old bonus-objective research fields were not inflated into a separate live-data finding when
no live encounter authored them. Conversely, reward metadata was counted only after its pool and node
resolver were connected.

## DTO/Transport/Persistence Evidence

- Core result/snapshot DTOs still produce research diffs and planning-return deliveries.
- Server preserves these deliveries before its trailing snapshot.
- Server compendium persistence stores only the current four ID sets; no tutorial/record profile exists.
- Current battle setup/update/checkpoint transport is preserved and legacy battle transport is rejected.
- Run/checkpoint state stores the old research/Monolith/BossOmen shapes and lacks the new cycle/profile
  ledgers identified in findings.

No DTO or migration was changed.

## Consumer Evidence

The external Unity repository and both rulebook-named Unity canonicals were readable. Current source
inspection confirmed:

- strict `research_deliveries` parsing and presentation;
- both Standard and Endless mode commands in the main menu;
- compendium-only global gameplay profile consumption;
- current battle setup/update/checkpoint consumption;
- absence of sweep and new profile/cycle consumers.

No Unity executable was available, so zero fresh Unity tests are claimed. Existing EditMode/PlayMode test
source was used as contract evidence, and no finding depends only on those unexecuted assertions.

## Legacy/Compatibility Audit

- Research/progress/pending delivery is a confirmed reachable legacy path (`GRB-M-002`).
- Old research/recent-floor encounter weighting remains active (`GRB-M-005`, `GRB-M-010`).
- Research completion remains an Endless terminal condition (`GRB-M-003`).
- Removed `timeline_delta`/`battle_delta` production transport was not found; server tests reject legacy
  battle result transport.
- Removed `choose_support` does not deserialize. Unity-local HQ “support” presentation maps to current
  typed HQ action capabilities and is not a compatibility alias.
- Tactical `Rest` is a tile-effect domain and not the removed noncombat Rest mode.
- No sweep alias/fallback exists.

## Scoped Diff And Dirty Preservation

Baseline and final repository commit are both
`f318414fb0156a67e04c5479000c2a2683fc0c09`; the rulebook hash is unchanged. The repository already had
broad user changes across Core, tests, canonical, live data and server. The audit did not reset, restore,
stash, clean, format or rewrite them.

Audit-created/updated content is confined to:

- `docs/goals/game_rulebook_runtime_alignment_audit/PLAN.md`
- `docs/goals/game_rulebook_runtime_alignment_audit/EXPERIMENTS.md`
- `docs/goals/game_rulebook_runtime_alignment_audit/EXPERIMENT_NOTES.md`
- `docs/goals/game_rulebook_runtime_alignment_audit/RULEBOOK_CLAIM_MATRIX.md`
- `docs/goals/game_rulebook_runtime_alignment_audit/MISMATCH_CATALOG.md`
- `docs/goals/game_rulebook_runtime_alignment_audit/COMPLETION_AUDIT.md`

Rust build/test artifacts were redirected to `/tmp/grb_audit_target`.

## Canonical Alignment

The investigation does not update canonical gameplay text. It reports where the current runtime/live
data differ from the updated gameplay canonical and preserves the repository's source-of-truth order.
Detailed targeting/runtime/data/wire claims delegated by the rulebook were checked against their named
adjacent canonicals rather than restated as new policy.

The catalog deliberately does not decide which side should change. That is consistent with
`GRB-AUD-SCOPE-001`/`002` and prevents an audit artifact from becoming a gameplay decision record.

## Validation Results

Fresh executable validation:

| Suite | Result |
| --- | --- |
| `cargo test -p game_core --lib` | 1,012 passed; 0 failed; 0 ignored |
| five `game_core` integration targets | 51 passed; 0 failed; 0 ignored |
| `cargo test -p game_server --lib` | 63 passed; 0 failed; 0 ignored |
| total Rust | 1,126 passed; 0 failed; 0 ignored |
| Unity | not executed; runner unavailable |

Artifact validation:

- coverage ranges: 58, contiguous L1-L1210, verifier ended at 1211;
- rulebook source: 58 headings, 365 list items;
- finding summary rows: 15;
- detailed finding headings: 15;
- matrix/catalog finding-ID set difference: empty;
- final rulebook SHA-256 matches baseline.

## Baseline Failures

No baseline test failure was discovered. Two investigation-command issues did not affect content:

- a first documentation patch failed its context check and changed no file;
- an early broad search named two nonexistent candidate source files and produced truncated output; the
  search was rerun against the actual files with narrow ranges.

## Cross-Component Findings

The largest cross-component mismatch is the research contract: old Core authority, live policy/rewards,
server mapping and Unity presentation remain internally consistent with each other while collectively
contradicting the updated rulebook. This is why it is classified as a live legacy path rather than a
Core-versus-Unity serialization bug.

The Standard, pool/Monolith and BossOmen findings are primarily Core/live-data ownership differences;
consumers receive the already-decided projection and cannot repair them. The absent sweep/profile/cycle
targets require multi-owner future implementation but no correction scope is approved here.

## Long-Term Direction Review

The audit output fits the repository's long-term ownership:

- full claim coverage remains separate from mismatch-only output;
- runtime/live data stay authoritative evidence rather than being edited to satisfy a document check;
- expected document-ahead work is separated from accidental contradictions and dormant design;
- exact future content authoring is not confused with missing engine authority;
- producer and consumer evidence is kept distinct;
- no compatibility shim or dual contract is proposed.

## Primary Verdict

**Complete / High confidence** for the investigation Goal.

This verdict means the requested read-only document/runtime mismatch audit is complete, not that the game
implements the rulebook. The current game does not fully align: 15 confirmed findings remain in
`MISMATCH_CATALOG.md`. Finding correction is outside this Goal and remains unapproved.

The unavailable Unity runner is a disclosed validation limitation, but not a completion blocker: all
required consumer sources were accessible, Core/server executable evidence is current, and no finding
rests solely on assumed Unity behavior.

## Status Transition

Applied transition: `Status: active` → `Status: complete` after final document/status validation passed.
`Implementation Complete` remains `not_applicable` because this is an investigation-only Goal.

## Remaining Risk And Required Action

- The catalog is a point-in-time audit of the recorded dirty working-tree generation. Any later code,
  data, rulebook or Unity change requires a new hash/generation check before reusing its verdict.
- No implementation action is authorized. If correction is desired, policy owners should choose scope
  and ownership per finding, then create a separate implementation Goal with migration/consumer/tests as
  appropriate.
- Future multi-chain BossOmen and exact fragment/reward authoring can make currently structural gaps more
  immediately player-visible; `GRB-M-008`, `GRB-M-011`, and `GRB-M-012` should be considered together.
