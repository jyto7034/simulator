# Phase 00 Current Tree Baseline

Captured: 2026-08-09 (Asia/Seoul)
Repository: `/mnt/f/work/simulator`
HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Baseline authority: current dirty working tree, not HEAD alone

## Exact Status Identity

The canonical reproduction command is:

```text
git status --short --untracked-files=all | LC_ALL=C sort | sha256sum
git status --short --untracked-files=all | wc -l
git status --short --untracked-files=all | awk '{k=substr($0,1,2); count[k]++} END {for (k in count) print k, count[k]}' | LC_ALL=C sort
```

Captured after the three P00 required documents and Master activation record existed:

| Measure | Result |
| --- | --- |
| sorted status entries | 278 |
| modified entries | 86 |
| untracked file entries | 192 |
| sorted status SHA-256 | `1bb7417418cd38f62719d7021f98876fcae657b6a2c4261aa7e212f6ccacbb96` |

The Phase path did not exist before initialization. Filtering only the three P00 files reconstructs the pre-P00
path/status set:

```text
git status --short --untracked-files=all \
  | rg -v 'core_enhanced/docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_00_current_tree_delta_inventory/' \
  | LC_ALL=C sort | sha256sum
```

Result: `275` entries, SHA-256
`43f4ae0861a80f8f800fd82a676930ef10822c1e71a9cbe82b3ac5284fea17a3`.

Filtering the entire new reaudit campaign reconstructs the exact completed Boss campaign baseline recorded by its
P07 and Master audit:

```text
git status --short --untracked-files=all \
  | rg -v 'core_enhanced/docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_(master|phase_00_current_tree_delta_inventory)/' \
  | LC_ALL=C sort | sha256sum
```

Result: `272` entries, SHA-256
`8d03ae2f1f4777976adc9a0f05c7b493119433f18cc33addd3ff6768fc780481`, exactly matching the historical Boss Master
completion audit. This proves no path/status entry outside the new reaudit campaign changed between that audited
baseline and P00 capture.

## Relevant Content Identity

Status identity does not hash file contents. The following command selects every current modified/untracked
production or current-method/canonical path relevant to this campaign while excluding historical Goal artifacts,
temporary directories and the unrelated server log, hashes each file, sorts the manifest, then hashes the manifest:

```text
git status --short --untracked-files=all | cut -c4- \
  | rg '^(core_enhanced/(src/|tests/|docs/(README\.md|core_runtime_contract\.md|data_loading_contract\.md|engineering_workflow\.md|game_rulebook\.md|goal_completion_review_guide\.md|skill_target_contract\.md|skills/|decisions/|goal_authoring_and_execution_guide\.md))|game_resources/data/|game_server/(src/|tests/|examples/|docs/))' \
  | LC_ALL=C sort | xargs sha256sum | LC_ALL=C sort | sha256sum
```

Result: `96` files, manifest SHA-256
`9638cfa2d05d8d04966cf49fde7abde79b318a25a6452e38c04fc3ce3284b284`.

This manifest is the content baseline that boundary Phases must compare when deciding whether previous Phase
evidence can be reused. A Phase that writes only its own audit artifacts must leave this count and hash unchanged.

## Boss-Critical File Identity

