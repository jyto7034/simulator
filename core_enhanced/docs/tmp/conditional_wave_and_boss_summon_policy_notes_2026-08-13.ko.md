# 조건부 Wave·보스 소환 정책 논의 임시 기록 — 2026-08-13

- 상태: 식별된 gameplay 정책 결정 완료 / 구현 미착수
- 문서 등급: temporary design notes / non-canonical
- 대상: 조건부 적 증원 Wave, 보스 소환물, 공용 Spawn Kernel, 전투 승패 정산
- 구현 범위 후보: Core, live RON, publication validation, 필요한 server projection
- 제외: Unity 검사·수정, 아군 소환, 실제 production 콘텐츠 설계

## 문서 책임

이 문서는 콘텐츠 표현력 보강을 위해 지금까지 논의한 정책을 임시로 보존한다. 확정 Goal이나 정본
gameplay/runtime/data 계약이 아니며, 이 문서만으로 구현이 승인되거나 완료됐다고 간주하지 않는다.

- 현재 정본 gameplay 규칙은 [`archive/game_rulebook_legacy_2026-10-04.md`](../archive/game_rulebook_legacy_2026-10-04.md)를 우선한다.
- 현재 runtime 불변식은 [`core_runtime_contract.md`](../core_runtime_contract.md)를 우선한다.
- 현재 live RON과 publication validation은 [`data_loading_contract.md`](../data_loading_contract.md)를
  우선한다.
- 현재 구현 준비도 배경은
  [`core_content_readiness_assessment_2026-08-13.ko.md`](core_content_readiness_assessment_2026-08-13.ko.md)를
  참고한다.
- Unity 개발 트리는 현재 Core와 차이가 크므로 이번 조사·구현 범위에서 검사하지 않는다.
- 현재 식별된 gameplay 결정은 이 문서에 반영됐다. Preflight에서 새로운 gameplay 선택이 발견되는
  경우에만 후속 질답을 재개하며, 그 전까지 구현자가 의미를 임의 확장하지 않는다.

## 논의 배경과 목표

현재 콘텐츠 제작을 시작할 수 있는 기본 Core는 존재하지만, 다음 두 표현력 공백은 일반 콘텐츠와 보스
콘텐츠 양쪽에 필요하다.

1. 전투 중 시간, HP, 사망, 적 group clear, 지점 도달에 반응하는 조건부 적 증원 Wave
2. 보스가 독립적으로 작성된 전용 유닛을 소환하고 그 결과를 Boss Runtime이 분기할 수 있는 기능

Boss Runtime의 production reference가 하나뿐인 문제는 실제 보스 콘텐츠 제작 단계에서 넓힌다. 새로운
무기 계열, 도구 전용 시스템과 그 밖의 미정 gameplay는 이번 기반 작업과 함께 진행하지 않는다.

이번 작업은 실제 콘텐츠를 정하는 단계가 아니다. schema/runtime/validation을 증명하는 데 필요한 최소
placeholder reference만 허용한다.

## 잠정 마일스톤

아래 순서는 Goal 작성 전의 잠정 구조이며 아직 실행 Goal이 아니다.

### M0 — 정책 기준선과 전투 종료 규칙 교정

- 이 문서에서 닫힌 정책을 gameplay/runtime/data 정본에 반영한다.
- 플레이어 전투 유닛 전멸을 자동 패배로 보는 현재 암묵 동작을 제거한다.
- 시스템 최대 전투 시간과 자동 Draw를 제거한다.
- 명시적 성공·실패 조건과 같은 시각의 정산 순서를 테스트로 고정한다.

### M1 — 공용 Spawn Kernel

- Wave와 보스 소환이 함께 사용하는 원자적·결정론적 spawn 요청과 결과 계약을 만든다.
- request identity, 중복 방지 ledger, provenance, 배치 실패와 event ordering을 고정한다.

### M2 — 조건부 Wave 증원

- typed trigger schema, live RON loading과 publication validation을 추가한다.
- 시간·group clear·HP·사망·지점 도달과 Wave별 fallback 시각 결합을 구현한다.
- dependency cycle, 도달 불가능한 required Wave와 잘못된 참조를 publication에서 거부한다.

### M3 — 독립 소환물 catalog와 lifecycle

- boss/환상체/침식체/직원과 구분되는 소환물 전용 RON catalog를 추가한다.
- 배치, scaling, provenance, 수량 제한과 소환자 제거 시 lifecycle을 구현한다.

### M4 — Boss Runtime 소환 leaf와 batch 조건

- 첫 소비자는 `BossTask::Summon`으로 제한한다.
- 소환 결과 분기와 `SummonBatchCleared` 조건을 Boss Runtime graph에 연결한다.

### M5 — 최소 수직 reference

- 실제 콘텐츠를 설계하지 않고 조건부 Wave와 보스 소환 경로를 각각 증명할 별도 개발용 placeholder
  FinalBoss를 둔다.
- production 콘텐츠의 양적 확장은 별도 콘텐츠 단계로 남긴다.

### M6 — 정합성·회귀 감사

- Core/live RON/server 경계, 결정론, reconnect projection과 publication failure를 재검토한다.
- Unity는 감사 대상에서 제외한다.

## 확정된 정책

### SP-01 — 공용 Spawn Kernel의 적용 범위

- Wave 증원과 보스 소환은 하나의 공용 Spawn Kernel을 사용한다.
- 내부 Kernel은 faction에 종속되지 않게 설계할 수 있지만 첫 production 소비자는 적 Wave와 보스
  소환물뿐이다.
- 아군 유닛 소환은 후속 구현으로 미룬다.
- 한 spawn request는 원자적이다. 요청한 구성원 전부가 생성되거나 아무도 생성되지 않는다.
- 생성 ID, 위치 선택과 event 순서는 같은 입력과 battle state에서 결정론적이어야 한다.
- stable request identity와 실행 ledger를 두어 같은 요청이 중복 실행되지 않게 한다.
- 존재하지 않는 actor, group, point와 definition 참조는 runtime fallback이 아니라 load/publication
  validation에서 거부한다.
