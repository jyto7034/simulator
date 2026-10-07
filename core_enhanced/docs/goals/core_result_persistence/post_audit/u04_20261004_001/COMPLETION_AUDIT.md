# U04 independent completion audit 001

**Verdict: Not Complete. Confidence: High.**

GS-CRRP-001 / GOAL-CRRP-01 / PHASE-CRRP-01 has one completion-blocking finding: its explicitly required canonical host protocol documentation is stale and omitted from the frozen audit target. The inspected runtime implementation and current execution evidence did not reveal a second blocking defect. Acceptance15 cannot close, and U05 is not eligible.

The audit was performed by a fresh implementation-nonparticipant GPT-6 Astra / High reviewer under the existing continuous execution authorization. This was audit-only; no product, canonical documentation, target, execution record, roadmap status or prior audit was repaired. The applicable GEX Reader Contract, §5/6.4/6.7 and GEX-20, accepted P2G handoff/freeze/phase/Goal records, original decision/policy meanings and predecessor requirements governed the review. Implementer coverage and handoff were used as locators, not accepted as verdicts.

## Exact target and currentness

Target: `execution/single_goal/target/u04_001/MANIFEST.json`.

- Manifest SHA256: `f1b9bd51c71fe415fccc79f8f073462915b7baa79b3440f20238d41f35a65f5f`.
- Product source SHA256: `cb033469ed897c2bb38cdd5cd2bc4561b8b820acd0b680d45c5381abf4b3c629`.
- Combined source/docs SHA256: `6e1faf37d1e5eb23b3b3493d2e60c56943d32d04eb23d006516cc44462efbb52`.
- Git base: `f318414fb0156a67e04c5479000c2a2683fc0c09` (dirty working tree; HEAD alone is not the audit target).

Entry and exit compare actual bytes for all 2,830 declared members and target artifacts; both have zero mismatches. The omitted required host document is separately frozen in `HOST_PROTOCOL_AT_AUDIT.md` and `OMITTED_SURFACE.json`, SHA256 `ebfb9534fc901afe8a40885c23c6504e6524f0b07420f83aa0f45a60898429c4`. Its omission is a scope completeness failure, not an undetected change to a declared member.

## Finding F01 — required canonical host contract is stale

`P2G_50_GOAL_SET.ko.md:82` explicitly requires `game_server/docs/unity_bridge_protocol.md` to document immediate settlement, receipt/choice, retreat, Continue/SavePoint, current codec/strict rejection, prepared durable acceptance/recovery/conflict and actual query/ack consumption.

The actual document still states independent profile persistence and write-failure-after-gameplay behavior at lines115–117, V4 active/V9 Core at line119, and previous-Core-payload backup rotation at lines121–124. These statements do not describe the implemented paired active+profile path: current V5/V10, conditional revision/tombstone, generic accepted intent and latest-generation backup. The new result/recovery contract is also missing. The document appears in none of the target source/record/reference membership.

Correction owner: **this Goal/phase's GEX-10, HOST/VERIFY canonical documentation obligation**. Update the protocol from actual source and public DTO/capability/query/ack/recovery behavior, explicitly distinguishing paired persistence from the still-supported profile-only dirty/retry mode. Preserve the old bytes and publish a successor target including the required surface and restoration/evidence bindings. Then perform a fresh independent GEX-20 audit. This report and previous evidence must remain unchanged.

`FINDINGS.json` records the exact ownership, locations, impact and correction boundary. This is not a claim of a newly reproduced runtime grant or persistence defect.

## Completion-condition and source judgment

`CONDITION_REVIEW.json` adjudicates all 15 frozen acceptances, V01–V15 and all 10 required dimensions. PE01–PE14 have passing direct runtime evidence. PE15 is Not Complete because required canonical host documentation and complete target inclusion are missing. V12's actual consumers and V14's restoration/state/source checks pass their runtime checks, but their canonical-contract/membership closure remains open. All ten dimensions have substantive current runtime evidence; producer/consumer completion remains qualified by F01.

`SOURCE_REVIEW.md` details the independent inspection of immediate terminal settlement; receipt-only ack; fixed entitlement/candidate/context and at-most-once claims; two retreats; whole-node pre-entry versus battle-entry versus SavePoint restoration; current Continue, load3 and permanent profile union; strict codec; sole prepared commit; paired intent/CAS/tombstone/backup/crash behavior; public ports and U03 first-fault gates; native work accounting; observer commit/end lifetime; and structural/legacy disposition. It includes the final fresh-monolith pre-clone fix and reasons why remaining profile-only behavior must be documented accurately.

