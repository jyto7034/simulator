# Phase 05 Candidate Reaudit

## Current Dispositions

| Candidate | Current evidence and Boss impact | Disposition | Current work/readiness |
| --- | --- | --- | --- |
| CRA-P05-C01 | V3 still carries preview caches; deserialization defaults skipped `enemy_stat_scale`; world cache early-return and enemy spawn setup still trust it | `still_valid` | correctness; durable Continue sequence first |
| CRA-P05-C02 | publication now enumerates every manifested difficulty case and five seeds for every encounter, builds the full empty-player setup through the runtime predicates, including Boss binding | `fully_implemented` | closed correction; retain negatives/matrix |
| CRA-P05-C03 | two instance→preview mappings, two world resolvers and unread `floor_index` threading remain | `still_valid` | behavior refactor; after C01 and public caller inventory |
| CRA-P05-C04 | private `ResolvedCombatSetupPlan` resolves encounter/rewards/scenario/BattleCore/execution once and production world planning consumes it; typed optional Boss setup is inside that plan | `fully_implemented` | closed behavior refactor; retain parity/ordinary opt-out |
| CRA-P05-C05 | elite/ordinary mission branches still return the same policy, unused archetype variant wrapper remains test-only and shape-only tests remain | `still_valid` | cleanup; assertion transfer/caller inventory |

## Deep Closure Evidence

### CRA-P05-C02

`manifested_difficulty_cases` enumerates every published profile's ordinary bands and role tuning, including both
Monolith stages. `validate_combat_preview_threat_warning_contract` crosses those cases with every encounter and seeds
`[0,1,17,41,99]`, generates the preview and calls
`CombatExecutor::validate_live_combat_setup_feasibility`. That path builds battlefield, player group, defense object,
enemy groups, authored tactical/win condition, optional Boss setup, scenario validation, BattleCore and initial
execution without world mutation. Structured live-candidate tests reject manifested-only preview failure and a Boss
binding that cannot build setup. This closes the original full correction, not just Boss bindings.

### CRA-P05-C04

`resolve_live_combat_setup_plan` performs the original target: one encounter lookup, preview/mission validation,
reward resolution/scaling, resolved battlefield plan, player start, scenario/Boss setup, obstacle/scenario validation,
BattleCore and execution. It returns node/mission/reward/battle/execution in one private typed plan. Production world
planning destructures that plan only to build the world-owned active session and commit its lifecycle. Legacy public
test-compatible builders remain, as the original compatibility boundary required. No original setup field or
ordinary no-binding parity seam remains outside the aggregate production path.

## Remaining Rows

- C01 retains original axes `3/3/3/3/3/3/unknown`. The production preview→cancel→SavePoint→durable Continue→start
  safety test is still absent; no V3 field removal is required to fix it.
- C03 retains `3/1/2/3/3/2/unknown`. `BattlefieldGenerationRequest.floor_index` remains unread and both conversion/
  resolver families remain; public `*_at_floor` removal is externally gated.
- C05 retains `1/1/1/1/2/1/unknown`. Cleanup must preserve the Boss category mapping, SplitRoom rejection and all
  stronger mission/reward assertions.

## New Candidate And Gates

Boss publication/setup is absorbed by the fully implemented C02/C04 targets. No distinct `CRR-P05-*` root cause was
found. Preview V3 field removal, public helper deletion, DTO/schema changes, content/balance and unknown external Rust
callers remain explicit gates.
