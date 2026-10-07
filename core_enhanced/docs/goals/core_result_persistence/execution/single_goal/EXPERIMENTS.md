# U04 실행·검증 기록

P2G40 receipt와 P2G50 quality2 표준 검증/의미 검토 PASS; P2G60 exact 인계 완료.
GEX00 before actual archive와 추출 hash 확인 완료. Production mutation 없음.
초기 baseline은 exact U03 closed evidence 재사용이며 U04 새 acceptance 실행은 아직 미수행.

GEX10 RESULT initial implementation: terminal transition invokes existing staged settlement; compact
receipt/ack-only continuation; ChooseOne retains ordinary RewardSession and original grant context;
defeat retry removed and terminal failure consumes node. result_check_001 failed E0382 (stage UUID read
after moving RewardSession). Saved stage UUID before move; result_check_002 cargo check PASS, exact
source unchanged. Existing warnings and new currently-unused helper warnings remain to resolve by scope.
Added explicit retreats_used and common max2 capability/handler/snapshot; normal challenge ordinal derives
from consumed retreats so process-exit recovery will not spend a retreat. New Continue codec is not yet
implemented; current source is a partial atomic implementation and not a completed/current delivery.

- result_retreat_check001: PASS cargo check; source identity unchanged. Result/retreat partial code only.
- continue_check001: retained failed attempt, wrong Cargo package selector `core_enhanced` (actual
  package is `game_core`); no Rust build attempted.
- continue_check002: retained failed compile, a historical V7 child still uses parent UuidManager
  import. Restored the required import, without re-enabling historical Continue acceptance.
- continue_check003: PASS `cargo check -p game_core`, exact source unchanged; cache/capture/serde
  wiring compiles. Old V9 reader and reward wrapper dead-code warnings remain cleanup work.
- continue_tests001: started actual public Core regression fixtures for first-SavePoint-independent
  Continue, two retreats/third entry/repeated quit, and immediate result settlement/ack restart.
- continue_tests001 completed PASS: 3 actual public Core fixtures, 0 failed, 1660 filtered,
  6.78s test execution / 86.54s compile+run, exact source unchanged. Restart restored current
  first-SavePoint-independent state, retained 2 successful retreats through repeated third-entry
  quits without consuming Load, and preserved an already completed battle before/after ack with
  unchanged resources/roster/inventory. Host durable IO is not covered by this result.
- result_continue_matrix001 started: broad current default-lib compatibility diagnostic. Existing
  raw CombatResult fixtures/old checkpoint schema assertions are expected to expose timing/schema
  changes; failures will be retained and resolved with equivalent actual-boundary coverage.
- result_continue_matrix001: retained FAIL, 1605 passed/58 failed, 220.42s tests, source unchanged.
  Failure inventory is in the immutable raw log. Most failures are obsolete V9/ack settlement/
  unbounded-retreat fixture expectations. A real compatibility finding was the terminal runtime-fact
  public view disappearing when raw CombatBattle was consumed: restored bounded ephemeral facts on
  the receipt (`serde(skip)`) and retained-extent admission; Continue never serializes these facts.
- choose_tests001: retained FAIL, 4/5 pass. Zero-candidate explicit summary passes. Two-candidate
  fixture lacked the existing Boss pool's featured grant kind; corrected the authored fixture to
  include a valid fragment candidate without changing the established mission reward policy.
- prepared_check001 started with runtime-observation enabled. New generic exclusive prepared owner
  supports command and host tick; first-fault mapping uses the existing runtime_error and prepared
  observer guard. Cached checkpoint handles now share immutable bytes instead of copying the complete
  continuation each live tick. Actual host journal integration remains pending.
- prepared_check001 PASS (runtime-observation enabled), source unchanged, generic command/tick
  prepared API and shared cached byte handles compile.
- durable_check001 retained FAIL: child journal module methods needed parent visibility; changed
  only the necessary methods to pub(super).
- durable_host_check001 PASS (server with runtime-observation), source unchanged. Command/tick use
  generic intent before infallible commit. Unsupported Reset and pending mutation/tick gates added.
- durable_tests001 PASS: 4 tests/0 failed/81 filtered. Actual subprocess termination at all 16
  intent/profile/active/latest-backup/clear stages; restart recovery and repeated recovery; damaged
  primary after recovery retains latest payload and Load3; pre/post acceptance IO distinction;
  stale revision and terminal/missing ABA; unsupported current/backup/profile/legacy/old+new journal
  and malformed journal preserve originals. 0.29s tests/77.48s build+run, exact source unchanged.
