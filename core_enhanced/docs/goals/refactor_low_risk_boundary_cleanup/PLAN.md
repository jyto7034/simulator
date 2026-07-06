# Refactor Low-Risk Boundary Cleanup

## Objective

Resolve the low-risk refactor findings from `docs/audit/combat_result_atomic_commit_notes.md` in one focused cleanup goal.

Scope:

- Remove shop `hidden_items` and `hidden_item_uuids` from Unity-facing selected-event snapshots while keeping them in core runtime state.
- Change boss omen terminal-distance calculation from DFS-like traversal to BFS shortest path.
- Replace silent `ApplyBossOmenStepResult` no-op with fail-fast behavior until the effect has real mechanics.
- Document that current battle records are abnormality codex records, not per-battle timeline archives.

## Source Of Truth Order

1. Runtime code and live RON/data.
2. Unity-facing snapshot/command DTO contracts.
3. Current canonical docs.
4. Existing refactor/audit notes.

Do not trust the audit note alone. Re-read each runtime path before editing.

## Plan

1. Re-read:
   - `src/game/world/snapshot.rs`
   - `src/game/behavior.rs`
   - `src/game/resources/selection.rs`
   - `src/game/boss_omen.rs`
   - `src/game/world/event_node.rs`
   - current docs mentioning shop DTO, boss omen effects, and battle records.
2. Shop DTO cleanup:
   - Keep `ShopSessionState.hidden_items` internal.
   - Remove hidden item fields only from Unity-facing selected-event snapshot DTOs.
   - Update snapshot tests/docs so hidden data is not serialized.
3. Boss omen BFS:
   - Replace `map_distance_to_terminal()` traversal with BFS.
   - Preserve unreachable behavior as `u32::MAX`.
   - Add focused branching graph test.
4. `ApplyBossOmenStepResult` fail-fast:
   - Replace no-op branch with explicit panic/fail-fast.
   - Check live RON does not rely on the no-op.
   - Add focused test that this effect fails until implemented.
5. Battle record codex clarification:
   - Do not change `abnormality_uuid` dedupe.
   - Update docs/comments to make clear these records are abnormality codex records, not per-battle archives.
6. Run focused tests after each small change.
7. Run broader `cargo check -p game_core` and relevant package tests.

## Completion Conditions

- Unity-facing shop snapshot no longer exposes hidden shop items.
- Core shop reroll behavior remains intact.
- Boss omen terminal distance returns shortest graph distance.
- `ApplyBossOmenStepResult` cannot silently pass as no-op.
- Battle record dedupe by abnormality is explicitly documented as codex behavior.
- No compatibility fallback or dual DTO schema is introduced.
- Focused tests cover every behavior change above.

## Validation Commands

Start focused, then broaden:

- `cargo test -p game_core shop --lib -- --test-threads=1`
- `cargo test -p game_core boss_omen --lib -- --test-threads=1`
- `cargo test -p game_core event_node --lib -- --test-threads=1`
- `cargo test -p game_core snapshot --lib -- --test-threads=1`
- `cargo check -p game_core`

Adjust exact focused filters after reading actual test names.

## Stop Conditions

Complete the goal and report questions if implementation requires:

- A Unity-facing DTO migration strategy beyond simple field removal.
- Real boss omen step-result gameplay semantics.
- Renaming `battle_records` across public DTO/save data.
- Changing shop reroll policy or hidden item generation.
