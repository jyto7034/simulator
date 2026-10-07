# R04 GEX-10 detailed implementation design

2026-09-11. Current `(GS-CARF-P2G50-002, GOAL-CARF-01)`, HOF-CARF-P2G60-002, WR002/Freeze003/PP004/PVR004. This is an implementation design under the accepted Goal, not a policy/acceptance owner or completion verdict. [PLAN](PLAN.md) and [baseline002](baseline/BASELINE_002.json) own authority, actual source, isolation and reintake. No production or runtime test has executed at this design's initial writing.

## 1. Realization boundaries

Add public value types in `game/battle/runtime_facts.rs`, engine state/helpers in `game/battle/core/runtime_facts.rs`, and synchronous native operation accounting in `game/battle/runtime_control.rs`. Preserve existing gameplay mutation, formula, trigger and carrier authority. AG-CARF-01 implements the producer/state/consumer cutover; WP-CARF-03 owns durable tests and actual measurements. A compiling partial helper is not an implemented Goal.

Public values are immutable snapshots. `ActionHandle` has private immutable attempt/action tokens and an ordinal; it is not deserializable, and pointer/token values are not serialized. `ActionSnapshot`, fact summaries and settled receipts serialize deterministic logical identities and actual facts only. No fake root, full Flow, branch history, fragment copy or new reaction is introduced.

## 2. Action identity, result ownership and reclamation

Use `ActionOrdinal(u64)`, `StrikeOrdinal(u64)` and `DeliveryOrdinal(u64)` with checked increment in the attempt-owned store; their counters do not consume or replace existing cast/projectile/area/damage-source/RNG/log counters. Internal identities carry the same immutable token pair as public handles but do not acquire result reservations merely by being copied.

```rust
pub struct ActionHandle {
    validity: Arc<ActionToken>,
}
struct ActionToken {
    attempt: Arc<AttemptToken>,
    ordinal: ActionOrdinal,
}

struct RuntimeFactState {
    attempt: Arc<AttemptToken>,
    next_action: u64,
    next_strike: u64,
    next_delivery: u64,
    next_fact: u64,
    active: Arc<BTreeMap<ActionOrdinal, ActiveAction>>,
    retained: BTreeMap<ActionOrdinal, RetainedAction>,
    actors: Arc<ActorSummaries>, // sorted exact-capacity actor rows
    providers: Arc<BTreeMap<EquipmentSourceSnapshot, ProviderFactSummary>>,
    latest_settled_action: Option<Arc<ActionSnapshot>>,
    summary: FactSummary,
    usage: RuntimeFactUsage,
    limits: RuntimeFactLimits,
}
```

The token payload and Arc overhead are included in retained source ownership. ActionSnapshot, DamageFact and DamageSegmentFact share immutable source values only when every actor/equipment/cast/authored/action slot is equal; each owner still carries a conservative full retained extent. Other observations preserve their existing value representation. Each `ActiveAction` owns its token, caster/skill/source, optional Accepted/Released timestamps, remaining authored control steps, exactly-once control terminal, outstanding finite delivery IDs and a compact fact summary. `RetainedAction` owns only the token and immutable `Arc<SettledActionReceipt>`. Control terminal is `Finished`, `Cancelled(existing cause)` or `Failed(execution reason)`. Effects settlement has a separate timestamp. An aborted operation publishes none of its new action state, facts or counters.

Query validates attempt token, ordinal lookup and action token, in that order. Tokens use identity comparison only; no ordering/RNG/semantic result depends on their addresses. Cloning an existing core preserves its handles. Two cloned cores are independent authorities: query validity and later state are relative to the core being queried. A new allocation in an aborted/divergent candidate has a fresh token, so restoring and reusing its ordinal cannot validate an escaped handle. Releasing/exiting one core cannot revoke an independently preserved clone through shared mutable token state.

