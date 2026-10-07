# R03 optional trace isolation — bounded source design 031

Status: source/design assistance under AUTORUN §8, not an implementation review or completion audit. Root remains the sole repository writer. This report makes no test/build/runtime-execution claim. It records the selected implementation direction from root as a proposal, not current implemented behavior.

Read window: 2026-09-10T12:34:57.537180+00:00–2026-09-10T12:46:12.829632+00:00. Close hash window: 2026-09-10T12:46:23.314791+00:00–2026-09-10T12:46:23.349772+00:00. Read method: read-only file bytes, SHA-256 of those exact complete bytes, then numbered selected lines decoded from the same bytes. Searches only located symbols. There are 26 captured reads over 25 paths. Existing reports were not rewritten.

Close drift: battle/core/authoring_observation.rs changed from d90d74984ce1eaf8d8334d6f78a3dddcf1213c66ff222aba3c3f9071c86109c0 to ffbe7c0338271d638d57b263ea1db9794c9623b5f4bd8f0f09cd450e62c9e87f. The latter was hash-only at close, not reviewed. Other captured paths matched. Findings below refer to captured versions and spans; they do not assess root's concurrent optional-trace implementation.

## Minimal owner and connection

Frozen Goal lines 56, 61–62 and 70 require optional overflow to create a gap without changing gameplay, mandatory admission/work/bindings/receipt, with actual occurrence distinct from log correlation. The existing mandatory witness already supplies the correct row producer: battle/core/authoring_observation.rs:225–258 commits counters and an ApplicationWitness after an actual HP/Stats mutation. Append an optional row **after** all existing mandatory assignments, borrowing the now-retained latest witness. Do not alter reserve_authoring_observation's charge, source-handle copies, counter logic, or return type. Optional observation returns locally or records a gap; no GameError, control::reject, mandatory poll, or mandatory resource admission.

Recommended small private types:

- TraceConfig: Copy, disabled or validated finite limits for retained rows, retained encoded bytes, per-row bytes and optional work per whole attempt.
- OptionalTraceState: Copy config/status/counters plus Option<Arc<String>> containing complete NDJSON rows. Lazy None avoids allocating a disabled/empty buffer. A Copy first-gap record holds the first omitted actual ordinal and a fixed reason enum outside the String.
- TraceAttemptGuard: operation-local scalar TLS allowance, outside every domain clone. It owns no rows, definitions, source handles or gameplay identity.
- Borrowed BattleTraceView: actual ObservationKey from the mandatory owner, &str encoded rows, retained row count/bytes and explicit gap/coverage metadata. The optional view is separate from every mandatory snapshot, result, receipt and binding source.

ApplicationWitness is already Serialize and its private serialize_source at lines 73–109 creates only borrowed stack views of authored source/cast/proc fields. Encode that exact witness rather than constructing a new Value, serializing raw database ownership, or reconstructing source from strings. Coverage is current HP/Stats application witnesses, including source=None where that is the actual producer result. It is not a claim to retain every cast, area, projectile or future effect event.

Rows need not repeat an entry key. Combat setup calls start_battle_execution at events/combat.rs:221–225 before the world binds the actual key at world/authoring_lifecycle.rs:85–120. Store actual witness ordinal/source then join the actual key in the borrowed active/terminal view. Do not generate another UUID/counter or reject a startup row merely because entry binding has not happened yet.

## Trusted configuration and attempt lifetime

TrialLimits currently has execution, max_total_work and max_receipt_bytes (trial.rs:18–23). Add the optional trusted configuration there, not to finite scenario JSON or input identity. Extend remaining's original-limit validation (35–64) before entering run_with_provenance. Explicitly define zero-limit behavior and finite hard ceilings in the owning implementation; this report invents no numeric defaults. Invalid trusted options may fail before construction. Valid optional limits, including exhaustion during execution, must not change mandatory acceptance.

