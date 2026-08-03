# Reward And Growth Canonicalization Completion Review

Review date: 2026-07-14

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Item Audit

| Finding | Integrated result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| DCA-M-27 reward documentation ahead of live catalog | current fixed combat rewards separated from generic/future reward capability | Complete | High | None |
| DCA-M-29 XP/level/tier distribution underdocumented | exact live policy, runtime calculation, result, and projection boundaries canonicalized | Complete | High | None |

## Runtime And Live-Data Review

- `GrantExecutor` is the single typed effect executor for generic reward/session/event/combat use.
  Callers stage all mutable grant state and publish only after every effect succeeds.
- Combat completion applies attempt employee conditions and per-employee survival XP in the same
  staged state used for result commit. Victory then adds fixed ClaimAll encounter rewards; defeat
  skips core rewards but retains the same committed survivor rule. Retreat restores pre-entry state.
- Live PVE encounters author fixed, nonempty reward UUID lists and ClaimAll. Exact integration
  evidence limits current effects to Enkephalin, equipment material, fragment research, and direct
  fragments. Equipment-pool rewards remain reachable from deterministic Reward nodes, not combat.
- Live growth policy owns 100-XP repeated requirements, 10 per-employee survival XP, and tier
  thresholds I/II/III at levels 1/3/6. Validation rejects empty/zero XP curves and invalid tier order.
- Employee experience retains current-level remainder and can cross multiple levels. Battle tier is
  level-derived projection, not a stat multiplier. GrowthStack is separate; KillStack is the only
  retained flat-attack interpretation and has no production producer.
- Live `experience_reward` keeps its existing effect/amount/target and now has accurate presentation
  text rather than claiming an unimplemented direct stat increase.

## External Contract Review

- Employee roster snapshots already expose level, experience, and derived battle tier.
- Reward and combat reward results already expose typed authored-effect
  `employee_experience_diffs`; automatic survival XP remains visible in node outcome experience
  before/after and the authoritative subsequent roster snapshot.
- game_server introduces no reward/growth mapper and passes typed Core payloads through. Current
  Unity code has no dedicated XP-diff reader, so no client-local calculation or competing authority
  exists. No DTO/transport change was required.

## Canonical Review

- The rulebook now distinguishes generic reward capability, current live combat rewards,
  deterministic Reward-node selection, authored XP total-pool distribution, and automatic survivor
  XP.
- The runtime contract records staging/atomicity, target eligibility, diff semantics, level/tier,
  and GrowthStack boundaries.
- The loading contract records the growth-policy validation and overflow selection rules.
- E.G.O residue, blueprint, probability candidate, random combat equipment, and direct level stat
  increase are explicitly not current behavior rather than silently presented as live policy.

## Validation Review

- Focused exact tests: level/tier 1, XP split 1, survival/redeploy XP 1, GrowthStack pipeline 1, live
  XP target 1, live fixed PVE set 1; all passed.
- Full live RON integration: 19 passed.
- Core all-targets: 737 passed, 0 failed, 0 ignored across nonzero targets.
- Formatting, diff check, current/future reward phrase, growth producer, and Unity consumer searches
  passed.
- game_server/Unity suites were not rerun because no schema, DTO, transport, mapper, or client code
  changed.

## Legacy And Fallback Audit

- No second XP executor, local Unity distribution, default target, level-up growth-stack mutation,
  tier stat fallback, or random combat equipment path was added.
- Historical goal/audit files may retain earlier future-direction text as history; the canonical
  rulebook/runtime/loading documents and live presentation now state current behavior.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: runtime/live-data behavior remains unchanged while canonical ownership now matches the
  data-driven growth policy and typed grant executor. Future reward content extends definitions and
  references; future growth balance changes the validated policy instead of adding code/client
  constants.

## Review Conclusion

G23 is Complete / High with no immediate correction. G16 may begin.
