# Employee Condition Lifecycle Migration Experiment Notes

## 초기화 기록

- goal_name: `employee_condition_lifecycle_migration`
- path check: `phase3_missing`
- 선행 Phase 1: complete
- 선행 Phase 2: complete
- production baseline: Core 766, game_server 30, Unity EditMode 240, Unity PlayMode 57 pass/4 explicit skip

## 구조적 판단

- Phase 3은 persistent condition, attempt-local condition, death/recovery와 purpose eligibility를 하나의 lifecycle cutover로 소유한다.
- 이 범위를 분리하면 Run HP/life/availability와 새 physical damage/trauma authority가 병행되므로 Child를 더 나누지 않는다.
- Phase 2 recruitment aggregate는 roster definition identity만 참조하며 condition 의미를 소유하지 않는다.

## Source-of-truth 충돌

해소됨. persistent/attempt/checkpoint/DTO/Unity는 physical damage와 trauma condition 하나를 사용한다. runtime battle HP는 전투 내부 상태이며 persistent condition authority가 아니다.

## 정책 질문

현재 시작을 막는 질문은 없다. 실제 Treatment content 수치가 migration 완료에 필수면 임의 결정하지 않는다.

## 외부 시스템과 직접 의존성

- game_resources Run HP/consumable/recovery authoring
- game_server shared Core DTO transport
- Unity roster/loadout/item/support/battle/maintenance readers와 affordance
- Core in-memory checkpoint 및 battle entry snapshot

## Dirty Worktree 주의사항

- employee, battle, world, reward, consumable, behavior/snapshot, tests와 canonical 문서가 이미 광범위하게 modified다.
- Phase 1/2 변경과 다른 사용자 변경을 모두 보존해야 한다.
- external game_resources/game_server/Unity는 현재 workspace 밖이므로 필요한 write는 권한 경계를 지킨다.

## 접근하거나 검증하지 못한 소비자

- 배포된 외부 durable save/database
- workspace 밖 비공개 client

game_server와 접근 가능한 Unity reader/presenter는 검증했다.

## 적용하지 않은 대안과 이유

- 기존 Run HP를 새 physical damage와 병행하지 않는다.
- mutable life_state/availability를 cache나 compatibility field로 남기지 않는다.
- 목적별 boolean helper를 다시 흩어 놓지 않고 중앙 evaluator로 모은다.
- Phase 4 memory/trust migration을 앞당기지 않는다.

## 생략한 검증과 근거

- 초기화 단계 production tests는 실행하지 않았다. 직전 Phase broad baseline을 기록했다.

## 남은 위험

- production physical Treatment 아이템의 실제 amount는 후속 콘텐츠 authoring이다.
- 외부 durable save가 발견되면 no-compatibility 정책과 별도 migration 결정을 다시 확인해야 한다.
- repository 전체 clippy `-D warnings`는 기존 lint debt 119개 때문에 green이 아니다.

## 범위 밖 후속 후보

- Phase 4 Psychology state contract
- 실제 Treatment 콘텐츠 밸런스 authoring
- account memorial/legacy
