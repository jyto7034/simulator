# Live Skill Catalog Evidence Contract

`docs/audit/live_skill_catalog_manifest.ron` is a versioned audit index derived from strict live RON.
It is not gameplay authority and it is not a behavior-test coverage certificate.

## What the manifest proves

`tests/live_skill_catalog_audit.rs` loads the published gameplay bundle and checks:

- every live `SkillDef` is classified as spatial or intentional Instant-only from its actual steps;
- abnormality, equipment activation, and active skill fragment owner sets exactly match actual live
  references;
- the derived complement is recorded explicitly as `unreferenced_skills`, and owners plus that set
  equal the live skill catalog;
- listed ids are unique because the audit converts each strict manifest list to a set and rejects
  any length loss; the manifest parser also rejects unknown fields.

The same integration test also derives all abnormality-origin fragment ids and compares them with
the backtick ids in `docs/skills/live_content_index.md`, preventing its discoverability list from
silently dropping or inventing fragments.

A green manifest test therefore proves exact membership, delivery classification, and reference
ownership/unreferenced status at that revision. The current unreferenced set contains
`unknown_distortion_strike`; the manifest does not invent an owner for it.

## What the manifest does not prove

The manifest does not prove that a skill's damage, target selection, repeats, conditions, area
geometry, buffs, presentation, or timing are correct. Listing a skill id is not behavior evidence.
Smoke tests that assert an ability cast or broad event category prove only those asserted facts.

Behavior evidence must name an executable test and the specific behavior it asserts. Examples of
strong evidence are the board scenarios under `tests/skill_test/`, exact runtime contracts in
`tests/skill_runtime_contract.rs`, and focused Core unit tests for shared targeting/delivery rules.
Completion reviews must cite those tests directly and must not cite manifest membership as complete
behavior coverage.

## Ownership and update lifecycle

The author of a live skill, delivery, abnormality reference, equipment activation, or active skill
fragment change owns the corresponding manifest update in the same change. The author must:

1. change live RON and its focused behavior/validation tests;
2. run strict live loading;
3. update manifest membership/classification only to match the loaded result;
4. update `docs/skills/live_content_index.md` when fragment/equipment discoverability changes;
5. run `cargo test --test live_skill_catalog_audit` and the named behavior tests;
6. record the named behavior assertions separately in the goal/completion evidence.

Reviewers verify that the live-data/reference change explains every manifest diff and that behavior
claims cite executable assertions. Editing the manifest merely to make the membership gate green is
not sufficient evidence.

Manifest `version` changes only when this file's machine-checked field meaning/schema changes, not
when ordinary catalog membership changes. `tests/live_skill_catalog_audit.rs` owns the strict parser
and expected version; schema revisions update both atomically without compatibility aliases.
