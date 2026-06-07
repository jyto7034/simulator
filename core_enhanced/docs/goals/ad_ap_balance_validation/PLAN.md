# AD AP Balance Validation Plan

## Current State

- Active sub-goal: `docs/ad_ap_balance_validation_goal.md`.
- Parent master goal: `docs/core_policy_implementation_master_goal.md`.
- Goal 10 Skill Fragment Compatibility is complete; this is master goal 11.
- Damage formula and combat preview code have been inspected.
- Current implementation finding:
  - Defense/magic resist mitigation alone cannot create permanent immunity when a damage request has `minimum_damage > 0`.
  - Current preview warnings are generated from actual preview spawn wave entries, with a small seeded rumor chance.
  - There is no explicit boss immunity exception metadata in current data.
- Canonical Unity docs live outside this repo:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

## Execution Guardrails

- Do not use `git restore`, `git reset`, or other git file modifier commands.
- Do not copy historical git contents into the current tree with `cp` without explicit user approval.
- Do not revert files to an older state without explaining why and asking the user first.
- Read runtime code and live data before trusting docs.
- Do not add an automatic balance solver.
- If a numeric threshold or boss exception shape cannot be justified from current code/data, stop and ask.

## Initial Scope

- Completed: inspect damage mitigation formula.
- Completed: inspect enemy stat sources and enemy rank/category data.
- Completed: inspect combat preview threat warning generation and validation.
- Implement minimal data validation for clear hard-counter risks:
  - Permanent type immunity is not representable through current defense/magic resist stats alone.
  - Warning omissions for clearly high defense / high magic resist waves use the existing code-backed thresholds.

## Open Questions

- None blocking for the current minimal implementation.
- Deferred policy: if future boss mechanics need actual type immunity, add explicit exception metadata and discuss its shape before implementation.

## Implementation Direction

- Add a shared `combat_balance` policy module for AD/AP warning thresholds and damage feedback mitigation threshold.
- Make `combat_preview` derive required briefing tags through that shared policy.
- Add `GameDataBase::new` validation that generated previews include every required briefing tag for representative seeds.
- Add focused tests proving current resistance math does not create permanent type immunity when minimum damage exists.
