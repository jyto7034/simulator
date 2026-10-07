# R03 semantic occurrence / terminal owner design 026

Bounded AUTORUN §8 source/design assistance for root, not implementation, test execution, or a completion audit. Root remains sole repository writer. Read window: 2026-09-10T12:01:26.412602+00:00 through 2026-09-10T12:06:34.150209+00:00; close hashes at 2026-09-10T12:06:48.722025+00:00. All numbered reads use Python Path.read_bytes(), UTF-8 splitlines(), SHA256 of those exact complete bytes. Companion JSON records every read span/time/full hash and close hash. No build/test/runtime/Git command was run. Discovery used rg; a few nonexistent guessed paths returned errors and supplied no evidence.

The Goal is unchanged: real effects and reproducible source, independent semantic occurrences, capture before terminal removal, trace-independent gameplay/RNG/admission/work/mandatory receipt, complete failed-candidate rollback. Goal54–72/96–104 and PLAN64–86/91–93 are the authority here. Earlier design00397's suggestion to put the capsule in GameCoreState predates the implemented GameCore-only Copy lifecycle; the following updates that owner recommendation, without rewriting the earlier report.

## 1. Smallest owner graph that fits current transactions

Keep TrialLifecycle and TrialBattleWitness Copy, including their existing actual readiness/elapsed/verdict producer checks. Add **separate** private metadata owners, all built in default release:

- BattleCore: Option<BattleObservation>, enabled only for the private authoring application's real planned battle. It owns mandatory checked application ordinals/counters, fixed-cardinality witness slots and source handles; optional bounded trace is a separately accounted child.
- GameCore: Option<TrialObservation>, enabled beside new_for_authoring's existing lifecycle. It owns the last immutable completed capsule, not the database, current runtime, or a whole public JSON result. An Option<Arc<TerminalCapsule>> makes subsequent complete GameCore forks copy only an admitted handle.
- TrialState continues owning content/scenario/initial authority/public results/receipts. Its private application accessor returns a **borrowed observation view** (active mandatory summary or retained capsule, matched to lifecycle entry ordinal). Serialize this view before the final receipt; never expose mutable BattleCore/observer or put it in the prior-public-result binding slots.

Concrete private shapes, with naming illustrative:

```rust
struct BattleObservationKey { entry_ordinal: u64, battle_uuid: Uuid } // Copy
struct ApplicationWitness {
    ordinal: u64,
    time_ms: u64,
    source_unit: Option<UnitInstanceId>,
    target_unit: UnitInstanceId,
    change: AppliedChange, // fixed owned scalar enum; see §2
    source: Option<Arc<SkillEffectExecutionSource>>,
    delivery: Option<DeliveryCorrelation>, // Copy, descriptive only
}
struct MandatoryBattleObservation {
    key: Option<BattleObservationKey>, // provisional until real live entry
    applications: u64,
    nonzero_hp_applications: u64,
    sourced_applications: u64,
    latest: Option<ApplicationWitness>,
    latest_nonzero_sourced_hp: Option<ApplicationWitness>,
}
struct BattleObservation {
    mandatory: MandatoryBattleObservation,
    trace: OptionalTrace, // bounded isolated owner, not a normal dynamic meter visit
}
struct TerminalCapsule {
    key: BattleObservationKey,
    actual_time_ms: u64,
    outcome_id: BattleOutcomeId,
    verdict: BattleVerdict,
    mandatory: MandatoryBattleObservation,
    trace: OptionalTrace,
}
struct TrialObservation {
    last_terminal: Option<Arc<TerminalCapsule>>,
}
```

This is bounded-cardinality mandatory ownership, **not a claim that two witness slots are a complete occurrence history**. The mandatory application count and last actual nonzero sourced witness provide trace-off evidence; optional rows carry per-occurrence detail up to their own explicit gaps. If a frozen consumer requires particular earlier source occurrences after eviction, that requires additional bounded mandatory slots/explicit coverage, not relabeling an evicted optional row as retained. Do not publish “all effects observed” from these HP/modifier hooks alone.

