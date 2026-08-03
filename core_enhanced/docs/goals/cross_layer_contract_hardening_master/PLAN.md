# Cross-Layer Contract Hardening Master Goal

Status: complete

Implementation complete: true

Created: 2026-07-11

Last updated: 2026-07-14

Goal type: orchestration master; this goal does not directly implement runtime, data, server, or Unity behavior.

## Objective

Coordinate the long-term repair of confirmed runtime, live-data, Core-server-Unity contract, test, and documentation findings without turning them into one unbounded implementation goal.

This master owns:

- a stable finding registry,
- one primary child goal owner for every finding,
- policy-decision gates,
- child-goal dependencies and execution order,
- per-child completion review gates,
- per-Phase completion reviews and correction loops,
- final cross-goal validation and closure evidence.

This master does not own:

- production implementation details,
- child-level trial and error,
- durable gameplay or wire policy,
- compatibility schemas,
- direct edits to Core, live RON, game_server, or Unity.

Each child goal owns one source-of-truth seam or one tightly coupled behavior boundary. A child goal is created only when work on that boundary actually starts.

## Normalized Scope

Two audit sources feed this master.

1. DCA: [documentation_code_alignment_audit_2026-07-10.md](../../archive/audits/documentation_code_alignment_audit_2026-07-10.md)
   - 52 raw findings.
   - Revalidated on 2026-07-11.
   - 51 actionable or policy-bearing findings.
   - DCA-M-26 is retained as an explicit excluded finding.
2. CPR: [comprehensive_project_review_2026-07-10.md](../../archive/audits/comprehensive_project_review_2026-07-10.md)
   - 15 findings.
   - 9 overlap DCA findings.
   - 6 are unique and require separate primary ownership.
3. Master review/correction findings:
   - 7 findings were confirmed after creation of the master.
   - MSTR-01 records the fixed-60%-HP redeploy implementation overriding the intended updated-trauma policy.
   - MSTR-02 records the live, player-impacting Artifact subsystem that has no canonical rulebook identity and is not surfaced by the normal Unity inventory.
   - MSTR-03 records that the live Shop has a valid node/session shell but no canonical economy contract, while its stock, discount, reroll, capability, and Unity presentation paths encode contradictory prototype behavior.
   - MSTR-04 records the accepted permanent abnormality/corrosion compendium as a game-global commit distinct from removed gameplay battle history and non-authoritative debug traces.
   - MSTR-05 and MSTR-06 record the R01 authority/resync corrections completed by G30.
   - MSTR-07 records R05's sequence-dependent Unity PlayMode pointer/teardown isolation failure.

Stable issue keys always include the source prefix.

- DCA-C-01, DCA-H-01, DCA-M-01, DCA-L-01
- CPR-C-01, CPR-H-01, CPR-M-01
- MSTR-01 through MSTR-07

Do not use bare H-01 or M-01 in child goals because the two reports reuse those identifiers for different findings.

Normalized registry size:

- 52 DCA entries, including one excluded entry.
- 6 unique CPR entries.
- 7 master-discovered policy/correction findings.
- 65 tracked entries in this master.
- 63 actionable or unresolved policy/correction entries.
- 1 policy-accepted/no-change entry.
- 1 excluded entry.
- 9 CPR aliases mapped to existing DCA ownership.

## Required Startup Protocol

At the start of this master and every resumed master run, read:

1. this PLAN.md,
2. EXPERIMENTS.md,
3. EXPERIMENT_NOTES.md,
4. [docs/README.md](../../README.md),
5. [docs/engineering_workflow.md](../../engineering_workflow.md),
6. [docs/goal_completion_review_guide.md](../../goal_completion_review_guide.md),
7. both audit inputs listed above.

Before starting a child goal, also read:

- that child goal's PLAN.md,
- that child goal's EXPERIMENTS.md,
- that child goal's EXPERIMENT_NOTES.md,
- the runtime, data, server, Unity, test, and canonical files named by its primary findings.

Do not create every child directory in advance. Create docs/goals/<child_goal>/PLAN.md, EXPERIMENTS.md, and EXPERIMENT_NOTES.md only when that child begins.

## Source Of Truth Order

Verify facts in this order:

1. actual runtime code and state mutation,
2. production-loaded live RON/data,
3. Rust typed DTO and serialization shape,
4. game_server mapping and WebSocket message ordering,
5. actual Unity reader, store, command bus, presenter, and test consumer,
6. latest canonical policy and transport documents,
7. goal documents and historical notes.

This master and its child documents are not gameplay or wire source of truth.

When evidence conflicts:

- record the current behavior separately from the intended policy,
- do not turn the current implementation into policy merely because tests encode it,
- do not implement a document promise when a user decision is required,
- set the child status to policy_decision_required and report the decision packet.

## Combat Exit Terminology

Use these terms distinctly in every child, test, and report:

- **Unit withdrawal** (`WithdrawUnit`, 유닛 철수): one deployed runtime unit voluntarily leaves the field; the battle continues; its withdrawal cooldown and redeployment policy apply; no battle-entry snapshot is restored.
- **Tactical defeat** (`UnitDied`, Battle HP 0, 전투불능): one runtime unit is defeated; attempt-local trauma changes immediately and same-battle redeployment may follow.
- **Battle-stage abandonment** (`RetreatBattle`, 전투 포기): the player abandons the entire combat stage; the exact authoritative pre-entry snapshot is restored.
- **Battle defeat** (`BattleEnd` with a non-player winner, 전투 패배): the simulation finishes without stage abandonment; committed result policy applies.

Do not use the bare Korean word `퇴각` in new policy text because it can be confused with unit withdrawal. Use `전투 포기 (RetreatBattle)` for the whole-stage command.

## Baseline At Master Creation

The following baseline was re-run on 2026-07-11 before this master was created.

| Command                                 | Result                                                                |
| --------------------------------------- | --------------------------------------------------------------------- |
| cargo fmt --all -- --check              | Passed                                                                |
| core_enhanced: cargo test --all-targets | Passed: 681, failed: 0                                                |
| game_server: cargo test --all-targets   | Passed: 19, failed: 6                                                 |
| cargo check -p game_server              | Passed                                                                |
| Unity EditMode/PlayMode runner          | Not run; external Unity project execution can mutate Library and logs |

The six game_server failures all stop before their intended assertions because the fixture lacks the mandatory Standard RunStart Visual Novel trigger. This is the baseline owned by G04. Child goals must distinguish these existing failures from regressions they introduce.

## Revalidated DCA Severity Summary

| Current classification    | Count |
| ------------------------- | ----: |
| Critical                  |     1 |
| Confirmed High            |     4 |
| Policy-dependent High     |     1 |
| Medium                    |    18 |
| Low                       |    27 |
| Excluded or informational |     1 |
| Total                     |    52 |

Important corrections from revalidation:

- DCA-C-01 is broader than battle Visual Novel updates; current pause, resume, and speed updates use the same broken same-seq/same-time side-band path.
- DCA-M-21 is current-live High, not a future-only risk. All 30 live abnormality basic attacks currently use pattern plus no authored defense tile range and all declare the Melee range role.
- DCA-H-10 breaks recruitment and reward identity, but Headquarters supplies and shop alternatives mean the entire Headquarters node is not necessarily blocked.
- DCA-H-07 is a stale Core document statement; current Unity screen-space HUD behavior is coherent.
- DCA-M-25 is a discoverability problem, not total probe absence. The external Unity repository contains active probes.
- DCA-M-26 is not an actionable contradiction.
- DCA-L-04 contains an incorrect generalization: incapacitation does not always transition an employee to Dead.

## DCA Finding Coverage Matrix

PLAN.md is the single progress source for this registry. Child documents may reference related findings but must not claim duplicate primary ownership.

