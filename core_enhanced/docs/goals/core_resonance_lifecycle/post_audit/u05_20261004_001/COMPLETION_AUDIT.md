# U05 independent GEX-20 completion audit — u05_001

**Verdict: Not Complete. Confidence: High. Audit closed.**

One completion-blocking finding exists: an externally authorized CF-D09 document migration occurred during the audit, invalidating the frozen target's current canonical-document identity. No distinct product/source/behavior finding was established. This is not a Complete verdict for either the whole Goal or a successor target.

The implementation-nonparticipant auditor `/root/u05_completion_audit_001` performed this review under [AUDIT_REQUEST.md](AUDIT_REQUEST.md). Only this audit directory and isolated `/tmp` files were written. Product, tests, canonical documents, execution records and frozen targets were not edited. Root confirmed that it did not perform the external migration and had kept its implementation freeze.

## Audit Target Record and currentness

| Item | Exact target |
| --- | --- |
| Goal / Goal Set / Phase / Handoff | GOAL-CRSL-01 / GS-CRSL-001 / PHASE-CRSL-01 / HA-CRSL-001 |
| Manifest | `execution/single_goal/target/u05_001/MANIFEST.json` |
| Manifest SHA256 | `66d7053f2d2c747d189cf2c31fdd4baea800c2900e4b3e8f741f07f4ad5cd99a` |
| Product, 368 files | `3129db450b14515564da3845313cf0ce1539e9f99c032c89788a3a5fe840d283` |
| Combined source/canonical, 376 files | `bed4a2150124a233f40d5ff70843771498cec73861f6217708b957c1e3abe8ab` |
| VCS baseline | `f318414fb0156a67e04c5479000c2a2683fc0c09` |

The authoritative manifest digest is recorded byte-for-byte in [CURRENTNESS_ENTRY.json](CURRENTNESS_ENTRY.json) and [CURRENTNESS_EXIT_INTERRUPTED.json](CURRENTNESS_EXIT_INTERRUPTED.json). Entry at 13:44:19 UTC checked all 3,437 bound members with zero mismatches. Exit at 13:51:33 UTC found 11 document/reference mismatches, zero target artifact mismatches and zero changed product members. [PRODUCT_EXIT.json](PRODUCT_EXIT.json) independently enumerates all product roots, including additions/removals: 368 members and the product digest are still exact.

All four U05 archives and four bound U04 archives were independently checked member-by-member, not merely by archive hash: U05 before 375, after 376, records 2,982, references 103; U04 before 370, after 374, records 2,376, references 134. Target patches and restoration receipts remain bound and unchanged. Existing successful forward/reverse restoration evidence was reused; the auditor did not claim a new restoration execution.

## F01 — current canonical target became stale during review

**Responsibility:** external CF-D09 / USER-18 migration plus the next execution owner's intake/rebinding; not a discovered U05 gameplay defect.

The current `game_rulebook.md` now contains the formerly named next policy; `game_rulebook_next.md` and the bound `game_rules_next/*` paths no longer exist. The migration's [record](../../../../roadmaps/core_first/execution/rulebook_path_migration_20261004_001/RECORD.ko.md) explicitly requires active executors to rebind current locators and identity while preserving frozen originals. Root independently confirmed this new user-authorized event after the auditor detected it.

The 11 mismatches are `docs/README.md`, `docs/game_rulebook.md`, `docs/game_rulebook_next.md`, five bound `game_rules_next` pages (`battle-controls`, `combat`, `equipment-and-fragments`, `open-decisions`, `results-and-save`), `roadmaps/core_first/ROADMAP.ko.md`, `roadmaps/core_first/execution/DECISIONS_20261004.ko.md`, and `skills/skill_authoring_model.ko.md`. Exact old/current hashes and missing-path distinctions are in the exit receipt.

This violates the requested exact-target/currentness completion requirement and leaves V14/X10 whole-target closure unsatisfied. The old filename must not silently confer old policy authority on the newly relocated current document. A frozen historical archive can remain valid while its current publication binding is stale.

**Correction and revalidation scope:** preserve this target and audit; intake CF-D09's actual before/after documents and user authority; check body/policy preservation and classify its delta; update active execution locators and source/canonical membership; capture a new target; request an independent impact-based re-audit with entry/exit currentness. Do not rewrite original Goal/Freeze/audit records or undo the authorized migration. Full product reruns are not justified by a proven locator-only delta while exact product and command-input bindings remain valid. If intake reveals a policy/product change, expand validation to that actual change. U06 remains gated until a successor whole-Goal Complete / High verdict.

