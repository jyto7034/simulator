# Phase 04 Candidate Reaudit

## Current Dispositions

| Candidate | Current evidence | Boss/current impact | Disposition | Work type/readiness |
| --- | --- | --- | --- | --- |
| CRA-P04-C01 | typed trigger validation still checks local fields/references but not repeat/source compatibility; runtime alone rejects invalid pairs | Boss battle triggers use the same runtime table but do not move it into publication | `still_valid` | correctness; structured negatives first |
| CRA-P04-C03 | map Event/Shop and HQ Shop activation still repeat persisted-session insert plus active mirror writes outside domain commit helpers | BossOmen/Monolith add consumers, not a paired-commit owner | `still_valid` | behavior refactor; paired-state characterization |
| CRA-P04-C04 | trigger and Monolith paths still build VN sessions separately and encode Tail versus Front placement through raw mutations | Boss battle trigger sources increase use of the trigger path without centralizing assembly | `still_valid` | behavior refactor; world characterization |
| CRA-F-C02 P04 contribution | HQ still hides active content during completion and uses its private transaction instead of an explicit common completion context | BossOmen is nested run state; it does not remove the bypass or transaction split | `still_valid contribution` | behavior refactor; merges with P03 HQ field-list evidence |

## Candidate Details

### CRA-P04-C01

`validate_trigger` accepts `MapNodeEntered + OncePerBattle`. A matching node reaches
`consumed_key_for_trigger`, which returns `InvalidStaticData` because the event has no battle UUID. The surrounding
node transaction rolls back, but publication has already accepted unusable production data. The original axes
`1/2/1/1/2/2/unknown` and test-first readiness remain. Mirror only runtime-rejected combinations in typed and
official-bundle validation; changing currently accepted battle `OncePerNode` meaning remains policy work.

### CRA-P04-C03

`commit_event_session` requires a run and writes both persisted/active states. `commit_shop_session` writes the run
cache only when a run exists and always writes active state. Initial Event/Shop map entry and HQ-to-Shop still perform
equivalent writes directly. The target remains explicit private Event and Shop paired-commit modes, preserving
session identity/history, checkpoint/re-entry, error ordering and current fixture behavior. Original axes remain
`2/1/2/2/2/2/unknown`.

### CRA-P04-C04

Trigger activation resolves definition/presentation/entry/UUID and appends to the queue; Monolith exchange repeats
that assembly and inserts at index zero. The distinct ordering is intentional. One private factory plus explicit
`Tail`/`FrontOfQueue` placement remains the correct consolidation target. Preserve UUID order, active
non-interruption, blocking and projection shape. Original axes remain `2/1/2/2/2/2/unknown`.

### CRA-F-C02 P04 Contribution

HQ completion still temporarily removes `active_node_content` so the generic planner does not reject the HQ
session, then executes through `with_headquarters_transaction`. P03 independently confirmed the ten-field rollback
list. P04 confirms the interactive-session/context half remains, so Final should retain one merged CRA-F-C02 rather
than split the same private HQ transaction cause.

## Gates And New Candidates

Public VN/session field removal, retry/idempotence meaning, authored source/repeat expansion and DTO/save consumer
rollout remain gated. Boss battle triggers and BossOmen do not create a distinct P04 authority defect. No
`CRR-P04-*` candidate was found.
