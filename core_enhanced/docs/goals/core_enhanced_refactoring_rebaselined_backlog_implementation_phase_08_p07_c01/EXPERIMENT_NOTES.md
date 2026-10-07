# Phase 08 CRA-P07-C01 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-P07-C01, Master Phase 08 of 44.
- Direct predecessor Phase 07 is `Complete / High`; Phase 09 remains pending.
- Mutation authority is limited to final-state tactical Focus/block/Rest reconciliation, its exact tests and nearest
  canonical.
- All three Child documents must be read fully before Phase 08 production/status inspection.

## Authority Notes

- Logical activity, stance, final tile and combat profile must be complete before tile effects are reconciled.
- A snapshot may describe the intended aggregate but must not become a second mutation authority.
- Rest epoch/deadline is runtime state: same-effect transitions preserve valid cadence while effect exit invalidates
  stale events and effect entry schedules one full interval.

## Policy Notes

- This is an existing-contract correction and needs no tactical balance decision.
- CRA-POL-006 forbids changing RON, checkpoint, DTO/wire/public Rust or external consumer shape incidentally.
- Adding a live Any weapon, changing worker eligibility or redefining Ground/Platform effects requires separate
  policy and is out of scope.

## Dirty Worktree Risk

The tree contains extensive user-owned tactical, field-work, movement, world-test, data and canonical changes.
Fingerprint every prospective target before editing. Do not reset, restore, clean, stash or broadly format; stop if
the required hunk cannot be separated.

## Testing Notes

- Ground-only tests are insufficient. Cross-kind Any-profile cases must observe the complete final aggregate.
- Cover both relocation and swap, including a worker leaving assignment and the same worker remaining assigned.
- Assert unrelated incoming modifier identity/value, Focus multiplicity, exact block, Rest epoch/deadline and stale
  Rest no-op, not only the projected tile label.
- Record runner counts; zero-test binaries are not proof.

## Out-Of-Scope Follow-Up Handling

Keep tactical world failure atomicity under Phase 09 CRA-P07-C02, movement backend consolidation under P07-C03 and
movement tick cleanup under P07-C04. Record independent findings without absorbing them.
