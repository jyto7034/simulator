# GOAL-CRRP-01 실행

GEX revision2026-09-13-gex-r7; P2G r7 quality2. Core-first U04/S01–08.
Mode goal-run: original continuous user request ../../P2G_00_INITIAL_001.json IN-001;
CF-D06 policy delegation and CF-D07 command evolution. No requested intermediate stop.
Exact member GS-CRRP-001 / GOAL-CRRP-01, PHASE-CRRP-01,
AG-CRRP-RESULT-PERSISTENCE + WP-CRRP-VERIFY.
Accepted upstream ../../P2G_60_HANDOFF_001.json HA-CRRP-001,
../../P2G_50_GOAL_SET.ko.md, FPF-CRRP-001 / PP-CRRP-001 / PVR-CRRP-001.
Exact references and digests are owned by the Handoff and P2G40/50/60 result records.

## Current execution state

GEX-00 and initial GEX-10 complete. Independent audit001 closed Not Complete / High with F01 only;
the assigned canonical host protocol was stale. The audit also alleged target omission, but direct
archive/member checks disproved that part; see the correction's MEMBERSHIP_ERRATUM.json.
Correction GEX-10 updates the document and evidence mapping; fresh GEX-20 remains required.
U03 independent audit003 closed Complete/High before U04.
No prior exact member execution records existed: bounded PLAN/EXPERIMENTS/NOTES/AUDIT search and
new member/directory creation history checked. This directory is the sole execution-document owner.
Role paths: this PLAN.md=current scope/basis/design/progress; EXPERIMENTS.md=actual attempts/evidence;
EXPERIMENT_NOTES.md=technical choices/failures; independent audit output is a fresh post_audit child,
referenced here when produced. The [first independent verdict](../../post_audit/u04_20261004_001/COMPLETION_AUDIT.md)
is preserved unchanged. Current correction: [F01 record](corrections/f01_host_contract_001/CORRECTION.md)
and [validation](corrections/f01_host_contract_001/VALIDATION.json). U04 is not yet Complete.

baseline/SOURCE_BEFORE.tar.gz and SOURCE_BEFORE.json preserve actual dirty before bytes;
ARCHIVE.json binds the archive and successful exact extraction. No reset/stash/reformat of user work.
70 reviewed sources remain exact. Initial full baseline is the exact U03 final Core1717/1717/1726,
host81/88+allocator1 and actual16 public consumers/PC576, fmt/check/event-log/clippy/release,
bound through ../../evidence/PREDECESSOR_BASELINE_001.json. One status-only ROADMAP successor
is explicitly attributed to U03 closure; product363 unchanged. This is not post-U04 validation.

## Implementation and readiness

Scope/meaning and current source design: ../../CODE_REVIEW_001.ko.md RQ01–08 TD01–08 V01–15;
exact Goal and Freeze15 elements. Internally implement RESULT→RESTORE→DURABLE→HOST→VERIFY.
All four producer/schema/file/consumer packages are one atomic delivery unit; partial code is not
independent completion. Existing GameCore prepared root and common grant/employee/node planners
remain sole owners. Existing standard completion journal IO is expanded, not a second domain engine.

Before changing each owner, refine its concrete receipt/phase validation, nonrecursive immutable
resume root, retreat overlay versus explicit checkpoint rewind, exclusive prepare API, journal
revision/tombstone and lock/publication order. Preserve native capture/work/poll and U03 origin gates.
Fallible work finishes before durable intent; intent acceptance is the irreversible result boundary
and later IO/transport failure is recovery-pending for the same accepted result. No fake domain defeat.
Compact receipt stores current result/choice summary, not full runtime/event log. New codecs include
only current state and strict references; V4/V9/unsupported originals are preserved and refused.

Ready inputs: Core source/adapters/actual stock host/tests and compiler available in current workspace;
local configured persistent_data directories absent, fixtures use isolated temp roots. No operational
file mutation/deployment is needed. Existing Rust build cache/toolchain may be reused. No new policy
question or required inaccessible source. Concrete helper and fixture names remain GEX local choices.

