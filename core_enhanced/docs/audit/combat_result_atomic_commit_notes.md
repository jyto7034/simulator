# Refactor Audit Resolution Notes

Status: temporary audit note.

This note records the current refactor discussion before it becomes an implementation goal. It is not a canonical gameplay contract yet.

## 1. Combat Result Atomic Commit

### Problem

`CompleteCombatResult` currently represents one player-visible action:

```text
confirm combat result
-> apply post-battle employee state
-> grant combat rewards
-> complete or advance the current node
-> return to map or finish the run
```

The implementation does not commit all of those changes as one atomic state transition. It first commits the staged combat result state, then calls `commit_staged_node_completion`.

Current risk:

```text
combat reward/state commit succeeds
-> node completion commit fails
-> CombatResult can remain reachable
-> CompleteCombatResult can be retried
-> rewards or post-battle state may be applied twice
```

The issue is not that staging is wrong. The issue is that there are two failure-bearing commit boundaries for one player-visible action.

### Current Code Pointers

- `src/game/world/combat.rs`
  - `handle_complete_combat_result`
  - commits staged combat state before node completion.
- `src/game/world/node_flow.rs`
  - `plan_complete_current_node`
  - `commit_staged_node_completion`

### Desired Direction

Treat combat result confirmation as one transaction.

Rules:

- All failure-prone validation and planning must happen before any real state mutation.
- Post-battle roster state, rewards, abnormality research, node completion, floor advancement, and run completion must be staged together or proven safe before commit.
- The final real state mutation should happen once, at the end.
- After the final commit starts, remaining work should be infallible or limited to non-gameplay side effects with explicit failure handling.

### Candidate Repair Shape

Preferred shape:

```text
1. Read current CombatBattleState and node context.
2. Plan node completion first.
3. Build a staged world/result state containing:
   - roster after post-battle resolution
   - inventory after rewards
   - skill fragments after rewards/research rewards
   - enkephalin after rewards
   - abnormality research after victory
   - node/map/run progression after completion
   - final GameState
4. Validate all reward modes, static references, and run failure branches before mutating `self.state`.
5. Commit the single staged result to `self.state`.
6. Return the BehaviorResult from committed state.
```

Fallback shape if full unified staging is too large:

```text
1. Preflight `commit_staged_node_completion` failure cases before applying combat rewards.
2. Ensure the later node completion path cannot fail after reward commit.
3. Add regression tests that inject or construct the previous failure mode.
```

The fallback is less clean and should only be used if the full unified staging proves too invasive.

### Test Contract

The implementation goal should add tests around user-visible behavior, not internal helper shape:

- A successful combat result confirmation grants rewards once and consumes/completes the node once.
- Repeating `CompleteCombatResult` after success is rejected or impossible.
- If node completion cannot be planned, no reward, employee state, research state, or inventory change is committed.
- Floor advance and run complete branches preserve the same atomicity.
- Final boss failure and non-player victory branches are checked separately because they skip normal reward flow.

### Open Notes

- SavePoint checkpoint writing inside node completion is a side effect. The implementation should decide whether checkpoint write failure is gameplay-fatal or should happen after state commit with explicit error reporting.
- Battle record file writing is also a side effect and should not create partial gameplay commits.
- This issue should probably become a focused goal before broader world state refactors.

## 2. Hide Shop Hidden Items From Unity-Facing Snapshots

### Decision

Proceed with this cleanup.

The game is single-player, so this is not a high-severity security issue. Still, it is a clean DTO boundary improvement: Unity should only receive the shop items currently visible to the player. Hidden/reroll-reserve items should remain core-internal state until they become visible after an actual reroll.

### Problem

`ShopSessionState` legitimately stores both visible and hidden items:

```text
visible_items = items currently shown in the shop UI
hidden_items  = core-internal reserve used by reroll behavior
```

The current selected-event shop snapshot exposes both visible and hidden items:

```text
visible_items
hidden_items
visible_item_uuids
hidden_item_uuids
```

That makes Unity aware of reroll-reserve items before they are actually revealed. This is unnecessary client knowledge and weakens the DTO boundary.

### Desired Direction

Keep `hidden_items` in core runtime state, but remove it from Unity-facing snapshot DTOs.

Target shape:

```text
Core ShopSessionState:
  visible_items
  hidden_items

Unity-facing selected_event Shop DTO:
  visible_items
  visible_item_uuids
```

Reroll flow:

```text
Unity sends reroll command
-> Core updates shop state
-> Core sends result/snapshot with the new visible_items only
```

### Expected Code Pointers

