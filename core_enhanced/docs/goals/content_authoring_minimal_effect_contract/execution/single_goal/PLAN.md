# R01 / GOAL-CAMEC-01 execution

## R02 correction003 fresh audit routing — 2026-09-10

R01 successor audit002 closed Complete / High, and the first R02 fresh audit independently accepted its
comment/report-only currentness at ATR-CAPG-R02-002. R02 then closed Not Complete on R02-AUDIT-F01.
The same authorized R02 owner corrected generated source diagnostics; current source/test comparison and
actual 1,217-test evidence are in [impact003](../../../content_authoring_pack_generation/execution/single_goal/R01_SUCCESSOR_IMPACT_003.md).
The next separate fresh R01 GEX-20 roles are AUDIT_TARGET_003.json, COMPLETION_AUDIT_003.md and
audit-evidence/successor_003/. Source is frozen before review. No implementation is assigned to that auditor;
no policy/Freeze/Goal/Handoff is changed and no positive verdict is prewritten. Prior routing and audits below
retain their historical targets. This R01 review restores prerequisite currentness before a distinct R02 audit.

## R02 successor re-audit routing — 2026-09-10

The original `COMPLETION_AUDIT.md` / ATR-CAMEC-R01-001 remains immutable historical evidence. R02's authorized
successor changes external publication, owned source and latest storage, making its affected currentness stale.
Under the same adopted AUTORUN, the successor fresh GEX-20 uses `AUDIT_TARGET_002.json`,
`COMPLETION_AUDIT_002.md` and `audit-evidence/successor_002/` in this directory as its exact audit roles.
The reviewer determines the verdict; this routing note neither restores currentness nor prewrites completion.
Implementation is owned by `(GS-CAPG-P2G50-001, GOAL-CAPG-01)` GEX-10 pass 001, with current validation and
the [successor impact map](../../../content_authoring_pack_generation/execution/single_goal/R01_SUCCESSOR_IMPACT_001.md).
The historical execution sections below remain unchanged. No R01 policy/Freeze/Goal/Handoff is revised here.

## Current pass and authority

2026-09-09. GEX-00 intake completed; GEX-10 implementation/validation closed against AUDIT_TARGET_001.json; next pass fresh GEX-20. No production mutation during GEX-00. Implementation writer: root. Bounded read-only source/roadmap assistance uses separate agents; completion review will use a fresh reviewer after freezing the implementation.

The current user adopted `core_enhanced/docs/roadmaps/content_authoring/AUTORUN_REQUEST.ko.md` for continuous R00–R24 work. This child is `goal-run` on `(GS-CAMEC-P2G50-001, GOAL-CAMEC-01)` only, not a roadmap Master. The parent selects subsequent eligible work after this child's terminal result. Applicable pipeline: `core_enhanced/docs/goal_execution_pipeline(1).md`, `2026-09-09-gex-r1`; P2G reader: `policy_to_goal_pipeline_revised_v2.md`, `2026-09-09-r2`. No applicable AGENTS.md was found in workspace or ancestor search. No pipeline, validator, global configuration or permission change is authorized.

## Exact upstream and intake

- Working revision `WR-CAMEC-P2G20-001`, Review `CRA-CAMEC-P2G20-001`, Freeze `FPF-CAMEC-P2G30-001`.
- Phase Plan `PP-CAMEC-P2G40-002`, passed Receipt `PVR-CAMEC-P2G40-002`; single `PHASE-CAMEC-01` owns `AG-CAMEC-01` (WP01+WP02) and `WP-CAMEC-03`.
- Current Handoff `HOF-CAMEC-P2G60-001`; all six source/context hashes match in [BASELINE_001.json](BASELINE_001.json). Existing artifacts retain their historical Authored / Not Activated / Implementation Authorized: No statements. This user's subsequent authorization is recorded here, without rewriting historical artifacts.
- The six reviewed policies, twelve acceptance members, IVA-CAMEC-01–06 and all ten required validation dimensions remain exactly as [the Goal](../../P2G_50_GOAL_SET_001.ko.md) defines them.
- No predecessor Goal dependency. Existing publication/SkillId/executor infrastructure is present. Relevant production/data/test/build inputs are unchanged against `WORKSPACE_BASELINE_001.json` (1,668 prior comparison paths); historical CRAR Master Attempt 3 is intake evidence, not a fresh test result or a prerequisite requiring the entire old project to be reimplemented.
- No previous execution directory/history existed for this Goal. Roles are this PLAN, [EXPERIMENTS.md](EXPERIMENTS.md), [EXPERIMENT_NOTES.md](EXPERIMENT_NOTES.md), and future `COMPLETION_AUDIT.md` in this directory. No positive audit is prewritten.

## Baseline, scope and preservation

