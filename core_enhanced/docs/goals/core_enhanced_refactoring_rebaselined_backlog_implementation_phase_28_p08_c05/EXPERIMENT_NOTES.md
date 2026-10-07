# Phase 28 CRA-P08-C05 Experiment Notes

Status: complete
Implementation Complete: true
Candidate: CRA-P08-C05
Master Phase: 28 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Locked Decisions

- This is a behavior-preserving committed-damage authority consolidation, not formula or combat-policy change.
- `calculate_damage` remains the pure arithmetic authority; existing committed HP/death finalization remains the
  mutation/lifecycle authority.
- Phase 11/13 own occurrence classification/dispatch and CRA-POL-002 one-shot non-recursive retaliation.
- Phase 12/CRA-POL-001 own provenance, HP events, normal death finalization and kill credit.
- Launch-source versus live-target timing, stable roll identity, basic-only OnAttack/OnHit, effect-before-activation,
  actual-delta resonance and event lineage remain exact.
- Current public Rust, RON, save/checkpoint, DTO/wire/server/Unity shapes remain compatible under CRA-POL-006.
- No targeting, projectile motion/collision, skill-area lifetime, DeathHandler cleanup or Phase 29 work is allowed.
- No silent normalization, fallback, alias, dual damage path, public narrowing or test-only production branch.

## Open Evidence Slots

1. Activation status, HEAD and normalized dirty hash.
2. Prospective source/test/data/canonical hashes before content inspection.
3. Complete damage formula/request/context/result/command type and reader inventory.
4. Every instant/projectile/general/retaliation/Boss-released production caller.
5. Source snapshot, target live lookup, roll identity and trigger policy per caller.
6. HP/death/resonance/result/event ordering and actual-delta ownership.
7. `BattleCommand::UnitDied` and every purported ghost-field semantic reader.
8. Public Rust, serde, V3, RON, server/Unity and unknown deployment consumers.
9. Old-production baseline and new characterization counts.
10. Minimal consolidation and duplicate/fallback/public-narrowing absence.
11. Focused/broad debug/release/static evidence.
12. Independent phase-exit verdict and Phase 29 handoff.

## Historical Risk Register

- A shared helper can accidentally recompute launch-source stats at projectile impact or freeze target defense too
  early, changing observable damage under drift.
- Inferring policy from a damage source enum can run basic OnAttack/OnHit triggers or resonance on general skill,
  area, buff or retaliation damage.
- Moving occurrence materialization can reorder effects, activations, retaliation or death relative to HpChanged.
- Using formula-derived `UnitDied` as a second death signal can duplicate death, kill credit, trigger dispatch or
  result counts.
- Applying planned rather than actual HP delta to resonance changes overkill/dead-target behavior.
- Flattening source/target absence or dead-participant handling can alter released projectile and fatal retaliation.
- Removing broad public request/context/result fields after workspace-only search can violate CRA-POL-006.
- Broad damage cleanup can absorb P09 targeting/motion or P08-C06 false DeathHandler cleanup.

## Historical Evidence Summary

- Original P08-C05 found one retained formula and three overlapping committed-hit executors, an ignored
  formula-derived UnitDied command and broad semantically unread carriers.
- Reaudit after Boss runtime found the same authority split; released Boss work continues through ordinary leaf
  damage/effect/death systems.
- Final order requires Phase 11 occurrence, Phase 12 HP/death/provenance and Phase 13 publication semantics first;
  all are `Complete / High` under CRA-POL-001/002.
- Required first proof is identical instant/projectile/general behavior under stat/source/target drift, crit,
  retaliation, kill, lineage and command order.
- Priority is `3 / 1 / 3 / 3 / 2 / 2 / unknown`, subtotal 14, Tier B.

## Activation Boundary

No Phase 28 production source, tests, live data or canonical content was inspected during Child initialization.
After Master sole activation, reread this entire Child plus final Phase 27 and Phase 11/12/13 audits, capture
prospective hashes using path discovery only, and only then begin current-source investigation.

## N01 — Activation Gate Closed

- The post-activation bounded reread is complete. HEAD and the 516-row normalized dirty baseline are recorded in
  E02, together with source/test/data/canonical prospective fingerprints collected before content inspection.
- Existing modifications on prospective targets are ownership boundaries. Before any edit, inspect the current
  worktree content, the HEAD-to-worktree diff and caller/consumer graph; preserve unrelated hunks exactly.
- The 57-byte Phase 27 approval marker is still byte- and metadata-identical. It is neither a test artifact nor a
  Phase 28 output and must remain untouched.