- trigger는 별도 명시가 없는 한 한 번만 실행된다.
- trigger 등록 시 이미 조건을 충족했다면 즉시 발동할 수 있다.
- 조건부 spawn 정산은 전투 승리 판정보다 먼저 끝나야 한다.
- arbitrary expression language는 만들지 않고 Core가 지원·검증하는 closed typed trigger만 사용한다.
- Wave와 소환물은 같은 Kernel을 쓰더라도 서로 다른 provenance와 lifecycle을 가진다.

### SP-02 — Wave와 소환물의 배치 차이

- 기존 Wave는 battlefield RON에 작성된 고정 spawn cell 계약을 유지한다.
- Wave setup/publication 시 요청 인원만큼 서로 다른 authored spawn cell이 있어야 한다.
- Wave는 발동 시 authored cell의 기존 점유 여부와 관계없이 고정 위치에 전원 생성한다. 기존 점유자를
  이동·despawn·사망시키지는 않으며 같은 cell 점유를 허용한다.
- Wave에는 runtime 빈자리 탐색, 점유 기반 실패, 대기열 또는 자동 재시도 정책을 추가하지 않는다.
- 소환물은 walkable cell에만 배치하되, 기존 점유자를 허용할지는 BR-03의 필수 RON
  `occupancy_policy`가 결정한다.
- 소환 위치는 기준점에서 가까운 후보를 결정론적으로 선택한다.
- `AllowOccupied`는 기존 점유자를 밀어내거나 제거하지 않고 같은 tile에 함께 생성하는 의미다.
- 하나의 summon request 안에서 새로 생성되는 복수 소환물은 occupancy 정책과 관계없이 서로 다른
  candidate cell을 사용한다. 같은 batch 구성원을 하나의 cell에 중첩 생성하는 기능은 초기 범위에
  포함하지 않는다.
- 전체 batch에 필요한 유효 candidate cell이 부족할 때의 처리는 BR-03의 필수 RON
  `on_insufficient_space` 정책을 따른다.
- 어떤 정책도 부분 생성 또는 기존 점유자 축출을 허용하지 않는다.

### WV-01 — Wave의 승리 필수 여부

- 모든 encounter Wave는 RON에서 `required_for_victory`를 명시해야 한다.
- 생략 default는 없으며 누락은 publication validation에서 거부한다.
- `required_for_victory: true`인 Wave는 아직 spawn되지 않았더라도 승리를 막을 수 있다.
- `required_for_victory: false`인 Wave는 승리를 막지 않는다.
- 보스 소환물에는 `required_for_victory`를 두지 않으며 항상 승리를 막지 않는다.
- 전투가 끝났을 때 남아 있는 소환물은 정리/despawn한다.
- 일반 전멸·보스 처치 mission은 주 objective 달성과 모든 required Wave group clear가 함께 필요하다.
- 주 objective나 named Victory signal이 먼저 충족돼도 required Wave가 남아 있으면 Victory 후보를
  battle-local latch에 보존하고 최종 승리를 보류한다. 모든 required Wave가 clear된 뒤에만 승리를
  확정하며, 보류 중 Defeat outcome이 충족되면 Defeat가 우선한다.
- 시스템은 위 조합을 지원하지만 초기 live RON에서는 주 objective 선충족 뒤 required Wave가 남는
  encounter를 작성하지 않을 계획이다. 이는 authoring 방침이며 runtime 지원을 제거하는 제약은 아니다.
- `SurviveUntil`과 `ProtectUnitUntil` 같은 시간 성공 mission에는 required Wave를 허용하지 않고 모든
  Wave를 non-required로 작성한다.
- 미래에 시간 생존과 전멸을 동시에 요구한다면 기존 조건을 암묵 결합하지 않고 별도 명시 계약을
  추가한다.

### WV-02 — 지원할 조건부 Wave trigger

초기 지원 의미는 다음과 같다. 최종 RON enum/field 모양은 Goal 전 schema 설계에서 정한다.

- 절대 전투 시각: `AtTimeMs`
- 적 group clear: `OnEnemyGroupCleared`
- 적 group clear 또는 deadline 중 먼저 충족: `OnEnemyGroupClearedOrTime`
- actor HP가 threshold 이하에 도달
- actor가 실제 사망
- actor가 authored point 반경에 도달
- actor의 point 도달 또는 deadline 중 먼저 충족

압박을 만드는 일반 Wave는 절대 시각 또는 `group clear or deadline`을 기본 선택으로 삼는다. 순수 group
clear는 이전 Wave를 정리해야만 진행되는 특수 순차 조우에 사용할 수 있지만, 낮은 화력으로 다음 Wave를
무기한 늦추는 것이 의도가 아니라면 deadline을 함께 둔다.

### WV-03 — 시간과 trigger 정산

- Wave 시간은 전투 시작 후의 절대 simulation time을 사용한다.
- 일시정지는 시간을 멈추며 배속은 simulation time 진행에 반영된다.
- 같은 시각에 여러 trigger가 충족되면 큰 authored priority 값부터 처리하고, 동률이면 stable trigger
  ID 오름차순으로 결정론적으로 처리한다.
- primary trigger와 그 Wave의 fallback 시각이 같은 event bucket에서 함께 충족되면 bucket 전체를
  정산한 뒤 primary가 참일 때 primary를 발동 원인으로 기록한다. Primary가 거짓일 때만 fallback time을
  원인으로 기록한다.
- 한 번의 큰 피해로 여러 HP threshold를 통과하면 통과한 threshold를 모두 발동시킨다.
- HP trigger, 피해 결과, 사망, despawn과 그에 따른 조건부 spawn을 같은 authoritative decision
  boundary에서 정산한 뒤 전투 종료를 판정한다.

### WV-04 — 적 group clear의 의미

