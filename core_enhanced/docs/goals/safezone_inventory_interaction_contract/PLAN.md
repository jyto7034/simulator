# Safezone Inventory Interaction Contract

## Status

complete

## 목표와 사용자-visible 문제

Safezone 준비의 소비 아이템 사용과 장비 loadout을 Core 권위의 live 기능으로 완결한다.
현재 장비 장착은 숨은 자동 조합을 수행할 수 있고 Unity 교체는 여러 command로 부분 적용될 수
있으며, 소비 아이템 modifier 덮어쓰기는 비가역 손실 확인 없이 실행된다. 또한 관련 Core
capability가 snapshot에 없으면 Unity의 fail-closed gate가 정상 기능을 막는다.

## 감사 finding ID

- `UPA-001` 중 Safezone item/loadout action capability 경계
- Maintenance 전용 operation capability는 이 Goal의 비범위다.

## 대상 저장소와 경로

- Core: `/mnt/f/work/simulator/core_enhanced`
- Live data: `/mnt/f/work/simulator/game_resources`
- Unity consumer: `/mnt/f/unity projects/ark`
- 필요 시 read-only server mapping: `/mnt/f/work/simulator/game_server`
- Goal 기록: `docs/goals/safezone_inventory_interaction_contract/`

## 현재 runtime 및 live data 근거

- 변경 전 Core `EquipItem`은 target 직원의 모든 장착 장비를 순회해 recipe가 맞으면 두 owned
  instance를 삭제하고 결과 장비를 생성·장착했다. live equipment RON에는 recipe 2개가 있었다.
- Unity `LoadoutDropHandler`는 이동/교체를 선행 `UnEquipItem` 1~2회와 `EquipItem`으로 분해했다.
- Core common capability 목록에는 Safezone inventory ActionKind가 없어 Unity live gate가 막혔다.
- 소비 아이템 runtime은 기존 modifier를 즉시 교체하고 환불하지 않았으나 Unity confirmation은 없었다.
- game_server는 typed Core result를 의미 변경 없이 전달하며 `Combined` 전용 mapping은 없었다.
  Unity도 `Combined` payload를 직접 읽지 않았다.

## Source of truth와 소유 계층

- Core는 장비/소비 아이템 mutation, payload validation, capability, command result의 권위자다.
- `game_resources`는 live equipment authoring data의 권위자다.
- game_server는 typed command/result/snapshot을 의미 변경 없이 전달한다.
- Unity는 capability와 snapshot을 소비하고 drag/drop, 확인 modal, 표시 피드백을 소유한다.
- 입력 지원 범위와 화면 상호작용의 durable 계약은 Unity canonical UI 문서가 소유한다.

## 유지할 불변식

- Unity가 장비 호환성, modifier 결과 또는 gameplay eligibility를 재추론하지 않는다.
- 장비 이동/교체는 성공 시 최종 상태 전체가 한 번에 commit되고 실패 시 아무 상태도 바꾸지 않는다.
- 귀속 장비 및 사용할 수 없는 직원에 대한 기존 Core 검증을 약화하지 않는다.
- 소비 아이템 사용 성공은 stack을 정확히 1 감소시키며 modifier 덮어쓰기의 환불 정책을 바꾸지 않는다.
- 기존에 생성되어 저장된 조합 결과 장비는 일반 소유 장비로 계속 유효하다.
- 기존 dirty worktree의 사용자 변경을 보존한다.

## 범위

- `EquipItem` 자동 장비 조합과 관련 schema/data/DTO/test/documentation의 완전 제거
- `EquipItem`을 장비의 직원 간 이동과 점유 슬롯 교체까지 포함하는 단일 원자적 최종 상태 command로 정렬
- Safezone `EquipItem`, `UnEquipItem`, `EquipSkillFragment`, `UnequipSkillFragment`,
  `UseConsumableItem`의 Core context capability projection과 handler 재검증 정렬
- 활성 modifier가 교체될 때만 Unity 확인 modal을 거쳐 소비 아이템 command 전송
- PC pointer 중심 입력을 이번 기능의 공식 범위로 canonical 문서에 확정
- producer, server 전달, Unity reader/handler/presentation 및 focused/broad tests 정렬

## 명시적 비범위

- Maintenance 전용 분쇄, 강화, 파편 성장 operation capability 수정
- Settings/checkpoint, disconnected overlay, in-flight global command gate
- keyboard-only loadout 조작, gamepad, touch 대체 조작
- 신규 장비 제작/조합 시스템 또는 recipe compatibility
- 장비/소비 아이템 밸런스 및 신규 콘텐츠
- 기존 조합 결과 장비의 삭제나 save migration
- 관련 없는 UI 폴리싱 및 구조 리팩터링

## 승인된 정책과 근거

- 사용자는 장비 조합 기능의 완전 제거를 승인했다.
- 장비 점유 슬롯 드롭과 직원 간 이동은 Core의 단일 원자적 교체로 처리한다.
- 활성 소비 modifier를 덮어쓸 때만 기존 효과 소멸과 환불 불가를 표시하고 확인 후 사용한다.
- 이번 기능의 공식 입력은 PC pointer 중심이다. 키보드는 일반 단축키만, gamepad/touch는 비범위다.
- 승인되지 않은 compatibility, dual-read, fallback은 추가하지 않는다.

## 미결정 정책

