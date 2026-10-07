# R03 optional trace isolation — bounded implementation review 032

This is AUTORUN §8 read-only implementation assistance, **not a completion audit**. Root owns production changes and test execution. No repository/source/docs/Git mutation, build, test, or runtime probe was performed. Root reported INPUTS_077 and a running focused109; its outcome was neither read nor inferred.

**Concrete defect result:** no reproducible source-level on/off mandatory-isolation, COW, partial-row, cap, UTF-8, source-scan or TLS-lifetime defect was established in the inspected production paths. This is a limited source result, not acceptance of R03 or of later source changes. The unexecuted test/measurement obligations below are separate from defects.

Read window 2026-09-10T12:51:57.336410+00:00–2026-09-10T12:56:28.659949+00:00; final close 2026-09-10T12:56:43.290859+00:00. Method: read-only complete file bytes → SHA-256 → numbered spans from the same bytes. 31 captures cover 27 unique paths. A failed locator search for nonexistent ability/mod.rs was followed by the actual ability.rs; no content claim uses that missing path.

## Current source facts and assessment

### Mandatory ownership and accounting stay separate

ApplicationWitness::trace_scan_work and the optional append do not call execution control. The actual commit hook (battle/core/authoring_observation.rs:313–348) first updates mandatory counters, latest sourced nonzero observation, and latest witness; only then borrows that witness into OptionalTrace::record. No new source Arc clone is introduced by this call, and it returns unit. Optional failure records a gap locally without changing the earlier mandatory reservation, source ordering, counters or domain result.

BattleCore stores OptionalTrace as a separate field (core/mod.rs:130–133), includes it in its complete destructure/visitor (140–236), initializes it disabled (509–513), and native Clone retains the buffer Arc. Ordinary observation-disabled cores reach the existing early return before the optional hook. reset_runtime_state_for_battle preserves selected options when reinitializing an enabled observer (core/sim.rs:1630–1633), clearing rows only for the new battle runtime.

OptionalTrace::visit_clone_shape (trace.rs:138–157) destructures all five actual fields and checks their exact types, then unconditionally scan_storage(Some(12)). There is no enabled/count/length/gap branch and no pointee traversal. Thus this adds exactly 12 slots and 12 scan-work units, plus 12 copy-work units when the enclosing meter finishes, in either trace mode. The parent meter may of course include its other fields. These are logical owner-copy units, not byte-size or allocator measurements.

The reservation is conservative for a fixed record with Copy options/counters/gap and one optional Arc handle. The existing generic Option visitor is value-dependent (clone_shape.rs:107–114), but this state deliberately does not use that visitor for its inner byte buffer or gap. The Arc visitor retains only a handle (152–157). Actual options and gap are Copy; their current accesses require that property. Exact field/type guards will reject an added owner field or an ownership-type replacement at these boundaries. They do not prove the purity of arbitrary future custom Serialize code.

World TrialObservation has Copy trace_options and the existing last_terminal Arc. Its struct visitor includes both (world/authoring_observation.rs:40–48), independent of enabled/disabled options. GameCore's complete visitor carries this owner. PreparedApplicationCommit moves it (world/application.rs:128–137,180–229); with_state_transaction restores it on Err (node_flow.rs:783–807), and the test rollback holder also restores it (active_run.rs:170–188,215–225). No separate trace history is attached to AcceptedStep or prior public Value slots.

### Actual writer handles COW and complete-prefix retention

The implementation selected **Arc<Vec<u8>>**, not the Arc<String> draft discussed in design031. RowWriter::append (trace.rs:260–303) calculates checked projected encoded length and checks total and row limits before any write/allocation. Row length includes the final newline because it passes through the same writer. start is the original complete prefix length; writes only append until final failure cleanup, so next - start cannot underflow on this private path.

For shared buffers or insufficient capacity it constructs a fresh Vec, precharging requested capacity + live-prefix length + new chunk length + one owner unit in the optional ledger. It reserves before copying and publishes the new Arc only after the prefix and chunk are copied. Capacity is at least next and at most max_bytes. This removes the earlier Arc::make_mut followed by reallocation/double-copy concern. Unique sufficient-capacity appends charge the chunk first and do not allocate. Growth is explicitly accounted each time it copies the prefix.

