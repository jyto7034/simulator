# Live Skill Catalog Evidence Governance Experiments

Goal: G20

Updated: 2026-07-14

## Entry gate

- G11, G10, G09, G13, and G18: Complete / High.
- Final Phase 5 schema/runtime/examples are fixed; G20 must not change gameplay behavior.

## Dirty baseline

- Simulator status fingerprint:
  `8b61c0f11f9264c1ed580495fc538a1b1d00ef105c8b390ad7cf2bc1e2280328`.
- Unity status fingerprint:
  `d4cbf7fe310ed5a6d2eb3337c344bc75b4963519f0caa944b70410bd9908daee`.
- Preserve all unrelated and closed-goal work; no cleanup/reset/stash is authorized.

## Revalidation

- Manifest v1's `behavior_coverage` is only three hand-maintained ID sets. The test unions them and
  compares to all live ids, so green proves membership bookkeeping, not behavior assertions.
- The category sets can be derived from live abnormality skill ids, equipment ability activations,
  and active skill fragment references; current test does not verify that category ownership.
- The first derived run exposed `unknown_distortion_strike` as live but unreferenced, although v1
  listed it as abnormality-owned. V2 records the derived complement explicitly instead of inventing
  ownership.
- The next derived check exposed that both A-rank original skills are also awakened fragment
  references. V2 records overlapping owners rather than forcing the old disjoint category fiction.
- No durable document says when/version/how the manifest is updated or prohibits using it as sole
  completion evidence.
- Live skill fragment RON has 30 abnormality-derived fragments plus one concept starter. The derived
  index lists only 22 and omits eight later trace fragments.

## Implementation and correction loop

- Bumped the strict manifest to v2 and renamed `behavior_coverage` to `reference_ownership`.
- The test derives abnormality, equipment activation, and active fragment references from loaded
  data, rejects duplicate manifest ids, checks spatial/Instant classification, and accounts for the
  entire catalog through real owners plus an explicit unreferenced set.
- First derived run corrected `unknown_distortion_strike` from falsely abnormality-owned to explicit
  unreferenced status. Second derived run added original A-rank skills as overlapping awakened
  fragment references; categories are intentionally not forced disjoint.
- Published `docs/audit/live_skill_catalog_evidence_contract.md`, linked it from the docs index and
  workflow matrix, and defined author/reviewer/version/behavior-evidence responsibilities.
- Updated the derived content index from 22 to all 30 abnormality-origin fragments, distinguishing
  the 22 with full placeholder E.G.O sets and the concept starter. Added a loaded-data assertion
  that the index's backtick fragment ids match exactly and contain no duplicates.

## Validation

- Focused manifest/index audit: 4/4. Focused skill-fragment RON load: 1/1.
- Broad Core all-targets: 751 passed (707 library + 3 + 4 + 20 + 13 + 4), 0 failed, 0 ignored.
- Broad game_server all-targets: 30 passed, 0 failed, 0 ignored.
- Formatting, repository diff check, stale overclaim/count, duplicate/ignored-test, and live count
  audits passed.
- No Unity code or contract changed. The current Phase 5 Unity broad baseline remains EditMode
  215/215 and PlayMode 52 passed with four explicit environment skips.
