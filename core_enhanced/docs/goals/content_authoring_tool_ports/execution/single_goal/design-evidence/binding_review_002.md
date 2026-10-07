# R03 private binding resolver — bounded read-only review 002

Role: source/design helper under AUTORUN §8, not a fresh completion auditor. Root remains the sole implementation/artifact writer. This helper ran no builds, tests, runtime, Git mutation or production edit. Only these new /tmp report files are written. Current source was changing while read; fingerprints and drift are recorded below. Prior reports remain unchanged.

The review found one concrete defect in the concurrently added assertion path. Root corrected it during this review, and the correction is visible in the final source snapshot. No additional concrete command-resolver defect was established in the requested scope. This is a bounded source conclusion, not a session, API/CLI, performance or R03 completion result.

## Criteria read

AUTORUN_REQUEST.ko.md 110–128 permits bounded delegated reading and separates implementation helpers from fresh audits. Frozen Goal 54–63, 90–104 and 124–132 owns public-core parity, bounded prior-result copies, complete candidate acceptance, failure preservation, opt-in control and optional-trace independence. PLAN detailed007/008 (232–263) owns immutable checked scenario limits, source/type validation, projected pre-copy shape/bytes, public command decoding, strict context decoding and quota adoption only with the accepted aggregate. No new policy is proposed here.

## BR02-F01 — malformed array source token was accepted by Exists(false)

Before the correction, source_at called scenario::at_pointer and returned Ok(None) for every lookup failure. at_pointer returned None for both missing entries and malformed array indices. resolve_assertion then accepted Exists(false) whenever the returned value was absent. For an accepted result {"array":[true]}, source "/array/01" with {"test":"exists","expected":false} therefore succeeded. Parsing the source pointer cannot reject "01" globally because an object can legitimately have that key; the actual container must decide. Command binding already converted this same None to an explicit error, so the defect was in the added assertion behavior.

The then-current assertion source test also expected "/array/01" to succeed. Root acknowledged that this was an incorrect oracle; the historic test receipt is not erased or treated as evidence for corrected behavior.

Root's corrected source snapshot is resolve.rs SHA 23119cb7eb3d193352db66bfe7d0ea31156cb435daed3bb4db00ca32b4e1c23b and scenario.rs SHA 0152fd9a1f33ad5d061102da34b5bfc13f8e282b697f4e0030dcb741320da859. source_at 108–114 explicitly rejects an invalid array_index before at_pointer. array_index 400–408 rejects empty, leading-zero, signed, non-ASCII and usize-overflow representations; valid representable out-of-range positions remain ordinary absence. Object keys retain literal lookup. Corrected test source 1046–1075 covers invalid-token negatives, canonical missing "/array/1", and object key "/01". This helper inspected that source but did not execute it.

The initial window fingerprints (resolve 823391e5… and scenario 77c27942…) preceded concurrent changes. They do not certify an atomic historical version of every live excerpt. The defect evidence is the actual live code read and root acknowledgment; the correction has the exact snapshot hashes above.

## Current producer → consumer and pre-copy path

1. ScenarioLimits::validate checks every positive caller ceiling against its default hard ceiling (scenario.rs 14–97). TrialScenario::from_json_slice applies raw byte/shape parsing, validates Document and clones the trusted limits into its private owner (187–265). Getters expose shared references, not mutable state; no Deserialize or public raw constructor can fabricate a checked TrialScenario.
2. validate_document checks combined chronological index, source eligibility and already-existing destinations. It forbids command/context root replacement and command/type descendants. Decoded destination token sorting rejects equal or ancestor paths (421–527). These checks remain trustworthy when resolve_command consumes the immutable scenario.
3. resolve_command requires active control and public_results.len() == current combined index, and rejects committed_copy_bytes above the stored total cap before subtraction (211–247). It has no mutable reference to scenario, source slots, application, RNG or committed quota.
4. source_at borrows the selected source slot. Pointer decoding and container lookup are charged before work; source availability, actual canonical array address, existence and declared JSON type are checked before any source subtree clone (78–120, 251–266). An empty pointer means source root. Escapes are decoded once by the shared bounded parser.
5. measure(value, None, stored_limits, per_step_cap) borrows each source and counts its encoded JSON bytes/nodes/depth. Each binding's encoded source size counts separately even when two bindings use the same source. Checked addition and subtraction enforce inclusive per-step and total-copy limits. Patch::insert retains references, not copied source Values (29–41, 267–284).
6. RequestView serializes a virtual {"command":…, "context":…} request using the patch trie (287–294, 405–523). Project counts each replacement root at the actual destination depth; it does not also count the replaced template subtree. Untouched template nodes and every inserted subtree share one aggregate node/byte budget. The wrapper is depth 0, its command/context children depth 1. This catches aggregate overflow that per-source checks alone cannot catch.
7. Counter checks each encoded byte chunk before incrementing; ShapeBudget checks depth and node admission before descent/increment. Bound failures stop virtual serialization and occur before command/context/source materialization (359–401, 524–563). Quotes, escaping and object keys are counted by serde_json's real serializer instead of an estimated string length.
8. Only after projection succeeds does the resolver precharge 3 × (encoded bytes + nodes), materialize the command/context, decode typed context, and call ApplicationCommand::from_document (295–312, 565–583). This keeps existing PlayerBehavior semantics and preserves ApplicationError rather than manufacturing a successful no-op (application.rs 151–165).
9. The returned total_copy_bytes is a candidate scalar. Any type, pointer, projection, fuel or decoder error drops locals and leaves the borrowed basis unchanged. There is no quota increment or input rewrite to undo inside the resolver.

