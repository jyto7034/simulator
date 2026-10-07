# Phase 11 CRA-P08-C01 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-P08-C01, Master Phase 11 of 44.
- Direct predecessor Phase 10 is `Complete / High`; Phase 12 remains pending.
- Mutation authority is limited to runtime occurrence collection/materialization, scoped callers, exact tests and
  nearest canonical/live contract evidence.
- All three Child documents must be read fully before Phase 11 production/status/data inspection.

## Authority Notes

- Item lookup order and proc state already have owners; the target is one occurrence plan, not a new item database.
- Keep current-hit modifiers explicit. Ordinary `Effect` commands and ability activations share occurrence context but
  do not become one generic untyped command.
- Damage formula/application and skill delivery remain leaf authorities. Dispatcher output may call them through
  current typed commands without consolidating their lifecycle.
- Death finalization remains the owner of when OnKill/OnDeath/OnAllyDeath occur and which units are inactive.

## CRA-POL-002 Notes

- OnHit CounterpartUnit means attacker. Counterpart BonusDamage is retaliation, never incoming-hit amplification.
- Calculate once from the triggering committed-hit snapshot and authored damage type; attribute to effect owner.
- Missing attacker or invalid/dead delivery target must follow the approved matrix/current committed-damage rules,
  never retarget or silently become amplification.
- Retaliatory damage must not collect OnAttack/OnHit item procs, preventing reflection loops.
- Multiple authored retaliation effects retain deterministic item/effect order and each executes at most once.

## Dirty Worktree Risk

The workspace contains extensive user-owned BattleCore, world combat, item/data, server, test and canonical changes
from the campaign and prior work. Fingerprint each prospective target before editing. Do not reset, restore, clean,
stash or broadly format; stop if the required hunk cannot be separated.

## Testing Notes

- The parasite proof must use the production live item definition, at least two active allies and an enemy, and
  assert exact per-ally StatChanged/final Attack/order—not merely activation presence or command success.
- An effect+activation case must observe durable command/event ordering.
- Cover all six triggers; OnBattleStart is not a substitute for attack/death matrices.
- Preserve proc identity/cooldown/max count and dead-caster activation tests; record exact nonzero runner counts.
- For retaliation, assert defender incoming HP excludes bonus, attacker receives the typed bonus once, cause/source
  is correct and reflective equipment cannot recursively bounce.
- Zero-test binaries and success-only assertions are not proof.

## Out-Of-Scope Follow-Up Handling

Keep runtime MaxHealth death under Phase 12, publication reference/matrix enforcement under Phase 13, committed-hit
consolidation under P08-C05 and public false-death cleanup under P08-C06. Record independent findings without
absorbing them.

## Completion Boundary

- Phase-exit verdict: `Complete / High`; improvement class `none`.
- Audit-time test gaps for AllEnemies and fatal-owner released retaliation were closed with test-only corrections and
  the full audit was restarted afterward.
- P08-C04 publication validation, P08-C05 committed-damage consolidation and P08-C06 cleanup remain later Master
  Phases. They are not compatibility branches or unresolved Phase 11 findings.