- Phase 28 production investigation may now start. Authorization remains limited to inventory, old-production
  characterization and the smallest behavior-preserving private consolidation; Phase 29 remains forbidden.

## N02 — Chosen Private Seam

- Consolidate only the calculate/commit portion in `damage_runtime.rs`. Callers keep ownership of target
  acceptance, projectile collision/miss/impact records, release resonance, skill lifecycle and autocast scheduling.
- The typed policy must distinguish basic committed hits from general commands and must carry basic occurrence ID
  and occurrence time explicitly. Inferring this from `DamageSource` would erase the current instant-versus-impact
  lifetime and could re-enter item triggers for skill, Buff or retaliation damage.
- The authority should live-read the active target, assemble basic-only occurrence inputs when requested, call the
  retained formula exactly once, commit through `apply_damage_result_and_record`, derive actual HP delta/kill from
  committed state, grant basic dealt resonance from that actual delta, and execute existing effect-before-activation
  batches. It returns the existing count outcome needed by command/skill callers.
- Public formula/request/context/result/command fields and variants remain. Private `AttackTargetSnapshot`,
  `BasicAttackDamageSnapshot` and `BasicAttackDamageResolution` may be removed only after all three committed paths
  use the new private authority and static searches prove them unused.
- New tests will characterize cross-path formula metadata under source/target drift and crit, actual-delta resonance,
  single death/finalization and general-command non-trigger behavior before the production seam changes.

## N03 — Implemented Authority And Preserved Owners

- The chosen seam is implemented as private `CommittedDamageInput` plus typed `CommittedDamagePolicy`. It is the
  only mutating caller of the retained pure formula and the only production caller of the retained HP/death commit
  helper.
- Instant basic, projectile impact and general `ApplyDamage` are the only entry callers. Basic occurrence identity
  and time are explicit inputs, so source snapshot versus live-target timing did not collapse. General damage has no
  path to basic triggers or resonance.
- Target acceptance, range, projectile movement/collision/miss/impact logging, release resonance, skill lifecycle,
  area scheduling and autocast scheduling remain with their prior owners. Phase 29 motion and Phase 38 public
  DeathHandler work were not entered.
- The old private snapshot/resolution layer was deleted only after all callers moved and negative searches proved it
  unused. Public damage carriers and formula-derived `UnitDied` remain for unknown consumers under CRA-POL-006;
  Core still treats that formula command as non-authoritative.

## N04 — Validation And Audit Handoff

- Characterization-first and post-change focused suites are green, including source/target drift, crit,
  actual-overkill delta, single causal death, lineage and general-command exclusion. Broad Core debug/release each
  pass 1,070 tests and broad server debug/release each pass 59 tests.
- Checks, formatting, static authority searches and non-strict Clippy compile gates are green. Existing warning debt
  is not reclassified as Phase 28 work and no strict-warning claim is made.
- All prospective data and canonical inputs remain byte-identical. The expanded dirty path set/hash and the
  user-approved Phase 27 log marker are unchanged.
- Completion conditions P28-C01 through P28-C10 have implementation evidence. P28-C11 stays pending until a fresh,
  separate full-guide audit re-reads the finalized evidence and returns its own verdict.

## N05 — Phase Exit Disposition

- The separate completion audit returned `Complete / High`, improvement class `none`; P28-C11 is closed.
- The durable downstream seam is one private typed committed-damage authority with motion, targeting, skill
  lifecycle and public carrier owners still separate. Phase 29 may rely on that seam but may not reopen P08 policy.
- Unknown deployed Rust consumers remain a recorded nonmaterial risk precisely because every public carrier and
  formula output was retained under CRA-POL-006. No follow-up correction is required in this Phase.

## Dirty Worktree Note

The repository is extensively dirty. Pre-initialization expanded state is 513 rows at normalized SHA-256
`0df73ab3...`. Every existing path remains user-owned. In particular, preserve the current 57-byte one-path exception
approval marker (`614d2fbd...`) unchanged; Phase 27's approval is not a general deletion/cleanup waiver. Never use
reset, restore, clean, stash or unrelated formatting to manufacture a clean baseline.

## External And Compatibility Note

Workspace Core/server/data consumers are accessible. Canonical external Unity evidence is accessible for the
unchanged wire boundary. Standalone/deployed Rust consumers of public battle carriers remain unknown, so private
consolidation is allowed but public removal/narrowing is not authorized without new evidence and rollout policy.