- choose_tests002 PASS: all5 new Core restart tests, 9.96s tests/60.06s build+run, exact source
  unchanged. Two actual ChooseOne candidates remain fixed through ack/select/claim restart; XP31
  goes only to the original deployed participant; duplicate claim/selection after claim and empty
  choice skipping are verified. Stable VN session and persistent trigger-consumption fields were
  also added to the continuation inventory; dedicated VN and every unfinished-node tests remain.

- `checkpoint_tests001`: FAIL (21/23, 57.25s); V10 fixture migration exposed two obsolete expectations (unsupported-version error text, export incorrectly assumed to read a manually replaced SavePoint). No product failure masked: negative graph cases retain error-cause assertions and immutable-root checks.
- `checkpoint_tests002`: PASS (23/23, 66.16s total / 12.13s tests), immutable source verified. Current and SavePoint graph validation, exact inventory limit, RNG continuation, content-generation pinning, nonzero R06/R09 fresh-process probes, original V7 preservation and explicit V7/V8→V9 conversions all exercised. V9 output remains unsupported by V10 Continue. Does not imply the remaining combat/host matrix passes.

- `combat_tests001` / `combat_tests002`: compile failures in newly migrated test helpers (moved projection and inner-doc placement/nonexistent error spelling). Originals retained; no tests claimed from these attempts.
- `combat_tests003`: 150/152 PASS (207.77s total, 163.04s tests), source unchanged. Actual pre-terminal resource failure/retry, native and authoring refusal, choice tamper strictness, event defeat/restart, trace separation, two-retreat bound and R06–R12 world paths pass. Remaining failures were two old defeat-retry expectations. Examining StandardCompletion exposed a real missing terminal classification: defeat at that terminal could otherwise take the success completion branch. Added StandardCompletion/SpecialRunCompletion alongside RunFinalBoss in final-boss defeat classification; now assert failed run, consumed node, no tutorial flag, no durable Continue, and test retreat on the independent pre-defeat branch.

- `result_continue_matrix002`: FAIL 2 / PASS1666 (516.94s total, 477.74s tests), exact source unchanged. The full default matrix now passes all combat/resource and R06–R12 paths. Remaining cases: new public prepare-Pause fixture omitted required battle authority; old VN rewind fixture injected nonexistent trigger IDs, now correctly refused by the checkpoint validator. Repair uses actual authority and retained trigger definitions; adds three repeated Continue round trips before explicit Load. Diagnostic `prepared_tests001` reproduces the missing-authority refusal (no product defect). Timing is recorded without declaring a performance pass or relaxing any cap.

- `durable_host_tests001`: correct production binary compiled, but selected `--bin game_server` contains **0 tests**. Recorded as build-only/zero-test, not host test evidence. Corrected next command to `--lib`.

- `durable_host_tests002`: FAIL 27 / PASS 58 of 85 (54.25s total, 35.46s tests), exact source unchanged. Four new file-journal/crash tests pass. Remaining failures traced to versionless fixture Core bytes, previous-generation backup expectations, retired Standard journal helpers, and actor expectations from independent post-commit writes. Migrate fixtures and replace obsolete terminal writers with generic conditional intent tests; preserve this failed run.

- `durable_host_tests003`: PASS 85/85 (113.47s total including build lock, 38.42s tests), source unchanged. Current conditional terminal journal tests replace obsolete Standard-only writer helpers, removed from production. Latest confirmed backup, unsupported-file protection, preintent failure/Core unchanged and postintent failure/Core committed+retry gating are exercised through real files and actor commands.
- `continuation_tests004`: PASS10/FAIL1 (73.91s total,13.77s tests), source unchanged. Prepared command/tick drop+commit, original fault authority, stable VN active+queue, five unfinished interactive node pre-entry roots pass. Event-combat fixture failed its probabilistic employee check before battle entry; align its seed/IDs with the retained successful Event fixture. Failed original preserved.

