# Phase 14 CRA-P09-C02 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-P09-C02, Master Phase 14 of 44.
- Direct linear predecessor Phase 13 is `Complete / High`; candidate technical dependency is none.
- Phase 15 remains pending. Phase 14 mutation authority is limited to max-kills terminal delivery, direct tests and
  nearest canonical evidence.
- All three Child documents and the Phase 13 audit must be fully read before production/status/data inspection.

## Authority Notes

- Active cast state owns pending delivery, step/repeat progression and final cleanup.
- Projectile advancement owns motion/collision and may report terminal facts, but must not independently create a
  second cast-completion authority.
- A single accepted impact may satisfy multiple stop predicates; these predicates compose into one terminal outcome,
  not multiple delivery resolutions.

## Compatibility Risk

Do not change collision geometry, damage/effect semantics, authored max-kill meaning, public projectile/cast shape or
RON/save/wire contracts to make cleanup convenient. If the current tree exposes an observable stop-reason priority
that historical evidence did not settle, stop for policy rather than choosing one implicitly.

## Dirty Worktree Risk

The workspace contains extensive user-owned Core, projectile, skill, test, data, server and canonical changes from
the campaign. Fingerprint prospective targets before editing. Do not reset, restore, clean, stash or broadly format;
stop if the required hunk cannot be separated.

## Testing Notes

- Strengthen the real production integration, not only a helper or isolated runtime record.
- Assert active projectile absence, active cast absence, pending count/completion cardinality and no later event.
- Cover same-time multiple collision candidates and combined max-hits/max-kills/expiry predicates.
- Preserve first accepted damage/effect, later-target exclusion, event cause/identity/source snapshot and repeat/step
  behavior.
- Zero-test filters and mere projectile absence without cast cleanup are not proof.

## Out-Of-Scope Follow-Up Handling

Keep P09-C01 publication under Phase 15, typed reach/usefulness/preview under Phases 16~18, committed damage under
Phase 28 and shared motion extraction under Phase 29. Record independent findings without absorbing them.

## 2026-08-10 Implementation Note

The live defect reproduced exactly: `max_kills` retired the runtime but left the first cast's pending delivery at
one. The correction is private and policy-neutral: one typed post-impact stop outcome is OR-composed with the
existing queued terminal bit at the sole cast-delivery resolver. No public/RON/save/wire type changed, and the event
record still precedes its effect records.

The strengthened max-kills proof now observes both runtime owners, and a separate same-time test freezes two
collision candidates plus max-kills/max-hits/expiry as one accepted terminal delivery. Focused runtime and released
work/Boss preservation suites are green. Broad/static validation and completion review remain before Phase 15.
