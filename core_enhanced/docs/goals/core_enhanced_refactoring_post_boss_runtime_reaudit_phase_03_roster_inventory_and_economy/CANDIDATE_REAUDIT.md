# Phase 03 Candidate Reaudit

## Current Dispositions

| Candidate | Current evidence | Boss/current impact | Disposition | Work type/readiness |
| --- | --- | --- | --- | --- |
| CRA-P03-C02 | enhancement publication still permits duplicate material IDs; admission checks each row against the same balance and execution consumes rows from live inventory in order | current Boss equipment setup only consumes enhancement projections and does not change maintenance cost authority | `still_valid` | correctness; test first |
| CRA-P03-C03 | roster slots and inventory `equipped_to` remain separate mutable ledgers coordinated manually by equip/unequip/starter/death/dismantle paths | player Boss setup reads the two-ledger graph but Boss runtime adds no transition owner | `still_valid` | behavior refactor; characterize after F-C01 economy validation |
| CRA-P03-C04 | `GrantExecutor` still receives five mutable ledgers; reward/shop/event/combat/admin callers still clone/pass/commit that field family independently | Boss result handling reuses existing staged combat/grant paths and introduces no grant ledger | `still_valid` | behavior refactor; caller parity first |
| CRA-F-C01 P03 contribution | checkpoint validation still has no roster/inventory/loadout/fragment/economy graph predicate | BossOmen is run state and private active Boss battle remains excluded; neither validates persistent asset edges | `still_valid contribution` | correctness; Final merges with P02 seed/progression evidence |
| CRA-F-C02 P03 contribution | HQ still owns a handwritten ten-field rollback list beside the generic non-battle transaction | Boss fields increase the future field-parity risk but do not create a separate HQ cause | `still_valid contribution` | behavior refactor; P04/Final required |

## C02 Detail

`validate_enhancement_recipe_references` checks nonempty costs, positive amounts and referenced material existence, but
does not reject or aggregate repeated `material_id` rows. `validate_enhance_equipment_payload` compares every row to
the same pre-command balance. `handle_enhance_equipment` then calls `consume` row by row on the authoritative
inventory and raises the level only afterward. A recipe `(dust, 2), (dust, 2)` with balance 2 therefore passes both
admission checks, consumes the first row and fails on the second, leaving a failed command partially mutated.

- Original axes remain `1/3/1/2/3/3/unknown`, subtotal 13/18.
- Preserve the current single-row live result, total cost, level/error/result-diff semantics and bound/max gates.
- The implementation Goal may reject duplicate rows at publication or execute one aggregated/staged plan, but must
  first add insufficient-aggregate no-mutation and exact-aggregate success tests.
- Recipe quantity/balance changes remain excluded content work.

## C03 Detail

`handle_equip_item` stages both roots but performs explicit slot removal/replacement/equip and reverse-binding writes.
`handle_unequip_item` mutates roster first and inventory second without one transition type. Starter setup, death
recovery, event fixtures and direct inventory APIs repeat parts of the same association. Current normal-path tests
pass, so this remains ownership/change-amplification risk rather than evidence of a reachable half-commit.

- Original axes remain `3/2/2/3/3/3/unknown`, subtotal 16/18.
- Preserve both V3 fields, base/instance identity, slot/type/bound rules, displaced-result ordering and current public
  API compatibility.
- Target remains one private typed transition authority; serialized-field deletion and public API removal remain
  migration/external-consumer gated.

## C04 Detail

No `EconomyGrantState` or equivalent shared capture/commit authority exists. The public executor still accepts
inventory, fragments, roster, UUID manager and Enkephalin separately. Generic reward, preview, shop, event, combat
and admin paths independently construct and commit that family. Current production callers stage before invoking the
incremental executor, so the original correction claim remains rejected while the behavior refactor remains valid.

- Original axes remain `3/1/2/3/2/3/unknown`, subtotal 14/18.
- Preserve effect/diff order, seed and UUID continuation, checked arithmetic, contexts and outer session transactions.
- Public `GrantExecutor` signature removal remains consumer gated; a private caller aggregate is policy-neutral.

## F-C01 P03 Contribution

`validate_run_checkpoint_payload` validates power, facility, node references, recruitment ownership and Monolith
state. It does not compose the payload's roster, inventory, equipment slots/reverse bindings, inventory reserve,
material stacks or fragment ownership/loadout/progress into one fail-closed economy predicate. Restore still assigns
these roots directly. Valid round trips therefore prove carriage, not acceptance safety. This confirms the original
P03 half; Final must merge it with P02's seed/progression half under CRA-F-C01.

## F-C02 P03 Contribution

`with_headquarters_transaction` still snapshots and restores game state, active content, node session, run, UUID,
Enkephalin, inventory, roster, fragments and checkpoint with ten explicit fields. `with_state_transaction` already
clones the complete non-battle state and preserves encounter-compendium monotonic union on rollback. Current BossOmen
is nested in `run`, so no eleventh assignment is currently missing, but new root fields can still bypass the HQ list
without a compile failure. P04 must independently reconcile interactive-session lifecycle before Final keeps the
merged CRA-F-C02 identity.

## Gates And New Candidates

- Checkpoint metadata rebinding/repair, removal of either serialized binding, public grant/inventory API removal and
  unknown external Rust/save consumers remain explicit gates.
- Boss runtime reads player equipment enhancements through established setup and keeps private battle state outside
  V3. It adds no persistent economy mutation authority or distinct P03 root cause.
- No distinct `CRR-P03-*` candidate was found.