- group이 성공적으로 spawn된 적이 있고 그 소속원이 모두 active 상태에서 벗어났을 때 cleared다.
- 소속원의 사망과 사망 이외의 despawn은 모두 group clear에는 기여한다.
- despawn은 kill이 아니므로 kill 통계, 처치 보상과 on-kill 효과를 만들지 않는다.
- 비어 있거나 아직 spawn되지 않은 group은 cleared로 보지 않는다.

### WV-05 — actor identity와 사망·지점 도달 trigger

- `OnActorDefeated`는 실제 사망에만 반응하며 despawn에는 반응하지 않는다.
- `OnActorReachedPoint`는 이동을 명령하지 않고 actor의 실제 이동 결과를 관찰한다.
- RON은 setup 시 바인딩되는 authored `actor_slot_id`, point ID와 tile 단위 반경을 명시한다. Core는
  actor slot을 실제 runtime unit identity에 연결하고 HP·사망·위치를 authoritative state/event에서
  관찰한다.
- Actor slot은 authored Wave actor에만 bind한다. 동적으로 생성된 소환물은 actor trigger 대상이 아니며,
  소환물 집단의 제거는 `batch_handle_id`와 `SummonBatchCleared`를 사용한다.
- 개별 actor 구성을 publication/setup에서 확정할 수 없는 generated Wave에는 actor slot trigger를
  작성하지 않고 group trigger만 사용한다.
- trigger 등록 시 actor가 이미 반경 안에 있으면 즉시 충족한다.
- 죽거나 despawn된 actor의 과거 위치로 새 reached-point trigger를 충족시키지 않는다.
- Actor-bound primary가 HP threshold, 실제 사망 또는 point 도달 전에 despawn되는 등 합법적 runtime
  경로로 영구히 불가능해져도 fallback 시각이 있으면 그 경로를 유지한다.
- Fallback 시각이 없는 non-required Wave는 primary가 영구히 불가능해진 시점에 취소한다.
- required Wave는 순수 reached-point trigger를 사용할 수 없다. 반드시 deadline을 가진
  `reached point or time` 의미로 작성하고, actor가 제거되어도 deadline 경로는 유지한다.

### WV-06 — Trigger dependency와 deadlock 방지

- actor/group trigger의 참조 대상은 의존 Wave보다 먼저 spawn될 수 있어야 한다.
- 서로의 spawn을 기다리는 Wave cycle과 도달 불가능한 dependency는 publication validation에서
  거부한다.
- required conditional Wave는 정적으로 도달 가능한 경로를 가지거나 필요한 deadline을 가져야 한다.
- runtime은 required Wave가 영원히 spawn되지 않아 승리가 막히는 deadlock을 허용하지 않는다.
- 취소된 Wave만을 유일한 source로 기다리는 deadline 없는 non-required 후속 Wave는 dependency graph를
  따라 stable 순서로 연쇄 취소한다. Required 후속 Wave는 이런 취소 경로를 가질 수 없으며 유효한
  fallback 시각이 없으면 publication에서 거부한다.

### WV-07 — Wave trigger RON 구조

- 하나의 Wave activation은 하나의 closed typed `primary_trigger`와 선택적 절대 battle time
  `deadline_ms`로 작성한다.
- 여기서 `deadline_ms`는 전투 종료 제한시간이 아니라 해당 Wave의 authored fallback 발동 시각이다.
  구현 schema에서 더 명확한 field spelling을 선택할 수 있으나 의미는 바꾸지 않는다.
- primary trigger와 deadline을 함께 작성하면 둘 중 먼저 충족된 원인으로 Wave를 정확히 한 번
  spawn한다.
- primary가 `AtTimeMs`인 경우 그 자체가 시간 trigger이므로 별도 `deadline_ms`를 허용하지 않는다.
- required Wave의 primary condition이 source actor 제거, 취소 또는 다른 합법적 runtime 경로로 영구히
  충족 불가능해질 수 있다면 `deadline_ms`를 반드시 작성한다.
- non-required Wave는 의도적인 취소를 표현하기 위해 deadline을 생략할 수 있다.
- author-facing arbitrary `AND`/`OR`, 중첩 조건 tree와 generic expression language는 초기 범위에
  포함하지 않는다.
- 지원하지 않는 조합, primary보다 이르거나 무의미한 deadline과 overflow 가능한 시간 값은 publication
  validation에서 거부한다.

### SM-01 — 소환물의 독립 domain

- 여기서 말하는 소환은 보스의 적 소환물이며 아군 소환이 아니다.
- 보스는 다른 보스를 소환하지 않는다.
- 소환물은 환상체, 침식 직원, 직원과 보스 중 어느 것도 아닌 독립적으로 작성된 전투 유닛이다.
- 소환물 전용 RON catalog가 stats, 기본 공격, skill과 표시 정보를 소유한다.
- 생성된 유닛은 `SummonedUnit` provenance로 소환 definition ID, summoner ID/instance와 summon
  sequence/batch를 기록한다.
- 소환물은 영구 도감 등록, 파편, 독립 encounter reward와 반복 보상 source가 아니다.
- 소환물의 damage와 kill은 일반 전투 통계와 on-kill 규칙에는 정상 반영한다.
- 소환물의 damage/kill source는 해당 소환물 자신이다. summoner boss의 damage/kill로 합산하지 않고,
  boss의 on-hit/on-kill 효과도 대신 발동시키지 않는다.
- event log에는 소환 definition과 summoner provenance를 남겨 이후 UI가 필요하면 그룹화할 수 있게 한다.
- 초기 범위에서 소환물이 다시 다른 유닛을 소환하는 재귀 소환은 금지한다.

### SM-02 — 소환물 scaling

- 소환물은 전용 definition의 base stats, 기본 공격과 skill을 사용한다.
- summoner의 HP, stats와 buff를 상속하지 않는다.
- encounter의 적 stat scaling은 한 번만 적용한다.
- phase에 따라 더 강한 소환물이 필요하면 같은 definition에 숨은 배율을 적용하지 않고 별도 summon
  definition ID로 작성한다.

