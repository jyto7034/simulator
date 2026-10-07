# Phase 25 CRA-F-C02 Experiment Notes

Status: complete
Implementation Complete: true
Candidate: CRA-F-C02
Source Candidates: CRA-P03-C05, CRA-P04-C02
Master Phase: 25 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Locked Decisions

- This is a behavior-preserving private transaction/context refactor, not an HQ behavior correction.
- CRA-P03-C05 and CRA-P04-C02 are merged because they point to the same helper, callers and target authority.
- The existing common non-battle transaction must own rollback; no new parallel aggregate is authorized.
- Encounter-compendium knowledge remains a monotonic excluded profile aggregate on rollback.
- Active battle remains outside the non-battle transaction and its rejection cannot be weakened.
- Completion receives explicit resolved-HQ context; hidden active-content removal is not retained as compatibility.
- Recruit/emergency/open-shop results, roots, errors, ordering and Phase 24 Shop pairing remain exact.
- Public/session/serde/V3/RON/DTO/wire/save shapes remain exact.
- Phase 26 VN assembly/queue placement is prohibited.

## Open Evidence Slots

1. Activation status, HEAD and normalized dirty hash.
2. Prospective source/test/data/canonical hashes before content inspection.
3. Complete `GameCoreState`, common transaction capture/restore and compendium exclusion semantics.
4. HQ transaction helper's exact ten-root list and every branch caller.
5. Common completion planner admission, active-content bypass and resolved-context seam.
6. Full branch validation/staging/commit/result/error order.
7. Baseline nonzero test filters and counts.
8. Test-first complete-state success and late-failure matrix.
9. Monotonic compendium and active-battle exclusion tests.
10. Smallest delegate/context implementation and removed legacy evidence.
11. Public/V3/RON/DTO/server/Unity/canonical comparison.
12. Focused/broad debug/release/static evidence and independent verdict.

## Historical Risk Register

- A new non-battle root can escape the handwritten HQ list without a compile error.
- Replacing the wrapper incorrectly could roll back encounter-compendium knowledge that the common transaction keeps
  monotonic, or could accidentally include/exclude another root.
- Making HQ completion context broad could weaken admission for non-HQ sessions or active battle.
- Planning completion earlier/later could change pending research, checkpoint, map progression or error order.
- Recruit stages UUID/roster/queue changes; emergency stages currency/grants; Shop stages queue plus paired session and
  transition. A late failure must restore the entire appropriate state, not only the visible branch result.
- Phase 24 deliberately centralized Shop paired commits; Phase 25 must call that seam and must not reintroduce direct
  writes.
- Existing happy paths alone do not prove rollback parity; injection/fixture design must remain behavior-observing.

## Historical Evidence Summary

- P03 identified ten explicit HQ snapshot/restore assignments beside the common full-state transaction and required
  full-state rollback parity.
- P04 identified hidden `active_node_content.take()`/restore around common completion planning and missing late-
  failure coverage.
- Reaudit found both halves still current; BossOmen is nested in run state and does not close either risk.
- Final merge targets the common transaction plus explicit resolved-HQ completion context, preserving branch policy,
  active-battle rejection and compendium monotonic exclusion.
- Exact linear row 25 depends on Phase 23 grant-state parity and Phase 24 Shop pairing, both now `Complete / High`.
- Required safety net is full-state success and late failure for recruit, emergency, open-shop and common completion.

## Activation Boundary

No Phase 25 production source, test implementation, live data or canonical content was inspected during Child
initialization. After Master sole activation, reread this entire Child and Phase 24's audit, capture prospective
fingerprints, and only then begin production investigation.

## Dirty Worktree Note

The shared repository contains extensive user-owned modifications. Pre-initialization state is 499 rows at
`1f576cd8...`. Phase 25 must use path fingerprints and scoped diffs, preserve all unrelated changes and never use
reset/restore/clean/stash to manufacture a clean baseline.

## 2026-08-10 — N01 Activation Baseline

- Mandatory post-activation reads completed: Child 201/63/72 and predecessor audit 309/309.
- HEAD `f318414f...`; expanded dirty state 502 entries at normalized SHA-256 `a8552b75...`.
- Exactly three new Phase 25 Goal paths explain the 499→502 increase; production/test/data/canonical status did not
  change during initialization or activation.
- Prospective hashes are recorded in E02. The initially guessed `world/mod.rs` path does not exist; `world.rs` is the
  module root and was fingerprinted without reading content.
