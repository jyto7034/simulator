# Experiment Notes

## Policy Notes

- Source-of-truth order: runtime code, live RON/data, Unity DTO contracts, then policy docs.
- New policy decisions discovered during implementation must be recorded as `사용자와 정책 논의 필요` and the active goal should complete with a policy-decision report.
- Facility-like structures are planned for the future, but current unsupported forms should fail validation until explicit schema exists.
- Resolved policy decision: `RUN_SYSTEM_POLICY data source` uses one integrated live RON file at `game_resources/data/run/policy.ron`. The file owns setup, support, post-battle, headquarters, and live deployment policy as one object.
- Resolved policy decision: `MapViewDto.available_node_ids` / `completed_node_ids` are removed from the DTO. `MapNodeDto.state` is the canonical Unity/server-facing source for availability and completion.
- Resolved implementation rule: Defense combat previews must use authored route data. Battlefield templates that can back Defense encounters define `defense_main`, and Defense PVE waves explicitly reference that route. Runtime preview generation no longer creates generated Defense routes or empty fallback waves.
- Map state SoT note: because `MapNodeDto.state` is now the DTO source for availability, runtime progression must clear stale `Available` states when the internal selectable set changes. Selecting one fork choice marks unselected previously available siblings as `Revealed`.
- Resolved implementation rule: starter employee loadouts are authored on `StarterEmployeeCandidate.starter_loadout`. Start-game initialization creates owned equipment instances, equips them on the employee, and stores the same equipped ownership in inventory. Starter baseline skill fragments come from the candidate loadout rather than `Employee::new`/runtime injection.
- Production loader rule: live skill fragment RON is loaded as-is. `SkillFragmentDatabase::with_builtin_starter` and `SkillFragmentMetadata::starter_basic_attack` are retained only under `cfg(test)` as unit-test fixture helpers, not as runtime/live-data fallbacks.
- Content note: current live starter equipment uses existing `standard_armor`; no new default weapon was invented because the employee default combat profile already supplies baseline basic attack behavior and no explicit starter weapon content exists yet.
- Resolved implementation rule: legacy random-event RON files are not kept in the live data tree. This includes the old event pools/events/reward/shop files and the legacy random-event abnormality file, because none are loaded by official runtime/test/server loaders and keeping them beside live data creates a false source of truth.
- Resolved implementation rule: battlefield archetype fallback selection is data-driven from live `map/battlefield_archetypes.ron` via `enabled` and `weight`. `SplitRoom` remains in schema/templates as future content but is disabled and validation rejects enabling it before implementation.
- Resolved implementation rule: `FacilityEntity` remains a future PVE schema variant, but current live PVE waves cannot use it; `GameDataBase` validation fails before runtime spawning would reach missing facility profile semantics.
- Resolved implementation rule: official server startup validates generated combat preview contracts immediately after live RON loading and `GameDataBase` construction.
- Resolved implementation rule: `battle_records/run_<seed>/*.json` is an always-on debugging artifact. It may be used by tests/debugging to inspect exported combat results, but runtime gameplay, replay reconstruction, live RON loading, and server contracts must not treat it as canonical source-of-truth.
- Resolved implementation rule: map generation policy values live in `game_resources/data/map/generation_policy.ron`. The migration moved the old generator constants verbatim into data instead of inventing new balance: early row category weights, pre-boss row category weights, safe replacement category order, combat row cap divisor, and row repair guarantees are now authored data.

## Follow-Up Candidates

- None yet.
