# S1 correction re-audit — Not Complete

2026-10-04 · GEX-20 · GS-CRSO-001 / GOAL-CRSO-S1 / PHASE-CRSO-S1 / HA-CRSO-001

**Verdict: Not Complete. Confidence: High.** Original F01 is closed, but the correction exposes an unhandled real host persistence transition, F02. S2 remains ineligible. No implementation, frozen record, source or prior audit was modified.

## Exact scope and currentness

Reviewer `/root/u02_s1_audit` remains implementation-nonparticipant, assigned GPT-6 Astra / High. This is a separate fresh read-only pass under the [continuous user authorization](../../../../roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md), GEX `2026-09-13-gex-r7` §5/6.4/6.7 and GEX-20. Existing FPF-CRSO-001, PP-CRSO-001/PVR-CRSO-001, GS-CRSO-001 and HA-CRSO-001 remain unchanged. Requirements are the original seven S1 acceptances, V01–08/V12 and ten required dimensions; no S2 work is audited.

- Target: [s1_f01_001/MANIFEST.json](../../execution/s1/corrections/f01_active_battle_001/target/s1_f01_001/MANIFEST.json).
- Manifest SHA256: `4ed7c32c15891a67ec4580a8ad7a11b409f92c707f1e73ffd4da84dee1b2fece`.
- Product digest: `aeb2671288385c1eb541781baa03631fa59dca089cfb341a6b13485845f46d32`.
- Source/document digest: `22215b467f824a72cdb538085dcfcc07207f0a9cabff0c0acb883cbadc97d376`.
- Repository `/mnt/f/work/simulator`, VCS `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- [Entry](ENTRY_CURRENTNESS.json) and [end](END_CURRENTNESS.json) are **current / unchanged**, all1,209 members. Membership is freshly enumerated, including new files, and bytes/VCS kinds/artifact hashes match. Prior audit001 is included in frozen references and preserved.
- Baseline bytes/membership exactly equal the previous S1 target's source-after. Only `application.rs` and `handlers.rs` differ. Removing the seven-line guard makes application bytes identical; all host bytes outside the expanded existing regression are identical.
- Independent `/tmp` forward/reverse patch application at fuzz0 and SOURCE_AFTER/RECORDS/REFERENCES archive restoration all match exact membership and bytes. [Evidence](EVIDENCE_REUSE.json), [checker](verify_evidence.py), [forward](restore_forward.log), [reverse](restore_reverse.log).

## F01 closed

The public restore method first checks owner/next-attempt revision, then rejects an active battle with `InvalidAction / active_battle_persistence_rollback` before revision advancement or private restore. Its general lifecycle predicate is consistent with the validated Core state. The private/non-Clone/non-Serde capability and legitimate non-battle rollback remain unchanged.

The current final host regression tests capture before entry, enter successfully, reject restore and compare both run and battle reconnect projections. The [independent public probe](rollback_battle_probe.rs) additionally consumes two pre-entry tokens at the same revision; both produce the explicit active-battle error and leave the battle/run/reconnect state unchanged. [Result](ROLLBACK_PROBE.json), [raw log](probe_run.log): compile0/run0. Compiler-artifact JSON from the exact current build selects the hashed libraries; the stale cached rlib from the earlier failed implementer probe is not used.

This closes the original unauthorized active-battle undo. It does not establish that every host caller is compatible with the refusal.

## F02 — persistence failure after a higher-peak battle entry reports rejection after commit

**Completion blocker. Owner: GOAL-CRSO-S1 / WP-CRSO-PORTS and S1VERIFY, with CORE integration as needed.** This is the host adapter's assumption about rollback availability across a state transition. Do not remove the correct F01 active-battle guard.

Actual source chain:

1. `core_enhanced/src/game/world/combat.rs:707` records certified power at combat entry. `RunPowerAuthorityState::record_combat_entry` raises the monotonic run peak when the current loadout is stronger.
2. `export_active_run_checkpoint` includes the current run peak alongside the last SavePoint payload. Therefore a successful battle entry can change the exported checkpoint even though no new SavePoint is created.
3. `game_server/src/game/player_game_actor/handlers.rs:417` compares before/after checkpoints after Core has executed and committed the command. When changed, it calls `store.publish`.
4. On publish failure, the host calls `restore_persistence_rollback` at line452 and propagates its error before normal command result/battle updates and the later profile-persistence path.
5. The corrected Core correctly rejects this restore because the command has already entered a live battle. The host consequently returns an error while the authoritative state has changed and durable monotonic metadata remains old.

The correction's unchanged-checkpoint argument only tested the first battle without an existing SavePoint. It is not a valid general proof for host entry behavior.

### Independent reproduction

[Core boundary probe](checkpoint_entry_probe.rs) uses the unchanged external R06 pack, seed77 and the first four existing preparation commands. Public admin commands create a valid SavePoint and grant Crimson Goggles; ordinary `CompleteNode` and `EquipItem` produce a supported saved/loadout state. These setup commands use release public APIs; no source, private state or serialized save is patched. It then selects an available combat node and confirms entry:

| Observation | Actual |
| --- | --- |
| Exported saved peak before entry | 204 |
| Exported peak after entry | 213 |
| Checkpoints equal | false |
| Battle active | true |
| Attempted pre-entry token restore | `active_battle_persistence_rollback` |

The [real host probe](host_peak_failure_probe.rs) feeds this pre-entry Core into the production `PlayerGameActor`, attaches a real Actix recipient, and installs an isolated `/tmp` `ActiveRunStore` with the valid peak204 checkpoint. Creating a directory at its `active_run.tmp` path injects the ordinary temporary-file write failure. The probe sends the actual `ExecutePlayerBehavior`/`ConfirmEnterNode` message and observes:

- Host command returns **Err `active_battle_persistence_rollback`**.
- A subsequent attach returns **two battle setup/update side messages**, proving the committed active battle remains.
- Durable store still loads **peak204**; the committed entry had peak213.

[Raw host log](host_peak_run.log), [Core log](checkpoint_probe_run_002.log), [hashes, inputs and observations](HOST_PERSISTENCE_PROBE.json), [compile invocation](host_peak_command.json). The host probe exits0 because its assertions demonstrate the defect; that is not a product PASS. The first audit-local Core probe used the wrong map DTO nesting, failed before entry, and is preserved as `checkpoint_probe_run_001.log`; correcting only that probe produced the valid reproduction.

Required correction: make the real host's persistence/publication handling coherent for battle entry that raises durable monotonic metadata, including write failure. Preserve the active-battle undo prohibition, the monotonic peak/profile contract, and the existing meaning of rejected versus committed commands. Do not swallow the restore error, skip the required durable peak update, or claim entry is always checkpoint-neutral. Choose an implementation consistent with frozen TD03 and the existing public transaction contract; if a new user-visible recovery policy is required, use its P2G owner rather than inventing it in this fix. Add a real-host regression for an existing SavePoint plus increased certified power and failed publish, then verify valid non-battle rollback and profile/peak handling, rerun affected final checks, freeze a successor and perform fresh independent audit.

## Whole-goal disposition and verified reuse

The prior [audit001](../s1_20261004_001/COMPLETION_AUDIT.md) bounded source reviews are reused only after exact delta/currentness checks. Seven acceptances remain accounted for:

| Acceptance / seams | Current disposition |
| --- | --- |
| PE01 / V01 defaults | Prior38-number/default/no-file parity review valid; correction does not change any setting or successful ordinary execution path. PASS for this seam. |
| PE02 / V02 invalid settings | Bounded strict parsing, field/path errors and prepublication refusal are unchanged and current full-suite selectors pass. PASS. |
| PE03 / V03 identity/isolation | Raw/effective identity, immutable generations, concurrent loaders and plan isolation unchanged. PASS. |
| PE04 / V04 constructor/restore | First-operation limits and host-profile save admission valid. F01 closed, but actual host entry/persistence boundary has **F02 FAIL**. |
| PE05 / V05/V08 API/CLI/host | Authoring/CLI settings and allowance/replacement semantics preserved. Host failure/commit semantics have **F02 FAIL**. |
| PE06 / V06 expression | Parser/compiler/retained evaluator profile and original legal typed ranges unchanged. PASS. |
| PE07 / V07 history | Count/byte/span/streaming import/dirty accounting/cursor/statistics and long-battle tests unchanged. PASS. |
| V08 public host integration | Correct new Core refusal is not yet handled by every real host caller. **Not Complete.** |
| V12 baseline/predecessors | Required current baseline and inherited selectors verified. Aggregate predecessor restoration and S1 completion remain withheld pending F02. |

All ten dimensions are applicable: success, invalid_input, ordering_concurrency, boundary_empty_max, live_data and legacy_fallback_absence retain their reviewed evidence; **failure_rollback, retry_idempotency, disconnect_restart_persistence and producer_consumer are incomplete because F02 leaves a rejected host command committed and durable metadata old**. No dimension is assigned an unsupported N/A. Existing command retries cannot be assumed safe after an apparently rejected entry; this exact recovery path needs owner validation.

[Mechanical evidence](EVIDENCE_REUSE.json) independently verifies all final receipt/log hashes and source-before/source-after identities. Core default and test-fixtures each **1700/0 failed/0 ignored**, server **76/0/0**. Current check/event_log_checks/Clippy/fmt/release/probe receipts pass. All S1 validation selectors, **88 inherited U01/R06/R07 condition rows**, and **7 U01 F01 selectors** occur in current successful tests. These test suites are verified reuse, not newly executed by this reviewer. The independent new executions are archive/currentness checks and targeted public/host probes.

The prior U0155-condition and R06(31acceptances→21integration)/R07(F01–F12) independent reasoning was read and compared to the cumulative S1 delta, not just this seven-line correction. Boss cancellation/queue, Flow/sustained/protection, R06 evaluator/source bindings and R07 copy algorithms remain unchanged; original defaults and schemas remain, while profile constructors/history/checkpoint admission/public ports were directly reviewed in audit001. All388 prior U01 source entries remain accessible;40 shared paths have the identified S1 changes, without additional unexplained delta. This supports reuse of unchanged algorithm evidence, but F02 affects the new public persistence consumer and prevents an unconditional current predecessor integration restoration. The next audit must close this host edge and finalize the preserved88-condition matrix before S1 acceptance; no prior historical verdict is rewritten.

Existing public profile evidence remains24 CLI cases/5 local host variants, boundary13 cases, public API/CLI21 cases and U01 oracle5/trace5/45 CLI commands under the prior source's exact receipts; guard-only impact permits their unaffected portions to remain evidence. PC36/R05 costs are reused only for byte-identical exercised algorithms:576rows,31success/5typed refusal,80 historical requested-detail differences. R05 raw101 and `warning_concurrent_peak_only` remain distinct; peak9,354,169 versus cap9,030,188, excess323,981 bytes. No new cost execution, raw pass, zero-warning or arbitrary latency gate is claimed. Inherited Clippy allowances and all failed receipts remain. No deployed Unity/remote or S2/Grafana completion is asserted.

## Closed pass

**Not Complete. F01 closed; F02 unresolved.** Return to S1 GEX-10 correction and a successor independent GEX-20. S2 must wait. Existing continuous authorization permits the correction without checkpoint approval. Frozen target and unrelated user work remain unchanged through the end check.