| Finding  | Revalidated verdict                                                                                         | Severity                   | Primary child | Policy gate                                                                                                                       | Status                                                                                  |
| -------- | ----------------------------------------------------------------------------------------------------------- | -------------------------- | ------------- | --------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| DCA-C-01 | Confirmed: event seq and battle time cannot order same-time playback, VN, capability, or checkpoint changes | Critical                   | G01           | Accepted `battle_state_revision` authoritative commit contract                                                                    | Complete / High; G01 review passed 2026-07-13                                           |
| DCA-H-01 | Confirmed generic authoring no-op; not live-authored                                                        | Medium                     | G09           | Accepted PD-06 Global reach plus required target-centered footprint                                                               | Complete / High; strict reach/footprint/captured-anchor review passed 2026-07-14         |
| DCA-H-02 | Runtime and documents choose different target authority                                                     | Medium                     | G10           | Accepted PD-07: every skill explicitly authors its own targeting selection; no weapon-profile inheritance fallback                | Complete / High; strict explicit selection review passed 2026-07-14                     |
| DCA-H-03 | Core docs conflict with runtime and Unity preview policy                                                    | Medium                     | G11           | Accepted: no enemy range-preview feature; remove projection and Unity consumption                                                 | Complete / High; enemy preview removal review passed 2026-07-14                         |
| DCA-H-04 | Strictness documentation exceeds current schema guarantees                                                  | Medium                     | G12           | Accepted PD-09 strict generated live-data contract                                                                                | Complete / High; exact-version all-or-nothing loader review passed 2026-07-13           |
| DCA-H-05 | Event node route and command flow absent in ArkClient                                                       | High                       | G03           | Accepted PD-03 illustrated employee-check Event flow                                                                              | Complete / High; typed Event route renders and dispatches exact commands                |
| DCA-H-06 | Event canonical DTO is older than typed Core output                                                         | Medium                     | G03           | Accepted PD-04 panel/employee/d20 Event DTO                                                                                       | Complete / High; canonical and Unity consume the typed panel/check DTO                  |
| DCA-H-07 | Runtime coherent; stale billboard wording only                                                              | Low                        | G13           | None                                                                                                                              | Complete / High; screen-space projection canonical review passed 2026-07-14             |
| DCA-H-08 | Six server tests fail from RunStart fixture drift                                                           | Medium                     | G04           | None                                                                                                                              | Complete / High; fixture and real startup-path regression review passed                 |
| DCA-H-09 | Unity item command sends the wrong field key                                                                | High                       | G02           | None; align to existing typed DTO                                                                                                 | Complete / High; exact target employee field with no alias                              |
| DCA-H-10 | Reward and recruitment identity readers mismatch Core                                                       | Medium                     | G02           | None; align to existing typed DTO                                                                                                 | Complete / High; candidate id and reward instance uuid consumed exactly                 |
| DCA-H-11 | Unity replaces battle capability with a static whitelist                                                    | Medium                     | G07           | Accepted PD-02 Core capability authority                                                                                          | Complete / High; static/local battle execution authority removed                        |
| DCA-H-12 | Blocking battle VN can reject reconnect recovery                                                            | Medium                     | G07           | Recovery authority                                                                                                                | Complete / High; typed query/recovery remains reachable during blocking VN              |
| DCA-H-13 | Recovery, checkpoint, and roster command flows missing in Unity                                             | High                       | G08           | UX and command surface                                                                                                            | Complete / High; typed capability-driven Unity flows and roster preflight closed         |
| DCA-H-14 | Forced BossOmen victory bypasses the independent Gate and advances the Floor directly                       | Medium policy regression   | G14           | Accepted: open a dedicated exit Gate after victory                                                                                | Complete / High; deterministic same-Floor Gate now owns the sole transition             |
| DCA-H-15 | BossOmen-chain boss defeat enters RunFailed without the documented checkpoint recovery action               | High policy regression     | G14           | Accepted explicit remaining-load checkpoint recovery                                                                              | Complete / High; qualified RunFailed exposes the existing load action only              |
| DCA-M-01 | Battle record identity is node-keyed, not abnormality-keyed                                                 | Medium                     | G16           | Superseded: remove the player/gameplay battle-record subsystem entirely                                                           | Complete / High; gameplay record subsystem removed                                      |
| DCA-M-02 | Checkpoint rewind can reuse VN identity; failure not reproduced                                             | Low                        | G17           | Reproduction first                                                                                                                | Complete / High; reproduced and global VN session identity stays monotonic              |
| DCA-M-03 | ReuseCastTarget docs omit runtime range and usefulness checks                                               | Low                        | G18           | None                                                                                                                              | Complete / High; final no-range-recheck/usefulness wording reviewed 2026-07-14          |
| DCA-M-04 | TileArea example contains removed range_units field                                                         | Low                        | G18           | None                                                                                                                              | Complete / High; concrete footprint example reviewed 2026-07-14                         |
| DCA-M-05 | RetargetOnStep examples do not match live catalog                                                           | Low                        | G18           | None                                                                                                                              | Complete / High; all four current live examples pinned 2026-07-14                      |
| DCA-M-06 | Live fragment index count is stale                                                                          | Low                        | G20           | None                                                                                                                              | Complete / High; all 30 abnormality-derived fragments pinned 2026-07-14                |
| DCA-M-07 | Maintenance Safe Node research delivery is ambiguous                                                        | Medium                     | G21           | Superseded by PD-13: deliver automatically on the authoritative return to Safe Zone/planning, not on category-specific node entry | Complete / High; atomic completion-only planning-return delivery                        |
| DCA-M-08 | Weapon block modifier is documented but unsupported                                                         | Deferred content debt      | G22           | Equipment balance and schema deferred until content scope is ready                                                                | Deferred by user 2026-07-12                                                             |
| DCA-M-09 | Manifest proves ID membership, not behavior coverage                                                        | Medium                     | G20           | Evidence semantics                                                                                                                | Complete / High; v2 evidence strength is exact and non-overclaiming                    |
| DCA-M-10 | FacilityEntity is wire-reserved but not runtime-produced                                                    | Low                        | G13           | Reserved variant policy if changed                                                                                                | Complete / High; reserved-versus-production wording review passed 2026-07-14            |
| DCA-M-11 | Visual Novel default_blocking is duplicate unused authoring                                                 | Low                        | G19           | Delegated technical default: remove duplicate unused authoring; do not define fallback authority                                  | Complete / High; duplicate field removed and trigger remains sole authority             |
| DCA-M-12 | Server bridge helper document contradicts side messages                                                     | Low                        | G07           | None after authority decision                                                                                                     | Complete / High; canonical helper matches battle side-message and stale-resync ordering |
| DCA-M-13 | Core allowed_actions is not an exact executable list                                                        | Medium                     | G06           | Accepted PD-02 two-level capability contract                                                                                      | Complete / High; typed context capability replaced old list authority                   |
| DCA-M-14 | Unity skill catalog example differs from typed DTO                                                          | Low                        | G02           | None                                                                                                                              | Complete / High; example uses typed range policy shape                                  |
| DCA-M-15 | Unity document names nonexistent ActComplete result                                                         | Low                        | G02           | None                                                                                                                              | Complete / High; canonical completion name is FloorAdvanced                             |
| DCA-M-16 | Combat result abnormality identity field names diverge                                                      | Low                        | G02           | DTO owner if consumer is added                                                                                                    | Complete / High; Unity model/docs/mock use primary_abnormality_id                       |
| DCA-M-17 | Recovery-node predicate is undefined and weakly live-reachable                                              | Low                        | G15           | Accepted: only immediately enterable Treatment/Recruitment-capable nodes defer non-boss NoDeployableEmployees failure             | Complete / High; one typed immediate recruitment predicate feeds projection and validation |
| DCA-M-18 | can_load reports capacity, not current command executability                                                | Low                        | G06           | Replace ambiguous affordance with typed scoped capability/eligibility semantics                                                   | Complete / High; redundant factual affordance removed                                   |
| DCA-M-19 | Required consumable use preview DTO is absent                                                               | Medium                     | G06           | Shared preflight contract in G06; concrete inventory eligibility in G24                                                           | Complete / High; shared capability and quantity-aware inventory eligibility agree       |
| DCA-M-20 | Current battle-stage abandonment consumes attempts and has an exhausted-abandonment node-completion path    | High policy regression     | G15           | Accepted exact pre-entry rollback removes attempt consumption and the exhausted-abandonment branch                                | Complete / High; exact pre-entry restore replaces every exhausted-abandonment branch    |
| DCA-M-21 | All current placeholder abnormality attacks use the missing-range Melee fallback                            | User-deferred content debt | G05           | Detailed abnormality combat designs must exist first                                                                              | Deferred by user 2026-07-11                                                             |
| DCA-M-22 | Event sequence influences delivery identity; outcome not reproduced                                         | Low                        | G17           | Reproduction first                                                                                                                | Complete / High; reproduced and gameplay/log identity domains decoupled                 |
| DCA-M-23 | Goal review and cleanup lifecycle order is undefined                                                        | Low                        | G25           | None                                                                                                                              | Complete / High; child, independent review, correction rerun and final closure order published |
| DCA-M-24 | Live catalog manifest has no durable update ownership                                                       | Low                        | G20           | None                                                                                                                              | Complete / High; durable author/reviewer/version lifecycle published 2026-07-14         |
| DCA-M-25 | Probe paths exist externally but workflow does not identify them                                            | Low                        | G25           | None                                                                                                                              | Complete / High; exact external paths, gates, contracts and evidence rules verified     |
| DCA-M-26 | Craft action wording and EquipItem combination are compatible                                               | Excluded                   | EX-01         | None                                                                                                                              | Removed-not-actionable                                                                  |
| DCA-M-27 | Reward documentation is ahead of fixed live rewards                                                         | Low                        | G23           | Reward policy if behavior changes                                                                                                 | Complete / High; current fixed live rewards separated from generic capability           |
| DCA-M-28 | Generic Maintenance text exceeds current live recipe scope                                                  | Deferred content debt      | G22           | Content scope and balance deferred until Equipment/Maintenance designs are ready                                                  | Deferred by user 2026-07-12                                                             |
| DCA-M-29 | XP, level, tier, and distribution policy is underdocumented                                                 | Low                        | G23           | None if documenting current behavior                                                                                              | Complete / High; exact XP distribution, level and tier policy canonicalized             |
| DCA-M-30 | Fixed category caps can make an item reward impossible to receive and block result completion               | Medium                     | G24           | Accepted infinite-inventory policy removes admission failure; automatic display reserve replaces hard caps                        | Complete / High; unbounded ordinary grants and 50/10 reserve review passed 2026-07-14   |
| DCA-M-31 | Consumable duration timing conflicts across documents                                                       | Medium                     | G15           | Accepted: decrement once on committed victory/defeat; restore on `RetreatBattle`                                                  | Complete / High; duration is staged once per committed result and restored on abandonment |
| DCA-L-01 | Endless briefing lacks Unity-local presentation resource                                                    | Low                        | G19           | None if aligning to live IDs                                                                                                      | Complete / High; exact Endless local presentation resource and fail-closed lookup       |
| DCA-L-02 | Root snapshot example omits active typed fields                                                             | Low                        | G02           | None                                                                                                                              | Complete / High; root example includes VN and skill catalog                             |
| DCA-L-03 | Unity reads display_name instead of Core name                                                               | Low                        | G02           | None                                                                                                                              | Complete / High; active modifier reads name only                                        |
| DCA-L-04 | Incapacitation terminology and original rationale need correction                                           | Low                        | G15           | Life-state meaning                                                                                                                | Complete / High; tactical RuntimeUnit death is distinct from persistent employee death  |
| DCA-L-05 | Checkpoint record preservation lives only in goal history                                                   | Removed policy surface     | G16           | Not applicable after removing gameplay battle records                                                                             | Not applicable; gameplay record/checkpoint surface was removed                          |

Coverage invariant:

- C: 1 primary assignment.
- H: 15 primary assignments.
- M: 31 primary assignments, including one excluded assignment.
- L: 5 primary assignments.
- Total: 52 exactly once.

## CPR Alias And Unique Finding Map

### CPR Findings Already Covered By DCA Ownership

| CPR finding | Primary DCA owner                                                              |
| ----------- | ------------------------------------------------------------------------------ |
| CPR-C-01    | DCA-C-01 / G01                                                                 |
| CPR-H-04    | DCA-H-05 and DCA-H-06 / G03                                                    |
| CPR-H-05    | DCA-H-09 / G02                                                                 |
| CPR-H-06    | DCA-H-10 / G02                                                                 |
| CPR-H-07    | DCA-H-13 / G08                                                                 |
| CPR-M-01    | DCA-H-08 fixture gate / G04; early supported-mode cardinality validation / G12 |
| CPR-M-03    | DCA-M-01 / G16                                                                 |
| CPR-M-04    | DCA-H-11 and DCA-M-13 / G07 and G06                                            |
| CPR-M-05    | DCA-H-01 / G09                                                                 |

### Unique CPR Findings

| Finding  | Current meaning                                                                                                          | Severity                                       | Primary child | Policy gate                                                                                         | Status                                                                                |
| -------- | ------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------- | ------------- | --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| CPR-H-01 | Runtime-instance results currently mix intended per-death penalties with employee-level XP, summary, and MVP aggregation | High                                           | G26           | Per-death penalty accepted; employee-level non-penalty aggregation remains structural               | Complete / High; per-instance penalties and employee outcomes separated               |
| CPR-H-02 | Fatal committed defeat can expose retry and cancel before RunFailed                                                      | High                                           | G15           | Accepted: NoLivingEmployees preempts retry/cancel after non-abandonment commit                      | Complete / High; staged fatal result preempts attempt retry/cancel                    |
| CPR-H-03 | Debug file export can break authoritative battle transition                                                              | High                                           | G16           | None for side-effect isolation                                                                      | Complete / High; diagnostic failure cannot reject authoritative transition            |
| CPR-H-08 | Event RewardEffect references validate late and one variant panics                                                       | High latent                                    | G12           | Remove or implement authored variant                                                                | Complete / High; shared Event RewardEffect validation and unsupported variant removal |
| CPR-H-09 | Battle-stage abandonment discards attempt-local employee condition changes                                               | Policy accepted/no change for casualty outcome | G15           | Exact pre-entry rollback confirmed, including persistent death and all authoritative gameplay state | Complete / High; world snapshot composes around G26 employee discard                  |
| CPR-M-02 | Extra waves and map-event references lack final bundle validation                                                        | Medium latent                                  | G12           | Stage and mission validation key                                                                    | Complete / High; final extra-wave/map payload validation                              |

### Master-Discovered Findings

| Finding | Current meaning                                                                                                                                                                                                                                                                                                                                                | Severity                                 | Primary child | Policy gate                                                                                                                                                                            | Status                                                |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| MSTR-01 | A historical goal explicitly replaced updated-trauma death redeploy HP with fixed 60% HP for implementation simplicity; runtime, tests, and the rulebook now enforce that superseded policy                                                                                                                                                                    | High policy regression                   | G26           | Per-death attempt transaction                                                                                                                                                          | Complete / High; attempt-local condition owns redeploy |
| MSTR-02 | Artifact is a live-reachable, side-global stat/trigger subsystem absent from the canonical rulebook; all owned copies auto-apply to every employee while the normal Unity inventory does not expose them                                                                                                                                                       | High policy debt                         | G27           | Accepted: remove completely without replacement; G28 owns the replacement Shop contract instead of G27 inventing content                                                               | Complete / High; subsystem deleted without replacement |
| MSTR-03 | Shop is live-reachable and its node/session command shell works, but the rulebook defines no merchandise, stock, price, sale, discount, refresh, or encounter-profile policy; live data contains inert discounts, unreachable stock, empty/placeholder pages, and whole-page swaps, while Unity discards the rich item data needed to make a purchase decision | High user-flow and policy debt           | G28           | Shared Shop, reusable pools, deterministic refresh, optional filtered selling, and no-arbitrage pricing are accepted; initial live presets and balance values remain content authoring | Complete / High; deterministic typed economy and Unity offer contract |
| MSTR-04 | The game has no player battle history but does require a permanent abnormality/corrosion compendium whose discovery commit is distinct from debug battle traces                                                                                                                                                                                                | High persistence and collection contract | G29           | Accepted PD-12 monotonic global compendium committed only when an encountered combat node is consumed                                                                                  | Complete / High; strict global profile and projection |
| MSTR-05 | Battle capability omission/disablement is not a Core execution gate, so a direct current-authority client can bypass blocking-VN mutation suppression                                                                                                                                                                                                          | High authority gap                       | G30           | No new policy; implement accepted PD-02 shared battle projector/preflight execution authority                                                                                          | Complete / High; G30 and R01 correction rerun passed  |
| MSTR-06 | Missing battle authority returns a typed error but, unlike stale authority, receives no current resync recovery batch                                                                                                                                                                                                                                          | Medium recovery gap                      | G30           | No new policy; apply accepted PD-01/PD-02 missing/stale recovery symmetry                                                                                                              | Complete / High; G30 and R01 correction rerun passed  |
| MSTR-07 | Unity PlayMode starter selection retains pointer hover after prior focused tests; the first failure aborts cleanup and cascades into unrelated broad-suite failures                                                                                                                                                                                              | Medium validation reliability gap        | G31           | No gameplay policy; deterministic Input System state and guaranteed test teardown                                                                                                     | Complete / High; G31 order/repetition review passed   |

