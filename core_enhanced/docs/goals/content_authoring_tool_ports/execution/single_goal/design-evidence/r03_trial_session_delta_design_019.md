# R03 TrialSession connection delta — bounded design assistance 019

This is a source/design helper report under the delegated AUTORUN §8 scope, not a completion audit, implementation, policy decision, or test result. Only the two new /tmp reports were written. Root owns production changes and validation. The comparison baseline is the existing trial_port_design_003.md and the source snapshots listed below; it does not rewrite that design. Full observer/semantic-occurrence design is deliberately outside this slice.

## 1. Existing seams and the remaining aggregate

Several older gaps now have real source implementations: CoreApplication::fork_for_authoring is a fallible controlled clone (application.rs:790–794); command/assertion resolution already enforces the checked scenario's limits and global source-index rules (resolve.rs:131–313); encode_receipt already makes its usage stamp include mandatory encoding and prefix-rewrite work (output.rs:70–126). The inspected authoring/mod.rs baseline exported grammar and describe/explain, with no Session export. A later mod.rs hash drift is unreviewed; this report makes no claim about the newly added source.

Root's selected next slice is coherent with these types: a **private** trial.rs initialization/one-step boundary, using all four real step ports. Public prepare/simulate must wait for actual battle-entry/time-zero/terminal production. CoreApplication is private-owned, not publicly clonable/importable through this layer.

The accepted aggregate must own or retain together:

- CoreApplication, including its current generation, runtime/RNG/UUID state.
- NativeStorageProvenance for that exact application lineage.
- The checked TrialScenario (or an immutable Arc to it), validated attempt/session limits, next global step index, accepted copied-byte/work/receipt-byte/clock counters.
- Immutable accepted records containing resolved command input/context when relevant, the actual public-result Value when present, assertion observation or clock-none outcome, and the committed receipt document.
- Root's Vec<Arc<AcceptedStep>> prefix. It admits vector storage and Arc handle copies before native copying, preserving old records without repeatedly cloning their JSON and receipt Strings.

TrialScenario owns its private parsed document and is not Clone (scenario.rs:188–263); public TrialStep construction alone must not bypass checked scenario ownership. AuthoringContent also exposes only immutable token/validation-usage accessors (mod.rs:87–98). ValidatedGameData is an Arc-backed immutable generation handle (game/data/mod.rs:229–285); no second content database or raw pack copy is required.

## 2. Initialization scope and candidate B lifetime

CoreApplication::new(token, run_seed) is infallible and delegates to GameCore::new (application.rs:797–803; world.rs:99–110). It must run **inside** run_with_provenance with NativeStorageProvenance::fresh(), following validated limits and the entry cancellation poll. Fresh provenance is justified by constructing this fresh application inside the scope; it must never be attached to an imported/existing application or a continuation of A (control.rs:155–171,215–254).

The initial state is not an active battle: GameCoreState::new sets not-started/default memory profile and empty run/battle state (world/state.rs:1816–1844). Root's planned initial DefaultMemory profile snapshot therefore uses the real public snapshot port, preserves its whole result, and creates an initial receipt inside this scope. This receipt does not invent a scenario step or shift the first preparation step's source index.

authoring::validate already opens its own acquisition control scope and separately unwraps its inner content result (mod.rs:102–150). The smallest initialization input is an already validated AuthoringContent; do not nest that public validate call inside the initialization scope. Its recorded validation usage remains distinguishable from new trial work. Combining acquisition and initialization into one attempt would require a separate private in-scope loader seam; it is unnecessary for the selected slice.

Preparing replacement B means: build local B with its token and fresh ledger, execute the checked preparation prefix through real public calls, and obtain actual B battle-entry/time-zero evidence before replacing A. All accepted intermediate B steps may advance **local B**; preparation failure or cancellation drops that whole local candidate and retains complete A: application/generation, RNG, UUID state, accepted input/history, receipt counters, and native provenance. Token validation success alone does not authorize a swap. Optional trace loss must remain gap-only; it cannot roll back mandatory work or alter gameplay/RNG.