The default engineering ceilings are 1 MiB resolved request, 1 MiB copied source per step, 8 MiB total copies, max depth 64 and max nodes 262144. These are finite logical/encoded-shape ceilings, not a measured allocator-byte or elapsed-time guarantee. Caller values remain stored, including narrower copy/shape ceilings; the resolver never silently replaces them with defaults.

## Context and public command parity

Context/Authority both deny unknown fields (resolve.rs 315–352). Missing battle_event_after_seq uses public default 0; explicit null fails u64 decoding. Missing/null source_command_id and expected_battle_authority become None. Present authority requires typed UUID and u64 revision. An incomplete placeholder can parse as a template but must become valid after bindings. Source declared type is only the binding JSON type; destination semantic type is deliberately owned by the final context/public command decoders.

The produced ApplicationExecutionContext has exactly the existing public fields (application.rs 219–232). execute_internal forwards them into existing domain execution (855–874). No private fixture authority or extra evaluator is introduced here. Resolved context preserves the actual JSON input, including omitted/default fields, separately from typed execution defaults, so receipt/replay can retain both meanings without normalization guesses.

The module exports TrialScenario and its read-only grammar types; resolve remains a private module and the prepared outputs pub(super) (authoring/mod.rs 3–15; resolve.rs 20–27). Public grammar structs can be independently constructed, but they cannot be inserted into a checked scenario through this API. The accepted-result adapter is still absent, so external API/CLI source-slot parity is not established by these private tests.

## Control and accounting: facts and limits

control::run installs one thread-local Attempt, rejects a nested replacement, inspects entry and acceptance, and removes the scope through a Drop guard (control.rs 159–205). inspect retains the first cancellation/work failure and charge checks addition and the inclusive max before incrementing (227–266, 332–346). resolve requires an active attempt, and even zero-length pointer charge(0) polls cancellation. Projection charges node traversal and encoded chunks; source object lookups charge token-length × container-width; patched object/array lookups precharge both projection and later materialization lookups. Equality assertions measure both borrowed values and precharge comparison; Exists/Type inspect the selected value without cloning a subtree.

Known bounded cost details are not hidden:
- Patch insertion itself has no extra charge inside BTreeMap::entry. Pointer-length work is already charged, and the grammar bounds it to at most 64 destinations × 32 segments, with 1024 pointer bytes each. That is a bounded bookkeeping region, not an unbounded content traversal. The exact allocation/comparison cost is unmeasured; a logical-work price is not an instruction counter.
- Materialize and public decoder cloning contain no internal cancellation poll. They are admitted by the full bounded projected shape and precharge. A cancellation arriving during this region is observed at outer control acceptance/final session poll; no wall-time stop deadline is proved here.
- Each source may be measured before discovering that the cumulative copy quota is exhausted. This consumes attempt-local logical work but makes no committed copy or quota mutation. Failed attempt work is not evidence that a source was accepted.
- The resolver's local phase variable labels Request/Application diagnostics. It does not change Attempt.phase. The session caller must install the proper TrialPreparation/TrialExecution phase guard, or a TLS-originated ExecutionFailure can carry the caller's wrong phase.
- Inner Result and outer control::run Result must be flattened in that order of authority: outer sticky cancellation/work failure wins. The source test helper demonstrates that shape at 605–616. The production session owner is not yet connected.

## Follow-up obligations, not present resolver defects

O01 — Accepted public source adapter: build the combined index history from actual accepted public command/snapshot/clock outputs only. Keep None for a successful step with no public result; failed steps never publish a slot; assertions/optional traces/receipts never become source values. Source existence and a caller-supplied slice length alone cannot certify origin. The private resolver intentionally relies on the forthcoming trusted adapter.