Do not use `Arc::strong_count`, Rust drop, query timing, age or latest-result replacement to decide whether an explicitly promised result can be discarded. An explicit public request reserves its completed-result slot at acceptance, while it is still active. Completion cannot then fail because a new result slot is needed. Internal automatic/proc/basic execution has no public reservation by default. After control terminal and all finite effects settle, merge bounded actor/provider/attempt summaries; retire internal working records or move promised results into the compact retained map.

Expose small native methods:

- `request_skill_action(...) -> Result<ActionHandle, GameError>`: same existing manual qualification, then admitted public result reservation and acceptance within one staged operation.
- `query_action(&self, &ActionHandle) -> Result<ActionSnapshot, ActionQueryError>`: no mutation, no reservation creation, no implicit success.
- `take_settled_action_result(&mut self, &ActionHandle) -> Result<Arc<SettledActionReceipt>, ActionQueryError>`: requires valid control terminal and EffectsSettled, prepares the immutable return value, then atomically removes the reservation/record. A repeated take/query returns a typed unavailable/released error. This never cancels execution.
- `runtime_fact_summary(&self)` and bounded per-actor/provider projection: actual current facts, without a historical action list.

Keep existing `apply_live_command(ActivateSkill)` as an unreserved operation returning its existing outcome; the new explicit request method selects public retention. Both use the same private qualification/acceptance implementation with an explicit retention disposition. The existing world wrapper at `world/combat.rs:2719–2792` consumes and discards `SkillActivated` and therefore must not accidentally reserve a public result. World/authoring/CLI consume bounded current snapshots instead. A future explicit native retain operation is unnecessary for this Goal.

Engine-owned attempt state is reclaimed on exit/reset/drop. External returned immutable receipts remain historical values and cannot authorize a new attempt. Host-held copies are measured separately from engine-owned retention; the engine cannot bound arbitrary external cloning of immutable return values.

## 3. Actual facts and strike/segment representation

`RuntimeFactSource` preserves independently optional actor identity, equipment provider snapshot, skill/cast source and authored effect source. It is a flat bounded record, not recursive parent history. The common execution context carries that source, optional originating action identity and optional finite delivery ticket. A source can outlive an action record without pinning its mutable result or acquiring a retention reservation. Unknown root/origin/copy is explicit absence.

Use separate typed facts for Damage, Heal, Cost/resource change and Death. Each actual mutation records checked fact ordinal, time, target/resource owner, source and actual before/after values. Requested amount is separate from effective HP/resource change. A zero actual result remains a real result; a rejected request is not a zero-success fact. Record signed Heal/negative resource modifiers faithfully rather than reclassifying them as new damage/cost gameplay. Actual resonance reset and tactical payment originate at their real owners.

A `StrikeScope` is local to one basic strike or one skill delivery iteration/target occurrence. It maps target IDs to checked strike ordinals before their first formula evaluation and tracks the next component/segment ordinal and whether that strike has already produced positive HP loss. `build_skill_step_commands` currently runs effect-major, then target-major; preserve that order. Repeated damage effects in one iteration targeting the same unit share the strike; a new repeat, projectile impact, area tick, target or cast does not. Bonus damage components of one basic strike share its strike. Nested triggered ability invocation receives its own action/strike scope and restores its caller's scope afterward.

The existing damage formula applies combined component damage in one HP mutation. Do not invent a per-component share of overkill HP loss. Preserve each component's requested/calculated damage, type, source and segment ordinal; the actual HP transition belongs to the mutation that contains those components. Multiple damage mutations in one strike share strike identity but retain their own actual transitions. Existing `CommandExecutionSummary.actual_damage_target_count` and previous-step HP-loss gates continue to use the old calculations; new distinct-strike summary counters do not change that gate.

`FactSummary` keeps checked counts/totals and bounded latest/latest-nonzero witnesses for each actual fact kind, including the last relevant strike/segment source and predeath state. Actor and provider summaries make distinct actual equipment copies observable even when their effects execute in the same operation. A current strike may retain bounded segment calculation details until its summary is finalized; do not append every past mutation/event to an attempt history. Promised action receipts retain their compact aggregate and bounded witnesses, not an unbounded list of effects.

