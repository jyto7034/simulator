# Project Map Documentation Experiments

This file records inventory commands, failed assumptions, documentation edits, and verification results for the project map documentation goal.

## Log

### 2026-07-07 - Goal Created

Initial goal workspace created.

Read:

- `docs/README.md`
- `docs/codex_goal_command.md`
- `docs/code_documentation_sync_guidelines.md`
- `docs/goals/documentation_runtime_contract_audit/PLAN.md`
- `docs/goals/core_large_scale_documentation_sync/PLAN.md`

Observed:

- Existing docs already use `docs/goals/<goal_name>/PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`.
- `docs/README.md` is the current docs index and already distinguishes durable docs from goal memory.
- Project-map work should create durable docs under `docs/` and `docs/maps/`, then update `docs/README.md` only for discoverability.

Commands run:

```text
rg --files -g 'AGENTS.md' -g 'docs/**' -g 'README*' -g 'Cargo.toml' -g 'package.json'
find . -maxdepth 2 -type d | sort
git status --short
sed -n '1,220p' docs/goals/documentation_runtime_contract_audit/PLAN.md
sed -n '1,220p' docs/goals/core_large_scale_documentation_sync/PLAN.md
sed -n '1,180p' docs/README.md
sed -n '1,240p' docs/codex_goal_command.md
sed -n '1,220p' docs/code_documentation_sync_guidelines.md
rg --files src tests config docs | sed -n '1,160p'
```

Validation:

- Not run yet. This commit only seeds the goal documents.

### 2026-07-07 - Layered Project Map Implemented

Read startup/source documents:

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

Inventory commands:

```text
find src/game -maxdepth 3 -type f | sort
find tests -maxdepth 3 -type f | sort
find config -maxdepth 3 -type f | sort
find /mnt/f/work/simulator/game_resources -maxdepth 3 -type f | sort | sed -n '1,160p'
sed -n '1,240p' src/game/battle/mod.rs
sed -n '1,260p' src/game/battle/core/mod.rs
sed -n '1,260p' src/game/data/mod.rs
sed -n '1,260p' src/game/world.rs
sed -n '1,220p' src/game/behavior.rs
sed -n '1,240p' src/game/world/state.rs
sed -n '1,220p' src/game/world/snapshot.rs
sed -n '1,220p' src/game/map/mod.rs
sed -n '1,220p' src/game/combat_setup/mod.rs
sed -n '1,220p' src/game/combat_preview/mod.rs
sed -n '1,220p' src/game/resources/mod.rs
find src/game/battle/core -maxdepth 3 -type f | sort
rg -n "load_live_embedded|include_str!|validate_all|validate_contract" src/game/data src/game/combat_preview src/game/map src/game/battle/buffs.rs
find /mnt/f/work/simulator/game_server/src -maxdepth 4 -type f | sort | sed -n '1,200p'
find "/mnt/f/unity projects/ark/docs" -maxdepth 2 -type f | sort | sed -n '1,120p'
rg -n "game_core|GameCore|PlayerBehavior|BehaviorResult|battle_update|battle_setup_snapshot|request_battle_state|admin" /mnt/f/work/simulator/game_server/src | sed -n '1,220p'
find /mnt/f/work/simulator/game_resources/data/events -maxdepth 3 -type f | sort
rg -n "data_loading_contract" docs src tests Cargo.toml
```

Created durable docs:

- `docs/PROJECT_MAP.md`
- `docs/maps/index.md`
- `docs/maps/runtime.md`
- `docs/maps/game-flow.md`
- `docs/maps/battle.md`
- `docs/maps/data-and-content.md`
- `docs/maps/unity-server-boundary.md`
- `docs/maps/tests-and-tools.md`

Updated:

- `docs/README.md`
  - Added `docs/PROJECT_MAP.md` to the top-level document table.
  - Replaced stale `docs/data_loading_contract.md` index references with `docs/maps/data-and-content.md` for navigation and `docs/core_runtime_contract.md` for implementation contract.

Validation commands:

```text
rg --files src tests config docs Cargo.toml
rg -n '```mermaid|docs/maps|PROJECT_MAP|unity_core_contract|core_unity_battle_transport_contract' docs/PROJECT_MAP.md docs/maps docs/README.md docs/goals/project_map_documentation
python3 - <<'PY'
from pathlib import Path
import re

docs = [Path('docs/PROJECT_MAP.md'), *Path('docs/maps').glob('*.md')]
missing = []
for doc in docs:
    if not doc.exists():
        continue
    text = doc.read_text(encoding='utf-8')
    for target in re.findall(r"\[[^\]]+\]\(([^)#][^)]+)\)", text):
        if '://' in target or target.startswith('/mnt/') or target.startswith('F:'):
            continue
        path = (doc.parent / target).resolve()
        if not path.exists():
            missing.append((str(doc), target))
if missing:
    for doc, target in missing:
        print(f'missing link: {doc} -> {target}')
    raise SystemExit(1)
print('markdown relative links ok')
PY
```

Results:

- `rg --files` completed and confirmed the expected new map files are present.
- Mermaid/contract search completed after rerunning with safe quoting.
- Markdown relative link checker passed: `markdown relative links ok`.
- Rust checks/tests were not run because this goal changed documentation only and did not alter code, DTOs, live RON, or tests.
