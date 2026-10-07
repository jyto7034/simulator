# DefenseRoute 전술 지휘 Phase 8 — Experiment Notes

Status: complete

## 초기 구조 판단

- Phase 8은 `DR-16` selection/focus/capture와 Black Round live migration만 소유한다.
- Existing `SkillTarget`, reach/range, step delivery와 tuning은 유지한다.
- Focus는 active battle/checkpoint state이고 hover/local candidate나 durable run save가 아니다.
- PlayerTile은 Core/fixture까지만 구현하고 live content를 만들지 않는다.

## 초기 dirty worktree

- Phase 0~7 verified output과 unrelated user dirty가 함께 존재한다.
- Existing `game_server/docs/unity_bridge_protocol.md` dirty를 보존했고 Unity runtime/contract/tests는 읽거나
  변경하지 않았다.

## 최종 구조 판단

- Required `SkillCastSelection`이 selection owner이고 `SkillTarget`은 effect target 의미를 계속 소유한다.
- Live RON은 25/25 explicit selection이다. PlayerUnit은 Black Round base/awakened 두 path, PlayerTile은 0이다.
- Core final validator가 missing/type mismatch와 unit/tile eligibility를 한 곳에서 판정한다. World/server는 그
  결과를 재추론하지 않는다.
- `PendingSkillCast`는 identity와 accepted tile을 저장한다. Direct unit invalidation은 no-op/no-retarget이고 area는
  accepted tile 고정이다.
- Tactical focus state는 purpose/caster/skill/restore playback만 갖고 begin/cancel/success가 각각 one projection
  commit이다. Preview와 rejection은 revision/event/state를 소비하지 않는다.
- Checkpoint/reconnect와 capability는 current focus를 그대로 투영한다. Skill catalog version은 3이다.
- Event log는 기존 `ManualCastStart.target`과 version 30을 재사용하고 focus-only semantic event는 없다.

## 직접 고정한 rejection/immutability

- missing/unknown/incompatible selection, player selection + RetargetOnStep
- focus 없는 player activation, focus 중 다른 command payload, missing/wrong target kind
- ally/dead/air-ineligible/useless/out-of-range unit, invalid/out-of-range tile
- caster non-manual/Defense/action lock/Hard CC/Silence/non-full resonance
- 어느 failure/preview도 focus/playback/resonance/cooldown/event/revision을 바꾸지 않음

## 직접 고정한 success/lifecycle

- Self/AutomaticUnit focus-free command와 PlayerUnit/PlayerTile exact payload capture
- unpaused X0_5 focus, Pause-start frozen focus, reconnect preservation, free cancel exact restore
- player activation의 target event + focus clear + playback restore + revision once
- dead selected direct unit no fallback/no refund, moving selected unit TileArea acceptance tile 고정
- strict server request/checkpoint/preview result와 refreshed action capabilities

현재 Phase 8 범위의 알려진 correctness gap이나 정책 미결정은 없다. Unity consumer adoption과 Phase 9 FieldWork는
범위 밖으로 유지했다.

## 초기 구조 판단

- Effect `SkillTarget`과 player selection authority를 합치지 않는다.
- All live skill migration must be exact and required; no default/archetype inference or compatibility alias.
- Tactical focus is authoritative submitted-selection state, not hover/local target state.
- PlayerUnit/PlayerTile capture/no-retarget and Pause playback restoration are transaction invariants.
- Black Round progression only is live PlayerUnit; PlayerTile remains engine/test only.

## dirty/out-of-scope

- Phase 0~7 verified output and unrelated user dirty remain preserved.
- Existing external Unity protocol dirty remains untouched; Unity runtime/contract/tests are excluded.
- Phase 9 FieldWork placeholder/state must not be introduced.

## 다음 안전한 작업

1. Inventory all live skill defs and current `SkillTarget`/activation target resolution.
2. Trace playback/checkpoint/action capability seams for authoritative focus.
3. Identify Black Round base/awakened chain and current retarget execution policies before mutation.