Predeath preparation captures the target's identity, position, pre-lethal HP and relevant currently active buff keys/application sources before any cleanup. Capture and native/authoring admission precede the lethal mutation. Finalization commits the mandatory Death fact and wave bookkeeping exactly once; optional OnDeath/OnKill/ally/proc dispatch remains separate. Damage, signed Heal/stat-HP lethal and direct summon Kill all enter this boundary; Despawn stays distinct. Preserve existing reaction order and sorted ally order, and do not run a new predeath reaction.

## 4. Source propagation and actual equipment copy

Introduce `BattleEquipmentBinding { base_uuid: Uuid, persistent_copy_uuid: Option<Uuid> }` as the element of `BattleUnitDraft.equipped_items`. Both real player draft producers in `combat_setup/player_spawns.rs:169,244` use the actual equipped inventory instance UUID. Both runtime constructors in `core/build.rs:139,263` copy it into `RuntimeItem`. Weapon/stat lookup reads `binding.base_uuid`. Keep `make_item_instance_id(base, side, unit, index)`, enhancement aggregation, cooldown keys and RNG identity unchanged.

`EquipmentSourceSnapshot { definition_uuid, runtime_item_instance_id, persistent_copy_uuid }` is copied while the provider is available. Scenario-only items use `None`; fragment identity remains absent. Carry this through `SourcedEffect`, `SourcedAbilityActivation` and the independent common source in `ActiveSkillCast.fact_source`. Legacy `SkillProcOrigin` retains its cooldown/narration identity; optional log/trace values contain no new action token or equipment-copy metadata. Update all nonempty draft/runtime-item literals and CloneShape visitors; do not guess a copy from base UUID, index or runtime UUID. Inventory writer/loader remain the existing durable identity authority.

| Boundary | Required concrete change |
| --- | --- |
| Item trigger collection and `TriggerOccurrencePlan` | Preserve ordered provider-bearing effect runs; keep effect-before-activation/current command order. Actor-only flattened batches lose the actual item. |
| `RuntimeCommandBatch` / `process_command_batches` | Add a scoped immutable common source; restore prior context across nested batches. Existing modifier source remains its existing numerical role. |
| `process_skill_commands` | Preserve known cast even if authored source is absent; replace common fact reliance on `source.zip(origin)`. Existing authoring/log adapter can still require both for its legacy value. Keep one strike scope across an iteration's effect-major command order. |
| TriggerAbility | Snapshot provider into proc/cast source before invocation; log event sequence remains narration only. |
| Damage launch snapshot | Preserve bonus-component provider separately from accumulated modifier values. Do not reconstruct every component as `CooldownSource::Unit` for fact provenance. Aggregate modifier contributors are bounded known contributor metadata, not a fabricated single provider or true-join/root. |
| Queued ApplyBuff | Carry common context and a finite application ticket regardless of authoring/log settings. |
| ActiveBuff / initial and repeated BuffTick | Retain actual application source, and carry the scheduling producer through the queue. Reapplication that keeps cadence does not retroactively rewrite an already queued tick's producer. No merged-contributor/root policy is inferred. |
| ScheduleAttack / AttackStart / AttackResolve / basic projectile | Carry immutable source and the explicit finite child ticket; automatic next attacks do not inherit that settlement obligation. |
| Skill projectile/area | Extend existing launch snapshot/context; released effects do not need a surviving caster or active cast record to identify their source. |

