# Phase 20 CRA-P02-C03 Experiment Notes

Status: complete
Implementation Complete: true
Candidate: CRA-P02-C03
Master Phase: 20 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Locked Decisions

- CRA-POL-006: current strict V3/public/save/wire behavior and every deliberate exclusion remain exact.
- F-C01 remains the pre-mutation admission authority; conversion does not duplicate or relax its domain graph.
- Work type is behavior-preserving refactor; current valid capture/restore behavior is presumed correct until current
  evidence proves otherwise.
- Consolidate only private bidirectional checkpoint mapping; do not merge `RunState`, persisted payload,
  next-Floor carry-forward or host-envelope ownership.
- Active battle/Boss/tactical state remains excluded; no version/field/error/lifecycle migration is authorized.
- Phase 21 progression encapsulation and every later refactor/cleanup are prohibited from this Child.

## Open Evidence Slots

- N02: activation expanded status, prospective target fingerprints and scoped dirty attribution.
- N03: current capture→export→parse→validate→restore call graph and exact field/lifetime table.
- N04: Core/server/public/save consumer and V3 compatibility inventory.
- N05: pre-refactor golden field/exclusion/bidirectional/failure-order characterization.
- N06: implemented private conversion, transferred callers and obsolete mapping removal.
- N07: final focused/broad/static/canonical evidence and independent-review handoff.

## 2026-08-10 — N01 Policy, Dependency And Activation Boundary

- Source/evidence: CRA-POL-006; original and re-audited P02-C03 candidate/boundary/test evidence; Final exact
  order/test/gates; F-C01 and Phase 19 completion audits; Master readiness transition.
- Observation: F-C01 closes invalid imported graph admission before this consolidation. The remaining problem is the
  correct but repeated private mapping between active run/root state and a strict derived V3 document.
- Observation: exact V3 semantic fields/values and active battle/Boss exclusion are the first proof. Host metadata,
  UUID/VN lifetime, `combat_recovery_restriction`, next-Floor carry-forward and public APIs are separate preservation
  seams rather than conversion expansion targets.
- Impact: one private checkpoint conversion may proceed only after golden characterization; schema/lifetime/public
  changes, repair, compatibility and Phase 21 work are forbidden.
- Disposition: activate Phase 20 only; do not inspect production until all Child documents and the final Phase 19
  audit are fully read and the dirty baseline is captured.

## 2026-08-10 — N02 Activation Dirty Baseline

- Source/evidence: expanded `git status --short --untracked-files=all`, scoped status and SHA-256 fingerprints after
  mandatory reads and before production/test/canonical content inspection.
