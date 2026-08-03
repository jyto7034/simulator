# Enemy Range Preview Removal Completion Review

Goal: G11

Review date: 2026-07-14

Guide: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Item Audit

| Item | Integrated result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| PD-08 enemy projection removal | Opponent/non-player checkpoint units omit `range_previews` in every update/recovery path | Complete | High | None |
| Unity enemy consumption removal | non-player actors never parse/store previews and reject the removed field | Complete | High | None |
| Player preview preservation | deployment facing query and deployed Player checkpoint overlay remain exact | Complete | High | None |
| Presentation exclusions | executing action telegraphs, areas/projectiles, hazards, and wave boundaries unchanged | Complete | High | None |
| Compatibility cleanup | no empty enemy field, alias, default object, server repair, or mock fallback | Complete | High | None |

## Expected Source Of Truth

Core retains two player-only preview surfaces: `DeploymentRangePreview` for local pending placement
facings and `LiveBattleUnitCheckpointDto.range_previews` for deployed `Side::Player` actors.
Opponent/non-player checkpoint actors have no preview field. Actual attacks and telegraphs use their
own execution/event presentation contracts and are not preview state.

## Runtime And Wire Evidence

- The single live checkpoint builder gates preview calculation on `Side::Player`. Because normal
  update, state request, setup-loss recovery, and resync reuse it, no parallel enemy path remains.
- `Option` plus `skip_serializing_if` removes the wire key rather than serializing `null` or an empty
  object. The renamed helper documents its player ownership.
- Exact Core serialization assertions prove Player presence and Opponent absence in the same live
  battle checkpoint. The existing deployment test proves Player checkpoint cells still equal the
  pre-deployment facing result.
- game_server forwards the typed update unchanged and retains only the independent deployment query
  mapping/test.

## Unity Evidence

- `BattlefieldFieldModel` branches on exact owner before preview parsing. Player actors require an
  object and fail closed when it is missing; non-player actors retain `RangePreviews == null` and
  fail closed if the removed key appears.
- `BattlefieldRenderHandler` still resolves a selected employee UUID and displays only the Player
  model's Core-provided cells. No enemy selection/local calculation path exists.
- Shared mocks now match the wire contract: Player actors contain the complete unavailable object,
  opponent actors contain no key. Pending deployment tests and renderer overlay infrastructure are
  preserved.

## Validation Review

- Focused Core: checkpoint side/serialization 1/1; preserved four-facing query 1/1.
- Focused Unity: side-aware reader/render 4/4; corrected battle mock flows 3/3.
- Broad Core: 746 passed, 0 failed, 0 ignored. Broad server: 30 passed, 0 failed, 0 ignored.
- Unity EditMode: 212/212. Final PlayMode: 52 passed, 0 failed, four explicit environment-dependent
  live-server skips.
- The initial PlayMode failure correctly exposed stale Player mocks; after the shared mock fix, exact
  and full reruns passed. Formatting, checks, console, scoped diff, legacy/fallback, and ignored-test
  audits passed.

## Legacy And Cross-Component Audit

- No production/current contract still calls a generic runtime-unit preview builder or promises
  `checkpoint.units[*].range_previews` for all sides.
- No enemy preview field is reserved in Core, repaired by server, accepted by Unity, or populated in
  current mocks. Historical goal documents remain evidence only.
- G11 does not preempt G10/G09: skill target selection, range policy, active execution geometry, and
  Player preview calculation are unchanged for their dedicated cutovers.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: one side gate at the canonical projection plus one strict client branch removes the
  unsupported feature from every transport lifecycle without deleting useful Player preview types or
  coupling to future targeting changes.

## Immediate Correction Candidates

None. The stale Player mock gap found by the first PlayMode run was corrected and fully revalidated.

## Follow-Up Refactor Candidates

None inside G11.

## Policy Questions

None.

## Review Conclusion

G11 is Complete / High with no immediate correction. G10 may begin.