If a cap, allocation or writer error occurs, record truncates only a unique candidate buffer to start. A shared unmodified prefix requires no detach or truncation. Counts and last_retained_ordinal advance only after serialization and newline both succeed (234–248). Partial JSON and a missing final newline therefore do not escape through view(). On first failure the independent Copy gap records first_ordinal/reason/dropped_rows=1. Later rows stop before scan/encoding/allocation and only checked-increment dropped_rows; counter overflow is reported by a flag without feeding semantic counters (192–218).

There is no String conversion/UTF-8 validation copy in this implementation. The private writer is fed by the fixed serde_json witness serializer, and view borrows complete encoded bytes. The source strings are valid Rust strings; serde escapes them, and truncation returns to a prior whole-row boundary. This supports valid complete NDJSON rows without an unsafe UTF-8 conversion. It does not make an arbitrary future caller's raw bytes valid.

Allocation capacity is a request, not an exact process-heap measurement. The Vec's actual allocator capacity can differ from the request, and the returned encoding error uses a fixed io::Error payload. These bounded implementation details are not a demonstrated on/off acceptance defect; the work model should not be presented as a proof of recovery from every global allocation failure.

### Source scan graph is complete for current types

trace_scan_work (core/authoring_observation.rs:73–150) charges a fixed 256-unit structural allowance plus checked lengths before serde begins. The seven base string leaves exactly match the current serialize_source representation: source_file; action definition_id/action_id; effect_id; field_path; origin_definition_id; cast skill_id. EffectOrigin::Added adds the eighth string. SkillId is transparently serialized over String (ability.rs:9–19). There is no missing current owned string under the inspected effect/cast/proc graph.

The function exhaustively destructures ApplicationWitness, SkillEffectExecutionSource, EffectSource, EffectSite, ActionSite, SkillCastExecutionSource and optional SkillProcOrigin, and exhaustively matches EffectOrigin. Other actual fields are constrained Copy. Current UnitStats/StatModifier have only fixed numeric/enumerated fields; CooldownSource and BattleEventCause are fixed Copy enum payloads; UnitInstanceId is a transparent UUID. serialize_source itself makes borrowed stack views, not owned source copies or a database walk.

Cargo.lock pins serde_json 1.0.140. Its ser.rs:2081–2124 does scan a string for escaping before writing a final unescaped fragment. The new length precharge occurs before that scan and does not use mandatory control. Output-byte and possible buffer-copy work are charged separately. The numeric 256 is an implementation work allowance, not a measured CPU-time promise; future custom Serialize changes still require source review even when the source type remains Copy.

### Trusted options, scope lifetime and ordinary behavior

TraceOptions validates positive rows <=8192, total bytes <=8 MiB, row bytes <=256 KiB and <=total bytes, and attempt work <=2^30 (trace.rs:35–44). Disabled None is valid. TrialLimits::remaining checks original execution and trace options before clipping mandatory remaining work and before scope installation (trial.rs:37–67). Invalid trusted limits therefore fail before construction or mutation. Trace option types/module/setters remain crate-private; no new external mutable authoring authority is exposed in the inspected mod exports.

prepare_owned installs one TraceAttempt before the entire control::run_with_provenance call (trial.rs:97–127); advance does the same before its single step (307–339). The test-only initial constructor also follows this pattern (206–232). This placement covers constructor, all preparation commands, time zero, receipt encoding, and the final mandatory acceptance poll. There is no per-prefix-step reset. The guard remains outside all gameplay drafts/rollback owners and releases on ordinary return or unwinding.

TLS stores only Copy limit/usage scalars (trace.rs:53–107). The guard is !Send/!Sync through its Rc phantom, so it cannot be dropped on another thread. Nested enter fails without replacing the existing budget and therefore creates no second guard that could clear it. Both trace-disabled and trace-enabled trials install a guard; the nested-owner diagnostic is an invariant error on either mode, not a trace-exhaustion branch. Missing scope in record instead produces MissingAttempt and a gap, leaving gameplay unchanged. An optional WorkLimit sets only optional exhaustion; candidate drop does not refund it.

