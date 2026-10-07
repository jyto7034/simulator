# Core Enhanced Refactoring Audit Phase 07 — Experiments

Status: complete

## 2026-08-07 — Activation gate

### Evidence and result

- Parent Master, Phase 0 boundary and Phase 6 completion audit were rechecked.
- Direct predecessor Phase 6 is `Complete / High`; the Phase 7 reserved path did not exist.
- Phase 5 setup, Phase 6 clock/order/advance and Phase 8~10 carrier boundaries were identified.
- No other active child exists. Phase 7 is investigation-only and production mutation remains prohibited.
- Exact start commit remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.

### Tests

No runner was executed during activation.

### Decision

Phase 7 may perform read-only production investigation and write only audit artifacts. Phase 8 remains pending.

## 2026-08-07 — Scoped production baseline

### Exact categories and initial hashes

| Category | Initial SHA-256 |
| --- | --- |
| battlefield + tile geometry (3 files) | `70c1fa3039c93cdc6530973f43f827463bee59cda293d82e1111819a64a82878` |
| continuous movement (9 files) | `fdab82a00e459bbce99d9a2091ae0bd97ed74f9d175d9d42b7676733eee1ffd5` |
| spatial/tactical core (5 files) | `30715eff8d173e6788e3f27814d313e702c14fce21e43f3e8302bf3c964d6ecc` |
| shared battle/resource carriers (8 files) | `59c9dcaf7256a4031496cd30499cbc1fc4d2d7601ec4d12c460b1c6636f740dd` |
| world/range carriers (3 files) | `3d7b7702a3075edca0c6d9cf39318ff62d7c265f81c2b607418e79d9b51e41dd` |
| behavior/policy + live RON (3 files) | `2e70e01b6988c6919d7f4e713caa6b36ea255d4700976d49589df10be30334df` |
| focused test source (1 file) | `87861813beff818f37e914cb3e25515e4b324508225711feb6c43fd03eae53ef` |
| nearest canonical (2 files) | `1c606fd56802a360a5451fce556f377f095430b6334df1e8707184d83355dae7` |
| scoped Git status text | `288d1bf0512cbbdc88f13b5b30caf0a18bc2a0a99f177468a35c15b2ea56a1dc` |

Battlefield paths are `battle/battlefield/{field,mod}.rs` plus `battle/tile_range.rs`; movement is all nine files
under `battle/core/movement/**`; spatial/tactical core is
`core/{field_work,spatial,tactical_relocation,tactical_stance,tactical_swap}.rs`. Shared carriers are
`core/{build,commands,mod,sim,types}.rs`, `battle/{scenario,types}.rs` and `resources/mod.rs`. World/range carriers
are `world/{combat,state}.rs` and `range_preview.rs`.

### Dirty-state result

Twenty-three scoped production/data/test/canonical paths were already modified or untracked, including all four
new tactical command modules. These are user-owned baseline changes.

### Tests

No runner was executed for the baseline.

## 2026-08-07 — Static spatial/tactical gaps before focused validation

### Evidence and result

- `apply_tactical_position_update` changes Ground focus reduction and block from old/new tile kind alone, without
  considering `UnitExclusiveActivity::FieldWorking`. A working Ground unit has already had Focus removed; moving it
  to Platform subtracts Focus a second time before field-work exit, while moving a working Platform unit to Ground
  adds Focus before exit and `transition_field_work_activity(false)` adds it again. If the unit remains assigned in
  a cross-kind work cell, the helper also exposes a non-`None` Platform Rest deadline while activity is still
  `FieldWorking`; the Rest event later no-ops only because its handler rechecks activity. Swap uses the same helper. The
  canonical explicitly permits field-working relocation/swap and requires `Ready + Defense` effects to be derived
  once from final occupancy. Existing field-work relocation/swap coverage uses Ground destinations only and asserts
  the tactical snapshot enum, not the aggregate incoming-damage modifier or unrelated modifier preservation.
- Every live mutating deploy/withdraw/stance/relocation/swap handler commits Core state, resource/mapping changes and
  the battle revision before `battle_update_dto_after(after_seq)` rejects a future cursor. Deploy/withdraw also call
  fallible employee-attempt reconciliation after Core mutation. This violates the active-battle contract that any
  failed mutation preserves state/resource/cooldown/event/revision. Current future-seq coverage exercises only the
  read-only `RequestBattleState` path; ordinary tactical atomicity tests cover earlier domain preflight rejection.
- Production `BattleCore` always defaults to the Rapier movement backend. The complete Direct collision/depenetration
  implementation and its backend switch are test-only at Core level; one authored-route integration test explicitly
  switches to Direct, so it does not protect production physics. Three parity tests and many Direct-only tests maintain
  a second collision authority despite no production selection path.
- The movement model exposes unused authority-shaped state: `MovementTickInput.now_ms` is discarded; `GoalAssigned`
  and `SegmentStarted` outputs are never produced; `UnitBody.mode` and `physics_handle` are never read; and stored
  `previous_position`, `velocity` and `goal` do not drive production movement. Core clones the body and overwrites its
  goal from the current planner map before each tick. Public Rust compatibility remains unknown, so removal requires
  a consumer inventory even though workspace production has no semantic consumer.
