# R06 P2G-20 current-code review

- Review ID: `CRA-CAVE-P2G20-001`
- Input: `WR-CAVE-P2G10-001`
- Source axis: 2026-09-13 Workspace Actual at HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`; the checkout is dirty and every selected file is fixed by SHA-256 in `P2G_20_SOURCE_EVIDENCE_001.json`.
- Boundary: static/read-only review only. No production source, data, build, test, implementation, release, or deployment claim is made here.

## System boundary

The current public route is external pack snapshot → RON parse/merge → cross-catalog validation → immutable `ValidatedGameData` → `GameCore`/headless authoring trial. Skills retain authored `SkillEffectDef` values and reconstructed `EffectSource`; there is no ValueExpr compiler or compiled-expression catalog. Battle execution creates commands in `BattleCore::build_skill_step_commands`; delayed projectile/area paths carry `DamageSourceSnapshot`. The independent target-usefulness path reads the same literal fields but performs a separate damage calculation. Persistent employee/loadout inputs reach `BattleUnitDraft`; `stat_pipeline` composes growth, Permanent effects, and enhancement modifiers. Active-run checkpoints serialize inventory, roster, growth, content identity, and typed resources. Certified power recomputes an employee profile from persistent inputs and has no input fingerprint/cache seam.

## C01 — finite typed expression and arithmetic (`CAVE-E01`)

- Current Actual: `Damage`/`Heal` use `i32 amount`; no ValueExpr, typed unit, rounding declaration, compile graph, or typed expression fault exists. Existing stat/damage helpers commonly use saturating arithmetic, which is not suitable as expression semantics.
- Approved Target and unchanged acceptance: exactly `PER-CAVE-E01-DECIDED-001` and its three acceptance members.
- Delta: introduce a finite authored AST plus an immutable compiled form with explicit units, checked intermediates, declared lossy-division rounding, and deterministic typed diagnostics. Existing literal Damage/Heal must compile to a Constant in the same runtime path.
- Proposed implementation: a narrow `game::value_expr` module; do not create a general gameplay VM. Compile by effect site during `GameDataBase` final validation and publish only the compiled catalog. Runtime evaluation returns a typed error before command/state mutation.
- Validation seam: compiler unit matrix; exact overflow/divide-by-zero/rounding cases; same compiled input determinism; no external code evaluation.
- Finding `TD01` (`technical_decision`): retain literal effect variants as the backward-compatible authored spelling and add explicit expression-bearing Damage/Heal forms, but lower both to one compiled amount plan and one executor. This preserves policy meaning and avoids a second literal calculator.

## C02 — snapshot/live moment (`CAVE-E02`)

- Current Actual: delayed carriers preserve source attack and timing in `DamageSourceSnapshot`, but `build_skill_step_commands` overwrites `base_damage` from the literal on impact; immediate and delayed paths have no typed evaluation context or moment validation. Directional projectiles and areas may not know their eventual target at release.
- Approved Target and unchanged acceptance: exactly `PER-CAVE-E02-DECIDED-001` and its three acceptance members.
- Delta: compiled references declare `Snapshot` or `Live`; a release materialization owns immutable numeric inputs/value for delayed delivery, while a live evaluation reads the still-valid effect-consumption context. Missing context is a typed failure.
- Proposed implementation: add an evaluation carrier to released projectile/area state (or its source snapshot) containing the compiled plan identity and captured snapshot inputs. Evaluate target-dependent Snapshot only where the target is known at materialization; reject unsupported target-dependent Snapshot/delivery combinations during publication. Live never rewrites captured facts.
- Validation seam: release, mutate source stat/HP/resource, then impact; compare Snapshot unchanged and Live changed. Test missing caster/target/delivery context and no command emission on failure.
- Finding `TD02` (`technical_decision`): fail publication for a Snapshot reference whose required target is not knowable at the selected delayed materialization boundary. Silently deferring it to impact would change Snapshot into Live.

## C03 — finite aggregates (`CAVE-E03`)

- Current Actual: effects iterate target slices and use ad hoc folds, but there is no authored Count/Sum/Max, grouping declaration, stable member identity, aggregate cardinality admission, or empty-Max fault.
- Approved Target and unchanged acceptance: exactly `PER-CAVE-E03-DECIDED-001` and its three acceptance members.
- Delta: add finite aggregate sources and an explicit grouping enum. The R06 minimum sources are authored expression members and resolved runtime targets; unsupported tile/group sources remain rejected rather than stubbed.
- Proposed implementation: compile aggregate members to a bounded flat plan; stable-sort dynamic members by `UnitInstanceId`; charge node/member/work budgets before evaluation; Count/Sum empty returns typed zero and Max empty returns a typed error.
- Validation seam: Max/Sum examples, empty cases, target order permutation, mixed grouping/unit rejection, at-limit and over-limit no-partial-state cases.
- Finding `TD03` (`technical_decision`): support only actually reachable R06 group producers. `GroupByTile` remains an explicitly unsupported authored value until a later card provides a real producer; this does not expand R06 into R10 target/flow work.

## C04 — Attack/SpellPower and actual-copy weapon growth (`CAVE-E04`)

- Current Actual: `UnitStats` has Attack but no SpellPower. `WeaponCombatProfile` controls behavior/cadence but contributes neither Attack nor SpellPower. Growth contains only `KillStack` and adds Attack. Enhancement is saved per `OwnedEquipment`, yet `BattleEquipmentEnhancement` drops `instance_uuid`; `stat_pipeline` then sums every enhancement row matching a definition UUID for every matching binding, enabling same-definition copy multiplication. Certified and battle profile paths both use this pipeline.
- Approved Target and unchanged acceptance: exactly `PER-CAVE-E04-DECIDED-001` and its three acceptance members.
- Delta: add zero-default SpellPower, zero-default weapon Attack/SpellPower contributions, SpellPower modifier support, and actual-copy identity on the battle enhancement projection. Compose base → weapon contribution → existing GrowthStack → Permanent effects → that binding's enhancement modifiers exactly once. Preserve fragment rank/growth behavior.
- Proposed implementation: keep behavioral weapon application in `UnitCombatProfile::apply_weapon_profile`; make `stat_pipeline` the sole numeric weapon contribution authority. Match enhancement by `persistent_copy_uuid` when present and never sum definition-wide levels into each binding.
- Validation seam: final Attack 100 including weapon 40; remove each component independently; two copies of the same definition at different enhancement levels; SpellPower-only modification; battle and certified profile parity; checkpoint round-trip.
- Finding `TD04` (`technical_decision`): add the persistent copy UUID to `BattleEquipmentEnhancement` instead of inferring it later. The actual item-slot and inventory producer already has this identity, so preserving it closes the defect without changing ownership policy.

## C05 — HP/time/resource/result references (`CAVE-E05`)

- Current Actual: runtime units expose current/max HP and battle time. R05 provides typed resource definitions, owner selectors, scopes, leases, a read-only `ResourceStore::value`, and source-carried finite capabilities. R04 provides exact-attempt `ActionHandle`, `query_action`, and typed `DifferentAttempt`/`Unavailable`/`StaleOccurrence`/`NotSettled` failures with bounded fact summaries. No expression adapter joins these authorities.
- Approved Target and unchanged acceptance: exactly `PER-CAVE-E05-DECIDED-001` and its three acceptance members.
- Delta: typed ref enums must resolve source/target HP, elapsed/remaining time, resource balance, and exact current-action result fields from one evaluation context. Resource owner/lifetime resolution must share the R05 helper rather than duplicate permissive matching; evaluation never mutates the store. Result reads must use the exact handle carried by `RuntimeFactSource` and only fields already settled for the current/earlier effect.
- Proposed implementation: extract a shared read-only resource lease resolver from `apply_typed_resource_operations`; map `ResourceFaultCode` and `ActionQueryError` into stable expression fault codes. Whitelist fact fields and time domains; do not offer a generic reflection/path reference.
- Validation seam: owner/lifetime separation and unchanged revision; stale/different-attempt/not-settled/future result; current/max/missing HP; elapsed/remaining time and missing deadline.
- Finding `TD05` (`technical_decision`): R06 result refs bind to the exact current action capability supplied by the execution source and a whitelisted already-produced fact field. Arbitrary ordinal lookup or serialized handle reconstruction is prohibited, preserving R04 authority.

## C06 — attack-speed points versus interval (`CAVE-E06`)

- Current Actual: `UnitCombatProfile::basic_attack_speed_percent` is a positive dimensionless input and `scale_attack_interval_ms` performs ceil division. Final `UnitStats::attack_interval_ms` is the only cadence used by runtime scheduling. Existing fragment reduction directly edits interval and is separate legacy behavior. Spell/area/projectile timers are independent.
- Approved Target and unchanged acceptance: exactly `PER-CAVE-E06-DECIDED-001` and its three acceptance members.
- Delta: expose the existing positive dimensionless input as `AttackSpeedPoints` in the typed stat/reference descriptor while keeping the serialized compatibility field. Reject zero before calling the conversion and never treat milliseconds as compatible expression points.
- Proposed implementation: reuse `scale_attack_interval_ms` as the sole points→interval conversion; do not globally rescale delays or rewrite the distinct legacy interval-reduction modifier in R06.
- Validation seam: exact ceil/minimum behavior and zero rejection; demonstrate only basic attack cadence changes while focus/area/projectile timing is unchanged.
- Finding `TD06` (`technical_decision`): preserve the current serialized field and fragment interval-reduction semantics, while naming/typing the value at the new authoring boundary. A field rename or Percent unification would create unnecessary compatibility change.

## C07 — shared compiled plan, projections, save, identity and Certified seam (`CAVE-E07`)

- Current Actual: content identity hashes resolved authored definitions and lives in immutable `ValidatedGameData`; `GameDataBase` can own another immutable compiled catalog. Explain returns authored effects/source but no compiled amount. Target usefulness separately computes literal damage. Range preview is spatial only. Inventory enhancement and employee growth already serialize in the V7 latest-only checkpoint; transient battle/action facts do not. Certified power recomputes persistent profiles and returns version/total/members, with no input fingerprint or cache.
- Approved Target and unchanged acceptance: exactly `PER-CAVE-E07-DECIDED-001` and its three acceptance members.
- Delta: runtime, usefulness preview, and explain must fetch the same compiled plan by `EffectSite`; remove the independent literal amount fold. Expose per-expression semantic fingerprint/source trace. Add an R06 Certified model-input fingerprint that changes with effective Attack/SpellPower/weapon/enhancement inputs while leaving total weights unchanged. Keep transient evaluation/result contexts out of checkpoint serialization.
- Proposed implementation: compiled catalog is constructed and validated inside the same `ContentGeneration` database publication and cannot be replaced independently. Explain includes authored and compiled plan/fingerprint. A shared observational evaluator serves target usefulness without mutation. Existing checkpoint inventory/growth path gets regression coverage and zero-default fields use compatible serde behavior.
- Validation seam: runtime/preview/explain parity; two content generations with changed expression produce different content/expression fingerprints; enhancement/growth round-trip; transient handle absent after restore; Certified input fingerprint changes but model version/weights do not.
- Finding `TD07` (`technical_decision`): the range-preview DTO remains spatial; quantitative effect preview belongs to the existing authoring explain/target-usefulness path. Falsely placing amounts into spatial cells would duplicate authority. AT06's range-preview obligation is satisfied by confirming the spatial consumer remains unaffected and by testing profile-derived range plus quantitative explain parity.

## C08 — public parser/compiler/executor and finite failure (`CAVE-E08`)

- Current Actual: `author_content validate|describe|explain|simulate` uses the production pack loader and immutable token. Manifest supported capabilities stop at `typed_resources_v1`. Control limits bound acquisition, validation, clone, output, and runtime work. There is no expression capability, descriptor, error taxonomy, AST budget, or expression runtime consumer.
- Approved Target and unchanged acceptance: exactly `PER-CAVE-E08-DECIDED-001` and its three acceptance members.
- Delta: add the expression schema to RON, semantic compiler, compiled catalog, actual Damage/Heal consumer, describe/explain output, `typed_value_expr_v1` capability, and public positive/negative pack fixtures. Pre-admit depth/node/member/retained/work costs; failure must propagate through the transaction instead of current `let Ok(..) else { return; }` swallow sites.
- Proposed implementation: typed `ValueExprFault` maps into `DataValidationError`/`LiveRonError` during publication and `GameError` at runtime. Refactor command construction/delivery callbacks enough to preserve transaction failure; do not register unimplemented references/groups.
- Validation seam: public CLI validate/describe/explain/simulate against external RON; unknown variant/reference/unit, depth/node/cardinality, overflow and divide-by-zero negatives; exact test counts and nonzero exits.
- Finding `TD08` (`technical_decision`): advertise the new capability only after the public external-pack path and runtime test pass. Safety ceilings remain execution-control metadata, not gameplay balance data.

## C09 — compatibility, acceptance suite and audit boundary (`CAVE-E09`)

- Current Actual: official and fixture skill RON use literal Damage/Heal amounts. Existing stat, growth, enhancement, checkpoint and content-pack tests cover their current shapes, but no T06 expression tests or R06-specific T18/T27 copy/cache tests exist. Released/deployed/server/Unity compatibility is outside this request.
- Approved Target and unchanged acceptance: exactly `PER-CAVE-E09-DECIDED-001` and its three acceptance members.
- Delta: preserve exact literal execution and round-trip; add direct unit, integration, external pack, checkpoint, copy-isolation and deterministic fault tests; run scoped and full core validation followed by a fresh GEX-20 audit. Keep R07/R19 and unresolved Replace/official weights explicitly incomplete.
- Proposed implementation: use existing official RON unchanged as the legacy oracle plus a dedicated expression fixture for new coverage. Record commands, source revision, executed/nonzero counts, fingerprints, and audit findings.
- Validation seam: T06 plus applicable T18/T27 matrix; `cargo fmt --check`, focused tests, all targets, feature checks and public CLI probes as selected by the later Goal/Handoff.
- Finding `TD09` (`technical_decision`): compatibility is demonstrated by unchanged legacy packs and stable results, not by rewriting all literals to the new syntax.

## Cross-cutting findings and review disposition

| ID | Classification | Finding | Disposition |
| --- | --- | --- | --- |
| F01 | `technical_decision` | The current enhancement projection loses copy identity and definition-wide summation can multiply levels. | Preserve `instance_uuid` in the projection and bind one enhancement row to one equipment binding in C04. |
| F02 | `technical_decision` | Delayed carriers know different context at release; target-dependent Snapshot cannot always be captured. | Validate moment/delivery compatibility and fail unsupported Snapshot combinations in C02. |
| F03 | `technical_decision` | Target usefulness is a second literal evaluator. | Replace its amount fold with the shared observational compiled evaluator in C07. |
| F04 | `technical_decision` | Runtime helper callbacks currently swallow some command/evaluation errors. | Carry typed failures through the existing candidate transaction so no partial state is committed in C08. |
| F05 | `out_of_scope` | Dynamic tile grouping, R07 fragment copy/awakening, R10 flow binding, and R19 final Certified weights need later owners. | Keep unsupported/unpublished; do not stub or claim complete. |

There is no new `material_policy`, no blocking `evidence_gap`, and no applicable P2G-20 repair obligation. All nine exact P2G-10 policy statements and all 27 Observable Policy Acceptance members remain byte-for-byte policy values in the reviewed revisions. Released Actual and Deployed/server/Unity/external service are explicit `N/A` under scope item O01; no compatibility claim is made for them.

## Implementation handoff shape

The smallest valid implementation is multi-phase but one AT06 publication boundary:

1. typed AST/compiler/catalog/faults and literal lowering;
2. shared evaluation contexts, moment carriers, resource/result adapters, aggregates, and Damage/Heal runtime/usefulness/explain integration;
3. SpellPower and numeric weapon contribution with actual-copy enhancement binding, compatible save projection, and Certified input fingerprint;
4. external RON capability/fixtures, deterministic limits, full acceptance evidence, and fresh audit.

Intermediate code may compile during execution, but R06 cannot be accepted until all four are integrated and the public pack/runtime/stat/save/projection seams agree. This review is not that implementation result.