## Child Goal Catalog

Only create a child directory when that child is started.

| ID    | Child goal name                                       | Primary ownership                                                    | Priority      | Main prerequisite                                                                                                                                            |
| ----- | ----------------------------------------------------- | -------------------------------------------------------------------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| G01   | battle_projection_revision_contract                   | DCA-C-01                                                             | P0            | Refined `battle_state_revision` contract accepted; restore G04 fixture before cross-server broad validation                                                  |
| G02   | unity_wire_identity_alignment                         | DCA-H-09, DCA-H-10, DCA-M-14, DCA-M-15, DCA-M-16, DCA-L-02, DCA-L-03 | P0            | Accepted PD-04 Event DTO boundary where shared wire migration overlaps; no aliases                                                                           |
| G03   | illustrated_employee_check_event_flow                 | DCA-H-05, DCA-H-06                                                   | P0            | Accepted PD-03/PD-04; G02 shared envelope; G06 capability seam; coordinate Event-combat boundaries with G07/G14/G15 and validation with G12                  |
| G04   | server_runstart_fixture_contract                      | DCA-H-08                                                             | P0            | None; first safe executable child                                                                                                                            |
| G05   | abnormality_basic_attack_range_authoring              | DCA-M-21                                                             | Deferred      | Detailed abnormality settings and user-approved range values                                                                                                 |
| G06   | contextual_action_capability_projection               | DCA-M-13, DCA-M-18, DCA-M-19                                         | P1            | Accepted PD-02; G01 establishes battle revision semantics first                                                                                              |
| G07   | battle_capability_and_reconnect_authority             | DCA-H-11, DCA-H-12, DCA-M-12                                         | P1            | G01, G04, and G06                                                                                                                                            |
| G08   | unity_recovery_and_roster_command_flows               | DCA-H-13                                                             | P1            | G06, G07, and G14 rollback decision                                                                                                                          |
| G09   | global_target_area_delivery_contract                  | DCA-H-01                                                             | P1            | Accepted PD-06; coordinate target-rule ownership with G10 and examples with G18                                                                              |
| G10   | skill_targeting_authority                             | DCA-H-02                                                             | P2            | Accepted PD-07 explicit per-skill targeting authority; no weapon-profile inheritance fallback                                                                |
| G11   | enemy_range_preview_removal                           | DCA-H-03                                                             | P2            | Remove enemy range-preview transport and Unity presentation without compatibility fields                                                                     |
| G12   | gameplay_ron_strictness_and_bundle_validation         | DCA-H-04, CPR-H-08, CPR-M-01 early-cardinality portion, CPR-M-02     | P1            | Accepted PD-09 strict generated-data contract; remove aliases and reject invalid bundles before runtime                                                      |
| G13   | battlefield_projection_canonical_cleanup              | DCA-H-07, DCA-M-10                                                   | P2            | G11 if preview policy changes                                                                                                                                |
| G14   | boss_omen_gate_and_rollback_policy                    | DCA-H-14, DCA-H-15                                                   | P1            | Victory Gate and defeat checkpoint recovery accepted                                                                                                         |
| G15   | battle_failure_abandonment_consumable_resolution      | DCA-M-17, DCA-M-20, DCA-M-31, DCA-L-04, CPR-H-02, CPR-H-09           | P1            | Accepted abandonment/result policy plus boss-immediate and capability-based non-boss recovery deferral                                                       |
| G16   | battle_record_removal                                | DCA-M-01, DCA-L-05, CPR-H-03                                         | P1            | Remove gameplay battle records; isolate optional developer diagnostics from authoritative transitions                                                        |
| G17   | runtime_identity_determinism                          | DCA-M-02, DCA-M-22                                                   | P2            | Reproduction experiment before design                                                                                                                        |
| G18   | skill_contract_example_cleanup                        | DCA-M-03, DCA-M-04, DCA-M-05                                         | P2            | G09 and G10 if semantics change                                                                                                                              |
| G19   | visual_novel_authoring_and_local_presentation         | DCA-M-11, DCA-L-01                                                   | P2            | Remove unused duplicate `default_blocking` authoring and align live presentation resources without fallback                                                  |
| G20   | live_skill_catalog_evidence_governance                | DCA-M-06, DCA-M-09, DCA-M-24                                         | P2            | Define manifest evidence semantics                                                                                                                           |
| G21   | safezone_research_delivery_policy                     | DCA-M-07                                                             | P2            | Replace node-category entry delivery with automatic authoritative Safe Zone/planning return delivery                                                         |
| G22   | equipment_maintenance_live_contract                   | DCA-M-08, DCA-M-28                                                   | Deferred      | Resume when Equipment/Maintenance content scope and balance are ready                                                                                        |
| G23   | reward_growth_canonicalization                        | DCA-M-27, DCA-M-29                                                   | P2            | Document current behavior unless policy changes                                                                                                              |
| G24   | inventory_reward_capacity_contract                    | DCA-M-30                                                             | P1            | Phase 6 first: establish final ordinary inventory identity/quantity/store boundary while leaving Artifact isolated for G27 removal                           |
| G25   | goal_lifecycle_and_probe_discoverability              | DCA-M-23, DCA-M-25                                                   | P2            | None                                                                                                                                                         |
| G26   | combat_employee_incapacitation_resolution             | CPR-H-01, MSTR-01                                                    | P1            | Accepted per-death penalty transaction and employee-level non-penalty aggregation                                                                            |
| G27   | artifact_legacy_removal                               | MSTR-02                                                              | P1            | Phase 6 last: remove Artifact after G28 removes every Shop/content dependency; do not adapt Artifact into G24's final ordinary inventory                     |
| G28   | shop_economy_and_offer_contract                       | MSTR-03                                                              | P1            | Phase 6 after G24: consume final quantity/identity authority, remove Artifact merchandise dependencies, and implement accepted Shop session/economy contract |
| G29   | global_encounter_compendium                           | MSTR-04                                                              | P1            | Accepted PD-12; coordinate node-consumption commit with G15 and global profile persistence outside run/checkpoint rollback                                   |
| G30   | battle_capability_execution_authority_correction      | MSTR-05, MSTR-06                                                     | P0 correction | R01 Partially complete finding; user authorization required before child creation/implementation                                                             |
| G31   | unity_playmode_pointer_isolation_correction           | MSTR-07                                                              | P0 correction | R05 Partially complete finding; exact/focused-to-full/repeated-full PlayMode proof                                                                            |
| EX-01 | excluded_audit_finding                                | DCA-M-26                                                             | Excluded      | No child directory                                                                                                                                           |

G13 is the primary owner of DCA-M-10 because the verified issue is the canonical description of a wire-reserved, non-produced variant. G12 may reference it only if a later schema decision changes that status.

## Policy Decision Register

A child may not implement a policy-bearing change until its decision is recorded in the child PLAN and the nearest durable canonical document.

