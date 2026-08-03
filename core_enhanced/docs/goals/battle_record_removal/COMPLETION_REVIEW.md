# Battle Record Removal Completion Review

Review date: 2026-07-14

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Item Audit

| Finding | Integrated result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| DCA-M-01 gameplay battle-record retention | removed from run state, every copy/restore path, and public query | Complete | High | None |
| DCA-L-05 checkpoint record preservation | obsolete preserve behavior removed with the state it served | Complete | High | None |
| CPR-H-03 diagnostic failure authority | battle-keyed trace is external and failure cannot reject CombatResult | Complete | High | None |
| PD-12 player battle-history removal | no replacement history, migration field, identity policy, DTO, or UI was added | Complete | High | None |

## Runtime Review

- `RunState` contains map/progression, research/history, BossOmen, sessions, previews, and attempts,
  but no completed battle collection. New-floor, run-complete, and checkpoint-load construction have
  no battle-record copy or preserve step.
- `GameCore` has no completed-record getter or store method. Active `CombatBattleState` remains only
  as current node content through the existing CombatResult lifecycle, and the event-log attachment
  reads that active content directly.
- `world::combat_trace` is a separate diagnostic module with no state reference. Its output identity
  is the actual battle UUID; combat node UUID is retained only as trace metadata.
- The live finish seam catches every trace export error, logs it, then publishes active result
  content and transitions to the same CombatResult. A forced filesystem failure proves this path.

## Data And External Contract Review

- No live RON or loader schema changed; battle history was runtime-only and never a live content
  contract.
- No Core serialized snapshot/result DTO, game_server mapper/message, or Unity reader contained a
  battle-record collection. Searches across all three layers found no removed field or alias.
- Current CombatResult stats and event-log attachment remain unchanged for the active result screen.
  Story documents remain an independent collection surface.

## Canonical Review

- The runtime contract now says result facts end with the active content lifecycle and defines only
  best-effort `target/combat_traces/run_<seed>/<battle_uuid>.json` diagnostics.
- The rulebook describes current-battle stats/event summaries, not accumulated player history.
- The engineering workflow classifies combat traces as regenerable, non-authoritative output whose
  failure cannot affect gameplay.
- Historical goal/audit/tmp files may retain the removed terminology as evidence; they are not
  current contract sources.

## Validation Review

- Exact state preservation tests: next floor 1, run complete 1, checkpoint restore 1; all passed.
- Retreat/rollback group: 5 passed.
- Live diagnostic/result tests: defense smoke 1, live RON route 1, forced export failure 1; all
  passed.
- Core all-targets: 738 passed, 0 failed, 0 ignored across nonzero targets.
- All-target check, formatting, diff, Core/server/Unity legacy-name, alias, and ignored-test audits
  passed. Server/Unity suites were not rerun because no serialized or client-facing contract changed.

## Legacy And Fallback Audit

- No `battle_records`, `battle_record`, abnormality-keyed output path, public getter, compatibility
  field, serde alias, silent replacement store, or G29 compendium placeholder remains in production
  code/current contracts.
- The tracked repository-root generated record files were already deleted at the dirty G16 baseline;
  their deletions align with PD-12 and were not restored or rewritten.
- Diagnostic files are never read back by runtime, result projection, save, checkpoint, server, or
  Unity code.

## Cross-Component Findings

- Active CombatResult data and optional developer traces have distinct lifetimes and no synchronization
  requirement.
- G29 receives a clean boundary: permanent encounter knowledge must be a game-global monotonic set at
  node consumption, not an adaptation of the removed record schema.
- No conflict with G17 identity rules or G23 reward/growth result commit behavior was found.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: one obsolete authoritative state domain and all of its copy/dedupe/query policy were
  deleted. Diagnostics are explicitly external, keyed by the correct occurrence identity, and
  unable to influence gameplay. Future compendium work starts from its own accepted global-profile
  contract instead of synchronizing against historical battle payloads.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

None inside G16. G29 is a separate accepted feature, not a G16 refactor.

## Policy Questions

None.

## Review Conclusion

G16 is Complete / High with no immediate correction. G29 may begin.
