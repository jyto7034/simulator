# P04 Outcome Cutover Completion Audit

- Goal ID: `GOAL-CWBS-P04-OUTCOME-CUTOVER`
- Phase ID: `PHASE-CWBS-04-OUTCOME-CUTOVER`
- Audit Pass: `1`
- Audit Date: `2026-08-17`
- Audited Target: hashes in `EXPERIMENTS.md / Current GEX-10 target identities`
- Dependency: `GOAL-CWBS-P03-SUMMON-CUTOVER / Complete / High / Audit Pass 1`
- Verdict: `Complete`
- Confidence: `High`
- Correction Required: `none`

## Fresh-pass method

This audit began after the P04 implementation record and target hashes were frozen. It re-read the P04 boundary and
mapped `PER-CWBS-BL-01..04` to current runtime, live data, node-policy, world-consumer and server-projection evidence. It
did not infer completion from the GEX-10 summary alone and did not repeat the run-wide full baseline.

## Dependency and target integrity

- P03 remains `Complete / High`; Wave and summon settlement still occurs within the same-time bucket that P04 evaluates
  only after draining.
- Scoped `git diff --check` and `cargo check -p game_core -p game_server` pass at the frozen target. Existing dirty work
  remains preserved; no reset, restore, stash, cleanup or unrelated promotion occurred.
- All 33 live encounters and seven fixed node policies load under the new strict schema. No production deployment or
  Unity change is claimed.

## Requirement-by-requirement audit

| Freeze acceptance | Current authoritative evidence | Audit result |
| --- | --- | --- |
| `BL-01` authored terminal only; postbattle failure separate | Direct `EndBattle`, forced winner, global cap and implicit Draw producers are absent. Unsatisfied authored conditions remain Running; world run-failure logic executes only after committed result consumption | pass |
| `BL-02` full bucket, Defeat-first, priority/ID ordering | Runtime drains the complete same-time bucket before evaluation; deterministic selector uses Defeat, priority descending and ID ascending. Same-time AttackResolve is logged before BattleEnd and explicit tie fixtures pass | pass |
| `BL-03` Pause no clock; same-boundary Defeat | Live pause regression preserves time/resource state; simultaneous matching Victory/Defeat chooses Defeat regardless of priority. No system maximum battle time exists | pass |
| `BL-04` neutral producers, exhaustive policy, exact-once consumption | PVE/node publication requires exact policies; typed Wave/Boss/summon producers expose facts only; result latches ID/verdict; selected policy alone owns rewards/bonus/progression; Defeat is non-positive; transactional rollback/retry/Retreat and BossOmen Victory/Defeat tests pass | pass |

## Focused validation evidence

- test-target compilation: game_core and game_server `--no-run` pass
- terminal/no-cap/live-entry group: `11 / 0`
- schema and runtime rejection predicates: named outcome `1 / 0`, unknown unit `1 / 0`, player death semantics `1 / 0`
- live PVE reader/publication/resolution: `4 / 0 + 1 / 0 + 1 / 0`
- full-bucket and result/public projection: `1 / 0 + 1 / 0 + server 1 / 0`
- atomic completion, retry/Retreat and repeated Retreat: `3 / 0`
- authored-time world and Pause: `2 / 0`
- BossOmen capstone Victory/Defeat: `2 / 0`
- production Core/server check: pass, zero warnings

The exact commands, detected failures and corrections are recorded in `EXPERIMENTS.md`. The inherited full baseline is
still `game_core 1,080 + game_server 72 = 1,152 / 0 failed`; the Master contract reserves its next run for P06.

## Scope and forward ownership

- SavePoint remains battle-free. Same-process active-battle/result state carries the selected outcome; there is no
  durable combat resync or outcome reselection authority.
- Compendium data replacement and the final rule “register only on committed Victory” remain P05 work. P04 only changes
  the temporary current consumer input to the committed verdict.
- New production encounter content, Unity presentation, promotion and deployment remain outside scope.

## Verdict

`GOAL-CWBS-P04-OUTCOME-CUTOVER` is complete at High confidence. All four assigned Freeze acceptances have current
code/data evidence and focused executable coverage, no P04 stop condition is active, and no correction Goal is required.
P05 may begin GEX-00 against this audited target.
