# Reward And Growth Canonicalization Experiments

Goal: G23

Updated: 2026-07-14

## 2026-07-14 — Dirty baseline

- Core: branch `main`, HEAD `c2c5dc5ec54bd26530e46f0d7ee7909de3ed7a1f`; scoped reward/growth
  fingerprint `de6197fc16c0687bc3f79eef47cab6aee03053ccd171621eaf351fab0bd6c814`.
- game_server: branch `main`, same HEAD; full diff fingerprint
  `c4d32a3a60e38e31d295661633976a16666cefbfc34d3df2108ce686a818702e`.
- Unity ArkClient: branch `master`, HEAD `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`; full diff
  fingerprint `ac0fff23bf4bd07ad8693c63af3c2a32955b091f89a364cc63e5f1f2177a4809`.
- All worktrees are heavily dirty. Canonical docs and several runtime trace files overlap completed
  predecessors; G23 will read those diffs before editing and will not touch unrelated hunks.

## Planned trace

1. Enumerate generic reward effect/target capabilities and actual live-authored uses.
2. Trace preview/staging/commit atomicity for reward sessions and combat results.
3. Trace automatic survival XP, participant identity aggregation, level thresholds, tier mapping,
   GrowthStack scaling, DTOs, and Unity consumption.
4. Compare canonical claims to current behavior and correct documentation-only drift unless an
   actual defect is proven.

## 2026-07-14 — Runtime and live-data trace

Reward authority:

- `RewardEffect` generically represents currency, XP, direct/pool equipment, equipment material,
  Artifact, consumable, skill fragment, fragment research, and fragment dust.
- Reward sessions and combat completion clone/stage inventory, fragment, roster, UUID, and currency
  state. A later failed effect prevents every earlier staged mutation from committing.
- Live combat encounters all author `ClaimAll` and fixed reward UUID lists. Their actual effect set
  is exactly Enkephalin, equipment material, skill-fragment research, and selected direct skill
  fragments. No live encounter references experience, equipment pool, Artifact, consumable, dust,
  E.G.O residue, blueprint, probability candidate, or completed-equipment effects.
- Map Reward nodes deterministically choose one non-XP reward definition from their pool. The
  current default pool can choose an equipment-pool reward; this is distinct from combat rewards.

XP/growth authority:

- `GrantExperience.amount` is one total pool. Valid employee UUIDs are sorted, deduplicated, and
  split evenly; remainder is assigned one each from the lowest UUID.
- `AliveRoster` checks persistent Alive only. `AllRoster` includes every roster member, while
  `CombatParticipants` and `SelectedEmployee` require explicit execution context.
- Every committed victory or defeat copies the attempt employee conditions, then grants the
  authored `post_battle_survival_xp` separately to each employee-level result that survived and is
  still Alive. Re-deploy instances aggregate to one employee row. Retreat restores the entry state.
- Current live growth policy is 100 XP per level (the final curve entry repeats), 10 post-battle
  survival XP per eligible employee, and battle tiers I/II/III at levels 1/3/6.
- A single grant can cross multiple levels. `experience` stores the current-level remainder.
- Battle tier is derived and projected but does not apply a generic stat multiplier. GrowthStack is
  separate; only KillStack maps to flat attack and no production path currently adds stacks.
- `employee_experience_diffs` reports authored reward-effect changes. Automatic survival XP is
  represented in node outcome experience before/after and the final roster snapshot instead.
- game_server passes typed results through without reward/growth mapping. Current Unity runtime has
  no dedicated employee XP/diff reader; the authoritative final state remains the root roster
  snapshot.

## 2026-07-14 — Canonical corrections

- Replaced future-facing combat reward claims with the exact current fixed PVE reward set, while
  separately documenting deterministic Reward-node equipment-pool selection.
- Added exact XP target/distribution, multi-level curve, tier derivation, committed-result survival
  XP, GrowthStack, staging, DTO, and consumer boundaries to the rulebook/runtime/loading contracts.
- Corrected live `experience_reward` presentation text: it no longer claims level-up directly
  improves stats and now states that XP is split and raises level/battle tier.
- No balance value, reward definition/effect, encounter reference, progression calculation, schema,
  DTO, server mapping, or Unity behavior changed.

## 2026-07-14 — Validation

- Multi-level/tier regression: 1 passed; 250 XP from level 1 produces level 3, 50 remainder, tier II.
- Exact AliveRoster total-pool split regression: 1 passed; 7 XP across three sorted employees yields
  3/2/2 with exact before/after diffs.
- Employee-level survival XP across redeploy instances: 1 passed.
- GrowthStack-before-equipment stat pipeline: 1 passed.
- Live XP target regression: 1 passed.
- New live PVE fixed effect-set integration: 1 passed; all encounters are nonempty ClaimAll and the
  observed set is exactly Enkephalin/material/research/direct fragment.
- Core all-targets: 737 passed total (695 + 3 + 3 + 19 + 13 + 4), 0 failed, 0 ignored; zero-test
  binary targets are not counted as evidence.
- `cargo fmt --all -- --check`, `git diff --check`, current/future reward phrase searches, growth
  source searches, and Unity XP-reader search passed.
- game_server and Unity suites were not rerun because no serialized shape, transport mapping, or
  client code changed. Full live RON loading (19 tests) passed through the production source set.
