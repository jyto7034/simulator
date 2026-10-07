# Game Rulebook Policy Implementation Master Phase 02 — Notes

## Structural Judgment

- P02 keeps WP001 atomic. Research removal, source/cycle authority, BossOmen, V4 and server mapping cannot be safely
  completed as independent production releases.
- Internally the implementation may use bounded test-first units, but no unit receives a Phase completion claim
  until the combined development boundary is runnable.
- P01 is the direct production dependency and its 150/200 checked-stat seam must be preserved.

## Source-Of-Truth Boundary

- One Core graph owns run-local source/cycle/mastery/attempt/chain/checkpoint state.
- Live RON owns explicit source identities, exact fragment and repeat reward data and BossOmen conditions.
- game_server stores exact V4 and maps committed Core DTO; it does not infer progression.
- Research/pending authority is removed, not hidden behind old names or compatibility.

## Dirty Worktree Warning

- Activation status shows broad pre-existing changes across Core battle/world/data, live RON and game_server.
- Many are unrelated tactical/Boss-runtime work; relevant paths may also contain current user development.
- P02 will snapshot exact diffs and classify target seams before editing. It will not reset, stash, restore, delete,
  whole-file replace or run a write formatter across user changes.
- If a target hunk cannot be merged safely, P02 stops rather than treating broad Master authority as ownership of
  unrelated work.

## Consumer Scope

- Confirmed P02 development boundary: Core, production live RON, game_server mapping/store.
- Unity inspection/mutation/runner remains explicitly excluded under the user's current instruction.
- This exclusion permits a server/Core development cutover but no Unity compatibility claim. No fallback, alias or
  ignored field will be added for the old client.

## Implemented Authority Result

- `EncounterProgressionState` is the single run-local owner for completed reward-source IDs, mastered Elite and
  NormalBoss cycles, Monolith and BossOmen. `RunState`, checkpoint and Active Run V4 embed this graph exactly.
- Live PVE RON owns 29 stable source rows. Core validation owns uniqueness, class-specific 5/10/20 repeat dust,
  encounter/abnormality/fragment cross references and ordinary Reward exclusion.
- Direct source rewards commit in combat result completion. There is no research progress, pending delivery or
  safe-zone collection authority.
- BossOmen live RON owns per-chain min Floor, optional exact fragment/awakening, dependencies and ordered sources;
  Core owns deterministic cycle/provisional/active/capstone runtime.
- Core payload and game_server envelope accept exactly V4. V2/V3 and removed/unknown shapes are errors, not
  migration inputs.

## Important Correction Found During Exit Work

The first direct NormalBoss integration test exposed a cross-boundary overwrite: terminal node planning captured
the old encounter graph, then Floor commit replaced the just-committed source ledger. The correction passes the
post-victory graph explicitly into completion planning. This seam is now covered by first/repeat/overflow rollback
tests and the forced FinalBoss victory→Gate→next-Floor test.

## Alternatives Not Applied

- No V3 migration/transform or dual compatibility period.
- No separate live FinalBoss content beyond WhiteNight; multi-chain proof is synthetic.
- No sweep command/result before P04.
- No permanent profile or fixed Standard fields before P03.
- No Unity cleanup inside this Phase.

## Final Risks And Boundaries

- The 29 sources and one WhiteNight chain are minimum development content, not final balance or encounter design.
- Unity remains deliberately uninspected and may not yet understand V4/current DTOs. Core/server do not carry an
  old-client fallback, and P02 makes no Unity compatibility claim.
- P04 owns the user-facing sweep action and its missing-count reacquisition branch. P02 exposes only the stable
  source/mastery/repeat ledger seam that P04 will consume.
- P03 owns fixed Standard maps and the successor permanent profile. P02 does not use or pre-author their fields.

## Initial Risks

- Broad dirty overlap may make safe ownership attribution difficult.
- Removing research affects state, reward, checkpoint, DTO, tests and canonical simultaneously.
- V4 and server durability must be changed without a partial accepted-response state.
- BossOmen event-started combat and active-chain failure semantics cross event, combat and Floor generation paths.
- The exact 29-source authoring may expose missing data validation or source identity assumptions.

## Activation Baseline Judgment

- Core 1065 and game_server 63 tests are green on the P02 activation tree.
- All 29 bootstrap encounter/abnormality/fragment triples remain present.
- Explicit source/repeat authoring is absent and research is live; these are confirmed target gaps.
- The one live `white_night` chain and its source references remain available, so no replacement FinalBoss content
  is needed.

## Exit Judgment

- Exact Core package: 1055 passed across unit and five integration binaries; no failures or ignored tests.
- Exact game_server package: 63 passed; no failures or ignored tests.
- Focused encounter progression 6, Monolith 22, BossOmen 15, V4 7, live RON 75 and server active-run 16 all pass.
- Current canonical documents describe V4, immediate source reward commit and strict source/BossOmen authoring.
- Scoped format/diff checks pass. Broad dirty user work remains present and was not reset, stashed or deleted.
