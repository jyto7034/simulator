# Project Map Documentation Notes

This file records working judgments, source-of-truth conflicts, policy questions, and follow-up candidates for the project map documentation goal.

## Initial Judgments

- The final map should be layered, not a single exhaustive Mermaid diagram.
- Mermaid should explain relationships; Markdown links should provide reliable click-to-open file navigation.
- The map must serve both onboarding and senior maintenance:
  - onboarding: reading routes, entry points, vocabulary,
  - senior work: ownership boundaries, source-of-truth direction, change-impact paths, verification surfaces.
- Goal docs are working memory. Durable project-map output belongs in `docs/PROJECT_MAP.md` and `docs/maps/*.md`.

## Open Questions To Confirm During Mapping

- Sibling repositories/directories were represented as external boundaries, not merged into the core map:
  - `/mnt/f/work/simulator/game_server`
  - `/mnt/f/work/simulator/game_resources`
  - `/mnt/f/unity projects/ark/docs`
- Unity canonical docs are represented as external paths and summarized only at the boundary level in `docs/maps/unity-server-boundary.md`.
- A separate `docs/GLOSSARY.md` was not created. The map pages currently provide enough context through reading routes and file links; revisit if project-specific vocabulary becomes a repeated onboarding blocker.

## Stale-Doc Candidates

- `docs/README.md` referenced missing `docs/data_loading_contract.md`. Existing goal notes in `docs/goals/ron_authoring_consistency_contract/EXPERIMENT_NOTES.md` already said that file is not present and `docs/core_runtime_contract.md` should carry the implementation-contract role. Updated `docs/README.md` to point data navigation at `docs/maps/data-and-content.md` and implementation contract at `docs/core_runtime_contract.md`.

## Follow-Up Candidates

- Add a lightweight Markdown link checker script if project-map docs become heavily linked.
- Add generated module inventory only if manual docs start drifting; avoid committing generated noise by default.
