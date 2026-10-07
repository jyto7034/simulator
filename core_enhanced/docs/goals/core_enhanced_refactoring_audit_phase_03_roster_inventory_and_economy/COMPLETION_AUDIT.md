# Core Enhanced Refactoring Audit Phase 03 — Completion Audit

Audit Mode: phase_exit
Audit Target: `core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy`
Current Status At Audit Start: active
Implementation Complete Claim: not_applicable
Policy Source: Parent Master CRA-01~CRA-13; frozen `engineering_workflow.md#리팩토링-판단`
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Primary Artifacts: `BOUNDARY_AUDIT.md`, `CANDIDATE_CATALOG.md`
Repository/Commit/Dirty State: `/mnt/f/work/simulator`, start commit
`f318414fb0156a67e04c5479000c2a2683fc0c09`, current dirty working tree

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy evidence | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P3-01 | exact boundary coverage | CRA-01, CRA-03, CRA-08, Phase scope | root ledgers, employee/resources/fragments, direct world commands, checkpoint carrier, live data/canonical/tests and later-domain exclusions reconciled | source/file/count inventory repeated in boundary artifact | pass |
| P3-02 | employee/roster authority trace | CRA-03, CRA-05 | definition ID → employee UUID → roster, condition/profile/growth and recruitment ownership traced | 16 employee + 3 recruitment + 23 starter/snapshot tests | pass |
| P3-03 | inventory/loadout authority trace | CRA-03, CRA-05, CRA-07 | instance/base/static UUIDs, equipment reverse binding, item slot, consumable/material ledgers and all direct transitions traced | 5 inventory + 4 slot + 28 equipment tests | pass |
| P3-04 | fragment/research/currency trace | CRA-03, CRA-05 | fragment ownership/progress/pending/loadout/dust, abnormality research and Enkephalin lifetimes traced | 13 fragment + 2 research + 40 map-flow and selected combat tests | pass |
| P3-05 | transaction atomicity | CRA-05, CRA-07 | grant callers, reward/shop/event/combat/admin stages, HQ rollback and maintenance cost order traced; C02 defect explicit | reward/shop/node/facility and atomic combat failure tests | pass |
| P3-06 | checkpoint/rollback and consumer seam | CRA-03, CRA-08, CRA-09 | strict carrier/direct restore, missing economy graph validation, battle/snapshot trust seam and later-domain limits explicit | node/facility/start/map/rollback tests | pass |
| P3-07 | canonical and test safety net | CRA-02, CRA-03 | live counts/values and canonical atomicity/identity compared with source; mismatch/gaps explicit | retained 201 pass, 0 failed/ignored; physical 291 disclosed | pass |
| P3-08 | candidate/disposition quality | CRA-04~CRA-08, CRA-12 | C01~C05 have work type, axes, authority, invariants, readiness; S01~S14 each has reason/disposition | required-field and signal-row recount | pass |
| P3-09 | dirty preservation/no mutation | CRA-10, CRA-11 | only Goal artifacts changed; existing production dirty work preserved | seven initial/final content/status hashes identical | pass |
| P3-10 | completion audit verdict | Phase completion gate | this audit independently rechecked artifacts, source, data, canonical, counts and hashes | all matrix rows pass | pass |

## Runtime And Ownership Evidence

- `GameCoreState` owns current Enkephalin, inventory, roster, fragment inventory/policy and UUID allocation;
  `RunState` separately owns recruitment and abnormality-research lifecycle.
- Employee definition ID and run-local employee UUID are separate. Recruitment accounts for definition ownership;
  the roster map owns employee instances and current condition/loadout/profile state.
- Equipment definition identity and owned instance UUID are separate. `EquipmentInventory` owns the instance,
  embedded metadata, enhancement and reverse employee binding; the employee item slot repeats the placement
  reference.
- Consumables are definition-keyed quantity stacks with a static UUID selector. Equipment materials, fragment dust
  and Enkephalin are distinct balances with different policies/consumers.
- Normal equip, consumable, death, dismantle, reward, shop and research flows were followed from admission through
  staged mutation and result/checkpoint consumer. Projection was not promoted to authority.

## Transaction And Correctness Evidence