ApplicationWitness's retained graph is small: SkillEffectExecutionSource contains Arc<EffectSource> and Arc<SkillCastExecutionSource> (skill_source22–35). EffectSource owns source_file/site/action/effect/field_path/origin_definition_id/origin strings (ability/authoring10–42); cast source owns SkillId and scalar caster/cast/log/proc data. Neither owns GameDataBase or a generation. CooldownSource is a Copy Unit/Item identity enum (cooldown7–11). The source structs themselves do not implement Serialize: build an explicit borrowed serialization projection for source.authored plus cast skill/caster/cast_seq/ability_event_seq/cause/proc fields. Never serialize a raw Arc graph recursively.

ContentIdentity is already in each receipt (trial172/183,463/479). Its contract+semantic_sha256 is an external identity namespace, not a new battle-owned String clone. Use the enclosing receipt's exact identity and scenario input hash/initial seed/player to namespace occurrences. The local key is (entry_ordinal, application ordinal); battle UUID and cast/log/delivery identities remain correlation. A reused gameplay UUID after retreat is distinguished by existing checked entry_ordinal. These ordinals must not feed gameplay RNG, event ordering, damage_source_seq, UUID manager or authority revisions.

A standalone exported occurrence must carry that enclosing namespace; a process-local Arc address or global generation counter is not reproducible identity. Raw source_file is the compiler's known path, not an inferred pack. When a pack/writer is required, use the same token's ContentSourceMetadata and actual DefinitionKey (content_pack/mod25–50, raw210–223); do not reopen files or parse names. Borrow/charge an actual metadata lookup outside the per-hit hot path. An Added origin's origin_definition_id and the selected owning skill definition are distinct fields; do not substitute one for the other.

## 2. Actual occurrence producers and source discipline

**Damage:** execute_committed_damage179–209 rejects absent/inactive target before calculating anything; it calculates result/summary at277–280. The actual mutation is apply_damage_result_and_record449–451, after its own target checks435–447. Record from actual hp_before/hp_after there, before record_event_log453 and death481–487. Use i64::from(after)-i64::from(before), not the existing log's narrowing i32 subtraction, for truthful full-u32 HP delta. Raw/final damage and critical are actual result fields but are not the actual clamped HP loss. Log ordering cannot provide the mandatory witness.

**Heal/signed HP:** apply_hp_delta_and_record499–529 returns on requested zero/missing/inactive; actual HP assignment528 precedes log531 and death559. Hook at that assignment with true applied before/after. Full-health healing can commit a zero delta; a requested zero that returned before mutation is not an application witness. Builder sim2091–2092 skips Damage(0); builder's applied_effect_count at2130/2145 is a generation estimate, not an actual effect success counter.

**Modifier:** commands602–606 applies the modifier and updates speed with actual stats before/after, before StatChanged608 and nested death at619–625. Make one AppliedChange::Stats { modifier, before, after } witness including its HP clamp. Do not separately count the companion record_runtime_stat_hp_change_and_finalize589 log as a second authored-effect success. If all HP mutations are instead intentionally separate observation channels, publish that distinction explicitly; neither count is a gameplay “effect count” by inference.

Admit the fixed observer operation and checked next ordinal before mutation, then commit the witness immediately after actual mutation and before callbacks/death/recursive effects. Updating counters only after process_commands returns reverses nested occurrence order and can miss an effect before its source is restored. No saturating/wrapping semantic ordinal, manufactured UUID, log-sequence allocation, or fake success on observation failure. Optional trace append runs after the mandatory witness exists and cannot reject its application.

The current command path is infallible at process_skill_commands487, process_commands/batches, and the HP helpers. control::charge failures already latch and the skill loop restores temporary source before poll (commands489–511). A new checked-observer invariant failure must also reach a real failure boundary. Current control only exposes charge/admit/poll; plain GameError returned then ignored by an infallible helper is insufficient. Minimal alternatives are:
1. give private observer reservation a typed sticky internal failure through the existing attempt owner, restore context, and stop at the existing prompt polls; or
2. propagate Result through every affected direct caller to process_event/world, including formerly infallible loops.
Do not simulate counter-overflow as an ordinary domain no-op or saturating count. Scoped work exhaustion is already finite, but that does not justify an unchecked counter.

