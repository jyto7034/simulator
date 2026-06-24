# Core Policy Map Progression Event Log Contract

## Objective

Implement the two immediate correction candidates from `docs/goals/core_policy_implementation_master_completion_review/AUDIT_REPORT.md`:

1. Remove the Unity-facing duplicate map progression id-list projection from `RunSnapshotDto.map_progression`.
2. Rename the battle `Timeline` type/JSON contract to an event-log name.

The implementation must follow the long-term policy direction. Do not keep compatibility fields, fallback paths, dual schemas, or old/new contract aliases unless a new user-confirmed policy explicitly requires them.

## Required Startup Protocol

At the beginning of this goal, before code search, edits, or validation, read and update:

- `docs/goals/core_policy_map_progression_event_log_contract/PLAN.md`
- `docs/goals/core_policy_map_progression_event_log_contract/EXPERIMENTS.md`
- `docs/goals/core_policy_map_progression_event_log_contract/EXPERIMENT_NOTES.md`

Also read the policy sources:

- `docs/goals/core_policy_implementation_master_completion_review/AUDIT_REPORT.md`
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- `docs/refactor_preparation_plan.md`

Treat actual runtime code, live RON/data, Unity-facing snapshot/command contracts, and latest policy documents as source of truth in that order.

## Confirmed Policy Decisions

### Map Progression Contract

`MapNodeDto.state` is the canonical Unity/server-facing source for map node availability and completion.

`RunSnapshotDto.map_progression` must not expose `available_node_ids` or `completed_node_ids` as a competing public projection. Internal runtime `MapProgression` may keep lists if they are necessary for progression calculation, but external snapshot consumers must derive availability/completion from node state.

### Battle Event Log Contract

The old `Timeline` type/JSON naming must be renamed to an event-log term. This is an intentional contract change.

Do not preserve the old `Timeline` contract as a compatibility alias. Update Rust types, serialized JSON contract, tests, validation wording, debug export naming where in scope, and server-facing references to use the new event-log terminology.

Recommended target naming:

- Rust domain type: `BattleEventLog`
- Entry type if renamed: `BattleEventLogEntry`
- Event enum: `BattleLogEvent`
- JSON field names should use `event_log` / `battle_event_log` rather than `timeline` where they represent the battle event log.
- Transport compression names should avoid `compressed_timeline`; prefer `compressed_event_log` or `compressed_battle_event_log`.

If the implementation discovers a naming split that would require a new Unity/server policy decision, record `사용자와 정책 논의 필요` in `EXPERIMENT_NOTES.md` and complete this goal with a policy-decision report instead of making an arbitrary choice.

## Scope

In scope:

- `MapProgressionSnapshotDto` and `RunSnapshotDto.map_progression` contract cleanup.
- Snapshot serialization/deserialization tests affected by the removed map progression lists.
- Server/admin/test callers that read map availability from snapshot progression lists.
- Battle event-log Rust type and field rename across core, tests, validation, debug/export code, and server transport.
- Unity/server-facing DTO and WebSocket payload field names affected by the event-log rename.
- Test updates that pin the new public contract and remove old contract expectations.
- Documentation updates only where needed to prevent old contract names from being treated as current policy.

Out of scope:

- Reworking internal `MapProgression` calculation storage unless necessary to remove external duplication.
- Unrelated movement, skill targeting, buff, reward, or item policy follow-ups.
- Cosmetic renames that do not affect the battle event-log contract.
- Broad Unity client code changes outside this repository unless a local contract fixture or server contract requires it.

## Plan

1. Re-read goal docs and policy sources.
2. Inventory all `MapProgressionSnapshotDto`, `map_progression.available_node_ids`, and `map_progression.completed_node_ids` usages.
3. Replace external map progression list consumers with `MapViewDto.nodes[].state` derivation or another already-canonical state source.
4. Remove `available_node_ids` and `completed_node_ids` from `MapProgressionSnapshotDto` serialization.
5. Add or update snapshot tests proving:
   - `RunSnapshotDto.map_progression` no longer exposes duplicate progression lists.
   - map availability/completion is still visible through per-node state.
