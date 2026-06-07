# BuffDatabase And Consumable Modifier Plan

## Objective

`docs/buff_database_and_consumable_modifier_goal.md`를 기준으로 전투 상태이상 buff와 consumable modifier의 경계를 명확히 하고, `BuffDatabase`를 `GameDataBase` source of truth로 통합한다.

## Current Plan

1. buff/consumable 호출부와 live RON 로딩 경로를 실제 코드 기준으로 분류한다.
2. `BuffDatabase`에 조회 API를 추가하고 `GameDataBase`/`GameDataBuilder`/`GameDataBaseParts`에 포함한다.
3. BattleCore runtime과 timeline validation이 `GameDataBase.buff_data` 또는 명시 registry를 사용하게 한다.
4. `ActiveConsumableModifier`로 snapshot/result 계약을 갱신한다.
5. 전투 buff와 consumable modifier의 차이, 적용 순서, duration 감소 정책을 문서와 테스트로 고정한다.
6. focused test부터 전체 검증으로 확장한다.

## Initial Findings

- `BattleCore`는 이미 `Arc<GameDataBase>`를 가지고 있어 runtime buff lookup은 `game_data.buff_data`로 옮길 수 있다.
- `TimelineValidator`는 현재 `GameDataBase`를 받지 않으므로, buff validation에는 명시적인 `Arc<BuffDatabase>` 주입이 필요하다.
- `GameDataBuilder::empty()` 기반 테스트가 기존 global buff registry에 암묵 의존한다. 기존 공식 동작을 보존하려면 builder 기본값은 live buff RON을 포함하는 편이 안전하다.
- `ActiveConsumableModifier`는 전투 상태이상 buff가 아니라 직원에게 적용되는 다음 전투/런 modifier다.

## Completion Checklist

- [x] `BuffDatabase`가 `GameDataBase` 구성 흐름에 포함됨.
- [x] live buff lookup source of truth가 `GameDataBase.buff_data` 또는 명시 registry로 정리됨.
- [x] `include_str! + static REGISTRY` live registry 제거.
- [x] skill/effect/timeline validation이 최신 buff source를 사용.
- [x] `ActiveConsumableModifier` 명칭/의미 정리.
- [x] 전투 buff와 consumable modifier 혼동 제거.
- [x] 아이템 modifier 적용 순서와 duration 감소 테스트 고정.
- [x] Unity-facing 계약 문서 갱신.
- [x] 레거시 테스트 ignored 없이 삭제/교체.
- [x] 검증 명령 통과.
- [x] 사용자와 의논하여 정해야 할 추가 정책 없음.