O02 — Aggregate acceptance: use a candidate application and candidate quota/input/receipt. Adopt returned quota only after actual command success, bounded mandatory receipt/output readiness and final cancellation poll, in the same aggregate commit. All mandatory failure paths preserve previous app/token/seed/RNG/input/result/quota/receipt. Optional trace overflow must only record a gap; it cannot change work, mandatory admission, commit, binding eligibility or RNG. The resolver contains no optional trace dependency.

O03 — Phase and operation lifetime: set actual TLS phase for preparation versus execution, preserve one budget across the operation rather than resetting it per binding/decoder, and ensure no return publishes a candidate before control acceptance. The resolver cannot enforce commit discipline in an absent caller.

O04 — Public parity and reproducibility: API and thin default-release CLI must share this parser/checked owner/decoder path, resolved templates/context and actual public result adapter. Dynamic authority/IDs must come from real public results. Root-reported private resolver tests are not standalone application/battle acceptance.

O05 — Cost and interruption evidence: measure the new shape/trie/materialize/decode and mandatory error/output path separately from optional observation. Boundary tests need exact byte/node/depth, repeated copies and session quota cap/cap+1, failed retry basis, and cancellation before final aggregate acceptance. Current source budgets are deterministic logical prices, not a measured latency or memory promise.

O06 — Clock ownership: scenario validates explicit delta and explicit delta sum; NextEventBucket contributes no input delta (scenario.rs 524–538). The actual clock adapter must account for public host scaling/pause/remainder/None and runtime event advancement with operation control and truthful receipt. This is still session integration, not a binding-resolver operation.

## Verification evidence boundaries

Root reported INPUTS028 resolver7 actualpass and check012 exit0 with 19 dead_code warnings while no session consumer exists. Root later reported INPUTS029 focused032 authoring31pass as historical evidence and retained it when the assertion oracle required correction. This helper did not run or recertify either record.

Source tests read include dynamic strings/authority and immutable basis (646–699), independent inclusive copy/request caps including escaped bytes (805–858), inserted depth and nodes (862–894), cancellation/tight fuel/retry (898–940), whole-request node total (944–967), and corrected assertion source cases (1040–1124). These are reviewed test definitions, not this helper's executions. Focused source coverage does not establish the missing session acceptance layer.

## Source identity and read method

Absolute root: /mnt/f/work/simulator. Preliminary sed/rg reads were live while root edited. A subsequent read captured each file's bytes, SHA256 and text together into functions memory, and numbered spans were inspected from that snapshot; no copied source file was written. At 2026-09-10T07:52:34.695465+00:00, final rehashes matched all eight snapshot hashes. Different files were read independently, so this is not an atomic repository snapshot. Later edits require a fresh comparison.

| Source path | Bytes | SHA256 | Reviewed spans |
| --- | ---: | --- | --- |
| core_enhanced/src/authoring/resolve.rs | 39575 | 23119cb7eb3d193352db66bfe7d0ea31156cb435daed3bb4db00ca32b4e1c23b | 1–580, 598–705, 805–968, 1040–1124 |
| core_enhanced/src/authoring/scenario.rs | 29210 | 0152fd9a1f33ad5d061102da34b5bfc13f8e282b697f4e0030dcb741320da859 | 1–290, 349–575 |
| core_enhanced/src/authoring/control.rs | 16254 | 39d5aa1869a16c2d62d4e74b64c6cac5fbd39553ecceca36b72f9cba1832d6c9 | 155–346 |
| core_enhanced/src/application.rs | 44770 | 636b3c9cc15ae2adfa39a15392e0dee22ee7c6683885087f9cc77cefda66081a | 120–250, 827–875 |
| core_enhanced/src/authoring/mod.rs | 4599 | ee366fbee60c246a464d3682287b58b93a44c0613b7e9be89a9f145267003a40 | 1–82 |
| core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/PLAN.md | 27512 | bc5b47c1f3f322bd06e38647225d708dd132a76c1d2bb6bfa49c7d6bfe1c155f | 232–263 |
| core_enhanced/docs/goals/content_authoring_tool_ports/P2G_50_GOAL_SET_001.ko.md | 33683 | bb0b69cfebceb17fd13374826e1dbb6ba6e38e711cb574acc29ca61ca2745f98 | 54–63, 90–104, 124–132 |
| core_enhanced/docs/roadmaps/content_authoring/AUTORUN_REQUEST.ko.md | 18499 | 3e05b131d445cd10ecf478ef959b80f55ce718c41c8a1820e5a5b7d969f9a252 | 108–140 |

Initial resolve/scenario window fingerprints are recorded fully in the JSON companion. No git status, process arguments, personal data, or unrelated source was inspected. Previous /tmp report files were preserved. These findings remain source/design assistance only.
