# Phase 08 CRA-P07-C01 Experiments

Status: complete
Implementation Complete: true

## 2026-08-09 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 07 CRA-P06-C05 has a current `Complete / High` audit with improvement class `none`.
- Phase 08 is next in the exact registry; Phases 09~44 remain pending and unauthorized.
- No candidate-specific policy ID applies. CRA-POL-006 limits the correction to internal tactical ownership with
  current RON/DTO/wire/public/checkpoint compatibility.

### Evidence Carried Into Initialization

- CRA-P07-C01 remains a correctness correction with `3/2/3/3/3/2/unknown = 16/18`.
- Historical defect: relocation/swap applies tile Focus/block/Rest before field-work reconciliation, so an
  `Any`-profile worker crossing Ground/Platform can expose effects incompatible with its final activity or receive
  duplicate Focus arithmetic.
- Required first proof: Any worker cross-kind relocation and swap must assert final activity, stance, Focus, block,
  Rest epoch/deadline, stale-event behavior and unrelated modifier preservation for exit/remain assignment cases.
- Preservation boundary: existing activity/stance/tile semantics, assignment/progress, resource/cooldown/revision,
  live data and all public/serialized shapes.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 08 coordination changes from `ready` to `active`; no other Child is active.
- Phase 08 production status/source/data investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents completely, capture expanded dirty status and prospective target hashes, then trace
current tactical effect state/mutations/snapshots/events and establish nonzero focused baselines.

## 2026-08-10 — E01 Dirty Baseline, Authority Trace And Focused Baseline

### Dirty Baseline

- Repository commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded status after Child initialization: 408 entries; status SHA-256
  `9771d27a9a9864ded7f2b5c8a19fdc24dfa9c504419a39677c67d9fe93d08c07`.
- The four likely Core targets are pre-existing untracked user files:
  - `tactical_stance.rs`: `1faca2dbc11d888619a9eb8ecbb1e60ce20fec4694c677da395b4c47f550235f`
  - `tactical_relocation.rs`: `ed0103d839393c59c422563cfa0cc0c12d2cb28d9ffef86d413d7425f4faf48d`
  - `tactical_swap.rs`: `2ed2240d4ca76fae27f55d1947975dd691e5e86149a3f017ba30f66dd9264929`
  - `field_work.rs`: `1bc178e2e4030706cd835b0f02a1c9dc8cc3abe04c7c38e25a00f62b5228a180`
- `world/tests/combat.rs` and `core_runtime_contract.md` are also pre-dirty and fingerprinted. The correction must
  preserve these user-owned files and keep every new hunk separable.

### Current Authority Trace

- `unit_tactical_snapshot` independently derives intended `None | DefenseFocus | Rest` from activity, stance and
  tile, then reads mutable block/deadline state produced elsewhere.
- `set_engagement_stance` directly changes Focus aggregate, block, Rest epoch/deadline and queue before publishing
  the stance event.
- `apply_tactical_position_update` directly removes/adds Focus by old/new tile, replaces block and updates Rest from
  tile change before relocation/swap calls `reconcile_field_work`.
- `transition_field_work_activity` separately adds/removes Focus, block and Rest while setting final activity/Defense.
- `reconcile_field_work` transitions only worker-set differences. A unit reassigned from one work cell to another
  remains in both sets, so no activity transition repairs the earlier tile-only effect mutation.
- `handle_rest_tick` correctly gates by epoch, final Defense+Platform+Ready and exact deadline, but a split mutation
  can still expose an invalid deadline until then.

### Nonzero Baseline

| Command family | Result |
| --- | --- |
| tactical stance/relocation/swap Core | 10 passed, 0 failed |
| field-work Core | 5 passed, 0 failed |
| live tactical relocation/swap world | 2 passed, 0 failed |
| live stance world | 1 passed, 0 failed |
| live field-work world | 1 passed, 0 failed |

### Next Experiment

Add accepted `Any`-profile cross-kind cases for relocation remain/exit, swap remain-assigned and Rest→field-work
entry. Assert final activity/effect, exact modifier/block, one epoch transition/deadline and stale Rest no-op before
introducing shared reconciliation.

## 2026-08-10 — E02 Any-Worker Cross-Kind Red Proof

### Added Characterization Cases

- Ground work cell → Platform work cell relocation with the same assigned `Any` worker.
- Platform work cell → non-work Ground relocation, requiring one final DefenseFocus application after work exit.
- Ground/Platform work-cell swap where both `Any` workers remain assigned but exchange tile kinds.
- Ready+Defense Platform Rest → Ground work relocation, requiring one epoch invalidation and stale Rest no-op.
- Every case seeds unrelated general/type-specific damage reduction and asserts the complete final tactical snapshot.

### Pre-Fix Result

| Exact filter | Result | Named failure |
| --- | --- | --- |
| field-work `any_` cases | 0 passed, 3 failed | relocation/swap exposed `Some(2500)` Rest while FieldWorking; work exit produced reduction `27` instead of `17` |
| stale Rest entry case | 0 passed, 1 failed | epoch became `3` instead of one invalidation to `2` |