- Reward apply/preview, shop purchase, event grants, combat result and admin grants all stage the same five
  persistent grant ledgers before calling `GrantExecutor`; no current caller commits an early successful effect.
- Shop sale and refresh establish checked next values before committing inventory/session/currency.
- Pending research delivery applies to a clone only at authoritative planning return; invalid entries leave the
  surrounding node/Floor transition unchanged.
- Headquarters' handwritten ten-field rollback is currently behaviorally overlapping with the generic non-battle
  transaction and is C05, not a demonstrated partial-commit bug.
- Enhancement validation permits duplicate same-material cost rows and checks them independently, while the
  handler consumes sequentially from live inventory. This establishes C02's failed-command asset-loss branch even
  though the one current live recipe has only one row.

## Checkpoint And Consumer Evidence

- Strict V3 carries and directly restores Enkephalin, inventory, roster, fragments and UUID state.
- Current validation proves map/facility/progression references, recruitment definition ownership and monolith
  state, but does not prove the internal roster/inventory/loadout/fragment graph.
- Battle equipment setup can consume slot base identity and instance enhancement without proving the inventory
  reverse binding/meta agrees; active-fragment profile application does not repeat ownership validation.
- Snapshot projection exposes inventory and roster binding representations separately. It is evidence of failure
  blast radius, not a repair authority.
- C01 therefore rejects internally inconsistent current-version payloads before mutation. Rebinding embedded
  metadata or repairing/migrating a save remains deferred policy work.

## Live Data And Canonical Evidence

- Live counts are employee 12, equipment 88, consumable 8, reward 13, fragment 31, shop context 3, and one each
  for equipment material, dismantle recipe and enhancement recipe.
- Direct policy/content values observed include starter Enkephalin 500, survival experience 10, one paid refresh
  use, headquarters emergency Enkephalin 120 and `standard_armor` enhancement max 5 at `equipment_dust` 2.
- Canonical agrees with current code on atomic safezone loadout, instance-owned enhancement, separate dust/material
  balances, cloned consumable grant, checkpoint quantity/reserve, planning-return research and staged shop buy.
- Canonical's no-partial-mutation maintenance requirement is violated by C02's schema-valid duplicate-row branch.
- Narrow current recipe coverage is impact-frequency evidence only; it is not used as correctness authority.

## Consumer Evidence

- confirmed: Core world commands, shared grant executor callers, strict active-run carrier, published RON inputs,
  snapshot projection and gated admin grant consumer.
- confirmed supporting only: event and combat staged persistent-asset commit; internal lifecycle/formulas remain
  later Phases.
- confirmed opaque for this boundary: server active-run bytes/results; no server-owned economy ledger was found.
- unknown: other public Rust grant/inventory consumers.
- explicitly excluded/unverified: Unity economy/loadout behavior.

No excluded or unknown consumer is represented as verified.

## Legacy, Fallback, And Migration Audit

- Removed artifact rewards, legacy shop vocabulary and removed consumable forms are explicitly rejected by current
  loaders/tests and are not fallback debt.
- Roster silent replacement is unreachable in valid current production evidence and is retained as a C01/C03
  defensive case, not inflated into a standalone correction.
- The staged grant helper is not labeled a correctness defect because all current Core callers supply rollback.
- Growth state is retained because current no-producer status is intentional serialized state, not deletion proof.
- Metadata rebind/save repair, public API removal, serialized binding removal and recipe/content rebalance are
  deferred migration/compatibility/feature work.
- Equipment material and fragment dust remain separate canonical ledgers.

## Scoped Diff And Dirty Preservation

The following initial/final aggregate SHA-256 hashes are identical:

| Category | Initial | Final |
| --- | --- | --- |
| Core employee/resource domain | `d89334ff357733097d2969039075eeb3af477db7d06dfc9d9f4b5d1de47f7a49` | same |
| Core world economy seams | `8a19fe8687c2790b8547d031f9a6c0965e1a5fbf31c4ca2b3e2b3dc860f6222b` | same |
| typed data/validation | `003119cf844cbf1c0b812ff945b0521cd395bd96df5fbad7f2325678dcfa40dd` | same |
| focused source/integration tests | `521fbd4dca7969311563281b07a31146802e8bb8645a2c62f0bd01124bef8270` | same |
| relevant live RON | `4a951620d6238ea3626da1c1af53da0f363ceb35da1f8fae0f9bdd92a405ef57` | same |
| routed canonical | `87540e3a2a2be57a5382c303d064ca11e0757807ffcc320dbb4e1fa13cc0fc04` | same |
| direct Phase production status | `ad4ec371d2343ac34e605d62e9ab5b933167db661464004766a0348485c379f2` | same |