`PREDECESSOR_REVIEW.json` covers all 112 inherited conditions and separately the original31 R06 acceptances. Original acceptance meanings, before/after source and changed terminal/restore/public lifetimes were considered, not just aggregate pass counts. `INHERITED_SELECTOR_SOURCE.json` and `CHANGED_INHERITED_TESTS.diff` preserve the direct test-body comparison:149 of164 selected inherited bodies are exact, and the15 changed/renamed bodies were individually inspected. V9→V10 field paths/test SavePoint helper changes do not waive original graph/content/error obligations. F09/F10 old-format resumability and seven independent-write host tests are explicitly superseded by approved U04 acceptance12/14; the historical V7/V8 converters still yield V9, which current Continue refuses. No old-save semantic compatibility is falsely claimed.

## Executed audit checks and reused current executions

The auditor executed the integrity/restoration, current receipt/input/runner/source binding, exact selector, raw numeric, public-record and service-reuse checks in this directory. Existing source-bound final product executions were validated and reused; the entire product suite was not needlessly rerun.

- All40 selected final receipts match their logs, archived command inputs, immutable runner and exact current product source before/after. All selected suite totals match actual result lines: Core1734/1734/1743, host89/97. The separately executed heap probe passes; the observation suite's ignored probe is not counted as already executed.
- Comprehensive nested predecessor/U02/U03/renamed/retired traversal checks411 selector references, with exact current named pass lines or explicit approved retired dispositions. The narrower current/inherited set of229 also passes. See `EVIDENCE_BINDINGS.json` and `ADDITIONAL_BINDINGS.json`.
- Actual stock continuation/rewind evidence has7 process starts each and78/114 messages. Source-reviewed producer assertions and raw records cover result-before-ack, unfinished restart, selected/claimed restart, stale actor refusal, legitimate same-battle-identity rewind and terminal unavailability. Five native fault modes compare equal; actual stock WebSocket fault/reconnect/save protection/Quit/new-owner Continue evidence is current.
- All8 archives were checked member-by-member: current before370/after374/records2362/references91, and the externally bound U03 four archives367/367/1748/175. Independent actual fuzz0 forward then reverse restoration matches all member names and bytes exactly. See `INTEGRITY.json` and restore logs.
- R05 raw exit101 is preserved. Concurrent peak exceeds the inherited cap by269,128 bytes; this is exactly the previously approved warning. Every other R05 hard guard passes. The corrected006 comparator binds the current005 receipt/raw, while incorrect earlier binding and failures remain preserved. No allocation-instrumented timing is treated as a latency gate. PC576 immediate non-timing equality, original source-bound comparator, plain snapshot/export original caps and separate allocation samples support the numerical judgment in `NUMERIC_REVIEW.json`.
- Thirteen service/config input hashes and all three reused audit/service artifacts are exact. Reuse is limited to unchanged service plumbing; changed producer lifetime has current tests and five-mode evidence. No fresh deployed service or browser verification is claimed.

## Dirty attribution, limits and next boundary

The declared target uses actual dirty before/after bytes; it does not attribute every working-tree change since HEAD to U04. The original initial368-source archive is preserved and combined with explicit supplemental Rulebook and status baselines to form the370-member before snapshot. Rulebook provenance is the unchanged immutable R08–R12 archive, and the documented U04 hunks account for the delta; status preserves the pre-U04 U03 closure bytes. Target restoration and membership agree. Unrelated dirty work and all historical failed attempts/audits remain untouched. Entry/exit git status is retained as supporting context, not used instead of file hashes.

This audit makes no Unity UI, remote deployment, Windows crash, generalized power-loss, future schema or20-minute load-test claim. Strict validation is structural state validation, not save authenticity. The retained R05 warning remains visible rather than silently rebaselined.

After recording exit currentness and the closed audit artifact manifest, this audit is closed and the target freeze is released **only for the owning correction GEX-10 and fresh re-audit** under existing authorization. The required Complete/High prerequisite for U05 is not satisfied. There is no separate GEX-30 for this single phase, and no dependent implementation may begin from this verdict.
