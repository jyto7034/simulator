# Core File Hierarchy Refactor Plan

## Objective

Refactor the current `src/game` file hierarchy so long-lived gameplay responsibilities live in clear modules instead of oversized root files, without changing public gameplay behavior, Unity-facing DTO shape, WebSocket contracts, or live RON schemas.

## Working Rules

- Prefer long-term structure over minimal patches.
- Read runtime code, live RON/data, Unity-facing snapshot/command contracts, then policy docs before deciding.
- Remove legacy only when it is truly obsolete; do not add compatibility layers without user approval and a removal condition.
- If a policy decision is discovered, stop the goal and report the question list.
- Do not run `git restore`, `git reset`, or copy historical git contents into current files.

## Phases

1. Split `src/game/world/admin.rs` responsibilities into admin submodules.
2. Split `src/game/world/tests.rs` by topic.
3. Split `src/game/combat_preview.rs` into a readable `combat_preview` hierarchy.
4. Move root combat preview/setup responsibilities from `src/game` into clearer sub-hierarchies.
5. Classify debug/export/backup directories and document the policy.
6. Run focused tests after each phase and broad checks at the end.

## Current Phase

Phase 1 is in progress. The first extraction target is the admin grant catalog because it has a distinct Unity-facing DTO contract and can be separated without changing command behavior.

## Completion Conditions

- `world/admin` has separated command/catalog/fixture/grant/test responsibilities.
- `world` tests are split by topic and remain discoverable.
- `combat_preview` responsibilities are split by DTO/build/template/validation concerns.
- Root `src/game` combat preview/setup files are moved or reduced to clear module entry points.
- Debug/export/backup directories are classified as live/test/dead and documented.
- No Unity-facing DTO, WebSocket command, snapshot, save, or live RON schema changes are introduced unless explicitly approved.
- Relevant focused tests and broad checks pass.
- If a user-policy decision is needed, the goal stops and reports the question list instead of guessing.

