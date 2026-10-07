# R03 actual lifecycle source review 022

This is a bounded AUTORUN §8 source review/design-assistance report, **not a completion audit or PASS verdict**. Root is the sole production writer. I read the listed source/tests without executing them. INPUTS063/focused084 and later results belong to root's execution evidence; this report does not claim those results.

In the inspected production slice I did not identify a concrete reachable defect in the actual entry/time-zero/elapsed/terminal/transaction connection. The changed reconciliation design invalidates an existing producer witness; it does not create readiness from game state. One initial test fixture defect was observed and then corrected during this read window; a separate threshold-time assumption needs an oracle tied to actual execution. Test source drift limits any statement about its final version.

## 1. Concrete finding and test-source drift

**F01 — historical test fixture error; corrected in the second read.** The initially inspected tests/combat.rs (SHA 4f39dc57…, lines 12761–12780) iterated terminal_time=[0,10] and unconditionally authored TimeReached{time_ms:terminal_time}. BattleScenario::validate_terminal_outcomes explicitly rejects TimeReached{0} (battle/scenario.rs:688–694). That input cannot exercise successful production time-zero terminal preparation; changing the lifecycle implementation to accept it would change existing content semantics.

At 11:30:05 UTC, test SHA a1e30f9f… used AllRequiredEnemyGroupsDefeated for the zero branch and TimeReached only for the positive branch (12777–12786). This removes the identified invalid-zero fixture. Its actual runtime outcome still requires root's recorded execution; this helper did not execute it. Original and corrected hashes remain separately recorded.

**N01 — positive-time test oracle must distinguish threshold from actual finish.** The second read still advances 50 ms for a TimeReached{10} condition and asserts observed total/last time equals 10 (tests/combat.rs:12800–12808). Existing simulation matches TimeReached by current_time >= threshold (sim.rs:1251), checks terminal after an actual event bucket (1952–1954), or at the requested target when the next queue event lies beyond it/queue is empty (1980–1992). enqueue_scenario_events does not automatically enqueue every terminal threshold (1518–1544).

Therefore “threshold 10 implies actual finish 10 under advance(50)” is not a general runtime invariant. The fixture would need an actual evaluation at 10 to justify that oracle; I did not establish such a queue event from the bounded fixture read. A small robust fixture can advance exactly 10, or author/verify a real intervening bucket and compare with the actual returned execution time. Do not backdate the witness or change runtime terminal evaluation merely to satisfy the assertion. This is an oracle limitation, **not a finding that observed elapsed should use the authored threshold**.

At close (11:31:19 UTC), tests/combat.rs had changed again to SHA 9558b813…; that version was only rehashed, not reviewed. Accordingly F01 is resolved only for the explicitly inspected a1e30f9f version, and N01 describes that same intermediate version. No production path drifted through the close rehash.

## 2. Producer connection and false-readiness review

The whole new authoring_lifecycle.rs was read (1–227). It owns only fixed Copy fields: u64, Uuid, bool, enum status and Option<BattleVerdict>. There are no Strings, Arc definition owners, map/history growth, log readers, clock forecasts or RNG calls. Every owner/enum is included in an exhaustive CloneShape. Publicly named fields are inside crate-private types; CoreApplication exposes only crate-private fresh construction and a copied accessor (application.rs:787–807), so this does not create an external mutable witness setter or raw core borrow.

GameCore::new initializes None; new_for_authoring enables an empty Default witness (world.rs:102–120). Empty means current=None and is_ready=false; it is not default success. Every hook/reconcile immediately returns for None. The ordinary path gains a fixed field/branch and clone-shape traversal cost, but no new lifecycle rejection or gameplay/RNG mutation. Overhead remains unmeasured here.

The actual producer order is correct for the inspected call path:

1. commit_planned_live_battle_start installs the real battle UUID/active owner, calls authoring_battle_entered, performs the actual 0-ms advance, then calls authoring_time_zero_completed only after that Result succeeds (combat.rs:683–691).
2. entered checks the actual installed UUID and execution time=0, checked-increments entries, and constructs EntryPending with time_zero_completed=false (lifecycle.rs:85–110). Failed preparation does not publish that candidate.
3. advance_staged reads the real pre/post BattleExecutionState times around kernel stepping/finalization (combat.rs:1483–1508); after successful result/VN preparation it calls authoring_battle_advanced (1568–1573). checked_sub, checked_add, previous-time continuity and allowed entry status are checked before updating the witness (lifecycle.rs:128–155).
4. transition_finished_live_battle_to_result calls authoring_battle_finished **before** active_battle=None (combat.rs:1396–1410). The hook requires the real active UUID, execution.is_finished(), matching observed time and a pending/running entry (lifecycle.rs:158–185); it stores the actual CombatBattleState verdict.
5. An immediate terminal may become Finished while time_zero_completed=false; the enclosing successful zero-time call then sets that flag. is_ready admits completed Running or Finished, so it does not need an active battle to survive terminal removal (58–65,113–125).