Use one TraceAttemptGuard inside prepare_owned's existing control closure before initialize_in_scope (94–124), enclosing the whole preparation prefix and final readiness receipt. Use one inside advance's closure (300–331), before prepare_step and its application fork. Never reinstall per prefix step, BattleCore clone, boss/spawn rollback or row. Do not reset an already active optional allowance through nested installation: reuse it or record an optional gap, without producing a gameplay failure. RAII drop restores the prior TLS state and must not perform fallible mandatory work.

Root selected missing-scope => optional gap only. This is suitable for a privately enabled core used outside the intended attempt route: it must not silently obtain unlimited trace work or fail the mandatory observer. The TLS record contains scalars only; a domain rollback must not refund already spent copy/encoding work. A later accepted battle owner may still have a complete retained prefix if the exhausted local candidate was discarded, but its next attempted row must see the exhausted attempt allowance.

Concrete config route:

1. initialize_in_scope:246 passes validated Copy config to CoreApplication::new_for_authoring.
2. application.rs:839–843 forwards it to GameCore's private constructor (world.rs:120–124).
3. Store config in the existing private TrialObservation owner alongside last_terminal. The owner is not Serialize. Its complete CloneShape field list must be updated.
4. world/combat.rs:487–499 obtains Option<authoring config> from that owner. Replace events/combat.rs:142's bool with this Copy option. None preserves ordinary execution; Some(disabled trace) still enables mandatory observation.
5. The ordinary generated-setup wrapper currently passes false at events/combat.rs:125; it passes None. The authoring path enables mandatory observation and configures trace before start_battle_execution.
6. reset_runtime_state_for_battle (core/sim.rs:1630–1633) must preserve the chosen config while clearing the newly started battle's rows/gap. Native domain clones preserve existing rows/config; they are not resets.

Existing PreparedApplicationCommit takes/moves the entire TrialObservation at world/application.rs:128–137,180–229. State-transaction rollback copies/restores it at node_flow.rs:783–807; the test rollback owner also preserves it at active_run.rs:170–188,215–225. Keeping config within this existing owner avoids a separate unpropagated GameCore field. BattleCore's own field, constructor, exhaustive visitor and reset still require explicit changes.

## Native clone and mandatory accounting

BattleCore derives Clone (core/mod.rs:81) and has an exhaustive CloneShape owner at 175–234. Add a standalone OptionalTraceState field, present for both trace modes. Its native clone is Copy metadata plus at most one Arc handle. It must not eagerly copy the String or re-encode witnesses.

Root selected a custom CloneShape implementation: destructure every OptionalTraceState field without '..', and charge a **fixed** slot/work shape independent of enabled, rows, buffer presence and gap. Use a fixed conservative shape covering its bounded metadata/handle storage; document that it models native handle-copy ownership. Do not traverse retained text. Config and gap types should remain Copy with complete type/field guards.

The existing generic Option visitor (clone_shape.rs:107–114) visits an inner value only for Some; directly using it for an optional trace payload would make admission trace-dependent. Arc's visitor (152–157) prices only a handle, as desired, but the containing state's outer shape must remain fixed. Do not route optional detach cost through BattleCore::prepare_authoring_clone_write or controlled_clone's mandatory meter. The active writer's separate budget owns those actual text copies.

This establishes equality between enabled/disabled mandatory accounting in the new implementation. It does not imply that adding a fixed field has zero cost relative to a historical binary. No-scope ordinary gameplay continues without authoring admission, and a disabled trace has no encoded buffer.

## Bounded writer: two concrete traps

**String scan before writer callback.** Cargo.lock:2009–2021 pins serde_json 1.0.140. Its ser.rs:2077–2126 scans a str byte-by-byte for escapes before writing an unescaped trailing fragment. A bounded Write implementation alone does not bound that scan. Before invoking the serializer, spend optional work for the borrowed source string lengths using checked O(1) len lookups. The current variable string leaves are:

- EffectSource.source_file;
- site.action.definition_id and action_id;
- site.effect_id;
- field_path and origin_definition_id;
- EffectOrigin::Added.source_definition_id when present;
- cast.skill_id.

