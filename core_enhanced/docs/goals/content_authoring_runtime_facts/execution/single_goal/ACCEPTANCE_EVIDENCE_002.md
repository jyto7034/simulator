# R04 implementation evidence — stable INPUT008

Owner `(GS-CARF-P2G50-002, GOAL-CARF-01)`, PHASE-CARF-01, HOF-CARF-P2G60-002; assigned AG-CARF-01 and WP-CARF-03. This is GEX10's implementation evidence map, not an independent completion verdict. Authority remains the exact [Goal](../../P2G_50_GOAL_SET_002.ko.md), Freeze003 and Plan004. All20 acceptance members,10 IVA,10 required dimensions and I01–06 are retained.


Correction002 supersedes evidence map001 without rewriting it. The initial [fresh audit](COMPLETION_AUDIT.md) closed Not Complete with F01/F02 and nonblocking O01. [Correction plan](CORRECTION_002.md), [narrow attribution](evidence/CORRECTION_002_ATTRIBUTION.json) and [full attribution](evidence/ATTRIBUTION_002.json) bind the repair. F01's new oracle failed on the old implementation (red204); current focused206/208 and full209 pass. The initial new-test ordinal/geometry mistakes and their corrections remain in EXP018/raw205/207. These passing executions support implementation closure, not a new audit verdict.

## Exact current validation

[INPUT008](evidence/IMPLEMENTATION_INPUT_008.json) identifies230 source/consumer/contract inputs stable after correction002: three source/test files changed from INPUT007,227 other identified inputs unchanged. The final audit target expands the surface to all applicable source/data/config and exact external outputs. Every `r04_<name>NNN.json` records the actual command, exit, raw log and digest. Old failed/intermediate commands remain historical in EXPERIMENTS and do not count as current PASS.

- [full209](evidence/r04_full209.json): `cargo test --locked --offline -p game_core`,1,363 PASS =1,311 library+52 integration tests,0 ignored/filtered. The five integration binaries run4/4/25/15/4 tests as recorded in the raw log;0 doctests are N/A and not gameplay tests. Existing assertions were retained; the original grouping/area assertions remain intact and two new boundary regressions now run in the full suite. Earlier packed-event consumer adaptations retain their historical attribution.
- [all-target211](evidence/r04_alltargets211.json), [default-release212](evidence/r04_release212.json), [consumer examples213](evidence/r04_examples213.json) pass. Default production builds use no fixture feature; standalone allocation-observer instrumentation is explicitly separate.
- [ordinary guards004](evidence/ORDINARY_GUARDS_004/RESULT.json), [guards210](evidence/r04_guards210.json): original public plain/allocator and kernel plain/allocator pass all unchanged caps, deterministic parity and repeatability. Kernel plain p95 event12.468494ms/25ms and concurrent7.453259ms/10ms. Allocator event41,660 operations,255,622 maximum peak; concurrent367,088 operations,9,007,676 maximum peak against9,030,228 original cap. Original final_performance code, fixture, oracle and baseline are unmodified. The concurrent peak margin is small and is not generalized to other workloads.
- [native budget210](evidence/ORDINARY_GUARDS_004/native_budget.json): original five-second fixture,67 binary-search cases plus default and6 exact/one-less reruns. Minima clone843,279/work2,600,736/retained6,090,008; exact minima retain identical complete native facts and log hash, one less gives the selected typed fault. Original default clone1,048,576/work10,000,000/retained8MiB suffice. This measurement creates no gameplay or balance threshold.
- [cost003](evidence/RUNTIME_COST_003/RESULT.json):685 plain samples/5 repeats and411 instrumented/3 repeats after one warmup each;126 fact queries allocate zero times;12 measured load/drop pairs reclaim each of four coexisting generations'1,658,777 retained bytes. Sampled fact high water:25 accepted/24 settled,1 active,1 delivery,4 actors,2 providers,39,892 retained-byte bound. Full phase-level timing/peak/retention/counts and exact binaries are retained. Requested allocator bytes are not RSS and step-sampled high water is not instruction-level high water.

