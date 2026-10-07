# U06 independent GEX-20 completion audit

Verdict: **Not Complete**. Confidence in the finding and target attribution: **High**. This is not a `Complete / High` issuance. One completion-blocking defect, **U06-F01**, remains in the U06 basic-Heal usefulness contract. Return to **GOAL-CBST-01 / PHASE-CBST-01, correction GEX-10**. Dependent U07 must not begin on this verdict.

The auditor is the separate implementation-nonparticipant GPT-6 Astra / High reviewer requested by the user. This fresh pass reviewed the frozen implementation; it did not implement or design a fix, modify source/data/canonical documents, edit the target or shared execution summaries, or delegate. Audit evidence was written only in this audit directory; temporary restoration and probes used `/tmp`, and compilation used the existing build cache. The parent froze target mutations for this pass. Existing continuous authorization permits the parent to start the correction after this audit closes, without a new stage permission request.

## Exact target and governing contract

- Goal Set `GS-CBST-001`, Goal `GOAL-CBST-01`, Phase `PHASE-CBST-01`; P2G freeze `FPF-CBST-001`, the P2G-40 plan, P2G-50 Goal Set and P2G-60 handoff in `core_battle_starter_tactics` are the accepted consumer chain.
- This is `single_phase`. The entire Goal Set, broad regression and applicable final baseline are assessed here. There is no separate GEX-30 that can discharge a missing condition.
- Governing execution rules: current `core_enhanced/docs/goal_execution_pipeline(1).md`, gex-r8, sections 5, 6.4, 6.7 and GEX-20. Policy meaning comes from the frozen handoff, `WORKING_POLICY.ko.md`, `IMPLEMENTATION_DECISIONS_20261005.ko.md` and canonical `game_rulebook.md` / `game_rules/`. Historical pause text is not current stop authority.
- Exact target: `core_enhanced/docs/goals/core_battle_starter_tactics/execution/single_goal/target/u06_001/MANIFEST.json`.
- Manifest SHA-256: `ce3789acbe476671b2791f59df99346e1660190089d71efe0776f20265ccd1a2`.
- Product SHA-256: `7db70db30e290a83db502b2fc92ad08b94ef5b6a9f70dec3939dcd89e0bf976a`.
- Combined source/data/canonical-document digest: `d858b183087eb45b0febb89a1e55a02f74fa559c050439a9a571a8d2aa3567f9`.
- Git HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09` is contextual, not the target identity. Membership contains 2,671 distinct paths: the source group has 392, records 2,215, references 72, with overlapping groups and separately identified external archives.

[CURRENTNESS_BEGIN.json](CURRENTNESS_BEGIN.json) records an independent byte/size check of every membership member, target artifact hashes, current record/product enumeration, membership-group union, and tracked/dirty attribution. The auditor checked every archive member against its expected manifest and restored SOURCE_BEFORE plus FORWARD.patch with fuzz zero to exact SOURCE_AFTER, then reversed to exact SOURCE_BEFORE. All passed. Historical U05 external archives were independently member-checked: before375, after377, records2992, references166; they were not assumed valid from an outer archive digest alone.

Dirty attribution is 37 tracked/clean, 188 tracked/dirty and 2,446 untracked across the full target union. The scoped before/after and predecessor records distinguish existing work from the U06 delta (75 modified and 7 added source/canonical paths). Unrelated dirty work was preserved and is not credited as U06 implementation. All current reference bytes match the frozen record. The initial audit restoration checker sorted `Path` objects differently from manifest string ordering; the resulting false mismatch is retained in `CURRENTNESS_BEGIN_SORTING_PROBE.json`, the checker was corrected, and exact restoration subsequently passed. This was an audit-checker issue, not target drift.

The ending check and closure are [CURRENTNESS_END.json](CURRENTNESS_END.json) and [AUDIT_CLOSURE.json](AUDIT_CLOSURE.json). The ending check completed at 2026-10-04T18:47:48.485365+00:00. Beginning and ending identity agree, attribution and membership are unchanged, and both checks have zero errors. Currentness is **current / unchanged**.

## U06-F01 — zero-effective basic Heal remains an eligible automatic action

**Owner:** U06 `GOAL-CBST-01`, Basic Heal producer and its query consumer. This is an in-scope implementation defect, not an unresolved product-policy choice or a predecessor-owned regression.

**Required meaning:** `WORKING_POLICY.ko.md:51`, PE-CBST-05, requires alive/deployed/recoverable allies with missing HP, excludes full-HP/unrecoverable/ineffective targets, and rechecks current eligibility at start and release. Basic healing is useful HP recovery. The applicable query contract exposes Core eligibility rather than independently inventing it.

**Current source:** `core_enhanced/src/game/battle/core/basic_heal.rs:19` checks activity, same owner, missing HP and range, but not whether the current authored Heal yields positive recovery. Automatic selection uses that predicate at line40. `resolve_basic_heal` uses it again at line145 and computes `flat + spell_power * spell_power_percent / 100` at line164. Thus eligibility can succeed when the amount is zero. `core_enhanced/src/game/battle/core/targeting.rs:77` uses the same predicate to populate public `eligible_target_ids`. `core_enhanced/src/game/data/abnormality_data.rs:116` validates Heal delivery and footprint, and accepts the zero coefficients used by the probe.

**Independent reproduction:** the actual frozen release `author_content validate` accepts an otherwise byte-identical copy of the frozen official positive pack after changing only the training single-Heal staff from `(flat:8, spell_power_percent:100)` to `(flat:0, spell_power_percent:0)`. Exit code is 0. The release binary SHA-256 is `fc6d74e0931f733972caee8f032a6b8dc464af9dd72cd03f832ae333c3b1257c`, matching the final implementation binary evidence. The preserved input, exact command, all input hashes, stdout/stderr and exit are in [ZERO_HEAL_PUBLIC_REPRO.json](ZERO_HEAL_PUBLIC_REPRO.json) and `zero_heal_public_input/`. The earlier temporary-input run is also retained in `ZERO_HEAL_PUBLIC_VALIDATE.json`.

The audit-local crate uses current `game_core`, actual strict `BasicAttackDef` decode, normal scenario admission and the current automatic scheduler. It explicitly supplies a synthetic actor/profile and an injury fixture of HP50/100; it does not claim to be a normal grant/host test. After 400ms:

| Authored/current amount inputs | Automatic starts | Positive HP events | Final HP |
|---|---:|---:|---:|
| flat0, percent0, spell power0 | 4 | 0 | 50 |
| flat0, percent100, spell power0 | 4 | 0 | 50 |
| positive control: flat10, percent0 | 4 | 4 | 90 |

All three definitions pass strict decode. Events include repeated AttackStart and AttackResolve for the ineffective cases. [ZERO_HEAL_RUNTIME_RECEIPT.json](ZERO_HEAL_RUNTIME_RECEIPT.json), [ZERO_HEAL_RAW.json](ZERO_HEAL_RAW.json) and `probe/src/main.rs` preserve the invocation, source/log/binary hashes and complete event observations. All non-probe lockfile package versions and sources match repository Cargo.lock. Failed probe construction attempts are retained and excluded from the finding; the successful final probe has a positive control and a pinned dependency lock.

The public-query impact is established by the directly reviewed shared predicate; the audit did not run a separate zero-amount host query. Existing positive official starters are not alleged to fail. Merely accepting zero coefficients is not independently classified as a schema violation: if that authored definition remains valid, current automatic eligibility still must honor the ineffective-target contract. The nonzero-percent/zero-current-stat case shows why a result based only on one all-zero schema input would be incomplete.

**Affected completion seams:** V05 fails; V15's Core eligibility consumer fails; V19's live/negative boundary coverage is incomplete; V20 cannot satisfy the no-unresolved-finding and Complete/High gate. Success, boundary/empty/max, live-data and producer-consumer dimensions cannot be fully closed.

**Required correction validation:** establish the frozen usefulness meaning on supported single and area Heal paths at action start and release, including zero effective amount, positive controls, current-stat changes, target loss/fullHP and no retargeting; validate that public eligibility agrees with the actual action; retain pure-heal/no-damage-proc and common CC/stance/work/Pause semantics. Include actual loader/authoring acceptance or typed refusal as appropriate to the existing contract, and actual public consumer evidence. This specifies required outcomes, not a fix design. Reassess changed source/native accounting/clone/queue/cost and predecessor consumers by impact, preserve original failures/oracles, freeze a new target and obtain a fresh independent GEX-20. Do not reuse this target's positive condition judgments blindly after a code change.

## Condition-level judgment and evidence validity

[CONDITION_JUDGMENTS.json](CONDITION_JUDGMENTS.json) carries every exact U06 policy revision and acceptance, selectors and receipt locators, the auditor's semantic judgment, all 141 inherited conditions and all 31 separately tracked R06 acceptances. The 31 original acceptance texts and IVA links were compared byte-for-byte as JSON values to original R06 coverage. This matrix is an index to reviewed evidence, not a substitute for source review.

| Seam | Judgment | Reviewed behavior / limiting reason |
|---|---|---|
| V01 TP | Pass | Common owner, cost/ceiling, shortage atomicity and battle-time recovery. |
| V02 deployment | Pass | Weaponless condition priority; normal roster10, deployed8, ninth refusal and freed slot. |
| V03 withdrawal | Pass | CC/work acceptance, fixed redeploy HP, new entity resonance, pending versus released lifetime. |
| V04 weapon/profile | Pass | Strict profile/delivery, real equipment consumers, special gun and default Damage compatibility. |
| V05 basic Heal | **Fail F01** | Effective-zero target/action boundary; positive/fullHP/area/order evidence remains valid. |
| V06 six starters | Pass | Normal grants/recruitment/copy transfer and six actual active outcomes. |
| V07 ordinary stance | Pass | New-action gate preserves started releases; refusal and Ground/Platform effects. |
| V08 Rest/CC | Pass | Full interval, pause/lethal ordering, tile-kind reset and facing preservation. |
| V09 relocation | Pass | Precommit validation, independent mobility gate, identity/delivery preservation and atomicity. |
| V10 forced Defense | Pass | 1499/1500ms boundary, stale/cancel cleanup and work exceptions. |
| V11 swap | Pass | Both sides validated before commit, one TP5, each cooldown, read-only preview. |
| V12 work | Pass | Object-owned progress, eligible contribution and one completion; fullHP/CC/Pause/exits. |
| V13 blocking | Pass | Swept contact, continuous position and retained links; capacity and invalidation. |
| V14 block range | Pass | Basic Damage relation only; other ranges unchanged and Heal uses ally route. |
| V15 query/preview | **Fail F01 consumer** | Query exposes zero-effective target as eligible through the common predicate. |
| V16 focus | Pass | One speed restore owner, manual pause and stale/terminal cleanup. |
| V17 pause/settings | Pass | Layered freeze/restore, command order, host-time distinction and explicit resume. |
| V18 lifecycle/store | Pass | Clone/roundtrip/rollback/reconnect and actual Continue/rewind/original preservation. |
| V19 evidence/performance | **Partial** | Current broad/public/cost evidence is valid; F01 boundary missing. |
| V20 closure | **Fail** | Normal command acceptance passes; unresolved F01 prevents the required audit gate. |

[EVIDENCE_VERIFICATION.json](EVIDENCE_VERIFICATION.json) independently binds 42 selected receipts to hashes, raw logs, command-input snapshots and current product identity. All 161 mapped condition rows have their named successful selectors in the selected current logs. Exact final suites are Core1791 default,1791 fixtures,1800 observation; host92 default and100 observation. The Core counts include integration tests (lib1734/1743 plus57); they are not incorrectly advertised as library-only counts. The one ignored heap test is explicitly run separately and passes. Current fmt/check/event-log/clippy and actual release/public commands are source-bound. The auditor reused these validated broad executions and did not rerun whole suites merely because this is a fresh audit.

Actual public evidence includes authoring/schema, old oracle31 successes/5 typed refusals, all38 default numbers, API21 commands, CLI13 positive/6 negative consumers, profile/boundary probes, Continue7 processes/78 messages, rewind7 processes/114 messages, real WebSocket/store behavior and five fault modes. Normal six-set tests use actual grants/equipment/copies; readiness, injury and opponent-position fixture setup is explicit. Existing generic gates provide shared combination coverage; neither the implementation nor this audit claims every Cartesian combination was separately executed.

All ten required dimensions are assessed, with no invented N/A. Failure/rollback, retry/idempotency, ordering/concurrency, disconnect/restart/persistence, legacy/fallback absence and the tested strict invalid-input routes retain valid current evidence. F01 prevents full success, boundary/empty/max, live-data and producer-consumer closure. Any correction changes must be assessed against all affected dimensions, even those passing on this target.

## Predecessors, costs and source guardrails

The immediate predecessor is the U05 `core_resonance_lifecycle/post_audit/u05_20261004_002` independent Complete/High audit and exact `target/u05_002`. Its original source baseline, archived records and references were verified; expected U06 deltas are attributed rather than pretending the old product digest still equals the current product. All 141 inherited requirements retain their original meaning, except explicit already-approved policy changes, and current affected-consumer evidence is recorded condition by condition. The 15 PE-CRRP and14 PE-CRSL rows that do not repeat an older `original_meaning_basis` use their exact immediate predecessor rows; this is a valid explicit chain, not missing evidence.

[PREDECESSOR_MEANING_CHECK.json](PREDECESSOR_MEANING_CHECK.json) validates the four original R06 meaning sources and all437 adapted harness/input files:419 old inputs remain byte-identical and18 scripts have accounted adaptations. Source review of the comparator found no rewritten expected numerical outcomes. All576 original PC workload rows retain gameplay equality and every non-timing cost field equality after the explicit source-derived40-byte RuntimeUnit bucket and80-byte initial ScenarioUnitSpawn allocation growth. These account for actual layout changes; they do not alter numeric caps. Original and immediate-U05 comparisons are retained. Historical failed hypotheses were not removed.

R05's raw exit101 is not called a raw pass. The final classification identifies only the inherited concurrent-peak warning:9,635,797 against9,030,188, excess605,609; other hard guards are zero. Event-heavy271,367 remains below276,174; retained265,798 and concurrent retained4,196,928 satisfy the existing contracts. The original allocation-instrumented latency is not a gate; shared-host timings remain observations. Separate public cost and allocator guards pass. The inherited20-minute stress exclusion remains explicit; no new exception or higher ceiling is authorized by this audit.

Actual service evidence reuse is limited to the exact13 unchanged Cargo/config/observation inputs in SERVICE_REUSE. The current changed producers are covered by observation/host/heap/fault and public recovery evidence. The earlier S2 service record contains4,197 actual metric samples, finite live/render frames, positive rejection reasons, selection1→2→1, and termination clearing active selection; it explicitly distinguishes the originally mislabeled duration panel from the actual later render. This audit accepts that captured local-service evidence for unchanged plumbing and does not claim services are currently running or that a deployed service was tested. Exact service/audit references and hashes are frozen target members.

Direct source review covered basic Heal/attack targeting and usefulness; current heal/damage/lifesteal and sustained modifier accounting; ordinary/forced stance, Rest and work; relocation/swap; blocking, swept movement application and cancellation; Core interaction/preview and world application routing; starter grants/roster/equipment/skill schema; native state layout/CloneShape and action/delivery accounting; current normal-starter and reconnect tests; and relevant canonical rule/document changes. Source authority remains in Core and explicit data. Reviewed new state is cloned/accounted with real consumers; public query charges its retained storage; released versus pending delivery and stale/cancel/terminal paths retain explicit owners.

The source-level guardrail review found no separate production fixture-name/probe branch, future-only abstraction, duplicate presentation authority, broad catch-all recovery, arbitrary delay workaround, disabled regression or unaccounted golden replacement. Historical fixture extraction preserves old sword/oracle inputs without claiming old fixture behavior is the new normal six-set behavior. Changed old assertions correspond to approved policy changes (work completion Attack, relocation no longer requiring Defense, focus speed while paused), not unexplained expectation weakening. Implicit Damage serialization omission preserves the old V11 canonical checksum and old-save Continue/re-export contract. F01 remains a real missing eligibility guard and is not excused by these otherwise satisfactory structural checks.

## Closure and limits

This audit covers local Linux Core, CoreApplication, real host actor, stock WebSocket and process/file boundaries. Unity, future deployment and individual final bosses remain outside U06. Valid historical evidence is reused with exact references and current producer-impact checks; this report does not claim every unchanged historical implementation file was reread from scratch. No inaccessible mandatory evidence or unrelated new blocker was identified.

The only open finding is U06-F01. The overall result is **Not Complete** with exact owner **GOAL-CBST-01 / PHASE-CBST-01**. Close this audit, correct in GEX-10 under the existing authorization, refresh affected evidence and the frozen target, then perform a fresh independent GEX-20. U07 is not eligible until the new current target earns Complete/High without unresolved findings.
