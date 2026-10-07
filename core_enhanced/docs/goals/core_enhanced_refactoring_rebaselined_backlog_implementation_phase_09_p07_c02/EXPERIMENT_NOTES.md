# Phase 09 CRA-P07-C02 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-P07-C02, Master Phase 09 of 44.
- Direct predecessor Phase 08 is `Complete / High`; Phase 10 remains pending.
- Mutation authority is limited to five tactical world commands' shared atomic boundary, exact tests and nearest
  canonical.
- All three Child documents must be read fully before Phase 09 production/status inspection.

## Authority Notes

- Validate battle UUID/revision capacity and event cursor before authoritative mutation whenever possible.
- Any operation that remains fallible after first mutation must execute on staged affected state; response DTO is
  part of transaction success, not a post-commit afterthought.
- Each serialized command is its own atomic unit. Do not batch commands or roll back an earlier success.
- Phase 08 owns final tactical effects; this Phase owns world-level failure atomicity around that Core transition.

## Policy Notes

- Failure-no-mutation is an existing canonical contract and needs no gameplay policy choice.
- CRA-POL-006 forbids changing command/result tags, nesting, checkpoint, RON, wire/public Rust or external consumer
  shape incidentally.
- Reprioritizing validation errors, costs, cooldowns, deploy HP/lock or incapacitation requires separate policy.

## Dirty Worktree Risk

The tree contains extensive user-owned world combat handlers/tests, Core tactical/deployment/field-work, behavior,
server and canonical changes. Fingerprint every prospective target before editing. Do not reset, restore, clean,
stash or broadly format; stop if the required hunk cannot be separated.

## Testing Notes

- A returned future-cursor error is insufficient proof unless complete state before/after is equal.
- Cover deploy, withdraw, stance, relocation and swap separately with otherwise-valid command inputs.
- Snapshot Core units/scenario/log/signals/queue, deployment mapping/salt/locks/facing, resource, employee attempt,
  focus, revision and response-visible projection.
- Exercise one real late reconciliation/Core failure seam, not a test-only error unrelated to production ordering.
- Record runner counts; zero-test binaries are not proof.

## Out-Of-Scope Follow-Up Handling

Keep battle-advance staging under its completed P06-C03 owner, manual skill/cast atomicity under later P09/P10
candidates and server actor ownership under Phase 10 boundaries. Record independent findings without absorbing them.

## Final Implementation Notes

- The complete affected authoritative mutation footprint is `ActiveBattleSession`; the five handlers read world
  roster/inventory/policy/game data before staging but do not mutate those external owners.
- `with_staged_tactical_world_mutation` owns revision-capacity preflight, command/Core/world/reconciliation execution,
  one revision commit, cursor DTO, result construction and final visual-novel/action-capability attachment.
- Live state remains the original session until command, reconciliation, revision and cursor DTO succeed. Final
  response attachment temporarily sees the staged session; on its error the original session is restored.
- This preserves historical reason priority: UUID/revision authority and capability gate first, handler-specific
  eligibility next, revision capacity at the former per-handler location, remaining Core validation/mutation in its
  original order, cursor after the staged command.
- The outer transport marks exactly the five result-producing commands as already attached. Read-only deployment
  preview, playback, advance, manual targeting/cast and combat completion continue through their existing paths.

## Final Failure Proof Notes

- `u64::MAX` is required for the cursor tests. `latest + 1` can become valid when the command appends one or more
  events and therefore does not reliably reach the late cursor rejection.
- The late reconciliation fixture must invoke a valid first mutation before the missing attempt entry is observed.
  With two deployed units, one unprocessed dead/missing employee and withdrawal of the other active unit, the
  production seam returns `UnitNotFound` only after Core withdrawal and lock mutation.
- `BattleAdvanceAtomicSnapshot` is reused because it covers every affected owner: Core transaction debug state,
  queue/log/execution, employee attempt, deployment, resource, focus/playback, revision and adjacent world-visible
  runtime state.

## Final Compatibility Notes

- No gameplay policy choice was needed. Failure-no-mutation and one-success publish were already canonical.
- No RON/live data, public type, command/result/event tag, payload nesting, checkpoint/save, server or Unity shape was
  added or changed.
- Complete Core and server compile prove current consumers; static caller inventory proves the boundary did not grow
  to unrelated commands.