Current direct external consumers, all with isolated durable outputs:

| Evidence / commands | Actual result |
| --- | --- |
| PREDECESSOR_PACK_003 /214 | Original packaging/consume commands PASS; custom files preserved; actual initial currency500→733→500. |
| PREDECESSOR_CLI_003 /215 |21 commands,13 CLI cases,6 forbidden consumer compile cases; finite preparation/receipt/trace contracts PASS. |
| PREDECESSOR_API_003 /216 |11 original public consumer checks PASS, including expected privacy/fixture-feature failures. |
| PREDECESSOR_DIAGNOSTIC_003 /217 | Valid and restored policy/boss fixtures identical;6 negative calls preserve exact source file/field attribution, no output/token, restored input bytes. |
| PREDECESSOR_SOURCE_003 /218 |36 failed replacements,3 successful replacements,3 caller-edit proofs; exact byte-source receipt preserved independently of semantic identity, off/on/gap equal per source. |
| RUNTIME_FACT_EXTERNAL_003 /219 |4 API replays,2 CLI simulations,2 queries;91 damage mutations/strikes,277 HP lost,2 deaths,1 actual tactical payment spending10; mandatory output equal off/on/gap/repeat. |
| RUNTIME_FACT_SOURCE_003 /220 | Trace-off native source is authoritative; actual delayed projectile/periodic area source compared with captured trace only as oracle. |
| RUNTIME_APPLICATION_003.json /221 |13 ordinary CoreApplication steps, no TrialSession observation owner; exact terminal native facts match Trial and result consumption releases current world fact owner. |
| PREDECESSOR_COMBAT_003 /225 |12 commands,6 API/4 CLI runs,2 explanations; original A18/B36 damage and7,950ms application preserved. |
| PREDECESSOR_RESOURCE_003 /226 |10 commands,6 API/2 CLI runs,2 explanations; actual resource mutation and source parity preserved. |
| PREDECESSOR_BUFF_003 /227 |10 commands,6 API/2 CLI runs,2 explanations; same resource fixture pack hashes and buff observations across trace modes. |
| PREDECESSOR_INVALID_003 /228 |13 invalid cases,26 API attempts and26 CLI attempts;29 top-level commands, typed deterministic rejection. |
| PREDECESSOR_DELIVERY_003 /229 |9 commands,4 API/2 CLI runs,3 explanations; delayed repeat/projectile/area and terminal19,950ms preserved. |
| PREDECESSOR_PREVIEW_003 /230 |4 commands,4 API replays,1 discovery,9 positive previews; actual public prepare/advance/query boundary. |
| PREDECESSOR_REPLACEMENT_003 /231 | Original replacement consumer checks failed/successful A/B swaps, continued A state and bounded rejection; exact receipt in output directory. |

## Acceptance and integration mapping

Abbreviated `E01[0]` below means exact `PER-CARF-E01-REVIEWED-001[0]`, similarly E02–05. Every named unit test appears as PASS in full209; its source defines the concrete oracle. Paths are under `src/game/battle/core/` unless stated. IVA rows are the exact `IVA-CARF-NN` obligations and do not define new acceptance.