### SM-03 — 소환 수량 제한과 counter

- 모든 summon definition은 양수 `max_alive`와 `max_total`을 반드시 가진다.
- 제한 없는 소환은 허용하지 않는다.
- 요청 수가 alive 제한이나 total 제한을 넘으면 부분 생성 없이 전체 요청을 `LimitReached`로
  실패시킨다.
- 실패한 요청은 `max_total`에 포함하지 않는다.
- 성공적으로 생성된 뒤 죽거나 despawn된 소환물은 `max_total`에 계속 포함한다.
- counter 범위는 `(summoner instance, summon_definition_id)`이며 전투 전체에서 유지한다.
- phase 전환으로 counter를 초기화하지 않는다.
- 서로 다른 summoner instance는 서로 다른 counter를 가진다.
- phase마다 다른 제한이 필요하면 별도 summon definition ID를 사용한다.

### SM-04 — Summoner 제거 시 lifecycle

- 모든 summon authoring은 `on_summoner_removed`를 반드시 명시하며 default는 없다.
- `Persist`: 이미 spawn이 commit된 소환물은 summoner가 사망·철수·despawn돼도 남는다. 단, 승리를
  막지 않으며 전투 종료 시 정리한다.
- `Despawn`: 이미 spawn이 commit된 소환물도 summoner 제거와 동시에 despawn한다.
- 이 despawn은 사망이 아니므로 kill 통계, on-kill과 처치 보상을 발생시키지 않는다.
- 아직 해결되지 않았거나 commit되지 않은 summon request는 두 설정 모두 summoner 제거 시 취소한다.
- 같은 시각이라도 `UnitSpawned` commit이 summoner 제거보다 먼저 확정된 경우만 이미 해결된 소환으로
  본다. 제거가 먼저 정산되면 예정된 소환은 취소한다.

### BR-01 — Boss Runtime의 첫 소환 소비자

- 초기 범위는 `BossTask::Summon` 하나로 제한하며 `SkillEffect::Summon`은 후속으로 미룬다.
- summon task는 공용 Spawn Kernel을 원자적으로 호출한다.
- Summon task 하나는 summon definition ID 하나와 양수 `count` 하나만 소유한다. 여러 definition을
  소환하려면 여러 task를 작성하며 여러 task를 하나의 원자적 혼합 batch로 묶는 기능은 후속 범위다.
- task 결과는 최소 `Spawned`, `LimitReached`, `NoSpawnSpace`, `Canceled`를 구분하고 Boss Runtime
  graph가 결과별로 분기할 수 있어야 한다.
- 소환 예고와 windup은 기존 `Wait` state/task 조합으로 작성한다.

### BR-02 — Summon batch

- 성공한 summon request마다 전투의 결정론적 UUID continuation에서 고유 `batch_uuid`를 만들어
  `summon_batch` identity로 사용한다. 실패하거나 취소된 request는 batch UUID를 만들지 않는다.
- batch는 실제 생성된 unit identity를 기록한다.
- batch의 모든 구성원이 사망하거나 despawn되면 `SummonBatchCleared`를 정확히 한 번 발생시킨다.
- 실패한 summon request는 batch를 만들지 않으며 cleared로 간주하지 않는다.
- 반복 소환의 각 batch는 서로 독립적이다.
- RON은 runtime UUID를 직접 참조하지 않고 pattern 실행 범위의 symbolic `batch_handle_id`를 선언한다.
  성공한 Summon task가 handle을 해당 실행의 `batch_uuid`에 bind하며 같은 pattern을 다시 실행하면 새
  scope에서 같은 handle 이름을 새 UUID에 bind한다. 전역 문자열 검색이나 암묵적인 "가장 최근 batch"
  선택은 없다.
- `SummonBatchCleared`, lifetime과 removal signal은 handle이 bind한 정확한 batch만 관찰하거나
  제거한다. 같은 task의 이후 실행, 다른 summoner와 다른 batch에는 영향을 주지 않는다.
- `SummonBatchCleared`는 Boss Runtime transition 조건으로만 사용하며 전투 승리 조건에는 관여하지
  않는다.

### BR-03 — 소환 위치 authoring

- 초기 `BossTask::Summon`은 `AroundSummoner`와 `SpawnZone` 두 배치 방식을 모두 지원한다.
- 위치 방식과 그 parameter는 summon unit definition이 아니라 소환을 실행하는 BossTask RON이
  선택한다. 같은 종류의 소환물을 pattern마다 다른 위치에 배치할 수 있어야 하기 때문이다.
- `AroundSummoner`는 양수 `radius_tiles`를 반드시 작성하고 현재 summoner 위치를 기준으로 그 반경
  안의 walkable cell만 탐색한다. 전장 전체를 암묵적으로 탐색하지 않는다.
- `AroundSummoner` 후보는 summoner의 현재 연속좌표와 tile 중심 사이의 제곱 유클리드 거리 오름차순,
  동률이면 tile `y`, 그다음 `x` 오름차순으로 선택한다. 장애물 너머라도 반경 안의 walkable cell이면
  후보이며 path reachability를 추가 조건으로 삼지 않는다.
- `SpawnZone`은 battlefield에 존재하는 authored zone ID를 참조하고 그 zone 안의
  walkable cell을 RON의 authored cell 순서대로 선택한다. Zone 안의 중복 cell은 publication에서
  거부한다.
- 존재하지 않는 zone, spawn에 사용할 수 없는 zone과 잘못된 parameter는 publication validation에서
  거부한다.
- 두 방식 모두 RON에 `occupancy_policy`를 반드시 명시하며 생략 default는 없다.
- `occupancy_policy: RequireUnoccupied`는 기존 active actor가 점유한 candidate cell을 제외한다.
- `occupancy_policy: AllowOccupied`는 기존 active actor가 점유한 candidate cell도 허용한다. summoner,
  아군, 적과 보호 대상 등 기존 점유자의 faction/classification으로 암묵 제한하지 않는다.