## 3. One step: producer, consumer, and ownership

The resolver consumes prior accepted **whole public envelopes** as &[Option<&Value>], whose length must equal the current global index (resolve.rs:131–152,212–239). preparation_steps and steps share one index sequence (scenario.rs:256–263). The adapter must admit this reference Vec's storage before building it. Assert and a HostTick returning None occupy a None slot; absence must not compress indices or be forged into an ApplicationResult containing null. A real snapshot whose read_model is null remains a Some whole snapshot envelope.

| Checked step | Actual public operation | Candidate-owned acceptance data |
|---|---|---|
| Command | resolve_command; CoreApplication::execute(player_id, command, context) | preserved resolved document/context, actual ApplicationResult, candidate total_copy_bytes |
| Snapshot | CoreApplication::snapshot(player_id, kind) | actual ApplicationSnapshot envelope |
| Clock | advance_active_battle_by; advance_active_battle_for_host_tick; advance_active_battle_to_next_event_bucket | actual ApplicationResult, or explicit HostTick None; actual clock witness dependency below |
| Assert | resolve_assertion against admitted prior references | assertion observation, no bindable public-result slot |

Exact public signatures are in application.rs:837–966. NextEventBucket takes no after_seq parameter. Snapshot kinds remain the existing seven; this layer must not introduce fixture authority or a private command bypass.

ResolvedCommand contains ApplicationCommand, ApplicationExecutionContext, separately retained resolved_context: Value, and the candidate cumulative total_copy_bytes (resolve.rs:21–27,307–313). ApplicationCommand::into_domain consumes the stored command document (application.rs:211–217,865–884). Root's chosen **before-execute bounded encode then charged bounded parse** preserves the resolved command Value without an unmetered deep clone. The context Value can move into the pending record; the decoded context moves into execute. Copy quota changes become accepted only with the whole step. No re-resolution after execution.

ApplicationResult and ApplicationSnapshot serialize as their whole envelopes; their payload/read_model accessors alone are insufficient for public pointer parity (application.rs:323–418,686–726). A private bounded/charged encode-and-parse adapter is needed for retained Value conversion and its new parse allocations. A bounded final encoder alone does not prove the public API's preceding JSON construction allocations are bounded. AssertionObservation is not Serialize (resolve.rs:124–129); use a small serializable receipt projection or intentionally add the needed internal serialization implementation. This is a concrete adapter need, not a missing whole-session defect.

## 4. Transaction and final receipt sequence

run_with_provenance returns Result<(T, ExecutionUsage, NativeStorageProvenance), ExecutionFailure>, where T can itself be Result<PendingAccepted, AuthoringError> (control.rs:215–254). It does not flatten that inner result. Its final cancellation/sticky-failure poll follows the closure and precedes extraction of the accepted ledger/usage.

Required sequence:

1. Validate the **original** ExecutionLimits before clipping max_work to remaining accepted session work. Explicitly reject zero remaining work before creating a zero effective max_work. Otherwise a normal exhaustion would become invalid-limits, or clipping could conceal an invalid requested limit.
2. Enter a scope with A's copied provenance, fork A through fork_for_authoring, and construct a local pending aggregate. Initialization substitutes new + fresh provenance. Resolver phase guards already distinguish preparation/execution; Snapshot/Clock require the enclosing correct phase.
3. Admit prefix handles/reference views, execute the public operation, retain bounded inputs/results, perform mandatory admission and actual-clock checks, compute candidate counters, and prepare all owned receipt payload, accepted-record/Arc storage and vector capacity.
4. Call encode_receipt **after every other charged mandatory operation**. Its committed_bytes argument is the previously accepted receipt total. Move the returned document/total into already prepared storage; leave no result parsing, capacity growth, history cloning, counter overflow check, or charged operation after the usage stamp.
5. Let the control scope's final poll run. Flatten both result layers. Only outer success **and** inner success permit attaching the returned candidate provenance/usage and performing an infallible aggregate swap.

