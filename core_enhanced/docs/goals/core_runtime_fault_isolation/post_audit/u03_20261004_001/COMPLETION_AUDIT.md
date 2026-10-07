# U03 independent GEX-20 completion audit

**Verdict: Not Complete. Confidence: High.** GOAL-CRFI-01 / GS-CRFI-001 / PHASE-CRFI-01 remains open. This is the single-phase Goal Set audit; no GEX-30 or U04 activation is issued.

The explicitly requested GPT-6 Astra / High auditor did not implement U03. Current basis is FPF-CRFI-003, PP-CRFI-002/PVR-CRFI-002, GS-CRFI-001, HA-CRFI-001; schema repairs are closed and do not change semantics. Target is `execution/single_goal/target/u03_001/MANIFEST.json`, SHA256 `36640f3db8ff2a95528202a8aedc63f1a024038f0ccf985d1241de76a811ad25`; product `d844a3e32bed90d19e0ead462e2394f97d61ba884f27b3108d26ad3b797a6560`.

## Blocking F01 — internal reconciliation failure remains retryable

`ActiveBattleSession::reconcile_employee_attempt` (`src/game/world/combat.rs:111–134`) returns `UnitNotFound` when a deployed/incapacitated employee is missing from the trusted attempt roster. This is an internal consistency failure, not a caller selecting an unknown unit. The related corroded nonemployee case also returns `InvalidAction`. `application/fault.rs::cause` treats both variants globally as nonterminal; public `advance_active_battle_by` consequently returns the error without recording the battle fault.

A bounded probe was run in an isolated extraction of the frozen after archive. It reused the existing actual employee reconciliation negative fixture, then called public CoreApplication advance twice. The only source difference was one appended audit test; no production function was edited. It executed one test and failed as expected (raw exit 101):

```text
first_error=unit_not_found fault_present=false
second_error=unit_not_found fault_present=false
```

This violates PE01/V02's explicit terminal treatment of trusted internal failures and PE02's first-cause/no-retry requirement. Consumers only enforce terminal gates and observer cleanup when the capsule exists. This proof deliberately installs the existing inconsistent trusted-state fixture; it does not claim ordinary external input can create that inconsistency.

**Owner:** GOAL-CRFI-01, actual Core runtime producer/operation and TD02 classification. Add typed origin at the owning boundary; preserve ordinary missing-unit/invalid-action refusals and private/unpublished retry. Do not globally make those shared error variants fatal. Add the durable public regression and normal-refusal controls; review other overloaded runtime producers. Refresh affected feature/public/host/observer evidence and create a successor independent audit before closure. Exact correction/revalidation obligations are in `FINDINGS.json`.

## Evidence and limits

Entry and terminal currentness, archive hashes, fresh source/record/reference membership and Git kinds are checked in the adjacent records. All 1163 frozen union members matched; source archives contain 363 before and 367 after files. Actual fuzz-zero forward and reverse patch application reproduced exact membership and bytes. All 36 declared receipt/source/log bindings matched their current product; no full suite was rerun merely for a new audit. The independent negative probe is preserved in `classifier_probe.rs`, `classifier_probe.log`, and `CLASSIFIER_PROBE.json`.

The pass stops at F01 before implementation correction, as requested. `CONDITION_REVIEW.json` preserves all 12 acceptances, V01–V12 and ten dimensions; `PREDECESSOR_REVIEW.json` preserves all 100 inherited conditions and 31 R06 maps. Unfinished full semantic/transitive-input adjudication is marked deferred rather than PASS. Native-fault successes and total test counts cannot establish the missing internal-error behavior. No source, frozen reference, older audit or policy was modified. No new waiver, numeric cap, 20-minute, deployed-service or Unity claim is made.