The bool is justified by the actual successful producer call, not game-state inference. Hook guards are not an adversarial public input surface: their private current callers supply real execution values. For example, entered does not independently detect every imaginable duplicate manual hook call, and time_zero_completed does not independently resimulate time=0; the inspected sole production sequence provides those facts. I did not promote hypothetical calls through private internals to production defects.

## 3. Reconciliation, exits and complete transaction ownership

execute_application calls reconcile only after its operation returns Ok, before final context validation/polls and construction of PreparedApplicationCommit (world/application.rs:184–202). Failed operations discard the local draft. Reconcile can only retain, reject or mark an existing witness Exited:

- No current witness plus no active battle remains not-ready; an unexplained active battle is an error.
- EntryPending at publication is rejected.
- Running is retained only with matching active UUID and completed time zero; missing active becomes Exited, a different UUID is rejected.
- Finished requires no active battle and completed time zero. If the actual root is no longer that battle's CombatResult, it becomes Exited.
- Exited remains exited; unexplained active state is rejected.

This use of game state is **invalidation of actual evidence**, not creation of readiness. Reconcile never sets time_zero_completed, increments entries, or fabricates terminal_verdict (lifecycle.rs:188–226). It also preserves terminal_verdict when marking Exited. A new real entry replaces current and receives a checked new ordinal; gameplay UUID reuse alone cannot restore an exited entry.

All standard public behavior mutations route through execute_application (world.rs:252–274). Consequently accepted retreat, result consumption/retry, run failure, StartNewGame, AbandonRun and checkpoint load are invalidated at the common successful boundary rather than requiring many duplicated exit hooks. Explicit exit reasons from design021 were recommendations, not a frozen requirement; the more compact Exited status cannot claim why a root changed, but it is sufficient to invalidate readiness in this slice. The original public command remains in the mandatory receipt for context.

The state-replacement cases I reread are covered by that boundary: StartNewGame clears active at world.rs:745; AbandonRun replaces state at 781; checkpoint restore replaces gameplay fields then goes to ViewingMap (node_flow.rs:1283–1289). The GameCore-owned cumulative elapsed/ordinal are not part of those state replacements. They remain monotonic across accepted gameplay rewind; a failed outer candidate restores the old owner.

PreparedApplicationCommit now carries the sidecar in its field, construction, destructuring and live assignment (world/application.rs:128–135,194–222). Complete GameCore clone and CloneShape carry it (world.rs:51–64), so CoreApplication's private controlled fork preserves it. with_state_transaction separately captures/restores the Copy sidecar beside its nonbattle state snapshot (node_flow.rs:788–804), preserving the actual inner rollback owner. The cfg(test) ActiveRunTransactionRollback likewise captures/restores it beside state/seed/token (active_run.rs:20–26,172–218).

This conclusion is scoped to fresh TrialState through actual public CoreApplication calls. It does not certify arbitrary fixture/imported roots or private callers that mutate GameCoreState outside the application boundary. No public imported-core support was added.

## 4. Actual clock cap, phase, receipt and cancellation ordering

TrialState installs a fresh provenance scope before new_for_authoring (trial.rs:103–114). Its per-step application is a controlled fork of A (214–217). After the real step/result conversion, it requires an actual lifecycle accessor value and compares cumulative elapsed against the checked scenario maximum (293–306). Missing owner and cap failures return request errors with the already-selected TrialPreparation/TrialExecution phase. Cap admission is before prefix/record allocation and final receipt encoding.

The copied lifecycle is serialized by reference in the mandatory StepPayload (333–364). No optional trace switch influences the fixed witness or its receipt. The copied value matches the candidate application at that point; subsequent code only prepares receipt/owners, then attaches returned provenance/usage and swaps the whole TrialState after both result layers succeed (192–205,365–380). Mandatory receipt failure or final control cancellation drops candidate app/witness as part of that existing owner; there is no shared mutable witness in A.

Actual elapsed uses the real execution clock, including speed scaling and next-bucket jumps; it does not use raw input delta. A None tick returns before the advance hook, leaving elapsed unchanged. checked arithmetic rejects backward/overflow observations, while the session cap rejects an otherwise valid actual jump before authoring publication. This is atomic accepted-state admission; it does not retroactively avoid work spent by the private candidate. Existing fuel remains responsible for internal execution bounds.

