# DefenseRoute 전술 지휘 Phase 9 — Experiments

Status: complete

## Baseline and discovery

- [x] Locate setup/scenario/live authoring and validation owners.
- [x] Locate authoritative battle-time scheduling and stale-event pattern.
- [x] Locate all player position/lifecycle mutation and spatial reconcile seams.
- [x] Locate activity/capability, checkpoint/resync/event and server projection owners.

## Focused implementation evidence

- [x] Strict definition and exact live object tests.
- [x] Assignment/activity lifecycle and movement/swap handoff tests.
- [x] Progress, multi-worker, Preserve, Hard CC/Pause no-catch-up tests.
- [x] Exactly-once repair/stale/full-HP completion tests.
- [x] Checkpoint/reconnect/event/server deterministic-shape tests.

## Broad and audit gates

- [x] Core broad suite.
- [x] Server broad suite.
- [x] RON loading suite.
- [x] Workspace all-target check.
- [x] Format and diff checks.
- [x] Canonical/static/compatibility/dirty audit.
- [x] Phase 9 completion audit.
- [ ] Master completion audit — owned by the parent Master after this child closes.

## 2026-08-06 — Definition and live reachability

- Added a strict PvE FieldWork object/cell/presentation/completion schema and scenario conversion/validation.
- Authored exactly one `black_box_emergency_repair` on Freischutz.
- Generated the encounter across 256 seeds and observed all three eligible Ambush/Medium templates; every result exposes
  `(2, 6)` as a valid Ground deployment cell.

## 2026-08-06 — Runtime integration

- Added object-owned progress/runtime generation and real `Ready | FieldWorking` activity.
- Reconciled assignment after deploy, withdraw, death, relocation and atomic two-unit swap final occupancy.
- Focused tests fixed Preserve, same-employee new-instance redeploy, enemy overlap/damage, multi-worker sum, Hard CC and Pause
  no-catch-up, Ground focus/Platform Rest suppression and restoration, stale completion and full-HP consumption.

## 2026-08-06 — Projection and validation

- Added separate setup definition, checkpoint state, semantic events and unit activity/control projection.
- Bumped setup to version 2 and event log to version 31; game_server transports the Core DTO without new work commands.
- Live setup-loss recovery preserves setup, progress, assignment and current revision.
- Final broad result: Core 910 total, server 56, RON 24, workspace check/fmt/diff all pass with no failed or ignored test.
