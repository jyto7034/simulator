# R01 implementation evidence for fresh GEX-20

Target: `(GS-CAMEC-P2G50-001, GOAL-CAMEC-01)`, `PHASE-CAMEC-01`, Handoff `HOF-CAMEC-P2G60-001`. This is the implementation owner's evidence map, not an audit verdict. Policy/acceptance wording remains in the immutable upstream Goal.

All commands use `/mnt/f/work/simulator`, default features and the recorded Rust/Cargo 1.91.1 toolchain. Final implementation/test inputs are [VALIDATION_INPUT_002.json](VALIDATION_INPUT_002.json). The full test run [core-full-003](evidence/camec-r01-core-full-003.log) reports library **1,114** plus integration suites **4+4+25+15+4 = 52**, total **1,166 passed**, zero failed/ignored/measured/filtered. New tests are included in those counts, not additional passes to sum again. This covers default core targets; optional event-log-check features, external server/Unity, deployment and performance SLAs are not claimed.

## Acceptance members

| Member | Implementation and direct evidence | IVA |
| --- | --- | --- |
| E01[0] | `AuthoredSkillEffect.op` feeds `sim::build_skill_step_commands` and the original command processor. `authored_literal_ron_executes_recomposed_definitions_with_exact_effect_sources` loads two RON definitions and runs each under three scalar inputs: (17,9), (21,9), (0,-5); asserts actual ordered damage/self-HP results and source. No content ID dispatch in the production delta. | 01 |
| E01[1] | `SkillDatabase::from_ron` and strict typed opcode enum reject inert `ExecuteFile`; `effect_support`/`set_effect_amount` reject unsupported op/delivery edits. `invalid_authoring_has_source_object_action_effect_and_field` and full publication negative test prove failures. | 01/02 |
| E02[0] | `EFFECT_SUPPORT`, `EffectProperty`/`PropertyType`, `EffectSource` and `LiveRonError` expose typed support and actual source/object/action/effect/field. Both raw malformed-op and semantic-negative/dangling-ref input are tested; missing IDs report only available identities. | 02/05 |
| E02[1] | `from_ron` is the same skill parser used by production `parse_live_skill_database`; the new RON battle fixture uses real `BattleCore`. Full embedded token publication and generated-preview validation are exercised by live loader tests. | 01 |
| E03[0] | `SameEffect` AND versus `EachExists` uses distinct red/blue effect predicates; scalar edit preserves a different damage and SelfUnit Heal. Carrier request returns all affected sites without mutation. Actual runtime six cases observe independent targets and values. | 03 |
| E03[1] | `matches_effects` first resolves exact SkillId; shared tags never identify `absent_form`. Two actual distinct definition IDs and the negative missing definition are tested. | 03 |
| E03[2] | Tags are consulted only by the typed metadata predicate, not op validation/dispatch. Script/unsupported-property fixtures fail. Effect carrier edit is rejected with action/source and two affected sites. | 02 |
| E04[0] | Original and Added source attribution are compiled per effect. The actual beta Heal observes Added(source alpha), separate own site, caster/cast/AbilityCast origin. `executed_proc_effect_retains_item_cast_and_authored_origin` observes real activation binding/item/proc event and verifies temporary context cleanup. | 04 |
| E04[1] | Same `recovery` predicate selects Added Heal; attribution differs from definition selection. This is authored metadata, not a runtime relic-grant system. | 03/04 |
| E05[0] | Bundle schema 2, latest wrapped RON and 25 changed/new Rust files cut over together. Schema 1 and bare legacy effect grammar reject. Existing live catalog, item activation, range/fragment and all core consumers pass in the full run. | 05 |
| E05[1] | No personal store was read or changed. `world/state.rs` and `world/active_run.rs` are byte-identical to the pre-edit snapshot. Full core run includes V5 envelope/checksum, V4 continuation→V5, capture/restore and public application checkpoint tests. New definition sources/runtime origins do not appear in those stored types. | 05 |
| E06[0] | Serde typed operation/property decoding and existing match dispatch remain the only authority. Inert unsupported op and unknown carrier field reject before publication. No OS/file/network eval entry or content-specific branch was introduced. | 02 |

PER prefix for these locators is `PER-CAMEC-Exx-REVIEWED-001`; all twelve are present in the original Goal. Every IVA is owned by this Goal's WP-CAMEC-03; IVA06 is the preservation evidence shared across these rows.

