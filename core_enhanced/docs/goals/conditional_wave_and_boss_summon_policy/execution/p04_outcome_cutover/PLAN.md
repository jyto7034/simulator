# P04 Outcome Cutover Execution Plan

- Goal ID: `GOAL-CWBS-P04-OUTCOME-CUTOVER`
- Phase ID: `PHASE-CWBS-04-OUTCOME-CUTOVER`
- Final Execution Nodes: `AG-CWBS-04-OUTCOME-CUTOVER` (`WP-CWBS-08 + WP-CWBS-09`)
- Accepted Handoff / Goal Set: `HA-CWBS-P2G60-002 / GS-CWBS-P2G50-002`
- Run Mode / Root Target: `goal-run / GOAL-CWBS-MASTER`
- Authorization Ref: `GEXAUTH-CWBS-001`
- Current Pass: `GEX-20 complete / Audit Pass 1 / Complete / High`
- Direct Dependency: `GOAL-CWBS-P03-SUMMON-CUTOVER / Complete / High / Audit Pass 1`
- Execution Directory:
  `core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/execution/p04_outcome_cutover`
- Document Set: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`; fresh GEX-20 creates `COMPLETION_AUDIT.md`

## Objective and Scope

Replace direct winner, the system battle-time cap and implicit Draw/Defeat with authored stable terminal outcomes. Settle
the complete simulation-time bucket, choose exactly one matching outcome with Defeat-first ordering followed by priority
descending and stable outcome ID ascending, and project the committed ID through result, node, reward, progression and
BossOmen consumption as one idempotent authority.

Covered revisions: `PER-CWBS-BL-01-CODE-003` through `PER-CWBS-BL-04-CODE-003`.

In scope: terminal outcome schema/compiler/evaluator, neutral closed conditions, committed outcome latch, result and public
projection, exhaustive node outcome policy, outcome-specific reward/bonus/progression, Defeat retry/failure semantics and
BossOmen capstone Victory/Exit Gate integration.

Out of scope: compendium profile replacement and final Victory-only registration (P05), new production encounter content,
Unity-specific presentation changes, production promotion and deployment.

## Current Boundary and Target Plan

- Every encounter declares stable named outcomes with a reachable Victory and closed conditions. Direct verdict-producing
  scenario events and implicit runtime verdicts are removed.
- Runtime evaluates outcomes only after the whole current bucket, including Wave and Boss/summon settlement. Any matching
  Defeat outranks every Victory; equal verdicts use priority descending then outcome ID ascending.
- No system maximum battle duration remains. A battle with no matching authored outcome continues, and Pause advances no
  simulation time.
- Each effective combat node policy covers every encounter outcome exactly once. Victory policies own reward mode/IDs,
  bonus and progression intent; Defeat policies own no positive reward or progression.
- The committed outcome ID is carried by the battle finish signal, result state and public summary. Consumption is staged
  in the existing world transaction and cannot be reselected or repaid after reconnect or duplicate completion.
- Retreat remains a separate non-terminal result path. Post-battle run-failure evaluation remains separate from battle
  outcome selection.

## Validation and Completion

- schema/publication rejection: empty/duplicate outcome IDs, no Victory, invalid priority/condition refs, non-exhaustive or
  duplicate node policy, and positive Defeat effects
- deterministic full-bucket paired outcomes: Defeat-first, priority descending, stable ID tie-break and pending Victory
  after Wave/summon settlement
- no implicit Draw, global cap or direct winner producer; authored no-terminal continuation and Pause clock behavior
- outcome ID round-trip through Core/result/public server projection and retry/Retreat paths
- outcome-specific exactly-once reward/progression consumption, no flat dual reward authority and duplicate completion
  rejection
- BossOmen capstone Victory handles chain and Exit Gate; Defeat retains existing retry/failure decision
- focused invalidated seams only in P04; the inherited 1,152-pass full baseline remains reserved for P06

## Stop Conditions

- any live encounter or effective node policy cannot be published exhaustively without inventing policy
- committed outcome, reward and progression cannot share the existing transactional consumption boundary
- the implementation requires P05 compendium schema replacement, Unity work, production promotion or external deployment
- a policy/scope decision is required, or overlapping existing dirty changes cannot be preserved safely
