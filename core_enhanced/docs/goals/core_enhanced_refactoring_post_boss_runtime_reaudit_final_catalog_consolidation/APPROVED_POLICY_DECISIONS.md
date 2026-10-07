# Approved Policy Decisions

Approval status: `approved`
Approval date: 2026-08-09 (`Asia/Seoul`)
Approval authority: user approval of the recommended defaults
Applies to: the 44 open identities in the rebaselined candidate catalog

These decisions close the five explicit policy choices identified by the reaudit and establish the default
compatibility posture for their dependent refactors and cleanup. They define implementation contracts, but do not
by themselves authorize production edits: each candidate still requires its own authorized `implement_and_verify`
Goal, named safety nets and dependency gates.

## Decision Index

| Policy ID | Approved decision | Direct candidates | Status |
| --- | --- | --- | --- |
| CRA-POL-001 | lethal MaxHealth mutation uses normal death finalization and causal source attribution | P08-C02; bounds P08-C05 | approved |
| CRA-POL-002 | OnHit CounterpartUnit BonusDamage is non-recursive retaliation against the attacker | P08-C04; bounds P08-C01/C05 | approved |
| CRA-POL-003 | invalid projectile/basic definitions fail strict atomic publication | P09-C01; bounds P09-C05 | approved |
| CRA-POL-004 | cast usefulness is any-step; execution usefulness is current-step | P09-C03 | approved |
| CRA-POL-005 | public effect preview is the deterministic union of step footprints | P09-C04; bounds P10-C03 | approved |
| CRA-POL-006 | current save/RON/wire/public API compatibility is preserved by default | cross-cutting compatibility and cleanup gates | approved |

## CRA-POL-001 — Lethal MaxHealth Mutation And Kill Credit

When a runtime MaxHealth modifier clamps an active unit's current HP to zero, the transition is lethal and must enter
the same production death-finalization authority as ordinary committed damage. An active zero-HP unit is never a
valid terminal state.

- Preserve the causal modifier provenance through `source_instance_id`.
- When a causal source unit exists, that unit is the killer and receives the ordinary kill credit. When the cause is
  system/environmental or otherwise has no source unit, the death has no killer and grants no kill credit.
- Commit the stat/HP/death transition atomically. Observable causal order is StatChanged, then HpChanged when HP
  actually changed, then the existing idempotent death-finalization sequence.
- Nonlethal MaxHealth changes retain current behavior except for the missing HP event/provenance correction.
- Decision-table tests must cover flat and percent reductions, nonlethal/lethal transitions, sourced and unsourced
  causes, exact event order, complete cleanup and exactly-once finalization.

## CRA-POL-002 — OnHit Counterpart BonusDamage Is Retaliation

For an OnHit effect, `CounterpartUnit` means the attacker. Therefore OnHit `CounterpartUnit + BonusDamage` is a
retaliatory damage application against that attacker; it must not increase the damage received by the effect owner.

- Calculate the bonus component once from the triggering committed-hit snapshot using the existing BonusDamage
  arithmetic and authored damage type, then deliver that component to the attacker through the committed-damage
  authority.
- Attribute the retaliation to the owner/source of the triggered effect so ordinary HP, death and kill-credit rules
  remain available.
- Apply each authored retaliatory effect at most once for the same trigger occurrence.
- Retaliatory delivery does not re-enter OnAttack/OnHit damage-proc collection. This prevents reflection loops while
  retaining ordinary damage, HP and death finalization.
- Publication/runtime matrix tests must distinguish retaliation from current-hit amplification and cover missing
  attacker, dead participants, multiple authored effects, ordering and no recursive chain.

This decision resolves the existing `thorned_lantern_guard`-style live authoring conflict without changing the
generic OnHit Counterpart meaning used by non-BonusDamage effects.

## CRA-POL-003 — Strict Projectile And Basic-Delivery Publication

RON and programmatic construction obey the same typed publication contract. Formerly constructible values that the
runtime cannot execute faithfully are invalid and must be rejected before publication.

- Projectile speed and every present optional count/time/limit field must be strictly positive.
- Basic attacks and weapon profiles accept only target-locked delivery fields that their runtime executes. Skill-only
  targetless, directional, area, lifetime, kill and pierce policies are rejected rather than ignored.
- Return field-specific typed errors with the exact object/path/cause.
- Publication is atomic: no partial startup, silent normalization, implicit instant fallback or permissive legacy
  branch is allowed.
- All current live data must continue to load; negative tables must cover both RON and programmatic construction.

## CRA-POL-004 — Multi-Step Usefulness

Overall cast admission uses an any-step rule over actual delivery targets: a cast is useful when at least one of its
steps has a useful executable effect after footprint materialization, clipping and target filtering. A useless anchor
does not make a useful affected footprint invalid, and a useful anchor does not make an empty/filtered footprint
valid.

After admission, each step's execution/no-op gate evaluates only that current step's actual delivery set. A no-op
current step does not retroactively cancel a cast admitted because a later step is useful. Stable target ordering,
ties, current release timing and existing no-target lifecycle behavior remain preserved.

Tests must cover useful-later/useless-first and useful-first/useless-later skills, anchor-excluding footprints,
clipped cells, friendly/hostile filters, manual and automatic acquisition, and execution parity.

## CRA-POL-005 — Union Effect Preview

The existing public `effect_preview_cells` representation remains one collection. For a multi-step skill it is the
deduplicated deterministic union of every step's actual spatial delivery footprint for the previewed cast; cast
reach remains a separate field and is never substituted for effect footprint.

- Use the same typed footprint query, board clipping and facing transforms as runtime delivery.
- Preserve the existing DTO/tag/nesting shape and exact server/Unity serialization.
- Instant/projectile steps without a spatial area contribute no area cells.
- Produce canonical deterministic cell order after deduplication and test it at edges and across facings.
- Per-step preview is not added under the current contract. A future UI requirement needs a versioned DTO/consumer
  migration approved separately.

## CRA-POL-006 — Compatibility-Preserving Default

Candidate implementation preserves current externally observable contracts unless a later, separately identified
policy explicitly approves a migration.

- Current V3 checkpoints are strict current-version documents. Internally inconsistent graphs fail before mutation;
  automatic repair/rebind and old-version migration are out of scope.
- Live RON publication is strict, whole-bundle and fail-fast. Candidate-specific invalid combinations approved above
  are rejected; unrelated fields are not silently removed or normalized.
- Current command/result tags, payloads and nesting—including direct-map RunComplete, top-level setup/update and
  nested resync—retain exact parity.
- Public Rust symbols, serialized fields, package targets and external fixture tools are not removed or narrowed on a
  negative workspace search alone. Require a consumer inventory plus deprecation/versioned migration, or a new
  explicit breaking-change approval.
- Boss runtime remains private and non-serde until a real public producer and consumer exist; a first Boss wire needs
  its own version and rollout policy.
- Unknown deployed Rust/Unity consumers remain an evidence gate. Internal/private consolidation may proceed when it
  does not alter their observable contract.

Admin durability/publication semantics and any unrelated future feature migration remain outside these decisions.

## Supersession Rule

An implementation Goal must cite every applicable `CRA-POL-*` ID in its acceptance criteria and add decision-table
or golden compatibility tests before changing behavior. A future policy may supersede one of these decisions only by
naming the affected ID, migration scope, consumer rollout and rollback behavior explicitly; incidental refactoring
does not supersede policy.