| Acceptance / IVA | Actual producer→consumer and direct oracle |
| --- | --- |
| E01[0] /01 | damage_runtime/basic_attack/skill_runtime preserve old formula, basic-only trigger, resonance and HP-loss gate; original full209 skill_runtime_contract15, all control/gate tests and original kernel parity. Combat225 A18/B36 verifies the literal production path. |
| E01[1] /01 | commands `runtime_facts_lethal_damage_retains_combined_hp_transition` requested2000/actual1000 and shared lethal fact; `runtime_facts_keep_overheal_zero_signed_heal_and_predeath_sources_separate` requested healing/zero/negative versus actual delta,0 Damage/Cost reclassification. |
| E01[2] /02 | sim CastEnd/manual request and world actual tactical payments feed Cost; `native_action_pending_death_preserves_cast_end_cost_without_release`, result-admission rejection, existing auto/manual/tactical tests; `runtime_facts_resource_modifiers_do_not_become_payments` requested−100/actual−30,0 Cost. External219/221 actual deploy20→10. |
| E01[3] /03 | death_runtime prepares predeath/fact before cleanup and keeps mandatory dead/wave processing separate from optional dispatch. Existing lethal/optional suppression/death/wave tests, stat modifier8-case table in commands, direct summon Kill versus Despawn tests all run full209. |
| E02[0] /04 | damage_runtime admits a checked `AdmittedStrike` after target qualification and before current-hit planning/formula, sets the transient source, and passes that same value to post-calculation fact preparation. `runtime_strike_admission_precedes_formula_limits_and_rolls_back` proves identity-first fault ordering, full rollback and a two-mutation same-strike retry at the last admissible ordinal for both committed policies. Existing commands `runtime_facts_group_effect_major_damage_by_actual_iteration_and_target`:2 effect mutations per direct strike, two distinct iterations,40 HP per strike; old actual-damage gate count remains2. New strike scope plus per-segment ordinal is not a replacement gate authority. |
| E02[1] /03 | Same signed-Heal/predeath test retains victim buff source and a different killer equipment source before cleanup, expiry equality and duplicate-death once. Stat-death8-case table retains prior80HP and sourced/unsourced distinction. |
| E02[2] /04 | Native shared queued source, active action source and mutation facts are independent of narration. Test clears event log and compares complete summary; delayed source220 and off/on/gap219, original deterministic guards verify current carriers. |
| E02[3] /04 | Public RuntimeFactView/CoreApplication/Trial/CLI consume one native authority. Describe219 and source220 expose only current supported scope; no full Flow/root/true-join/zero-HP proc qualification added. |
| E03[0] /05 | sim request qualification occurs in native transaction before successful handle publication. `native_action_result_admission_rejects_before_focus_and_queue_mutation`, discarded-request and original no-target/invalid-state tests verify typed failure and unchanged state. |
| E03[1] /05 | `native_action_projectile_first_hit_is_at_300_after_accept_0_release_100` queries the actual handle at0/99/100/299/300: Accepted0, Released/control Finished100, zero damage and HP100 through299, actual first hit300 with HP100→93 and matching source/action/target, EffectsSettled300. The existing area test remains separate: hits100/200, expiry/settlement300. Homing uses current contact geometry; no future adapter is substituted. |
| E03[2] /06 | `native_action_withdraw_before_release_and_after_release_have_distinct_progress` t50/t150; existing HardCC/Silence/InterruptCast and released projectile/GroundFixed/Follow tests preserve cause-specific behavior. Death keeps existing queued-step/Follow semantics. |
| E03[3] /06 | runtime_actions cancelled-control/delivery-once, clone-relative validity/aborted ordinal reuse, public reserved-result slots; `native_action_piercing_empty_final_expiry_closes_carrier_without_releasing_gate` distinguishes carrier closure from deferred control. Exact caps refuse rather than evict. |
| E04[0] /07 | world combat `native_runtime_facts_preserve_actual_same_definition_inventory_copies_through_player_drafts` obtains actual two same-definition copies from owned inventory, checks runtime providers and distinct transient IDs. |
| E04[1] /07 | Same test covers AtBattleStart and Wave build paths plus inventory serialization roundtrip. player_spawns/draft and deploy/build pass EquippedRef instance IDs; full209 live_item_skill_activation4 and equipment/save tests preserve existing runtime identities. |
| E04[2] /07 | Runtime source explicitly uses None when no persistent copy exists; scenario/provider assertions and actual219/220 source output retain absence. No fragment-copy creation or guessed ID fallback. |
| E05[0] /08 | `native_action_failed_timestamp_keeps_prior_successful_prefix_and_retries_recover`, discarded-request/result-take and predeath refusal; world/application candidate boundaries include native state, queue/resources/RNG/IDs/log and prior committed timestamp prefix. Original full209 late-failure/atomic authoring tests pass. |
| E05[1] /08 | Clone-independent cancellation/completion, aborted-token reuse, bounded long internal execution; actual221 result owner None and cost003 reconnect/replacement/drop. Existing world entry/retreat/restart and old-authority tests run full209. |
| E05[2] /08 | Runtime state participates in live clone/snapshot/rollback. Existing V6/checkpoint/profile/owned-copy tests run full209. Durable battle/action handle serialization is N/A: no live BattleCore/TrialSession durable writer exists; diagnostic ordinal serialization is not a restorable handle. No new save-loss policy. |
| E05[3] /09 | RuntimeFactLimits/native RuntimeControl and authoring budgets price admission, COW physical trees, source/queue storage, predeath, retained results and counter overflow. Native boundary tests, exact210, original210,126 zero-allocation queries and actual cost003 prove finite behavior and trace/seed parity. |
| E05[4] /10 | full209/all-target211/default212, all current external consumers214–231 and canonical Core Runtime/Skill Target/Data Loading contracts. Scoped attribution002 and predecessor comparison003 identify required fresh R01→R02→R03 restoration; R04 fresh GEX20 remains separate. R05+ is not claimed complete. |

