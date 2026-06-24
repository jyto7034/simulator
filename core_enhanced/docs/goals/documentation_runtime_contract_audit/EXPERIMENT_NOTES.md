# Documentation Runtime Contract Audit Notes

## Initial Judgment

The audit should treat documents as claims, not facts. The working priority is:

```text
runtime code
-> live RON/data
-> Unity-facing snapshot/command/WebSocket contract
-> latest policy documents
```

`docs/README.md` and `docs/code_documentation_sync_guidelines.md` now point to external `core_unity_battle_transport_contract.md` as the live battle transport canonical document. During the audit, verify that no local document still treats the superseded setup/update split documents as the active source of truth.

## Known High-Risk Areas To Check First

- Battle transport:
  - final `battle_update` followed by `combat_result` `state_snapshot`
  - command result accepted/error split
  - `request_battle_resync` and setup-loss recovery
  - `battle_setup_snapshot` vs `combat_preview` responsibility
- Battle checkpoint:
  - `checkpoint.units[*].stats`
  - `checkpoint.units[*].hud`
  - `checkpoint.units[*].unit_source`
  - `checkpoint.units[*].range_previews`
  - deployment `position` and `facing`
- Movement and attack timing:
  - `MovementSegmentStarted` presentation source
  - checkpoint `world_position` as reconcile target
  - attack eligibility using runtime tile at release/resolve time
- Targeting and delivery:
  - tile-based basic attack eligibility
  - hit-scan vs projectile delivery
  - basic projectile target-only continuous collision
  - `range_units` not used as official basic attack eligibility
  - `range_units` still available for future contagion/contact/aura/continuous skill mechanics
- Enemy/wave identity:
  - `CorrodedEmployee`, `Abnormality`, `FacilityEntity`
  - profile id vs abnormality id
  - `profile_role`
  - normal/special/legacy echo policy
- Content docs:
  - confirmed abnormality roster
  - tool abnormalities
  - current live placeholder skill/equipment data vs design-only notes

## Stop-First Policy Questions

If these appear during audit, stop rather than silently choosing:

- A document requires a DTO field that runtime does not emit and Unity could already depend on current shape.
- A live RON schema needs a migration or defaulting policy.
- A gameplay document says one failure/reward/death/retreat timing while code implements another.
- A content document lists a roster item not present in live data, and it is unclear whether the document is design-only or runtime-required.
- A Unity document expects behavior that would require client UX changes, not just server/core cleanup.

## Follow-Up Candidates

Record, but do not automatically implement unless required for this audit:

- Removing completed goal directories after their policies are fully absorbed into canonical docs.
- Splitting `docs/README.md` goal index into generated inventory if manual maintenance becomes noisy.
- Adding a lightweight script that checks README goal coverage against `docs/goals/*/PLAN.md`.

## Audit Classification Notes

Top-level canonical/local policy docs:

- `docs/README.md`
- `docs/code_documentation_sync_guidelines.md`
- `docs/game_rulebook.md`
- `docs/skill_target_contract.md`
- `docs/codex_goal_command.md`
- `docs/refactor_preparation_plan.md`

External canonical Unity-facing docs:

- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

Content/design source docs:

- `docs/skills/lobotomy_content_catalog.md`
- `docs/skills/abnormality_skill_design_notes.ko.md`
- `docs/skills/tool_abnormality_wiki.md`
- `docs/skills/skill_fragment_system.md`
- `docs/skills/skill_fragment_wiki.md`
- `docs/skills/ego_equipment_wiki.md`

Goal docs:

- `docs/goals/*` is working memory and audit context, not permanent source of truth. Do not mass-edit old goal docs unless their README/index metadata is wrong or a goal has explicitly become canonical.

## Source-Of-Truth Nuances Found

- Setup-loss recovery:
  - Canonical direction is not hard resync inside the same battle. Unity discards the broken battle scene and returns to `node_confirm`.
  - Runtime may still send a normal command response around that state transition. The meaningful source for Unity scene recovery is the `node_confirm` `state_snapshot`, not a rebuilt `battle_setup_snapshot`.

- `range_units`:
  - It is not official basic attack eligibility in DefenseRoute.
  - Runtime still keeps `range_units` for movement/approach helpers and future contagion/contact/aura/continuous mechanisms.
  - This is not a contradiction as long as target eligibility uses tile range policy and final cells.

- Weapon archetypes:
  - Runtime enum still contains `GrenadeLauncher`.
  - Live equipment RON currently uses `Sword`, `Spear`, `Shield`, `Bow`, `Gun`, `Staff`.
  - `GrenadeLauncher` should be treated as enum-only/future candidate until a later content/schema goal activates it.
  - `Axe`, `Crossbow`, and `Shotgun` are not current runtime enum variants.

- Content docs:
  - `lobotomy_content_catalog.md` and `abnormality_skill_design_notes.ko.md` describe target content design, not proof that each detailed mechanic has live RON/runtime support.
  - Missing detailed boss/elite mechanics in live RON are implementation backlog, not an audit mismatch by themselves.

## Follow-Up Candidates From Audit

- Decide whether `WeaponArchetype::GrenadeLauncher` should stay as an unused future enum variant or be removed until intentionally reintroduced.
- Migrate or delete `/mnt/f/work/simulator/game_resources/data/abnormalities/legacy_random_event_abnormalities.ron` through a content/data cleanup goal if it is no longer loaded or needed.
- Consider a lightweight `docs` linter that warns when local canonical docs refer to superseded external battle setup/update split contracts as current sources.