- `durable_host_tests004`: PASS7/7 U04 tests (103.32s total with build lock,10.46s tests), immutable source. Actual scheduled-tick function preintent failure twice preserves Core/checkpoint and emits no battle update; same bucket retry settles once and fresh Core restores before ack. Two restored actors: accepted abandon invalidates with a revision; stale actor abandon refuses without Core/file change. Protected legacy/version profile capability and forced reset/retry/new-run preserve originals.
- `continuation_tests005`: PASS12/FAIL1 (61.67s total,16.90s tests), immutable source. Event pre-entry restoration now passes. Receipt phase/outcome/ack/entitlement tamper cases pass. Terminal choice fixture manually inserted a terminal into a nonfinal template and was correctly rejected; use the authored final Boss slot/template and its existing StandardCompletion instead. No weakening of graph validation.

- `durable_observation_tests001`: PASS8/8 (111.80s total,14.66s tests), source unchanged. New prepared terminal tick discards without published end/commit, then commits with positive work/admission and exact unobserved result/snapshot/checkpoint equality. All seven file/actor cases also pass with the observation feature.
- `terminal_choice_tests001`: test compilation failed because a fixture-only encounter selector edit also matched the earlier nonterminal helper (`terminal` out of scope). Narrowed that edit; no test-pass claim from this run.

- `continuation_tests006`: PASS13/FAIL1 (55.40s total,19.43s tests), unchanged source. SavePoint load now explicitly tests two retreats, Continue, explicit Load and Continue through all three loads; retreat ledger rewinds, cumulative count and separately merged permanent profile do not. Final Boss fixture retained a live authored slot policy overriding its newly selected encounter reward policy; remove that fixture override, preserving production validation.

- `terminal_choice_tests002` / `durable_admin_tests001`: compile refusal in the new Core prepared-admin adapter (`&self` passed to the existing `&mut self` prepare owner). Corrected adapter receiver; no tests executed, both original failures retained.

- `terminal_choice_tests003`: PASS1/1 (44.88s total,3.59s tests), unchanged source. An authored final-floor Boss slot carries a real ChooseOne entitlement: restart before ack, after ack and after claim; duplicate claim refuses; final ExitReward clears Continue.
- `durable_admin_tests002`: PASS1/1 (92.87s total including build lock,7.19s tests), unchanged source. Public prepared-admin drop preserves snapshot/cache; host preintent IO failure preserves Core; accepted postintent failure retains candidate and blocks further admin mutations; Retry publishes the same candidate, which a fresh Core restores. All preceding compile failures remain separately recorded.

- `public_continuation001`: fixture package validation refused before server startup: TimeReached Victory is valid only for explicit Defense. Restrict the isolated authored timer+ChooseOne edits to normal_corroded_patrol/normal_breach_response. Production catalogs and validation remain unchanged.
- `compat_prepared_public_001`: external public prepared/Core/actor compilation succeeded; preintent command/root/file and discarded-attempt assertions passed, then the predecessor's absolute observer commit count failed (3 vs1). The paired profile store adds one accepted MergeProfile at command entry and one at attach. Revised successor explicitly asserts exactly those two named accepted merges and zero gameplay commits, retaining the original current-battle/root/byte/first-fault assertions. Original failure and public consumer output remain.

- `integrated_release001`: PASS optimized Core/stock-host bins including runtime-observation (84.26s), exact source unchanged after workspace fmt.
- `public_continuation002`: PASS (17.45s), exact integrated source. Actual packaged/validated catalog, stock release WebSocket, both persistence stores, 7 process starts and71 messages. Before-first-SavePoint Continue, same-process reconnect pause, unfinished combat abrupt restart to pre-entry, actual scheduled tick result persisted before acknowledgement, restart before/after selection and claim, duplicate claim refusal, ExitReward and durable abandon tombstone all pass. Authored short timers/ChooseOne affect only two normal Defense fixture encounters; no admin command or production-catalog modification.
- `compat_prepared_public_002`: PASS current external public Application+actor consumer with exact root/byte refusal twice, discarded prepared observations (and exactly two explicit profile sync commits), then accepted retry with peak204→213. `compat_prepared_boundary_002` checks external visibility/borrow/nonclone/nonserde/once-commit boundaries. Remaining compatibility sequence continues under `_002` receipts; intermediate results are not a complete inherited acceptance claim.

