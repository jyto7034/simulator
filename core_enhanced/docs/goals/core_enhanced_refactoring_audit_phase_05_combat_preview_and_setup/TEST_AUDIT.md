# Core Enhanced Refactoring Audit Phase 05 — Test-Code Audit

Audit Date: 2026-08-07
Policy: Parent CRA-14 and Phase 5 test-code audit policy

Green counts are execution evidence, not the reason for retaining a test. Each disposition records production
meaning, reachability, assertion quality, overlap and the replacement safety net.

## Family Dispositions

| Family | Production path and unique invariant | Assertion quality / overlap | Disposition |
| --- | --- | --- | --- |
| `combat_preview` template/parser/instance tests | official ASCII/template parser and runtime generated battlefield topology, route, zones and obstacles | strong negative field/topology assertions; handcrafted invalid instances cover cases live data cannot | `retain` |
| `combat_preview` seed/archetype/override tests | deterministic template selection, authored battlefield/obstacle override and Boss fallback | direct output equality and topology; broad all-archetype and every-authored-template tests overlap in input source but not fallback-vs-publication meaning | `retain` |
| threat-warning tests | typed serialized warnings, profile-derived required tags, rumor seed/no-duplicate/current tag set | direct tag/source/status assertions; official RON test uniquely covers current live profiles | `retain` unit and live integration layers |
| difficulty/Monolith/serialization tests | manifested tuning owns internal stat scale and Monolith composition while serialized preview omits it | the round-trip assertion is valuable evidence but stops before checkpoint/cache reuse | `retain`; `strengthen` with C01 durable active-run sequence |
| generated-wave resolution tests | generated budget scaling, manual count invariance, deterministic entries/appearance seeds and runtime source construction | direct counts/source/determinism; no accepted-data insufficient-capacity negative | `retain`; `strengthen` through C02 publication case |
| mission-policy unit tests (5) | supported types, SplitRoom rejection and live mission pair | two tests cover no production distinction: elite branch equals normal and archetype wrapper has no production caller | `retain` supported/SplitRoom/live-pair tests; `merge/remove` the two no-op helper tests under C05 |
| reward multiplier tests (3) | quantity rounding/minimum and singular identity preservation | direct effect-vector equality; Monolith small-quantity case uniquely pins one-pass rounding | `retain` |
| `reward_policy` tests (4) | featured/allowed reward identity for Defense/Boss | forbidden and Boss-positive are unique; two currency-only Defense failures duplicate, with one adding empty acceptance | `merge` the two Defense featured tests while preserving empty acceptance and error text; retain others |
| combat-power tests (4) | persistent formula inputs, determinism, temporary/condition exclusions and armed eligibility boundary | direct member/total comparisons; dead case is a readable boundary subset but also asserts empty total | `retain` |
| `events/combat` preview-to-scenario tests (13) | strict preview/encounter match, wave timing, tactical/objective override, defense object/route, survival and full setup | production builders and scenario assertions; `defense_node_type_builds_fixed_defense_objective` is strictly subsumed by the stronger default-defense test | `retain` unique 12; `merge/remove` the strict subset under C05 |
| world preview/difficulty tests | full node UUID seed and missing manifested difficulty rejection | production `GameCore` path and exact error; no durable serialized-cache test | `retain`; `strengthen` with C01 |
| world no-deployable/weaponless tests | battle-entry failure, armed membership, reason priority and recovery policy | full command/state assertions; superficially similar tests cover distinct run failure/reward/HQ branches | `retain` |
| world setup snapshot/route/setup-loss tests | actual scene source, survival variant, catalog refs, authored runtime route and loss recovery | strong result/checkpoint parity; DTO shape becomes Phase 10 consumer but setup invariant remains here | `retain` |
| full live DefenseRoute smoke | official RON can construct/start/advance/resolve one battle | production entrypoint; expensive but uniquely covers end-to-end setup feasibility for current content | `retain` as representative smoke, not exhaustive publication proof |
| `tests/ron_loading.rs` PvE/preview group | official references, authored scenario content and required live warning tags | production loader; large scenario test mixes content pins but uniquely catches current bundle drift | `retain`; Phase 1 remains loader-structure owner |
| bonus-objective/result tests | post-battle event-log evaluation | result calculation is Phase 8, not setup | `defer` to Phase 8; execution not required for Phase 5 closure |

## Exact Redundancy Decisions

1. `elite_map_nodes_prefer_high_pressure_missions` has no distinguishing assertion because elite and ordinary
   Combat currently return the same one-element policy. Remove with the no-op branch or replace with a full category
   matrix if the helper remains.
2. `battlefield_archetype_does_not_change_mission_variant` only calls an unused wrapper that ignores archetype. Its
   real invariant is already covered by `CombatMissionVariant::default_for_node_type` and live pair/setup tests.
3. `defense_node_type_builds_fixed_defense_objective` asserts only objective/player plan/win-condition shapes that
   `defense_node_type_builds_default_black_box_defense_objective` asserts more exactly along with unit reference,
   route, enemy movement and defense-object position.
4. `policy_requires_at_least_one_featured_grant_kind_when_rewards_exist` and
   `defense_policy_requires_defense_identity_reward` use the same Defense currency-only failure. Merge the latter's
   empty-reward acceptance into the former or vice versa.

No other test was proven to have zero unique contract coverage. Similar template, retry and live tests are retained
where their authority source or failure semantics differ.

## Missing Safety Nets

1. C01 production sequence across preview cancel, sibling SavePoint, durable active-run JSON Continue and eventual
   battle draft scale.
2. C02 source-aware publication negatives for spawn-cell capacity and authored tactical/field-work/objective setup.
3. C03 golden conversion completeness and normal/Event/Boss/Monolith cache-key matrix after resolver consolidation.
4. C04 deterministic enemy runtime UUID/order plus full typed setup-plan parity and late-failure world no-mutation.

## Runner Evidence

Seventeen focused invocations executed 79 test instances: 79 passed, 0 failed, 0 ignored/skipped. Three instances
overlapped filters, so 76 unique tests were observed. The long live DefenseRoute smoke passed in 33.89 seconds.
Exact commands and per-runner counts are recorded in `EXPERIMENTS.md`.

## CRA-14 Verdict

- Meaning and production reachability: classified for every assigned preview/setup family.
- Assertion quality: pure topology/formula tests, official publication tests and full world handoff tests are
  distinguished.
- Duplication: four exact merge/removal cases are named with surviving safety nets.
- Coverage gaps: C01/C02 correctness gaps remain explicit despite all green executions.
- Cleanup disposition: every assigned family has `retain`, `strengthen`, `merge/remove` or later-Phase ownership.