That is seven always-present authored/cast strings for a sourced witness, eight with Added. EffectSource/ActionSite/EffectSite and the enum fields are read at ability/authoring.rs:10–42; cast ownership at skill_source.rs:8–35. The serialized cause and cooldown origin are fixed Copy enums (event_log.rs:328–344 and cooldown.rs:7–11). Use exhaustive destructuring/matching to flag changes in this fixed source graph; do not silently rely on an unguarded numeric string count. Fixed enum field names/primitives have bounded formatting; actual emitted bytes are charged separately. No mandatory CloneMeter, output Writer, projection helper or Execution control should be called.

**COW and subsequent growth can copy the prefix twice.** Root proposes Arc<String> with detach len plus append-chunk cost precharged. That is necessary but, alone, insufficient for a blanket copy-work bound: detaching a shared String may allocate/copy its live prefix, then appending beyond the new capacity may copy that prefix again. Unique-owner growth also has a possible prefix copy.

Small alternatives are (a) precharge both possible detach and growth copies before their operations, or (b) on shared detach, build a fresh String after try_reserve_exact(old_len + chunk_len), then copy old text and the chunk once before publishing the new Arc. Later unique growth still needs its own precharge. The latter avoids guessing the post-clone capacity. Neither logical byte caps nor reserve_exact promise an exact allocator/OS heap bound. Allocation refusal that can be returned locally becomes an optional gap; do not claim recoverability from every process-wide allocation failure.

Suggested append order, with every failure confined to this optional state:

1. If disabled or already gapped, return in constant time. Check row-count arithmetic/cap and available total/per-row encoded bytes, including the newline.
2. Save original complete-prefix length. Precharge the fixed source scan and a bounded per-row operation allowance in the optional ledger.
3. Serialize the borrowed witness directly through an optional writer. Before each chunk: checked projected row/total lengths, optional emitted-byte work, and any imminent detach/growth copy allowance. Reserve before mutating. UTF-8 validation for String appends is local and its scan must be covered by the byte allowance.
4. Append the newline under the same checks. Only then increment retained rows and update last retained actual ordinal.
5. On any failure truncate the candidate's partial tail to the saved complete-prefix length and set the Copy first-gap marker; do not detach a still-shared, unmodified buffer solely to truncate to its existing length. Do not refund spent optional work. Never publish a partial JSON row.
6. After the first gap, never attempt more serialization or buffer growth. Later omitted count can be derived from the mandatory actual application count and retained prefix rather than maintained as another semantic identity.

A fixed reason enum can distinguish row/row-byte/total-byte/work limits, encoding, allocation, missing scope and checked-arithmetic failures. Error reporting itself should not allocate unbounded source strings. The optional writer must not produce a domain error after gameplay was already committed locally.

## Terminal retention and read-only output

TerminalCapsule currently derives Serialize and its entire value is included in TrialObservationView.last_terminal (world/authoring_observation.rs:17–43). Adding a normal serialized trace field here would immediately change mandatory receipt bytes/work at trial.rs:166–195 and 460–498. Root's selected **#[serde(skip)] trace field** plus a separate borrowed view is the minimal compatible route. Do not add a variable flag, gap, coverage label or trace-limit echo to either mandatory payload.

capture_authoring_terminal (78–122) validates the actual key/finished state, performs mandatory outcome-copy/owner admission, creates a private unique Arc, and takes the mandatory observer before publishing last_terminal. Add an allocation-free empty trace placeholder to that private capsule. After the mandatory take has succeeded, move the OptionalTraceState out of the active battle into the unique capsule without new encoding, cloning or optional admission. Terminal transition already calls capture before active_battle=None (world/combat.rs:1397–1412), outside the debug-only export block. Optional status is retained even if it contains a gap; trace failure never blocks terminal capture.

The separate borrowed view should expose active and last-terminal encoded prefixes, their actual key and honest HP/Stats-only coverage. It needs no Arc clone, mutable borrow, RNG, ledger read/write or Value construction. Optional transport encoding/export may fail separately without altering session acceptance. Keep trace out of AcceptedStep.public_result and every prior-result binding adapter.

