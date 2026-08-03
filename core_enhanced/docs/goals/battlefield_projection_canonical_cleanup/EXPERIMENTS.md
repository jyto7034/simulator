# Battlefield Projection Canonical Cleanup Experiments

Goal: G13

Updated: 2026-07-14

## Entry gate

- R04, G11, G10, and G09: Complete / High.
- DCA-H-07 was revalidated as Low stale Core wording; Unity implementation, canonical transport,
  and PlayMode proof already agree.
- DCA-M-10 is Low reserved-variant wording; no runtime feature decision is authorized.

## Dirty baseline

- Simulator status fingerprint:
  `9592214e1a80a08a502c3539648ad36959d02e85bb9e4d1fd2060361514b3040`.
- Unity status fingerprint:
  `d4cbf7fe310ed5a6d2eb3337c344bc75b4963519f0caa944b70410bd9908daee`.
- Both worktrees contain extensive closed-goal and unrelated changes. G13 is documentation-only;
  preserve all existing hunks and do not stash/reset/restore/clean.

## Revalidation

- Core runtime canonical still said the HUD uses camera billboard/up-vector alignment, contradicting
  the actual dedicated `ScreenSpaceOverlay` canvas.
- Unity's actor HUD test pins screen-space canvas mode and the projected lower world anchor.
- `BattleUnitSourceIdentity` and Unity's strict parser retain `facility_entity`, but live PVE
  validation rejects every FacilityEntity wave and setup has no FacilityEntity profile builder.
- External transport text correctly hinted that implementation was required, but still grouped
  FacilityEntity under “current production DTO variants,” overstating reachability.

## Canonical correction

- Core runtime now states the exact two-stage contract: actor lower world anchor projected through
  `Camera.WorldToScreenPoint`, rendered in a dedicated screen-space overlay; no world-space
  canvas/mesh or camera billboard/up-vector alignment.
- External transport now distinguishes defined wire enum shapes from current production actor
  variants and marks both Facility setup/preview and live actor identity as reserved/unproduced.
- The catalog-reference paragraph no longer casually describes a current “general facility
  entity”; it identifies the profile id as wire-reserved.
- No Rust, RON, server, Unity code, schema, or runtime behavior changed.

## Validation

- Core focused live PVE reference test: 1/1; it would panic on a live FacilityEntity wave.
- Unity exact HUD PlayMode test: 1/1; it pins `ScreenSpaceOverlay` and projected lower-anchor
  position. Final cleared console: zero errors.
- Immediately preceding broad G09 gate on the same production worktree: Core 750, server 30, Unity
  EditMode 215, Unity PlayMode 52 passed with four explicit environment skips.
- Current-canonical billboard/production-Facility searches and Core/Unity scoped whitespace checks
  passed.
