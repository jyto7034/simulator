# DefenseRoute 전술 지휘 Phase 7 — Experiment Notes

Status: complete

## 초기 구조 판단

- Phase 7은 `DR-14`의 explicit two-unit atomic swap만 소유한다.
- Phase 6 occupied relocation은 계속 reject하며 server/Core가 relocation↔swap을 fallback하지 않는다.
- Cost/cooldown tuning은 Phase 6 `tactical_relocation` policy를 재사용하고 swap-specific data를 만들지 않는다.

## 초기 dirty worktree

- Phase 0~6 verified output과 unrelated user dirty가 함께 존재한다.
- 기존 외부 Unity protocol dirty를 보존하고 Unity runtime/contract/tests를 읽거나 변경하지 않는다.

## 최종 구조 판단

- Swap은 `TacticalSwapUnits` → world serialized handler → `BattleLiveCommand::TacticalSwapUnits`의 독립 route다.
  Relocation occupied rejection이나 fallback 의미를 바꾸지 않는다.
- Pair-aware common occupancy와 shared destination/effect helpers가 single relocation과 batch swap의 장기 seam이다.
- Two-source/two-destination preflight 뒤 Core commit helper에는 recoverable error가 없으며 derived block/target은 두
  final body가 모두 반영된 후 한 번 refresh된다.
- Cost/cooldown은 required `tactical_relocation(5,10000)`만 authority다. Swap-specific RON/DTO/save field는 없다.
- Event version 30, behavior/result/presentation/server action은 one-release strict current cutover다.

## 직접 고정한 rejection/immutability

- same employee, either Attack, Hard CC, pending skill cast, active movement segment, cooldown
- invalid/static tile, pair destination의 제3 Player employee/DefenseObject, insufficient resource
- 어느 실패도 position/facing/effect/cooldown/resource/event/revision을 바꾸지 않음

## 직접 고정한 success

- pair occupancy exclusion, explicit facings, cost once, both cooldowns, event/revision once, Pause frozen time
- same runtime IDs와 per-unit HP/resonance/action cooldown/stance preservation
- Any weapon Ground↔Platform DefenseFocus/Rest batch migration과 fresh full Rest interval
- strict command has no destination/path and released delivery follows Phase 6 tracking contract

현재 Phase 7 범위의 알려진 correctness gap이나 정책 미결정은 없다. Unity runtime/contract/tests와 Phase 8~9
placeholder는 추가하지 않았다.