없음. 조사 중 승인된 의미만으로 결정할 수 없는 새 사용자-visible 정책이 발견되면
`policy_decision_required`로 전환한다.

## 구현 범위

- Core equipment schema에서 recipe metadata/field/index/validation을 제거하고 old `recipes` field를
  strict RON unknown field로 거부한다.
- Core `EquipItem`은 cloned roster/inventory에 이동·교체를 stage하고 result를 만든 뒤 두 state를
  함께 commit한다.
- Safezone inventory ActionKind를 context와 실제 executable payload 기준 capability로 투영하고 각
  handler validator가 같은 context를 재검증한다.
- Unity replacement/transfer는 `EquipItem` 하나만 보내며 active modifier가 있는 item-use drop은
  typed confirmation state/modal/action을 거친다.
- Core rulebook과 Unity Core/UI canonical 문서를 승인 정책에 맞게 갱신했다.

## 사용자-visible 완료 조건

- 장비를 점유 슬롯 또는 다른 직원에게 드롭하면 성공 시 기존 장비가 가방으로 돌아가고 새 장비가
  한 번에 목표 직원에게 장착되며 중간 해제 상태가 노출되지 않는다.
- 어떤 장비 드롭도 두 장비를 소비하거나 제3의 조합 장비를 생성하지 않는다.
- 장착 해제와 파편 장착/해제 및 소비 아이템 사용이 승인된 Safezone context에서 Core capability로
  노출되고 실제 handler validation과 일치한다.
- 활성 modifier가 없는 소비 아이템 드롭은 즉시 사용하고, 활성 modifier가 있으면 경고 확인 전에는
  command를 보내지 않는다.
- 확인 취소 시 inventory와 modifier가 변하지 않는다. 확인 시 기존 Core의 소비/교체 결과를 따른다.
- pointer drag/drop이 정식 조작으로 동작하며 비지원 입력을 지원한다고 문서나 UI가 주장하지 않는다.

## DTO migration 여부

완료: `EquipItemOutcomeDto::Combined`와 ingredient/result UUID serialized shape를 제거했다.
`Equipped` outcome만 유지한다. game_server generic mapping과 Unity 접근 가능한 소비자에 별도
compatibility는 두지 않았다.

## Data 또는 live RON migration 여부

완료: `EquipmentRecipeMetadata`, `EquipmentDatabase.recipes`, recipe index/lookup/validation과 live
recipe 2개를 제거했다. strict deserialization은 제거된 `recipes` field를 거부한다.

## Save migration 여부

없음. save/checkpoint는 recipe나 조합 진행 상태를 저장하지 않는다. 기존 결과 장비 definition과
owned instance는 일반 장비로 유지된다.

## 필수 focused 검증

- Core equipment focused 24/24 pass: 신규 장착, 점유 슬롯 교체, 직원 간 이동, bound/invalid 및
  active-fragment 불일치 원자적 거부, consumable 동작.
- removed recipe strict rejection 1/1, capability focused 1/1, live RON 20/20 pass.
- legacy symbol/schema/result/live recipe scoped search 0건. canonical 문서에는 부재 정책만 남는다.
- serialized `EquipItem` result는 `Equipped`만 포함하며 live WebSocket result에도 조합 field가 없다.
- Unity item/loadout focused 6/6 pass: single `EquipItem`, employee transfer, modifier 없음 즉시 전송,
  modifier 있음 confirm/cancel/confirm.
- 일반 Unity PlayMode 55 pass / 0 fail / 4 explicit-live skip가 pointer/drag/modal 경계를 포함해 통과했다.

## 필수 broad 검증

- Core `cargo test --all-targets --quiet`: 755 pass / 0 fail / 0 ignored.
- game_server `cargo test --all-targets --quiet`: 30 pass / 0 fail / 0 ignored.
- Unity EditMode: 231 pass / 0 fail / 0 skip.
- Unity PlayMode: 55 pass / 0 fail / 4 explicit-live skip.
- 직접 기동한 game_server WebSocket에서 5개 Safezone capability, equipment replacement/employee move,
  조합 field 부재, consumable overwrite `replaced_modifier`를 확인했다.

## 생략할 검증과 근거

- keyboard-only/gamepad/touch: 승인된 입력 범위 밖이다.
- 신규 조합 결과 콘텐츠 밸런스: 조합 기능을 제거하므로 비범위다.
- 기존 결과 장비의 별도 save migration: recipe/progress가 save되지 않고 definition/owned instance는
  그대로 유효하므로 생략했다.
- 4개 explicit Unity live PlayMode test는 Event/Maintenance/combat/endless 전용이며 이번 완료 조건과
  무관하다. Safezone inventory live 경계는 별도 WebSocket probe로 검증했다.

## 중단 조건

- 관련 dirty 변경과 안전하게 병합할 수 없음
- 외부 소비자가 제거 DTO/schema를 필수로 사용하지만 접근할 수 없음
- 승인되지 않은 save/compatibility 또는 장비 replacement 의미가 필요함
- 필수 baseline 없이 변경 전후 회귀를 구분할 수 없음
- 단일 원자적 replacement가 현재 command identity로 표현 불가능하고 새 wire 정책 결정이 필요함

## Compatibility

없음. 자동 조합, recipe schema, `Combined` 결과에 대한 compatibility를 유지하지 않는다.
