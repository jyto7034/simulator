# Completed Goal Document Retirement

## Objective

Resolve P-012 by reducing confusion between completed goal documents and current canonical policy documents.

`docs/goals` currently contains many completed planning artifacts. They are useful history, but when they sit beside canonical docs without clear status, readers must repeatedly decide whether a goal document is current policy, obsolete context, or implementation history.

## Source Of Truth Order

1. Current runtime code and live RON/data.
2. Unity-facing snapshot/command contracts.
3. Canonical docs such as `docs/game_rulebook.md`, transport contracts, and README docs index.
4. Completed goal documents.
5. Audit notes.

## Scope

- Classify goal documents into active, completed-history, superseded, and canonical-policy-needed.
- Move or mark completed goals so they are not mistaken for source-of-truth policy.
- Absorb still-relevant policy into canonical docs before retiring a goal document.
- Update docs index/README so readers know where current policy lives.

## Non-Goals

- Do not delete policy that has not been absorbed into canonical docs.
- Do not change runtime code.
- Do not rewrite every goal document into prose.
- Do not treat old goal docs as authoritative over runtime code.

## Plan

1. Re-read:
   - `docs/README.md`
   - canonical docs listed there
   - goal directories that are still referenced by canonical docs or current implementation plans.
2. Build a status inventory:
   - active implementation goal;
   - completed and fully absorbed;
   - completed but policy still needs absorption;
   - superseded/obsolete;
   - audit/history only.
3. For each completed goal:
   - extract any still-current policy into canonical docs;
   - remove stale implementation-plan wording from canonical docs;
   - mark or move the goal as history according to the chosen convention.
4. Update README/index guidance:
   - canonical docs are source of truth;
   - goal docs are execution history unless explicitly active.
5. Run doc grep checks for known stale phrases from recent audits.

## Completion Conditions

- Readers can identify canonical policy without opening dozens of completed goal docs.
- Completed goal documents are clearly historical, archived, or removed after policy absorption.
- No current policy exists only inside a completed goal document.
- `docs/README.md` accurately explains the role of goal docs.
- Known stale audit claims are no longer presented as current policy.

## Validation Commands

- `rg -n "draft|temporary|TODO|superseded|goal|PLAN.md|source of truth" docs`
- `rg -n "hidden_items|battle_uuid|battle_records|ApplyBossOmenStepResult|difficulty|available_node_ids" docs`
- `cargo check -p game_core`

`cargo check` is included only as a safety net if docs reference generated code paths or doctest-like snippets are touched.

## Stop Conditions

Complete the goal and report questions if:

- A completed goal contains policy that conflicts with canonical docs and runtime code.
- The user wants goal documents physically deleted rather than archived/marked.
- A policy cannot be classified without a design decision.
- The docs index needs a larger reorganization than this cleanup goal should own.
