# Roster Display Order Authority Separation

Status: complete

Created: 2026-07-15

Last updated: 2026-07-15

## Goal

Remove employee-card display order from Core gameplay authority and its wire contract, then make Unity-local UUID order the only presentation authority and expose the approved sequential formation interaction.

User-visible problems:

- checkpoint load and `RetreatBattle` currently restore a display-only order,
- rearranging cards currently requires a Core gameplay command and capability,
- Node Preview exposes temporary up/down controls in the wrong context,
- Unity has no local order that survives authoritative snapshot replacement,
- SafeZone Item has no direct click-in-order formation interaction.

## Audit Finding

None. This Goal comes from an explicit user policy change on 2026-07-15. It supersedes only the roster-order portion of the completed DCA-H-13 / CPR-H-07 G08 work; it does not reopen recovery or checkpoint command policy.

## Target Repositories

- Core and Goal docs: `/mnt/f/work/simulator/core_enhanced`
- game_server: `/mnt/f/work/simulator/game_server`
- Unity: `/mnt/f/unity projects/ark`

## Current Evidence

The following was established by pre-implementation static discovery and must be revalidated after the policy gate because all target worktrees are heavily dirty.

- Core defines `RosterOrder`, stores it in `GameCoreState`, copies it into run checkpoint and battle-entry rollback state, and restores it on checkpoint load and `RetreatBattle`.
- Core exposes `MoveRosterUnit`, its capability/result, root `roster_order`, and employee `roster_slot`.
- Core deployment, cost, employee statistics, and battle rules use UUID-based roster/eligibility state rather than `RosterOrder`.
- No production live RON/data reference to roster display order was found.
- game_server consumes Core `PlayerBehavior`/`RunSnapshotDto` directly; its roster-specific code is a deserialize test and generated Rust fixture path rather than independent state.
- Unity has one `roster_order` reader, a typed command/gateway/handler path, Node Preview up/down controls, and mocks/tests for the old contract.
- Unity roster screens currently enumerate the server `roster.employees` array and have no committed local UUID order service.
- Unity currently has no `PlayerPrefs` or general local settings store. `GameWsClient` holds the authentication `playerId` privately and exposes no settings identity; `Application.persistentDataPath` is used only for the wire log.
- SafeZone Item uses a production roster action-card panel with reusable employee hit targets.

## Source Of Truth And Ownership

After completion:

- Core owns employee UUID, ownership, life, availability, `available_for_combat`, statistics, loadout, battle deployment, eligibility, costs, and deploy/withdraw commands.
- game_server forwards the reduced typed Core contract without roster-order reinterpretation.
- Unity-local presentation state owns employee-card order and the transient formation draft.
- Core snapshot order, checkpoint, rollback, and command results never authorize or restore presentation order.

## Invariants

- Employee ownership, life, availability, statistics, loadout, battle eligibility, cost, priority, command behavior, and outcomes do not change.
- `LoadRunCheckpoint` and `RetreatBattle` retain their gameplay restore semantics but cannot restore Unity-local display order.
- Formation clickability uses Core `available_for_combat`; Unity does not reconstruct that rule.
- Employee UUID is the only order identity. Names and array indices are not identity keys.
- Local ordering changes presentation only and never causes automatic deployment or gameplay priority.
- Event performer candidates and combat-result statistics keep their domain-projected ordering.
- No dual schema, alias, fallback, deprecated command, or hidden Core order remains.

## Scope

In scope:

- remove Core `RosterOrder`, fixed slots, accessors, synchronization, checkpoint/battle-entry copies, command/result/capability, `roster_order`, and `roster_slot`,
- remove corresponding game_server deserialize coverage and regenerate actual Rust JSON fixtures,
- remove Unity's old reader, action constant, command bus, gateways, handlers, payloads, Node Preview rows/arrows, mocks, fixtures, and tests,
- add one Unity-local UUID order with automatic employee add/remove/availability reconciliation,
- use that order on employee-card roster surfaces,
- implement the approved formation draft and click interaction on the SafeZone Item employee presentation surface,
- replace legacy tests with exact DTO-absence, reconciliation, rollback-boundary, and Unity interaction evidence,
- update current canonical Core and Unity documents.

Explicit non-scope:

- employee gameplay, deployment, checkpoint-use, recovery, BossOmen, reward, retry, or battle-resync policy changes,
- live RON schema/content or balance changes,
- Event candidate or combat-result sorting,
- cloud/cross-device synchronization,
- creation of final illustrated background, character art, animation, or unrelated Rest gameplay,
- unrelated transport, store, UI, or runtime refactors,
- cleanup of unrelated dirty files.

