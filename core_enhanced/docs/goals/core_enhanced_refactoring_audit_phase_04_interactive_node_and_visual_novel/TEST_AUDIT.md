# Core Enhanced Refactoring Audit Phase 04 — Test-Code Audit

Audit Date: 2026-08-07
Policy: Parent CRA-14 and Phase 4 test-code audit policy

Green counts are execution evidence, not the reason for retaining a test. Each disposition below records the
production meaning and replacement safety net.

## Family Dispositions

| Family | Production path and unique invariant | Assertion quality / overlap | Disposition |
| --- | --- | --- | --- |
| `world/event_node.rs` unit tests (5) | exact outcome/chance matrix, equipped-owned key/modifier selection and reason priority used by Event admission/result | direct pure-function assertions; the four-case natural-extreme example is a strict subset of the exhaustive matrix | `retain` matrix/chance/evaluator tests; `merge` the redundant four-case example into matrix diagnostics or remove it after exact equivalence check |
| `world/tests/map_flow.rs` Event group | command/root DTO equivalence, live projection, exact duplicate/no-regrant, stale no-mutation, checkpoint one-roll reuse, living candidates, combat retreat/resume, published branch outcomes and final cleanup | production `GameCore` commands plus live and synthetic fixtures; superficially similar checkpoint tests differ in grant, payload and published-content coverage | `retain`; Phase 10 later owns DTO-shape refactor, but these remain boundary characterizations |
| `world/tests/node_sessions.rs` HQ group | recruitment queue exact ownership, candidate stability, recruit/emergency/shop branch and node cleanup | assertions observe queue/roster/state/result, but no forced late completion failure after staged HQ mutation | `strengthen` with C02 full-state rollback characterization; retain all current unique branch tests |
| `world/tests/node_sessions.rs` Shop group | active entry/exit, persisted page/purchase history, live Standard/Endless merchant selection | current tests observe both active and run state; no assertion that every entry/update uses one paired commit path | `strengthen` with C03 paired-state equality; retain current tests |
| `world/tests/node_sessions.rs` Reward group | active claim/exit, clone-staged multi-effect failure and result diffs | grant-detail tests overlap Phase 3 authority but uniquely connect an active Reward session to result/cleanup | `retain` lifecycle/failure tests; Phase 3 remains effect arithmetic owner |
| `world/tests/facility.rs` SavePoint/Maintenance | recovery/checkpoint and independent maintenance action availability | strong production command/state assertions; broader checkpoint/power tests belong Phase 2 and are supporting only | `retain` Phase 4 lifecycle tests; no merge with HQ/Shop |
| `world/tests/facility.rs` VN trigger group | map-enter activation, priority/id queue ordering, preserved active VN plus rewound trigger history/new UUID/stale rejection | strong top-level state and command assertions; lacks invalid repeat/source publication case | `retain`; `strengthen` data validation under C01 rather than adding another runtime happy path |
| `world/tests/snapshots_and_start.rs` pre-run/VN group | exact RunStart flow without run/map, trigger-owned blocking, choice/advance, background gate, top-level projection and trigger-history checkpoint semantics | production command and snapshot assertions; missing-trigger fixture proves runtime defense but official loader test owns production rejection | `retain`; do not treat fixture-only missing trigger as production loader evidence |
| `data/visual_novel_data.rs` tests (3) | graph speaker activity/branching and removed blocking authority | direct validation errors and positive graph; no overlap with runtime scene progression | `retain` |
| `data/visual_novel_trigger_data.rs` tests (2) plus relevant `live_ron.rs` tests | typed ID/reference and official RunStart count/policy rejection | good provenance assertions; runtime-invalid map/repeat pair absent | `strengthen` with C01 negative matrix and structured live error assertions |
| `world/tests/combat.rs` VN subset | first-update battle-start trigger, threshold crossing once, battle projection and blocking command/tick gate | production battle transport assertions; battle mechanics remain later Phase, trigger/session meaning is unique here | `retain`; do not merge with map-trigger tests because event/timing and transport differ |
| Monolith unit tests | run-state threshold/start/accept/decline and oldest-four consumption | pure state is strong, but no world VN queue/hook/Shop integration exists | `strengthen` with C04 world lifecycle characterization |
| `tests/ron_loading.rs` relevant coverage | official embedded publication and current live graph/reference acceptance | production entrypoint coverage; broad loader ownership was audited in Phase 1 | `retain` as integration gate; add C01 negative case at the closest source-aware unit surface |

## Redundancy Decision

`event_check_outcomes_pin_natural_extremes_and_modified_threshold` has no unique behavioral case: the exhaustive
`outcome_matrix_exhausts_every_face_dc_and_modifier_boundary` executes every raw face for all allowed DC and
modifier boundaries and includes diagnostic context. It is safe to merge/remove only if the surviving matrix keeps
readable failure labels for natural 1, natural 20 and threshold crossing. No other Phase 4 test was proven to have
zero unique contract coverage.

## Missing Safety Nets

1. Publication rejection for every repeat/source pair the current runtime rejects, especially
   `MapNodeEntered + OncePerBattle`.
2. A late HQ recruit/emergency completion failure that compares recruitment, roster, UUID, currency, map,
   checkpoint, active content, node session and game state before/after.
3. Event/Shop entry and update characterization proving persisted session equals the active mirror at every commit.
4. A world Monolith exchange VN test proving front-of-queue ordering, blocking, accept/decline, oldest-four
   consumption and subsequent Shop availability.

These are candidate prerequisites, not reasons to fail the current audit or claims that current happy-path tests
are low quality.

## Runner Evidence

Nine focused invocations executed 126 tests: 126 passed, 0 failed, 0 ignored/skipped. Filters overlap, so this is an
execution count rather than a unique-test count. Exact commands and per-runner results are in `EXPERIMENTS.md`.

## CRA-14 Verdict

- Meaning and production reachability: classified for every assigned family.
- Assertion quality: state/result/identity/no-mutation assertions distinguished from fixture construction.
- Duplication: one exact redundant unit example confirmed; apparent integration overlap retained where live,
  checkpoint, grant, retreat or transport meaning differs.
- Cleanup disposition: every family has `retain`, `strengthen` or `merge`; no unsupported bulk removal remains.