Queue metadata remains outside `BattleEvent` priority comparison. Private PackedEvent stores AttackStart and ContinuousMovementTick inline and larger events in admitted immutable Arcs. Both equality and ordering delegate to complete original event views; no metadata or pointer participates in priority. A source-free entry has no source allocation. Shared large payload/source unpacking performs native scan/copy and independent authoring admission before materialization; direct raw fixture dispatch remains allocation-free. Heap slots grow to exact demand; same-time buckets retain their prior geometric growth and array order with physical admission before allocation. Retained measurement includes slot capacity and each shared payload conservatively. No runtime causal path depends on optional observation being enabled.

## 5. Current carrier progress and finite tickets

Accepted is committed only after existing qualification and new native admission. A pending manual/auto focus owns the same action identity later transferred to invocation; do not allocate a second public action at CastEnd. Proc invocation admits its own action after current origin/target-policy checks. Preserve targetless directional qualification.

Finite tickets are checked monotonically allocated ordinals associated with an action token. The action owns the outstanding ID set. Closing a missing/already closed ticket does not decrement twice, recreate an action or overwrite a terminal receipt. No unbounded closed-ticket tombstone list is needed because IDs do not repeat within the attempt.

| Current event | Control milestone | Finite effect obligation |
| --- | --- | --- |
| Qualified manual/auto focus | Accepted; pending focus/locks/queue commit together | No Released before invocation. |
| Successful CastEnd or immediate proc invocation | Released; authored step controls become pending | Existing caster/target restrictions remain. |
| Condition-unsatisfied or zero-repeat step | Mark that authored control step skipped once | No synthetic hit or damage-gate result. |
| Instant / zero-duration area | Mark dispatched after all existing repeat commands finish | Wait for queued ApplyBuff/explicit ExtraAttack children, not independent status lifetime. |
| Projectile spawn | Mark step dispatched after all current launches | Carrier ticket survives control finish/cancel. Each queued impact has its own pending ticket so removal cannot settle before queued effect application. |
| Persistent area registration | Mark step dispatched | Carrier ticket closes on actual expiry/removal; pending tick/application tickets remain distinct. |
| ExtraAttack | Parent command is dispatched | Transfer one finite child ticket through existing attack start/resolve/projectile; failed/no-op child closes without successful hit. |
| All authored control steps dispatched/skipped, no deferred control | FlowFinished; write Finished terminal once | May still have released effects outstanding. |
| Control terminal and no outstanding finite tickets | Preserve original Finished/Cancelled/Failed terminal | EffectsSettled; retire internal record or fulfill its pre-reserved public result. |

Do not use `SkillStepProgress.pending_delivery_count` as the action settlement authority. It belongs to the existing damage gate. GF002: a piercing projectile can expire after earlier hits with no new final-interval impact and no old terminal gate callback. Close its actual carrier ticket, wait for already queued impacts/applications, and leave any still deferred control unresolved. Calling `resolve_skill_step_delivery` merely to repair observation is forbidden because it also releases deferred steps and cleans casts.

Step controls use bounded pending/dispatched bookkeeping keyed by authored step index; repeated/deferred event delivery cannot decrement twice. A t150 cancellation fixture needs a genuinely undispatched later step. A one-step projectile action may already have Finished at t100, and withdrawal at t150 cannot replace that terminal.

## 6. Existing cancellation and execution failure

| Cause | Preserved current scope and observation |
| --- | --- |
| Withdrawal | Pending focus and active cast control are removed; caster-owned Follow areas removed. Mark affected unfinished control Cancelled and close actually removed Follow tickets. Released projectile/GroundFixed continue. |
| HardCC | Pending focus/basic windup cancellation only; existing released step/projectile/area behavior remains. |
| InterruptCast | Pending-only/no-refund; existing locks/resource/released runtime remain. |
| Silence | Rejects new starts; does not retroactively cancel pending/released actions. |
| Death | Preserve actual movement/lifecycle/wave/buff/graveyard/trigger handling; no withdrawal-style cast/Follow cleanup. A matching pending CastEnd can invoke a dead-caster no-op and still reset resonance/recovery. Record failed/no-release control and actual Cost; preserve allow-dead OnDeath invocation. Existing queued steps/Follow processing stays as currently implemented. |
| Native/authoring limit fault | Abort the enclosing operation; no Accepted/receipt/partial state survives. This is not a committed action cancellation. |

