# State Authority and Mutation Map

- 기준선: `B-001`
- 상태: 최종

## A-001. Root authority

| gameplay fact | owner | external read/query | mutation entry | visibility/transaction note |
| --- | --- | --- | --- | --- |
| 전체 world/run 상태 | private `GameCore.state: GameCoreState` | typed/JSON snapshot 및 result | `GameCore::execute*` → domain handler | state type/module은 외부 비노출, sibling world modules는 field 직접 mutation |
| static game data | `GameCore.game_data: Arc<GameDataBase>` | handler/snapshot 내부 lookup | constructor에서 주입 후 replacement 없음 | official validated path와 unsafe public alternate path; RUNTIME-AUDIT-007 |
| run seed | `GameCore.run_seed`, `GameCoreState.uuid_manager` | generation/identity 내부 사용 | constructor/checkpoint continue | namespace counters restored; deterministic probes pass; GAP-003 continuation scope |
| player profile | `GameCoreState.player_profile` | clone getter/snapshot | public merge/reset, domain commit | active-run rollback에서 local profile 보존 의도 확인 |
| run | `GameCoreState.run: Option<RunState>` | run snapshot | map/node/event/shop/combat modules | floor/run scoped field가 한 struct에 집합 |
| active battle | `GameCoreState.active_battle: Option<ActiveBattleSession>` | battle result/update/reconnect | combat/battle handlers | `BattleCore`, projection revision, deployment/playback을 session이 함께 소유 |
| enkephalin | `GameCoreState.enkephalin` | getter, run snapshot, eligibility/result | economy grant/shop/HQ/admin 및 public `set_enkephalin` | RUNTIME-AUDIT-001 후보: public test-only bypass |

## A-002. Root command ordering

```text
external Rust caller
  -> GameCore::execute_with_transport_context
  -> validate_battle_command_authority
  -> gate_behavior_action
  -> execute_validated_behavior_domain
  -> map/HQ/equipment/shop/reward/event/VN/battle handler
  -> optional battle/VN transport projection attachment
  -> BehaviorResult | GameError
```

- `PlayerBehavior` match는 현재 variant에 exhaustive하다. Domain 내부의 fallback `unreachable!`는 root match가
  동일 enum을 정확히 라우팅한다는 정적 guard를 가진다.
- preflight가 모든 domain validation을 중앙 소유하지 않고, 각 handler가 payload/transaction validation을
  추가 소유한다. 세부 mutation ordering은 각 vertical path에서 기록한다.

## Public Root Mutation Entries (current inventory)

- `GameCore::new*`
- `GameCore::merge_encounter_compendium_profile`
- `GameCore::merge_player_profile`
- `GameCore::reset_player_profile`
- `GameCore::execute*`
- `GameCore::continue_from_active_run_checkpoint`
- `GameCore::restore_active_run_transaction_rollback`
- `GameCore::set_enkephalin` (commented test helper but production-visible; RUNTIME-AUDIT-001)
- `GameCore::execute_admin_command` with 20 exhaustive typed admin variants

## A-003. Run and battle nested owners

- `RunState`: map/progression/facility/encounter/event/shop/combat-preview/attempt/recruitment/record/recovery를
  소유한다. Several collections are `HashMap`; authority ordering impact is C04 follow-up.
- `ActiveBattleSession`: identity/revision, `BattleCore`, execution, entry rollback snapshot, employee attempt,
  tactical resource, deployment, playback, focus를 소유한다.
- `BattleEntrySnapshot`: battle 진입 이전의 non-battle roots를 clone하며 restore 시 `active_battle = None`으로
  만든다. `player_profile`과 `run_power_authority`는 snapshot field에 없으므로 보존/merge 의미를 C03/C08에서
  확인한다.

## A-004. BattleCore internal authority

- owner: `BattleCore`
- mutation engine: event heap + `run/step/apply_live_command` methods
- nested state: scenario/wave/summon/boss runtime, units/tactical/items/graveyard/buffs/casts/procs/projectiles/areas/
  movement/field-work/block/Rapier/log/counters/cause stacks
- authority exposure: `units`, `battlefield`, `game_data`, `event_log`, sequences, seed and recording stacks are public;
  `RuntimeUnit` fields are broadly public; `enqueue_event` is public production API (RUNTIME-AUDIT-003)
- world integration: `ActiveBattleSession` is private behind `GameCore`, and current non-test world paths observed here read
  these public fields rather than mutate them directly.

## A-005. Projection and retained-history authority

- `ActiveBattleSession.battle_state_revision` is the state-projection revision; the event log's monotonic `seq` is a
  separate ordered history cursor. DTO exposes both and server tracks last delivered event seq.
- `BattleCore.event_log.entries` is the sole retained battle event history, but simultaneously serves live delta,
  result attachment, debug export and tests. It is append-only and unbounded; delta extraction scans all retained history
  (RUNTIME-AUDIT-009).
- visual novel and battle side messages attach after domain handling; inconsistent staging across handler families creates
  RUNTIME-AUDIT-005, not a second authority value.

## Final ownership assessment

- GameCore roots generally have a single concrete owner and typed mutation path.
- exceptions are production-visible fixture/debug mutation surfaces (`set_enkephalin`, public mutable BattleCore/RuntimeUnit
  and `enqueue_event`) and the public validated/unvalidated data construction conflation.
- no independent duplicate currency, run, active-battle, or profile authority was found in the audited production path.