| Decision                                      | Current conflict                                                                                                                                                                                                  | Affected children                 | Recommended starting direction                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| --------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| PD-01 Battle side-band revision               | event seq and time cannot order pause, speed, VN, capability, or other same-time authoritative checkpoint changes                                                                                                 | G01, G07, G08                     | Accepted 2026-07-11 and refined after code review on 2026-07-12: add battle-UUID-scoped `battle_state_revision` as the authoritative battle-state commit order. It is not a packet counter. Event seq remains event-delta continuity; battle time remains simulation/presentation time. Use the detailed accepted contract below.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| PD-02 Capability meaning                      | payload-less state affordance diverges from payload-sensitive execution and Unity substitutes local/static rules                                                                                                  | G06, G07, G08, G03, G21, G24, G28 | Accepted 2026-07-12 after code review: remove `allowed_actions` and replace it, without a compatibility alias, with a two-level Core-authoritative contract: typed context `action_capabilities` plus typed payload eligibility in domain projections/previews. Projection and commit share the same validators, while execution revalidates atomically. Battle capability is committed under PD-01 revision semantics. Use the detailed accepted contract below.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| PD-03 Event support                           | live Event is Core-reachable but Unity lacks the intended illustrated narrative, choice, employee selection, one-shot d20 resolution, and failure-completion flow; prior Event-to-VN interpretation was incorrect | G03, G06, G07, G12, G14, G15      | Accepted and user-corrected 2026-07-12: Event is an illustrated text-and-choice node, not Visual Novel. After local image/text progression, the player selects a choice and employee, then Core resolves exactly one unmodified d20. Natural 1/20 are critical failure/success; 2..19 use `roll >= required_roll`. Success performs the authored action; failure shows authored failure text and completes on the final click. No retry or reroll. Use the corrected detailed contract below.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| PD-04 Event DTO                               | stale Unity and provisional Core shapes do not express image/text panels, employee candidates, one-shot d20 result persistence, or the final player flow                                                          | G02, G03, G06, G12                | Accepted and user-corrected 2026-07-12: one typed Event DTO uses ordered image/text panels, tagged advance versus employee-check choices, Core eligibility, a scene-qualified `ResolveEventChoiceCheck`, and persisted raw d20 outcome. Remove Event VN fields, old preview/hint fields, modifiers, Raw parsing, and dual command/schema paths. Use the corrected detailed contract below.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| PD-05 Abnormality attack ranges               | all 30 current placeholder entries rely on the Melee fallback                                                                                                                                                     | G05                               | Deferred by user 2026-07-11 because detailed abnormality settings are not finalized. Keep the fallback only as an explicitly temporary placeholder; do not invent or mass-author values. Start G05 when per-abnormality combat designs are approved, then assign explicit reusable/custom patterns and remove the fallback.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| PD-06 Global target reach and area delivery   | `WholeFieldValidTiles` overloads target acquisition and effect footprint; validation accepts a TileArea shape that runtime resolves as an empty step                                                              | G09, G10, G18                     | Accepted 2026-07-12: separate global unit-target reach from required TileArea footprint. A selected unit's tile is captured at cast selection and remains the attack center even if that unit dies or withdraws. Remove the mixed legacy contract without compatibility. Preserve a future independent caster-tile-centered radial field-wave delivery as documented extension only; do not expose an unimplemented schema variant. Use the detailed accepted contract below.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| PD-07 Skill target authority                  | UnitTargetRule versus weapon profile inheritance                                                                                                                                                                  | G10, G09, G18                     | Accepted 2026-07-12: basic attacks use the weapon `targeting_profile`; every skill explicitly authors its own cast-target selection, possibly through an explicitly referenced reusable preset. Missing skill selection fails live loading. Weapon changes never silently change skill targeting, and no inheritance fallback or dual authority remains.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| PD-08 Enemy preview                           | all-unit wire projection versus player-only UI                                                                                                                                                                    | G11, G13                          | Accepted 2026-07-12 for the current game: do not expose enemy range preview. Remove normal/checkpoint projection, Unity parsing/rendering, canonical promises, mocks, tests, and empty compatibility fields. Actual actions, active attack telegraphs, hazards, projectiles, and wave boundaries remain separate presentation. A future preview feature requires an intentional DTO revision rather than a reserved field now.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| PD-09 RON strictness                          | mixed deny-unknown and accepted aliases                                                                                                                                                                           | G12, G19, G22                     | Accepted 2026-07-12: live gameplay RON is generated by a dedicated authoring program and is maximally strict. Require exact schema version and canonical fields/variants, reject unknown/removed/missing required fields, aliases, duplicate IDs, invalid ranges/combinations/references, unsupported variants, and partial bundles. The authoring tool validates before export and Core independently validates again from shared schema/validator authority. Save migration is a separate contract.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| PD-10 BossOmen transition                     | BossOmen-chain final boss bypasses Gate and checkpoint recovery                                                                                                                                                   | G14, G08                          | Accepted 2026-07-11: victory commits boss results and clears the chain, then opens a dedicated exit Gate as the sole Floor-progression route; Gate confirmation applies normal trauma recovery and advances the Floor. Defeat commits the result and enters RunFailed. If a SavePoint checkpoint and load count remain, RunFailed exposes an explicit checkpoint-load action; accepting it consumes one load and restores that checkpoint. With no remaining load, the run ends. This is the chain's capstone boss, not the final boss of the Endless run.                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| PD-11 Failure and battle-stage abandonment    | retry, cancel, casualty, result, and consumption timing conflict                                                                                                                                                  | G15, G26                          | Accepted 2026-07-11/12: `RetreatBattle` is an exact quick-load of authoritative pre-entry state. Every committed result uses one coherent CombatResult and commits its authored consequences; abandonment restores them. A boss defeat enters RunFailed immediately regardless of map recovery nodes, while the previously accepted remaining-checkpoint-load action may still be offered from RunFailed. For a non-boss combat failure whose committed result leaves zero deployable employees, defer RunFailed only when the current map exposes at least one immediately enterable node with explicit Treatment or Recruitment capability. During deferral, only qualifying recovery nodes may be entered. After that node resolves, re-evaluate deployability; if no deployable employee exists, enter RunFailed. Merely future-reachable, label-based Safe, Shop, or other nodes do not qualify.                                                                                              |
| PD-12 Record and compendium semantics         | gameplay battle-record identity conflicts with the intended permanent encounter collection                                                                                                                        | G16, G29, G15                     | Corrected and accepted 2026-07-12: remove all player-facing/authoritative battle-record history. Keep only non-authoritative debug traces. Separately maintain a game-global monotonic compendium of canonical abnormality IDs and corrosion profile IDs actually spawned in a combat node; commit discoveries only when that node is consumed. Once committed, discoveries survive run checkpoint load, older save load, new runs, failure, and all ordinary rollback, and disappear only through explicit whole-game/compendium reset.                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| PD-13 Safe Zone research delivery             | runtime hard-codes delivery on Support, HeadquartersContact, Shop, and Reward node entry while Safe Zone is the intended reception fiction                                                                        | G21                               | Corrected and accepted 2026-07-12: remove the node-category delivery allowlist. Queue completed research during node activity and automatically deliver all pending results on the authoritative Core transition back to map/planning that Unity presents as Safe Zone. Do not trust a Unity-local screen-enter signal. Delivery is idempotent and atomic; RunFailed does not synthesize a Safe Zone return.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| PD-14 Equipment and Maintenance scope         | documented block modifier and generic recipe scope exceed live data                                                                                                                                               | G22                               | Reconfirmed deferred by user 2026-07-12. The existing Maintenance foundation—node/session, equipment and fragment dismantle/enhance, fragment awakening, previews, costs/gains, equipped-item warnings, Core/Unity commands—is sufficient. Do not invent block values, generic recipes, E.G.O growth, or balance. Resume incrementally when real content is authored.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| PD-15 Infinite inventory and reward admission | fixed category caps can reject grants; current consumables are per-copy despite being stateless copies                                                                                                            | G24, G27                          | Accepted 2026-07-11 for canonical ordinary inventory: item grants never fail because of occupied slots. Logical `reserved_slot_count` starts at 50 and grows atomically by enough 10-slot increments before a successful new-slot addition; quantity increases do not grow it and its persisted high-water value never shrinks. Identical consumables share one definition-id quantity stack with no authored stack cap; equipment remains per-instance; currency, equipment materials, skill fragments, and fragment dust remain separate non-slot resources. Artifact does not participate: PD-17 removes that legacy subsystem completely.                                                                                                                                                                                                                                                                                                                                                      |
| PD-16 Employee incapacitation resolution      | runtime instance history maps to one persistent employee while tactical deaths affect the next deployment                                                                                                         | G26                               | Accepted 2026-07-11: every tactical death applies the complete authored death-penalty bundle exactly once, regardless of deployment count. The attempt-local employee state immediately receives trauma, Run HP loss, injury, applicable trust/consumable mitigation, death-prevention consumption, and life-state transition. Fresh redeployment uses that updated state; a dead employee cannot redeploy. Victory or defeat commits all accumulated per-death penalties; battle-stage abandonment discards all of them. Preserve instance replay, but aggregate XP, summary rows, and MVP by employee rather than treating them as death penalties.                                                                                                                                                                                                                                                                                                                                              |
| PD-17 Artifact system identity                | Artifact is absent from the rulebook but is live-reachable through shops, automatically applies every owned Artifact to every allied unit's stats/triggers, and is not shown by the normal Unity inventory        | G27, G24                          | Accepted 2026-07-11: remove Artifact completely without replacement. Delete live Artifact RON and shop references, loader/database/registry types, ordinary inventory/DTO/error paths, reward/admin variants, scenario/runtime/stat/trigger/cooldown/event-log surfaces, server/Unity contract fields, and legacy tests. Do not migrate names, effects, prices, IDs, or add compatibility decode/state layers. `ContainmentRelic`/격리 유물 was an assistant-invented unaccepted sketch and is not part of the game. A future system, if any, requires a separate user-approved goal from first principles.                                                                                                                                                                                                                                                                                                                                                                                        |
| PD-18 Shop identity and economy contract      | The rulebook recognizes Shop only as a one-use Safe Node while runtime uses fixed UUID pages, hidden swaps, inert discounts, and incomplete Unity offers                                                          | G28, G27, G24                     | Accepted 2026-07-12: one shared Shop serves Headquarters-support and exploration Unknown-Shop contexts through distinct composed presets/local entries. Deterministic sessions own stable `offer_count`, explicit offer quantities, current page/purchased holes, non-repeatable purchase history, refresh use count, and RNG continuation across reopen/reconnect; checkpoint load restores the session. Refresh is optional, atomically replaces the whole page, costs the same authored `base_cost` per use in the initial contract, and never produces partial pages. Selling is optional and quantity-aware for stacks; equipment uses exact instance UUID and quantity one. Core owns checked quotes, typed eligibility, atomic commits, and strict cross-Shop no-arbitrage. Remove prototype visible/hidden, `can_reroll`, fixed 50% resale, Artifact stock, and UUID-only Unity behavior without compatibility. Exact initial pools and balance remain content authoring, not policy debt. |

### Accepted PD-01 Battle State Revision Contract

Identity and lifetime:

- Wire field: `battle_state_revision`.
- Scope: one `battle_uuid`. A new battle UUID starts a new sequence.
- Initial committed battle projection uses revision 1.
- Core owns the current committed revision as authoritative battle-session state.
- Use checked increment. Overflow is an error; never saturate and reuse a revision.

Commit semantics:

- Increment exactly once after a successful transaction changes any client-authoritative battle projection.
- The revision covers simulation/checkpoint state, playback, deployment, skill readiness, battle Visual Novel projection, and exact battle capability.
- Complete nested triggers, Visual Novel transitions, capability recalculation, and all other mutations before stamping the revision and building the DTO.
- Failed/rejected commands, no-op requests, `RequestBattleState`, serialization, duplicate resend, and resync do not increment revision.
- Repeated Pause/Resume or speed requests that would not change state must be rejected or treated as a non-mutating result; they must not consume a revision.
- A tick blocked by pause or blocking VN and returning no battle update does not increment revision.

Wire and client ordering:

- Every top-level `battle_update`, nested `battle_resync.update`, and battle command acknowledgement carries the committed `battle_state_revision` associated with that result.
- Unity orders ordinary battle state by `(battle_uuid, battle_state_revision)`.
- A higher revision is newer even when event seq and battle time are unchanged.
- An equal or lower revision on an ordinary update is duplicate/stale and does not replace state.
- Event seq validates `events_delta` continuity only. It is not a state revision.
- `server_battle_time_ms` drives simulation and presentation anchoring only. It is not a staleness key.
- A newer revision with an event-delta gap requests resync rather than fabricating or skipping events.
- `battle_resync` is an explicit authority override: it may reapply the same current revision to repair local state, but it may not roll back to an older revision.
- Reconnect preserves the current active battle revision. Missing setup recovery sends the immutable setup plus the current revisioned state.

Ownership and transport:

- Remove `last_pushed_event_log_seq` from `ActiveBattleSession` and the root gameplay snapshot.
- Core owns the immutable battle event log and exposes read-only update/resync projection with an explicit `after_seq` input.
- The active server connection/outbox owns its event-delivery cursor.
- Command-triggered updates and automatic tick updates use one ordered per-session battle outbox, or an equivalent ordering boundary, so a later event delta cannot overtake the delta it depends on.
- Revision detection does not substitute for ordered event transport.
- G06/G07 replace Unity's static battle-action whitelist with Core context capability plus battle-domain payload eligibility included in the revisioned projection.

Required completion evidence:

- Core tests: initial revision, one increment per successful transaction, same-time playback/VN/capability increments, no increment on failure/no-op/read/resync, final update ordering, and checked overflow.
- Server tests: top-level update, nested resync, and command acknowledgement carry the same revision; command and tick emission cannot invert dependent deltas; reconnect/setup recovery preserves revision.
- Unity tests: same-seq/same-time higher revision applies; equal/lower ordinary revision is ignored; newer revision with seq gap requests resync; same-revision resync repairs state; battle UUID change resets local revision scope.
- Cross-repository fixture: actual Rust-serialized setup/update/resync payloads are consumed by Unity tests. Mock-only agreement is insufficient.
- Battle time remains unchanged by pure playback/VN state changes and is never used as a fallback staleness key.

### Accepted PD-02 Action Capability Contract

Canonical shape and meaning:

- Remove root `allowed_actions: Vec<ActionKind>` rather than redefining it or retaining a compatibility alias.
- Core exposes typed `action_capabilities` for context-level command families. Each entry contains a stable typed action identity, scope (`gameplay`, `query`, or `recovery`), enabled state, and a stable typed disabled-reason code when blocked.
- Omit actions irrelevant to the current context. Keep context-relevant blocked actions as disabled entries so Unity can explain why they are unavailable.
- A parameterized context capability is enabled only when at least one currently executable payload exists. This does not imply that every possible payload is executable.
- Concrete payload eligibility belongs to typed domain projections or previews: Shop offers and sale quotes, inventory items, employees, tiles, skills and targets, map nodes, rewards, Visual Novel choices, and equivalent domain identities.
- Do not introduce a generic untyped payload bag. Each domain keeps its typed identifiers and eligibility result.
- Large or combinatorial payload spaces use an authoritative preview/preflight for the selected payload rather than projecting an unbounded Cartesian product.

Authority and validation:

- Capability and payload eligibility are snapshots for presentation, not authorization tokens.
- Projection/preflight and command execution call the same pure domain validators and produce the same typed reason vocabulary. Do not duplicate eligibility policy in `ActionScheduler`, DTO builders, handlers, server adapters, or Unity.
- Every command revalidates atomically against current authoritative state before mutation.
- Battle mutation commands carry the expected `battle_state_revision`. A stale revision returns a typed stale-revision result and the current authority/resync path; it does not attempt the mutation against a newer state.
- Battle context capability and battle payload eligibility are part of the complete revisioned projection defined by PD-01. Non-battle context capability belongs to the authoritative root snapshot; concrete non-battle eligibility belongs to its domain DTO or preview.
- Read-only queries and recovery-only operations remain explicitly scoped and are not presented as ordinary gameplay actions.

Legacy removal and client behavior:

- Remove `GameCoreState.action_validator`'s cached HashSet and the state-only `ActionScheduler` table as execution authorities. Any retained presentation cache must be revision-keyed and derived from the canonical projector.
- Remove Unity's static `battleRuntimeActions` whitelist, local capability substitution, and locally invented eligibility rules. Unity renders Core capability/eligibility, localizes typed reason codes, and submits the selected typed payload.
- Blocking Visual Novel state exposes only its valid progression form: advance or concrete choices, never both by default.
- Playback exposes Pause or Resume according to current state. Selecting the already-active speed is disabled or a non-mutating no-op and does not consume a battle revision.
- Remove legacy completeness tests that require every state-table action to be present regardless of payload. Do not preserve them as ignored tests.