- `AllowOccupied`로 생성해도 기존 점유자를 이동·despawn·사망시키지 않는다. spawn 이후의 collision,
  separation과 movement는 기존 전투 runtime 규칙을 따른다.
- `RequireUnoccupied`에서 active actor는 현재 연속좌표 중심을 Core의 기존 `project_to_tile()` 규칙으로
  투영한 logical tile 하나만 점유한다. Body 반경이 다른 tile에 걸쳐도 추가 점유로 보지 않는다.
  Player, opponent, summoner, summon과 보호 대상에 같은 규칙을 적용하고 dead/withdrawn/despawn actor는
  점유하지 않는다. Static defense object는 authored logical tile 하나를 점유한다.
- 두 방식 모두 RON에 `on_insufficient_space`를 반드시 명시하며 생략 default는 없다.
- `on_insufficient_space: Fail`은 전체 batch를 생성하지 않고 task 결과 `NoSpawnSpace`를 즉시 반환한다.
- `on_insufficient_space: WaitUntilAvailable(timeout_ms: ...)`는 spawn을 commit하지 않은 채 task를
  대기시킨다. timeout은 양수 battle simulation time으로 반드시 명시하며 일시정지 중에는 흐르지 않는다.
- 대기 중 전체 batch를 놓을 공간이 생기면 그 decision boundary에서 수량 제한을 다시 확인한 뒤 전원을
  원자적으로 생성한다.
- timeout까지 공간이 생기지 않으면 아무도 생성하지 않고 `NoSpawnSpace`를 반환한다. 대기 중
  `max_alive`/`max_total` 제한에 걸리면 `LimitReached`를 반환한다.
- timeout 시각과 공간 확보가 같은 simulation timestamp에 발생하면 같은 event bucket 전체를 먼저
  정산한다. 그 뒤 batch 전체 공간이 있으면 spawn을 성공시키고, 여전히 부족할 때만 `NoSpawnSpace`를
  반환한다.
- 대기 중 summoner 제거 또는 owning State exit가 먼저 확정되면 요청을 `Canceled`로 끝내며 batch와
  total counter를 만들지 않는다.
- `WaitUntilAvailable`은 RON에 명시된 bounded 동작이며 Kernel의 숨은 무기한 재시도가 아니다.
- `AllowOccupied`에서는 기존 점유 상태가 공간 부족 원인이 되지 않는다. 다만 전체 batch에 필요한 서로
  다른 walkable candidate cell 자체가 부족하면 `on_insufficient_space`를 적용한다.
- 현재 전장은 runtime 중 walkability나 authored zone cell 수가 늘어나지 않으므로
  `AllowOccupied + WaitUntilAvailable` 조합은 publication validation에서 거부한다. 이 조합의 공간
  부족은 기다려도 해소되지 않기 때문이다.
- 두 방식 모두 SP-02의 고유 cell과 전원 생성 원자성을 동일하게 따른다. `PartialSpawn`, 같은 batch의
  동일 cell 중첩, 기존 점유자의 강제 이동·despawn과 무제한 대기는 지원하지 않는다.
- 한 위치 방식이 실패했다고 다른 방식으로 자동 fallback하지 않는다. fallback이 필요하면 Boss Runtime
  graph에 별도 task/transition으로 명시한다.

### BR-04 — 소환물 행동은 기존 Actor Behavior Runtime 재사용

- 소환물 전용 AI scheduler나 `HuntPlayers`·`JoinRoute`·`HoldPosition` 같은 중복 behavior enum을 새로
  만들지 않는다.
- summon unit definition은 stats, 기본 공격, skill과 함께 사용할 기존 Boss Actor Behavior RON을
  참조할 수 있다.
- 생성된 각 소환물은 독립된 `BossActorBehaviorRuntime` instance를 가지며 기존 `RouteCombat`,
  `AcquireTarget`, `MoveToTarget`, `Wait`, target query와 movement primitive를 조합해 행동한다.
- route 기반 행동을 사용하는 소환에는 유효한 authored route context를 명시적으로 제공해야 한다.
  route context가 없는 소환 행동은 target 추적 등 route가 필요 없는 기존 task로 구성한다.
- 원래 boss/encounter가 `BossEncounterRuntime`을 계속 소유한다. 소환물에는 별도
  `BossEncounterRuntime`, encounter phase, Wave scheduling 또는 승패 판정 권한을 만들지 않는다.
- Boss Actor Behavior Runtime을 사용한다는 사실은 소환물을 boss classification, boss kill target,
  도감 대상이나 독립 reward source로 승격시키지 않는다.
- 초기 범위에서 summon unit이 참조하는 behavior graph에는 `Summon` task를 허용하지 않는다. 재귀
  소환 시도는 publication validation에서 거부한다.
- 소환물이 사망하거나 despawn되면 그 actor behavior runtime과 미해결 actor-owned task를 함께
  종료한다. 이미 release된 detached delivery는 기존 전투 lifecycle을 따른다.
- 구현 작업은 공용 Spawn Kernel이 생성한 소환물과 기존 Actor Behavior Runtime 사이의 typed setup 및
  lifecycle adapter를 추가하는 데 집중한다.

### BR-05 — 선택적 lifetime과 signal 기반 batch 제거

- Core의 death와 despawn lifecycle은 구분한다. RON author는 lifetime과 signal마다 제거 mode를
  `Despawn` 또는 `Kill`로 반드시 명시한다.
- `BossTask::Summon`은 선택적인 lifetime removal을 가질 수 있다. 작성한다면 양수 `lifetime_ms`와
  `mode`를 함께 명시하며, 생략하면 lifetime으로 제거되지 않는다.
- lifetime은 battle simulation time 기준이며 일시정지 중에는 흐르지 않는다.
- `BossTask::Summon`은 선택적인 removal signal을 가질 수 있다. 작성한다면 검증 가능한 `signal_id`와
  `mode`를 함께 명시한다.