| Current file | SHA-256 |
| --- | --- |
| `core_enhanced/src/game/data/boss_runtime_data.rs` | `a5fa4f770365a8a5e2db5188bb1c5f7627fcca4fa844a89d12ebade9bf75f306` |
| `core_enhanced/src/game/data/live_ron.rs` | `a0402c6e7a188817223910309f4aff9bfba4a41a0f292fce5fb1daeb0b16f03c` |
| `core_enhanced/src/game/data/mod.rs` | `70f6fbf61cb9e5e26e1aea51df94656998a7c3d4530c4362a1ab3a3b8b535e64` |
| `core_enhanced/src/game/data/pve_data.rs` | `3dfb09a666671d0da5ec691a136bcf1cba88cbb6564297c9e63f2a13671f65e2` |
| `core_enhanced/src/game/combat_setup/boss_runtime.rs` | `5fcb690ee1d1c2c1c7899cd92b0ccfc258cabe4a658fd59cb4f4c793d62ce357` |
| `core_enhanced/src/game/events/combat.rs` | `999f9fb16711f90dfc63619749bbda565ec8fa5765169794aaaa1014155a326a` |
| `core_enhanced/src/game/battle/scenario.rs` | `6a65d9f6026950fbebc34e8070196c289c355b1d12c14950faf88ed4a11e6d8d` |
| `core_enhanced/src/game/battle/enums.rs` | `c76de66154a123217fb16da8d3fa8ccc640746bb384b75f6503976f1c1a55d9e` |
| `core_enhanced/src/game/battle/core/boss_runtime.rs` | `a4b99993f0be867d940952febd2c31cc611993205db310d246e17fae1d0066fd` |
| `core_enhanced/src/game/battle/core/mod.rs` | `a8acf3fcdffc6d39f48c90fa9065e998480827e6b9f086050b82fc42cd4552e5` |
| `core_enhanced/src/game/battle/core/sim.rs` | `780361c78d42170dac289f841e58da7afc5fe2c419e1c3a6bdfcc499a90abbbb` |
| `core_enhanced/src/game/battle/core/movement/planner.rs` | `cb4cbec373f8638eba3f2eeb6602bbaca45b35f72bf2854aa0e0678b80fb8d8f` |
| `core_enhanced/src/game/battle/core/basic_attack.rs` | `071b9a4552d5bb2cae7b94cbdf4b55e2f47bcaf60e6488ad180fc50751a86339` |
| `core_enhanced/src/game/world/tests/combat.rs` | `4cd00ae3db74d0ac0020c7e523432ae58c95cf4b67711140b326f5d0d25acc3c` |
| `core_enhanced/tests/ron_loading.rs` | `75558ff28be83cafcfe535252f552903b2077249c2cd81ef00b41d63195854b3` |
| `game_resources/data/boss_runtime/base.ron` | `f696c9d054ed1e4397809f80c2707e06614f4324a1829bb7efa43ccb666c223a` |
| `game_resources/data/pve/encounters.ron` | `0b3f03b7e68c3aff5db6270e9eaa83eacb4eb2da188e55058899db22f0a3ee1c` |
| `game_server/src/game/player_game_actor/handlers.rs` | `63afec7f591c80ba78a369513e403571a5502712b8e7b1190705271d69021e6b` |

These hashes freeze identity only. They do not prove behavior or candidate disposition.

## Dirty Categories And Ownership

| Category | Current evidence | P00 handling |
| --- | --- | --- |
| Core canonical/method docs | modified plus untracked authoring/decision docs | read-only current method/contract evidence |
| Core runtime/data/test | many modified files plus untracked Boss/field-work/tactical modules | read-only production evidence; assign to P01~P10 |
| production live data | modified existing RON plus untracked `boss_runtime/base.ron` | read-only authored evidence |
| game_server | modified docs/example/actor source | read-only public/host boundary evidence |
| original refactor audit | untracked Goal directories | historical evidence; never rewrite |
| Boss Runtime campaign | untracked Goal directories | historical evidence; never rewrite |
| tactical campaign | untracked Goal/source surface | user-owned parallel history; only impact-check shared files |
| unrelated log | `game_server/logs/app.log.2026-08-09` | explicitly preserve and exclude |
| this reaudit | Master plus active P00 artifacts | only authorized mutation scope |

## External Unity Baseline

Repository: `/mnt/f/unity projects/ark`

| Measure | Result |
| --- | --- |
| HEAD | `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf` |
| sorted status entries | 2156 |
| sorted status SHA-256 | `951364f7944787d594040e71707548d2a922887d9c5a23b9d0c982e5aac2e6d2` |

The historical Boss completion audit recorded 2154 entries. P00 does not infer ownership of the additional two
current entries and adopts the current 2156-entry tree as the read-only consumer baseline. No external file was
written. Boundary Phases must inspect only exact public-contract evidence they need and classify all other external
state as unverified or out of scope.

## Preservation Gate

At every Phase exit:

- compare the 96-file relevant content manifest against
  `9638cfa2d05d8d04966cf49fde7abde79b318a25a6452e38c04fc3ce3284b284`;
- compare status after excluding the entire new reaudit campaign against the 272-entry
  `8d03ae2f...` baseline;
- ensure the unrelated server log remains present and untouched;
- record any external status drift as external state, not as a campaign mutation;
- do not use unchanged hashes as candidate closure evidence without reading the assigned runtime/data/test path.

