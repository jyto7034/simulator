# Run Policy Master Final Completion Review

Review date: 2026-07-27

Verdict: **Complete / High**

Implementation complete: true

## Audit Table

| Phase | Audited source of truth | Verdict | Fit |
| --- | --- | --- | --- |
| P00 SavePoint/Gate Trauma | Core recovery/node transaction, live RunPolicy RON, server/Unity non-combat projection | Complete | High |
| P01 failure/checkpoint/active-run | checkpoint payload, monotonic envelope, server active-run transaction, Unity continue/load UX | Complete | High |
| P02 local compendium | Core profile/commit producer, independent host store, root/Unity projection | Complete | High |
| P03 facility map | authored templates, assignment/generator/progression, semantic-slot Unity reader | Complete | High |
| P04 power/difficulty | certified power, run peak/Floor anchor, generation-time difficulty and Unity projection | Complete | High |
| P05 monolith | Elite exclusion, merchant/conversion/result/exchange, Shop guarantee and dormant key persistence | Complete | High |

## Cross-Component Seam Audit

| Question | Direct evidence | Result |
| --- | --- | --- |
| P03~P05 state checkpoint/active-run round-trip | `RunCheckpointPayload`, `RunState`, `ActiveRunDocument`, checkpoint validation/restore tests | Complete |
| monotonic metadata rollback exclusion | active-run envelope carries `loads_used` and run peak outside rewind semantics | Complete |
| compendium separation | Core clone/restore union plus distinct server store/lock/current/backup transaction | Complete |
| generation atomicity | `generate_current_floor_map` manifests anchor then template assignment and node difficulty once | Complete |
| monolith Shop guarantee | `FacilityGenerationRequirements::guarantee_reachable_shop` consumed by generic generator without topology rewrite | Complete |
| exact discovered/defeated commit | actual `UnitSpawned`, victory and `UnitDied` identities; retry/retreat do not commit | Complete |
| strict Unity projection | facility template/version/semantic slot reader; exact NodeConfirm `kind_id`/`category`; strict compendium/checkpoint readers | Complete |
| deferred key non-interference | corrupted key persists with no consumer; existing FinalBoss classification/completion remains unchanged | Complete |

## Direct Implementation Evidence

- P00/P01: `world/node_flow.rs`, `world/state.rs`, `world/active_run.rs`, server
  `persistence/active_run_store.rs` and player handlers own recovery, payload validation, monotonic metadata and atomic
  host publication.
- P02: Core encounter compendium state and combat result producers own set meaning; server
  `encounter_compendium_store.rs` owns its independent lifecycle. Checkpoint restore merges the local profile rather
  than rewinding it.
- P03/P04: `map/facility.rs`, `facility_generator.rs`, `progression.rs`, `combat_power.rs` and live
  `facility_templates.ron`/RunPolicy data own authored topology, assignment, anchor and fixed node difficulty.
- P05: `monolith.rs`, world encounter/shop/combat paths and live policy/shop/PVE data own Elite exclusion,
  conversion attempts, ordered residuals, one-shot Shop guarantee and corrupted-key persistence.
- Unity: `NodeMapSnapshotModel.cs`, checkpoint/compendium readers and their EditMode/PlayMode consumers read Core
  authority; they do not infer topology, eligibility, power or profile state.

## Findings And Corrections

### F-01 — External probe/canonical drift

The external probes and nearby contracts still described Support/Rest, retried random Standard floor-1 starts for a
SavePoint that is authored on `standard_branching_a`, and asserted an older checkpoint summary. They now use exact
Maintenance/SavePoint terminology, explicit run-start shape, authored `save_archive` identity and the current
loadable/location metadata. Both live WebSocket probes pass.

### F-02 — Unity NodeConfirm legacy aliases

`NodeMapSnapshotModel` accepted the removed `node_kind_id` and `node_type` map aliases. The reader now accepts only
`kind_id` and `category`; `NodeMapSnapshotModel_DoesNotAcceptLegacyNodeConfirmAliases` protects the fail-closed
boundary. The focused test and all 259 EditMode tests pass.

### F-03 — Admin SavePoint invalid facility slot

`admin_enter_save_point` appended `slot_id: admin_fixture`. SavePoint completion could create a Core checkpoint, but
the server rejected active-run publication because that slot is outside the authored facility template. The command
now deterministically uses an authored SavePoint-capable facility and its `save_archive` slot, clears stale
floor-local references only when replacing the debug fixture map, and retains canonical `kind_id: save_point`.
The new Core regression validates the completed checkpoint; live save and load publish successfully.

No finding required a new content or balance decision. No hidden-boss production content was added.

## Validation

- Rust format/check: `cargo fmt --all -- --check`, Core check and server check all succeeded.
- Core: library **787 passed**; integration groups **3/4/23/13/4 passed**; failures 0.
- Live RON: integration **23 passed** and filtered live-RON suite **52 passed**; failures 0.
- Server: **52 passed**, failures 0.
- Focused seams: **44 passed**, including facility, generator, monolith, compendium, active-run, server persistence
  and the new admin SavePoint checkpoint regression.
- Live WebSocket: admin non-combat scenario passed; authored SavePoint saved at **0/3** loads and restored at
  **1/2** with exact `standard_branching_a/save_archive/save_point` identity.
- Unity MCP `ark@b49d32f6f2f70bbe`, Unity 6000.3.4f1: compile errors **0**; EditMode job
  `c948fec856c24644ae75b52ace695fdf` **259/259 passed**; PlayMode job
  `0ff24f7b419c45d1a25a264171a30560` **62 passed, 4 explicit live opt-in skipped, 0 failed**.
- Python probe syntax and scoped `git diff --check` succeeded.

## Final Verdict

- Primary verdict: **Complete**
- Long-term fit: **High**
- Improvement class: **none required for the current master**
- Remaining risk: economy and combat coefficients still need future playtesting. The hidden-boss actor, encounter,
  skill, administrator VN, capstone, FinalBoss reclassification and special completion remain an explicitly separate
  future content Goal and are not prerequisites for this verdict.
