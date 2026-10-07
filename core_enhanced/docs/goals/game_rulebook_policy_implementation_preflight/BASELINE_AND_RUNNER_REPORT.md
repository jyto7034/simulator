# Game Rulebook Policy Implementation Preflight — Baseline And Runner Report

Status: complete_for_pf06
Evidence Date: 2026-08-12
Repository HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Unity Validation: explicitly_excluded_by_user

## Outcome

Current dirty-tree Core, production live-RON loading and game_server baselines are green. These runs prove that the
current pre-implementation tree is internally executable; they do not prove target rulebook behavior. Several
passing tests intentionally assert the legacy research, generated Standard and old BossOmen contracts that WP001
and WP003 must replace.

## Executed Test Runners

| Scope | Exact command | Actual tests | Pass | Fail | Ignored/skipped | Filtered | Result |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| Monolith/run-policy focused | `cargo test -p game_core data::run_policy_data -- --quiet` | 17 | 17 | 0 | 0 | 995 lib + 51 integration | green |
| Core library | `cargo test -p game_core -- --quiet` | 1012 | 1012 | 0 | 0 | 0 | green |
| Core `live_item_skill_activation` | same full Core command | 4 | 4 | 0 | 0 | 0 | green |
| Core `live_skill_catalog_audit` | same full Core command | 4 | 4 | 0 | 0 | 0 | green |
| Production RON integration | same full Core command, `tests/ron_loading.rs` | 25 | 25 | 0 | 0 | 0 | green |
| Core `skill_runtime_contract` | same full Core command | 14 | 14 | 0 | 0 | 0 | green |
| Core `skill_test_suite` | same full Core command | 4 | 4 | 0 | 0 | 0 | green |
| Core doc tests | same full Core command | 0 | 0 | 0 | 0 | 0 | no tests |
| game_server library | `cargo test -p game_server -- --quiet` | 63 | 63 | 0 | 0 | 0 | green |
| game_server main/doc tests | same server command | 0 | 0 | 0 | 0 | 0 | no tests |

Full Core total: **1063 passed, 0 failed, 0 ignored**. Full game_server total: **63 passed, 0 failed, 0 ignored**.
The focused command's integration binaries ran zero selected tests because the name filter did not match them;
these are reported as filtered, not as a successful integration sample. The unfiltered full Core command is the
broad evidence.

The full commands were executed twice during final evidence capture: once with normal per-test output and once with
`--quiet` to preserve exact runner totals. Both completed successfully; the counts above use the concise final run.

## Static Current-Tree Checks

### Bootstrap references

The read-only manifest check extracted every Elite/NormalBoss/FinalBoss table row from
`CONTENT_INPUT_CANDIDATES.md` and searched exact IDs in production catalogs.

```text
manifest rows=29 missing_encounters=0 missing_abnormalities=0 missing_fragments=0
live class counts Elite=13 NormalBoss=12 FinalBoss=5
tutorial pool tutorial_f1_training_budget target-authored-pending
tutorial pool tutorial_f3_salvage target-authored-pending
tutorial pool tutorial_f3_budget target-authored-pending
boss omen chains=1 white_night=1
```

All other reused ST1 topology, encounter, event, reward, shop, headquarters and briefing identities were found;
the check printed no missing reused identity. Target `reward_source_id`, `fragment_id` and `repeat_reward` authoring
has zero current production hits. The current class counts differ from the target manifest by the known
Queen/Mountain reclassification and Moonlit Wail exclusion.

### Save/store and old-authority baseline

- Core `ACTIVE_RUN_CORE_PAYLOAD_VERSION` is 3.
- game_server `ACTIVE_RUN_VERSION` is 3.
- game_server compendium `PROFILE_VERSION` is 1.
- Current checkpoint payload contains `RunAbnormalityResearchState`, and current DTO/result/store paths still expose
  research/pending delivery.
- Current target successor types/fields such as a unified encounter progression ledger and explicit reward-source
  authoring are absent.
- Current live Monolith stat values are `100/125`; target `150/200` is not implemented.
- Current BossOmen schema has one-step source data but lacks per-chain fragment/awakening/dependency DAG fields.
- Current fragment catalog contains 29 `BasicAttackModifier` placeholders as intended.

### Dirty ownership check

The repository was broadly dirty before this Preflight. Relevant Core, live RON and server files remain modified by
the user. For `WP002`, both target files are dirty, but a zero-context diff search found no current changed hunk for
`stage_2_stat_multiplier_percent`, `stage_3_stat_multiplier_percent` or their reward counterparts. This makes the
first package eligible for a scoped activation handoff, not permission to overwrite whole files. The implementation
Goal must repeat the exact-hunk check immediately before editing.

No production source, RON, server, Unity or canonical rulebook file was changed by Preflight execution. Only the
active Goal directory contains Preflight documentation edits.

## Runner And Fixture Availability By Package

| Package | Activation baseline | Existing reusable fixtures | Target-first additions required |
| --- | --- | --- | --- |
| `WP002` | focused run-policy, `--test ron_loading`, full Core | Monolith ordering/composition, live full-bundle load, preview scale | live 150/200, derived every-stat, zero/reversed/non-representable rejection, reward/wave unchanged |
| `WP001` | full Core + full game_server | combat-result rollback, retreat, V3 graph mutation, BossOmen current E2E, active-run store recovery | encounter ledger/cycles, 29 source rows, V4 strict corpus, per-chain DAG/failure, no research fields, mapping/order |
| `WP003` | full Core + full game_server | map generation, compendium monotonic merge/store recovery, title capability mapping | fixed ST1, actor-only F3 override, V1 successor migration, atomic completion+flag, Endless gate and records |
| `WP004` | full Core + full game_server after WP001/WP003 | action capability, staged reward/XP/duration rollback, reconnect mapping | sweep eligibility/result/no-side-effect/exclusive branch/failpoint/checkpoint/reconnect corpus |

Each implementation Goal must rerun its activation baseline on its then-current tree. This Preflight baseline does
not substitute for that gate.

## Exact Static Commands

```text
git rev-parse HEAD
git status --short -- <scoped Core/RON/server/Goal paths>
sha256sum core_enhanced/docs/game_rulebook.md <readiness artifacts>
rg -l '^Status: active$' core_enhanced/docs/goals/*/PLAN.md
rg -n 'stage_[23]_(stat|reward)_multiplier_percent' <(git diff -- game_resources/data/run/policy.ron core_enhanced/src/game/data/run_policy_data.rs)
rg -n 'ACTIVE_RUN_(CORE_PAYLOAD_)?VERSION|PROFILE_VERSION' core_enhanced/src game_server/src
rg -c 'BasicAttackModifier' game_resources/data/skill_fragments/base.ron
```

The bootstrap extraction loop and its output are recorded in `EXPERIMENTS.md`.

## Omitted Or Unavailable Evidence

- Unity current tree, EditMode/PlayMode and live-client probe: explicitly excluded by the user because that
  development tree has large unrelated drift. Completed Unity tests: 0; no pass/fail count or consumer claim.
- Target behavior tests: not executable before production implementation; this dossier specifies them instead.
- Final content/balance/playability: minimum bootstrap only; no final-content approval or playtest claim.
- Public release gate: intentionally absent for this development-only project.
- Unrelated repository suites: not required to prove current package baselines and not run by this Preflight.

## PF-06 Verdict

Core/live-RON/game_server runner evidence is reproducible, executes non-zero relevant tests and has no unexplained
failure. The user-approved Unity omission is explicit. PF-06 is complete and does not block
`ready_for_wp002`.