- 검증된 Boss Runtime action 또는 encounter mechanic이 해당 signal을 발생시키면 그 summon task가
  pattern-local `batch_handle_id`에 bind된 대상 batch의 생존 구성원을 선택된 mode로 일괄 제거한다.
- signal 제거 대상은 전역 소환물, 동일 definition 전체 또는 다른 summoner의 소환물이 아니라 명시적으로
  연결된 summon batch다.
- `Despawn`은 사망 event, on-death, kill 통계, 처치 보상과 공격자 on-kill을 만들지 않는다.
- `Kill`은 정상 death lifecycle과 소환물 자신의 on-death를 실행하고 actor-defeated 조건에 반영한다.
  lifetime/signal이라는 기믹성 제거에는 공격자/killer가 없으므로 kill 통계와 공격자 on-kill은 만들지
  않는다.
- 소환물 domain은 어느 mode에서도 독립 encounter 처치 보상, 도감 기록과 fragment/reward source를
  만들지 않는다.
- lifetime과 removal signal을 함께 사용할 수 있다. 먼저 충족된 원인과 그 mode로 batch를 제거하며
  뒤늦은 만료나 같은 signal의 중복 수신은 안전한 no-op이다.
- Terminal outcome이 관찰하는 neutral signal은 한 번 발생하면 battle 동안 latch한다. Required Wave
  gate가 나중에 열려도 그 Victory 후보를 보존한다.
- Batch removal signal은 batch가 생성되어 handle에 bind된 뒤 발생한 signal만 구독한다. Batch보다
  먼저 발생한 과거 signal은 새 batch를 소급 제거하지 않으며, 같은 signal의 이후 발생은 그때 살아
  있고 해당 signal에 연결된 기존 batch만 제거한다.
- 같은 시각의 batch 생성과 removal signal은 authoritative event total order에서 먼저 commit된 사실을
  기준으로 판단한다.
- 두 mode 모두 제거된 구성원을 `max_alive`에서 즉시 제외하지만 성공한 소환의 `max_total` 사용량은
  복구하지 않는다.
- death 또는 despawn된 구성원은 모두 `SummonBatchCleared` 판정에 정상 반영한다.
- 존재하지 않는 signal, 중복 signal ID, producer가 없는 signal, 유효하지 않은 mode와 batch에 연결되지
  않은 consumer는 publication validation에서 거부한다.

### BL-01 — 플레이어 전멸은 자동 패배가 아님

- active player unit이 하나도 없다는 사실만으로 전투를 패배 처리하지 않는다.
- deploy 가능한 직원이 없더라도 authored 실패 조건이 충족되지 않았다면 전투는 계속된다.
- 전투 실패는 보호 대상 파괴, 제한시간 만료와 특수 조건처럼 encounter에 명시된 조건만 만든다.
- 전투 뒤의 run failure 판정은 별도 lifecycle이며 이 규칙과 혼합하지 않는다.

### BL-02 — 같은 시각의 성공·실패 우선순위

같은 simulation timestamp에 성공과 실패가 함께 충족되면 다음 순서로 정산한다.

1. 같은 시각 event bucket 전체를 처리한다.
2. HP trigger, 피해, 사망, despawn과 조건부 spawn을 안정 상태까지 정산한다.
3. 명시적 실패 조건을 먼저 평가한다.
4. 하나라도 실패 조건이 충족되면 패배다.
5. 실패가 없을 때만 성공 조건을 평가한다.

자동 `Draw` 결과는 두지 않는다. 미래 콘텐츠가 무승부를 요구하면 별도 명시 계약으로 추가한다.

- 여러 Defeat 또는 여러 Victory outcome이 같은 bucket에서 충족되면 큰 authored priority 값부터,
  동률이면 stable outcome ID 오름차순으로 정확히 하나를 선택한다.
- 시스템 최대 전투 시간과 그 상한에서의 암묵 Draw/Defeat는 제거한다. 전투는 authored terminal
  outcome이 충족되거나 플레이어가 Retreat할 때까지 계속된다.

### BL-03 — 제한시간의 공통 시간 기준

- 이 절의 제한시간은 encounter가 명시한 terminal outcome 시간이며 시스템 최대 전투 시간이 아니다.
- Authored 제한시간은 절대 battle simulation time 기준이다.
- 일시정지 중에는 흐르지 않는다.
- 제한 시각과 성공 조건이 같은 event bucket에서 충족되면 BL-02에 따라 제한시간 실패가 우선한다.

### BL-04 — Named terminal outcome과 권한 분리

전투 사건, 승패 판정과 노드 후속 처리를 세 계층으로 분리한다.

`Boss Runtime / encounter mechanic`:

- Core가 구현·검증하는 typed action으로 중립적인 named signal을 발생시킨다.
- signal producer는 그 signal이 승리인지 패배인지, 어떤 보상을 주는지 결정하지 않는다.
- 자유로운 클라이언트 문자열 명령, Unity authority와 임의 script signal은 허용하지 않는다.
- Boss Runtime의 내부 pattern outcome이나 phase transition도 명시적 signal producer가 없으면 전투
  결과가 아니다.

`Encounter RON`:

- 하나 이상의 named `terminal_outcome`을 선언한다.
- 각 outcome은 stable `outcome_id`, `Victory | Defeat` verdict, closed typed condition과 authored
  priority를 가진다.
- 공통 condition에는 보호 대상 파괴, 제한시간 만료, actor 사망, required Wave clear와 시간 생존이
  포함될 수 있다.
- 정화·봉인·의식·특수 gauge처럼 조우 고유 condition은 중립 signal ID를 관찰한다.
- BattleCore의 terminal result는 단순 winner뿐 아니라 정확히 하나의 committed `outcome_id`를 가진다.
- 같은 event bucket에서 Defeat와 Victory outcome이 함께 충족되면 BL-02에 따라 Defeat를 선택한다.
  같은 verdict의 여러 outcome이 충족되면 authored priority, stable outcome ID 순으로 하나를 선택한다.