Initialization constructs the authoring application, then sets options before profile projection or command execution (trial.rs:247–256). The current setter only writes the authoring owner; world setup forwards owner.trace_options as Some for authoring, whereas ordinary setup passes None (world/combat.rs:487–502; events/combat.rs:113–145). It enables observation/trace before start_battle_execution (221–225). This is consistent with current callers. A future use of the private setter during an already running battle would change only future setup configuration; it must not silently become a public live-trace toggle.

### Terminal, receipt and borrowed view

TerminalCapsule.trace is explicitly serde(skip) (world/authoring_observation.rs:21–38). Mandatory TrialObservationView remains active mandatory + terminal capsule. Both Ready and StepPayload still serialize only that mandatory view (trial.rs:145–203,466–523). No optional flags, lengths, gap, limits or bytes were added to mandatory receipts or binding inputs.

capture_authoring_terminal completes mandatory validation/admission, creates the unique unpublished capsule, takes the mandatory observer, then mem::take moves trace into it (121–166). No optional encode, buffer clone or charge occurs on this move. The caller captures before active_battle=None and outside the debug-export cfg block (world/combat.rs:1399–1414). The immutable capsule survives active removal through the existing terminal owner.

authoring_trace first validates the mandatory active owner/key and returns only borrowed TraceView values with actual ObservationKey (world/authoring_observation.rs:73–90; application.rs:852–859). It neither constructs Value nor detaches/mutates/charges. Startup rows need no fake key: actual witness ordinal is encoded and the bound key is supplied by the enclosing view. Retention remains active plus last terminal, not an archival history of every exited battle. The row channels describe current HP/Stats coverage; public view/CLI naming must preserve that scope.

## Test support read; separate outstanding validation

The read test sources are useful support, not a verdict:

- trace.rs:348–475 cover exact row/total bytes, final-newline rollback, shared-prefix isolation, row gap, work-limit boundary, candidate-drop nonrefund, missing scope, nested enter and invalid options.
- core/commands.rs:2653–2745 apply actual sourced damage twice, compare native clone and mandatory ExecutionUsage/observation/event log across off/on/work-gap, and inspect retained ordinal/source.
- trial.rs:682–775 compare whole receipt documents, public results and accepted/copy/receipt counters over an actual prepared trial until a real HP/Stats mutation. The fixture requires nonzero applications; it does not require reaching terminal.

Bounded follow-up oracles not established by these inspected fixtures: escaped/multibyte and Added/proc source graph at exact scan-work/row-byte limits; populated-prefix COW failure at the precise copy-work threshold (including retaining A); actual terminal move and subsequent completion with on/off byte-identical mandatory receipts; a mandatory work/receipt budget just sufficient with optional overflow; observation/source reset and repeated read-only trace access. These are gaps in this helper's inspected execution evidence, not assertions that no other root test exists or that production is defective. Root is independently running/adding tests.

## Drift and manifest

First close detected changes to trace.rs and trial.rs. Their relevant production ranges were re-read from the new bytes and compared exactly with the original captured lines: trace.rs:1–319 unchanged; trial.rs:18–140,145–232,234–270,307–343,466–523,528–539 unchanged. Later test additions were not reviewed and no outcome was inferred. Final close matched every latest captured version. Thus production conclusions retain the stated scope; test-support discussion refers to the earlier captured tests.

