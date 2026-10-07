# R04 implementation evidence — stable INPUT007

Owner `(GS-CARF-P2G50-002, GOAL-CARF-01)`, PHASE-CARF-01, HOF-CARF-P2G60-002; assigned AG-CARF-01 and WP-CARF-03. This is GEX10's implementation evidence map, not an independent completion verdict. Authority remains the exact [Goal](../../P2G_50_GOAL_SET_002.ko.md), Freeze003 and Plan004. All20 acceptance members,10 IVA,10 required dimensions and I01–06 are retained.

## Exact current validation

[INPUT007](evidence/IMPLEMENTATION_INPUT_007.json) identifies230 source/consumer/contract inputs unchanged since the final timestamp-bucket admission. The final audit target expands the surface to all applicable source/data/config and exact external outputs. Every `r04_<name>NNN.json` records the actual command, exit, raw log and digest. Old failed/intermediate commands remain historical in EXPERIMENTS and do not count as current PASS.

- [full164](evidence/r04_full164.json): `cargo test --locked --offline -p game_core`,1,361 PASS =1,309 library+52 integration tests,0 ignored/filtered. The five integration binaries run4/4/25/15/4 tests as recorded in the raw log;0 doctests are N/A and not gameplay tests. Existing assertions were retained; private packed-event consumer syntax was adapted without changing expected behavior.
- [all-target171](evidence/r04_alltargets171.json), [default-release172](evidence/r04_release172.json), [consumer examples173](evidence/r04_examples173.json) pass. Default production builds use no fixture feature; standalone allocation-observer instrumentation is explicitly separate.
- [ordinary guards003](evidence/ORDINARY_GUARDS_003/RESULT.json), [allocator168](evidence/r04_finalalloc168.json): original public plain/allocator and kernel plain/allocator pass all unchanged caps, deterministic parity and repeatability. Kernel plain p95 event13.180868ms/25ms and concurrent8.182800ms/10ms. Allocator event41,660 operations,255,622 maximum peak; concurrent367,088 operations,9,018,492 maximum peak against9,030,188 original cap. Original final_performance code, fixture, oracle and baseline are unmodified. The concurrent peak margin is small and is not generalized to other workloads.
- [native budget167](evidence/r04_finalbudget167.json): original five-second fixture,67 binary-search cases plus default and6 exact/one-less reruns. Minima clone843,279/work2,600,736/retained6,090,008; exact minima retain identical complete native facts and log hash, one less gives the selected typed fault. Original default clone1,048,576/work10,000,000/retained8MiB suffice. This measurement creates no gameplay or balance threshold.
- [cost002](evidence/RUNTIME_COST_002/RESULT.json):685 plain samples/5 repeats and411 instrumented/3 repeats after one warmup each;126 fact queries allocate zero times;12 measured load/drop pairs reclaim each of four coexisting generations'1,658,777 retained bytes. Sampled fact high water:25 accepted/24 settled,1 active,1 delivery,4 actors,2 providers,39,892 retained-byte bound. Full phase-level timing/peak/retention/counts and exact binaries are retained. Requested allocator bytes are not RSS and step-sampled high water is not instruction-level high water.

Current direct external consumers, all with isolated durable outputs:

| Evidence / commands | Actual result |
| --- | --- |
| PREDECESSOR_PACK_002 /174 | Original packaging/consume commands PASS; custom files preserved; actual initial currency500→733→500. |
| PREDECESSOR_CLI_002 /175 |21 commands,13 CLI cases,6 forbidden consumer compile cases; finite preparation/receipt/trace contracts PASS. |
| PREDECESSOR_API_002 /176 |11 original public consumer checks PASS, including expected privacy/fixture-feature failures. |
| PREDECESSOR_DIAGNOSTIC_002 /177 | Valid and restored policy/boss fixtures identical;6 negative calls preserve exact source file/field attribution, no output/token, restored input bytes. |
| PREDECESSOR_SOURCE_002 /178 |36 failed replacements,3 successful replacements,3 caller-edit proofs; exact byte-source receipt preserved independently of semantic identity, off/on/gap equal per source. |
| RUNTIME_FACT_EXTERNAL_002 /179 |4 API replays,2 CLI simulations,2 queries;91 damage mutations/strikes,277 HP lost,2 deaths,1 actual tactical payment spending10; mandatory output equal off/on/gap/repeat. |
| RUNTIME_FACT_SOURCE_002 /180 | Trace-off native source is authoritative; actual delayed projectile/periodic area source compared with captured trace only as oracle. |
| RUNTIME_APPLICATION_002.json /181 |13 ordinary CoreApplication steps, no TrialSession observation owner; exact terminal native facts match Trial and result consumption releases current world fact owner. |
| PREDECESSOR_COMBAT_002 /185 |12 commands,6 API/4 CLI runs,2 explanations; original A18/B36 damage and7,950ms application preserved. |
| PREDECESSOR_RESOURCE_002 /186 |10 commands,6 API/2 CLI runs,2 explanations; actual resource mutation and source parity preserved. |
| PREDECESSOR_BUFF_002 /187 |10 commands,6 API/2 CLI runs,2 explanations; same resource fixture pack hashes and buff observations across trace modes. |
| PREDECESSOR_INVALID_002 /188 |13 invalid cases,26 API attempts and26 CLI attempts;29 top-level commands, typed deterministic rejection. |
| PREDECESSOR_DELIVERY_002 /189 |9 commands,4 API/2 CLI runs,3 explanations; delayed repeat/projectile/area and terminal19,950ms preserved. |
| PREDECESSOR_PREVIEW_002 /190 |4 commands,4 API replays,1 discovery,9 positive previews; actual public prepare/advance/query boundary. |
| PREDECESSOR_REPLACEMENT_002 /191 | Original replacement consumer checks failed/successful A/B swaps, continued A state and bounded rejection; exact receipt in output directory. |

