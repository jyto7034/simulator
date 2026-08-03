# Battle Projection Revision Contract Experiment Notes

Status: complete

Implementation complete: true

Created: 2026-07-12

## Binding Structural Decisions

- `battle_state_revision` is a Core-owned authoritative commit revision scoped by battle UUID.
- Event sequence remains immutable event-log continuity; battle time remains simulation/presentation time.
- Server connection/session owns the sent-event cursor and ordered delivery.
- Unity applies ordinary state by battle UUID and revision, then separately validates event continuity.
- Resync reapplication is explicitly allowed at the same current revision; ordinary duplicate application is not.
- No compatibility field or revisionless fallback is authorized.

## Questions To Answer From Current Code

- What is the smallest complete transaction boundary that observes every client-authoritative projection mutation exactly once?
- Which mutations currently bypass world-level handlers or finalize nested VN/trigger changes after update construction?
- Where are event cursors stored in Core, server actor, and Unity today?
- Can command response and tick push ordering be proven by one actor mailbox/outbox, or do asynchronous sends require an explicit queue?
- Which setup/update/resync/command-result DTOs are concrete Rust types versus ad-hoc JSON?
- Which Unity stores compare event sequence/time before reading checkpoint or VN state?
- Does any save/checkpoint serialize active battle connection cursor state and therefore require removal/migration?
- What actual Rust JSON path can be consumed by Unity tests without hand-maintained mocks?

## Cross-Goal Boundaries

- G06 owns generic context capability and payload eligibility semantics.
- G07 owns final battle capability/reconnect authority integration after G01 and G06.
- G01 must expose a revisioned projection seam that those goals can extend, but must not invent their final action policy.
- G08 consumes the later recovery UX/command surface.
- G12 owns strict generated-data validation and does not overlap G01 unless current battle DTO construction reads authored schema.

## Known Dirty Worktree Risk

Core battle/world DTOs, server handlers/state/messages, Unity battle runtime/store, and canonical documents already contain user-owned changes from the Visual Novel work and audits. Inspect exact current diffs before every edit and preserve unrelated modifications.

## Follow-Up Candidates

None yet. Record only after current source tracing.

## Current Transaction Table

| Operation | Current mutation | Current projection/cursor | Current ordering defect |
| --- | --- | --- | --- |
| Initial battle start | Creates active battle and first simulation step | Core drains from cursor 0; optional setup then update | No revision; triggers finalize after update construction |
| Server tick | Advances simulation, deployment signals, possibly finishes battle | Core drains and advances Core-owned cursor | Tick push originates from player actor and may race session-sent command side messages |
| Pause / resume | Assigns boolean unconditionally | Core drains immediately | Same-state no-op indistinguishable; equal seq/time can be dropped |
| Set speed | Assigns speed and clears remainder unconditionally | Core drains immediately | Selecting current speed consumes a conceptual update; equal seq/time can be dropped |
| Deploy / withdraw / activate skill | Mutates battle and deployment state | Core drains immediately | No revision ties command acknowledgement to projection |
| Battle VN advance/choice | Mutates global VN state | Pure query from latest event seq, then attaches VN | Equal seq/time update is dropped by Unity |
| Request battle state | Calls reconciliation then pure query from supplied seq | No Core cursor advance | Query is not fully read-only; resync has no distinct revision reapply rule |
| Root snapshot | Reads gameplay state | Includes Core `last_pushed_event_log_seq` | Connection delivery state leaks into authoritative snapshot |
| Re-entry / new battle | Reuses `node_id.0` as battle UUID | Event seq restarts from 1 under same UUID | Unity cannot distinguish a new battle authority sequence |
| Session attach | Executes `RecoverBattleSetupLoss` and discards active battle | Returns root NodeConfirm snapshot | Violates reconnect preservation and current-revision resync |

## Provisional Long-Term Seam

1. `ActiveBattleSession` owns only current `battle_state_revision`, initialized to 1, plus gameplay state and immutable event log.
2. Successful authoritative mutation paths finalize triggers, VN, deployment reconciliation, and capability projection, then call one checked revision commit exactly once before building the DTO.
3. Read-only update projection accepts explicit `after_seq` and never reconciles, advances a cursor, increments a revision, or otherwise mutates Core.
4. game_server `PlayerGameActor` owns the connection's `(battle_uuid, last_sent_event_seq)` and is the single sender for successful command responses, side messages, tick pushes, and terminal snapshots.
5. The command acknowledgement copies the resulting update's `battle_state_revision`.
6. Unity requires a positive revision, applies ordinary higher revisions regardless of equal time/seq, ignores equal/lower ordinary revisions, and uses a distinct authoritative-resync application mode that may reapply the current revision.

This is a technical implementation interpretation of accepted PD-01. Revise it if current-source evidence shows an invariant gap; do not replace it with a timestamp/event workaround.

## Identity And Atomicity Refinement