Ordinary committed invocation inability has an explicit execution failure reason distinct from an admission/budget fault. Public qualification errors remain typed `GameError`; new native budget/counter failures use a structured `BattleRuntimeFault` variant/code with site/limit/requested values. Do not format a new allocated diagnostic after a failed allocation admission.

## 7. Public atomicity and native work accounting

Keep native first-fault/work budget in a synchronous operation scope outside cloned gameplay state. Nested internal calls join the active budget rather than resetting it. Route applicable existing `control::charge`/`poll`/reject calls to this separate native scope as well as the existing authoring scope; do not redefine `control::is_active()` or activate authoring-only behavior for ordinary native callers. First fault is returned at the public commit boundary even when an infallible producer used an early return. A failed public call ends its native scope; retry starts a fresh recoverable operation.

Cardinality/retention/source-byte counters live in `RuntimeFactState`, participate in rollback, and are checked before new allocation. Authoring-only early guards in `admit_authoring_runtime_replacement` and serialized accounting do not automatically become native checks; new state has its own maintained admission counters. Existing authoring cancellation/typed failure behavior remains intact.

| Public boundary | Atomic implementation |
| --- | --- |
| start | Clone before reset/startup, then commit core and return initialized execution state. |
| apply_live_command / source-command wrapper | One candidate includes narration-source setup, existing deploy/withdraw/manual/tactical work, new facts and final poll. |
| step one timestamp | Stage both core and `BattleExecutionState` before clock update/bucket pop. Commit together after all same-time work/hooks/terminal checks. |
| step until/by | One operation budget; stage each timestamp and preserve earlier successful timestamps when a later one fails. Include the no-event clock-advance branch. |
| finalize | Prepare result/new summary first, then commit finalized state; no premature finalized flag on failure. |
| direct public stance/relocation/swap | Same staged protection around existing implementation and later reconciliation/logging/queue work. Live-command dispatcher calls inner methods to avoid duplicate copies. |
| outer GameCore application | Preserve its existing full world candidate commit, including tactical signals, focus restoration, roster/attempt reconciliation and DTO construction. It may discard its whole failed world request even though a bare multi-timestamp core call preserves its earlier timestamp prefix. |

Add narrowly scoped crate-private `*_in_candidate` ports for existing world rollback owners. `world/application.rs:185–206` already clones and commits once; current world combat step/live callers and fresh uninstalled battle startup can use those ports. An authoring budget being active does not prove a receiver is staged. Preserve local Spawn Kernel/summon/boss rollback copies that recover rejected child operations while the enclosing gameplay continues. Price new copied fact state at each actual local copy site. Preserve Spawn Kernel request reservation before its existing snapshot.

Prepare authoring event-log copy-on-write on staged candidates using the existing `prepare_authoring_write` path. Native history detaches lazily with bounded geometric spare (initial186, absolute8,192 entries). A native step-until call may reuse its preceding candidate units buffer only with matching unsaturated insertion lineage, length and key iteration; callbacks/authoring disable that buffer. It never survives a public-call boundary. Every field is copied exhaustively, including values whose reusable strings/range rows avoid allocation. Native maps are never rebuilt/sorted or reseeded to manufacture a storage proof. Arbitrary effects on external captured state inside user-supplied FnOnce setup callbacks are not BattleCore-owned rollback state.

## 8. Native bounds, clone contribution and retained allocation

