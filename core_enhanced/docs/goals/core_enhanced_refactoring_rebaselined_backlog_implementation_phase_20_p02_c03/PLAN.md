# Phase 20 Goal — CRA-P02-C03 Consolidate RunState And V3 Checkpoint Conversion

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P02-C03
Master Phase: 20 of 44
Work Type: behavior-preserving refactor
Production Mutation Authorized: yes — the user's Master execution request remains active; authorization is limited
to this Phase after Phase 19's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Replace the repeated private RunState→current-V3 checkpoint capture and V3 checkpoint→RunState restore field copies
with one typed private conversion authority. The exact current V3 serialized field set, every persisted value and
every deliberate exclusion/lifetime must remain unchanged. F-C01's strict pre-mutation admission graph remains the
gate before conversion or restore.

This Phase is complete only when the current capture/serialize/validate/restore/Continue/explicit-load graph and
every run-owned/root-owned/excluded field are inventoried; golden tests first freeze the exact V3 semantic field set,
bidirectional value parity and exclusions; one private checkpoint conversion owns the mapping; duplicate private
copies and only those copies are removed; active battle/Boss state, root monotonic identities, next-Floor
carry-forward and fixture/public seams retain their distinct lifetimes; invalid documents still fail before mutation
through F-C01; Core/server/public/save/wire behavior remains exact; focused and broad debug/release validation passes;
and an independent phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical and re-audited evidence found correct behavior but hand-maintained inverse mappings between active
`RunState` and the strict V3 checkpoint payload. A newly persisted run field can be omitted from capture or restore
unless multiple field lists are updated together. Separate next-Floor carry-forward logic looks similar but owns a
different lifetime and must not be generalized into checkpoint persistence.

This is change-amplification and omission risk, not a current save defect. The target is private conversion
consolidation with zero V3, restore, runtime, host or consumer behavior change.

## Dependency And Policy Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 20 registry row.
- Direct linear predecessor: Phase 19 CRA-P01-C03, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_19_p01_c03/COMPLETION_AUDIT.md`.
- Correctness dependency: Phase 02 CRA-F-C01, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_02_f_c01/COMPLETION_AUDIT.md`.
- Approved compatibility policy: CRA-POL-006 in
  `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`.
- Original evidence:
  - `../core_enhanced_refactoring_audit_phase_02_run_map_and_persistence/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_02_run_map_and_persistence/BOUNDARY_AUDIT.md`
