# Phase 08 Boundary Audit

## Authority Result

- Trigger occurrence collection and command materialization belong to BattleCore; publication validity belongs to
  GameData equipment/skill/buff catalogs. C01 and C04 share a matrix but remain runtime versus publication work.
- Runtime stat transition must bridge UnitStats arithmetic with Core HP/death/event authority. Persistent prebattle
  stat composition remains separate and event-free.
- `calculate_damage` is the retained pure formula. A C05 committed-hit authority must surround it without absorbing
  P09 target selection, projectile collision or area lifetime.
- `core::death_runtime::finalize_unit_death` is production death authority. Top-level `battle::death` is disconnected
  and cannot be used as the implementation base.

## Boss Boundary

Boss runtime controls task ownership, action admission and cancellation epochs. Released projectile/damage/effect/
death work stays under the ordinary leaf systems, as both current source and the released-projectile cancellation
test show. This is a consumer seam, not a parallel P08 authority. C01/C05 remain independent and no new Boss
candidate is introduced.

## Policy And External Gates

- C02 implementation requires the modifier-caused death/kill-credit decision; test characterization and nonlethal HP
  correctness do not.
- C04 reference validation is policy-free. Freezing OnHit Counterpart BonusDamage compatibility still requires the
  authored target-meaning decision.
- C05 public carrier pruning and C06 public module deletion require accessible/external Rust consumer inventory or an
  explicit breaking-cleanup decision. No server/Unity DTO maps these private runtime carriers in current workspace
  evidence; inaccessible consumers remain unverified.
- P09 owns delivery/geometry integration; P10 owns public command/result projection. Neither is claimed complete here.

## Reconciliation And Handoff

All six original rows remain valid with their original work types. No `CRR-P08-*` row is handed to Final. P09 may
consume the retained pure damage/effect leaves while independently auditing targeting, delivery and projectile
authority.