**Do not reuse ambient recording_skill_effect_source as direct attribution without scoping it.** process_skill_commands493–508 installs authored+cast source around the whole command. A lethal application calls finalize_unit_death, which invokes process_command_batches (death_runtime155–164); batches call process_commands_with_modifier_source directly without clearing that field (commands456–470). Thus an unsourced nested item effect can inherit the lethal skill's ambient recording source. A minimal observation-only fix is a private direct-effect context installed by the skill loop, and a take/restore guard around unsourced process_commands and process_command_batches; alternatively explicit provenance parameters to the mutation leaf. Keep existing recording/wire behavior unchanged. Nested actual skill invocation installs its own source; restore the outer source on all exits. Tests already exercise sourced lethal max-health change followed by nested heal/kill procs (commands4017–4211); extend assertions to ensure each direct effect is attributed accurately.

Current Instant process_skill_commands call is sim2406–2411; immediate tile-area call2560–2565; projectile impact961–966 uses source_snapshot.skill_cast_source; persistent-area tick765–770 does likewise. Area creation stores cast source at539–542; area_tick_source_snapshot87–95 clones it while independently changing actual tick roll/time. SkillCastExecutionSource.cast_seq at sim467 is an existing skill_cast_seq counter; ability_event_seq/cause/proc.event_seq are log correlations. Neither is a per-target/repeat semantic application key. Pass a Copy delivery correlation (Instant, projectile delivery_id, area_id) at these actual call sites if the receipt promises delivery identity. Do not recover it by walking a log parent chain. All source references may be shared by multiple distinct applied ordinals.

Other current effect paths are materially separate, and must be labeled/connected honestly:
- ApplyBuff only queues BattleEvent::ApplyBuff with IDs/duration/cause (commands744–757; enums127–140); actual application is sim3219–3259. No authored effect handle travels with that event or BuffTick. Queue admission is not applied buff success.
- ModifyDamage is folded into sibling Damage snapshots at sim2062–2072, never executes a standalone command. A Damage witness cannot claim the contributing modifier effect's separate origin unless that attribution is carried explicitly.
- ModifyTacticalResource creates BattleLiveSignal at sim2246–2263; the current signal has only unit/skill/amount (sim202–207), no exact effect/cast source. Its actual resource consumer is beyond this bounded leaf read.
- ModifyResonance and InterruptCast have different actual mutation predicates (mod694–744, commands647–679). ExtraAttack schedules an event (commands731–742); scheduling is not a future hit occurrence.
These are source-wiring obligations if the advertised observation coverage includes them. They are existing runtime operations, not permission to implement R06 chain/event gameplay or expand literal editable support.

## 3. Creation, terminal, exit and complete rollback

Current actual setup constructor is CombatExecutor::resolve_live_combat_setup_plan_with_authored_policy219–220. Its only production callers found by rg are its default wrapper113 and world/combat487. The feasibility validator uses the default wrapper85, so merely checking TLS-active would also observe generated preview kernels.

Small explicit opt-in: add a private observation config parameter at the lower setup boundary, default None in the wrapper, and pass Some only from GameCore's real plan when its private trial owner is enabled. Install a provisional BattleObservation after BattleCore::new_from_database and before start_battle_execution. Runtime reset sim1630–1662 must reset counters/rows but preserve the enabled config; ordinary constructors stay None. No public fixture/import enable API is needed.

Actual source confirms startup1831–1849 validates field and enqueues scenarios; build_runtime_field421–429 only checks tiles; enqueue_scenario_events1518–1544 queues spawn actions. OnBattleStart effects run during first time-zero drain1924–1933. Therefore the absolute smallest current HP-only installation could occur at world entry before advance(0); the pre-start explicit config matches design00389 and avoids having two definitions of observation coverage. Neither option may count generated feasibility as a trial entry.

World commit636–691 allocates gameplay battle_uuid683, installs active686, transitions687, marks actual entry688, then calls advance(0)689 and time_zero_completed690. Bind the provisional observer to the newly checked lifecycle entry ordinal inside authoring_battle_entered; require observer/lifecycle agreement, never infer it from logs. A same-time terminal can transfer its capsule before time_zero_completed; do not freeze a false time_zero_completed into the capsule. Keep that completion authority in the existing lifecycle and read it when preparing the receipt.