- signal ID와 producer, condition consumer가 publication 시 연결되어야 한다. missing/duplicate declaration,
  producer가 없어 도달 불가능한 terminal outcome과 잘못된 참조는 validation에서 거부한다.
- 모든 encounter는 정적으로 도달 가능한 Victory outcome을 최소 하나 가져야 한다. Defeat outcome은
  encounter 설계에 따라 생략할 수 있으며, 자동 패배나 시스템 timeout을 보완하기 위해 암묵 삽입하지
  않는다.

`Node RON`:

- 해당 encounter에서 도달 가능한 모든 `outcome_id`에 정확히 하나의 `outcome_policy`를 명시한다.
- Victory outcome policy는 전투 보상의 유일한 authority다. 결과별 reward ID와 bonus objective 허용
  목록, 아래 progression effect를 소유한다.
- outcome별 차등 authored reward는 Victory에만 적용한다.
- Progression은 하나의 `apply_progression` boolean으로 묶지 않고 최소 다음 항목을 closed typed policy로
  각각 명시한다.
  - 환상체 승리 횟수와 Elite 숙달 반영
  - fragment 또는 repeat-reward source 반영
  - assignment pool/cycle 해결 처리
  - BossOmen chain 완료와 Exit Gate 공개
- BossOmen capstone의 모든 Victory outcome은 active chain 완료와 authored Exit Gate 공개를 반드시
  수행한다. 누락 policy는 publication에서 거부하며 격파 횟수, 파편과 기타 보상은 outcome별로 다르게
  작성할 수 있다.
- Bonus objective는 stable objective ID를 가진다. 각 Victory outcome policy는 적용할 objective ID
  목록을 명시하며 Defeat outcome에는 bonus objective를 연결할 수 없다. 기존 encounter를 전환할 때는
  기존 Victory outcome에 현재 bonus objective를 모두 연결한다.
- Victory verdict는 기존 node completion 흐름으로, Defeat verdict는 기존 battle failure/retry/run
  failure 흐름으로 전달한다. Node RON이 encounter verdict를 반대로 재해석하지 않는다.
- Defeat outcome에는 authored positive reward, bonus reward, fragment, encounter progression과 victory
  count를 연결하지 않는다. 살아남은 직원의 생존 XP만 game rulebook의 공통 CombatResult 규칙으로
  지급한다.
- Defeat result commit은 일반 Normal/Elite/NormalBoss의 node attempt 한 회를 소비한다. 모든 attempt를
  소진하면 보상 없이 Completed transit node가 되며 Monolith, FinalBoss와 Event 전투의 예외는
  game rulebook을 그대로 따른다. outcome policy가 이 attempt/run-failure 규칙을 바꾸지 않는다.
- Defeat outcome도 결과 ID를 보존하고 명시적인 빈 policy를 작성하여 log/UI와 validation에서 유형을
  구분할 수 있다. 누락 outcome, 중복 policy와 다른 encounter의 outcome 참조는 publication validation에서
  거부한다.
- 기존 `PveEncounter.reward_uuids`와 평면 `authored_combat.reward_ids`는 모두 제거하고 기존 Victory
  보상을 명시적인 outcome policy로 전환한다. Compatibility fallback이나 두 경로의 동시 적용 없이
  이중 지급 권한을 없앤다.
- encounter 고유 fragment/repeat reward source도 단순 `encounter_id`가 아니라 committed outcome을
  구분하고 node outcome policy가 명시적으로 참조해야 한다.
- 실제 `UnitDied`, kill attribution, on-death와 on-kill은 outcome progression과 별개의 battle event다.
  비사망 Victory가 death event를 합성하지 않는다.
- terminal outcome과 그 reward/progression commit은 authoritative result consumption에서 원자적이고
  idempotent해야 하며 reconnect/resync가 outcome을 다시 선택하거나 같은 result를 재지급하지 않는다.

### BL-05 — 단일 영구 도감 등록

- 영구 도감은 `discovered`와 `defeated`를 구분하지 않고 canonical ID별 단일 registered set만 가진다.
- 실제 전장에 spawn된 opponent `Abnormality`와 `CorrodedEmployee`만 등록 후보다. Preview, briefing,
  authored Wave reference와 아직 spawn되지 않은 적은 후보가 아니다.
- 해당 combat node를 최종 Victory로 완료하면 그 attempt에서 실제 spawn된 모든 후보를 등록한다. 실제
  사망 여부와 primary/non-primary 구분은 도감 등록에 영향을 주지 않는다.
- Defeat outcome, retryable/final Defeat와 Retreat는 도감을 변경하지 않는다.
- 도감 등록은 outcome policy가 선택하는 `DefeatedPrimary | DiscoverOnly | None` credit이 아니며 모든
  Victory에 적용하는 공통 규칙이다. Outcome별 compendium credit schema는 만들지 않는다.
- 기존 개발 중 PlayerProfile의 `discovered`/`defeated` 구조와 데이터는 폐기한다. Migration이나 합집합
  승계는 제공하지 않고 새 단일 도감 schema의 빈 profile로 다시 생성한다. 이는 정식 배포 전 개발
  기준선 교체이며 compatibility 경로를 유지하지 않는다.

### PS-01 — Active battle 상태 보존

- trigger one-shot ledger, summon request ledger, summon counter, batch membership와 cleared 여부는
  authoritative active-battle state의 일부다.
- `batch_handle_id -> batch_uuid` binding, 발생한 neutral signal과 terminal signal latch, required-Wave에
  의해 보류된 Victory 후보와 최종 committed `outcome_id`도 authoritative state의 일부다.
- reconnect/resync나 동일 active battle 상태 복원에서 재발동·재추첨·counter 초기화가 일어나면 안 된다.
- 이 정책은 active battle을 durable SavePoint run payload로 새로 저장한다는 뜻이 아니다.

