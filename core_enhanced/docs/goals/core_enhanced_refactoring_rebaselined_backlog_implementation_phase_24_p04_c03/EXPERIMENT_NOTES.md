# Phase 24 CRA-P04-C03 Experiment Notes

Status: complete
Implementation Complete: true
Candidate: CRA-P04-C03
Master Phase: 24 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Locked Decisions

- This is a behavior-preserving private ownership refactor, not an Event or Shop behavior correction.
- Persisted run sessions and active-node mirrors remain distinct required roles.
- Event and Shop may keep distinct commit paths; superficial field similarity is not authority to genericize them.
- Required-run versus active-only semantics must be explicit and remain behaviorally exact.
- Session identity, Event scene/check/grant/battle history and Shop page/offer/purchase/refresh history remain exact.
- Checkpoint/re-entry/retreat, result/error order, public/admin fixtures and serialized consumers remain exact.
- Phase 25 HQ transaction and Phase 26 VN assembly/placement are prohibited.

## Open Evidence Slots

1. Activation status, HEAD and normalized dirty hash.
2. Prospective source/test/data/canonical hashes before content inspection.
3. Exact persisted Event/Shop cache types, keys and active mirror variants.
4. Complete Event/Shop/HQ production entry/update caller list.
5. Existing helper signatures, clone/move order and missing-run semantics.
6. Read/projection/checkpoint/re-entry/retreat/public/admin consumer inventory.
7. Baseline nonzero test filters and current counts.
8. Test-first complete paired equality and explicit no-run matrix.
9. Smallest private domain boundary and exact migrated callers.
10. Negative direct-write/fallback and Phase 25/26 prebuild search.
11. Public/V3/RON/DTO/server/Unity/canonical comparison.
12. Focused/broad debug/release/static evidence and independent audit verdict.

## Known Historical Risks To Recheck

- A caller may update only the active mirror or only the persisted cache and still pass shallow projection tests.
- The two values may be equal at entry but diverge after check, purchase, refresh or other in-session mutation.
- Clone versus move ordering may matter for session identity/history or error timing.
- Missing-run Event and Shop behavior historically differs; merging it silently would break fixtures or runtime use.
- HQ-to-Shop occurs inside a broader HQ transaction and must not prebuild or weaken Phase 25 rollback/context work.
- Checkpoint/re-entry and retreat may read the persisted copy at a different lifetime than active projection.
- Public/admin fixtures may construct active-only content even if production map entry always has a run.
- Negative workspace search cannot justify removing public/session/serialized surfaces.

## Historical Evidence Summary

- Original evidence identifies direct paired writes at Event map entry, Shop map entry and HQ-to-Shop in addition to
  the domain helpers.
- Reaudit confirms both helpers and direct sites remain, and BossOmen/Monolith add consumers rather than authority.
- Final row 24 requires every Event/Shop/HQ entry/update to preserve paired state and explicit no-run semantics.
- Test disposition is `retain + strengthen`; current Shop session tests observe both roles but do not prove every
  transition uses one paired authority.
- P04-C03 is a required dependency of Phase 25 CRA-F-C02, so HQ Shop behavior must be frozen without implementing
  the common HQ transaction.

## Activation Boundary

No Phase 24 production source, test implementation, live data or canonical content has been inspected during Child
initialization. After Master sole activation, reread this entire Child and Phase 23's audit, capture prospective
fingerprints, and only then begin production investigation.

## 2026-08-10 — N01 Activation Baseline

- Mandatory post-activation reads completed: Child 191/51/60 and predecessor audit 264/264.
- HEAD `f318414f...`; expanded dirty state 496 entries at normalized SHA-256 `e843ff04...`.
- Exactly three new Phase 24 Goal paths explain the 493→496 increase; production/test/data/canonical status did not
  change during initialization or activation.
- Prospective hashes are recorded in E02. Most likely targets are pre-existing modified files, so all Phase edits
  must remain separable and no unrelated cleanup is permitted.
- Disposition: activation gate complete; current source/caller/test investigation may begin, while production
  mutation still waits for test-first paired-state characterization.

## 2026-08-10 — N02 Current Semantics And Characterization

- Exact production duplication is one normal Event entry, one map Shop entry and one HQ-to-Shop handoff in addition
  to the current domain helpers. Shop purchase/refresh already delegate; Event scene/check updates already delegate.
- Started-Combat Event re-entry is persisted-only by design because Combat becomes active content. Shop sale does not
  mutate session history. Neither belongs in the paired consolidation.
- Run-required Event/map-Shop entry returns `MissingResource("RunState")` before active mutation. In-session Shop
  updates without a run preserve active-only state; this is exercised by the existing public-like unit fixture.
- Test-first complete-value equality and no-run evidence passes 70/70 across changed families after correcting the
  initially assumed error variant. Full baseline support is 106/106.
- Target is one required-run Event helper and one typed-mode Shop helper. P24-C01 and P24-C03 are evidenced;
  production edits may begin without changing gameplay, persistence or Phase 25 ownership.

## 2026-08-10 — N03 Implemented Authority And Focused Evidence

- Event's single required-run paired helper owns normal entry and all active Event mutations. Shop's single typed-
  mode helper owns map/HQ entry and purchase/refresh, with required-run versus active-only allowance explicit.
- Complete session equality, IDs/history/page/purchase/check state, checkpoint/re-entry, authored scene advance,
  exact no-run error and active-only updates remain green through the new boundary.
- Focused implementation evidence is 109/109 and Core all-target check/fmt/diff pass.
- Persisted-only Event combat handoff, admin fixtures, public session types, serialized caches, snapshot readers,
  common completion and HQ transaction are deliberately retained.
- P24-C01–P24-C08 are complete; P24-C09 awaits broad validation and P24-C10 awaits independent review.

## 2026-08-10 — N04 Final Validation And Review Handoff

- Focused total is 109/109. Broad totals are Core 1006 unit + 51 integrations in debug/release and server 59 in
  debug/release; four checks, formatter and diff gate pass.
- Diagnostic Clippy remains Core 82/135 and server 3/7, with no Phase 24 helper/caller warning.
- Canonical hashes remain `6e20d6a6...`, `6c53ecd5...`, `4d734af0...`, `a3224e24...`; no public/session/schema/
  wire behavior changed.
- Activation was 496 rows at `e843ff04...`; pre-audit is 498 at `dceeff2a...`. The scoped production/test hashes
  are recorded in E05 and unrelated dirty work remains preserved.
- P24-C01–P24-C09 are complete. P24-C10 awaits the independent `Complete / High` phase-exit audit; Phase 25 remains
  pending and no common HQ transaction/context work was prebuilt.

## 2026-08-10 — N05 Completion Disposition

- Independent audit confirms one Event required-run commit owner and one Shop typed-mode commit owner cover every
  legitimate paired production write. The persisted-only Event combat handoff and fixture/admin direct setup remain
  correctly classified exceptions.
- Public/session/serde/checkpoint/DTO/server/Unity behavior is unchanged, and no Phase 25 HQ transaction replacement
  or Phase 26 VN placement abstraction was introduced.
- Verdict is `Complete / High`; improvement class `none`. No policy question, compatibility layer, correction or
  Phase-specific follow-up remains.
- Final audit-inclusive status is 499 rows at `1f576cd8...`; unrelated dirty work remains present.