6. Inventory all battle `Timeline` type, field, JSON, validation, debug export, server transport, and test references.
7. Choose the concrete event-log naming shape from the confirmed policy above, unless a new policy decision is discovered.
8. Rename the domain type and public contract without old compatibility aliases.
9. Update server transport fields such as compressed battle result attachments to use event-log naming.
10. Update validation messages, docs, tests, and fixtures that refer to the old public timeline contract.
11. Run focused validation after each small change group.
12. Run broad validation at the end.
13. Record all attempts, failures, fixes, and final validation results in `EXPERIMENTS.md`.
14. Record policy decisions, tradeoffs, and out-of-scope follow-ups in `EXPERIMENT_NOTES.md`.

## Validation Plan

Focused validation should include:

- Snapshot tests covering map progression DTO shape.
- Tests that exercise map node selection/completion through `MapNodeDto.state`.
- Battle event-log serialization/deserialization tests.
- Timeline/event-log validation tests, especially version rejection and battle result attachment tests.
- Server contract compile/check after DTO and transport rename.

Broad validation should include:

- `cargo check --lib`
- focused affected tests by module/name
- `cargo test --test ron_loading -- --test-threads=1`
- `cargo test --test live_skill_catalog_audit -- --test-threads=1`
- `cargo check -p game_server`
- `cargo test --lib -- --test-threads=1`
- `git diff --check`

If a command is skipped, record the reason in `EXPERIMENTS.md`.

## Completion Conditions

- `RunSnapshotDto.map_progression` no longer serializes `available_node_ids` or `completed_node_ids`.
- All external map progression availability/completion consumers in this repository use `MapNodeDto.state` or another canonical state source.
- Tests pin the absence of duplicate map progression lists from the external snapshot contract.
- The battle `Timeline` public type/JSON contract has been renamed to event-log terminology.
- Old `timeline`/`Timeline` public contract names are removed from current runtime/server-facing behavior, except historical docs or explicit migration notes.
- No old/new dual schema or compatibility alias remains for the renamed event-log contract.
- Focused and broad validation results are recorded.
- `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md` are updated with the final outcome.

## Final Outcome

Status: implemented and verified.

Completed:

- `MapProgressionSnapshotDto` now exposes only `current_node_id`; `available_node_ids` and `completed_node_ids` are no longer serialized through `RunSnapshotDto.map_progression`.
- Snapshot tests assert that both `map` and `map_progression` omit duplicate availability/completion id lists.
- Battle event-log runtime types and public fields were renamed from `Timeline`/`timeline` terminology to `BattleEventLog`/`event_log` terminology.
- Server combat-result attachment transport now uses `compressed_event_log`, `event_log_encoding`, and `event_log_gzip_base64`.
- Current policy baseline docs were updated to describe `BattleEventLog` as the official contract.
- Focused and broad validation passed; see `EXPERIMENTS.md`.

Remaining external follow-up:

- Unity client code outside this repository must be updated to consume the new event-log field names.

## Stop And Complete Conditions

Complete this goal with a policy-decision report instead of continuing implementation if runtime reading reveals a new required decision involving:

- Unity-facing DTO shape beyond the already confirmed map progression list removal and event-log rename.
- Save data migration that cannot be handled by removing legacy contract fields.
- UX meaning changes beyond field/name contract updates.
- Gameplay semantic changes hidden behind the rename.
- A conflict between live server transport needs and the no-compatibility policy.

When this happens, record `사용자와 정책 논의 필요` in `EXPERIMENT_NOTES.md`, list options and recommendation, and report the blocked implementation area.

## Non-Goals

- Do not preserve old contract names as aliases.
- Do not add debug/helper fields that recreate removed map progression lists.
- Do not use ignored tests to preserve legacy behavior.
- Do not update tests by only changing strings if the underlying public contract still exposes old fields.
- Do not refactor unrelated follow-up candidates from the completion review unless required to reduce this goal's blast radius.