Terminal ordering is exact:
1. advance_active_battle_staged finalizes the actual kernel result1493–1505.
2. Actual public update and VN projection succeed1544–1568.
3. authoring_battle_advanced1569 stores observed elapsed.
4. transition_finished_live_battle_to_result1401 validates finished lifecycle.
5. Before active_battle=None1409, move mandatory observation + optional trace into the private terminal capsule, using the actual battle outcome_id/verdict and active.execution.last_event_time_ms.
6. Finish transition_to; outer execute_application reconciles/polls before commit.

Copy the needed outcome_id only after a controlled borrowed field projection; move the observation and immutable source handles instead of cloning runtime or logs. Allocate/admit the one new immutable capsule owner before destructive take. A later transition/projection/receipt/control failure drops the candidate capsule with the candidate application. Retain it across CompleteCombatResult, failure acknowledgement, checkpoint/run resets; tag it with its entry ordinal so it cannot masquerade as the current battle. Replace on the next completed battle or drop on TrialState replacement. Prior accepted serialized receipts remain independent owners. The existing public terminal result is already stored in AcceptedStep, so duplicating a whole public Value in the capsule is unnecessary.

Exit without terminal: retain the last accepted step's receipts; a retreat must not invent a terminal outcome. If the requested observer API promises the final pre-retreat active summary after gameplay snapshot.restore, move it into a distinct Exited capsule in handle_retreat_battle2803–2810 before consuming entry_snapshot. Do not use the terminal capsule variant for retreat. If only prior accepted observations are exposed, explicitly report no terminal capsule. Reconcile after active removal can invalidate readiness, but cannot recover an observer already dropped.

Complete owner propagation:
- GameCore derived Clone / exhaustive CloneShape world51–64 includes TrialObservation.
- PreparedApplicationCommit must carry/move the new owner in all construction/destructure/commit sites (world/application128–134,194–222).
- with_state_transaction captures/restores it beside lifecycle (node_flow788–798); complete all fallible capture admission before native clone.
- cfg(test) ActiveRunTransactionRollback adds the same field/capture/restore (active_run20–25,178–182,211–218).
- BattleCore derived Clone / exhaustive visitor adds mandatory observer and transient context (core/mod135–225); immutable source Arcs cost handle copies. Trace gets a constant reserved visit independent of rows/flag, not generic Vec or Option recursion that changes work.
- Whole BattleCore candidate owners already cover boss staged clone1974–1995, spawn before clone651 and restores655/664/692/697, summon before316 and rollback398/406/413/420/473. ActiveBattleSession tactical staging clones at world/combat1842–1848 and restores original on VN failure1859–1861. New BattleCore observation state then follows these actual rollback owners automatically.
- Opaque fork application802–805 clones complete GameCore. Trial prepare and advance flatten both outer control Result and inner domain Result (trial102–122,313–325); no outer-Ok(domain-Err) acceptance. New observation view/capsule ownership/receipt encoding must be prepared before that final swap. B preparation uses one scope and must pass actual readiness before A replacement126–138.

Do not put observer/capsule in GameCoreState checkpoints, V6 payloads, public mutable fixture state or shared mutable Arc<Mutex<...>>/TLS output sinks. Cancellation source may be shared read-only; trial A/B observation may only share immutable handles. Retained source metadata alone does not pin the full generation; TrialState's existing token still does. Drop/replacement releases the capsule/trace/source handles when no accepted owner remains.

## 4. Optional trace and mandatory self-receipt

OptionalTrace may use bounded owned rows or an immutable Arc buffer with isolated copy-on-write. Its count/byte ceilings must be checked before retaining/encoding each row; source text expansion needs its own capped writer so large authored strings cannot allocate first. All optional work, duplicate source encodings, clone/detach, and rows use an explicit separate allowance. A row overflow/encoding failure produces a fixed gap/drop status and drops only optional data. It must not call execution control admission, mandatory output::Writer, or change mandatory counters/bindings. Its memory/work are real and require separate measured claims; no bound here proves all heap safety.