Goal ownership:

- G06 owns the shared capability DTO, action scope, stable common reason vocabulary, shared projector/preflight seam, removal of `allowed_actions` and duplicate action-gate authority, and root/server/Unity consumer migration for the common contract.
- G07 owns battle context capability, battle payload eligibility, expected-revision command behavior, reconnect/resync integration, and removal of the Unity battle whitelist.
- G03 owns Event payload eligibility; G21 owns Safe Zone research delivery; G24 owns inventory and consumable eligibility/previews; G28 owns Shop offer, sale, and refresh eligibility.
- G08 consumes the completed battle/recovery capability contract for player-facing recovery and roster flows.
- G06 must not absorb every domain behavior rewrite. Domain children implement their payload rules after the shared seam exists.

Required completion evidence:

- Common parity tests prove that a projected enabled capability/payload succeeds when executed against the same authoritative revision, while a projected disabled payload is rejected with the same typed reason.
- Mutation-between-preview-and-commit tests prove atomic revalidation and typed stale-state handling.
- Context tests prove irrelevant actions are omitted, relevant blocked actions retain reasons, and query/recovery scopes do not leak into gameplay presentation.
- Battle tests prove Pause/Resume exclusivity, current-speed no mutation, valid Visual Novel progression shape, revision participation, and reconnect/resync preservation.
- Domain tests cover at least one enabled and disabled payload for every migrated domain owner; G24 includes consumable preview and G28 includes purchase, sale, and refresh eligibility.
- Unity tests consume actual Rust-serialized capability and domain fixtures, localize typed reason codes, and prove no static/local capability substitute remains. Mock-only agreement is insufficient.

### Accepted PD-03 Illustrated Employee-Check Event Flow Contract

Player-visible flow:

1. Enter the Event node and open a dedicated Event screen.
2. Show ordered Event panels made of an image and one or more text entries. There are no Event speakers, character portraits, speaker positions, facial expressions, or Visual Novel dialogue staging.
3. A click while text is animating reveals the current text immediately. A click after reveal advances the local text/panel cursor.
4. After all pre-choice panels are shown, reveal the authored choices.
5. The player selects a choice, then selects one employee from the Core-projected eligible candidates. The employee portraits are performer-selection UI, not characters appearing in the Event scene.
6. Selecting an employee enables the progress button. Choice and employee selection are local and non-authoritative until that button is pressed.
7. Pressing progress sends one scene-qualified resolve command. Core rolls exactly one d20 and commits the performer, raw roll, outcome, and choice resolution atomically.
8. A successful result performs the selected choice's authored action, such as granting an item or starting the existing battle flow.
9. A failed result enters the authored failure panel sequence. After its text is fully shown, the next player click advances/completes the Event, and normal node-consumption rules consume it.

Initial d20 policy:

- The initial check is pure `1d20`. Employees, level, equipment, effects, trust, trauma, injuries, and other state add no numeric modifier.
- Do not add dormant ability-score, flat-modifier, bonus-die, advantage/disadvantage, reroll, or inspiration fields. Those require later playtest evidence and a separate approved extension.
- Natural 1 produces `critical_failure`; natural 20 produces `critical_success` regardless of the authored required roll.
- For natural 2 through 19, `raw_roll >= required_roll` succeeds and a lower roll fails.
- Initial authored `required_roll` is an integer from 1 through 20. Values outside the die domain fail live loading.
- `critical_success` uses the normal success action and `critical_failure` uses the normal failure sequence in the initial version. The distinct result kind exists for UI feedback and future evidence, not separate initial rewards or penalties.
- A check may be attempted exactly once. There is no employee swap after resolution, reroll, or alternate-choice retry.
- Only choices with a meaningful authored success/failure branch use this check flow. Plain narrative progression does not roll.

Authority and persistence:

- Core's persisted Event session is authoritative for node/Event/scene identity, committed choice, selected employee, raw roll, resolved outcome, once-only effect commit, pending Event-started combat, failure presentation state, and completion.
- The resolve command carries `node_id + event_id + current_scene_id + choice_id + employee_uuid`. Core revalidates the current session, choice, and employee eligibility before rolling.
- RNG is Core-owned. Unity never pre-rolls, chooses a result, or calculates a hidden modifier.
- Core persists the roll/result before returning it. Reconnect, duplicate delivery, repeated button input, and a later state snapshot return the same committed result and never consume RNG again.
- Before progress is pressed, changing the locally selected choice or employee has no Core mutation. A reconnect may clear that uncommitted UI selection.
- The selected employee identifies the performer and is retained for result presentation/idempotence only. The initial contract does not automatically target that employee with rewards, injury, trauma, or other personal effects.
- Success effects, the committed-roll marker, resulting scene/action transition, and immediate node completion where applicable form one atomic transaction.
- Failure is a normal gameplay outcome, not a command error. Failure completion occurs only after the failure panels are presented and the final advance command succeeds.

Event-started combat:

- A successful choice may start an encounter through the existing battle setup/update/result pipeline. It does not create an Event-only combat protocol.
- The Event choice, performer, and d20 result remain committed while the battle is active and through reconnect/retry paths; the player cannot reroll the Event check.
- Battle victory, defeat, retry, `RetreatBattle`, checkpoint loading, and node completion follow the accepted G14/G15 contracts. Battle rewards remain owned by the battle/reward pipeline rather than an Event-specific duplicate reward policy.
- Whole-stage `RetreatBattle` restores the battle's pre-entry state, which is after the Event check was committed; it does not return to employee selection or roll again.

Event versus Visual Novel:

- Event is not a Visual Novel presentation mode. Remove the `EventSceneEntered` Visual Novel trigger source/event, Event-to-VN runtime calls, tests, and canonical promises; live data does not use them.
- Global Visual Novel remains a separate system for actual Visual Novel content and does not serve as an Event fallback.

Authoring and validation:

- Event authoring defines ordered presentation panels, checked choices, required rolls, success actions, and failure presentation/termination.
- A checked choice must have exactly one success action contract and one failure presentation path. Unsupported or missing branches fail live loading.
- Remove immediate/staged `ApplyBossOmenStepResult`, Event-specific combat reward-commit variants, implicit Event IDs, missing-encounter panics, and other panic-only or duplicate-authority schema.
- Live loading rejects invalid IDs, empty panels/text, invalid required-roll bounds, missing employee-check choice data, broken/cyclic/unreachable scene graphs, unsupported effects, and missing encounter/item/reward references.
- PD-03 does not decide Event frequency, repeat suppression, rarity, weighting, or content rotation. Those remain separate content/progression policy.

Required completion evidence:

- Core tests cover natural 1/20, the `>=` boundary for 2..19, no modifiers, exactly one RNG consumption, duplicate-command idempotence, stale scene/choice/employee rejection, and atomic success/failure commit.
- Flow tests cover text reveal/advance, choice appearance only after pre-choice presentation, local choice/employee selection, progress enablement, successful item grant, successful battle start, failure panels, final click completion, and normal node consumption.
- Recovery tests prove reconnect/checkpoint/battle retry cannot reroll or replace the committed employee/choice/result.
- Removal tests/searches prove no Event-to-VN trigger path or Event-specific duplicate battle reward policy remains.
- Live validation loads the actual Event/map bundle and rejects malformed check/presentation/action references before play.
- Server/Unity tests consume actual Rust snapshots/results and complete the full Event flow without mock-only or Raw-JSON agreement.

### Accepted PD-04 Illustrated Event DTO Contract

Canonical envelope and ownership:

- Root snapshot retains `{ type: "event", node_id, event }`. `EventState` command results and `selected_event.event` serialize the same canonical inner DTO.
- Core constructs the typed DTO. game_server forwards it without Event-specific renaming, flattening, enrichment, or JSON repair.
- The following authoritative state snapshot decides the final route/state. A command result may only accelerate rendering of the same committed result.

Canonical inner shape:

```text
event
  event_id
  current_scene_id
  presentation
    panels[]
      image_id
      text_ids[]
  interaction
    type: advance
    | type: employee_check_choices
      choices[]
        choice_id
        label_id
        required_roll
        eligibility
          enabled
          disabled_reason_code?
        employee_candidates[]
          employee_uuid
          eligibility
            enabled
            disabled_reason_code?
  committed_check?
    choice_id
    employee_uuid
    raw_roll
    required_roll
    outcome: failure | success | critical_failure | critical_success
```

Presentation semantics:

- Replace provisional `background_id + script_ids/page_ids` with ordered `panels[] { image_id, text_ids[] }`, giving each image an explicit ordered text group.
- Every panel has one valid Event image resource and at least one valid localized text resource. Reusing the same image ID across panels is explicit authoring, not implicit carry-forward fallback.
- Unity owns only the local panel/text reveal cursor keyed by `(node_id, event_id, current_scene_id, committed_check identity)`. Same-state snapshots preserve it; a new authoritative scene/result or fresh reconnect starts that presentation sequence at its first entry.
- Click-to-reveal and click-to-advance do not send a Core command for every text entry. Unity sends `AdvanceEventScene` only when the authoritative `advance` interaction is reached after local presentation.
- There is no Event `speaker_id`, portrait, character position, expression, camera, or Visual Novel script field.

Choice, employee, and command semantics:

- `employee_check_choices` is structurally distinct from `advance`. Choices are displayed only after Unity finishes the current pre-choice panels.
- Each choice carries its authored `required_roll`, typed PD-02 choice eligibility, and typed per-employee candidate eligibility. Unity joins `employee_uuid` to the authoritative roster for the bottom performer portraits.
- The initial DTO has no employee ability value, modifier list, final roll value, advantage state, bonus dice, success-action preview, combat preview flag, reward hint, penalty hint, or free-form risk field.
- Unity keeps pre-confirmation choice/employee selection locally. The progress button is visible/enabled only when both current typed eligibilities permit resolution.
- Replace resolving `SelectEventChoice` with one typed `ResolveEventChoiceCheck` command carrying `node_id`, `event_id`, `current_scene_id`, `choice_id`, and `employee_uuid`. Do not retain both command shapes.
- `committed_check` appears only after Core accepts and resolves the command. It contains the raw d20 and distinct outcome needed to replay the result UI without rerolling.
- Root PD-02 `action_capabilities` owns command-family enabled/reason state; choice and employee entries own payload eligibility. Unity does not infer either from roster stats or Event effects.

Success and failure projection:

- A successful immediate grant may return its typed effect/result and then complete the node atomically. A successful combat action transitions to the existing battle projection while the Event check remains persisted in the run session.
- A failure transitions to an Event scene whose `presentation.panels` contain the failure text/image and whose interaction is `advance`; the final `AdvanceEventScene` completes/consumes the node.
- Critical result kinds do not introduce separate initial effect DTOs. Critical success follows the success action; critical failure follows the failure presentation.

Unity and resource contract:

- Unity implements a dedicated typed Event reader/model, route, presenter, local choice/employee selection state, progress command builder, d20 animation, and result/failure presentation.
- Generic `SelectedEventModel.Raw` parsing, dictionary fallback, raw-ID display, `Unknown` routing, and Visual Novel fallback are forbidden.
- A cross-repository manifest covers every live Event image, text, and choice-label ID. Missing resources fail build/release validation before gameplay.

Goal ownership:

- G02 owns shared envelope/canonical-document alignment where the generic selected-event boundary overlaps.
- G03 owns Event authoring/runtime/session/roll DTOs and commands plus the Unity Event route and interaction flow.
- G06 supplies common capability/eligibility types and reason vocabulary; G03 supplies Event-specific validators.
- G12 owns live-bundle and cross-repository resource-reference validation seams.
- G07/G14/G15 continue to own shared battle transport/result/rollback behavior consumed by successful Event combat actions.

Required completion evidence:

- Exact Rust serialization tests cover root Event snapshot, unresolved choice state, committed roll, failure presentation, success action, and `EventState` result with one canonical inner shape.
- Unity consumes actual Rust fixtures for panels, choices, employee candidates, progress enablement, d20 result animation, failure advance, item success, and battle transition.
- Tests prove no modifier fields exist, natural criticals override the threshold, `raw_roll == required_roll` succeeds for 2..19, and committed results survive duplicate/reconnect paths.
- Presentation tests prove click-to-reveal, local text/panel advance, choices hidden until presentation ends, and no per-line Core command.
- Live RON and canonical Unity documents migrate in one direction without aliases; old Event background/script/page/preview/selectable/SelectEventChoice/VN fields are rejected or absent from active contracts.
- Manifest tests pass for the complete live Event bundle and fail for missing image/text/label resources.