## Acceptance and integration mapping

Abbreviated `E01[0]` below means exact `PER-CARF-E01-REVIEWED-001[0]`, similarly E02–05. Every named unit test appears as PASS in full164; its source defines the concrete oracle. Paths are under `src/game/battle/core/` unless stated. IVA rows are the exact `IVA-CARF-NN` obligations and do not define new acceptance.

| Acceptance / IVA | Actual producer→consumer and direct oracle |
| --- | --- |
| E01[0] /01 | damage_runtime/basic_attack/skill_runtime preserve old formula, basic-only trigger, resonance and HP-loss gate; original full164 skill_runtime_contract25, all control/gate tests and original kernel parity. Combat185 A18/B36 verifies the literal production path. |
| E01[1] /01 | commands `runtime_facts_lethal_damage_retains_combined_hp_transition` requested2000/actual1000 and shared lethal fact; `runtime_facts_keep_overheal_zero_signed_heal_and_predeath_sources_separate` requested healing/zero/negative versus actual delta,0 Damage/Cost reclassification. |
| E01[2] /02 | sim CastEnd/manual request and world actual tactical payments feed Cost; `native_action_pending_death_preserves_cast_end_cost_without_release`, result-admission rejection, existing auto/manual/tactical tests; `runtime_facts_resource_modifiers_do_not_become_payments` requested−100/actual−30,0 Cost. External179/181 actual deploy20→10. |
| E01[3] /03 | death_runtime prepares predeath/fact before cleanup and keeps mandatory dead/wave processing separate from optional dispatch. Existing lethal/optional suppression/death/wave tests, stat modifier8-case table in commands, direct summon Kill versus Despawn tests all run full164. |
| E02[0] /04 | commands `runtime_facts_group_effect_major_damage_by_actual_iteration_and_target`:2 effect mutations per direct strike, two distinct iterations,40 HP per strike; old actual-damage gate count remains2. New strike scope plus per-segment ordinal is not a replacement gate authority. |
| E02[1] /03 | Same signed-Heal/predeath test retains victim buff source and a different killer equipment source before cleanup, expiry equality and duplicate-death once. Stat-death8-case table retains prior80HP and sourced/unsourced distinction. |
| E02[2] /04 | Native shared queued source, active action source and mutation facts are independent of narration. Test clears event log and compares complete summary; delayed source180 and off/on/gap179, original deterministic guards verify current carriers. |
| E02[3] /04 | Public RuntimeFactView/CoreApplication/Trial/CLI consume one native authority. Describe179 and source180 expose only current supported scope; no full Flow/root/true-join/zero-HP proc qualification added. |
| E03[0] /05 | sim request qualification occurs in native transaction before successful handle publication. `native_action_result_admission_rejects_before_focus_and_queue_mutation`, discarded-request and original no-target/invalid-state tests verify typed failure and unchanged state. |
| E03[1] /05 | `native_action_area_observes_accept_release_control_finish_and_actual_settlement` observes t0 Accepted/t100 Released/t300 actual hit/settlement with distinct control terminal. Current area/projectile/queued-step integration, not a future boss adapter. |
| E03[2] /06 | `native_action_withdraw_before_release_and_after_release_have_distinct_progress` t50/t150; existing HardCC/Silence/InterruptCast and released projectile/GroundFixed/Follow tests preserve cause-specific behavior. Death keeps existing queued-step/Follow semantics. |
| E03[3] /06 | runtime_actions cancelled-control/delivery-once, clone-relative validity/aborted ordinal reuse, public reserved-result slots; `native_action_piercing_empty_final_expiry_closes_carrier_without_releasing_gate` distinguishes carrier closure from deferred control. Exact caps refuse rather than evict. |
| E04[0] /07 | world combat `native_runtime_facts_preserve_actual_same_definition_inventory_copies_through_player_drafts` obtains actual two same-definition copies from owned inventory, checks runtime providers and distinct transient IDs. |
| E04[1] /07 | Same test covers AtBattleStart and Wave build paths plus inventory serialization roundtrip. player_spawns/draft and deploy/build pass EquippedRef instance IDs; full164 live_item_skill_activation4 and equipment/save tests preserve existing runtime identities. |
| E04[2] /07 | Runtime source explicitly uses None when no persistent copy exists; scenario/provider assertions and actual179/180 source output retain absence. No fragment-copy creation or guessed ID fallback. |
| E05[0] /08 | `native_action_failed_timestamp_keeps_prior_successful_prefix_and_retries_recover`, discarded-request/result-take and predeath refusal; world/application candidate boundaries include native state, queue/resources/RNG/IDs/log and prior committed timestamp prefix. Original full164 late-failure/atomic authoring tests pass. |
| E05[1] /08 | Clone-independent cancellation/completion, aborted-token reuse, bounded long internal execution; actual181 result owner None and cost002 reconnect/replacement/drop. Existing world entry/retreat/restart and old-authority tests run full164. |
| E05[2] /08 | Runtime state participates in live clone/snapshot/rollback. Existing V6/checkpoint/profile/owned-copy tests run full164. Durable battle/action handle serialization is N/A: no live BattleCore/TrialSession durable writer exists; diagnostic ordinal serialization is not a restorable handle. No new save-loss policy. |
| E05[3] /09 | RuntimeFactLimits/native RuntimeControl and authoring budgets price admission, COW physical trees, source/queue storage, predeath, retained results and counter overflow. Native boundary tests, exact167, original168/170,126 zero-allocation queries and actual cost002 prove finite behavior and trace/seed parity. |
| E05[4] /10 | full164/all-target171/default172, all current external consumers174–191 and canonical Core Runtime/Skill Target/Data Loading contracts. Scoped attribution001 and predecessor comparison001 identify required fresh R01→R02→R03 restoration; R04 fresh GEX20 remains separate. R05+ is not claimed complete. |

