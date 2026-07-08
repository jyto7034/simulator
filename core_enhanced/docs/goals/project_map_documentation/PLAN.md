# Project Map Documentation Goal

## Objective

Create a durable, clickable, layered project map for `core_enhanced` and its runtime/client-facing boundaries so new contributors can orient quickly and senior contributors can inspect ownership, source-of-truth boundaries, and change impact paths.

## Goal Mode Working Method

This goal produces documentation, not gameplay behavior.

At the beginning of this goal and every resumed run, read:

- `docs/goals/project_map_documentation/PLAN.md`
- `docs/goals/project_map_documentation/EXPERIMENTS.md`
- `docs/goals/project_map_documentation/EXPERIMENT_NOTES.md`
- `docs/README.md`
- `docs/code_documentation_sync_guidelines.md`
- `docs/core_runtime_contract.md`
- `docs/game_rulebook.md`
- `docs/skill_target_contract.md`
- `docs/refactor_preparation_plan.md`
- `Cargo.toml`
- `src/lib.rs`
- `src/game/mod.rs`

Before writing any durable map, confirm the current file/module layout from runtime code with `rg --files`, `Cargo.toml`, module roots, tests, live data references, and the current docs index. Treat existing documentation as context, not proof.

## Source Of Truth

Use facts in this order:

1. Actual runtime code and module declarations.
2. Live RON/data and embedded loader code.
3. Unity-facing snapshot/command/WebSocket/admin contracts.
4. Current durable docs.
5. Goal documents and historical notes.

If a document and code disagree, map the code and record the stale-document candidate in `EXPERIMENT_NOTES.md`. If the disagreement implies a gameplay, DTO, UX, schema, or ownership policy decision, stop and report the question instead of silently choosing.

## Target Deliverables

Create or update these durable docs:

- `docs/PROJECT_MAP.md`
  - Main entry point for the project map.
  - Contains the top-level module/boundary map, reader routes, and links to detailed map pages.
- `docs/maps/index.md`
  - Index of detailed maps and the intended reading order.
- `docs/maps/runtime.md`
  - Core runtime ownership map: world state, behavior dispatch, battle runtime, event log, snapshots, data loading, validation.
- `docs/maps/game-flow.md`
  - Player-visible flow map: run start, node map, events, shop/support/maintenance, combat, rewards, progression.
- `docs/maps/battle.md`
  - Battle runtime map: setup, commands, targeting, movement, damage/effects, lifecycle, event log, checkpoint/result.
- `docs/maps/data-and-content.md`
  - Live RON/data/content map: loaders, validation, skills, equipment, fragments, abnormalities, waves, rewards.
- `docs/maps/unity-server-boundary.md`
  - Core/server/Unity boundary map and links to external canonical Unity docs.
- `docs/maps/tests-and-tools.md`
  - Tests, probes, battle records, debug exports, and tool/support surfaces.

Optional follow-up deliverables if the map reveals a clear need:

- `docs/GLOSSARY.md`
- `docs/PROJECT_TOUR.md`
- focused updates to `docs/README.md` so it points to the new map entry point.

## Map Design Requirements

- Use Mermaid for architecture and flow diagrams where it clarifies relationships.
- Do not create one giant graph that tries to contain every file.
- Prefer layered diagrams:
  - top-level boundaries,
  - major runtime flows,
  - ownership/source-of-truth maps,
  - detailed per-domain maps.
- Keep Mermaid nodes semantic. Put clickable file lists below the diagrams with normal Markdown links.
- Do not rely on Mermaid `click` as the primary navigation mechanism because viewer support varies.
- File links must be relative Markdown links so GitHub, VS Code, JetBrains, and local Markdown previews can open files directly.
- Every major box in a durable diagram should have nearby related file links.
- Map current live paths only. Historical names can appear only when they explain migration context and must be labeled as historical.
- Senior-reader needs are first-class:
  - show ownership boundaries,
  - show source-of-truth direction,
  - show change-impact paths,
  - show test/probe evidence locations,
  - distinguish runtime code, authored data, DTO contracts, and docs.
- New-reader needs are also first-class:
  - provide an initial reading route,
  - explain entry points,
  - avoid unexplained project-specific vocabulary where a glossary link would help.

## In Scope

- Inventory the current `core_enhanced` Rust package layout.
- Inventory durable docs and classify canonical docs versus goal memory.
- Map runtime boundaries under `src/game/**`.
- Map live data/content ownership and validation paths.
- Map battle runtime and event/checkpoint/result boundaries.
- Map world/node progression and player behavior dispatch.
- Map test suites and debug/probe artifacts enough for change validation.
- Map relevant sibling/external boundaries when readable and canonical:
  - `/mnt/f/work/simulator/game_server`
  - `/mnt/f/work/simulator/game_resources`
  - `/mnt/f/unity projects/ark/docs`
