# Phase 14 Goal — CRA-P09-C02 Resolve Max-Kills Projectile Delivery Exactly Once

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P09-C02
Master Phase: 14 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 Master execution request remains active; authorization
is limited to this Phase after Phase 13's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Make projectile stop-policy termination and skill-cast delivery resolution one exactly-once lifecycle. When an
accepted directional projectile impact reaches `max_kills`, the projectile and its owning active cast must both
disappear through one terminal delivery resolution, including when max-kills, max-hits and expiry become true at the
same timestamp. No stale nonterminal handoff, pending-delivery leak or duplicate step/cast completion may remain.

This Phase is complete only when the current cast→released projectile→advance→queued impact→damage/kill result→stop
policy→delivery resolution→step/repeat/cast cleanup graph is identified; the existing max-kills path and combined
stop-policy boundaries first reproduce the leaked cast; one typed stop outcome is consumed by the existing cast
delivery owner; projectile removal and pending-count decrement happen exactly once for the same impact; first
accepted collision/effect ordering, later-target exclusion, deterministic identity/cause/source snapshot,
repeat/step sequencing and all nonterminal cases remain exact; public/RON/save/wire shapes remain compatible;
focused/broad validation passes; and the phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical audit and reaudit evidence found that fixed directional projectile advancement queues an impact with a
terminal flag derived from collision/expiry state. The impact handler later applies `max_kills`, which can remove the
active projectile after damage, but passes the stale original `terminal = false` to the cast delivery resolver.
`pending_delivery_count` then remains one and the active cast can leak indefinitely. The current max-kills test proves
damage exclusion only and does not assert projectile/cast cleanup or exactly-one completion.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 14 registry row.
- Direct linear predecessor: Phase 13 CRA-P08-C04, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_13_p08_c04/COMPLETION_AUDIT.md`.
- Candidate technical dependency: none. Phase 13 is the Master activation gate, not a terminal-delivery semantic
  prerequisite.
- Downstream seam: Phase 29 CRA-P09-C05 may extract shared projectile motion only after P09-C01 publication,
  P09-C02 terminal delivery and P08-C05 committed-hit boundaries are frozen.
- Historical evidence:
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/TEST_AUDIT.md`
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/BOUNDARY_AUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/BOUNDARY_AUDIT.md`
- Final execution evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

No candidate-specific gameplay decision is open: the correction restores the existing terminal-delivery lifecycle.
Cross-cutting CRA-POL-006 still preserves current public/RON/save/wire contracts and forbids a compatibility shim or
unrelated migration.

## Priority And Current Disposition

- Priority axes: `1 / 3 / 3 / 2 / 3 / 2 / unknown`; known subtotal `14/18`.
- Final disposition at activation: `still_valid`, correctness correction, exact terminal tests first.
- Current live-tree disposition: reproduced and corrected test-first. A private typed post-impact stop outcome now
  reaches the existing delivery resolver; broad/static validation is green and independent completion review is
  pending.

## Current And Target Authority

- Historical current authority: projectile advancement and impact handling own motion/collision and may remove the
  projectile as a side effect of kill-stop; `resolve_skill_step_delivery` separately owns pending delivery, step,
  repeat and cast cleanup and receives the pre-kill-stop terminal flag.
- Target authority: stop-policy evaluation returns one typed terminal outcome to the existing delivery resolver.
  The same impact causes projectile retirement and one pending-count transition, and only that resolver completes
  the step/cast.
- Exact current type/function/file names, event ordering and test construction must be rediscovered after the Child
  read/dirty gate; historical names are evidence, not mutation instructions.

## In Scope

- identify the active skill cast, released projectile, scheduled advance/impact events, collision ordering, damage
  execution summary, hit/kill/expiry stop policies, pending delivery and completion consumers;
- capture expanded dirty status, scoped diff and fingerprints for every prospective runtime/test/canonical target;
- strengthen the real max-kills integration to assert active projectile absence, active-cast absence, exactly one
  terminal delivery/step/cast completion and no later projectile event;
- add same-timestamp multiple-collision and combined max-kills/max-hits/expiry cases, including ties where multiple
  stop reasons become true for one accepted impact;
- introduce the smallest typed terminal outcome at the stop-policy/delivery seam and remove the side-effect-only
  kill-stop/stale nonterminal handoff;
- preserve first accepted impact and effect order, kill/hit accounting, later-target exclusion, deterministic IDs,
  cause lineage, source snapshot, repeat/retarget/step sequencing and released-work semantics;
- update the nearest runtime/skill canonical only after the corrected lifecycle is implemented and verified.

## Out Of Scope

- Phase 15 P09-C01 projectile/basic definition publication validation or live data migration;
- Phase 16/17/18 typed range, usefulness and preview corrections;
- Phase 28 P08-C05 committed-damage consolidation or damage arithmetic/effect meaning;
- Phase 29 P09-C05 shared continuous projectile motion extraction;
- changing collision geometry, sweep timing, target filters, max-kill authored meaning, damage/effects, proc or death
  behavior;
- changing public projectile/cast structs, RON schema/data, save/checkpoint, DTO/wire/server/Unity contracts;
- fallback, duplicate delivery resolver, compatibility alias, hidden side table or future-Phase placeholder;
- unrelated refactor, formatting or dirty-worktree cleanup.

## Preserved Invariants

1. The first accepted collision and committed effect occur in the same deterministic order and with the same cause,
   source snapshot and identity.
2. Reaching max-kills excludes every later candidate exactly as before.
3. Each released delivery increments and decrements pending count once; one impact cannot complete a step/cast twice.
4. Max-hits, max-kills, lifetime/expiry and collision/board-exit termination compose deterministically at one time.
5. Nonterminal impacts retain their projectile and cast until a later terminal event.
6. Repeat, retarget and later-step scheduling remain owned by the existing delivery resolver.
7. Withdrawal/cancellation after release and Boss ordinary-leaf usage remain exact.
8. Current live RON, public Rust, V3 checkpoint and DTO/wire/server/Unity shapes remain exact under CRA-POL-006.
9. No Phase 15/28/29 behavior, compatibility shim, alias or placeholder authority is introduced.
10. Pre-existing user changes are preserved without reset, restore, clean, stash or unrelated formatting.

## Implementation And Dirty-Worktree Strategy

After all three Child documents exist and Master records sole activation, read them completely and reread the Phase
13 completion audit. Then capture expanded status and prospective target diff/hash before inspecting current source,
tests or canonical. Treat existing hunks as user-owned and edit only the terminal-delivery seam, direct behavior
proofs and nearest canonical.

Use test-first proof in this order:

1. Trace the current typed events, projectile and cast state plus exact stop-policy evaluation order.
2. Run nonzero skill projectile/runtime and released-work baselines.
3. Strengthen max-kills and add combined same-time boundary tests; prove projectile/cast leak or duplicate risk red.
4. Introduce one typed outcome consumed by the existing delivery resolver; do not duplicate completion state.
5. Re-run exact reds, complete projectile/cast lifecycle, Boss released-work and live RON suites.
6. Run complete Core, affected server, release and static/dirty/compatibility checks.

## Migration, Deployment And Compatibility

- No RON schema/data, V3 save/checkpoint, DTO/wire, API version or public Rust migration.
- No content rewrite: this is runtime lifecycle correction for already constructible programmatic policy data; live
  RON may not currently author max-kills.
- No dual terminal path or compatibility interval. The existing cast delivery resolver owns immediate cutover.
- Core/game_server validate from one repository state. Unity build/runtime and live WebSocket probes are omitted
  unless current inspection finds a serialized/message-order change, which would trigger a policy stop.

## Validation Plan

- Baseline: skill projectile primitive, skill runtime contract, commands/projectile integration, released-work/Boss
  lifecycle and full live RON with nonzero counts.
- First proof: projectile/cast absence plus exactly-one terminal completion under max-kills; same-time multiple
  collisions and combined max-hits/max-kills/expiry.
- Focused final: terminal and nonterminal decision table, repeat/later-step progression, deterministic event cause/
  identity/source, no later event, withdrawal/Boss released-work preservation.
- Broad: complete affected Core integration families, complete Core lib, affected game_server targets and live RON.
- Static: constructor/caller/removal inventory, stale side-effect-only stop path search, public/serde/save/wire
  disposition, scoped status/diff/hashes, canonical comparison, formatter and `git diff --check`.
- Zero-test filters, helper-only state and compile-only success do not count as lifecycle proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P14-C01 | complete cast/projectile/impact/stop/delivery graph identified | producer→events→stop outcome→pending/cleanup inventory | pass — E02 |
| P14-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diff and hashes | pass — E01/E04 |
| P14-C03 | current max-kills pending-cast leak reproduced | exact nonzero projectile/cast/terminal red proof | pass — E02 |
| P14-C04 | combined stop policies resolve one deterministic outcome | same-time max-hits/max-kills/expiry decision table | pass — E02/E03 |
| P14-C05 | projectile and cast retire exactly once | one terminal delivery/step/cast completion and no later work | pass — E03 |
| P14-C06 | existing collision/effect/repeat/released-work behavior remains exact | focused preservation matrix | pass — E03/E04 |
| P14-C07 | focused and affected broad validation passes | nonzero counts and static checks | pass — E03/E04 |
| P14-C08 | canonical/compatibility and downstream seams are current | contract/consumer/legacy audit | pass — E04 |
| P14-C09 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | pending |

## Stop Conditions

Stop and do not activate Phase 15 if the current tree already resolves max-kills terminal delivery exactly once;
the historical leak cannot be reproduced through a real production path; combined stop reasons require a new
gameplay priority or observable reason contract; correcting lifecycle would change damage/collision/max-kill meaning,
public/RON/save/wire shape or Phase 28/29 authority; dirty overlap is inseparable; a required runner selects zero
tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