- Observation: HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`; expanded status is 468 entries at
  normalized SHA-256 `3571680ccb302552f804e18af9681ed9a3afa9d252d4d195b96746ea57e9d6dc`.
- Prospective Core hashes: `world.rs` `8965b7cf...`, `world/state.rs` `74c3dabb...`, `node_flow.rs` `d6924d02...`,
  `active_run.rs` `bea18fa6...`, `facility.rs` `35ce3dfe...`, checkpoint validation `d54ebe77...`, facility tests
  `3e13a4c0...` and snapshot/start tests `7885a26f...`.
- Prospective host/canonical hashes: player handler `63afec7f...`, active-run store `a8a7394c...`, runtime canonical
  `6e20d6a6...`, rulebook `6c53ecd5...` and Unity bridge canonical `4d734af0...`.
- Observation: all listed tracked targets are modified and checkpoint validation is untracked. The overlapping dirty
  content substantially predates Phase 20; path status alone cannot identify Phase ownership.
- Impact: every Phase edit must be minimal, hash-tracked and reconciled against activation content. No reset,
  restore, clean, stash, broad formatting or unrelated checkpoint correction is permitted.
- Disposition: baseline gate complete; current production/test/canonical inspection may begin.

## 2026-08-10 — N03 Current Conversion Graph And Lifetime Split

- Source/evidence: complete reads of `world/state.rs`, `world/node_flow.rs`, `world/active_run.rs`,
  `world/uuid_manager.rs`, the F-C01 checkpoint validation suite and the relevant SavePoint/active-run sections of
  the runtime canonical and rulebook.
- Observation: `save_run_checkpoint` manually clones two root power fields, thirteen `RunState` fields and six
  other root fields into the flat 21-field `RunCheckpointPayload`. `restore_run_checkpoint_payload` manually builds
  the inverse `RunState`, restores the root-owned values, preserves the VN-session UUID counter monotonically,
  clears node/session/active-battle transients and enters `ViewingMap`.
- Observation: `combat_recovery_restriction` is the only `RunState` field deliberately omitted and restores as
  `None`. Active battle and tactical/Boss runtime are not payload fields. `loads_used`, run peak, player identity,
  checksum/current/backup and host envelope version remain outside the Core checkpoint document.
- Observation: next-Floor carry-forward is a different lifecycle. It carries only facility state, abnormality
  research/history, monolith and recruitment while deliberately clearing Floor-local caches; it must not reuse the
  checkpoint conversion.
- Observation: F-C01 validation precedes mutation in both explicit load and Continue. Explicit load also enforces
  the live seed and increments the monotonic load count only after successful restore; Continue injects the host
  player identity/new UUID manager and does not consume a load.
- Impact: the safe seam is one private payload capture/restore authority over the existing flat schema. Validation,
  transition, load accounting, Continue bootstrap and next-Floor carry-forward stay at their current callers.
- Disposition: characterize the exact 21-field document and restore-then-capture parity before replacing either
  manual mapping.

### Exact Field And Lifetime Table

| Owner/lifetime | Exact fields | Capture/restore policy |
| --- | --- | --- |
| checkpoint power generation | `power_model_version`, `current_floor_power_anchor` | exact capture and restore; run peak stays outside |
| checkpoint-owned `RunState` | `map`, `map_progression`, `run_progression`, `facility_run_state`, `abnormality_research`, `abnormality_encounter_history`, `monolith`, `boss_omen`, `event_sessions`, `shop_sessions`, `combat_previews`, `abnormality_attempts`, `recruitment` | exact clone/move in both directions through one declared field list |
| checkpoint-owned root state | `visual_novel_trigger_checkpoint_consumed`, `enkephalin`, `inventory`, `roster`, `skill_fragments`, `uuid_manager` | exact restore except UUID keeps the live VN-session counter maximum |
| run/root monotonic or host | `loads_used`, `run_peak_certified_power`, player identity, envelope/checksum/current/backup | absent from Core checkpoint; explicit load/Continue/host retain distinct policies |
| transient/excluded | `combat_recovery_restriction`, active battle/Boss/tactical/event state, active node content/session, game state, active VN, compendium | absent; restore clears recovery restriction, node content/session and active battle while compendium/player/VN bootstrap lifetimes remain outside conversion |
| Floor transition | facility state, research/history, monolith, recruitment plus separately supplied Boss omen | independent carry-forward; never uses checkpoint conversion |

## 2026-08-10 — N04 Consumers And Compatibility Surface

- Source/evidence: Core active-run export/Continue, server player handler, `ActiveRunStore`, facility tests,
  checkpoint validation tests, runtime canonical, rulebook and external Unity Core canonical.
- Observation: Core serializes a private strict top-level `{ version, checkpoint }` V3 document. Server code treats
  the Core payload as opaque, owns strict envelope V3/checksum/current/backup and separately max-merges
  `loads_used`/run peak. Unity sees only projected checkpoint availability/location/load count, never this document.
- Observation: existing code and tests access flat `RunCheckpointPayload` fields directly, so nesting or
  `serde(flatten)` would needlessly alter a Rust/save compatibility seam and may interact badly with strict unknown
  field denial.
- Observation: battle rollback snapshots and next-Floor state are consumers of the surrounding world state but are
  not active-run V3 conversion callers. They must retain their independent lifetime rules.
- Impact: retain the public struct, exact flat serialization, method signatures and server/Unity behavior; add only
  crate-private conversion helpers and transfer the two Core call sites.
- Disposition: no canonical wording change is currently necessary because the canonical already states the exact
  lifetime/ownership boundary this refactor preserves.

## 2026-08-10 — N05 Pre-refactor Characterization Baseline

- Source/evidence: debug filtered test runs before any Phase 20 test or production edit.
- Observation: F-C01 checkpoint validation is 4/4 green; facility/SavePoint/checkpoint coverage is 11/11 green;
  server active-run/handler persistence coverage is 16/16 green.
- Observation: current tests strongly cover invalid graph admission, monotonic load count/peak, UUID continuation,
  Floor anchor replay and host persistence. They do not yet freeze the exact Core V3 key set or exercise a valid
  Continue restore followed by a fresh checkpoint capture and semantic-document comparison.
- Impact: add behavior-green characterization for exact top-level/checkpoint keys, deliberate exclusions and full
  restore-to-recapture parity. No behavioral red is expected or required for this structural candidate.
- Disposition: test-first gate is open; production conversion remains unchanged until the new characterization is
  observed green on the manual implementation.

## 2026-08-10 — N06 Implemented Conversion And Duplicate Removal

- Source/evidence: Phase hunks in `world/state.rs`, `world/node_flow.rs` and the post-change symbol/caller search.
- Observation: a private macro now owns the thirteen-field relationship once and generates the existing public flat
  payload plus crate-private capture/restore methods. The root special cases live in that same conversion impl.
- Observation: SavePoint capture and restore each have one delegation call. Restore transition remains in
  `GameCore`; validation remains before that call in explicit load and Continue.
- Observation: search finds no remaining manual `RunCheckpointPayload { ... }` construction or second inverse
  `RunState { ... }` restore copy outside the generated authority.
- Impact: adding a checkpoint-owned run field now requires one field-list edit and generates both directions while
  the serialization surface remains field-for-field identical.
- Disposition: implementation gate complete; no improvement work, schema change or Phase 21 encapsulation added.

## 2026-08-10 — N07 Final Evidence And Audit Handoff

- Source/evidence: focused and complete debug/release Core/server suites, checks, formatter/diff checks, Clippy,
  static searches, current hashes/status and canonical comparison.
- Observation: characterization is 6/6 debug/release; facility 11/11, snapshot/start 23/23 and server active-run
  16/16 are green. Complete Core is 997 unit + 51 integrations in each profile; complete server is 59/59 in each.
- Observation: strict global Clippy fails only on broad existing warning debt. Scoped non-strict Core output has no
  Phase-line diagnostic; three server warnings are in unchanged pre-existing regions.
- Observation: activation and current expanded status are both 468 entries with identical normalized SHA-256
  `3571680ccb302552f804e18af9681ed9a3afa9d252d4d195b96746ea57e9d6dc`, so no new status path was introduced.
- Current source hashes: `state.rs` `1e5f9177...`, `node_flow.rs` `a311f02b...`, checkpoint validation
  `9d95703b...`. The runtime/rulebook/Unity canonicals retain their activation hashes and already describe the exact
  preserved boundary, so no canonical edit is warranted.
- Observation: a concurrent debug/release full-Core attempt raced on a shared debug trace file; isolated runs are
  fully green. This is recorded as runner orchestration evidence and not concealed as a product pass.
- Impact: P20-C01–C09 have sufficient current evidence. Phase 21 remains uncreated/pending and no future production
  authority was prebuilt.
- Disposition: set the Child to `review_pending`; request independent P20-C10 phase-exit audit.

## 2026-08-10 — N08 Completion Audit Disposition

- Source/evidence: fully reread 289-line `COMPLETION_AUDIT.md`, SHA-256 `49537a4863...`.
- Observation: verdict is `Complete / High`; all P20-C01–C10 pass and no immediate correction, follow-up refactor or
  policy question remains.
- Disposition: mark Phase 20 complete and return activation authority to the Master for the Phase 21 readiness
  protocol.