C01/I01 maps E01; C02/I02 maps E02; C03/C04/I03 maps E03; C05/I04 maps E04; C06/I05 maps E05[0–2]; C07/I05 maps E05[3]; C08/I06 maps E05[4] and the required subsequent audit. Plan SC01–06 follows these same producer/fact/action/copy/transaction/current-consumer boundaries. Detailed physical representations and deviations from initial implementation are recorded in DESIGN001 and EXP014–016; no reviewed policy meaning was changed.

## Required dimensions and changed-input seams

| Required dimension | Direct evidence |
| --- | --- |
| success | Actual formula/HP/resource/death/action/copy rows E01–04; external179–181 and185–190. |
| invalid_input | Qualification/limit E03[0], invalid188, API176 expected forbidden boundaries. |
| failure_rollback | E05[0], exact native167, public source178/replacement191; failed candidates preserve old handle authority and source. |
| retry_idempotency | E03[3]/E05[0–1], duplicate settlement, discarded ordinal, failure followed by success. |
| ordering_concurrency | Existing same-time/death/trigger/gate/carrier full164 tests, mixed packed-event priority test, original parity/repeatability168/170. Logical concurrent sessions preserve independent owners; no multithreaded gameplay claim. |
| boundary_empty_max | Overkill/overheal/signed zero, empty final piercing, predeath/source/tree/result/action/ordinal limits, exact/one-under167 and native tests. |
| disconnect_restart_persistence | E04 copy roundtrip/E05 live clone/reconnect/result/retreat/drop and existing checkpoint/profile tests; durable live attempt explicitly N/A as above. |
| live_data | Real literal RON in179 and185–190 parsed/validated/compiled by production API/CLI; official packaging174 and malformed188. |
| producer_consumer | Native mutations/current carrier/inventory→RuntimeFactView→ordinary app and authoring/CLI, actual181 exact native/Trial parity. |
| legacy_fallback_absence | Original guards and full suite; no log authority/guessed persistent copy/root/future executor. API176/CLI175 privacy negatives; source absence179/180. |

Changed damage/death/source/carrier/queue/transaction/storage inputs invalidate old runtime receipts; all affected tests and listed consumers are current executions. Pure archived pre-R04 receipts are historical provenance only. R01–03 positive audits remain immutable old verdicts; their applicability is stale after R04 and requires fresh read-only restoration before R04 positive completion. No inaccessible required core consumer is known. Server/Unity/deployment/personal stores and future R05–24 behavior are excluded.