## Validation and completion

Own V01–15 durable fixtures before independent audit: actual manual/tick immediate result and ack
restart, fixed ChooseOne2/0/context/retry, success retreat2 and third challenge after restart, every
current unfinished node's pre-entry restoration, first-SavePoint-independent Continue, trauma/load3
and backup monotonicity, employee once/death/consumable, same-process reconnect/U03 faults, real file
subprocess crash every publication stage, concurrent claims/stale revision/terminal ABA, strict legacy
original protection and malformed import, public Core/stock websocket/observer parity.
Full appropriate default/test-fixtures/observation Core and host matrix, allocator,fmt/check/event-log/
clippy/release and affected inherited112 + separate R06 semantic31. Ten dimensions required.
Retain raw failed/zero runs and the approved prior R05 concurrent-peak exception; no new arbitrary cap.
No performance/native work-accounting regression is assumed away because source tests stayed unchanged.

After self-verification, capture exact source/data/doc/evidence target and exercise fuzz0 forward and
reverse restoration. Changed actual sources must be archived; unchanged predecessor proof can be
external exact path+digest/member bindings to avoid recursive archive growth. All original audits stay.
Fresh implementation-nonparticipant GPT-6 Astra / High GEX20 must close Complete/High; implement and
re-audit findings under existing authority. No U05 dependent implementation before independent pass.
Single-phase Goal has no Master/GEX30. Stop only for user stop, approved roadmap completion, or an
actual unresolvable current authority/environment blocker. Intermediate reports do not stop work.

### GEX-10 concrete RESULT refinement

ConfirmedCombatResult is a compact immutable Arc receipt in GameCoreState, independent of live BattleCore.
It contains battle identity, selected-event summary, the already computed acknowledgement result and
post-display phase. Node/session consumption happens during settlement; CombatResult lifecycle validation
binds the receipt rather than assuming an unconsumed node. Ack only releases the receipt and enters the
stored next phase or existing InReward flow. The common ActiveNodeContent::Reward owns actual candidate
selection; a small CombatRewardContinuation owns original grant context/return phase. No duplicate session
owner or new reward engine. Captured summary reports event-log availability honestly; native full event
history is not durable. Existing raw CombatBattle is transient input to the current settlement planner.
Victory planner passes a pending entitlement alongside existing unconditional grant/encounter effects.
Defeat consumes current node even when it immediately causes run failure; it never creates a retry.
BehaviorResult may derive Clone for the immutable receipt acknowledgement; hot clones are Arc-only.

### GEX-10 RESTORE / DURABLE refinement

Continue V10 is now wired to the sole prepared Core draft. Its immutable cached current payload has
an optional independent SavePoint and no snapshot recursion. Core continuation/choice/strict receipt tests, 23 V10 checkpoint tests, and actual conditional file/crash/actor/observer/admin tests have passed on their recorded source revisions. Current full default and test-fixtures Core suites each pass1733 tests. Public continuation, actual rewind/repeated battle identity, stale actor claim and five-mode fault stock WebSocket checks pass on compact V10. The remaining observed Core/host/static matrix is running; no independent verdict yet. This is partial self-verification only.

Durable IO will extend the existing profile journal owner with a generic application intent containing
expected revision, fresh revision, latest active document or tombstone, and merged profile. Active V5
retains a revision even when no run remains. Profile -> active locks are always acquired in that order.
After intent rename, ambiguous directory-sync failure must preserve the accepted candidate and pending
intent; returning a normal pre-acceptance failure there could replay a result after dropping its Core
root. Before intent rename failures discard the candidate. Recovery publishes the same revision and
requires the old or already-published revision, never overwriting an unrelated later writer.

Current and backup will hold the same latest confirmed generation after intent completion. Profile V3
meaning is retained. Version headers of current, backup, legacy files and old/new journals are checked
before any publish/reset/recovery that could overwrite them; unsupported originals are protected even
when another supported copy exists. Corruption reset is distinct from version incompatibility.

