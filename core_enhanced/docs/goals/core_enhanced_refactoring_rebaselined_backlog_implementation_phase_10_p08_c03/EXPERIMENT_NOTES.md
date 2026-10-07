# Phase 10 CRA-P08-C03 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-P08-C03, Master Phase 10 of 44.
- Direct predecessor Phase 09 is `Complete / High`; Phase 11 remains pending.
- Mutation authority is limited to the proven percentage-arithmetic owners, exact tests and nearest canonical.
- All three Child documents must be read fully before Phase 10 production/status inspection.

## Authority Notes

- The declared destination stat domain, not an intermediate machine type, is the final saturation authority.
- Wide signed/unsigned intermediates must preserve the existing truncation-versus-ceil distinction.
- Apply the destination floor/cap once at the owning boundary; do not introduce a second balance pipeline.
- Complete current call-site/range inventory precedes helper extraction or mutation.

## Policy Notes

- This is policy-free valid-input hardening: saturation restores existing declared domains.
- Do not change authored percentages, rounding, stat order, certified-power composition or HP/death semantics.
- CRA-POL-006 forbids incidental RON, public Rust, checkpoint/save, DTO/wire or external consumer changes.
- Any ambiguity in destination caps, lethal current-HP behavior or public numeric widening stops the Phase.

## Dirty Worktree Risk

The workspace has extensive user-owned Core/world/data/server/canonical changes from the active campaign and prior
work. Fingerprint every prospective target before editing. Do not reset, restore, clean, stash or broadly format;
stop if the required hunk cannot be separated.

## Testing Notes

- A debug panic alone is not the whole proof; record the corresponding release result so wrap and saturation are
  distinguished.
- Cover 0 and 1 explicitly, plus relevant type maxima, `u64::MAX` interval and `i32::{MIN,MAX}`-class percentages.
- For percentage-ceil, assert the mathematical result clamped to the destination domain rather than a narrowed cast.
- Exercise high Attack and MaxHealth through the actual 207% draft scale and assert capped final current HP.
- Retain an exact ordinary/live-value golden so hardening cannot silently rebalance current content.
- Record runner counts; zero-test binaries are not proof.

## Out-Of-Scope Follow-Up Handling

Keep trigger occurrence dispatch under Phase 11, runtime MaxHealth HP/death lifecycle under Phase 12 and general
damage/trigger consolidation under later P08 candidates. Record independent findings without absorbing them.

## Current Live Arithmetic Notes

- Exactly seven call sites consume `percent_amount_ceil`; do not duplicate a second ceil helper in the stat pipeline.
- Saturating the helper to `u32::MAX` composes with existing caller-specific `saturating_add/sub` and explicit
  Defense/resonance caps while preserving ordinary results.
- MaxHealth's destination cap belongs to `UnitStats` and is `i32::MAX as u32`; draft assignment currently bypasses
  it. Prefer a narrow destination-owned setter/range operation over exporting the constant to unrelated consumers.
- After draft scale, employee-condition HP uses a wide product. The non-employee ratio does not; high full-health
  values would collapse after its `u32::saturating_mul`. Include this in the 207% exact-current-HP proof.
- Current user edits in `employee.rs` are at deployment eligibility/checkpoint identity regions, not the helper/test
  hunk. The canonical is broadly modified by prior Phases and must receive only a localized additive contract entry.

## Final Implementation Notes

- Signed AttackIntervalMs percentage application is destination-owned by `UnitStats`; it does not widen the public
  modifier or stat types and does not reuse the `i64` flat-delta path for values that cannot fit it.
- Unsigned ceil is one shared `u128` calculation returning saturated `u32`. Existing subtract/add/cap consumers keep
  their own semantic destination rules.
- MaxHealth scale is the only draft field routed through `set_max_health`; this restores the already-existing cap and
  current-HP invariant rather than introducing a new one.
- The final non-employee current-health ratio uses wide multiplication and the original floor/minimum/cap order.
- Debug and release execute the same boundary table; ordinary/live stat and certified-power outputs remain exact.

## Final Validation And Compatibility Notes

- Exact new proof is 3/3 in debug and 3/3 in release; pre-fix was 0/3 in each profile.
- Complete Core library is 958/958 with every live integration binary passing; game_server all-target compile, format
  and diff checks pass.
- No content, public/serde carrier or migration was introduced. Phase 11 can consume the same stat/effect shapes, and
  Phase 12 now receives stable numeric cap/current-HP behavior as required by the conservative dependency edge.
- The only discarded executions selected zero tests due short names plus `--exact`; they are documented and never
  counted. The isolated `/tmp` release target is build output, not a repository or contract artifact.