## Meaning, conditions and source review

The auditor read the GEX reader, §5/§6 quality and completion requirements, GEX-20, the whole-Goal P2G-50 semantics, frozen acceptances, and the reviewed P2G-20 policy elements. Original CAVER D09 V01–V21 and original R06/R07 F01–F12 meanings were restored directly rather than inferred from test-name counts. U04's exact prior audit and frozen evidence were read for predecessor applicability.

[CONDITION_REVIEW.json](CONDITION_REVIEW.json) records all 14 current validations, 127 inherited conditions, 10 dimensions and 31 R06 acceptance mappings, with individual original meanings, current successful selectors, public evidence and audit dispositions. The following are source-level judgments supporting that matrix; they do not override F01.

| Conditions | Independent source/evidence conclusion |
| --- | --- |
| V01–V03 | Selected form maximum is strict/nonzero; deployment uses floor(max/5) plus the existing charge bonus. Employee global-second pulses do not catch up after blocked intervals. Basic/Rest resonance removal preserves HP Rest and autonomous opponent recovery. Positive gains share the composed use/HardCC gate. |
| V04, V09, V11 | Input validates before commit, consumes once and creates a private action credential. Deferred start and corrosion use that credential at zero resonance. Duplicate inputs/callbacks do not duplicate consumption; preview does not replace final admission. Rejected operations retain focus/playback/revision and transactional state. |
| V05–V07 | Actual future Flow/attack control is separate from released projectile/area carriers. Selected finite beneficial leases consult their owning sustained records; shield exhaustion, overlay use, refresh/replacement and stale expiry are covered. Independent hostile/area/permanent effects do not create unbounded use. HardCC/InterruptCast cancel future control without refund or later resumption; late Silence differs from pre-admission Silence. Mandatory direct target loss cancels, while accepted area tile geometry persists. |
| V08, V10, V12 | Payload/kind/range/eligibility and valid empty-tile distinctions are enforced. Manual RetargetOnStep is rejected; automatic behavior survives through an explicit live Funeral catalog split. Required lifetime declarations, selected capacities and removed employee/Rest fields are checked by normal loaders/consumers with precise errors. Query can correctly report unlocked use with unsettled released effects. |
| V13 | Supported snapshot/reconnect boundaries preserve use state. Fresh process Continue restores the durable pre-entry/completed boundary; it is not presented as running-battle restoration. Current continuation V11 and conversion-only V12 are explicit. Unsupported original fixtures remain preserved. |
| A08–A12, X01–X08 | Boss action/waiter, corrosion, Flow, sustained channel/overlay, protection/CC and public cross-boundary owners retain their original authority. New employee use state composes with them; source review found no substituted parallel owner. Summons remain the existing independent boss mechanism, not an invented employee SkillEffect API. |
| IVA-CAVER-01–21, F01–F12 and R06 mappings | Original expression units, rounding, release/impact context, aggregation, copy/resource identity, rollback, trace and public consumer meanings remain exercised on the current source. Intentional syntax/state migrations preserve original oracle behavior and historical inputs. |
| PE-CRSO-01–12, PE-CRFI-01–12 | Existing observation/fault services are reused. Native work/clone/retained accounting includes added state; owning transactions rollback on faults. Public mode parity, bounded queues, observer cleanup, fault root/store/authority and normal-player isolation are current. A public receipt is valid evidence where a requirement has no unit selector. |
| PE-CRRP-01–15 | Result/Continue/rewind ownership is preserved. Current Core/host selectors and actual stock-WebSocket process/file probes cover terminal-before-ack, fixed choices, duplicate/stale writers, rewind and grant identity, separate active/profile publication, reconnect and unsupported originals. |
| V14, X09–X10 | Source tests/builds and numeric evidence remain current. Target artifacts are intact. Final canonical publication/currentness fails under F01; no whole completion is awarded. |

Direct review covered the complete resonance runtime and its main tests, scoped forward changes in simulation, commands, basic attack, Flow/cast/sustained owners, deployment/death, RuntimeUnit and clone shape, authoring/employee/fragment validation, persistent conversions, DTO/snapshot seams, major protection/overlay/channel tests and the actual host deferred-acceptance test. Structural review found bounded per-unit use state, existing action identity reuse, explicit selected-effect ownership and source-bound metering. No test-only gameplay branch, catch-all compatibility fallback, duplicate event scheduler, or proportionality/maintainability defect was established in this scope.

