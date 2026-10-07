# GOAL-CRSO-S1 independent completion audit — Not Complete

2026-10-04 · GEX-20 · GS-CRSO-001 / GOAL-CRSO-S1 / PHASE-CRSO-S1 / HA-CRSO-001

**Verdict: Not Complete. Confidence: High in the blocking finding.** F01 permits the new public persistence rollback capability to undo an active battle. S2 is not eligible. Return to the owning S1 GEX-10 correction pass, then a fresh independent GEX-20. No implementation was changed in this audit.

## Authority and exact target

This separate implementation-nonparticipant reviewer `/root/u02_s1_audit` was assigned as GPT-6 Astra / High under the user's [continuous authorization](../../../../roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md). Applied GEX revision `2026-09-13-gex-r7`, especially §5, §6.4, §6.7 and GEX-20. Audit work is limited to this directory and isolated `/tmp` probes/restoration. Scope is CONFIG/CORE/PORTS/S1VERIFY; S2 observation, exporter and Grafana are excluded.

Exact upstream: [FPF-CRSO-001](../../P2G_30_FREEZE_001.json), [PP-CRSO-001](../../P2G_40_PHASE_PLAN_001.json), [PVR-CRSO-001](../../P2G_40_RECEIPT_001.json), [GS-CRSO-001](../../P2G_50_GOAL_SET.ko.md) and [HA-CRSO-001](../../P2G_60_HANDOFF_001.json). Frozen references and plan/receipt bindings were checked; requirements were read directly, including seven acceptances, V01–08/V12 and all ten required dimensions. Implementer PLAN/RESULT/NOTES/COVERAGE/PREDECESSOR_IMPACT were treated as claims and evidence locators, not verdicts.

- Target: [s1_001/MANIFEST.json](../../execution/s1/target/s1_001/MANIFEST.json).
- Manifest SHA256: `e0340068b2185b5c8a73502cad230659a60670ab7f1a5fcc0054b10064b945f6`.
- Product source digest: `8d5c757c6064856fde003bab3b92fb35b3129d4d4cc618beddebdc6042944017`.
- Source plus document digest: `62b0739c58fc9894b8184cbdb689d8e068db9d15f717b979e5c54bef6609ba54`.
- Repository `/mnt/f/work/simulator`; VCS `f318414fb0156a67e04c5479000c2a2683fc0c09`; 49 scoped changed paths.
- [Entry](ENTRY_CURRENTNESS.json) and [end](END_CURRENTNESS.json): **current / unchanged**. All 1,141 members, VCS kinds, artifact hashes and freshly enumerated product/record/reference membership match. Audit output and isolated builds are excluded. Existing dirty/untracked work is preserved.
- Independently restored before → after → before with `patch --fuzz=0` in `/tmp`, including exact added/removed membership and bytes. SOURCE_AFTER, RECORDS and REFERENCES archives also restore exactly. See [mechanical evidence](EVIDENCE_REUSE.json), [forward](restore_forward.log), [reverse](restore_reverse.log), [verification code](verify_evidence.py).

## F01 — active-battle restore bypasses the non-battle capability boundary

**Severity: completion-blocking. Owner: GOAL-CRSO-S1 / WP-CRSO-CORE, with PORTS and S1VERIFY impact.**

`CoreApplication::restore_persistence_rollback` at `core_enhanced/src/application.rs:1083` checks application owner and next-attempt revision, but does not reject a currently active battle. The internal `try_restore_active_run_transaction_rollback` routes into the general application transaction; its draft restore replaces the current state with the captured non-battle state. The forced-pause route check is not a general active-battle refusal.

