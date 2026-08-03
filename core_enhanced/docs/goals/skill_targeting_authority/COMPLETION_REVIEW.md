# Skill Targeting Authority Completion Review

Goal: G10

Review date: 2026-07-14

Guide: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Item Audit

| Item | Integrated result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| Explicit skill selection | every skill cast reads its authored `SkillTarget` and `UnitTargetRule` | Complete | High | None |
| Missing selection failure | `EnemySingle.rule` is presence-required and all live skills are pinned | Complete | High | None |
| Basic-attack separation | weapon `targeting_profile` remains authoritative only for basic attacks | Complete | High | None |
| Compatibility cleanup | missing, legacy inheritance, unknown dual-authority, and stale fields reject | Complete | High | None |
| G09 boundary | reach, captured footprint, air eligibility, and delivery geometry remain unchanged | Complete | High | None |

## Expected Source Of Truth

`SkillDef.cast_targeting` is the sole skill cast-selection authority. Its explicit target kind and,
for `EnemySingle`, required `UnitTargetRule` drive runtime selection. Equipped weapon data owns only
`RuntimeUnit.basic_attack.targeting_profile`; it is never inherited by a skill.

## Schema And Content Evidence

- `SkillCastTargetingDef` accepts only `Explicit`; there is no loadable fallback or weapon-profile
  variant.
- `SkillTarget` denies unknown fields, and `EnemySingle.rule` no longer has a serde default. Missing
  selection and a simultaneous `targeting_profile` field fail before database publication.
- The live-bundle regression inventories all 25 skill ids and pins each exact authored selection.
  Existing live RON was already explicit, so the cutover required no inferred balance rewrite.

## Runtime Evidence

- Skill cast selection, preflight/usefulness, and step resolution consume the skill-owned target
  rule. Scoped searches found no read of basic-attack targeting profile on those paths.
- A conflicting-candidate test changes the same caster from LowDefenseFirst to DefaultForward while
  its LowestHealthEnemy skill selects the same weak, high-defense unit both times.
- The existing basic-attack profile regression still proves LowDefenseFirst weapon behavior, so the
  separation did not erase weapon authority.

## Cross-Layer Evidence

- game_server continues forwarding the unchanged typed data and passes its full suite.
- Unity contains no local skill-selection reconstruction or weapon fallback. Canonical Unity
  wording treats projected `cast_target.target_policy` as skill-owned metadata and does not use a
  weapon to fill it.
- G11's Player-only range-preview and opponent-field absence contract remains unchanged. G09 still
  owns acquisition reach versus delivery-footprint geometry.

## Validation Review

- Focused Core: strict missing/unknown authority 1/1, SkillDatabase negative schema group 1/1,
  weapon-independent skill runtime 1/1, live 25-skill inventory 1/1, basic-attack profile 1/1.
- Broad Core: 748 passed, 0 failed, 0 ignored. Broad server: 30 passed, 0 failed, 0 ignored.
- Unity EditMode: 212/212. PlayMode: 52 passed, 0 failed, four explicit environment-dependent
  live-server skips.
- Check, formatting, scoped Core diff, final Unity console, fallback/alias/stale-document, and
  relevant ignored-test audits passed.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: target selection has one strict skill-owned authority while basic attacks retain their
  independent weapon-owned policy; the split is explicit in schema, execution, content, and docs.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

None inside G10. G09 is the planned owner of Global reach and target-centered footprint semantics.

## Policy Questions

None.

## Review Conclusion

G10 is Complete / High with no immediate correction. G09 may begin.