## Approved Policies

- Core roster display order and all related wire fields/commands are removed without compatibility.
- Unity owns local UUID display order.
- checkpoint load, battle abandonment, reconnect, and reordered root arrays do not replace committed local order.
- new eligible employees append; removed employees are pruned.
- an employee becoming unavailable leaves the active order and later numbers compact; becoming available appends to the active order.
- dead/unavailable employees are visible where the roster surface supports them but are not clickable and receive no active number.
- Node Preview roster rows and up/down controls are removed.
- entering formation mode starts an empty draft: every clickable unit is red and unnumbered.
- clicking an unselected unit appends it, turns it blue, and shows its one-based order at that unit's top-left.
- clicking a selected unit removes it; all later numbers decrement by one. Clicking it again appends it at the end.
- complete is enabled only when every currently clickable employee is assigned once; complete commits atomically and cancel preserves the previous committed order.
- formation mode owns unit clicks while active so normal unit actions do not fire simultaneously.

## Resolved Persistence Policy

The user approved the long-term recommendation on 2026-07-15:

- committed order survives Unity application restart,
- storage is device-local and scoped by the stable player/account UUID used for `/game` authentication,
- settings use a versioned JSON document under `Application.persistentDataPath`,
- player identity is injected explicitly into both transport/session and settings ownership rather than discovered through a device-global key,
- formation draft clicks are not persisted; only atomic completion commits,
- settings do not sync across devices and Core/game_server store no order,
- production JSON storage is behind an injectable boundary; tests use isolated memory storage,
- invalid local settings are explicitly validated and quarantined/reset with presentation-visible diagnostics rather than used as gameplay facts.

## Post-completion UX Placement Override

On 2026-07-15 the user first moved the formation editor from Rest to SafeZone Loadout, then explicitly selected the SafeZone Item subscene as the final surface:

- the SafeZone Item employee panel owns formation start, employee assignment, complete, and cancel,
- normal Item employee selection and Loadout long-press remain active outside formation mode,
- formation mode temporarily owns those employee-card clicks and suppresses Loadout long-press,
- leaving Item cancels the transient draft and preserves the committed order,
- Rest and Loadout expose no formation editing controls.

This supersedes only the original Rest-surface placement. Core/server authority separation, local persistence, eligibility, draft, and atomic-commit policies are unchanged.

## Implementation Scope And Sequence

After the policy decision:

1. Re-read all three Goal documents, revalidate runtime/data/DTO/server/Unity consumers and canonical docs, inspect target diffs, and record baseline failures.
2. Remove Core state, rollback copies, command/capability/result, and root/employee fields while preserving gameplay rollback and deployment tests.
3. Align game_server and regenerate the Rust-produced Unity fixture without adding a server-local replacement.
4. Add Unity-local committed order/reconciliation and migrate all in-scope roster presentation consumers.
5. Remove every Unity old-contract sender/reader/mock/UI path, including Node Preview arrows.
6. Add the approved formation draft, outline, badge, toggle-compaction, complete, and cancel behavior on the SafeZone Item employee surface.
7. Update canonical docs, run focused/broad/cross-layer validation, and audit legacy symbols and dirty-worktree preservation.

If the SafeZone Item employee surface lacks production unit hit targets or highlight/badge anchors, functional local-order migration may proceed only if independent of that UX; the Goal cannot be complete until the required formation interaction is implemented and validated or the user changes the completion condition.

## User-Visible Completion Conditions

- Core, server JSON, and Unity wire consumers contain no `RosterOrder`, `MoveRosterUnit`, `roster_order`, `roster_slot`, or `roster_slots` contract.
- checkpoint load, `RetreatBattle`, reconnect, and authoritative array changes do not revert Unity-local order.
- employee add/remove/availability transitions follow the approved append/prune/compact policy.
- Item Use, Loadout, Maintenance, Battle roster, and applicable Rest roster surfaces use the same local active order without changing gameplay facts.
- Node Preview shows no roster-order rows or arrows.
- Formation mode renders red unassigned units, blue assigned units, and exact top-left `1..N` badges.
- Reclicking an assigned unit removes it and visibly compacts later numbers.
- Dead/unavailable employees cannot be assigned.
- Complete/cancel are atomic and no Core command is sent.
- Focused and broad evidence proves unchanged gameplay, exact reduced DTOs, producer-consumer agreement, and real Unity interaction.
- Current canonical docs match the implemented boundary and no unapproved compatibility remains.