In schematic form: obtain (pending_result, usage, provenance) from the scope; propagate outer error; propagate pending_result's inner error; attach only those successful usage/provenance values; swap the complete aggregate. An outer Ok containing an application/domain Err must discard the candidate ledger and every candidate record/counter. The current public world/application transaction commits core state before the facade constructs its response (world/application.rs:174–212; application.rs:865–884), so its internal transaction cannot replace this authoring aggregate transaction.

ApplicationResult's AcceptedRecoveryPending is a successful public disposition for projection recovery, not automatically an inner domain error (application.rs:355–405). A sticky execution-control failure still takes precedence through the outer final poll. Keep this distinction when mapping public response semantics.

Root's work accounting resolves the receipt self-reference correctly: receipt payload contains prior_accepted_work, fixed-width header contains the final attempt usage; effective max_work is bounded by remaining session work. Consequently the accepted sum after scope return is infallible under the prior checked bound. No new charged work or fallible serialization follows encode_receipt (output.rs:70–126). Receipt failure/cancellation drops the pending candidate. Current outer control errors return no usage/provenance; do not manufacture zero failed-attempt usage or claim complete rejected-attempt accounting without an explicit additional seam.

If an AcceptedStep is Arc-owned, prepare its allocation while it is still unique; fill its receipt field by move before publishing into the already admitted prefix. Sharing it prematurely and later using Arc::make_mut would introduce another owned copy. Moving a prepared record/Arc and scalar counters into the final aggregate requires no additional deep allocation.

## 5. Entry/time-zero and terminal dependencies before public exposure

There is a real battle producer to connect: commit_planned_live_battle_start installs the active battle, enters InBattle, and invokes advance_active_battle_by_internal(0, true, 0) (world/combat.rs:636–689). CombatExecutor startup checks control around BattleCore construction/start (game/events/combat.rs:135–233). Therefore merely creating a CoreApplication, starting the run, or selecting a public event is not proof that B's actual trial battle preparation succeeded.

The resulting BattleAdvanced can carry setup and LiveBattleUpdateDto.server_battle_time_ms (world/combat.rs:1321–1352; behavior.rs:1980–1989). However terminal time-zero handling can remove active_battle immediately after producing the result (world/combat.rs:1393–1406,1454–1568). An in_battle-only readiness check rejects valid immediately terminal starts. Setup also has a recovery path (world/combat.rs:1664–1683), and AcceptedRecoveryPending may lose projection data; a setup-shaped JSON value alone is not a durable actual-entry identity.

Before exposing prepare/simulate, a minimal private per-operation witness must establish actual B entry/time-zero completion, terminal status, and actual consumed battle time before active state disappears. It must survive internal staged commit/rollback correctly and must distinguish loader validation execution from the real fresh trial. The full readonly observer/occurrence design remains a separate mandatory dependency; this report does not choose its representation or event semantics.

Scenario grammar caps requested clock deltas; NextEventBucket contributes no static delta (scenario.rs:437–455,520–545). HostTick can return None, and scaled runtime time or next-bucket jumps require actual before/after time (world/combat.rs:1454–1504,1598–1647). Existing grammar alone cannot prove the session's actual cumulative-clock bound. Actual clock admission must occur before the outer aggregate swap, including immediate-terminal transitions. This is another reason to keep the current initialization/step layer private until the corresponding witness/terminal producer is wired.

## 6. Validation seams proposed, not executed

Focused checks should compare complete A continuation before/after rejected command, mandatory receipt overflow, cancellation at final poll, and an outer-Ok/inner-Err path, including the provenance ledger and accepted input/result prefix. Exercise initialize's fresh-ledger scope, initial receipt without source-index shift, original-invalid limits despite clipping, zero remaining work, all four public step results including HostTick None, and resolver copied-byte rollback. Assert stamped receipt usage equals returned attempt usage, and that adding a step does not deep-clone old JSON/receipts. Public prepare acceptance additionally needs actual entry/time-zero, immediate terminal, and valid-load-but-trial-failure preservation cases. These are suggested oracles, not this helper's executions or a completion verdict.

## 7. Read method, hashes, and drift

