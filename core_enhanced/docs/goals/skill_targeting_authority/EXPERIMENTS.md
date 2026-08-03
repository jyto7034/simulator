# Skill Targeting Authority Experiments

Goal: G10

Updated: 2026-07-14

## Entry gate

- R04: Complete / High.
- G11: Complete / High; enemy preview is removed before targeting authority changes.
- Accepted PD-07 fixes explicit skill-owned selection and leaves G09 delivery geometry blocked.

## Dirty baseline

- Core status fingerprint: `afa21897a28d222c4e317ffbf46c8f7c8f43b8d532181928e3245f648647de32`.
- game_server status fingerprint: `67d79bc9d7fb88a02a329652e488d9a9c953864988f380a1469bf712183df0c9`.
- Unity status fingerprint: `cd6eeed715cde88a4f1e429accbff76ccf3a2d8a75d31f3c0c712ecb2b7db28a`.
- Core/server tracked diff fingerprint:
  `ffc2da8546f76fadf571917d93c2e03a4a3caf0fefdd99380dc55111136ec80e`;
  Unity tracked diff fingerprint:
  `43b9919aa4931f206ee198cae352c31c20eae16fcdebd330bbb3e01a536504ce`.
- Worktrees remain heavily dirty; preserve every unrelated and closed-goal hunk.

## Runtime and schema trace

- `SkillCastTargetingDef` has only the strict `Explicit` variant. Skill cast resolution and
  usefulness checks consume its `SkillTarget`/`UnitTargetRule` directly and do not read the unit's
  basic-attack `targeting_profile`.
- Basic attacks have a separate target selector whose only profile input is
  `attacker.basic_attack.targeting_profile`.
- The remaining implicit skill behavior was `#[serde(default)]` on `EnemySingle.rule`, which made
  an omitted selection silently mean `Nearest`.
- All 25 current live skill definitions already author `cast_targeting` and an explicit rule where
  applicable; no balance-data rewrite was required.

## Implementation

- Made `SkillTarget` reject unknown fields and made `EnemySingle.rule` presence-required.
- Extended strict raw-schema tests to reject missing cast targeting, missing enemy selection,
  removed weapon-inheritance variants, unknown competing authority, and legacy range fields.
- Added a runtime regression in which LowDefenseFirst and DefaultForward weapon profiles disagree
  with the skill's LowestHealthEnemy rule; both casts select the skill-authored target.
- Added a complete live inventory assertion for all 25 skill ids and their exact selection rule.
- Updated Core data-loading, rulebook, skill-target, and Unity catalog contracts. Public serialized
  DTO shape and current live RON values did not change.

## Validation

- Focused Core: missing/unknown authority 1/1; strict SkillDatabase schema group 1/1; weapon-
  independent skill runtime 1/1; live 25-skill inventory 1/1; preserved basic-attack weapon-profile
  selection 1/1.
- Core all-targets: 748 passed (705 library + 3 + 3 + 20 + 13 + 4), 0 failed, 0 ignored.
- game_server all-targets: 30 passed, 0 failed, 0 ignored.
- Unity compatibility: EditMode 212/212; PlayMode 52 passed, 0 failed, four explicit live-server
  environment skips. No Unity production code changed. Final console contained zero errors after
  clearing test-runner result noise.
- `cargo check --all-targets`, formatting, Core diff check, strict-schema, weapon-inheritance,
  stale-document, and ignored-test audits passed. Pre-existing unrelated Unity diff whitespace is
  outside G10's unchanged code surface.
