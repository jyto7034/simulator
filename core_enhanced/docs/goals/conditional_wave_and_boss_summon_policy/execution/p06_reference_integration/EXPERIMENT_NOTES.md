# P06 Reference Integration Current Notes

## Current Summary

- Current pass: `GEX-20 complete / Complete / High / AUDIT-CWBS-P06-GEX20-001`
- Dependencies: `P01..P05 Complete / High`
- Policy question: `none`
- Blocker: `none`
- Completion: `Complete / High`
- Full validation: final frozen Core `1,131 / 0`, server `73 / 0`, 25-member pre/post hash `MATCH`; earlier failed/hung
  attempts and their corrections remain recorded in `EXPERIMENTS.md`

## Preserved Boundaries

- Placeholder is nonproduction proof data, not a lore/balance/reward decision or promotion.
- Existing WhiteNight/Little Red content is not repurposed as the placeholder.
- Representative vertical coverage does not require encoding every supported trigger/placement/failure combination.
- Normal BossOmen, fragment/repeat reward, all-FinalBoss record and ordinary progression must not consume the placeholder.
- Active battle/candidate state remains same-process only and SavePoint-free.
- Unity, deployment and Released/Deployed claims remain outside scope.

## Initial Technical Direction Within Approved Policy

- Inspect the current closed PVE, Boss Runtime, summoned-unit, BossOmen and progression registries and choose the smallest
  explicit reference-only discriminator that all relevant selectors can enforce.
- Author a dedicated live encounter/boss/reference identity using only the already implemented strict Wave, Summon and
  outcome schemas.
- Add one durable vertical integration fixture plus explicit registry-exclusion assertions; reuse predecessor focused
  tests for the broader combination matrix.
- Update only current canonical documents whose assertions conflict with the implemented target.
- Freeze relevant inputs, run the final full Core/server suites, and perform fresh GEX-20 and Master GEX-30 audits. A
  failed full attempt is evidence, not a baseline; after the narrow correction, re-freeze before the affected rerun.

## Current Technical Decisions

- A new exhaustive `ReferenceFinalBoss` enum variant is safer than a bool/default or ID-prefix convention: compiler
  matches and exact-class production selectors make accidental promotion observable, while the explicit test locks every
  closed registry that is allowed to exclude by absence.
- The dormant fragment row exists only because the strict abnormality catalog requires every abnormality to reference a
  valid fragment metadata identity. It is not a reward source; PVE reward and ordinary reward registries contain no grant
  path for it.
- The placeholder intentionally proves one representative conditional trigger and one summon/signal lifecycle. Other
  supported trigger, occupancy, wait/failure and removal combinations remain predecessor focused fixtures as authorized.
- Existing predecessor audits remain current: P06 consumes their public seams and adds an expected standalone reference,
  explicit enum branch, integration tests and canonical publication without changing their approved semantics.
- Empty command-unit scenarios do not gain an implicit terminal outcome merely to make tests terminate. The test-only
  finite-causality drain explicitly ignores the self-rescheduling movement clock; production event scheduling is
  unchanged.
- Production content-index audits select by the typed reference encounter discriminator and exclude its required
  dormant catalog fragment; reference metadata remains loadable validation data but is not promoted into released
  content documentation.
- Production-only RON audits likewise derive exclusions from the typed reference encounter rather than raw ID prefixes;
  reference definitions remain subject to connected-data validation and dedicated zero-authority assertions.
- Projectile characterization stops at an explicitly authored test-only time outcome once its required causal events
  are observable; it no longer depends on an unrelated fixture battle eventually escaping a stalemate.
- Replacement-session attach is a disconnect boundary: it auto-pauses the active battle, increments battle revision and
  returns immutable setup plus the paused current update. Reconnect does not resume combat implicitly.