A trap in the **current** receipt: output::Writer32–48 charges each actual serialized byte; encode_receipt93–118 reserves only usage prefix, then serializes the payload. Putting variable-length trace enabled/count/gap fields in ordinary StepPayload changes mandatory work/bytes/admission. Either expose optional coverage beside the mandatory receipt, or reserve an always-present, fixed-length padded coverage block with identical charge/admission in both modes, then fill it without additional execution charge before final acceptance. The current usage-prefix technique is a precedent, not already an implementation of coverage reservation. Fixed mandatory source/counters may be ordinary bounded payload because they are generated identically regardless of trace mode.

Likewise CloneShape for generic Vec traverses live rows and Option depends on presence; do not derive a generic visitor across OptionalTrace. Use an explicit constant slot reservation in its parent and keep optional storage accounting separate. Readonly observation serialization must not mutate/drop active rows, advance counters, derive gameplay IDs, or consume next-step mandatory fuel.

## 5. Concrete verification seed and missing acceptance

An actual read fixture exists: tests/fixtures/content_authoring/literal_effects.ron1–32 contains alpha/beta, true damage17 plus other_damage3, delayed(1ms) heal9, and beta's Added(alpha) origin. tests/skill_runtime_contract.rs46–198 parses it, changes literal amounts to17/21/0 and9/-5, executes actual fixture BattleCore, and compares real HP deltas/exact sources. This is an excellent narrow new producer oracle, **not a public default-release authoring acceptance**: it uses GameDataBuilder/direct fixture units/enqueue.

Extend that fixture's observer variant to retain mandatory witnesses independently of logs: expected damage difference4, zero-damage no application, signed heal actual delta, exact Added source; insert an extra unrelated log, use a tiny/closed log, disable trace, force optional overflow, and compare semantic application IDs/counters/source and gameplay continuation. Repeated same-effect hits/area ticks need distinct ordinals with shared source; killed caster projectile keeps source. Add nested lethal modifier→unsourced heal to detect ambient-source bleed. Force mandatory observation/capsule/receipt failure on a candidate and verify original observer counters/source/next result remain identical; no accepted ordinal holes from rejected drafts.

For terminal lifetime, current world test combat12759–12849 already covers actual terminal removal at t0 and positive elapsed, ordinary vs authoring, plus CompleteCombatResult preserving verdict. Extend its assertions to the retained source capsule; its existing immediate-terminal case has no damaging effect and cannot alone prove terminal nonzero provenance.

The public path exists in tools/battle_kernel_api_audit/valid/src/main.rs17–119 (external load → start/VN/starter/map/select/confirm → readonly reconnect). It currently stops before nonzero combat. Current trial.rs572–636 constructs finite public bindings for a test. Root's required default-release fixture must combine a validated external literal change with such an explicit prepared scenario, actual deploy/skill/clock authority, nonzero source receipt and terminal. Keep test-time index selection outside the production runner. No such new execution was performed or claimed here.

## 6. Drift and scope limit

Close hash drift occurred only in PLAN.md, application.rs and control.rs among these 41 input paths. Their later bytes were hashed but not reread; findings reference the captured read versions, and root must reconcile its concurrent deltas. Other source/fixture hashes matched at close. This is a focused source/design map, not a whole observer implementation review, source-completion verdict, or test result. The copied owner/context/queue/source/trace accounting transitions still require implementation and focused verification by root.

### Exact input manifest (full hashes; read spans per companion JSON)