Workspace `/mnt/f/work/simulator`; branch `cleanup/simulator-dirty-worktree-20260803`; HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`. [BASELINE_GIT_STATUS.txt](BASELINE_GIT_STATUS.txt) records existing modified/deleted/untracked work. [BASELINE_001.json](BASELINE_001.json) identifies scoped bytes; `/tmp/camec-r01-baseline` preserves pre-edit copies for task diffing. No reset/clean/stash, personal-save operation, commit, deployment or external consumer change.

Allowed: effect authoring/compiled metadata, typed query/support/diagnostics, existing skill command/source consumers, latest embedded RON and direct fixtures, core contracts and these execution records. Preserve literal Damage/Heal, shared target/delivery, ModifyDamage folding, HP-loss gate, signed Heal, released projectile/fixed area and cast cancellation. New generation/copy/save state, full Flow/relic activation, gameplay/balance/UX choices and server/Unity implementation are outside this child.

## GEX-10 detailed design / before mutation

`SkillStepDef.effects` becomes `Vec<AuthoredSkillEffect>` wrapping the existing typed `SkillEffectDef` op. Required authored `id` is unique within its action; tags are plain metadata; `origin` distinguishes Original from an authored additional effect's source definition. `EffectSite` combines SkillId/action/effect, independently of vector order. Compiled source records add logical file and actual raw field locator, with compiler-owned original-definition attribution. No arbitrary property evaluator or runtime content-name dispatch is introduced.

The same compiled wrapper is consumed by skill validation, target usefulness, live tactical signals and the common `sim::build_skill_step_commands` used by Instant/projectile/area. Damage/Heal descriptors and scalar edit/query operations match the same typed op, with signed Heal and nonnegative Damage. Same-effect all-tags/property is separate from existential predicates over distinct effects; exact definition selectors never alias tags. Carrier edits are rejected with the action and affected effect addresses. Public support is limited to validated literal Damage/Heal with Instant; other existing op/delivery execution is preserved without advertising their future editing contracts.

Raw parser, latest embedded bundle schema, RON and all direct Rust/RON fixtures cut over together. Old unwrapped authoring is rejected. Existing range-resolution errors become typed data errors before live publication. Parse failures retain field paths and recover available skill/action/effect identities from a location-only parse; no invalid token is produced. `SkillDatabase` builds source addresses at construction/compile, before publication; failed candidates cannot replace a valid database or gameplay state.

Runtime source travels alongside commands into the existing executor and bounded event entries as core-only metadata (excluded from wire serialization), linked to actual caster/cast/ability/proc cause. No separate unbounded event store. Released payload origin must be captured before ActiveSkillCast removal; looking it up only at impact would lose provenance after withdrawal/death. Existing event sequencing, HP aggregation and delayed cancellation stay unchanged. Exact runtime placement is checked against existing captured projectile/area fields before editing.

Storage remains SkillId/gameplay-only: authored definition/source indices do not enter checkpoint or ActiveRun payload. Direct fixture constructors assign explicit stable IDs; there is no production fallback generating IDs for old authoring. Baseline focused command `cargo test -p game_core --lib game::data::skill_data -- --nocapture` completed with 13 passed, 0 failed/ignored; 1,095 filtered out, Rust/Cargo as recorded. This is baseline evidence, not evidence for the new contract.

No inherited mandatory initial/final run-count directive applies. Record actual toolchain, commands, input hashes, exit counts and failures; retain old evidence as historical only. Focused seams: skill_data/live_ron, skill_runtime_contract, live_skill_catalog_audit, live_item_skill_activation, fragment/preview, world checkpoint. Shared schema/fixture changes require final core all-target compilation and core library/integration suite. New end-to-end fixtures must cover all twelve acceptance members/IVA01–06 and ten dimensions, including invalid publication, stable reordered addresses, same-effect AND versus distinct existential queries, specific ID versus tags, independent scalar mutation/shared carrier rejection, runtime/proc source, late events and persistence preservation.

GEX-10 completion requires actual implementation and direct evidence for every applicable Goal condition, current input-to-seam coverage, canonical docs and an immutable audit target. Then stop implementation and perform fresh read-only GEX-20. Findings return to a separate correction pass. Only `Complete / High` closes this child; R02–R24 and R00 intake obligations remain parent work.

Material policy, missing required consumer, unsafe concurrent writer, unavailable mandatory reviewer or actual resource/permission limits are stop conditions. Ordinary technical errors are correction work. No unsupported feature is represented by empty execution or a successful fallback.

## GEX-10 exit / 2026-09-09

AG-CAMEC-01 and WP-CAMEC-03 are implemented with direct evidence for 12 acceptance members and 10 required dimensions in ACCEPTANCE_EVIDENCE_001.md. Final core-full-003: 1,166 passed, no failures/ignored/filtered tests; all-target and optimized lib compilation passed on unchanged production bytes. Scoped format and baseline preservation checks passed. No material policy/required consumer gap identified by this owner. AUDIT_TARGET_001.json freezes source, data, tests, canonical contracts, exact upstream and evidence. Implementation mutation has stopped. These are implementation exit claims for independent GEX-20 assessment, not a completion verdict. No formal predecessor Goal or successor audit repair is required for this first Single Goal.