### Accepted PD-06 Global Target Reach and Area Delivery Contract

Canonical meaning and ownership:

- Global reach means that cast targeting may select any currently valid authored unit target on the battlefield. It does not mean that every tile or unit is affected.
- Cast targeting is the sole owner of target kind, selection rule, reach, and ground/air eligibility. A step that reuses the cast target does not re-author or revalidate a second range contract.
- `TileArea` is the sole owner of the actual affected-tile footprint. Its concrete pattern or resolved preset is required and non-empty.
- Remove the legacy use of one `defense_tile_range` as cast reach, target-candidate filter, and delivered hit footprint. Do not retain dual fields, aliases, or fallback interpretation.

Target-centered area execution:

- The stationary final boss may select a valid unit anywhere on the battlefield without movement or facing.
- At successful cast selection, Core captures the selected unit's tile as the immutable delivery anchor.
- The target-centered footprint is projected from that captured tile, clipped to valid battlefield tiles, and filtered by the skill's authored hit policy.
- A selected unit dying or withdrawing before resolution does not cancel or relocate the attack. Delivery still resolves at the captured tile and may hit any other eligible unit in the footprint.
- Deployed units do not move under the current game rules. Do not introduce cast-time versus resolve-time tracking, homing, or follow-target policy until an actual movement mechanic requires it.
- Symmetric footprints require no caster facing. If a future asymmetric footprint needs orientation, it must author an explicit orientation policy rather than silently depend on a possibly absent current facing.

Selection, projection, and execution:

- Core owns deterministic target eligibility, the authored selection rule, tie-breaking, and RNG. Unity does not infer or replace them.
- A normal rule such as Nearest does not silently become area-hit maximization. A skill that should prefer the anchor producing the most useful hits authors an explicit area-aware selection rule.
- Candidate preflight, AI usefulness, attack telegraph geometry, authoritative affected-tile projection, and runtime execution use the same geometry and hit-eligibility primitives.
- Global reach, target-centered TileArea, and a true whole-field delivery remain distinct contracts.

Authoring and validation:

- Reject TileArea without a concrete footprint, Global reach with a competing cast-range pattern, and reused-target steps that redefine target reach or air eligibility.
- Reject incompatible anchor, orientation, tracking, or hit-filter combinations during live bundle loading.
- Remove the accepted-but-empty `WholeFieldValidTiles + TileArea` legacy shape. Live data currently contains no use requiring migration compatibility.
- Catalog/snapshot DTOs project target reach separately from delivery footprint. Unity consumes the typed separation and does not reconstruct it from legacy range fields.

Deferred expanding field wave:

- A future final-boss delivery may originate at the caster's reference tile and expand radially by battlefield-space distance until it covers the field.
- The wave is circular, crosses void and invalid tiles without being blocked, activates only valid tiles, and applies effects when its front reaches an occupied valid tile. One unit is hit at most once per wave unless a later skill explicitly authors different repeat semantics.
- The wave stops while battle time is paused and advances according to battle speed. Its state is battle runtime only; normal saves occur outside battle, and battle-stage abandonment discards it with the entire attempt.
- This turn does not authorize wave runtime, RON, snapshot, or Unity implementation. Keep the delivery architecture extensible, record the behavior as a follow-up candidate, and do not add an enum/schema variant that can load but not execute.

Required completion evidence:

- A stationary no-facing caster globally selects a far valid unit, then hits only the captured target-centered footprint rather than the caster area or whole field.
- Target death/withdrawal before resolution preserves the captured tile; edge/void clipping, ground/air filters, and deterministic tie-breaking match projection and execution.
- Area-aware selection, when explicitly authored, evaluates the complete projected footprint; ordinary selection rules retain their authored meaning.
- Live loading rejects every removed mixed/empty schema, and actual Rust catalog/snapshot fixtures prove distinct reach and footprint consumption in Unity.
- Searches and tests prove no compatibility alias, duplicated step range authority, or unimplemented field-wave variant remains.

### Accepted PD-07 Explicit Per-Skill Targeting Authority

- A basic attack uses the equipped weapon's `targeting_profile`.
- Every skill independently and explicitly authors its cast-target selection rule. Two skills may intentionally reference the same reusable preset, but neither inherits through omission.
- Missing selection, implicit weapon inheritance, historical fallback, and simultaneous weapon/skill authority fail strict live loading.
- Changing weapons does not change an equipped skill's target-selection behavior.
- PD-06 cast targeting owns the skill's target kind, explicit selection, reach, and ground/air eligibility. G10 implements the authority, G09 consumes it for Global reach, and G18 updates examples.

### Accepted PD-08 Current Enemy Preview Boundary

- Do not expose an enemy range-preview feature in the current game. Remove its Core normal/checkpoint DTOs, server transport assumptions, Unity readers/rendering, mocks, tests, and empty compatibility fields.
- This does not remove already executing actions, explicitly authored attack telegraphs, projectile/area presentation, battlefield hazards, or the visible boundary of an active wave.
- Employee placement and allied skill targeting previews are outside this removal.
- If playtesting later proves enemy range preview necessary, introduce it as a new intentional DTO and UX revision. Do not reserve a dormant field now.

### Accepted PD-09 Strict Generated Live-RON Contract

- A dedicated authoring program generates gameplay RON. Human-edit convenience does not justify permissive runtime parsing.
- Require an exact supported schema version, canonical field names and enum variants, every semantically required field, unique canonical IDs, valid numeric/cardinality bounds, valid combinations, and complete cross-bundle references.
- Reject unknown or removed fields, aliases, duplicate IDs, first-match duplicate behavior, unsupported/runtime-panic variants, invalid placeholders, and partial-bundle loading.
- Defaults exist only for intentionally optional gameplay meaning, not to conceal missing generator output. The strict writer should emit required values explicitly.
- The authoring program validates before export and Core independently validates before play. Prefer a shared schema/validator authority over two drifting rule implementations.
- Diagnostics identify the source file, domain, object ID, field path, and cause. Save-data migration is a separate concern and does not weaken live content parsing.

### Accepted PD-12 Battle-Record Removal and Permanent Encounter Compendium

Battle history removal:

- The game creates no player-facing or authoritative gameplay battle history. Remove persistence, checkpoint copies, public DTO/UI, identity/key policy, and legacy tests.
- Story documents are an independent collection system. Optional developer combat traces are non-authoritative, live outside save/checkpoint/gameplay state, and can never fail or alter an authoritative transition.
- Active-battle event/unit-instance facts may support simulation and diagnostics but are not retained as player battle records after their runtime purpose ends.

Compendium identity and commit:

- Maintain a game-global set of discovered canonical abnormality IDs and discovered corrosion profile IDs. Do not invent species grouping or variant hierarchy in the initial contract.
- During a combat-node attempt, collect only IDs actually spawned on the battlefield. Preview-only and authored-but-never-spawned wave entries do not qualify.
- Commit the collected discoveries only when victory or final defeat consumes the combat node. A non-consuming retry state and battle-stage abandonment do not register them.
- Node consumption and discovery addition form one reliable idempotent commit. Repeated IDs are set union and never duplicate entries.
- Persist only discovered IDs; name, image, description, and exposed combat information resolve from the current strict live compendium catalog.

Persistence semantics:

- Committed discoveries are monotonic game-global state. Run checkpoint load, older run-save load, new-run creation, RunFailed, and ordinary rollback never remove them.
- Do not put a rollback copy of this set in the run checkpoint or merge by replacement. Loading older state retains the union already committed in the global profile.
- Only an explicit whole-game/profile or compendium reset may remove discoveries.
- G16 removes battle records and isolates diagnostics. G29 owns global compendium persistence and node-consumption discovery commit, consuming G15's authoritative result/node transaction seam.

### Accepted PD-13 Safe Zone Research Delivery

- Research completion queues pending results during node activity. It does not deliver on entry to Support, HeadquartersContact, Shop, Reward, Maintenance, or any other category.
- Remove the Rust category allowlist and the misleading `deliver_research_if_safe_node` entry hook.
- Deliver every pending result automatically when Core commits the authoritative transition from resolved node content back to map/planning state that Unity presents as Safe Zone.
- Safe Zone remains a non-consuming Unity presentation/prepare space; a Unity-local screen-enter message is not authority and cannot trigger grants.
- Delivery and pending removal are atomic, idempotent, and included in the authoritative transition result for Safe Zone notification. Reconnect or repeated rendering cannot duplicate them.
- Event-started combat and other nested content wait until their owning node flow resolves and returns to planning. Gate transition delivers on arrival at the new Floor's planning/Safe Zone boundary. RunFailed does not fabricate a Safe Zone return.

### Accepted PD-18 Shop Session, Quantity, and Economy Addendum

- Every authored offer entry explicitly declares grant quantity. Stack-sale commands explicitly declare sale quantity; Core validates ownership and calculates the checked total quote. Equipment sale uses exact instance UUID and quantity one.
- A Shop session persists definition/context identity, the current page including purchased holes, purchased non-repeatable merchandise, used/remaining refreshes, and deterministic RNG continuation. Reopen and reconnect never regenerate stock.
- Exploration Shop completion discards its completed session. A Headquarters sub-Shop follows its parent Headquarters session lifecycle without regenerating when reopened inside that session. Run checkpoint load restores the captured Shop session.
- Initial refresh cost is the same authored `base_cost` for every successful use. Do not infer escalation from use count; a future cost curve requires an explicit schema extension.
- The previously accepted preset composition, stable offer count, full-page atomic refresh, optional selling/tags, strict loss-making resale, typed Core quotes/eligibility, and no-compatibility prototype removal remain binding.
- Exact initial merchandise, weights, buy/sell percentages, and high/low-value content remain incremental authored balance work.

A policy decision packet must include:

- the question,
- current runtime behavior,
- live data evidence,
- typed DTO and Unity effect,
- alternatives and trade-offs,
- recommended option,
- migration and test impact.

## Dependency Graph

```text
Phase 0 trustworthy baseline
  G04
    -> R00 phase review

Phase 1 authority and strict-data foundation
  G01 -> G06 -> G07
  G12 common strict loader/bundle authority
    -> R01 phase review

Phase 2 combat transaction and boss recovery
  G26 -> G15 -> G14 -> G08
    -> R02 phase review

Phase 3 Event vertical slice
  G02 -> G03 -> G19
  consumes G06/G07/G12/G14/G15
    -> R03 phase review

Phase 4 persistent/meta state and Safe Zone return
  G17 reproduction gate
  G23 reward/growth canonicalization
  G16 -> G29
  G21 after combat/Event/Gate return seams stabilize
    -> R04 phase review

Phase 5 targeting and battlefield contract
  G11 -> G10 -> G09
  G13 and G18 after runtime/schema cutover
  G20 evidence governance last
    -> R05 phase review

Phase 6 economy, inventory, Shop, and Artifact cutover
  G24 ordinary inventory/quantity authority
    -> G28 final Shop session/economy
       -> G27 Artifact removal
    -> R06 phase review

Phase 7 final governance and canonical synchronization
  G25
  all phase review reports
    -> final DCA/CPR/MSTR cross-check
    -> broad Core/server/Unity validation
    -> R07 master completion review

Deferred content lane
  G05 / PD-05 only after abnormality combat designs are approved
  G22 / PD-14 only after Equipment/Maintenance content and balance are approved
  each activated deferred phase receives its own completion review goal
```

Do not run children in parallel merely because they appear on separate lines. Parallel implementation is allowed only when the child PLAN proves there is no shared public DTO, live schema, state transition, Unity store/command consumer, save boundary, or durable policy. Read-only discovery for a later phase may proceed, but no later-phase implementation may merge before the current phase review passes.

## Execution Phases

