# Unity Wire Identity Alignment Experiments

Goal: G02

Updated: 2026-07-14

## 2026-07-14 — Goal initialized

Question:

Which reported identities still mismatch on the current Core/server/Unity worktrees, and which are
already obsolete after intervening changes?

Evidence available at initialization:

- R02 closed Phase 2 as Complete / High.
- The Master assigns seven primary findings and the shared PD-04 Event envelope seam to G02.
- All repositories are heavily dirty; existing unrelated changes remain user-owned.

Claim status:

No G02 implementation success is claimed. Perform current-code reproduction and classify every
finding before editing.

## 2026-07-14 — Exact Unity red reproduction

Question:

Do the reported item, HQ/reward, and active-consumable identities still fail against current Rust
shapes?

Evidence:

- Added four focused Unity tests using only `target_employee_uuid`, candidate `id`, reward `uuid`,
  and active modifier `name` while rejecting old snapshot aliases.
- Red job `b5ad8169fdcf409eafd44a3a73d85984`: 0 passed, 4 failed for the intended reasons—missing target
  key, empty current-shape collections, preserved legacy alias, and effect-name fallback.

Result:

DCA-H-09, DCA-H-10, and DCA-L-03 remained reproducible. The failures were contract failures, not
fixture/compiler noise.

Decision and next step:

Replace Unity send/read paths with the exact typed fields and update current mocks without aliases.

## 2026-07-14 — Wire identity correction and envelope proof

Evidence:

- `GameCommandBus.UseConsumableItem` sends `target_employee_uuid` and no `employee_uuid` alias.
- `RecruitmentCandidateView` reads candidate `id`; `RewardOptionView` reads reward option `uuid`.
- `RosterModel` reads active modifier `name` only.
- `CombatResultModel` exposes `PrimaryAbnormalityId` from `primary_abnormality_id`.
- Ark mock HQ/reward/combat-result/roster fixtures now use actual Rust snapshot identities.
- Core tests pin nonempty HQ candidate `id`, reward `uuid`/definition `id`, absence of snapshot
  aliases, and structural equality of root `selected_event.event` with `EventState.event`.
- Focused Unity correction job `ad3e6dcc45094b5e87d3bd92acacf6b1`: 5/5 passed.
- Core Event envelope 1/1, HQ identity 1/1, and reward identity 1/1 passed.

Failed experiment retained:

- An admin HQ fixture was initially used for the candidate serialization assertion, but its empty
  candidate catalog correctly returned zero candidates despite `candidate_count: 2`. That failed
  assertion was not counted; the assertion moved to the actual gameplay HQ fixture with three
  authored candidates.

## 2026-07-14 — Canonical and broad validation

Evidence:

- Unity canonical root example now includes `visual_novel` and `skill_catalog`; skill examples use
  `range_policy`; completion names `FloorAdvanced`; combat result uses
  `primary_abnormality_id`; HQ/reward identities and active modifier `name` are explicit.
- The noncombat probe and implementation goal use the same identities. Core runtime contract pins
  the shared Event envelope and field ownership.
- Core all-targets: 723 passed (`682 + 3 + 3 + 18 + 13 + 4`), 0 failed/ignored.
- game_server all-targets: 28 passed, 0 failed/ignored.
- Unity EditMode job `dd7656da1c664d338f548e42e0d8d5fe`: 194/194 passed.
- Unity ordinary PlayMode job `e3a8a9a9a6144c8c806b781e6881364e`: 51 passed, 0 failed,
  2 explicit environment-dependent live tests skipped.
- Unity post-clear error console: 0.
- Core/server formatting and diff checks, scoped Unity diff check, exact field searches, alias
  searches, and relevant ignored-test searches passed.

Result:

All seven findings are closed. The Event envelope is stable for G03, with no compatibility reader,
mock-only identity, or game_server JSON repair.

Decision and next step:

Completion review returns Complete / High / no immediate correction. Unlock G03.