- Add a distinct internal `node_id` to `ActiveBattleSession`; battle UUID can no longer double as map-node identity.
- Allocate a fresh deterministic battle-session UUID for each newly committed battle entry and restart its revision sequence at 1.
- Preserve the existing node-keyed legacy result/record identity only where later G16 still owns its removal; do not rename that legacy field into a false battle identity.
- Move deploy `next_instance_salt` consumption after successful BattleCore deployment.
- Resolve withdraw unit identity without removing it from deployment state, execute the BattleCore command, then commit the deployment removal and redeploy lock.
- A failed operation must leave both gameplay state and revision unchanged.

## Reconnect Refinement

Current server attach behavior discards the active battle. G01 must replace it with an attach result that can send the root snapshot plus immutable setup and a read-only current-revision update. Attach/reconnect itself consumes no revision. Tick scheduling begins only after this ordered recovery batch is emitted.

G07 will later combine this revisioned recovery substrate with exact battle capability authority and blocking-VN command gating; G01 must not retain the destructive recovery path as a compatibility fallback.

## Canonical Conflicts To Correct After Runtime

- External battle transport and Unity core canonical sections currently require active-battle discard and NodeConfirm recovery on setup loss.
- Root Unity canonical exposes `last_pushed_event_log_seq` in `game_state_context`.
- These statements are superseded by accepted PD-01 and must be replaced only after runtime/server/Unity tests prove the new contract.

## Required Actual-Rust Fixture Work

The current server has exact Rust serialization tests, but Unity consumes only hand-built revisionless `JObject` fixtures. G01 must create a generated or checked-in Rust-produced JSON fixture and a Unity test that reads that exact artifact; matching two manually maintained shapes is insufficient.

## Resumed Cutover Judgment

- The present compile break is inside the approved atomic DTO cutover and is not a reason to add a legacy overload, `Option<u64>` adapter, or replacement Core delivery cursor.
- Introduce one internal helper only if it represents the accepted transaction seam: finalize derived projection state, checked-commit exactly once when mutation occurred, and build a pure update for the caller-provided event cursor.
- A same-value Pause, Resume, or speed command is a successful non-mutating result under PD-01: it may return the current projection but cannot reset playback remainder or consume a revision.
- `RequestBattleState` and reconnect/resync projection must not run deployment reconciliation. Any required reconciliation belongs to the preceding authoritative mutation transaction.
- Initial battle activation begins at revision zero internally and the first fully finalized projection transaction commits it to revision one; revision zero must never cross the wire.

## Implemented Core Commit Seam

- Simulation transactions preflight revision availability, step/finalize BattleCore, evaluate battle-start/time triggers, then refresh/reconcile deployment, commit once, and build the update.
- Playback commands commit only when their value actually changes; selecting the current value returns the current projection without resetting accumulated tick remainder.
- Battle VN commands preflight overflow before mutating VN, then commit the complete projection only after the VN transition succeeds.
- `RequestBattleState` accepts an explicit event cursor and borrows the active battle immutably; it does not reconcile or commit.
- Deploy salt and withdrawal map membership now change only after BattleCore accepts the command. This closes the G01-observed partial-failure leak while remaining within the accepted transaction invariant.
- The current Core package compiles, but compilation cannot prove the one-revision or atomic-failure behaviors; focused tests are the next evidence gate.

Focused-test note: the first suite attempt reached a test-only `Uuid` dereference compile error and executed zero tests. It does not change the commit-seam judgment and must remain classified as failed evidence until the corrected nonzero rerun passes.

The corrected combat run confirmed why query reconciliation must stay removed: the sole failure is a test that injects an impossible half-transaction by directly changing BattleCore state, then asks projection to repair it. The durable invariant is that authoritative simulation/command finalization reconciles deployment before commit; projection only observes the committed state.

Core focused closure note: the corrected authoritative-finalization test and 44 other combat flows pass. This is sufficient to move to the server boundary, but not to mark the Core portion broadly complete until all-targets validation runs after DTO/server/Unity migration.

## Server Ordering Implementation Judgment

- The ordered boundary is the `PlayerGameActor` mailbox, not a second free-standing queue: it executes a command/tick, builds the complete result, emits the batch in order, updates its cursor as a battle side message is emitted, and only then accepts the next actor message.
- The session actor remains the physical WebSocket writer, but it no longer composes or re-emits successful player-command battle batches. Its attach callback sends the one returned bootstrap batch before scheduling ticks.
- Errors may still be returned through the requesting session because they carry no committed battle delta; successful authoritative state messages have one owner.
- Attach resets the connection cursor, projects from sequence zero, returns root snapshot then immutable setup/current update, records the delivered cursor, and starts ticking only after the session sends that batch.

Production compile confirms the structure is type-coherent, but it intentionally says nothing about test-only JSON constructors or actor ordering. The next gate must execute nonzero actor/message/state tests.

The first test build confirms the migration is bounded and mechanical at the shape layer. The reconnect test is the one semantic legacy expectation: it must prove preserved InBattle authority and ordered setup/current-update recovery rather than be mechanically updated to a new field name.

The 22/24 run proves the remaining failures are observation-boundary drift: command updates now appear in the same probe as tick updates because that probe represents the one socket/outbox. Tests must partition batches by order/cursor, not infer message ownership from absence.

