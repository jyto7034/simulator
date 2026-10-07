# DefenseRoute 전술 지휘 Phase 6 — Experiment Notes

Status: complete

## Source-of-truth와 구조 판단

- 정책 authority는 `DR-04`, `DR-05`, `DR-12`다.
- Position은 `RuntimeUnit.body`, stance/tile/effect/cooldown은 `BattleCore::tactical_states`, 전술 포인트는
  Phase 3 `TacticalResourceState`가 각각 하나의 authority다.
- World는 deployment zone과 employee/loadout context를, Core는 final tile/static/occupancy/affinity/source/action
  validation과 state mutation을 소유한다.
- Player tactical occupancy는 active player `Combatant | DefenseObject` logical projected tile만 포함한다.
  Enemy overlap은 physics/targeting 의미로 남고 player slot을 점유하지 않는다.

## Transaction 판단

- 모든 fallible preflight 뒤 같은 runtime body/tactical effect/cooldown/event를 갱신하고 world가 cost/deployment
  facing/revision을 commit한다. Serialized actor 경로이므로 두 preflight 사이 concurrent mutation은 없다.
- Relocated blocker 관계만 먼저 제거해 다른 blocker의 stable engagement를 보존하고, body mutation 뒤 existing
  deterministic refresh를 사용한다.
- Same-tile facing-only는 정식 paid command지만 tile kind가 같으므로 Rest epoch/cadence를 바꾸지 않는다.
- Ground↔Platform은 Any profile만 허용하며 old focus modifier를 빼고 new effect를 더해 다른 modifier를 보존한다.

## Delivery와 lifecycle 판단

- Relocation은 release 전 windup/cast-focus에만 막히며 이미 released projectile/area를 취소하지 않는다.
- Homing/Follow 계열은 live body를, Fixed/GroundFixed는 stored aim/geometry를 이미 사용하므로 relocation-specific
  retarget/fallback state를 만들지 않는다.
- HP, buffs, resonance, next action/basic cooldown, stance와 stance cooldown은 같은 instance에 남는다.
- Withdraw/redeploy는 Phase 1대로 새 instance이므로 relocation cooldown을 상속하지 않는다.

## DTO/version/migration 판단

- Command는 exact `tactical_relocate_unit { employee_uuid, destination, facing }`이고 path/focus/swap field가 없다.
- Tactical DTO는 relocation deadline과 Core reason+global resource availability의 eligibility를 필수 투영한다.
- Event variant 추가로 battle event-log version은 strict 29다.
- Active battle은 durable active-run save payload가 아니므로 active-run V3와 setup version은 바꾸지 않는다.
- Alias/default/dual schema/compatibility reader는 없다.

## Dirty와 비범위 판단

- Phase 0~5 verified output과 unrelated user docs/content dirty를 보존했다.
- 기존 외부 Unity protocol dirty는 이번 Phase 대상이 아니며 읽거나 수정하지 않았다.
- Phase 7 swap, Phase 8 manual focus/targeting, Phase 9 FieldWork production placeholder를 추가하지 않았다.

## 다음 Phase seam

- Phase 7은 이 Phase의 destination/occupancy와 per-instance relocation cooldown을 재사용하되 occupied relocation의
  fallback이 아니라 별도 two-unit atomic swap command여야 한다.
- Swap은 양쪽 source/effect/position을 batch preflight한 뒤 cost 5를 한 번 지불하고 양쪽 cooldown을 10000ms로
  설정해야 하며 중간 점유 상태를 외부에 노출하면 안 된다.

## 초기 구조 판단

- Phase 6는 relocation, logical occupancy와 Ground/Platform derived effect atomic transition만 소유한다.
- Occupied destination swap은 Phase 7의 명시적 command이며 fallback을 선행하지 않는다.
- Phase 5 tactical state와 Rest epoch는 immutable predecessor input이 아니라 relocation transaction에서
  position과 함께 안전하게 갱신할 authority다.

## Dirty와 비범위

- Phase 0~5 verified output과 unrelated user dirty를 보존한다.
- Unity runtime/contract/tests는 읽거나 변경하지 않는다.
- Phase 7~9 placeholder state/command/content를 추가하지 않는다.

## 다음 안전한 작업

1. Position/spatial/action-busy/zone/occupancy/DTO path를 읽는다.
2. 수정 대상 diff와 focused baseline을 확보한다.
3. Core transaction과 common destination validation 설계를 확정한다.
