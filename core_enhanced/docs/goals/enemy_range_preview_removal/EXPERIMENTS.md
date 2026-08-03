# Enemy Range Preview Removal Experiments

Goal: G11

Updated: 2026-07-14

## Entry gate

- R04: Complete / High; Phase 4 persistence/profile/delivery boundaries are fixed.
- Accepted PD-08 removes enemy preview only and forbids a dormant compatibility field.
- G10 and G09 remain blocked until this removal and review complete.

## Dirty baseline

- Core view status fingerprint at R04 handoff:
  `38e36d105ef98ce13906acbe28affc95711810aa840f7e21ab89390f97633d91`.
- game_server view status fingerprint:
  `a564aa827f7346c1b58930df2acc509e9137cdd4b341a93790f08a5737b3f0df`.
- Unity status fingerprint:
  `cd6eeed715cde88a4f1e429accbff76ccf3a2d8a75d31f3c0c712ecb2b7db28a`.
- Core/server tracked diff fingerprint:
  `68a951cfc8e1458e6e6a853306eac897f0e3b6dd4b72febd03bdcc517fdf1135`;
  Unity tracked diff fingerprint:
  `705344cba78f3aef476002656a2c4c2f7e80125bbbac117bff642c780d0c6c96`.
- All worktrees are heavily dirty. No stash/reset/restore/clean/baseline commit is authorized.

## Runtime trace

- `LiveBattleUnitCheckpointDto.range_previews` was required for every active unit, and the sole
  checkpoint builder called `range_previews_for_runtime_unit` without a side gate. Normal update,
  state request, reconnect recovery, and resync all reuse this projection.
- `DeploymentRangePreview.payload.facings.*.range_previews` is a separate read-only pre-deployment
  player query and must remain.
- Unity `BattlefieldFieldModel` parsed `range_previews` for every actor. Rendering used a selected
  employee UUID and therefore did not intentionally display an enemy overlay, but enemy preview
  state still crossed and remained in the client model.
- game_server has no checkpoint-field mapper; it generically forwards Core battle updates. Its only
  named range-preview code is the preserved deployment query test.

## Implementation

- Changed the checkpoint unit field to an absent-when-none optional wire value and populated it only
  for `Side::Player`. Renamed the runtime builder to
  `range_previews_for_player_runtime_unit` to make ownership explicit.
- Core wire regression asserts Player units serialize the field and every Opponent unit omits it.
  The deployed-player equality assertion still proves checkpoint preview matches the preserved
  facing-query result.
- Unity now requires `range_previews` on Player checkpoint units, never parses it for non-player
  actors, and rejects an enemy compatibility field. Its unit model holds preview state only for the
  Player branch.
- Updated Unity test helpers and `ArkMockSnapshots` so player checkpoint actors carry an exact
  unavailable preview object while opponent mocks carry no field.
- Core runtime, server bridge, Unity core, and battle transport documents now state the exact
  side-specific contract. No live RON or skill targeting semantics changed.

## Validation and correction log

- Core all-target check passed. Exact defense checkpoint serialization and preserved pending
  deployment-facing preview tests passed 1/1 each.
- Unity script validation returned zero errors for production and tests. The four exact Player /
  opponent parser/render tests passed 4/4.
- First full PlayMode run failed its earliest battle mock tests because old Player mocks omitted the
  newly required field; later UI tests cascaded from leaked failed-test objects. This was a genuine
  G11 mock-contract gap, not counted as green evidence.
- Added the strict Player-only preview object to the shared mock update builder. The three earliest
  failed battle mock tests then passed 3/3, and the complete PlayMode rerun passed.

## Broad validation and hygiene

- Core all-targets: 746 passed (704 library + 3 + 3 + 19 + 13 + 4), 0 failed, 0 ignored. Check,
  formatting, and repository diff check passed.
- game_server all-targets: 30 passed, 0 failed, 0 ignored. Formatting passed; no server production
  mapping changed.
- Unity EditMode: 212/212. Unity PlayMode final rerun: 52 passed, 0 failed, four explicit live-server
  environment skips. Final console query after clearing test-runner result noise returned zero errors.
- Scoped Unity diff check passed. Searches found no old generic runtime helper, generic
  `checkpoint.units[*].range_previews` current promise, compatibility alias/fallback, or relevant
  ignored test.
- Existing executing attacks, telegraphs, hazards, projectile/area presentation, wave boundaries,
  deployment-facing query, and selected Player overlay code remain intact.