Only audit-owned Master/Phase Goal artifacts changed. Cargo used ignored build artifacts under repository `target`;
production files and existing user dirty changes were not restored, formatted, moved, deleted or modified.

## Validation Results

- inventory/item-slot/employee/fragment/research: 40 retained pass, 0 failed/ignored.
- equipment command tests: 28 retained pass, 0 failed/ignored.
- recruitment/reward/shop: 11 retained pass, 0 failed/ignored.
- node sessions/facility/start-snapshot: 52 retained pass, 0 failed/ignored.
- live RON/item-skill integration: 27 retained pass, 0 failed/ignored.
- map flow: 40 retained pass, 0 failed/ignored.
- selected combat reward/research/rollback: 3 retained pass, 0 failed/ignored.

Retained evidence totals 201 passed, 0 failed, 0 ignored/skipped across 18 invocations. An eight-filter parallel
batch of 90 tests was rerun after its aggregated output was truncated; physical execution therefore totals 291
tests across 26 invocations. The uncaptured batch is not counted as pass evidence. Filtered counts are nonmatching
tests, not skips.

## Baseline Failures

- No retained test baseline failure was observed.
- Parallel runners waited for Cargo package/build locks and completed normally.
- One eight-filter aggregated result exceeded the capture context. Re-running each filter preserved 90/90 pass
  evidence; the duplicate physical executions are explicitly counted above.
- One source search used nonexistent `crates/game_core/...` paths. `rg --files` established the actual
  `src/game/world/tests/...` paths and the search was rerun; no source or validation evidence remained missing.

## Long-Term Direction Review

| Candidate ID | Work type | Current→target authority result | Preserved invariant result | Validation evidence | Audit verdict |
| --- | --- | --- | --- | --- | --- |
| CRA-P03-C01 | correctness correction | unchecked V3 economy graph → pre-mutation domain validator | valid V3 shape/state, no repair/rebind, load semantics preserved | corrupted-V3 matrix and unchanged-state assertions required first | pass |
| CRA-P03-C02 | correctness correction | per-row live consumption → aggregate/staged atomic cost authority | current single-row recipe/result and exact total cost preserved | duplicate-row insufficient/success tests required first | pass |
| CRA-P03-C03 | behavior-preserving refactor | distributed two-ledger writes → private transition aggregate | both V3 bindings, instance semantics and outcomes preserved | complete transition characterization required | pass |
| CRA-P03-C04 | behavior-preserving refactor | repeated five locals → typed grant transaction state | order, seeds, UUIDs, diffs and outer session lifetimes preserved | caller parity/late-failure matrix required | pass |
| CRA-P03-C05 | behavior-preserving refactor | ten-field HQ rollback → existing non-battle transaction | HQ results/errors and compendium union preserved | full-state rollback parity required | pass |

- Fit: High
- Improvement class: immediate test-first correction recommended for C01/C02; ordered follow-up refactors for
  C03~C05
- Reason: the audit follows current asset authority and all direct commits, proves two specific correction seams,
  distinguishes staged duplication from defects, avoids content/save/external policy invention and preserves dirty
  production.

## Primary Verdict

Complete

## Status Transition

- from: active
- to: complete
- Phase verdict: `Complete / High`
- next Phase activation: allowed after Parent Master registry update

## Remaining Risk And Required Action

- C01/C02 should be implemented under separately authorized test-first correction Goals.
- C03 must follow C01 or otherwise establish the same valid aggregate precondition before transition refactoring.
- C04 must not merge shop/event/combat/session policy or break public API compatibility.
- C05 must retain encounter-compendium monotonic behavior and reject active-battle transaction use.
- Metadata rebind/save repair, content rebalance, public API removal and Unity/external consumers remain
  deferred/unverified.
- Phase 4 must audit session lifecycle independently; this Phase proves only direct asset commit staging.

