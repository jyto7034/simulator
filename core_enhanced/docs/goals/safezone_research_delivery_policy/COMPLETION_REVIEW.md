# Safe Zone Research Delivery Policy Completion Review

Goal: G21

Review date: 2026-07-14

Guide: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Item Audit

| Item | Integrated result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| PD-13 queue boundary | research completion queues during all node activity and entry | Complete | High | None |
| Planning-return authority | one Core staged helper consumed by interactive, combat, and Gate completion seams | Complete | High | None |
| Atomicity/idempotence | cloned all-pending grant, commit-after-validation, once-only pending removal | Complete | High | None |
| Non-return exclusions | cancel, load, reconnect, retry, retreat, RunComplete, and RunFailed do not grant | Complete | High | None |
| Nested flow composition | Event combat waits for owning Event; nested result wrappers retain completion facts | Complete | High | None |
| External notification | exact command-result fact, server ordering, strict Unity reader and one-snapshot latch | Complete | High | None |

## Expected Source Of Truth

`SkillFragmentInventory.pending_research_deliveries` is the durable queue. Core alone turns it into
owned fragment counts. The qualifying event is a successfully committed transition from resolved
node content to `ViewingMap` planning, expressed only as `NodeCompleted` or `FloorAdvanced`.
Node categories, entry/state DTOs, snapshots, server transport, and Unity screens are consumers or
presentation context and hold no grant authority.

## Runtime Evidence

- `skill_fragment.rs` validates and applies every pending entry to a cloned inventory before
  replacing the live value, so a late invalid identity/stacking failure cannot partially grant or
  clear the queue.
- `node_flow.rs` stages that inventory at the interactive completion and Gate seams. It commits the
  staged value only after the completion remains a planning return; recovery resolution that becomes
  RunFailed returns without committing it. SavePoint persists the post-delivery value.
- `combat.rs` stages delivery in the same combat-result state only when the final visible completion
  remains a planning return. Retry, retreat, fatal/final failure, no-recovery failure, standard /
  Endless RunComplete, and Event battle entry are excluded.
- The previous category allowlist and entry hook are removed. No Support/HQ/Shop/Reward/Maintenance /
  Event state result has a delivery field, and node outcome no longer carries a meaningless empty
  duplicate.

## External Contract Evidence

- Core command contracts require the delivery vector on only `NodeCompleted` and `FloorAdvanced`.
  `CombatRewardsGranted`, `EmployeeRecruited`, and `EmergencySuppliesGranted` retain their typed
  recursive completion rather than flattening or rebuilding it.
- game_server serializes the complete Core payload and sends the successful result before its normal
  trailing snapshot. It does not interpret pending inventory or Safe Zone state.
- Unity strictly validates exact fields and canonical ordering from the command result. Invalid
  planning completion shapes fail closed. The notification survives the trailing snapshot once and
  then clears; reconnect or repeated screen rendering cannot grant or recreate it.
- The generated Rust fixture and tracked Unity fixture are exactly equal and include `total_count`.

## Validation Review

- Core focused research/transition/failure/DTO matrix passed, including exact atomic failure,
  SavePoint, Gate, Event combat, retry/retreat/exhaustion, terminal failure, recovery failure, and
  standard completion boundaries.
- Broad Core: 746 passed, 0 failed, 0 ignored. Check, formatting, and diff hygiene passed.
- Broad game_server: 30 passed, 0 failed, 0 ignored. Exact result-before-snapshot preservation test,
  check, and formatting passed.
- Unity focused: 5/5; EditMode: 211/211; PlayMode: 52 passed, 0 failed, four explicit environment-
  dependent live-server skips. Script validation and final console error query were clean.
- A zero-test guessed Rust filter and transient Unity asset-import ordering error were diagnosed,
  corrected, and excluded from evidence. The exact reruns passed.

## Legacy And Fallback Audit

- Current production and canonical contracts contain no category delivery policy, node-entry caller,
  entry/state delivery field, Unity screen-enter grant, reconnect grant, server repair mapping, or
  relevant ignored test.
- Historical initial-audit prose still records the removed behavior as evidence of the former
  baseline; it is not a current contract. The live Unity probe and all canonical documents were
  updated.
- Full Unity diff hygiene remains noisy only from unrelated pre-existing CRLF/whitespace; every G21
  Unity file passes scoped `git diff --check`.

## Cross-Component Findings

- G21 composes G15's consuming combat-result transaction, Phase 3's Event ownership, G14's Gate /
  RunFailed boundaries, G23's fragment/reward state, and G29's separate global profile without
  changing any of them.
- Terminal completion deliberately differs from planning return. Pending research surviving
  RunComplete/RunFailed is explicit and tested, preventing a hidden synthetic Safe Zone.
- Category expansion now requires no delivery-policy edit: any future node that resolves through the
  canonical completion seam inherits the same behavior automatically.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: one durable queue, one staged Core operation, two exact public completion types, generic
  transport, and a strict presentation-only reader remove the old cross-layer category coupling.

## Immediate Correction Candidates

None. The review moved Gate staging to the clean pre-commit point and added an explicit Unity latch
test before the final broad validation.

## Follow-Up Refactor Candidates

None inside G21.

## Policy Questions

None.

## Review Conclusion

G21 is Complete / High with no immediate correction. R04 may begin.