- `integrated_core_default001` and `integrated_core_fixtures001`: each 1673 PASS / 1 FAIL, source unchanged, respectively 605.39s / 629.91s total. The VN history fixture rebuilt live data without its boss runtime catalog, so validation refused `suppress_little_red` before the test. Correction retains the full validated generation via `GameDataBuilder::from_validated` while changing only authored VN triggers. The sequential matrix stopped after this refusal; later suites were not claimed.
- `compat_*_002`: all16 procedures completed on source `aa7bb6cd3754e1847fd9ca86e7d797e646e7886c08116e9b7858f52504b0020c`. Prepared public/boundary, release, defaults38, oracle5, trace5, CLI, profile24 commands/6 hosts, boundary13, publicAPI21, PC setup/execution/original comparison/immediate comparison, R05 raw/classification retained. PC576 preserves exact non-timing fields. R05 raw101 is only the approved concurrent-peak warning (201114 bytes excess); other hard guards pass. This is pre-compaction evidence, not silently rebound to the later source.
- `final_fault_001_*`: five actual fault observation modes have exact result equality, queue-full drops are observed, selected battle ends without another commit; stock WebSocket actual tick fault/reconnect/replacement/Quit/Continue passes with101 messages. All source identities unchanged; public fault source/config copied from U03 and output/import owners changed to U04 only.
- `public_guards001`: genuine new regression, raw101. Run snapshot p95 231108ns/48591bytes and cached export31ns pass latency; V10 export36674bytes exceeds unchanged32768byte cap because current and equal SavePoint are both serialized. Preserve raw output, do not waive/raise the cap. A wire-only current-reference for an identical SavePoint and single admitted JSON-value construction is under correction; independent gameplay rewind ownership remains required.

- `continuation_compaction_tests001`: 14PASS/2FAIL. All choice/receipt/unfinished-node/new sweep cases pass. The internally tagged snapshot enum buffers nested integer-keyed maps as serde Content, causing distinct SavePoint deserialization to refuse a u64 map key. Corrected the private wire to externally tagged strict current/snapshot variants, preserving direct map-key decoding; no validation weakening. Also replaced large `json!` interpolations with explicit owned Value moves so the admitted graph is not secretly cloned by serde. Equal-reference import and later independent rewind are rechecked in002.

- Compact V10 final targeted checks: continuation_compaction_tests00216/16; vn_history_tests0011/1; public_guards002 full plain guard PASS, snapshot206161ns/48591bytes, checkpoint40ns/18538bytes; public_guards_allocator001 PASS (627 allocations/0 reallocations/74834 peak/0 retained bytes, low and high history equal). Exact raw receipts retained.
- Integrated current completed suites: {'integrated_core_default002': [1733, 0, 0], 'integrated_core_fixtures003': [1733, 0, 0]}. Remaining observed Core/host/check matrix still in progress. integrated_core_fixtures002 failed compiler query-cache allocation (OS12); retry003 uses explicit --jobs=1 and passed. No source change/cap relaxation for this environment failure.
- Current compatibility completed: compat_*_003 plus compat_default_parity_004, all16 actual procedures. Initial default_parity003 failed because the new evidence directory was missing; resume_after_defaults.py creates the directory without rewriting earlier outputs. PC576 original/immediate non-timing equality passes; R05 raw101 classified warning_concurrent_peak_only; excess314044 bytes, no other hard failure.
- public_continuation003 PASS: seven actual process starts,78messages; independent stale actor claim fails CAS with exact current/backup files and loser choice preserved. public_rewind003 PASS:114messages, real Load then repeated identical battle UUID/candidates/settlement/assets, legitimate new claim once and duplicate denial. Initial rewind001 selected the node converted to SavePoint;002 used the next Black Box encounter outside the timer fixture.003 includes all three reachable Normal Defense fixture timers; original production data unchanged.
- final_fault_003 full five observation modes and stock WebSocket PASS. final_fault_002 websocket reached a lower-work next room after admin SavePoint and did not reach native cap; budget256 probe failed startup preview admission (spawn_request_id_scan requested273). Final isolated fixture binds that next room to the original high-work patrol, keeps max_work1000, and preserves production/old fixture bytes. This fixes reproducibility, not runtime behavior.

### 2026-10-04 08:14Z — fresh import accounting correction
Pre-fix integrated_core_observation003 and integrated_host_default003 both passed with source_unchanged. The matrix parent was deliberately stopped after the running host-default child completed, before further obsolete-source checks. Their receipts/raw logs remain. A source review found Current SavePoint wire resolution cloned its fresh Monolith native table before registering capacity. import_clone_regression001 was a test compilation failure (ContentContract::default absent);002 compiled and reproduced clone peak595 vs expected851. The corrected test fixture uses ContentContract::current; no product behavior had changed for002. The product fix seeds only the fresh serde owner before the first independent clone; it does not certify general arbitrary roots. import_clone_regression003 and the successor integrated/public matrix record the corrected source. EVIDENCE_SELECTION_PRE_IMPORT_FIX.json preserves earlier intended selections; final selections point to new executions, never relabel old runs as current.