This owner retains one active and the latest terminal trace. Replacing last_terminal releases the former unless an independent owner still holds it. Retreat/reset that removes an active battle does not manufacture a terminal capsule; an exited nonterminal trace history would require a separate deliberately owned extension. Do not claim all-battle archival retention. Copies during candidate staging can coexist with A; per-buffer bounds and separate attempt copy work limit that work, but do not establish a global process heap limit.

## Meaningful validation seams; no execution in this report

- Extend the actual HP/Stats fixture at core/commands.rs:2545–2588,2600–2649: same sourced 17-damage applications, source=None operations, repeated identity and native clone. Compare trace disabled/enabled with all ExecutionUsage fields, mandatory observation serialization, gameplay state/IDs/RNG continuation and next result identical.
- Exact row/byte/newline limits, escape-heavy and multibyte source text, a failed partial row, shared-prefix detach, unique growth, exhausted optional ledger and missing scope: each produces a complete prefix and honest gap, with the same mandatory result/receipt and unchanged A buffer.
- Use mandatory work/output limits that are just sufficient in the disabled case. Enabling trace and forcing its overflow must still succeed with byte-identical mandatory receipts, accepted work, copy quotas and public-result binding behavior.
- Extend actual terminal-owner fixtures at world/tests/combat.rs:12854–12895 to inspect moved trace after active removal and subsequent result completion. Check the terminal mandatory JSON remains exactly identical across trace modes, including immediate terminal and nonzero-time cases.
- Private Trial prepare/advance/replacement oracle: successful B publishes its own prefix; failed/cancelled/mandatory-receipt-rejected B leaves all A owners/trace/receipt/RNG unchanged. Optional overflow alone accepts B. Whole preparation shares one optional allowance; internal rollback cannot replenish it.
- Repeated borrowed trace queries do not change any next result or budget. Root-reported regression runs are not this helper's execution evidence. Performance/peak allocation measurements remain an implementation validation task; asynchronous wall-clock cancellation races are not a deterministic timing-equality claim.

## Read manifest

Each hash covers the complete file; ranges are the actual inspected lines. JSON records every read timestamp/version and close hash separately.