1. `core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/design-evidence/trial_port_design_003.md` — read SHA256 `3cc58bce2565c6e14e03ac7d4987caffcacc9ebec5224b4eb44cae6505308162`; close unchanged.
2. `core_enhanced/docs/goals/content_authoring_tool_ports/P2G_50_GOAL_SET_001.ko.md` — read SHA256 `bb0b69cfebceb17fd13374826e1dbb6ba6e38e711cb574acc29ca61ca2745f98`; close unchanged.
3. `core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/PLAN.md` — read SHA256 `5fef609f4a206c67dc31e621601e702b138257d1920dad0dac80ea14b7fc581e`; close drift `280bc62ae3714861713baaedd55a525ced4b555f80ff5fb49aa0fbf14a13fb2f`.
4. `core_enhanced/src/game/battle/skill_source.rs` — read SHA256 `798d5a96e4550bab80cda46a724f13990be5cffe73d6a3535c179f2d3a8d7d30`; close unchanged.
5. `core_enhanced/src/game/battle/recording.rs` — read SHA256 `e3c1afedaf9b5ec61c76d415e2123dbea8a9c6d9213da5393dddfaf0e6beaf2d`; close unchanged.
6. `core_enhanced/src/game/battle/core/commands.rs` — read SHA256 `b32f1b9a9f47c7124d3944cf6bd409a6e514fb489ee36b9cbba8ea251177a542`; close unchanged.
7. `core_enhanced/src/game/battle/core/damage_runtime.rs` — read SHA256 `c2faa80ae08c04c294defdc2474765128b4d336e0be51d3d57145ae591e43cbf`; close unchanged.
8. `core_enhanced/src/game/battle/core/sim.rs` — read SHA256 `9e3a3ef70179d302be10b8d6f0ca7873885cd448ac629e893505f6856b35f5b4`; close unchanged.
9. `core_enhanced/src/game/battle/core/skill_runtime/projectile.rs` — read SHA256 `8780ed276ff988e6ba8cb1e15e3cdc8d645f73e9207a0b71854286b2130f580c`; close unchanged.
10. `core_enhanced/src/game/battle/core/skill_runtime/area.rs` — read SHA256 `b7cad108fd507a19ee7893bffc763bd9be06f6ab2187306a19f21b48042ea89d`; close unchanged.
11. `core_enhanced/src/game/world/authoring_lifecycle.rs` — read SHA256 `69e06f862a961fe0ff1d9b1b35fbdb869d1db4e0cb4db69e7f71efdda552aa8c`; close unchanged.
12. `core_enhanced/src/game/world/combat.rs` — read SHA256 `b78e0de0ab027b3f99ffbd8c078df6ea8fff5fd1559992ed600a4a3579ce0062`; close unchanged.
13. `core_enhanced/src/game/events/combat.rs` — read SHA256 `d88719417886eea018ae08a66c750884f163051f160d0b9b21ece3c1eddffc66`; close unchanged.
14. `core_enhanced/src/game/ability/authoring.rs` — read SHA256 `c4ca67ceac4502c2c31c8c91a584e09dff3c87fd8976269d436dbddbc2e08e1a`; close unchanged.
15. `core_enhanced/src/game/battle/core/mod.rs` — read SHA256 `d9b6f0b7e48cdd9a8d0cf2e72ddb678ac58c2a260e6cc7cb24d3dcacdae276ec`; close unchanged.
16. `core_enhanced/src/game/world.rs` — read SHA256 `6dc90b38c0d4aa09a692b757665d45ff47fce274aa47df955eb23e8d8fff2de7`; close unchanged.
17. `core_enhanced/src/game/world/application.rs` — read SHA256 `5b1a67a4550f31f03e7baeb1db45d8293147978e90d0b4b53df99f934481b2f0`; close unchanged.
18. `core_enhanced/src/game/world/node_flow.rs` — read SHA256 `949e4d04f2b8d64d6e591c0a9fea1ee23da0d43d452a97f6b5344e1b97d0bdcc`; close unchanged.
19. `core_enhanced/src/game/world/active_run.rs` — read SHA256 `1f37b9d6e08f767e9bc439571da4b9ae0628176f5e6cb40f3356f18a01f88d12`; close unchanged.
20. `core_enhanced/src/game/battle/core/death_runtime.rs` — read SHA256 `abdcb6de0b58487011f28df0652a7531fccabec2fa2e6ac833662d30eaf6ce71`; close unchanged.
21. `core_enhanced/src/authoring/trial.rs` — read SHA256 `0500b82ac8bc8cfc4cbbbf076520834e27de756147fab7d97f74659cb7cdd22a`; close unchanged.
22. `core_enhanced/src/application.rs` — read SHA256 `f61cb3fa2421e96e737c63bd83791853509841bc2840e4f087dbb36d5ddd87e1`; close drift `e6a48261df88b551d430433da6e1e5da8bb426d20b2fc2c96fb7642318aea1e6`.
23. `core_enhanced/src/game/battle/event_log.rs` — read SHA256 `49ddbf7545b8939ab4ae2b3cd42a87e1b42fbe97c7dbdfec67201daf33b63619`; close unchanged.
24. `core_enhanced/src/game/data/content_pack/mod.rs` — read SHA256 `cdca9db0f33f6590406162db40c8a238fe195f55c5e4754383fb86065fa03fab`; close unchanged.
25. `core_enhanced/src/game/battle/damage.rs` — read SHA256 `20b826044fa46bd7b10cb07bd0fec1a5b540a90228c95ead1bea36fefbe0c2da`; close unchanged.
26. `core_enhanced/src/game/world/tests/combat.rs` — read SHA256 `c919d9adaa093572e37d0ce9c6113d6ac75e61f0949a6d45ce968116cf14d787`; close unchanged.
27. `core_enhanced/docs/roadmaps/content_authoring/AUTORUN_REQUEST.ko.md` — read SHA256 `3e05b131d445cd10ecf478ef959b80f55ce718c41c8a1820e5a5b7d969f9a252`; close unchanged.
28. `core_enhanced/src/game/battle/core/build.rs` — read SHA256 `99e777d2791b3b6223bdbaada1eba0d3f18c267e1a7fd2565d0f6108d0199761`; close unchanged.
29. `core_enhanced/src/game/battle/core/boss_runtime.rs` — read SHA256 `e7049c14d235cf5415b9785c33b8450dfff12b36571f830d5f7d9475090f3516`; close unchanged.
30. `core_enhanced/src/game/battle/core/spawn_kernel.rs` — read SHA256 `fab3d3c46f54e76281bf98f6e7430192a9599b5c6dd364190f6378b1ae2bd9a0`; close unchanged.
31. `core_enhanced/src/game/battle/core/summon_runtime.rs` — read SHA256 `d045c86228b29964214d3c544af889aeb1983707ed4bd9cc354b1a5303320e0c`; close unchanged.
32. `core_enhanced/tests/skill_runtime_contract.rs` — read SHA256 `d93e878163e5e4cd0f8f2a2be68c2c914e0c219150c9a73961113bf9124db8e8`; close unchanged.
33. `core_enhanced/tests/fixtures/content_authoring/literal_effects.ron` — read SHA256 `1bd51a81d6b528ea4a80895bf95348c8417fd7113c77a73c6a2543d7821cf8c0`; close unchanged.
34. `core_enhanced/tools/battle_kernel_api_audit/valid/src/main.rs` — read SHA256 `029179ae0b7d715ae01001e377800ec128276616be1406204750be94915cd745`; close unchanged.
35. `core_enhanced/src/game/data/content_pack/raw.rs` — read SHA256 `72c6fa4917831d76c87c0f9a0cd58c8be3c8e5342235c8ef01a80a94c94852e7`; close unchanged.
36. `core_enhanced/src/game/data/content_pack/schema.rs` — read SHA256 `3c44c1cb3fc4df3ef5ebea29cfbdb939a3614735ec77e2391f2ee4b4a1fd0616`; close unchanged.
37. `core_enhanced/src/authoring/output.rs` — read SHA256 `2951433ffb08b52568028b70485cc12992c3c172a064e44df4795b1a22c71ddb`; close unchanged.
38. `core_enhanced/src/authoring/control.rs` — read SHA256 `1a15e8bdd7618572950ddc69ad25b5cda674eebfecd0f544010abb3477b2fed8`; close drift `2937619104c4091cbc580e36fb09c40d14f8cca474fa7a885be328a3e87d6478`.
39. `core_enhanced/src/game/battle/cooldown.rs` — read SHA256 `fa401d4db5a2c6572a8ef404979303fe4c5b957e1aa3160f62befef12a3d55f4`; close unchanged.
40. `core_enhanced/src/authoring/clone_shape.rs` — read SHA256 `6e56d3990b14357c98663ace941fc44b6cde7c41667829b4dddec78e71adbc6b`; close unchanged.
41. `core_enhanced/src/game/battle/enums.rs` — read SHA256 `d54700d4690fd1cf110a9ccab6e22def7b85053dcbbad2db105f01b1280ba919`; close unchanged.