Use finite trusted-host `RuntimeFactLimits`; data cannot select unlimited values. Initial engineering ceilings mirror the scale of existing authoring limits: active actions4096, public result reservations4096, outstanding finite deliveries8192, actor summaries4096, provider summaries16384, authored control steps4096 per action, current strike segments8192, captured predeath buff entries4096 and newly owned retained source/result payload8MiB. Native operation work starts at10,000,000 and new-state clone slots at1,048,576. These are safety parameters, not balance rules; validate their actual high-water/latency/allocator fit before completion and lower test configurations exercise inclusive bounds/one-over failures. No zero/unlimited fallback. Any measured adjustment stays in this engineering owner and is recorded with its evidence.

Charge admission before allocating maps/sets/current-strike buffers/source objects/predeath snapshots/public receipts. Count public reservations while active as well as completed, and keep exact used cardinality/byte totals with checked arithmetic. A new immutable source object's payload is admitted separately; `Arc<T>` CloneShape charging one handle does not price the pointee. Current immutable compiled source metadata may be shared without copying strings, but its newly retained references/payload extent remain part of the new-state accounting. Replace old bounded latest witnesses only after preparing/admitting the replacement and update byte totals exactly.

Actor summaries use ordered exact-capacity vector rows; insertion shift work and reserve occur only after admission. Shared actor, active and provider storage detaches only for actual writes. Physical B-tree bounds include eleven key/value slots, twelve child pointers and a conservative header per node; at most1+(len-1)/5 nodes are charged, including one possibly empty root. Active and promised-result tree growth is reserved at acceptance, independently of later settlement; take releases its reserved structural delta. Child sets retain a conservative empty-root bound, and empty unique action trees release their allocation at publication. New containers and active/reserved result state have exhaustive CloneShape visitors. Native clone preflight visits RuntimeFactState and StrikeScope with the existing closed CloneShape ownership rules, pricing shared Arc handles without charging their immutable pointees as repeated deep copies; introduced carrier metadata retains a separate conservative extent contribution. Full newly retained payload admission remains independent. The native budget seeds then-current retention once per public operation and admits cumulative positive growth without refunds for candidate discard or replacement, in addition to the current live 8MiB ledger. Native preflight bounds the newly introduced state and its copied/retained contribution; the existing authoring closed-construction visitor remains unchanged in meaning. `BattleCore.units` is a legacy public HashMap: its private history only records `insert_runtime_unit`, and public callers can mutate allocation history outside that path. Neither `len()` nor current `capacity()` proves arbitrary deletion-churn clone storage. This design does not claim a hard total clone bound for arbitrary externally mutated legacy maps, does not add a new rejection/ownership policy for them, and does not reseed their history. Preserve ordinary native Clone staging, prove finite new-state contribution, and measure integrated total costs for actual supported production inputs. Current opaque authoring construction retains its existing full storage-proof boundary.

Do not perform a full retained-result/past-action scan on every admission. Maintained new-state cardinalities are O(1) prechecks; clone traversal remains proportional to currently owned active/promised results and bounded summaries. Long automatic execution must not grow completed-action storage with total lifetime action count.

## 9. Direct consumers, lifecycle and durable applicability

Always-on BattleCore facts are the authority. Extend current world battle observation and authoring `MandatoryObservation`/view/trial receipt projections with bounded fact/action/source summaries. Preserve existing R03 counters/source-byte receipts and optional trace controls. The default-release CLI obtains the same projection through the existing authoring API, with truthful describe/explain support; it does not reconstruct facts from logs or accept a private fixture-only success path.

Attempt creation/reset initializes fresh token/counters; same-attempt live clone/reconnect sees the copied state. Retreat entry snapshot restoration can reuse gameplay UUIDs, but reentry constructs a fresh attempt token. Result finalization projects finite summaries before authoritative attempt state is dropped. Existing V6 save/load does not serialize an active battle attempt; prove that N/A with the actual writer/loader. Existing inventory persistent-copy UUIDs roundtrip unchanged. Do not add a save migration or defer ownership to R20.