- Preview topology validation unions deployment-zone cells but does not reject the same cell in multiple zones;
  world lookup selects the first matching zone kind. Official ASCII template construction makes current live Ground
  and Platform cells disjoint, so no current live-data failure is established. This is a Phase 5 preview/publication
  validation carrier rather than a duplicate Phase 7 runtime candidate.
- Battlefield valid/static topology, player logical occupancy from authoritative runtime bodies, deterministic block
  refresh, Pause-aware battle time, tactical resource recovery/payment and field-work generation scheduling otherwise
  have one coherent runtime authority in the inspected paths. Effect/stat and target/delivery interpretations remain
  Phase 8/9 carriers.

### Focused validation reason

Run the smallest existing battlefield, movement, stance, field-work and world tactical families to establish current
behavior and test counts. Green results cannot close the missing cross-tile FieldWorking modifier assertion, mutating
future-cursor transaction, production-backend integration gap or ghost movement model recorded above.

## 2026-08-07 — Focused validation

| Command/filter group | Passed | Failed | Ignored | Meaning |
| --- | ---: | ---: | ---: | --- |
| battlefield field + tile range + movement types | 8 | 0 | 0 | topology, rotation and world/data conversion |
| movement engine + planner + Rapier backend | 59 | 0 | 0 | current Direct/Rapier, routing, block and collision behavior |
| spatial + tactical stance + field work | 19 | 0 | 0 | geometry and ordinary tactical/activity transitions |
| tactical run-policy + resource state | 7 | 0 | 0 | positive bounds, saturation, recovery and payment |
| authored-route integration exact test | 1 | 0 | 0 | route progress, currently forced through Direct |
| nine exact world spatial/tactical tests | 9 | 0 | 0 | movement DTO, field work, deploy/Pause/cursor and tactical commands |

Twenty-three valid invocations executed 103 passing tests. Two parallel exact world invocations yielded while still
waiting for build serialization and did not return terminal test summaries; they were discarded and rerun serially,
each executing one passing test. No scoped `#[ignore]` marker exists. Green execution does not cover the static gaps
recorded before the runner.

## 2026-08-07 — Candidate and CRA-14 disposition

- Confirmed C01 full-state tactical-effect derivation and C02 atomic tactical world mutation as correctness
  candidates; both require failing late-boundary tests first.
- Confirmed C03 production-only Rapier backend and C04 minimal movement model as cleanup candidates with external
  Rust compatibility gates.
- Identified 17 Direct-dependent test bodies and two test-only Rapier board-wall tests for assertion transfer/removal;
  retained production Rapier, route/block, resource, Pause and field-work scheduling families.
- Classified deployment-zone ambiguity as a Phase 5 publication handoff and invalid TestFixture Core deploy as
  unverified hardening rather than overstating current live reachability.

## 2026-08-07 — Final no-mutation proof

All final hashes equal their initial values:

| Category | Initial SHA-256 | Final SHA-256 | Result |
| --- | --- | --- | --- |
| battlefield + tile geometry | `70c1fa3039c93cdc6530973f43f827463bee59cda293d82e1111819a64a82878` | `70c1fa3039c93cdc6530973f43f827463bee59cda293d82e1111819a64a82878` | identical |
| continuous movement | `fdab82a00e459bbce99d9a2091ae0bd97ed74f9d175d9d42b7676733eee1ffd5` | `fdab82a00e459bbce99d9a2091ae0bd97ed74f9d175d9d42b7676733eee1ffd5` | identical |
| spatial/tactical core | `30715eff8d173e6788e3f27814d313e702c14fce21e43f3e8302bf3c964d6ecc` | `30715eff8d173e6788e3f27814d313e702c14fce21e43f3e8302bf3c964d6ecc` | identical |
| shared battle/resource carriers | `59c9dcaf7256a4031496cd30499cbc1fc4d2d7601ec4d12c460b1c6636f740dd` | `59c9dcaf7256a4031496cd30499cbc1fc4d2d7601ec4d12c460b1c6636f740dd` | identical |
| world/range carriers | `3d7b7702a3075edca0c6d9cf39318ff62d7c265f81c2b607418e79d9b51e41dd` | `3d7b7702a3075edca0c6d9cf39318ff62d7c265f81c2b607418e79d9b51e41dd` | identical |
| behavior/policy + live RON | `2e70e01b6988c6919d7f4e713caa6b36ea255d4700976d49589df10be30334df` | `2e70e01b6988c6919d7f4e713caa6b36ea255d4700976d49589df10be30334df` | identical |
| focused test source | `87861813beff818f37e914cb3e25515e4b324508225711feb6c43fd03eae53ef` | `87861813beff818f37e914cb3e25515e4b324508225711feb6c43fd03eae53ef` | identical |
| nearest canonical | `1c606fd56802a360a5451fce556f377f095430b6334df1e8707184d83355dae7` | `1c606fd56802a360a5451fce556f377f095430b6334df1e8707184d83355dae7` | identical |
| scoped Git status text | `288d1bf0512cbbdc88f13b5b30caf0a18bc2a0a99f177468a35c15b2ea56a1dc` | `288d1bf0512cbbdc88f13b5b30caf0a18bc2a0a99f177468a35c15b2ea56a1dc` | identical |

Only Master/Phase audit artifacts changed. Cargo produced ignored `target` artifacts; no production/data/test/
canonical/server/Unity file was edited, formatted, moved or deleted.