- Disposition: activation gate complete; current source/caller/test investigation may begin, while production
  mutation still waits for test-first full-state characterization.

## 2026-08-10 — N02 Discovered Contract And Test Boundary

- The common transaction is broader than the historical HQ list by construction: it captures every present
  non-battle root and therefore automatically covers VN/trigger, starter/pending-mode and run-power additions. Profile
  knowledge is the only deliberate rollback exclusion; active battle prevents transaction entry entirely.
- Full rollback fingerprinting uses the current state-root inventory rather than the deleted helper's list. The run
  is serialized through `RunCheckpointPayload::capture` plus the non-checkpoint combat-recovery restriction; remaining
  roots are represented directly or by an equivalent deterministic projection.
- Recruit/emergency invalid pending research reaches a failure after branch staging but before completion commit, so
  it proves no partial queue, UUID/roster, Enkephalin, map, research or lifecycle state escapes. The open-Shop outer
  injection proves complete restoration after the real Shop handoff and separately proves the compendium union.
- Success-path full-root comparisons permit only each branch's named mutation roots and assert every other root exact.
  Existing branch-specific checks then prove the allowed mutations' semantic result.

## 2026-08-10 — N03 Implementation And Compatibility Decision

- A typed context enum is preferable to a boolean: the call site names the resolved-HQ fact, and the planner rejects
  that fact unless current active content is truly HQ. No node identity comparison was added because historical HQ
  completion did not impose one; adding it here would be a behavior change rather than a refactor.
- The real recruit/emergency `active_node_content = None` writes remain lifecycle commit operations. Only the temporary
  validation bypass was removed.
- Open-Shop does not complete the node at handoff, but it now receives the same full transaction protection as the
  other branches while retaining the Phase 24 persisted/active paired-commit authority.
- Canonical inspection requires no edit: the private transaction and completion-context representation is absent from
  all public, server and Unity contracts, whose currently documented behavior is unchanged.
- Phase 26 symbols/ownership remain absent; broad validation and the independent audit are the only remaining gates.

## 2026-08-10 — N04 Broad-Gate Disposition

- Complete Core and server tests/checks are green in both profiles. Exact counts are recorded in E05; no zero-case or
  ignored-case evidence is used.
- Static proof closes both historical root causes: one common full-state rollback authority serves all HQ branches,
  and explicit typed admission replaces the hidden content mutation. The narrower admission cannot authorize a
  non-HQ caller or weaken public direct-completion rejection.
- Status cardinality/hash stayed identical to activation despite implementation because all affected production/test
  paths were already dirty. Final content hashes, scoped source reads and negative searches distinguish this Phase's
  verified state without claiming ownership of unrelated edits.
- Non-strict Clippy debt is byte/line-adjacent pre-existing repository debt and unchanged in aggregate. It is recorded
  diagnostically, not used to weaken the green required gates.
- No implementation gap is known. Independent review must still reread the full 874-line authoring guide, 567-line
  completion guide, current Child, exact implementation/tests/consumers and current command evidence before choosing
  confidence or permitting Phase 26.

## 2026-08-10 — N05 Audit Correction Disposition

- The first review pass exposed an evidence-granularity gap, not a runtime defect: top-level full-state assertions
  could hide unintended changes inside the allowed `run_payload` root. Treating that as sufficient would not meet the
  Child's stricter complete-state condition.
- Nested payload comparison now makes the success contract exhaustive at both state-root and serialized run-field
  levels while branch-specific assertions verify every permitted change. No production special case or helper-shape
  assertion was introduced.
- The accidental zero-case Shop namespace command is not counted anywhere. Its corrected 7-case run is the only Shop
  audit evidence. This distinction preserves the guide's nonzero-run rule.
- Since the correction changes only test helpers/assertions, server binary inputs, live data, public types, save/wire
  and canonical content remain byte-identical. Complete Core suites/checks were nevertheless rerun in both profiles.
- No material finding remains after revalidation. Final verdict still requires a fresh current-document/source pass
  and durable `COMPLETION_AUDIT.md`.

## 2026-08-10 — N06 Final Disposition

- Final independent verdict is `Complete / High`; the resolved assertion-granularity finding leaves no follow-up.
- The common non-battle transaction is the sole HQ rollback authority and the typed planner context is the sole
  resolved-HQ admission representation. All external behavior and schemas remain unchanged.
- Phase 26 may be initialized next but is not active yet. Its VN assembly/queue-placement authority must be
  rediscovered from current evidence rather than inferred from Phase 25's private context enum.