| Phase | Scope                                                                  | Required children in order                            | Entry gate                                          | Completion review                                        | Status      |
| ----- | ---------------------------------------------------------------------- | ----------------------------------------------------- | --------------------------------------------------- | -------------------------------------------------------- | ----------- |
| 0     | Trustworthy baseline                                                   | G04                                                   | Master documents and accepted policy registry exist | R00 `cross_layer_phase_00_baseline_completion_review`    | complete    |
| 1     | Revision, capability, strict live-data foundation                      | G01 -> G06 -> G07; G12; G30 correction                | R00 passes                                          | R01 `cross_layer_phase_01_authority_completion_review`   | complete    |
| 2     | Employee attempt state, exact rollback, BossOmen transition/recovery   | G26 -> G15 -> G14 -> G08                              | R01 passes                                          | R02 `cross_layer_phase_02_combat_completion_review`      | complete    |
| 3     | Illustrated employee-check Event vertical slice and real VN separation | G02 -> G03 -> G19                                     | R02 passes                                          | R03 `cross_layer_phase_03_event_completion_review`       | complete    |
| 4     | Record removal, compendium, reward state, Safe Zone delivery           | G17 reproduction; G23; G16 -> G29; G21                | R03 passes                                          | R04 `cross_layer_phase_04_persistence_completion_review` | complete    |
| 5     | Enemy-preview removal and final targeting/range authority              | G11 -> G10 -> G09 -> G13/G18 -> G20; G31 correction   | R04 passes                                          | R05 `cross_layer_phase_05_targeting_completion_review`   | complete    |
| 6     | Infinite ordinary inventory, final Shop, Artifact removal              | G24 -> G28 -> G27 in one coordinated cutover campaign | R05 passes                                          | R06 `cross_layer_phase_06_economy_completion_review`     | complete    |
| 7     | Workflow/canonical sync and final cross-goal validation                | G25 plus final evidence/canonical updates             | R00..R06 pass                                       | R07 `cross_layer_master_final_completion_review`         | complete    |

### Phase 0 — Trustworthy Baseline

Implementation:

1. G04 restores the intended server fixture and regression baseline without weakening assertions.
2. Record dirty-worktree state, existing failures, exact test counts, available Unity runners, and actual Rust fixture generation paths.
3. Do not start policy implementation inside G04.

Exit conditions:

- Core and game_server baselines distinguish pre-existing failures from new failures.
- Required Unity fixture/test entry points are known and at least one real Rust-serialized fixture path is proven.
- Zero-test filtered commands and unavailable runners are explicitly classified.

R00 focus: verify that the baseline is reproducible and that later phases have trustworthy evidence gates rather than merely green documentation.

### Phase 1 — Authority and Strict-Data Foundation

Implementation order:

1. G01 performs the atomic Core/server/Unity `battle_state_revision` cutover.
2. G06 replaces root `allowed_actions` authority with common context capability and typed eligibility primitives.
3. G07 applies revisioned capability, expected-revision commands, reconnect, and resync to battle.
4. G12 establishes exact-version generated-RON loading, canonical diagnostics, duplicate/unknown/unsupported rejection, and the shared bundle-validation seam. Later domain goals must consume this seam rather than introduce their own lenient loaders.
5. R01 found that G07 battle context capability is not yet an execution gate and missing authority
   lacks stale-equivalent resync. G30 must correct both through one shared authority seam, then R01
   must be rerun.

Exit conditions:

- No battle mutation or complete battle projection escapes revision ordering.
- No Core execution or Unity presentation authority depends on the removed static/cached action tables or whitelist.
- Projection/preflight and execution share validators and typed reason vocabulary.
- The production loader fails invalid generated data before play and does not partially accept a bundle.
- Direct clients cannot execute a battle action omitted/disabled by the current revisioned
  capability, and both missing/stale battle authority receive typed non-mutating current resync.

R01 focus: review cross-layer ordering, capability parity, strict loader ownership, stale/gap handling, and searches for old authority/fallback paths.

### Phase 2 — Combat Transaction and Boss Recovery

Implementation order:

1. G26 creates attempt-local employee condition state, per-tactical-death idempotence, trauma-aware redeployment, and employee-level non-penalty aggregation.
2. G15 builds the explicit victory/defeat/battle-stage-abandonment transition table and exact pre-entry restore over G26 commit/discard semantics.
3. G14 implements BossOmen committed defeat, dedicated victory exit Gate, and checkpoint recovery policy on the stable result transaction.
4. G08 implements the final Unity recovery/roster command flow against PD-01/PD-02 and G14/G15 authority.

Exit conditions:

- Repeated tactical deaths apply exactly one penalty each, while XP/result rows/MVP remain employee-level.
- Every committed result is atomic; `RetreatBattle` restores every authoritative pre-entry mutation including attempt/RNG/consumption state.
- BossOmen victory opens a Gate without direct Floor advance; defeat enters RunFailed and exposes only valid recovery.
- Actual Unity commands consume the authoritative revision/capability contract.

R02 focus: execute the full state-transition matrix, repeated-death/redeploy flows, victory/defeat/abandonment, boss Gate, checkpoint recovery, and legacy fixed-60%/partial-rollback searches.

### Phase 3 — Event Vertical Slice

Implementation order:

1. G02 aligns the shared selected-event envelope and existing Unity wire identities needed by Event.
2. G03 implements PD-03 and PD-04 as one end-to-end flow: panels, choices, employee eligibility, one-shot d20, success action or failure presentation, idempotence, and node consumption.
3. G19 removes duplicate VN authoring authority and proves actual Visual Novel content remains separate from Event with valid Unity resources.

Exit conditions:

- One canonical Rust Event shape is consumed by a typed Unity route without Raw/VN fallback.
- Item success, battle-start success, failed check, final failure advance, reconnect, duplicate command, and battle abandonment/retry follow the accepted shared transitions.
- Live Event RON and Unity resource references fail validation before play when invalid.

R03 focus: review the complete player flow rather than DTO shape alone, then cross-check Event-to-battle-to-Safe-Zone behavior against Phase 2.

### Phase 4 — Persistent/Meta State and Safe Zone Return

Implementation order:

1. G17 performs its required reproduction experiment and either implements a proven identity defect or records `Not applicable` with evidence.
2. G23 canonicalizes reward/growth behavior needed by result and research paths without inventing new balance.
3. G16 removes gameplay battle records and isolates optional developer traces from authoritative transitions.
4. G29 adds the monotonic game-global abnormality/corrosion compendium on G15's node-consumption commit seam; it does not reuse record schema.
5. G21 removes category-entry research delivery and delivers pending research on the authoritative planning transition presented as Safe Zone.

Exit conditions:

- No player/save/checkpoint battle history remains; diagnostic failure cannot affect gameplay.
- Actually spawned encounter identities commit to the global compendium only when the combat node is consumed and survive ordinary load/rollback/new-run flows.
- Every node/Event/combat/Gate return uses one Core-owned Safe Zone delivery boundary without Unity-local grant authority or duplicate delivery.

R04 focus: inspect save/checkpoint/global-profile boundaries, atomic node consumption, old-save union behavior, debug isolation, and every planning-return path together.

### Phase 5 — Targeting and Battlefield Contract

Implementation order:

1. G11 removes enemy range-preview projection/Unity behavior first, reducing the public range surface while retaining actual telegraphs and hazards.
2. G10 establishes explicit per-skill selection authority and removes weapon inheritance/fallback.
3. G09 separates Global target reach from required delivery footprint, captures the selected target tile, and unifies AI/preflight/runtime geometry.
4. G13 and G18 update battlefield/canonical examples only after runtime and live schema cut over.
5. G20 proves live catalog evidence semantics against the final schema.

Exit conditions:

- No enemy preview compatibility field/reader/mock remains.
- Basic attacks use weapon profiles; skills explicitly author selection and fail strict loading when incomplete.
- Global reach never means whole-field damage; target-centered TileArea resolves from the captured tile after target death/withdrawal.
- Core projection, AI usefulness, telegraph geometry, and execution share the same range primitives.

R05 focus: use live RON, far-target/no-facing, target disappearance, edge/void, air filter, deterministic selection, Unity fixture, and legacy mixed-range searches.

### Phase 6 — Economy, Inventory, Shop, and Artifact Cutover

Implementation order:

1. G24 establishes the final ordinary-inventory transaction boundary: definition-ID consumable stacks, equipment instances, explicit quantities, reserve high-water growth, and Core/Unity projection. Artifact remains an isolated legacy subsystem only until G27 and is not adapted into the new inventory.
2. G28 implements the final Shop against G24 identity/quantity authority, removes Artifact merchandise dependencies, and migrates deterministic session, pool, refresh, sale, quotes, checkpoint, and Unity behavior.
3. G27 removes the now-unreferenced Artifact subsystem from live data through runtime/server/Unity/tests without replacement.

These three goals may use one coordinated integration branch because an intermediate mainline state could break Shop/inventory contracts, but each keeps its own source-of-truth boundary, documents, tests, and completion review. Do not create a public dual inventory or compatibility schema.

Exit conditions:

- Ordinary grants never fail for occupied slots; stack/equipment quantity semantics are shared by rewards, Event, Shop, save, and Unity.
- Shop reopen/reconnect/checkpoint behavior is deterministic; refresh/sale/no-arbitrage and atomic quantity commits match Core projection.
- No Artifact code, data, DTO, reward/admin, Shop, stat/trigger, mock, document, or ignored test remains.

R06 focus: review the entire acquisition-to-storage-to-sale loop with actual live data and Rust-to-Unity fixtures, then perform a repository-wide Artifact/legacy Shop/fixed-capacity audit.

### Phase 7 — Final Governance and Canonical Synchronization

Implementation:

1. G25 verifies goal lifecycle/probe discoverability and completion evidence governance.
2. Reconcile DCA, CPR, MSTR, PD, child, and Phase registries.
3. Move implemented durable gameplay/wire policy into the nearest canonical documents and mark historical goal text as history rather than authority.
4. Run final broad Core, game_server, live-data, serialization, Unity EditMode/PlayMode, and gameplay-flow validation.

Exit conditions:

- R00..R06 all pass and every correction goal is closed and re-reviewed.
- Every active-scope finding has exactly one terminal disposition and implementation owner.
- No goal document is the only source for a live gameplay or wire contract.
- Deferred G05/G22 remain explicitly user-deferred with triggers, or have been activated and reviewed separately.

R07 focus: apply the full completion guide across Phase reports and cross-component seams. R07 may mark the active master scope complete only when no required correction, unresolved policy, missing validation, or document-only implementation remains.

### Deferred Content Lane

- G05/PD-05 starts only after per-abnormality combat designs and range values are approved.
- G22/PD-14 starts only after concrete Equipment/Maintenance content, recipes, E.G.O growth, and balance are approved.
- Neither child is filled with placeholder values to make the active phases appear complete.
- When activated, each receives normal child documents and a dedicated completion review goal using the same procedure below.

## Per-Child Startup Gate

Every child goal must begin by:

1. creating its PLAN.md, EXPERIMENTS.md, and EXPERIMENT_NOTES.md,
2. recording its exact primary finding IDs,
3. reading the current runtime and live-data path before documents,
4. identifying every typed DTO, server mapper, and Unity consumer in scope,
5. running and recording a baseline,
6. listing in-scope and out-of-scope behavior,
7. recording unresolved policy gates,
8. defining user-visible completion conditions,
9. defining focused and broad validation commands.

If a focused command matches zero tests, record it as a failed experiment and replace it with a command that runs the intended test.

## Per-Child Completion Review Gate

A child does not become complete when its implementation checklist is checked.

Immediately after implementation, run [goal_completion_review_guide.md](../../goal_completion_review_guide.md) against:

- current runtime behavior,
- production live data,
- exact serialized DTO shape,
- game_server message ordering,
- actual Unity consumer behavior,
- user-visible tests,
- validation and negative tests,
- legacy and compatibility searches.

Use the exact guide verdicts:

- `Complete`,
- `Partially complete`,
- `Document-only complete`,
- `Policy decision needed`,
- `Not applicable`.

Also record long-term fit (`High`, `Medium`, or `Low`) and improvement class (`none`, `follow-up refactor`, or `immediate correction recommended`). A dependent child may not begin until its prerequisite review verdict is `Complete` or `Not applicable`, with no `immediate correction recommended` finding.

## Per-Phase Completion Review Gate

Finishing the last child in a Phase changes the Phase to `review_required`; it does not complete the Phase. Create the review goal named in the Phase table with the mandatory files:

- `docs/goals/<phase_review_goal>/PLAN.md`,
- `docs/goals/<phase_review_goal>/EXPERIMENTS.md`,
- `docs/goals/<phase_review_goal>/EXPERIMENT_NOTES.md`,
- `docs/goals/<phase_review_goal>/COMPLETION_REVIEW.md` for the guide evidence output.

The review goal is read-only by default. It follows [goal_completion_review_guide.md](../../goal_completion_review_guide.md) and must:

1. collect every Phase child, accepted policy, expected user-visible behavior, runtime/data/DTO/Unity ownership, and claimed validation,
2. review every completed child item using the guide's evidence template and source-of-truth order,
3. search old names, fields, variants, aliases, fallback, compatibility, ignored tests, mock-only agreement, and debug bypasses,
4. run focused validation for each child and the smallest broad suite that exercises the complete Phase blast radius,
5. perform the guide's cross-component pass across children, live RON, server ordering, Unity consumption, save/checkpoint state, and canonical documents,
6. produce an item-by-item verdict table, detailed risky/incomplete evidence, cross-component findings, immediate correction candidates, follow-up refactors, commands/results, and policy questions,
7. update this master Phase table with the review goal link, verdict, and remaining risks.

A Phase passes only when:

- every required item is `Complete` or evidence-backed `Not applicable`,
- no item is `Partially complete`, `Document-only complete`, or `Policy decision needed`,
- every item has `High` or acceptable `Medium` long-term fit,
- no item has `Low` fit or `immediate correction recommended`,
- all required focused, live-data, serialization, gameplay, server, and Unity validation for the Phase passed,
- the cross-component pass found no competing source of truth, stale live schema, mock-only seam, or unowned correction.

An acceptable `Medium` fit must have no current user-visible/contract risk and must register a bounded follow-up owner and trigger. It cannot hide a compatibility layer or required cleanup.

If the Phase does not pass:

1. set the Phase status to `correction_required`,
2. create bounded correction child goal documents for each independent source-of-truth seam,
3. do not silently implement broad fixes inside the review goal,
4. complete and individually review the correction goals,
5. rerun the same Phase completion review and append the new evidence rather than overwriting the failed review history,
6. block implementation merge for the next Phase until the Phase passes.

Read-only investigation for later phases may continue while corrections are underway, but no later Phase may claim implementation completion or merge a conflicting public-contract change.

## Common Implementation Rules

- Choose the smallest coherent long-term structure, not the smallest patch.
- Do not expand a child beyond its source-of-truth seam.
- Do not add compatibility aliases, dual fields, fallback paths, ignored legacy tests, or test-only production branches.
- Remove obsolete code, data, DTO fields, mocks, tests, and documents together after proving they have no live consumer.
- Record failed approaches and why they failed in the child EXPERIMENTS.md.
- Record out-of-scope improvements in the child EXPERIMENT_NOTES.md and move on.
- Do not silently choose a DTO shape, live schema, migration, UX, balance, content deletion, failure meaning, reward timing, or consumption timing.
- Policy questions produce policy_decision_required with Implementation complete: false.

## Common Test Ladder

Each child selects the applicable subset and records exact commands and test counts.

1. smallest focused behavior or contract test,
2. related integration test,
3. live RON loading and negative validation test,
4. exact Core-to-server serialized DTO test,
5. actual Rust JSON consumed by Unity reader or command test,
6. gameplay success, failure, retry, battle-stage abandonment (`RetreatBattle`), unit withdrawal, reconnect, or recovery flow,
7. cargo check for affected packages,
8. Core broad tests,
9. game_server broad tests,
10. Unity EditMode/PlayMode and required probes.

Compile success alone is not behavior proof. Mock-to-mock agreement without typed Core JSON is not wire-contract proof.

## Master Cross-Goal Review

After R00..R06 pass and all active-scope children reach a terminal review verdict, R07 performs the master cross-goal review:

1. Re-read both audit inputs.
2. Verify all 62 normalized entries have one terminal disposition.
3. Verify no finding was silently dropped or counted as complete only because its document changed.
4. Inspect shared DTO, live schema, identity, state transition, and Unity files for conflicting child changes.
5. Verify all policy decisions were promoted to the nearest durable canonical document.
6. Search for legacy aliases, dual schemas, fallback paths, stale mocks, and ignored tests.
7. Run broad Core and game_server validation.
8. Run required Unity tests and probes or explicitly record why the master cannot complete.
9. Recheck every Phase review's remaining risks and follow-up owners; do not rely on its old pass verdict if later phases invalidated an assumption.
10. Record the final cross-goal review in R07 `COMPLETION_REVIEW.md` and this master EXPERIMENTS.md.
11. Update the Phase and child progress tables and only then set Status: complete.

## Master Completion Conditions

This master is complete only when:

- every DCA, unique CPR, and MSTR finding is complete, policy_accepted_no_change, not_applicable, or excluded with evidence,
- no policy decision remains unresolved,
- every child has passed its completion review gate,
- every activated Phase has passed its dedicated completion review and no later Phase invalidated that evidence,
- Critical and High user-flow findings are closed,
- Core, server, and Unity use the same serialized contracts,
- live RON changes are protected by startup validation and gameplay tests,
- Core broad tests pass,
- game_server broad tests pass,
- required Unity EditMode/PlayMode and live probe validation pass,
- no unapproved compatibility alias, fallback, or dual schema remains,
- durable canonical documents match final behavior,
- temporary audit notes are retired or explicitly superseded,
- Implementation complete is true.

A policy-decision report is a valid child deliverable but is not implementation completion. The child remains policy_decision_required until the user decides and the child is resumed or explicitly retired.

## Master Stop Conditions

Stop the affected child and report instead of guessing when:

- a Unity-facing DTO shape must change,
- live RON schema or content values must change,
- save-data migration is required,
- UX meaning or balance values are undecided,
- existing content must be removed or replaced,
- failure, reward, battle-stage abandonment, checkpoint, or consumption timing changes,
- two children require conflicting changes to the same public contract,
- required external Unity validation cannot be run and is necessary for completion,
- progress would require reverting unrelated dirty-worktree changes.

Within the current Phase, an unrelated blocked child does not prevent work on another child only when the PLAN proves the boundaries are independent. Later-Phase implementation remains gated by the current Phase review; only read-only discovery may proceed early.

## Current Progress

| Child | Status                | Implementation complete | Review verdict                          | Next action                                                                                                                                       |
| ----- | --------------------- | ----------------------- | --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| G01   | complete              | true                    | Complete / High                         | Revision/cursor/order/reconnect contract closed; preserve as G06/G07 foundation                                                                   |
| G02   | complete              | true                    | Complete / High                         | Preserve exact shared envelope and generic wire identities for G03                                                                                |
| G03   | complete              | true                    | Complete / High                         | Preserve the typed Event vertical slice and Phase 2 battle composition                                                                           |
| G04   | complete              | true                    | Complete / High                         | Preserve the trustworthy server fixture/startup baseline established by R00/G12                                                                   |
| G05   | deferred_by_user      | false                   | —                                       | Do not start until detailed abnormality settings are approved; then remove the temporary fallback                                                 |
| G06   | complete              | true                    | Complete / High                         | Typed common capability/preflight/root contract closed; preserve the explicit G07/domain ownership boundaries                                     |
| G07   | complete              | true                    | Complete / High after G30 and R01 rerun | Preserve corrected battle context and recovery authority                                                                                          |
| G08   | complete              | true                    | Complete / High                         | Preserve typed recovery/checkpoint/roster consumption and incomplete-runtime scope narrowing for R02                                               |
| G09   | complete              | true                    | Complete / High                         | Preserve strict cast reach versus TileArea footprint ownership and captured-anchor delivery                                                        |
| G10   | complete              | true                    | Complete / High                         | Preserve explicit skill-owned selection and basic-attack-only weapon targeting through G09                                                        |
| G11   | complete              | true                    | Complete / High                         | Preserve Player-only preview surfaces and opponent field absence                                                                                  |
| G12   | complete              | true                    | Complete / High                         | Exact schema v1, typed diagnostics, full validation/publication boundary closed; preserve for later domain schema extensions                      |
| G13   | complete              | true                    | Complete / High                         | Preserve screen-space HUD projection and wire-reserved Facility reachability wording                                                              |
| G14   | complete              | true                    | Complete / High                         | Preserve dedicated Gate and qualified RunFailed checkpoint authority for G08                                                                      |
| G15   | complete              | true                    | Complete / High                         | Preserve exact abandonment, fatal priority, once-only result, and typed recovery restriction for G14                                              |
| G16   | complete              | true                    | Complete / High                         | Gameplay battle history removed; preserve only battle-keyed non-fatal diagnostics and hand the clean boundary to G29                              |
| G17   | complete              | true                    | Complete / High                         | Preserve checkpoint/global identity separation and gameplay/log sequence independence                                                             |
| G18   | complete              | true                    | Complete / High                         | Preserve final reuse/retarget/TileArea/catalog-v2 examples as downstream documentation                                                             |
| G19   | complete              | true                    | Complete / High                         | Preserve trigger-only blocking and exact local Standard/Endless presentation                                                                     |
| G20   | complete              | true                    | Complete / High                         | Preserve derived v2 membership/delivery/reference evidence and named-test behavior boundary                                                       |
| G21   | complete              | true                    | Complete / High                         | Preserve the atomic completion-only planning-return delivery boundary                                                                             |
| G22   | deferred_by_user      | false                   | —                                       | Resume only when Equipment/Maintenance content scope and balance are ready                                                                        |
| G23   | complete              | true                    | Complete / High                         | Preserve canonical fixed reward, XP distribution, level/tier, and GrowthStack boundaries                                                         |
| G24   | complete              | true                    | Complete / High                         | Preserve equipment-instance/definition-stack authority, 50/10 reserve high-water, and Artifact isolation for G28/G27                             |
| G25   | complete              | true                    | Complete / High                         | Preserve durable lifecycle order, historical-goal classification, canonical routing and verified probe registry                                  |
| G26   | complete              | true                    | Complete / High                         | Preserve attempt-local employee condition and employee-level aggregation for G15                                                                  |
| G27   | complete              | true                    | Complete / High                         | Preserve complete deletion; do not reintroduce a replacement, compatibility field, reward/admin or battle path                                    |
| G28   | complete              | true                    | Complete / High                         | Preserve deterministic typed Shop sessions, checked economy, G24 transactions, and Artifact-free merchandise                                    |
| G29   | complete              | true                    | Complete / High                         | Preserve actual-spawn consuming commit, monotonic global union, strict catalog projection, and diagnostic isolation                              |
| G30   | complete              | true                    | Complete / High                         | MSTR-05/MSTR-06 corrected and R01 correction rerun passed                                                                                         |
| G31   | complete              | true                    | Complete / High                         | Preserve class-level deterministic pointer/root isolation; R05 correction rerun passed                                                            |
| R00   | complete              | true                    | Complete / High                         | Phase 0 passed independently; Phase 1 unlocked                                                                                                    |
| R01   | complete              | true                    | Complete / High after correction rerun  | Phase 1 closed; preserve as Phase 2 predecessor                                                                                                   |
| R02   | complete              | true                    | Complete / High                         | Phase 2 closed; preserve transaction/recovery boundary for Event work                                                                             |
| R03   | complete              | true                    | Complete / High                         | Phase 3 closed; preserve Event/VN separation for persistence and targeting work                                                                   |
| R04   | complete              | true                    | Complete / High                         | Phase 4 closed; Phase 5 G11 unlocked                                                                                                               |
| R05   | complete              | true                    | Complete / High after correction rerun  | Phase 5 closed; preserve G31 deterministic Unity validation foundation                                                                            |
| R06   | complete              | true                    | Complete / High                         | Phase 6 closed; preserve the one Core economy boundary and clean legacy audits                                                                   |
| R07   | complete              | true                    | Complete / High                         | Master closed; preserve terminal registry, deferrals, canonical ownership and current combined validation                                        |
| EX-01 | excluded              | false                   | not_applicable                          | No child directory                                                                                                                                |

The master and Phase 7 are Complete / High after R07 independently reviewed the current combined
state on 2026-07-14. No active child, phase review or correction remains. G05 and G22 remain
user-deferred under their explicit activation triggers; EX-01 remains not applicable.