Update Core Runtime Contract with actual fact/source/strike, handle reservation/take, progress/cause-specific cancellation, rollback/clone-relative validity, native-bound scope and save applicability. Update Skill Target Contract with actual dispatch/settlement distinction and preserved gates/carrier lifetime. Update Data Loading Contract for real paired equipment binding and current public output. Do not change rulebook gameplay meanings or claim later-R features.

## 10. Focused implementation and verification order

1. Paired equipment binding and provider snapshots across both real draft/build paths, stat/weapon readers and all affected literals/clone shapes.
2. Common source/fact/action store plus native admission/control types; connect all current source/mutation/queue/carrier producers before exposing completed support.
3. Bare staged operations, narrow world candidate ports, explicit native handle request/query/take and bounded current observer/public/CLI projections; integrate all introduced state into lifecycle/clone/result/save applicability.
4. Focused V01–10 tests and literal external RON/API/CLI fixtures, then final required full core/all-target/default-release/current predecessor checks and measured new-state cost. Canonical contracts change alongside real support.
5. Freeze stable implementation/evidence and stop mutation before a separate fresh GEX-20 reviewer. This design's helpers are not completion auditors.

Concrete additional checks within existing IVA ownership: component-vs-strike overkill/overheal/signed-Heal; actual resonance/tactical pay and rejection; predeath victim buff provider A vs killer B, mandatory death/wave once with optional reactions suppressed; two actual same-definition equipment copies; log/diag off/eviction and pack/container-order determinism; real t0/100/300 and cancellation50/150; dead pending CastEnd actual resource reset without false release; postdeath queued/Follow behavior; piercing earlier-hit/empty-final-expiry with pending impacts and unresolved deferred control; duplicate/stale token and escaped aborted allocation with reused ordinal; cloned authority branch differences; failed native/authoring request/timestamp/take rollback; public reservations survive later actions, completion needs no new slot, take→retry reuses capacity; long automatic workloads retire completed internal records; attempt exit versus externally held immutable projection memory; actual V6/inventory roundtrip.

## 11. Preparation status

Source producer/caller/clone/lifecycle boundaries were read by root and bounded same-model read-only helpers. Current upstream correction and explicit GEX successor intake are closed. Representation, progress, cancellation, atomicity, retention, direct consumer and validation ownership are specified above. Implementation may refine private spelling/layout while preserving these meanings and current acceptance. Engineering ceilings require the planned actual measurements; no new runtime result is claimed here. Any newly discovered false upstream premise or material observable meaning returns to its owner before adopting it.


## 12. Implementation refinements under the same acceptance

EXP007 separates arbitrary `ResourceChangeFact` from actual `CostFact`; negative modifiers therefore never increment consumed-cost counts. The separate summary retains the signed actual change and its latest witness. Actual world deploy is included alongside relocation/swap. Existing tactical live signals carry source/time and a finite delivery until the real world consumer applies/clamps the resource and records it; merely draining a signal is not an applied fact.

The action store retains one latest immutable settled action for the attempt and per actor, with explicit extent admission, so an unreserved completed action remains observable after its mutable working record retires. This does not create a promised result reservation or an action history. Borrowed `RuntimeFactView` serializes bounded ordered rows; owned `RuntimeFactSnapshot` is used only where terminal/native result ownership requires it. Default-release CoreApplication and existing authoring/CLI receipts consume these observations. The accepted unreserved world ActivateSkill contract is unchanged; no new retained world request API is required. Native handle request/query/take remain the existing direct BattleCore port design.


Private movement state shares its immutable backend across candidates. The backend never inserts joints; its empty joint collections are created only at the two existing body-removal calls and cleared on reset. Existing conservative fixed clone charges remain unchanged. Targeting borrows authored TileRangePattern rows, keeps owned fallback defaults, and resolves delivery without constructing a range; membership traverses the original complete validated pattern without materializing/sorting a temporary tile list. These changes preserve original targeting, overflow/error, collision and queue ordering semantics. EXP013 onward records failed and successful immutable original guards; only the final current validation and fresh audit may close the Goal.