- `src/game/resources/selection.rs`
  - `ShopSessionState` should keep `hidden_items`.
- `src/game/behavior.rs`
  - `SelectedEventSnapshotDto::Shop` should drop `hidden_items` and `hidden_item_uuids`.
- `src/game/world/snapshot.rs`
  - shop snapshot mapping should serialize only visible items.

### Test Contract

Add or update a snapshot-facing test:

- Shop runtime state may contain hidden items.
- Shop selected-event snapshot exposes visible items.
- Shop selected-event snapshot does not contain `hidden_items` or `hidden_item_uuids`.
- Reroll still changes the visible items through core state/result flow.

## 3. Boss Omen Terminal Distance Should Use BFS

### Decision

Proceed with this cleanup.

This is an implementation correctness issue, not a gameplay policy change. The function name and usage imply graph distance to the terminal node, so the implementation should return the shortest edge distance.

### Problem

`map_distance_to_terminal()` currently uses a `Vec` as a stack:

```text
frontier.pop()
-> depth-first traversal
-> returns the first terminal distance encountered
```

In a graph with multiple paths to the terminal, the first terminal reached by DFS is not necessarily the nearest terminal path. This can make boss omen candidate sorting use an arbitrary path length instead of the shortest graph distance.

### Desired Direction

Replace the DFS-like traversal with queue-based BFS:

```text
frontier.pop_front()
-> breadth-first traversal
-> first terminal hit is shortest distance
```

Rules:

- Return the shortest bidirectional graph distance from the candidate node to the terminal node.
- If the terminal is unreachable, keep returning `u32::MAX`.
- Preserve deterministic tie-breakers outside the distance function.

### Expected Code Pointers

- `src/game/boss_omen.rs`
  - `map_distance_to_terminal`
  - candidate sort in `choose_source_node`

### Test Contract

Add a focused test with a small graph where one node has both:

- a short path to terminal
- a longer path to terminal

The distance function must return the short path length.

## 4. Boss Omen Step Result Effect Must Not Be Silent No-Op

### Decision

Use fail-fast for now.

`ApplyBossOmenStepResult` must not remain a silent no-op. Since concrete boss omen step-result mechanics are not fully designed yet, the current cleanup should not invent partial gameplay semantics. Instead, if live data uses this effect before it has a real implementation, core should fail fast so QA catches the invalid data immediately.

### Problem

`EventChoiceEffect::ApplyBossOmenStepResult` currently does nothing at runtime:

```text
ApplyBossOmenStepResult => {}
```

That makes the RON contract misleading. Data authors may believe the choice applies or consumes an omen step result, while the runtime silently ignores it.

### Desired Direction

Near-term rule:

```text
ApplyBossOmenStepResult is not implemented.
If encountered in live event choice effects, panic/fail-fast instead of doing nothing.
```

Long-term rule:

```text
When boss omen step-result mechanics are designed, replace the fail-fast branch with explicit runtime behavior and tests.
```

### Guardrails

- Do not implement partial omen-step consumption until the full chain lifecycle is checked.
- Avoid double-consuming omen steps if node completion hooks already consume source nodes.
- Do not keep compatibility behavior that silently ignores the effect.

### Expected Code Pointers

- `src/game/world/event_node.rs`
  - `apply_event_choice_effects`
  - `EventChoiceEffect::ApplyBossOmenStepResult`
- `src/game/boss_omen.rs`
  - source node completion and chain progression hooks should be checked before any future real implementation.

### Test Contract

Add or update a focused test:

- An event choice containing `ApplyBossOmenStepResult` must fail fast until the effect is implemented.
- Ordinary event choices without that effect continue to work.

## 5. Single Official Live RON Loader Entry Point

### Decision

Proceed as a separate medium-sized refactor goal.

The goal is not to merge all RON files into one file. The goal is to make the code path that reads live RON files and assembles `GameDataBase` canonical and single-entry.

### Problem

Live RON loading is currently assembled in more than one place:

```text
game_server/src/main.rs
tests/common/mod.rs
src/game/world/tests/mod.rs
```

Each loader reads many RON files and wires them into `GameDataBuilder`. When a new data domain is added, every loader must be updated manually. This already caused drift when event and boss omen data were added.

### Desired Direction

Add one official core-side loader entry point, for example:

```text
GameDataBase::load_live_embedded()
```

or equivalent naming.

All production and live-data tests should call that one loader:

```text
game_server -> GameDataBase::load_live_embedded()
tests/common -> GameDataBase::load_live_embedded()
```

