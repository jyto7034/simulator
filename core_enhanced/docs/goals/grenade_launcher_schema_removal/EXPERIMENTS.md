# Grenade Launcher Schema Removal Experiments

Status: complete

## 2026-07-18 — Child Initialization

- Read Master G06 scope, dependency order and approved P06.
- Confirmed the child path was available and no child dirty state existed.
- Mandatory child documents created before schema/reference search or mutation.

## 2026-07-18 — Current Schema Removal

### Inventory

- Production authority consisted of one enum variant and one exhaustive test-helper match arm.
- No live equipment or skill-fragment RON used the variant.
- External current Unity contract and visual briefs still listed/assigned the variant.
- External target documents already contained unrelated user changes; each was copied from its exact
  dirty working version, minimally patched, reviewed as a one-purpose diff, then copied back.

### Implementation and documents

- Removed `WeaponArchetype::GrenadeLauncher` and its ranged helper match arm.
- Added `removed_grenade_launcher_weapon_archetype_is_rejected` to prove RON fails closed.
- Removed the value from current external enum and prompt inventories.
- Changed Melting Love/Adoration's former current assignment to “current archetype undecided” and
  preserved the canister/sprayer shape only as a future visual concept.
- Added explicit future/non-current and joint-design reintroduction rules to Core and external visual
  content documentation.
- Historical audit/goal references were preserved.

### Validation

- Focused removed-value regression: 1 passed.
- `cargo test -p game_core --test ron_loading --quiet -- --test-threads=1`: 20 passed.
- `cargo test -p game_core --lib --quiet -- --test-threads=1`: 739 passed.
- `cargo fmt --all -- --check`: passed.
- Core and external `git diff --check`: passed.
- Production search finds `GrenadeLauncher` only inside the intentional negative test; durable
  current docs mention it only in explicit removed/future wording.

### Disposition

- `DCA2-BATTLE-002` and P06 complete.