### RF-01 — 최소 placeholder 최종보스 reference

- 초기 수직 검증용으로 기존 WhiteNight bootstrap과 분리된 개발용 placeholder FinalBoss 하나와 이를
  load/setup할 최소 encounter 연결을 live RON에 둔다.
- 이 reference의 목적은 조건부 Wave, 보스 소환, batch lifecycle, neutral signal, named terminal outcome과
  node outcome policy가 실제 publication부터 전투 종료·결과 소비까지 연결됨을 증명하는 것이다.
- 실제 세계관 설정, 완성된 패턴, 난이도, 밸런스와 production 보상 설계를 확정하지 않는다.
- 기존 validation이 요구하는 필수 field와 연결 값만 최소로 작성하고 placeholder임을 ID와 문서에서
  명시한다.
- 하나의 live encounter에 모든 trigger·배치 위치·실패 조합을 억지로 넣지 않는다. placeholder는 대표
  수직 경로를 증명하고, 나머지 지원 조합과 실패 경계는 focused automated test fixture가 검증한다.
- placeholder는 직접 처치와 비사망 특수 승리 또는 서로 다른 실패 outcome 중 대표 경로를 사용해
  outcome별 node policy가 다른 결과를 낼 수 있음을 증명한다. Death·kill, 단일 도감 등록과 reward는
  이 문서의 공통 정책 및 실제 battle event대로만 반영한다.
- Placeholder는 일반 BossOmen pool/cycle, fragment/repeat-reward source와 영구 "모든 FinalBoss 격파"
  기록 집계에서 제외한다. 수직 검증 외의 run progression 권한을 갖지 않는다.
- placeholder를 정식 production 콘텐츠로 승격하려면 이후 별도 콘텐츠 authoring과 밸런스 검토를
  거친다.

## 알려진 현재 구현 차이

아래는 논의 중 확인한 구현 기준선이며 Goal 전 current-tree preflight에서 다시 검증해야 한다.

- 일부 기존 `WinCondition` 경로는 active player unit이 없으면 자동 패배시키므로 BL-01과 어긋난다.
- 현재 runtime과 정본 runtime 계약에는 시스템 최대 전투 시간과 상한 Draw가 있어 BL-02와 어긋난다.
- 현재 `PveWaveData.required_for_victory`에는 생략 시 참으로 보는 default가 있어 WV-01과 어긋난다.
- 현재 scenario trigger는 전투 시작·절대 시간 중심이며 WV-02의 조건부 trigger를 모두 지원하지 않는다.
- 현재 Boss Runtime에는 summon task/batch condition이 없다.
- 현재 skill effect에는 summon effect가 없으며 이는 이번 범위에서도 추가하지 않는다.
- 현재 소환물 전용 live RON catalog와 publication validation은 없다.
- 현재 battle terminal result는 winner 중심이며 encounter/node까지 보존되는 stable `outcome_id`가 없다.
- 현재 `PveEncounter.win_condition`은 named 복수 terminal outcome과 결과별 priority를 표현하지 않는다.
- 현재 `PveEncounter.reward_uuids`와 node의 `AuthoredCombatNodePolicy.reward_ids`는 outcome을 구분하지
  않는 서로 다른 평면 보상 권한이다.
- 현재 encounter progression reward source는 `encounter_id`에 연결되어 outcome별 reward source를 구분하지
  않는다.
- 현재 compendium과 PlayerProfile은 `discovered`/`defeated` 두 set과 primary/non-primary별 별도 규칙을
  사용하므로 BL-05의 단일 등록 정책과 어긋난다. 개발 중 구 profile migration은 만들지 않는다.
- 현재 BossOmen의 live bootstrap은 WhiteNight 하나이며 별도 개발용 placeholder의 pool/record 제외
  연결은 아직 없다.

## Goal 문서 분할 방침

정책 구현은 하나의 Master Goal과 M0~M6에 대응하는 일곱 개 Phase Goal로 나눈다.

- Master Goal은 전체 목적, 공통 불변식, phase dependency, 범위 제외와 최종 완료 조건을 소유한다.
- 각 Phase Goal은 직전 phase의 committed 산출물을 입력으로 사용하며 독립 검증과 handoff를 남긴다.
- M0의 전투 종료 규칙 교정 없이 Spawn Kernel이나 조건부 Wave 구현을 시작하지 않는다.
- M1 Spawn Kernel이 완료되기 전에 M2 Wave와 M3 소환 consumer가 각자 spawn 구현을 만들지 않는다.
- M3 catalog/lifecycle과 M4 Boss Runtime adapter가 준비된 뒤 M5 placeholder를 publication한다.
- M6 감사 전에는 Master Goal을 완료 처리하지 않는다.
- 실제 Goal 문서는 이 임시 기록만으로 바로 만들지 않는다. current-tree read-only 검사, 불일치 기록,
  구현 난이도·의존성 검토와 preflight를 먼저 수행한다.

## 남은 정책 결정 상태

- 현재 식별된 gameplay 의미의 어려운 결정은 모두 답변됐다.
- Defeat outcome별 positive reward는 제공하지 않으며 이 항목을 위한 game rulebook 변경도 하지 않는다.
- 기존 compendium profile은 개발 기준선에서 폐기하고 migration하지 않는다.
- 구체적인 Rust type 이름, module 위치, RON field spelling과 test command는 current-tree preflight에서
  증거를 수집한 뒤 Goal 문서에 고정한다.
- preflight에서 새로운 gameplay 선택이 발견되는 경우에만 정책 질답을 재개한다.

## 후속 진행 원칙

- 통상적인 결정론, validation과 실패 원자성 세부는 구현 친화적인 보수적 기본안으로 정리한다.
- gameplay 의미가 달라지는 어려운 정책만 사용자에게 한 번에 하나씩 질문한다.
- 현재 tree를 다시 검사하고 readiness review를 거쳐 Master Goal/Phase Goal을 작성한다.
- 과거 완료 Goal은 수정하지 않는다.