1. `core_enhanced/src/authoring/trace.rs` — lines 1–477; SHA-256 `814828ba4995efe4899861105adea958cc088ab0abf596c80c4e63d614a9d690`; 2026-09-10T12:51:57.336410+00:00.
2. `core_enhanced/src/game/battle/core/authoring_observation.rs` — lines 1–349; SHA-256 `c06107c3469c202f60c7a2ed192b7f25ec9dc34ad60e9263895f3962ca0b9f00`; 2026-09-10T12:51:57.376761+00:00.
3. `core_enhanced/src/game/world/authoring_observation.rs` — lines 1–168; SHA-256 `a2781cf3d8d4d2ab1ae7a161aca07c98be2ca9f46cde4fafc37f51eedd97b3e1`; 2026-09-10T12:51:57.413323+00:00.
4. `core_enhanced/src/authoring/trial.rs` — lines 18–140, 229–270, 300–343, 525–542, 675–780; SHA-256 `2c380f98f8c63df2df41b6cc1f8910070febc902293de1ebf5f3bb519969c59b`; 2026-09-10T12:51:57.446180+00:00.
5. `core_enhanced/src/game/battle/core/mod.rs` — lines 72–143, 155–238, 498–520; SHA-256 `a2adab8d8a561c11b46629671df774c8c1f0ff9c414dc27480b386b7edd9ccd3`; 2026-09-10T12:52:35.420048+00:00.
6. `core_enhanced/src/game/battle/core/sim.rs` — lines 1628–1644; SHA-256 `b87b72a6455fa3286968c5edeffd57b36ba57442a66ae611a23eea9c65750e53`; 2026-09-10T12:52:35.472975+00:00.
7. `core_enhanced/src/application.rs` — lines 835–890; SHA-256 `7c024e9966a145569177670da15ef91303548783c23ed6917612e83f300a230a`; 2026-09-10T12:52:35.488194+00:00.
8. `core_enhanced/src/game/world.rs` — lines 51–69, 108–126; SHA-256 `cf60b014bebf94d30011756daf69c07d94109ac1caadd98e54069ceff8dbb6b9`; 2026-09-10T12:52:35.517149+00:00.
9. `core_enhanced/src/game/events/combat.rs` — lines 113–145, 216–236; SHA-256 `e898cd30482ec766c0b91de1551f57325b32cae15e43dd4c4e84f205cec29cba`; 2026-09-10T12:52:35.589490+00:00.
10. `core_enhanced/src/game/world/combat.rs` — lines 487–506, 1397–1416; SHA-256 `c32d4fb21715166a2f84c0d684cb817a218c23b666990f930887c2b0d9547e54`; 2026-09-10T12:52:35.597690+00:00.
11. `core_enhanced/src/game/world/application.rs` — lines 128–137, 180–229; SHA-256 `15350bf76915a8ff981d16dc71c4cf242d2ace5eec7522cd48d079a30c3dada1`; 2026-09-10T12:52:35.678129+00:00.
12. `core_enhanced/src/game/world/node_flow.rs` — lines 783–807; SHA-256 `ec966a461ebb32b352a13e7422c7e14013290bc8790e751ac0af46efe1fd433d`; 2026-09-10T12:52:35.650764+00:00.
13. `core_enhanced/src/game/world/active_run.rs` — lines 170–188, 215–225; SHA-256 `4b90d0697662a9d3ea93707c3dbe0fe4c2f9ff1085b3790a97e6733f18a3fd4d`; 2026-09-10T12:52:35.718567+00:00.
14. `core_enhanced/src/authoring/clone_shape.rs` — lines 107–114, 152–157, 319–351; SHA-256 `6e56d3990b14357c98663ace941fc44b6cde7c41667829b4dddec78e71adbc6b`; 2026-09-10T12:52:35.748772+00:00.
15. `core_enhanced/src/authoring/control.rs` — lines 155–179, 195–255, 388–470; SHA-256 `5361b2eac90d518d17bedbc856b1b974489af9db839d7cb5bb59c677d7b263e9`; 2026-09-10T12:52:35.787301+00:00.
16. `core_enhanced/src/authoring/trial.rs` — lines 145–232, 466–523; SHA-256 `2c380f98f8c63df2df41b6cc1f8910070febc902293de1ebf5f3bb519969c59b`; 2026-09-10T12:52:35.766937+00:00.
17. `core_enhanced/src/game/ability.rs` — lines 1–23; SHA-256 `c7cec94493e6de6d2ebb005941f3f80a276396e3803afb07f5d5a884d65ab195`; 2026-09-10T12:53:14.286040+00:00.
18. `core_enhanced/src/game/ability/authoring.rs` — lines 10–42; SHA-256 `c4ca67ceac4502c2c31c8c91a584e09dff3c87fd8976269d436dbddbc2e08e1a`; 2026-09-10T12:53:14.290632+00:00.
19. `core_enhanced/src/game/battle/skill_source.rs` — lines 1–35; SHA-256 `798d5a96e4550bab80cda46a724f13990be5cffe73d6a3535c179f2d3a8d7d30`; 2026-09-10T12:53:14.325372+00:00.
20. `core_enhanced/src/game/battle/cooldown.rs` — lines 1–12; SHA-256 `fa401d4db5a2c6572a8ef404979303fe4c5b957e1aa3160f62befef12a3d55f4`; 2026-09-10T12:53:14.367594+00:00.
21. `core_enhanced/src/game/battle/event_log.rs` — lines 328–344; SHA-256 `49ddbf7545b8939ab4ae2b3cd42a87e1b42fbe97c7dbdfec67201daf33b63619`; 2026-09-10T12:53:14.376726+00:00.
22. `core_enhanced/src/game/stats.rs` — lines 170–278; SHA-256 `9c3d49b44c773288da84e35bc0411292e423583a4b481a1e044622c138d10269`; 2026-09-10T12:53:14.413677+00:00.
23. `core_enhanced/src/game/battle/core/commands.rs` — lines 2650–2775; SHA-256 `2ec3833b9e01d8aa2828d86929190cd949df45b6d538fb1718cb3bb58eeef528`; 2026-09-10T12:53:14.480895+00:00.
24. `core_enhanced/src/authoring/mod.rs` — lines 1–30; SHA-256 `339376e1e7766e349a10c7e2e864ded3ef524d825f7e2962c00b9eecb370ceb3`; 2026-09-10T12:53:14.485349+00:00.
25. `core_enhanced/src/game/battle/core/mod.rs` — lines 140–156; SHA-256 `a2adab8d8a561c11b46629671df774c8c1f0ff9c414dc27480b386b7edd9ccd3`; 2026-09-10T12:55:28.486084+00:00.
26. `core_enhanced/src/game/battle/ids.rs` — lines 1–49; SHA-256 `063075a0f4ac754132949232bd9db555277b4828c43aaecc9a57fd7561069ac9`; 2026-09-10T12:55:28.525612+00:00.
27. `Cargo.lock` — lines 2009–2021; SHA-256 `80eed115bdcf0aa881da5b6c1525e6f19a0d6d8e1101ee4b285840ab7b3a8f19`; 2026-09-10T12:55:28.569086+00:00.
28. `/home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.140/src/ser.rs` — lines 182–194, 2077–2126; SHA-256 `5352ffeafb6397b4a837ca4d40b0353510788d445938f1bff96cde66d0467c1d`; 2026-09-10T12:55:28.608443+00:00.
29. `core_enhanced/docs/goals/content_authoring_tool_ports/P2G_50_GOAL_SET_001.ko.md` — lines 54–62, 70–72; SHA-256 `bb0b69cfebceb17fd13374826e1dbb6ba6e38e711cb574acc29ca61ca2745f98`; 2026-09-10T12:55:28.620314+00:00.
30. `core_enhanced/src/authoring/trace.rs` — lines 1–319 (reread identical); SHA-256 `24113aba868cd1093da5190097c901759b8e1e09271f20dd8eaeb0daebd01164`; 2026-09-10T12:56:28.658455+00:00.
31. `core_enhanced/src/authoring/trial.rs` — lines 18–140 (reread identical), 145–232 (reread identical), 234–270 (reread identical), 307–343 (reread identical), 466–523 (reread identical), 528–539 (reread identical); SHA-256 `fcc3c905f32e99257bc063496fafcbf725c07e761e454bc4a7d215140d76b9fb`; 2026-09-10T12:56:28.659949+00:00.