1. `core_enhanced/src/game/world/authoring_observation.rs` — lines 1–123; SHA-256 `6b7daeeb067414d92308d7e1477a6347bbaf894d57e381b15f854f0181f6fa4e`.
2. `core_enhanced/src/game/battle/core/authoring_observation.rs` — lines 1–259; SHA-256 `d90d74984ce1eaf8d8334d6f78a3dddcf1213c66ff222aba3c3f9071c86109c0`.
3. `core_enhanced/src/authoring/trial.rs` — lines 18–86, 93–201, 229–254, 455–515; SHA-256 `6c4f9335002ddf1111aa7e882a46e37aeffd445e5fadd0c657f24111ece55f0d`.
4. `core_enhanced/src/game/world/authoring_lifecycle.rs` — lines 85–128, 194–214; SHA-256 `544ee93dc3417d18e1c0ccf608d5cb55461a1d1e245d694e43df05ef7d3989d7`.
5. `core_enhanced/src/game/events/combat.rs` — lines 105–148, 216–236; SHA-256 `0e471b379338406b7ab5be92aa759f3a67effb3d6edb263075a04488a682ba32`.
6. `core_enhanced/src/game/world.rs` — lines 51–69, 108–128; SHA-256 `cf60b014bebf94d30011756daf69c07d94109ac1caadd98e54069ceff8dbb6b9`.
7. `core_enhanced/src/application.rs` — lines 835–872; SHA-256 `c7d67c4f842409d5ee34496a920aa1d8230cf996117773fac293028f115d2f5d`.
8. `core_enhanced/src/game/world/combat.rs` — lines 480–503, 1396–1418; SHA-256 `a8d10a2a4ad7e1338868abd1b7d9ae12e61b04fafaccd01a1a2b999f6c7dea85`.
9. `core_enhanced/src/game/battle/core/mod.rs` — lines 72–85, 126–143, 175–188, 224–234, 499–509, 310–334; SHA-256 `05cb798e42b9c2b38e00ce10dfc6f67a50b177f098bbe0fb50dd712d136f15d3`.
10. `core_enhanced/src/game/battle/core/sim.rs` — lines 1630–1637; SHA-256 `1061fdc3774f67504c9ba727602b03c0a16bc75ef8cc4f8b336693f07caefc3a`.
11. `core_enhanced/src/authoring/clone_shape.rs` — lines 17–35, 107–114, 152–157; SHA-256 `6e56d3990b14357c98663ace941fc44b6cde7c41667829b4dddec78e71adbc6b`.
12. `core_enhanced/src/authoring/output.rs` — lines 1–50, 68–125; SHA-256 `2951433ffb08b52568028b70485cc12992c3c172a064e44df4795b1a22c71ddb`.
13. `core_enhanced/src/authoring/trial.rs` — lines 300–359, 413–459; SHA-256 `6c4f9335002ddf1111aa7e882a46e37aeffd445e5fadd0c657f24111ece55f0d`.
14. `core_enhanced/src/game/world/node_flow.rs` — lines 783–807; SHA-256 `ec966a461ebb32b352a13e7422c7e14013290bc8790e751ac0af46efe1fd433d`.
15. `core_enhanced/src/game/world/active_run.rs` — lines 18–29, 170–188, 215–225; SHA-256 `4b90d0697662a9d3ea93707c3dbe0fe4c2f9ff1085b3790a97e6733f18a3fd4d`.
16. `core_enhanced/src/game/world/application.rs` — lines 128–137, 180–229; SHA-256 `15350bf76915a8ff981d16dc71c4cf242d2ace5eec7522cd48d079a30c3dada1`.
17. `core_enhanced/src/authoring/control.rs` — lines 155–179, 195–255; SHA-256 `5361b2eac90d518d17bedbc856b1b974489af9db839d7cb5bb59c677d7b263e9`.
18. `core_enhanced/docs/goals/content_authoring_tool_ports/P2G_50_GOAL_SET_001.ko.md` — lines 54–72; SHA-256 `bb0b69cfebceb17fd13374826e1dbb6ba6e38e711cb574acc29ca61ca2745f98`.
19. `Cargo.lock` — lines 2009–2021; SHA-256 `80eed115bdcf0aa881da5b6c1525e6f19a0d6d8e1101ee4b285840ab7b3a8f19`.
20. `/home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.140/src/ser.rs` — lines 182–194, 2077–2126; SHA-256 `5352ffeafb6397b4a837ca4d40b0353510788d445938f1bff96cde66d0467c1d`.
21. `core_enhanced/src/game/ability/authoring.rs` — lines 10–42; SHA-256 `c4ca67ceac4502c2c31c8c91a584e09dff3c87fd8976269d436dbddbc2e08e1a`.
22. `core_enhanced/src/game/battle/skill_source.rs` — lines 8–35; SHA-256 `798d5a96e4550bab80cda46a724f13990be5cffe73d6a3535c179f2d3a8d7d30`.
23. `core_enhanced/src/game/world/tests/combat.rs` — lines 12854–12895; SHA-256 `0af3821cce23bad88d0f9594377a65289b1bef8586c64975b23469a9568dbc62`.
24. `core_enhanced/src/game/battle/core/commands.rs` — lines 2545–2588, 2600–2649; SHA-256 `91239303eae0622ffc9151a759c6638a8005cefdbfab265df67e34accbfda984`.
25. `core_enhanced/src/game/battle/cooldown.rs` — lines 1–12; SHA-256 `fa401d4db5a2c6572a8ef404979303fe4c5b957e1aa3160f62befef12a3d55f4`.
26. `core_enhanced/src/game/battle/event_log.rs` — lines 318–344; SHA-256 `49ddbf7545b8939ab4ae2b3cd42a87e1b42fbe97c7dbdfec67201daf33b63619`.
