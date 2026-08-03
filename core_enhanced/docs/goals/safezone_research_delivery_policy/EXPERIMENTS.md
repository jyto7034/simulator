# Safe Zone Research Delivery Policy Experiments

Goal: G21

Updated: 2026-07-14

## Entry gate

- G29 completion review: Complete / High; global compendium state is independent from run-local
  research and must not be touched by delivery changes.
- G15 completion review: Complete / High; consuming combat result and retreat/failure boundaries are
  authoritative and must be composed rather than reinterpreted.
- R03: Complete / High; Event and global VN are distinct, and nested Event-started combat resolves
  through its owning Event node before planning return.

## Initial task

- Record the three dirty worktree baselines and target overlap before editing.
- Trace pending research creation, storage, delivery executor, result projection, and every current
  caller from runtime code before choosing the shared return seam.
- Enumerate all state transitions that actually return resolved content to map/planning separately
  from retries, retreat-to-NodeConfirm, RunFailed, RunComplete, and read-only reconnect/snapshot.

## 2026-07-14 — Dirty baseline

- Core: branch `main`, HEAD `c2c5dc5ec54bd26530e46f0d7ee7909de3ed7a1f`; status fingerprint
  `ef2476878933565e5f3d5d181e12eed3331fca72ebceec0d3c46da40a0dbf435`, tracked diff
  fingerprint `3481628ebf1ef48b71084b69c4d0d788e55f19c071592bff130c9c4b7a87f84a`.
- game_server workspace view: branch `main`, same HEAD; status fingerprint
  `d2d559392970e8fcd3bfd45ca6fe1fa2065bcbfbe1d7dc8966b401113506aec9`, same repository tracked
  diff fingerprint.
- Unity ArkClient: branch `master`, HEAD `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`; status fingerprint
  `ee407e1ee2c30f6ec2229e660408e0a9acb5a72dbf44747b21ac1d7dc648048e`, tracked diff
  fingerprint `ad13308d27abb34c71be371664f5d67440938bf786765219018dc84d88739d6a`.
- All worktrees remain heavily dirty. Likely G21 Core seams and tests are already modified by closed
  G15/G23/G03 work; classify each overlapping hunk before editing. Do not stash, reset, restore,
  clean, baseline-commit, or modify unrelated Unity/server files.

## 2026-07-14 — Runtime trace

- `ResearchDeliveryPolicy` held a Support/HeadquartersContact/Shop/Reward category allowlist, and
  `deliver_research_if_safe_node` mutated inventory during node entry.
- Entry/state DTOs carried `research_deliveries`, while `NodeOutcomeSummary` carried a permanently
  empty delivery vector. Neither represented the accepted planning-return authority.
- Ordinary interactive completion converges in `commit_interactive_node_completion`; combat result
  completion composes `CombatResultNodeCompletion`; direct Gate confirm owns its separate floor
  transition. Event-started combat remains inside its owning Event session until that final
  completion seam.
- Retry returns to `NodeConfirm`, retreat restores the battle-entry snapshot, cancel/checkpoint load
  are non-consuming, and RunFailed/RunComplete are terminal rather than planning-return results.

## 2026-07-14 — Core implementation

- Removed the category delivery policy, its `GameCoreState` field, and the entry hook/callers.
  `NodeEntered`, Support/Maintenance/HQ/Shop/Event/Reward state results, and node outcome no longer
  expose delivery facts.
- Made `SkillFragmentInventory::deliver_pending_research_with_policy` transactional by applying all
  pending entries to a clone and replacing the live value only after every identity/count succeeds.
- Added one staged planning-return helper. Interactive `NodeCompleted`/`FloorAdvanced`, direct Gate,
  and consuming combat result paths use it; SavePoint stores the already-delivered inventory in its
  checkpoint.
- Combat planning distinguishes consuming planning return from retry, retreat, fatal/final failure,
  no-recovery RunFailed, RunComplete, and Endless completion. Event-started combat delivers only
  through final Event node resolution.
- `BehaviorResult` and command-result contracts require `research_deliveries` only on
  `NodeCompleted` and `FloorAdvanced`; typed nested completions preserve that exact payload.
- Added a live-catalog Rust fixture generator for the public completion payload.

## 2026-07-14 — Server and Unity implementation

- game_server remains a generic typed pass-through. A focused actor test proves a delivery-bearing
  `NodeCompleted` command result requests the normal trailing snapshot, preserving result-before-
  snapshot ordering and every exact delivery field.
- Unity added one strict `ResearchDeliveryNotificationModel` that accepts only direct or typed
  nested planning completions. It rejects missing/unknown/duplicate/unsorted identities and invalid
  counts; non-planning results produce no synthetic delivery.
- `ArkClientBootstrap` consumes only a successful command result. A small notification latch keeps
  the one-shot fact visible through the immediately following root snapshot and clears it on the
  next unrelated store update; screen entry/reconnect never sends or infers a grant.
- Core, server, Unity core, and Unity non-combat probe documents now describe the same completion-
  only authority and exact `{ fragment_id, count, total_count }` fact.

## 2026-07-14 — Focused validation and corrections

- Core `cargo check --all-targets` and formatting passed. Focused research tests passed 18/18.
  Exact Support, SavePoint, Gate, standard RunComplete, retry/retreat/exhausted defeat, final-boss
  RunFailed, Event-started combat, recovery-node RunFailed, atomic failure, and DTO placement tests
  passed.
- Gate delivery staging was moved after Gate/progression/floor planning validation and immediately
  before the first commit operation. The corrected exact Gate test passed 1/1.
- game_server check/formatting and the exact ordering/pass-through test passed 1/1.
- Unity script validation found zero errors in all G21 files. `ArkClientBootstrap` retained one
  generic pre-existing `Update()` string-allocation warning. The five focused contract/latch tests
  passed 5/5.
- The first Unity refresh compiled the consumer before importing the new reader asset and briefly
  reported the missing type; explicit asset import/domain refresh cleared it. Final console query
  returned zero errors.
- Parallel Core/server Cargo checks contended on their shared package/target locks; serialized reruns
  completed successfully. A mistakenly guessed Gate filter selected zero tests and was not counted;
  the exact discovered test name was rerun and passed.

## 2026-07-14 — Broad validation and hygiene

- Core `cargo test --all-targets`: 746 passed (704 library + 3 + 3 + 19 + 13 + 4), 0 failed,
  0 ignored. Core `cargo check --all-targets`, `cargo fmt --all -- --check`, and repository
  `git diff --check` passed.
- game_server `cargo test --all-targets`: 30 passed, 0 failed, 0 ignored. Its all-target check and
  formatting passed.
- Unity EditMode: 211/211 passed. Unity PlayMode: 52 passed, 0 failed, with four explicit live-server
  tests skipped because those fixtures require an independently running server. G21 introduced no
  ignored/skipped test.
- The Rust generator output matched the tracked Unity fixture byte-for-byte through `diff -`.
- Full Unity `git diff --check` still reports pre-existing unrelated CRLF/whitespace in `.gitignore`,
  Rider metadata, and `GameUiSelectionState.cs`; scoped G21 Unity diff check passed.
- Production/current-contract searches found no category allowlist, entry delivery caller, entry/
  state delivery field, Unity screen-entry authority, server JSON repair, or relevant ignored test.
  Remaining old entry-policy wording is confined to the explicitly historical initial-audit goal
  `core_component_refactor_master/reward_shop_support_refactor.md`; current canonical/probe docs are
  corrected.
- Dirty worktrees remained intact. No stash, reset, restore, clean, destructive command, baseline
  commit, or live RON edit was performed.