## Required dimensions and structural preservation

| Dimension | Direct evidence |
| --- | --- |
| success | Six actual RON battle cases; Damage, zero Damage and signed Heal; source/origin and independent self/enemy targets. |
| invalid_input | Negative Damage, duplicate effect ID, missing effect ID, overflow i32, dangling Added definition/buff, unsupported operation/property and old grammar; typed diagnostics; existing duplicate action/definition and strict-loader suite. |
| failure_rollback | `effect_authoring_failures_never_publish_or_change_an_existing_token`: early invalid op/scalar and late missing buff deny token, preserve previous publication; shared-carrier/scalar rejection leaves candidate unchanged. |
| retry_idempotency | Repeated valid/invalid publication and invalid-then-valid retry preserve contents; metadata reorder test preserves logical addresses; full cast cleanup/late-delivery tests. |
| ordering_concurrency | Actual ordered Damage/Damage/Self Heal; ModifyDamage folding unchanged; existing HP-loss gate Pending/terminal, multi-hit, kill-stop and late-step tests all pass. Per-command source envelope delegates to the same executor and preserves saturating summary aggregation. |
| boundary_empty_max | Empty/missing/duplicate address, empty-predicate disposition, Damage 0/i32::MAX/negative, Heal i32::MIN candidate and negative live execution, raw i32 overflow rejection. Existing damage/HP clamping remains covered by full core. |
| disconnect_restart_persistence | Existing isolated checkpoint/capture/restore/V4/V5 tests in full suite; relevant store source hashes unchanged. No network reconnect claim. |
| live_data | Full production embedded manifest/catalog and generated-preview validation, late candidate failures, latest-only manifest/grammar negatives. |
| producer_consumer | Typed support→raw compiler→actual common executor, SourceMap through direct and delayed deliveries, real proc binding, live item/catalog and preview/fragment tests. |
| legacy_fallback_absence | Scoped production diff and strict negative tests: no schema-1 parser alias, bare effect fallback, duplicate executor, arbitrary script/property evaluation or fixture-name production branch. Other existing operations/deliveries and V4/V5 readers intentionally remain in their previous scope. |

`withdraw_preserves_released_skill_projectile_with_launch_snapshot` now removes the active cast and clears previous log entries before impact, then observes the retained authored source. `withdraw_preserves_ground_fixed_area_and_ends_follow_areas` checks fixed-area provenance after withdrawal and preserves follow-area cancellation. `withdraw_cancels_queued_unreleased_skill_step` and existing post-death projectile tests are also in the final full run. Cast/proc origin is captured in non-serialized Arc snapshots; it is not inferred from evictable log ancestors.

Metadata uses the existing bounded event-entry container and shared definition/cast records. It adds no independently growing trace collection. The serialized byte budget excludes skipped metadata; the entry-count/lifetime bounds and immutable shared catalog bound retention. Full core exercises determinism/roll identity and history bounds. This is not a new numeric performance or memory SLA claim.

## Failures, correction and limits

Intermediate compile runs found direct typed fixture consumers and corrected their wrapper/source-envelope arguments. Initial skill-data run found an old raw-deserializer error-text expectation; the public typed loader keeps enriched errors while the pre-existing direct serde cause text remains intact. First full run: **1,113 pass / 1 fail** because the `ModifyStabilization` negative fixture still used the old outer grammar. It now uses schema-2 wrapper with the same rejected opcode; the assertion was retained, and separate tests reject old outer grammar. No test was deleted/ignored/weakened. Intermediate logs are preserved in `evidence/`; final full run is the current evidence.

No external consumer contains direct constructions of the changed SkillEffect/SkillStep/DamageSourceSnapshot/BattleEventLogEntry types in the inspected `game_server` Rust paths. No external files were modified or integration success claimed. Existing CRAR audit is historical intake infrastructure evidence; changed schema/source contracts are validated here, not described as unchanged CRAR current implementation. This Goal has no formal predecessor Goal dependency.

New source attribution is core-only and omitted from current wire serialization; checkpoint state remains unchanged. Future full Flow, relic activation/grants/copies/generation and arbitrary derived-event provenance remain outside this minimum Goal. R02 latest-only generation/save transition is a later owner and does not retroactively remove R01's preserved V4/V5 contract.
