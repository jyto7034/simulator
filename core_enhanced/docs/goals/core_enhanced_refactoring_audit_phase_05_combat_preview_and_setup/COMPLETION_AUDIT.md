# Core Enhanced Refactoring Audit Phase 05 — Completion Audit

Audit Mode: phase_exit
Audit Target: `core_enhanced_refactoring_audit_phase_05_combat_preview_and_setup`
Current Status At Audit Start: active
Implementation Complete Claim: not_applicable
Policy Source: Parent Master CRA-01~CRA-14; frozen `engineering_workflow.md#리팩토링-판단`
Repository Baseline: start commit `f318414fb0156a67e04c5479000c2a2683fc0c09` plus initial dirty tree

## Completion Evidence Matrix

| ID | Requirement | Primary evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- |
| P5-01 | exact boundary coverage | `BOUNDARY_AUDIT.md` inventories primary runtime, carriers, data, canonical and consumers | initial/final seven-category hashes | pass |
| P5-02 | preview authority | manifested/Event input, seed, generation, cache and serde trace | preview + map authority tests | pass |
| P5-03 | setup authority | wave/mission/battlefield/player/enemy/reward scenario trace | setup/executor/live RON tests | pass |
| P5-04 | run→battle handoff | plan/preflight/commit/transaction boundary | world start/setup/playability tests | pass |
| P5-05 | determinism/identity | template/wave/warning/appearance/runtime UUID and retry classification | deterministic preview/wave tests | pass |
| P5-06 | predecessor/later seams | Phase 1~4 and Phase 6~10 ownership matrix | no duplicated predecessor candidate | pass |
| P5-07 | live data/canonical | published RON and three canonical comparison | 3 official RON tests + live smoke | pass |
| P5-08 | CRA-14 | `TEST_AUDIT.md` 15 family rows | invariant/assertion/overlap/disposition recount | pass |
| P5-09 | candidates/dispositions | 5 candidates, 12 signals | 50 required candidate field matches | pass |
| P5-10 | dirty preservation/no mutation | all initial/final aggregates identical | scoped status hash identical | pass |
| P5-11 | independent verdict | this phase-exit audit | all rows pass | pass |

## Runtime And Authority Evidence

- A normal Combat/Boss node's payload and manifested `MapNodeCombatDifficulty` are the exact preview/setup input.
  Event-authored combat uses the committed Event encounter and baseline non-manifested tuning.
- Template, generated-wave, warning, appearance and runtime enemy identities use explicit deterministic seeds. The
  resolved preview wave entries, not briefing, are the spawn source.
- Manifested difficulty owns enemy stat, generated-wave budget, extra wave and reward multiplier. Current roster
  changes relative threat/readiness only and `floor_index` is not read by generation.
- Player certified membership and live deployment share condition-plus-weapon admission. Persistent profile inputs
  are separated from current HP, eligible condition magnitude and battle-scoped modifier inputs.
- Setup constructs and validates the scenario before returning a world plan. Normal node confirmation runs inside
  the full state transaction; active lifecycle after successful handoff remains Phase 6.

## Correctness Evidence

### C01 — durable serialized preview cache

The correction is fully evidenced rather than speculative:

1. node selection caches an available Combat preview;
2. cancel leaves that cache while returning to the same completed-component navigation frontier;
3. a sibling SavePoint can be completed, and completed-node graph traversal keeps the original available Combat
   node selectable;
4. SavePoint capture includes all preview cache entries;
5. durable active-run JSON omits `enemy_stat_scale`, whose unit test proves default-on-read;
6. Continue restores the cache without validation;
7. both battle-start resolvers return the cache before difficulty regeneration; and
8. enemy draft creation consumes that default scale.

This contradicts the explicit canonical rule that serialized `CombatPreview` must not become setup source. The
target can preserve current V3/wire shape by rederiving internal setup input, so no migration policy blocks it.

### C02 — publication setup feasibility