Special tests may still build custom `GameDataBase` values, but live RON contract tests should not duplicate the production loader graph.

### Rules

- Keep RON files split by domain.
- Make the RON-to-`GameDataBase` assembly code single-source.
- Server and tests must load the same embedded live data set unless a test explicitly documents an override.
- New live RON domains should require editing one loader path, not three.

### Expected Code Pointers

- `src/game/data/mod.rs`
  - likely home for the official embedded loader.
- `../game_server/src/main.rs`
  - should delegate to the official loader.
- `tests/common/mod.rs`
  - should delegate to the official loader.
- `src/game/world/tests/mod.rs`
  - should either delegate or clearly separate custom fixtures from live RON loading.

### Test Contract

- `cargo check -p game_core` and game server check should still compile after delegation.
- Live RON loading tests should use the official loader.
- Add a small test or assertion that critical data domains are present through the official loader:
  - events
  - boss omen chains
  - abnormalities
  - pve encounters
  - run policy

## 6. Embedded RON Loading Is The Current Policy

### Decision

Keep `include_str!` / embedded RON loading for now.

The current project should treat live data loading as embedded static data loading. RON edits require recompilation before they affect the running server or tests.

### Clarification

Current loading style:

```text
include_str!(".../game_resources/data/*.ron")
-> RON contents are embedded at compile time
-> runtime server restart alone does not reload changed RON files
```

This is acceptable for the current phase because it is simple, reproducible, and deterministic.

### Rules

- Do not introduce runtime hot reload as part of the loader single-source cleanup.
- Use naming/documentation that makes the embedded nature clear, such as `load_live_embedded`.
- If runtime data packs, mod support, or hot reload become a real product goal, design that as a separate system.

### Future Direction

Potential future APIs, not current scope:

```text
GameDataBase::load_embedded()
GameDataBase::load_from_files(path)
GameDataBase::load_with_overlays(base, mods)
```

Those would require explicit data versioning, validation failure handling, path policy, and mod overlay rules.

## 7. Battle Records Are Abnormality Codex Records, Not Per-Battle Archives

### Decision

Do not change the current `abnormality_uuid` dedupe behavior.

The intended product meaning of these records is an abnormality codex/observation record, not a per-battle timeline archive. Therefore, saving one representative record per abnormality is valid.

### Clarification

The misleading part is naming and documentation, not necessarily behavior.

Current behavior:

```text
if a record with the same abnormality_uuid already exists:
  skip recording
```

This is wrong only if `battle_records` means "every individual battle timeline". It is acceptable if the meaning is:

```text
abnormality codex battle/observation record
one representative record per abnormality
```

### Desired Direction

Keep the dedupe behavior, but document the meaning clearly.

Potential future cleanup:

```text
battle_records -> abnormality_battle_records
or
battle_records -> codex_battle_records
```

Do not rename during unrelated refactors unless the blast radius is small and tests are updated.

### Test Contract

If this area becomes a goal:

- Repeated battles against the same abnormality keep one codex record.
- Battles against different abnormalities create separate codex records.
- Per-battle debug/event-log archives, if needed, must use a different storage path and key such as `battle_uuid`.

## 8. Validation Matches Must Not Silently Accept New Skill/Effect Variants

### Decision

Proceed with this cleanup.

Validation code should not use broad wildcard branches to silently accept future skill/effect variants. If a variant requires no extra validation, list it explicitly so future enum additions force a compiler-visible decision.

### Problem

Validation code shaped like this is risky:

```rust
match effect {
    EffectA => validate_a(),
    EffectB => validate_b(),
    _ => {}
}
```

When a new `EffectC` is added, the compiler does not force validation code to be updated. The new variant may then pass live RON validation without any explicit decision.

### Desired Direction

Prefer exhaustive matches in validation paths:

```rust
match effect {
    EffectA => validate_a(),
    EffectB => validate_b(),
    EffectC => {}
}
```

Rules:

- Do not use `_ => {}` for gameplay skill/effect validation unless there is a very narrow, documented reason.
- Variants that require no validation should still be listed explicitly.
- Adding a new variant should force the developer to decide its validation behavior.

### Expected Code Pointers

- `src/game/data/validation.rs`
- `src/game/data/skill_data.rs`
- Any other skill/effect/RON validation match that currently relies on broad wildcard branches.

### Test Contract

This is mostly compile-time protection, but implementation should also keep live RON loading tests passing.

Useful checks:

- Existing live RON still validates.
- No wildcard branch silently swallows known skill/effect variants in validation code.
- If a new variant is added later, validation code should fail to compile until updated.