- Add links to external canonical Unity docs by path, but do not create local stale copies.
- Update `docs/README.md` only if needed to make the new map discoverable.

## Out Of Scope

- Gameplay behavior changes.
- DTO or WebSocket contract changes.
- Live RON schema/content changes.
- File moves or module refactors.
- Rewriting existing source-of-truth docs beyond minimal index/link updates.
- Making a generated documentation site.
- Creating compatibility layers, fallback paths, or dual documentation paths for stale contracts.

## Implementation Plan

1. Inventory project shape.
   - Read `Cargo.toml`, module roots, `src/game/mod.rs`, major submodule roots, tests, config, and docs index.
   - Record command results and surprising findings in `EXPERIMENTS.md`.

2. Identify durable boundaries.
   - Runtime domains: behavior, world, map, battle, data, resources, stats/growth/rewards, admin/debug.
   - External boundaries: server messages, Unity-facing docs, live RON/data, battle records/probes.
   - Record uncertain ownership or stale docs in `EXPERIMENT_NOTES.md`.

3. Draft the map taxonomy.
   - Decide which relationships belong in `PROJECT_MAP.md` versus detailed `docs/maps/*.md`.
   - Keep each page focused on one reader task.

4. Create map documents.
   - Add Mermaid diagrams and nearby Markdown file links.
   - Prefer short "When changing X, inspect Y" sections for senior impact analysis.
   - Add "Start here" reading routes for new contributors.

5. Verify links and consistency.
   - Check that linked files exist.
   - Check Mermaid fences are syntactically plausible.
   - Re-read changed docs for stale claims, broken relative paths, and duplicated source-of-truth wording.

6. Update discovery.
   - Update `docs/README.md` if the new map should be the first visible project navigation entry.

## Completion Conditions

- `docs/PROJECT_MAP.md` exists and acts as the single durable entry point for the project map.
- `docs/maps/` contains focused detailed maps for runtime, game flow, battle, data/content, Unity/server boundary, and tests/tools.
- Major runtime domains have both a Mermaid overview and clickable Markdown links to relevant files.
- The map distinguishes source-of-truth code/data/DTO/docs instead of treating documents as primary facts.
- Senior change-impact paths are present for common changes:
  - player command/behavior,
  - Node Map/world flow,
  - battle runtime,
  - skill/targeting/range,
  - live RON/data,
  - Unity-facing DTO/transport,
  - tests/probes/debug artifacts.
- New-contributor reading routes are present.
- External Unity canonical docs are linked by external path where relevant and are not copied locally.
- `docs/README.md` points to the new project map if appropriate.
- `EXPERIMENTS.md` records inventory/search commands and validation.
- `EXPERIMENT_NOTES.md` records unresolved ownership questions, stale-doc candidates, and follow-up map improvements.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

Stop and report questions instead of guessing if the mapping work finds:

- unclear source-of-truth ownership between core, server, Unity, or live RON/data,
- a current durable doc contradicting runtime behavior in a way that changes policy,
- Unity-facing DTO or transport ambiguity,
- live RON schema/content ambiguity,
- save migration or backward compatibility implications,
- UX meaning changes,
- gameplay rule changes,
- test/probe evidence that contradicts the intended map.

사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Verification Commands

Use documentation-focused verification. Adjust as the final changed files require.

```text
rg --files src tests config docs Cargo.toml
rg -n "```mermaid|docs/maps|PROJECT_MAP|unity_core_contract|core_unity_battle_transport_contract" docs
python3 - <<'PY'
from pathlib import Path
import re

root = Path(".").resolve()
docs = [Path("docs/PROJECT_MAP.md"), *Path("docs/maps").glob("*.md")]
missing = []
for doc in docs:
    if not doc.exists():
        continue
    text = doc.read_text(encoding="utf-8")
    for target in re.findall(r"\[[^\]]+\]\(([^)#][^)]+)\)", text):
        if "://" in target or target.startswith("/mnt/") or target.startswith("F:"):
            continue
        path = (doc.parent / target).resolve()
        if not path.exists():
            missing.append((str(doc), target))
if missing:
    for doc, target in missing:
        print(f"missing link: {doc} -> {target}")
    raise SystemExit(1)
print("markdown relative links ok")
PY
```

Run code checks only if the goal accidentally requires code/test changes:

```text
cargo check
cargo test -- --test-threads=1
```