Corrected-source public verification: guards003 and allocator002 pass the original caps; actual continuation004 and rewind004 pass7 stock-process starts each (78/114 messages). All16 compat_*_005 commands now complete: defaults38, boundary13, API21 commands/13 CLI/6 private negative compile; PC576 exact non-timing comparisons pass. R05 raw101 remains warning_concurrent_peak_only: current maximum9,299,316 vs original cap9,030,188 bytes, excess269,128; no other hard guard failures. Allocation-instrumented timing observations are not latency gates. Current source SHA256cb033469ed897c2bb38cdd5cd2bc4561b8b820acd0b680d45c5381abf4b3c629. Final fault004 probes and observed Core/host/static matrix are still running, so this is not GEX-10 completion.

Final public matrix completed: final_fault_004_SUMMARY.json confirms exact off/none/detached/queue_full/on native result equality, selected observer cleanup without extra commits and real stock WebSocket fault/reconnect/write protection/Quit/new-owner/Continue. final_public_matrix.py exited0. Current release/guards/continuation/rewind and completed default/fixture suites share sourcecb033469ed897c2bb38cdd5cd2bc4561b8b820acd0b680d45c5381abf4b3c629 and exact log hashes. Remaining observed Core/host/static matrix is still required before GEX-10 closes.

Final binding correction: the first current_003 R05 summary accidentally read the older compat_r05_003 receipt/log; the actual raw measurement came from compat_r05_005 on current sourcecb033469ed897c2bb38cdd5cd2bc4561b8b820acd0b680d45c5381abf4b3c629. The old summary carried source08de3d272ce402e9549dfc320ff6bb804f602c9eb4a52563f045966856f9aaaa, which was mistakenly copied into two progress notes above (now corrected explicitly). Original helper, analysis receipt/log and summary remain unchanged. summarize_r05_current.py binds the same current raw bytes to the correct current receipt/log and writes R05_SUMMARY_CURRENT_003.json via compat_r05_compare_006. The first coverage index is preserved under evidence_binding_correction_001; regenerated coverage additionally checks the R05 summary source, raw hash and actual exit. No product or measured result changes, and no measurement rerun is needed.

최종 자체 검증: GEX_10_RESULT.json. 기본/fixture/관측 Core·host와 전체 검사 및16 public 소비자, 실제 다섯 fault 관측 모드·stock WebSocket 영속/재접속·선택권 동시성·SavePoint 재전투를 현재 source에서 검증했다. R05 raw exit 101 / warning_concurrent_peak_only는 승인된 해석과 원자료를 분리 보존한다. 새 독립 완료 판정은 포함하지 않는다.

### F01 canonical host correction

First independent audit001: Not Complete / High, F01 only, closed09:29:04Z. Source-bound correction_f01_host_contract001 exits0; exact product cb033469ed897c2bb38cdd5cd2bc4561b8b820acd0b680d45c5381abf4b3c629 is unchanged and40 original execution receipts/logs/archived inputs/runners are exact. The checker verifies the before document against the auditor copy, current command/status vocabulary and local links; semantic source mapping is in corrections/f01_host_contract_001/CORRECTION.md. No product suite rerun or new runtime outcome is claimed. Required host doc is now part of the successor full before/after source archive and evidence map. Prior failures and Not Complete audit remain unchanged.

### Audit001 membership factual erratum — 2026-10-04T09:42Z

Direct comparison of original unchanged u04_001 MANIFEST/MEMBERSHIP and actual SOURCE_BEFORE/AFTER archive members disproves audit001’s claim that the host protocol was omitted. Its exact stale bytes were already in all original inputs. The valid F01 remains the required content update. MEMBERSHIP_ERRATUM.json records exact rows/hashes and actual archive checks. My preceding correction notes repeated the false omission claim; that part is superseded by this explicit correction. The supplemental HOST_PROTOCOL_BEFORE is a redundant exact original copy, not a newly added member. Original audit and unaudited targetu04_002/result002/coverage002 stay unchanged. Current correction is GEX_10_RESULT_003.json, COVERAGE_003.json and target/u04_003; product and corrected host-document bytes are unchanged from correction verification. Fresh audit must adjudicate this factual erratum as well as F01 content. No self-awarded closure or U05 advancement.
