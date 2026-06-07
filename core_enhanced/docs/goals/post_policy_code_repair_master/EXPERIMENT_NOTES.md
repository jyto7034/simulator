# Post Policy Code Repair Master Notes

## Source-Of-Truth Notes

- `docs/goal.md` is the active instruction entry point.
- `docs/post_policy_code_repair_master_goal.md` is the master implementation plan.
- Test verification report says `game_core` previously failed on `authored_protect_unit_tactical_plan_overrides_default_defense_contract`.
- Test verification report says `game_server` test cfg previously failed because test fixtures missed `mobility_kind`, `target_traits`, and `unit_deploy_costs`.
- Technical debt report now treats `Observed` threat warning as an unused contract surface to remove, not a future observation system.
- False threat rumor policy is fixed:
  - 20% chance.
  - At most one rumor.
  - Candidate must be absent from actual resolved spawn wave warning tags.
  - Retreat/re-entry marks false rumor `Disproved`.
  - Real warnings stay `Unverified`.

## Open Policy Stops

- External stale resource file deletion requires user approval after audit.
- If `effective_profile_error` needs a different shape than `{ code: string, message: string }`, stop and ask.