This cap applies after every step, including command-triggered time-zero, not just Clock syntax. max_clock_delta_ms remains a raw-request grammar limit; the implementation adds the distinct cumulative actual limit. It does not promise a new actual-per-call bound. The last receipt position is preserved; no new lifecycle work occurs after the mandatory receipt stamp. The private initialize receipt still describes fresh initialization, not prepared battle readiness.

## 5. What the new tests do and do not establish

The first new world test (12670–12756) meaningfully exercises actual public behavior start, inner rollback after an accepted nested entry, pause/None tick, X2 host time, next-bucket actual time, whole clone, retreat invalidation and UUID reuse with a new ordinal. The second test covers terminal retention/removal, immediate terminal intent, and Finished→Exited on successful result consumption. These are source-level oracle observations, not this helper's executions.

Before relying on this slice's failure claims, focused validation still needs actual TrialState coverage of: cumulative cap equality and cap+1 with preserved A witness/receipt/quota/provenance; cap error phase in preparation and execution; time-zero failure/cancellation and mandatory receipt rejection after witness mutation; final-poll cancellation; malformed/internal arithmetic-continuity failures without partial witness publication; and checkpoint/abandon/new-run invalidation plus cfg(test) rollback restoration. This is a list of uncovered oracles in the **two inspected new world tests**, not a statement that no other tests exist or that whole public Session is a current defect.

Full optional source/occurrence/terminal capsule retention, public prepare/simulate and complete acceptance remain separate work. A fixed lifecycle verdict/time cannot recover source lost before active-battle removal. No completion verdict or broad safety/performance guarantee follows from this review.

## 6. Read provenance

Source discovery used bounded rg searches. Two exploratory nonexistent filenames (world/tests.rs and battle/events.rs) were not treated as evidence. Inspected content came from Python Path.read_bytes with numbered UTF-8 ranges; SHA-256 covers the same complete bytes. Only listed spans were read, except authoring_lifecycle.rs which was read in full. 15 read snapshots across 11 files. Close rehash: 2026-09-10T11:31:19.666127+00:00. Only the new test file drifted, as described above. No source/docs/Git mutation, build/test/runtime execution, or old-report overwrite occurred.

1. `core_enhanced/src/game/world/authoring_lifecycle.rs`: 1–227; SHA-256 `69e06f862a961fe0ff1d9b1b35fbdb869d1db4e0cb4db69e7f71efdda552aa8c`.
2. `core_enhanced/src/game/world.rs`: 15–125; 248–278, 728–784; SHA-256 `57d47fb5fdb7a728d9b7e1ba08b6efb0a7b56de0c28380f59b413999a662bd7f`.
3. `core_enhanced/src/application.rs`: 780–824; SHA-256 `c37e28bfccbed675dc268a59b477ddc3041759d02ba9e6e94019c8a9506a84e8`.
4. `core_enhanced/src/game/world/application.rs`: 128–229; SHA-256 `5b1a67a4550f31f03e7baeb1db45d8293147978e90d0b4b53df99f934481b2f0`.
5. `core_enhanced/src/game/world/node_flow.rs`: 784–810; 1252–1295; SHA-256 `949e4d04f2b8d64d6e591c0a9fea1ee23da0d43d452a97f6b5344e1b97d0bdcc`.
6. `core_enhanced/src/game/world/active_run.rs`: 18–30, 169–225; SHA-256 `1f37b9d6e08f767e9bc439571da4b9ae0628176f5e6cb40f3356f18a01f88d12`.
7. `core_enhanced/src/game/world/combat.rs`: 633–695, 1393–1413, 1454–1580; SHA-256 `b78e0de0ab027b3f99ffbd8c078df6ea8fff5fd1559992ed600a4a3579ce0062`.
8. `core_enhanced/src/authoring/trial.rs`: 95–173, 173–222, 284–389; SHA-256 `a636a0269b568beed306542d77d19ac5d23ab3bf2578ba1731e494e8a40db4d5`.
9. `core_enhanced/src/game/world/tests/combat.rs`: 12660–12820; 12758–12823; SHA-256 `4f39dc572e7975ff7a4aa93b92e563b11e164efa894cfab3508adaa68d03d5d9` → `a1e30f9fd0997b0f21e28636aae8af8af4249f799308662001c552b8f396204e`.
10. `core_enhanced/src/game/battle/scenario.rs`: 664–704; SHA-256 `376db1336ecfc11788f7f468f88310ee37896c219dcb5679fbfdfa6f681eca4d`.
11. `core_enhanced/src/game/battle/core/sim.rs`: 1230–1278, 1945–2005; 1518–1572; SHA-256 `9e3a3ef70179d302be10b8d6f0ca7873885cd448ac629e893505f6856b35f5b4`.