Server focused closure note: the 24/24 pass proves one actor mailbox is an equivalent ordered outbox for current synchronous Core commands and scheduled ticks. The session remains only the physical WebSocket serializer and attach bootstrap sender; it does not compose successful player-command batches.

## Unity Application Seam

- Ordinary application validates the typed shape first, resets scope on battle UUID change, ignores revision `<= LastAppliedRevision`, then validates `after_seq == LastAppliedSeq` only for a genuinely newer projection.
- Resync validates the same required DTO, may apply `revision == LastAppliedRevision`, rejects revision rollback, and still requires the server delta to begin at the client's declared known event sequence.
- `server_battle_time_ms` is retained for clock anchoring and movement presentation but no longer participates in state freshness.
- Legacy nested setup inside `battle_resync` remains invalid; reconnect recovery uses the ordered standalone `battle_setup_snapshot` plus update batch. Rejection no longer clears a valid current battle.
- The Rust generator's read-only compare mode is the provenance gate for the Unity fixture; the Unity test consumes that file instead of reconstructing the shape in C#.

Unity focused closure note: the open Unity 6000.3.4f1 Editor refreshed and compiled with zero Console errors, the complete `ArkClientContractTests` EditMode class passed 182/182, and the affected `ArkClientMockRenderTests` PlayMode class passed 49/49 through Unity MCP. This supplies non-mock consumer evidence for the actual-Rust fixture and visible-flow evidence for updated mock/raw payloads. Broad cross-layer validation remains the next completion gate.

Broad Core closure note: all 682 nonzero game_core all-targets tests pass, including live RON loading and integration suites. This is one more test than the Phase 0 baseline and confirms no baseline test was silently removed during G01. Server broad validation and final audits remain open.

Broad server closure note: all 25 nonzero game_server all-targets tests pass, preserving the full Phase 0 count while adding revision/reconnect/order assertions inside that suite. Zero-test binary/example targets are not misreported as evidence; typed fixture exact comparison remains the generator proof.

Formatting audit note: the first rustfmt check failed only on mechanical formatting in G01-touched Rust files and must be corrected/rechecked. Whole-repository Unity diff checking is polluted by unrelated pre-existing CRLF `.gitignore`/`.idea` additions; G01 must use an exact affected-path diff check and must not normalize those user-owned files.

Formatting closure note: after standard rustfmt, the rustfmt check, Core/server diff checks, and exact G01-path Unity diff check all pass. The unrelated Unity CRLF files were preserved and remain explicitly outside this goal.

## Final Audit Correction Seam

- Root `InBattle` context cannot expose `Option<u64>` under the accepted required-revision contract. `GameState::InBattle` without `ActiveBattleSession` is an internal invariant violation, not a wire state that Unity should recover by inventing revision 1.
- `ArkMockSnapshots` must consume the required root revision exactly. Its current `?? 1` is a test/mock fallback and conflicts with the explicit no-revisionless-mock completion condition.
- This is a technical completion correction inside PD-01, not a new DTO policy: the field and name are already accepted and shipped by the cutover; only nullability/fallback remains to remove.
- Both external canonical battle documents still describe superseded destructive setup-loss/reconnect behavior and one still exposes the removed Core cursor. They must be synchronized only to the proven runtime/test contract; historical goal documents remain historical and are not rewritten.

The first Unity compile after applying this correction seam failed only because the validated revision variable was not passed to the separate mock update helper. The correct fix is explicit parameter flow, not reintroducing a fallback or reading the context twice inside the helper.

Root/mock strictness closure note: explicit parameter flow compiles with zero Unity errors and preserves 182/182 EditMode plus 49/49 PlayMode. The last revisionless root/mock seam is therefore removed without a compatibility field or inferred value.

## Final Cross-Layer Audit Result

- Core owns checked revision commits and immutable event logs; no connection cursor remains in gameplay/root state.
- game_server owns the per-connection cursor and one actor-mailbox emission order for successful command and tick batches.
- Rust root/update/resync/acknowledgement shapes carry required revision where applicable; actual typed fixture equality is proven.
- Unity ordinary application is UUID/revision-first, seq-continuity-second, and time-presentation-only. Same revision is accepted only through explicit resync.
- Root/mocks no longer fabricate a revision. An InBattle state without an active session is treated as an internal invariant violation rather than serialized as revision null.
- Canonical transport and Unity integration documents now match same-battle recovery and removed cursor ownership. Historical completed goal documents remain historical evidence and were intentionally not rewritten.
- Final nonzero evidence: Core 682, server 25, Unity EditMode 182, Unity PlayMode 49; all pass. Formatting, exact fixture comparison, and scoped diff checks also pass.
- No remaining G01 policy question or compatibility requirement was found. Completion still depends on the formal review verdict.

## Completion Review Disposition

- `COMPLETION_REVIEW.md`: Complete / High / improvement class none.
- DCA-C-01 and CPR-C-01 are closed by G01.
- G06 may consume the revision seam; it must not reopen revision meaning or move cursor/freshness authority.
- The bounded lack of a live WebSocket-to-Editor probe is documented for R01 cross-component review and is not an immediate correction because typed server ordering, exact Rust fixture equality, and actual Unity consumer tests all pass.