C01/I01 maps E01; C02/I02 maps E02; C03/C04/I03 maps E03; C05/I04 maps E04; C06/I05 maps E05[0–2]; C07/I05 maps E05[3]; C08/I06 maps E05[4] and the required subsequent audit. Plan SC01–06 follows these same producer/fact/action/copy/transaction/current-consumer boundaries. Detailed physical representations and deviations from initial implementation are recorded in DESIGN001, CORRECTION002 and EXP014–018; no reviewed policy meaning was changed.

## Required dimensions and changed-input seams

| Required dimension | Direct evidence |
| --- | --- |
| success | Actual formula/HP/resource/death/action/copy rows E01–04; external219–221 and225–230. |
| invalid_input | Qualification/limit E03[0], invalid228, API216 expected forbidden boundaries. |
| failure_rollback | E05[0], exact native210, public source218/replacement231; failed candidates preserve old handle authority and source. |
| retry_idempotency | E03[3]/E05[0–1], duplicate settlement, discarded ordinal, failure followed by success. |
| ordering_concurrency | New pre-formula identity/first-fault/rollback test and existing same-time/death/trigger/gate/carrier full209 tests, mixed packed-event priority test, original parity/repeatability210. Logical concurrent sessions preserve independent owners; no multithreaded gameplay claim. |
| boundary_empty_max | Overkill/overheal/signed zero, empty final piercing, predeath/source/tree/result/action/ordinal limits, exact/one-under210 and native tests. |
| disconnect_restart_persistence | E04 copy roundtrip/E05 live clone/reconnect/result/retreat/drop and existing checkpoint/profile tests; durable live attempt explicitly N/A as above. |
| live_data | Real literal RON in219 and225–230 parsed/validated/compiled by production API/CLI; official packaging214 and malformed228. |
| producer_consumer | Native mutations/current carrier/inventory→RuntimeFactView→ordinary app and authoring/CLI, actual221 exact native/Trial parity. |
| legacy_fallback_absence | Original guards and full suite; no log authority/guessed persistent copy/root/future executor. API216/CLI215 privacy negatives; source absence219/220. |

Correction002 changes damage admission/fact integration and their native tests only. Earlier R04 death/source/carrier/queue/transaction/storage changes remain owned by the original R04 delta. All affected tests and listed consumers are current INPUT008 executions. The initial audit findings remain in the old report until a new independent pass assesses the repaired target. Pure archived pre-R04 receipts are historical provenance only. R01–03 positive audits remain immutable old verdicts; their applicability is stale after R04 and requires fresh read-only restoration before R04 positive completion. No inaccessible required core consumer is known. Server/Unity/deployment/personal stores and future R05–24 behavior are excluded.