## Evidence independently checked and evidence reused

[EVIDENCE_VERIFICATION.json](EVIDENCE_VERIFICATION.json) verifies each selected receipt/log hash, exact before/after product identity, archived/current command inputs, each mapped selector's actual successful log line, all 437 original/migrated adapter files and R06 references. The initial checker errors are retained separately: one used string rather than Path ordering for an aggregate digest; another incorrectly required unit selectors for public-only conditions. They were corrected as auditor-checker issues, not product findings.

Existing valid executions were reused: Core 1,758 default, 1,758 fixtures, 1,767 observation; host 91 default and 99 observed, with the one intentionally excluded heap test separately passing; fmt/check/event-log checks, Core/host clippy and release. There was no blanket product rerun by the auditor. Public consumer source and raw output were inspected, including schema negatives, independent area, 21 public API commands (six expected negative privacy compilations), five fault/observer modes, continuation and rewind traffic. Seven actual process starts and 78 continuation messages, and seven starts and 114 rewind messages, substantiate the process claims. The public host test for paused acceptance uses the normal actor command and reconnect; its test clock probes do not bypass gameplay mutation authority.

The original PC gameplay comparator passes all 576 rows / 36 conditions (31 success, five typed refusals). The earlier 96 after-log mismatches are retained: an isolated nonemployee TestFixture inherited changed employee defaults. PC003 explicitly restores that private helper's historical 0/100/1000 input; production and Employee defaults remain unchanged. Hooks and the copied current source are separately archived. This input correction is applicable and does not relax gameplay/HP/fact/queue/error comparisons.

**Costs increased.** The strict unchanged-cost failure is correctly retained. The auditor read the cost comparator and source basis, and independently executed its full raw-row comparison with output redirected solely to [PC_ACCOUNTING_RECHECK.json](PC_ACCOUNTING_RECHECK.json). All 576 non-timing rows match after the declared deltas only: 64 bytes per unit for absent use/pulse options; the additional RuntimeFactSource primitive clone visit and corresponding work; source-count patterns for each measured transaction; allocator bytes derived from the actual bucket-capacity progression/clones; and 128-byte retained/requested differences for the two-unit adapter. Every other non-timing field, including full refusal text and request counts, remains compared. This is source-explained cost accounting, not unchanged-cost parity or a replacement for numerical performance gates.

The plain public guards preserve their original ceilings: snapshot p95 221,310 ns < 500,000, encoded snapshot 49,375 < 65,536 bytes, retained export 31 < 100,000 / 18,376 < 32,768, bounded history scaling and stage slope; low/high allocator staging is exactly 75,090 requested bytes / 627 requests with zero retained at end. R05 raw101 retains only the prior authorized concurrent-peak warning: 9,497,009 versus 9,030,188, excess 466,821 bytes; other hard failures are empty. Instrumented latency remains observational and is not claimed as a passing latency gate. No numeric ceiling or new exception was introduced. The previously authorized 20-minute stress exclusion remains a limitation.

CF-D08 was independently accepted for its then-current target: [DOCUMENT_BODY_CHECK.json](DOCUMENT_BODY_CHECK.json) proves the historical body from `게임 개요` matches U04 before and the next-policy body from `어떤 게임인가` matches its pre-correction bytes. CF-D09's archive preserves those exact pre-migration hashes. CF-D08 did not justify undoing product policy or promoting the old rulebook. The newly discovered CF-D09 current publication must nevertheless be separately rebound as F01 requires.

## Closure and limits

The review covers actual local Linux Core, stock WebSocket and file/process boundaries. It does not establish deployed-service or Unity implementation readiness. Source review and successful finite executions are scoped evidence, not proof of every future input. The existing approved R05 warning and stress exclusion remain explicit.

This audit is closed with **Not Complete / High**, solely because of F01. Root may release the audit freeze to perform the authorized migration intake, current-document correction and new target capture. This release does not authorize declaring U05 complete or beginning gated U06 implementation. Preserve all prior failures, raw evidence and this audit. A successor impact-based independent audit can reuse the exact unchanged product evidence after rechecking its applicability and currentness.
