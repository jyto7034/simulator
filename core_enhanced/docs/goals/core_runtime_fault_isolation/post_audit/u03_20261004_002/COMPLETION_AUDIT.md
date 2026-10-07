# U03 successor independent GEX-20 audit

**Verdict: Not Complete. Confidence: High.** GS-CRFI-001 / GOAL-CRFI-01 / PHASE-CRFI-01 remains open. This is a single-phase whole-Goal-Set audit; no GEX-30 or U04 activation is issued.

The requested GPT-6 Astra / High reviewer remains an implementation nonparticipant. Exact policy basis is FPF-CRFI-003 / PP-CRFI-002 / GS-CRFI-001 / HA-CRFI-001. Target `execution/single_goal/corrections/f01_origin_001/target/u03_002/MANIFEST.json` has SHA256 `453c015f3bc19cb122356c4a99d1812c84229e3e030105d7456c087126193541`; product SHA256 `f062f046ff719b3268361615ce21124b3a03519d2104aa18579de7b27dc77e73`; source/document union `fb93c18265005317ca9e80faffc411f8c075de9e903f59de49f283ffaeaede7e`.

## F01 disposition

The prior employee-reconciliation/lifecycle finding is corrected at trusted producers with fixed typed invariant sites. Current public tests cover four origins, manual deployment, complete root retention, first authority and repeated gates; ordinary missing-unit/invalid-action controls remain benign. This does not establish all internal-origin classification: the successor review confirmed F02 below.

## Blocking F02 — two manual runtime failure origins remain retryable

1. `src/game/battle/core/sustained.rs:1123–1125` resolves the trusted sustained record's present Goodbye reference and maps a missing definition to ordinary `InvalidAction`. Public reinput does not pass through the new event/boss trusted wrapper before that return.
2. `src/game/battle/core/sim.rs:948–953` computes admitted cast deadline using checked `time_ms + 1` or `time_ms + focus_time_ms`, but maps overflow to the same ordinary error. The action has already been accepted into the candidate; the manual application path lacks a trusted classification for this checked arithmetic failure.

Both were independently executed through real public CoreApplication commands in a disposable extraction of the exact frozen after archive. Only test helpers and tests were appended; existing production functions were not altered. Each probe executed one test and failed the required terminal assertion (raw 101):

```text
ordinary_stale_activation=invalid_action ordinary_invalid_target=invalid_action subsequent_advance=ok fault_present=false
trusted_lookup_first=invalid_action fault_present=false
trusted_lookup_second=invalid_action fault_present=false
focus=0 first=invalid_action second=invalid_action fault_present=false
focus=100 first=invalid_action second=invalid_action fault_present=false
```

The missing reference and maximum internal clock are deliberate internal negative fixtures, analogous to the contract's checked revision/reconciliation negatives. They do not claim ordinary external input can corrupt trusted state or naturally reach the maximum clock. The failure under review is the returned internal error's classification: PE01/V02 requires a terminal fault; PE02 requires first-cause retention and no repeated execution. Neither is satisfied here. Existing rollback protects the candidate, but later application/host/writer/observer gates cannot enforce terminal policy without a capsule.

**Correction owner:** GOAL-CRFI-01 / TD02, actual sustained-reinput and admitted-cast producers. Distinguish present trusted reference loss and checked deadline overflow with typed origin. Preserve stale activation, invalid target, Hello-active and valid unsupported reinput refusals; do not globally promote `InvalidAction`. Review equivalent trusted lookup producers. FINDINGS.json distinguishes confirmed cases from the unprobed tactical-swap lookup candidate and defines revalidation requirements.

## Verified evidence and remaining scope

Entry/end currentness records bind the exact target. All 1728 frozen members were freshly checked for bytes/kinds/membership. Archives unpacked exactly; actual fuzz-zero forward/reverse patch application reproduced 367-file before/after sets. All 36 declared current receipt/source/log bindings matched. Existing full matrices were reused without broad reruns; the two new bounded probes were run only for concrete classification gaps. Raw failures and prior audit history are preserved.

The broader successor review included original policy meanings, public/private transaction boundaries, host/session/timer/management/writer/recovery consumers, observation terminal cleanup, canonical contract and inherited-condition mapping. The pass stops at F02 before product repair, as required. CONDITION_REVIEW.json preserves all 12 acceptances/V01–V12/10 dimensions; PREDECESSOR_REVIEW.json preserves all 100 inherited conditions and 31 R06 maps. Their complete independent semantic/transitive-evidence closure remains a successor-audit requirement and is not inferred from implementation PASS totals.

The exact remaining work is F02 in FINDINGS.json, followed by corrected source-bound validation, a new frozen target and full successor GEX-20 closure. No product/source/frozen evidence/status file was modified. No new waiver, numeric cap, 20-minute, remote-service or Unity claim is made.