Prepared command and host-tick ports retain exclusive-borrow commit, original authority validation,
U03 first-fault poisoning and PendingObservation semantics. Host persistence uses candidate checkpoint,
profile and terminal disposition before commit. An actual terminal without pending choice invalidates
regardless of BattleAdvanced/ack result type. Actors retain the acquired active revision across commands;
conditional publication rejects stale claims and prevents missing/terminal ABA. Ordinary live ticks
with unchanged immutable continuation do not perform disk IO.

### GEX-10 completed source refinements awaiting integrated validation

Retired Standard-only journal writers have been removed; current file publication has one application-intent path. Test-only setup adapters still exercise the same CAS install owner. Current and backup contain the latest confirmed payload; unsupported legacy/current/backup/journal originals are protected from recovery/reset/new publication. A distinct protected profile status publishes only Retry, with reset_required false.

Continue validates required nullable members, graph/content/economy/resource/UUID/RNG state, bounded retreat ledger and both persistent/rewinding VN identities, candidate catalog effects, participant context, result/ack/next-phase consistency and candidate-summary equality. Actual final pending choice survives until claim plus Exit, then terminal tombstone applies. Validation is structural, not an authenticity or anti-cheat claim.

Prepared metadata copying belongs inside its native/authoring budget. Command, auto tick and trusted admin share the exclusive prepared Core root and durable intent acceptance; observer publication is deferred to commit and discarded preparations publish no end. Existing first-fault entry gates include all added prepare ports. Admin owner review and rationale are in EXPERIMENT_NOTES.

08:14Z accounting review correction: compact V10 Current-reference decode must seed the fresh Monolith table before controlled cloning. A new regression reproduced595/851 slots on the uncorrected path. Corrected source supersedes earlier pass receipts only for final currentness; all originals remain. Full source-bound successor selection is EVIDENCE_SELECTION.json, while previous selection is retained separately. Independent GEX-20 has not started.

GEX-10 자체 검증 종료: GEX_10_RESULT.json, COVERAGE.json, PREDECESSOR_IMPACT.json에 정확한 source와 실행 증거를 결속했다. 독립 감사 판정은 아직 없다. target/u04_001의 실제 before/after와 정역 복원을 고정해 별도 GPT-6 Astra / High GEX-20에 인계한다. U05는 독립 Complete/High 이후에만 실행한다.

### F01 correction GEX-10 — 2026-10-04

Independent audit001 closed Not Complete / High (F01 only). The required canonical host protocol is now reconciled with actual current source and its exact before bytes archived. Product/source and all40 final executions remain unchanged; correction verification receipt correction_f01_host_contract001 passes. COVERAGE_002.json and GEX_10_RESULT_002.json own the current correction basis, including PE15/V12/V14 documentation and target membership. Target/u04_002 must pass actual restoration before fresh independent GEX-20. Old audit/target/result/coverage remain original; U05 is still blocked.

### Audit001 membership factual erratum — 2026-10-04T09:42Z

Direct comparison of original unchanged u04_001 MANIFEST/MEMBERSHIP and actual SOURCE_BEFORE/AFTER archive members disproves audit001’s claim that the host protocol was omitted. Its exact stale bytes were already in all original inputs. The valid F01 remains the required content update. MEMBERSHIP_ERRATUM.json records exact rows/hashes and actual archive checks. My preceding correction notes repeated the false omission claim; that part is superseded by this explicit correction. The supplemental HOST_PROTOCOL_BEFORE is a redundant exact original copy, not a newly added member. Original audit and unaudited targetu04_002/result002/coverage002 stay unchanged. Current correction is GEX_10_RESULT_003.json, COVERAGE_003.json and target/u04_003; product and corrected host-document bytes are unchanged from correction verification. Fresh audit must adjudicate this factual erratum as well as F01 content. No self-awarded closure or U05 advancement.