Reads used Python Path.read_bytes plus UTF-8 numbered slices; SHA-256 was calculated from the exact complete bytes supplying each slice. Only listed spans were inspected, not every line of each full hashed file. JSON includes every read timestamp/span/version and the close rehash manifest. No Git command, runtime/build/test, or production write was performed.

Close rehash: 2026-09-10T11:09:10.062828+00:00. All inspected paths remained at their original hash except:
- control.rs changed 8230d912… → 1a15e8bd…; a fresh 90–149 read confirmed ExecutionLimits::validate is now pub(super), resolving the originally identified sibling visibility gap. Other control claims cite the earlier full-byte version and its inspected spans.
- mod.rs changed 3230dc02… → ac6470f5… by close. The new bytes were only rehashed, not reviewed; baseline export observations above are explicitly historical. Concurrent private trial implementation is outside this report.

1. `core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/design-evidence/trial_port_design_003.md`: read 1–85; 85–163; SHA-256 `3cc58bce2565c6e14e03ac7d4987caffcacc9ebec5224b4eb44cae6505308162`.
2. `core_enhanced/src/authoring/mod.rs`: read 1–156; SHA-256 `3230dc02dc8cf274359bceb886533bb4e41aeb079d2dffeda84c46d17c0679e7`.
3. `core_enhanced/src/authoring/scenario.rs`: read 1–264; 421–455, 520–545; SHA-256 `0152fd9a1f33ad5d061102da34b5bfc13f8e282b697f4e0030dcb741320da859`.
4. `core_enhanced/src/authoring/resolve.rs`: read 1–29, 124–318; 317–356; SHA-256 `cb5665565b5680d5f7e509c0630f315633e91596e77b6973a2f9c91ca1c01ef3`.
5. `core_enhanced/src/authoring/output.rs`: read 1–174; SHA-256 `d56c244833316a3049409cdd85bc8a203748f79d57b15788ebe484ed2b588e5e`.
6. `core_enhanced/src/application.rs`: read 144–239, 323–418, 781–966; 15–43, 686–726; SHA-256 `6e77a86f121fd092c83b68a6021e86a99c2d24d657fd8a7d18247f5c81e54f8e`.
7. `core_enhanced/src/authoring/control.rs`: read 150–309, 310–357, 415–449; 90–149; SHA-256 `8230d912809e601520f9c2acd7c4dadf0f67a186634309617bd8fa34b4173841` → `1a15e8bdd7618572950ddc69ad25b5cda674eebfecd0f544010abb3477b2fed8`.
8. `core_enhanced/src/game/world.rs`: read 50–110; SHA-256 `496b24ed05866ffc88baebaa0f7076ad46569af7b3386b7727d43bc922b72a91`.
9. `core_enhanced/src/game/world/state.rs`: read 1816–1844; SHA-256 `05c7ce0fbd02258e4f34f611e059f0bb8eb6d738947baeced9b425f8f9a11336`.
10. `core_enhanced/src/game/world/combat.rs`: read 636–689, 1360–1410, 1454–1570, 1598–1647; 1321–1352, 1642–1693; SHA-256 `c8e2c36f80a7c1f3a407fb661cc42a535b61512a1997e21d0de078a142c8d46b`.
11. `core_enhanced/src/game/events/combat.rs`: read 135–233; SHA-256 `d88719417886eea018ae08a66c750884f163051f160d0b9b21ece3c1eddffc66`.
12. `core_enhanced/src/game/behavior.rs`: read 1980–1989, 3400–3423; SHA-256 `c38657dd64aeae18fbb6a4998cfeb65cc45b015a897f379360236b2b4c3e2552`.
13. `core_enhanced/src/game/world/application.rs`: read 171–218; SHA-256 `a13098c116e6db6f2e19f390306b8b8e8915d9b3d03e4d7a425675cef404731b`.
14. `core_enhanced/src/game/data/mod.rs`: read 221–285; SHA-256 `cc1712c4a52b39470935d7efe6fed263e92a768481e8c5c283f6c4ba93f79f9e`.