- Re-audited evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/BOUNDARY_AUDIT.md`
- Final order/test/gate evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

CRA-POL-006 fixes the current checkpoint as a strict current-version document. It permits private mapping
consolidation but does not approve a version bump, field deletion, old-version reader, repair/rebind, active battle
or Boss persistence, public API narrowing or host-envelope change.

## Priority And Current Disposition

- Priority axes: `2 / 1 / 2 / 3 / 2 / 3 / unknown`; known subtotal `13/18`.
- Re-audited disposition: `still_valid`, `behavior refactor; golden parity first`.
- Dependency disposition: F-C01 is already `Complete / High`; invalid seed/progression/asset/economy graphs now fail
  before restore, so conversion consolidation cannot hide an admission gap.
- Current live-tree disposition: discovery pending until all three Child documents and the complete final Phase 19
  audit are fully read and the scoped dirty baseline refreshes historical source/test/canonical targets.

## Current And Target Authority

- Historical current authority: active `RunState` owns live run/Floor state; the checkpoint payload is a derived
  restorable V3 representation; private capture and restore manually enumerate inverse run/root mappings.
- Target authority: one private typed checkpoint-owned aggregate or explicit conversion pair owns the bidirectional
  field relationship once while existing writer/validator/restore entrypoints retain their responsibilities.
- F-C01 remains the sole contextual pre-mutation admission graph. Host envelope/checksum/current-backup and monotonic
  metadata remain server-owned and the Core payload remains opaque to the server.
- `RunState::for_next_floor_from` and any current successor retain their separate carry-forward lifetime.
- Exact current types, field counts, call graph, visibility and error order must be rediscovered only after the
  mandatory Child/predecessor read and dirty-baseline gate. Historical counts are evidence, not edit instructions.

## In Scope

- inventory SavePoint capture, Core V3 export/import, F-C01 validation, Continue, explicit checkpoint load, restore,
  rollback and host opaque-payload consumers;
- enumerate every active run field, root field, V3 wrapper/payload field and deliberate exclusion with owner and
  lifetime;
- capture expanded dirty status, scoped diffs and target SHA-256 fingerprints before production inspection/edits;
- add test-first golden semantic JSON field-set/exclusion coverage and full capture→restore→recapture value parity;
- prove active battle/Boss/tactical/event-queue state remains excluded and restore clears the same transient state;
- prove player identity, compendium, run peak, load count, VN/UUID monotonic/checkpoint identities and
  `combat_recovery_restriction` retain exact current behavior;
- introduce the smallest private typed conversion authority and route both capture and restore through it;
- remove only the proven duplicate private bidirectional mapping;
- update the nearest checkpoint/runtime canonical only if internal ownership wording materially changes.

## Out Of Scope

- changing V3 version, serialized field names/types/defaults/tags, JSON meaning, host envelope or checksum;
- adding/removing fields, aliases, dual read/write, old-version migration, document repair/rebind or fallback;
- persisting active battle, Boss runtime, tactical state, event queue, server actor/session or transport identity;
- changing F-C01 validation rules/order/errors, explicit-load/Continue seed semantics, load-count timing, run peak,
  UUID/VN identity behavior, rollback, retry, recovery, SavePoint or last-completed-checkpoint meaning;
- merging `RunState` with the persisted payload or merging checkpoint conversion with next-Floor carry-forward;
- Phase 21 P02-C04 progression mutation/availability encapsulation or direct BossOmen/admin changes;
- cross-actor lease/conflict policy, admin durability policy, public method cleanup or external migration;
- broad formatting, data rewrite or dirty-worktree normalization.

## Preserved Invariants

1. The current Core payload remains strict V3 with exact order-insensitive serialized meaning and field set.
2. Every currently checkpoint-owned run/root value round-trips exactly through capture, export, import and restore.
3. Every deliberately excluded field/state remains excluded and retains its current reset/preservation lifetime.
4. F-C01 rejects invalid current documents before destination mutation; no conversion bypass or repair exists.
5. Explicit load and Continue preserve their distinct seed, load-count, transport-player and rollback semantics.
6. Active battle/Boss/tactical runtime state is not serialized and is cleared/restored exactly as today.
7. Next-Floor carry-forward remains a separate lifecycle and is not widened to checkpoint persistence.
8. Host V3 envelope, checksum, backup/tombstone and monotonic fields remain server-owned and unchanged.
9. Public Rust, server, DTO/wire, Unity and fixture/tool contracts remain exact under CRA-POL-006.
10. Pre-existing user changes are preserved without reset, restore, clean, stash or broad unrelated formatting.

## Implementation And Dirty-Worktree Strategy

After all three Child documents exist and Master records sole activation, read them completely and reread the full
Phase 19 completion audit. Capture expanded status plus prospective source/test/canonical hashes before detailed
production inspection. Treat every existing hunk as user/campaign-owned until reconciled.

Use characterization-first proof in this order:

1. Trace all capture/export/import/validate/restore/load/Continue/rollback/host entrypoints without editing them.
2. Build an exact lifecycle table for current run fields, root fields, V3 wrappers and deliberate exclusions.
3. Run current checkpoint/facility/active-run/server baselines and record nonzero counts.
4. Add a semantic golden V3 field-set/exclusion test and exhaustive bidirectional value-parity controls before
   refactoring; current correct behavior must pass.
5. Introduce one private typed conversion seam downstream of F-C01 validation.
6. Route capture and restore through it and remove only inverse mapping duplication proven transferred.
7. Re-run focused, complete Core debug/release, server persistence/actor debug/release and static/compatibility gates.

This is behavior-preserving, so an intended behavioral red is not required. Structural completeness evidence may
fail only if it directly proves duplicated mapping authority; test setup/serialization noise is not policy red.

## Migration, Deployment And Compatibility

- No migration, compatibility interval or coordinated rollout is approved or needed for a private-only refactor.
- Core V3 payload bytes may differ in object-key ordering only where JSON semantics are already order-insensitive;
  semantic field names/types/values and strict reader behavior must remain exact.
- Server continues to persist opaque Core JSON inside the unchanged host envelope.
- Unknown deployed Rust/save tooling is bounded by exact public/V3 compatibility; no negative search authorizes
  removal or narrowing.
- If consolidation requires any field/version/error/lifetime/public change, stop as `policy_decision_required`
  rather than widening this Phase.

## Validation Plan

- Baseline: valid V3 round trip, F-C01 checkpoint validation, SavePoint/facility load, active-run Continue/export,
  node-session/start-snapshot checkpoint behavior and server active-run store/actor families.
- Test-first golden: exact top-level and nested V3 field sets, every current run/root value, active-battle/Boss and
  host/root exclusions, capture→restore→recapture semantic equality.
- Failure order: representative invalid seed/progression/equipment/fragment payloads still reject before state,
  map, UUID, load-count or export mutation.
- Lifecycle: explicit load versus Continue, UUID/VN identities, load count/run peak, last SavePoint and next-Floor
  carry-forward remain distinct.
- Broad: complete Core debug/release, official integrations and complete game server debug/release.
- Compatibility/static: public/API and serde inventory, exact golden JSON, caller/obsolete-copy search, scoped
  status/diff/hashes, formatter and diff check.
- Zero-test filters, compile-only success, order-sensitive raw JSON text and snapshots without semantic field/value
  assertions do not count as sufficient proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P20-C01 | complete capture/restore/consumer graph and field lifecycle identified | exact owner/lifetime table | complete |
| P20-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diff and hashes | complete |
| P20-C03 | current V3 behavior characterized before refactor | golden field/exclusion and bidirectional parity tests | complete |
| P20-C04 | one private checkpoint conversion authority exists | source/state/caller inventory | complete |
| P20-C05 | duplicate inverse field mapping is removed | obsolete-copy and call-site audit | complete |
| P20-C06 | exact V3 shape and all current values/exclusions remain | semantic JSON and lifecycle table | complete |
| P20-C07 | F-C01 admission and failure atomicity remain exact | negative provenance/no-mutation matrix | complete |
| P20-C08 | runtime/host/public/save/wire lifetimes remain exact | Continue/load/server/consumer evidence | complete |
| P20-C09 | focused and broad validation is green | nonzero debug/release/Core/server/static counts | complete |
| P20-C10 | canonical/downstream boundary and audit are current | no-prebuild audit and `Complete / High` review | complete |

## Stop Conditions

Stop and do not activate Phase 21 if any serialized field/version/value/exclusion or restore lifetime must change;
F-C01 would be bypassed or reordered; next-Floor and checkpoint lifetimes cannot remain distinct; public/server/save
consumers require migration; user-owned dirty overlap cannot be separated; required tests select zero cases; or the
independent audit is below `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
