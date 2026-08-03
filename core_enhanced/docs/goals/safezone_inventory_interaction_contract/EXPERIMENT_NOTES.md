# Safezone Inventory Interaction Contract — Experiment Notes

## 초기 상태 — 2026-07-17

- Goal 경로 충돌 없음: `docs/goals/safezone_inventory_interaction_contract/`는 생성 전 존재하지 않았다.
- Core, `game_resources`, game_server, Unity 모두 기존 사용자 변경이 많은 dirty worktree다.
- 직접 겹치는 것으로 보이는 Core `maintenance.rs`, `inventory.rs`, `equipment_data.rs`,
  `live_ron.rs`, `action_capability.rs`, equipment tests, canonical 문서와 Unity
  `LoadoutDropHandler.cs`, `ItemUseActionHandler.cs`, contract tests, canonical UI 문서도 이미 dirty다.
- production 수정 전 각 대상의 현재 diff와 baseline을 다시 좁혀 확인하고, 관련 없는 변경은 보존한다.
- 기존 삭제/untracked 파일을 정리하거나 stash/reset/restore하지 않는다.

## 구조적 판단

- 자동 장비 조합 제거, 원자적 replacement, 관련 action capability, Unity drag/drop/confirmation은
  Safezone inventory interaction이라는 하나의 producer-consumer 계약 경계로 함께 닫는다.
- Maintenance 전용 operation capability는 같은 감사 finding에 포함됐지만 독립적으로 구현·검증할 수
  있어 후속 후보로 남기며 이번 Goal에 섞지 않는다.

## Source-of-truth 충돌

- 변경 전 canonical 계약은 Safezone item/loadout action을 허용했지만 Core common capability는 이를
  투영하지 않아 Unity가 fail-closed로 차단했다. Core capability를 실제 payload validator와 정렬했다.
- Unity의 two-command replacement는 Core의 하나의 최종 loadout mutation 의미와 충돌했다.
  `EquipItem` 하나를 desired final state command로 확정했다.
- 소비 아이템의 즉시 overwrite/no-refund runtime과 destructive action confirmation UI 원칙 사이의
  공백은 active modifier가 있을 때만 confirmation하는 승인 정책으로 해소했다.

## 정책 질문

- 없음. 사용자가 조합 완전 제거, 원자적 replacement, modifier overwrite 확인, pointer 중심 범위를
  승인했다.

## 외부 시스템과 직접 의존성

- game_server command/result mapping
- Unity Editor/test runner와 실제 live WebSocket 검증 환경

## 범위 밖 후속 후보

- UPA-001의 Maintenance operation capability 완결
- UPA-002 Settings/checkpoint load
- UPA-003 disconnected input blocker
- UPA-004 global in-flight commit gate

## 적용하지 않은 대안과 이유

- 자동 조합 유지 또는 명시적 crafting UI: 사용자가 완전 제거를 승인했다.
- 점유 슬롯 드롭 거절: 사용자가 원자적 교체를 승인했다.
- modifier 무확인 즉시 덮어쓰기: 사용자가 덮어쓰기 확인을 승인했다.
- keyboard/gamepad/touch 대체 조작: 승인된 이번 입력 범위 밖이다.

## 접근하거나 검증하지 못한 소비자

- 현재 workspace의 Core, game_resources, game_server, Unity production/test/canonical 소비자를 확인했다.
- 별도 배포 client나 외부 authoring tool은 존재 여부를 확인하지 못했다. strict schema removal이므로
  old `recipes` field를 쓰는 외부 bundle/tool은 명시적으로 실패하며 compatibility는 승인되지 않았다.

## 생략한 검증과 근거

- keyboard-only, gamepad, touch: 승인된 입력 범위 밖이다.
- Unity explicit-live PlayMode 4건: Event/Maintenance/combat/endless 전용이다. Safezone inventory는
  별도 실제 WebSocket scenario로 검증했다.
- 조합 결과 장비 save migration: save에 recipe/progress가 없고 결과 definition/instance는 일반 장비로
  유지되므로 migration하지 않았다.

## 남은 위험

- 현재 workspace 밖 old equipment authoring consumer가 있다면 `recipes` field 제거에 맞춰 갱신해야 한다.
  접근 가능한 consumer에서는 legacy 0건이다.
- UPA-001의 Maintenance 전용 operation capability 누락은 승인 범위 밖 후속 후보로 남는다.
- global in-flight duplicate command gate는 UPA-004 후속 후보다. 이번 Goal은 multi-command replacement를
  제거했지만 모든 UI command의 중복 입력을 일반화해 막지는 않는다.

## 적용된 장기 구조

- 장비 replacement는 Unity orchestration이 아니라 Core transaction 하나가 소유한다.
- Core는 실제 validator를 capability eligibility에도 재사용하고 실행 시 다시 검증한다.
- atomicity는 live state를 순차 변경하지 않고 cloned inventory/roster를 완성한 뒤 함께 교체한다.
- Unity confirmation은 gameplay overwrite 여부를 계산하지 않고 snapshot의 active modifier 존재와 이름만
  읽으며, 최종 소비/교체 결과는 Core가 다시 검증한다.

## Canonical 갱신

- `docs/game_rulebook.md`
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/ui/screen_spec.md`
- `/mnt/f/unity projects/ark/docs/ui_style_guide.md`

## Dirty worktree 보존

- reset, restore, checkout, clean, stash를 사용하지 않았다.
- 관련 없는 modified/deleted/untracked 파일을 삭제하거나 되돌리지 않았다.
- 기존 대규모 dirty 상태 때문에 전체 저장소 diff를 이번 Goal 변경처럼 표현하지 않고 대상 파일의
  현재 content, focused/broad test와 scoped diff check로 완료 증거를 제한했다.