## Migration

- DTO migration: intentional breaking removal of the old command/result/capability and snapshot fields.
- Live RON/data migration: none expected; revalidate before implementation.
- Core save migration: none expected because known checkpoint/battle-entry copies are in-memory typed state; stop if durable serialized state is discovered.
- Unity local-data migration: add a new versioned, player-scoped local JSON settings document. There is no existing local schema to migrate and no one-time import from legacy Core `roster_order`.
- Compatibility: none approved. Core/server/Unity require a coordinated contract cut. Any transition window requires separate user approval with affected versions, duration, tests, and removal condition.

## Required Validation

Focused evidence must cover:

- exact Core root/employee JSON absence,
- removal of behavior/result/capability deserialization/serialization,
- unchanged checkpoint and exact `RetreatBattle` gameplay restore,
- unchanged deployment eligibility/cost and employee facts,
- actual Rust fixture consumption by Unity,
- local order initialization, root replacement, add/remove, availability loss/recovery, and duplicate/blank local input handling,
- common order across in-scope Unity roster surfaces,
- Node Preview legacy absence,
- formation append, reclick removal, compaction, completion, cancel, and disabled-unit clicks in EditMode and real PlayMode rendering/input.

Known focused baseline commands to revalidate before edits include:

```bash
# /mnt/f/work/simulator/core_enhanced
cargo test -p game_core roster_order --lib -- --test-threads=1
cargo test -p game_core move_roster_unit_ --lib -- --test-threads=1
cargo test -p game_core retreat_battle_restores_exact_pre_entry_snapshot_and_rng_continuation --lib -- --test-threads=1
cargo test -p game_core load_run_checkpoint_restores_visible_run_state_but_not_load_count --lib -- --test-threads=1

# /mnt/f/work/simulator/game_server
cargo run --example generate_action_capability_fixture -- "/mnt/f/unity projects/ark/Assets/Tests/Fixtures/action_capability_contract.json"
```

Final focused filters must target current replacement tests; zero-match legacy filters are not success. Unity Test Runner jobs and filters are selected after current tests/tooling are revalidated and must report actual nonzero counts.

Broad validation:

```bash
# /mnt/f/work/simulator/core_enhanced
cargo fmt --all -- --check
cargo test --all-targets

# /mnt/f/work/simulator/game_server
cargo check
cargo test --all-targets
```

Also required:

- full Unity EditMode suite,
- relevant full Unity PlayMode suite,
- actual checkpoint/retreat/reconnect flow using current Rust server data,
- scoped legacy search across Core, game_server, Unity runtime, mocks, fixtures, tests, and current canonical docs.

Skipped by design unless discovery changes scope:

- live RON loading/content tests, because static discovery found no production data dependency,
- cloud synchronization tests, because cloud persistence is out of scope,
- final art/animation validation, because asset creation is out of scope; functional scene interaction remains mandatory.

## Completion Audit Resolution

The implementation and required functional validation are complete. The final dirty-worktree concern was resolved by explicit user acceptance on 2026-07-15:

- `Assets/Screenshots/battlefield_live_validation.png` and `Assets/Screenshots/battlefield_overlay_inclusive_validation.png` were already modified at Goal start.
- Existing full/live PlayMode tests delete and regenerate those validation PNGs. They were executed before their dirty-output behavior was noticed, so the pre-run uncommitted bytes were not backed up and no alternate copy was found.
- Starting the required local game_server also appended Goal-run entries to the already-untracked `game_server/logs/app.log.2026-07-15` generated log.

The user confirmed that the previous generated artifacts were not important and approved retaining the regenerated PNGs and appended generated server log. No source, fixture, canonical document, or gameplay data outside the Goal was intentionally reset or overwritten. The preservation exception is therefore accepted and no further production modification is needed.

## Stop Conditions

Change status as required and stop affected modifications if:

- discovery proves this request contains independent contract boundaries that cannot be completed atomically,
- a durable save or deployed compatibility requirement is found,
- removal changes employee gameplay, checkpoint, retreat, deployment, reward, or battle meaning,
- Unity would need to infer Core eligibility,
- inaccessible external consumers are essential to safe contract removal,
- the real SafeZone Item interaction cannot be implemented or validated without unapproved asset/scope expansion,
- required Core/server/Unity baseline or final runner cannot execute,
- target dirty changes cannot be preserved safely.

## Canonical Documents

At completion, update the closest current Core runtime/rulebook statements and the external Unity wire/UX canonical documents. Historical G08 Goal records remain history and must not be rewritten as current policy.