The [canonical runtime contract](../../../../core_runtime_contract.md#trusted-runtime-safety-profile) explicitly requires rejection of both capture and restore during an active battle. TD03 and the S1 Goal require an opaque **non-battle** persistence capability that cannot become an active-battle safety/authority bypass. Capture admission alone does not enforce the restore boundary across a state transition.

The independent [public probe](rollback_battle_probe.rs) loads the existing external R06 content pack and executes the first six real preparation commands at seed77. It captures immediately after `SelectMapNode`, then successfully executes `ConfirmEnterNode`. The application is now in an active battle, and a new capture correctly returns `None`. Consuming the pre-entry token nevertheless returns `Ok(())`, removes the battle and makes the entire run snapshot equal the pre-entry snapshot. The [raw log](rollback_battle_probe.log) ends with the expected-refusal assertion failure, exit101. [Probe identities and invocation](ROLLBACK_PROBE.json) preserve compiler/link artifacts, source and log hashes. The source-level missing guard independently establishes the defect; the probe uses existing release rlibs without modifying or rebuilding the target.

The existing `first_public_battle_entry_uses_host_work_limit_and_active_battle_has_no_rollback_token` checks **capture after entry**, not **restore using a token captured before entry**. Foreign/stale/duplicate token tests likewise do not cover this transition. Their success cannot close F01.

Required correction: enforce the frozen non-battle restore authority at the actual restore boundary, preserving live state on refusal. Add the precise capture-before-entry → successful battle entry → restore-refusal regression and confirm the active battle remains unchanged; check relevant host persistence failure behavior across entry, existing valid non-battle rollback, monotonic profile/peak preservation, stale/foreign/duplicate token rejection and allowance/first-fault behavior. Re-run affected checks and the owning final baseline as required, freeze a successor target and request fresh independent audit under the existing continuous authorization. Do not merely weaken documentation or mark the present successful rollback as expected; a policy change would require its P2G owner.

## Condition and dimension disposition

The following PASS rows are bounded review findings for the stated seams, not a positive aggregate completion verdict.

| Frozen acceptance / validation | Independent disposition and evidence |
| --- | --- |
| PE01 / V01 | Default providers read the single embedded RON; engineering ceilings remain separate. All38 number parity, no-file/explicit effective identity and real default CLI evidence verified. No new gameplay or RSS cap found. PASS for this seam. |
| PE02 / V02 | Strict private DTO, bounded bytes/nesting, checked target conversion, field-aware structural errors, bounded host/CLI I/O before content loading; explicit invalid profile does not fall back. Current parser and host selectors plus real invalid CLI/startup evidence support PASS. |
| PE03 / V03 | Immutable generation-owned Arc, distinct raw/effective identity, semantic identity preserved, profile-specific catalog plans/evaluator limits and concurrent loader tests. No mutable global profile found. PASS for reviewed identity/isolation seam. |
| PE04 / V04 | Native/history settings are installed in BattleCore before initial execution; checkpoint resource extent is checked against current host policy before commit. Restore/save preservation tests exist, but public persistence restore crosses into active-battle undo: **FAIL F01**. |
| PE05 / V05/V08 | Profile-aware CLI, authoring admission/trial helpers, typed legacy explicit-limit API and real server command adapters are present. Trial replacement and nested allowance tests pass. The required opaque non-battle host boundary is incomplete: **FAIL F01**. |
| PE06 / V06 | Parser profile is intersected with acquisition limits; catalog compilation and retained plans use selected expression limits. Typed ExprLimits has no invented upper cap; preview/evaluator semantics remain shared. Current expression/query tests and public CLI evidence support PASS for this seam. |
| PE07 / V07 | Selected count/byte/span retention applies to construction, import and streaming deserialize. Dirty mutation invalidates accounting; result statistics remain outside history. The 1000ms battle under a 1ms/1event window does not end early and retains final stats, with cursor expiry. PASS for reviewed retention seam. |
| V08 public boundary | CoreApplication replaces private GameCore/BehaviorResult use; actual host bootstrap/command/reconnect, public API/CLI and negative compile consumers are evidenced. **Incomplete due F01** despite successful aggregate tests. |
| V12 regression and prerequisites | Current full suites and inherited selectors are verified. Cost and public predecessor evidence remain available with warnings preserved. No new aggregate predecessor restoration or S1 Complete verdict is issued while the new public authority defect remains; see predecessor section. |

| Required dimension | Disposition |
| --- | --- |
| success | Valid explicit/default CLI/host/API and first-entry paths are evidenced; aggregate completion withheld. |
| invalid_input | PASS for strict settings boundaries and explicit bad-file startup/publication rejection. |
| failure_rollback | **FAIL F01**: active-battle state can be replaced by a pre-entry token. |
| retry_idempotency | Existing foreign/stale/double/revision-exhaustion tests pass; transition authority remains **FAIL F01**. |
| ordering_concurrency | Concurrent immutable profiles, original nested-ledger tests and default gameplay oracles pass for reviewed seams. |
| boundary_empty_max | Parser size/depth/zero/overflow/ceilings, legal large typed expression limits and history empty/minimum/span tests pass. |
| disconnect_restart_persistence | Existing host persistence/reconnect and monotonic tests pass; capability transition gap prevents full acceptance. |
| live_data | External official/R06 packs and actual local server processes are present; no fixture-only production claim. |
| producer_consumer | Profile flows loader → generation/catalog → application/BattleCore and authoring/CLI/host; rollback consumer authority is incomplete. |
| legacy_fallback_absence | No private Core re-export or invalid-profile fallback found; newly exposed rollback creates forbidden active-battle authority, **FAIL F01**. |

## Evidence reuse and predecessor impact

[Verified evidence reuse](EVIDENCE_REUSE.json) recalculates raw-log hashes, exact source identity, test totals and selectors. Final default/test-fixtures suites each contain **1700 passed / 0 failed / 0 ignored**; server all-targets has **76 / 0 / 0**. All S1 selectors, all **88 inherited U01/R06/R07 condition selector rows** and **7 F01 selectors** are present in both Core suites. Check, event_log_checks, Clippy, fmt, release and public receipts were compared to current product bytes. Earlier receipts differ only by the three explicitly captured cfg-test/integration include inputs; final full suites include them, while earlier production/public runs do not compile those test inputs. This is limited evidence reuse, not a claim of newly executing those suites.

The actual producer/consumer delta was reviewed: default delegation; content parse/catalog profile propagation; battle construction and reset; diagnostic history; checkpoint resource admission; public application rollback; server DTO/command/persistence adapters. No fixture-name production branch, arbitrary timestamp workaround, new expression authority or silent invalid-profile fallback was found. F01 is a source-level authority defect that passing tests missed.

[Predecessor review](PREDECESSOR_REVIEW.json) records changed shared paths, every inherited selector row, prior exact target and the scope of remaining review. U01's cancellation owner/queue and R06/R07 effect/resource algorithms are not changed by this delta; defaults remain the same. Current U01 5-case oracle/trace and 9-pack45-command evidence, PC36 576 rows/31 successes/5 typed refusals, and immediate-U01 non-timing/allocation parity support preservation of those established paths. Existing R06 requested-detail differences remain80. These implementer raw results remain distinguishable from this audit's new probe; no full test rerun was performed merely because the reviewer is fresh.

The prior U01 Complete/High remains an immutable historical verdict. This failing S1 audit does **not** claim unconditional restoration of every predecessor integration contract over the new public authority. The successor correction audit must finalize relevant predecessor currentness/impact before any S1 acceptance or S2 start. It may reuse verified unaffected condition evidence rather than repeat unrelated full historical audits.

Known limitations remain: R05 raw exit101 is not a pass; peak9,354,169 versus cap9,030,188 is the existing `warning_concurrent_peak_only` policy classification, excess323,981 bytes, with no other hard guard failure claimed. Clippy warnings and inherited `-A clippy::never_loop` / `eq_op` allowances are preserved. No zero-warning, deployed Unity, remote-service, S2 or20minute-run claim is made.

Audit-local checker bookkeeping: its first archive comparison failed because Python Path ordering differs from the manifest's relative-string ordering; correcting the audit-only comparator made exact restoration pass. No archive or source bytes were changed to satisfy it. This is separate from the actual product F01 probe failure, which remains preserved and unresolved.

## Closed pass

**Not Complete; F01 unresolved; S2 blocked.** Correction owner is S1 GEX-10, followed by a new independent GEX-20 on a successor target. Existing user authorization already permits that transition; this report requests no checkpoint approval. End-currentness establishes that this finding and verdict apply to the original frozen target without audit-time mutation.