The failures prove both historical shapes: tile-only mutation survives when the worker set is unchanged, and an
activity transition can repeat arithmetic/scheduling already performed by position update.

### Next Experiment

Derive one typed tactical effect from full state, make snapshot and mutation share it, defer moved-unit effect
application until final field-work occupancy is known, and reconcile old→final exactly once.

## 2026-08-10 — E03 One Final-State Tactical Effect Reconciliation

### Production Change

- Added one private `DerivedTacticalEffects` value derived from final activity, stance, tile kind and weapon profile.
  It owns the intended tile effect, Focus contribution and exact block capacity; tactical snapshot uses the same
  derivation.
- Added one `reconcile_tactical_effects` transition. It changes only a different Focus contribution, always installs
  final block, and changes Rest epoch/deadline only when Rest active membership changes.
- Stance now commits the logical stance/cooldown before invoking the transition.
- Position update now commits body/tile/facing/cooldown only. Relocation/swap capture previous effects, defer moved
  units during field-work activity reconciliation, then apply each moved unit's previous→final effects once.
- Ordinary field-work entry/exit sets final activity+Defense first and then invokes the same transition. A worker
  that stays in the assignment set is still reconciled by its relocation/swap owner.

### Exact Green Proof

| Command family | Result |
| --- | --- |
| new Any-worker relocation/swap cases | 3 passed, 0 failed |
| new stale Rest entry case | 1 passed, 0 failed |
| tactical stance/relocation/swap Core | 10 passed, 0 failed |
| field-work Core | 9 passed, 0 failed |
| live tactical relocation/swap world | 2 passed, 0 failed |
| live stance world | 1 passed, 0 failed |
| live field-work world | 1 passed, 0 failed |

### Compatibility And Canonical

- No public type, serialized field, policy/live RON, checkpoint, server or Unity shape changed.
- The nearest runtime canonical now states full-final-state derivation, moved-worker deferral, one reconciliation,
  Rest membership epoch semantics and unchanged same-effect cadence.

### Next Experiment

Run formatting/static inventories, affected tactical/field-work/movement and live-RON families, complete Core and
server compile; then capture final dirty hashes and conduct the independent completion audit.

## 2026-08-10 — E04 Broad Validation And Dirty Preservation

### Broad And Static Validation

| Check | Result |
| --- | --- |
| live RON family | 63 passed, 0 failed |
| complete Core library | 948 passed, 0 failed, 0 ignored, 0 filtered |
| game_server `cargo check --all-targets` | pass |
| `cargo fmt --check` | pass |
| repository `git diff --check` | pass |
| Focus/block/Rest write inventory | runtime transition writes consolidated in `reconcile_tactical_effects`; deploy baseline and Rest continuation remain distinct lifecycle owners |
| policy/data/public-shape inventory | no RON, policy, checkpoint, DTO/wire/public Rust or server shape change |

### Dirty Preservation

- Expanded status remains 408 entries with the same SHA-256 as activation,
  `9771d27a9a9864ded7f2b5c8a19fdc24dfa9c504419a39677c67d9fe93d08c07`; no path was added, removed or had its
  tracked/untracked classification changed during production work.
- Final hashes of the four pre-existing untracked Core targets:
  - `tactical_stance.rs`: `d6397d834d058f43b121f424b9289dfbde89f555292acaea23e47884e50ca908`
  - `tactical_relocation.rs`: `f61632ba763c93fb40d30a3f9aec7c6ff79a4726295d0dfbf4251afe5b2d55fb`
  - `tactical_swap.rs`: `96658a9fe1d98b5d69dffb9094d8c9ddf5fa40a985dda6a82e63002d9a73a7c0`
  - `field_work.rs`: `2229921f996084fa43d2aaffabdf4c96e5c1f7873e1af7b7f5367e17552f4a60`
- Final pre-dirty canonical SHA-256:
  `eb5fbf7dca54b9720ccaab9bc20f1c8f4557aa8f6e057b7a2ef220586233bcb6`.
- No reset, restore, clean, stash, deletion, move, broad formatting or live-data mutation was performed.

### Next Experiment

Re-read the completion-review guide completely, independently audit runtime/data/DTO/persistence/consumer,
legacy/static ownership, canonical and dirty preservation, then write `COMPLETION_AUDIT.md`. Do not activate Phase
09 unless the verdict is `Complete / High`.

## 2026-08-10 — E05 Independent Phase Exit Audit

- Re-read all 567 lines of `goal_completion_review_guide.md` before the audit pass.
- Independently checked current runtime transition/callers, Any validation/live data, DTO/world/server projection,
  active-battle persistence boundary, legacy writes, canonical and pre-existing untracked-file preservation.
- Audit artifact creation moved expanded status from implementation-final 408 to audit-final 409 entries; final
  status SHA-256 is `6bcff381a8e54bc9464907256fc3b4f7211676681988dacd304729f2599031cb`.
- `COMPLETION_AUDIT.md` records every matrix row pass, primary verdict `Complete`, fit `High` and improvement class
  `none`.
- Phase 08 status changes to `complete`; Master may make Phase 09 ready.