Official validation invokes preview generation and warning checks, whereas runtime setup adds enemy-cell capacity,
tactical/field-work positions, unit/objective references and full scenario validation. Typed data can therefore pass
publication and fail only at entry. Current live content is green, but accepted-schema correctness still requires a
pure setup feasibility predicate. The candidate consumes, rather than duplicates, Phase 1 C02's future seed/stage
matrix.

## Test-Code Quality Evidence

- Preview topology and deterministic tests, setup builder tests, world handoff tests and official RON tests protect
  different authority levels and were not collapsed by name similarity.
- Four exact merge/removal targets retain stronger named safety nets: two no-op mission helper tests, one subset
  default-defense test and one duplicate Defense reward-policy failure.
- C01 durable Continue and C02 accepted-but-infeasible data cases remain explicit missing safety nets.
- Seventeen focused invocations executed 79 passing instances, 76 unique after overlap, with zero failure/ignored.
  The runner count supported but did not replace the source/meaning audit.

## Candidate Quality Review

| Candidate | Work type | Authority result | Readiness | Audit verdict |
| --- | --- | --- | --- | --- |
| CRA-P05-C01 | correctness correction | serialized cache becomes presentation-only; node difficulty rederives setup | test first | pass |
| CRA-P05-C02 | correctness correction | publication runs pure full-setup feasibility | after Phase 1 stage matrix, test first | pass |
| CRA-P05-C03 | behavior-preserving refactor | one resolved input/conversion/cache resolver; ghost Floor removed | after C01 and caller inventory | pass |
| CRA-P05-C04 | behavior-preserving refactor | one typed validated setup plan before world commit | after C02 and parity tests | pass |
| CRA-P05-C05 | code cleanup | no-op policy branches/helpers/tests removed with assertion transfer | caller inventory first | pass |

Every candidate records problem evidence, current/target authority, preserved invariants, removal scope, seven
priority axes, safety net, policy decision, readiness and disposition. Priority and readiness remain separate.

## Deferred And Rejected Scope

FacilityEntity activation, manual-wave rebalancing, battle-time wave regeneration, defense-object content data,
current-roster/Floor rescaling, generic start atomicity and broad cache deletion are rejected or deferred for explicit
reasons. Phase 1 C02, Phase 3 economy graph and Phase 4 Event paired commit keep their ownership. Public Rust helper
removal and Unity behavior remain unverified compatibility scope.

## Scoped Diff And Dirty Preservation

All initial/final aggregates are identical:

| Category | SHA-256 |
| --- | --- |
| `combat_preview/**` | `4aed5617eb6b380fb598bf04b159c9dcd4836a3aa2ba8785db395daf8f5618c5` |
| `combat_setup/**` | `ff28e961f42593c479ec442dae0124ae7df3dc9f2a44fb3cf7d96bf15585c9c3` |
| shared handoff/runtime | `c7f2067e427cf5e6b4c0dd2dd4573d62b502024949f7f78c9e904a5d1d0ccdf6` |
| typed data + live RON | `d69711b49028add693fb4f835cd3f5f81084b525133dabce83b7b8e49e0dff75` |
| focused test source | `b8d5e387304896d60918cd1bd339c7eccd569d522da54c894db417dc8886af72` |
| nearest canonical | `da3f35594f77174b9251da9b6ee7a2fb2e9e4fb9bfeb2da33c366034325c6fac` |
| scoped Git status text | `7e768dd5d69ea320d641cdb919f65ea8adc12030977d28a7879f951e4846025a` |

Only Master/Phase audit artifacts changed. Existing user production edits were not restored, formatted, moved or
deleted. Cargo used ignored `target` artifacts only.

## Primary Verdict

Complete

## Long-Term Direction Review

- Fit: High
- Improvement class: C01/C02 correctness first; C03/C04 consolidation after their prerequisite; C05 isolated cleanup
- Reason: the audit identifies a production-reachable cross-serialization authority defect, distinguishes setup
  feasibility from predecessor preview coverage, preserves manifested difficulty and retry semantics, and applies
  CRA-14 with exact surviving safety nets.

## Status Transition

- from: active
- to: complete
- Phase verdict: `Complete / High`
- next Phase activation: allowed
