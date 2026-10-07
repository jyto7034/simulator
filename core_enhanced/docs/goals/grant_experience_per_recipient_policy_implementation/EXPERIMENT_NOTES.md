# GrantExperience Per-Recipient Policy Implementation — Notes

## Structural Judgment

- One `GrantExecutor::grant_experience` function owns the amount arithmetic for all four targets, so this is one
  atomic single-Goal contract rather than a Master campaign.
- The target enum, context and result DTO already express the approved policy. No new abstraction or public field is
  required; the legacy is the central quotient/remainder calculation and its assertions/presentation text.
- Gameplay/balance impact is high because logical total XP scales with recipient count, even though code locality and
  migration risk are low.

## Current And Target Source-Of-Truth Boundary

- Current runtime: every resolved target receives the full amount.
- Approved target: every resolved target receives the full amount.
- Gameplay canonical already records the approved target.
- Runtime canonical now records the verified implementation and no longer carries the temporary mismatch sentence.
- Live `experience_reward` description states per-target full payment while retaining amount 200 / AliveRoster.

## Target Semantics

- Alive means `Employee::is_alive()` / physical damage below Dead threshold, not combat-deployable.
- Critical and Breakdown employees remain alive recipients; Dead is excluded only by AliveRoster.
- CombatParticipants and SelectedEmployee require context; AllRoster includes Dead.
- Sort/dedupe and current membership filtering are independent from per-recipient amount and remain unchanged.

## Existing Specialized Paths

- Post-battle survival XP uses SelectedEmployee per surviving participant and must remain participant-scoped.
- Mastered Elite sweep uses SelectedEmployee once for every living roster employee and already realizes full amount.
- P04 could be simplified to one AliveRoster grant after the correction, but that is unnecessary churn and explicitly
  excluded. Focused regression is sufficient.

## Live Data And Balance Boundary

- `experience_reward` is an independent live catalog entry with amount 200 / AliveRoster.
- Current production encounters and `default_treasures` do not reference it, reducing immediate bootstrap impact.
- If later connected, every living employee receives 200 XP and may cross two current 100-XP levels. This is the
  approved semantics applied to the existing value, not an automatic balance endorsement.
- This Goal changes only the description. Any numeric tuning belongs to content/balance work.

## Dirty Worktree Warning

- The repository is broadly dirty from prior user work and completed campaigns.
- `reward.rs`, `node_sessions.rs`, `base.ron`, `game_rulebook.md` and `core_runtime_contract.md` already contain
  changes. Before any edit, capture each target's current diff and classify Goal-owned hunks.
- Do not reset, restore, stash, reformat unrelated files or rewrite historical Goal evidence.
- If the exact arithmetic/assertion/description hunk contains unowned competing edits, stop rather than overwrite.

## Consumer And Migration Scope

- Core is the authority; game_server transports existing diffs and needs only regression verification.
- DTO/JSON/RON schema/save versions do not change.
- Unity remains explicitly uninspected and unmodified. Shape stability means no compatibility fallback is needed or
  authorized.

## Applied Alternatives

- Chosen: remove central distribution arithmetic and keep all callers on one authority.
- Not chosen: multiply authored amount by target count before calling the existing split function; this would hide
  legacy semantics, risk overflow and preserve the wrong authority.
- Not chosen: add a per-recipient flag or new target variants; dual semantics and migration are unapproved.
- Not chosen: caller-specific fixes for Reward/Combat/Sweep; they would create competing amount authorities.
- Not chosen: retune amount 200 in the correctness change.

## Remaining Discovery Before Mutation

- Completed: exact target hunk ownership and fresh four-command legacy baseline are recorded in `EXPERIMENTS.md`.
- The XP arithmetic block, live description line and runtime mismatch sentence have no competing edit. Existing broad
  changes in the same files remain user-owned and must be preserved.
- Concrete test placement is `src/game/world/tests/node_sessions.rs`, using the existing staged
  `apply_grant_effects_with_context` boundary so target resolution, employee mutation and diff projection are tested
  together. Live authored data remains covered in `tests/ron_loading.rs`.

## Omitted And Follow-Up Work

- Membership-invalid context fail-versus-filter semantics is a separate validation question.
- XP balance, new reward entries, content authoring and Unity integration are separate future scopes.
- No other policy question remains for this Goal.

## Final Structural Result

- GXP-C01 removed the quotient/remainder branch from the existing central reducer; no replacement layer, flag or
  caller-specific arithmetic was introduced.
- The four target meanings, deterministic membership handling, staged transaction boundary, growth reducer, DTO
  shape and save versions are unchanged.
- Completion audit verdict is `Complete / High`, improvement class `none`.
