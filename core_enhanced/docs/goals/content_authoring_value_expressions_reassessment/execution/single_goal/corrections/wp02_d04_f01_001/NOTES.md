# R06 WP02 D04-F01 교정 메모

## 원인과 수정 경계

target-locked projectile의 release capture는 실제 고정 target에 결속된다. target 소실·inactive·미명중 종료에서는 projectile caller가 정상적으로 빈 impact target set을 넘긴다. D04의 ordered executor는 대상별 amount가 하나도 필요하지 않은데도 `capture_for(effect_index, None)`으로 첫 capture를 조회해 `InvalidStaticData`를 반환했다. 독립 검토의 상수식·resolved Sum × Snapshot/Live 4건 실패 로그를 `evidence/predecessor_review/`에 원본 bytes 그대로 복사했다.

`execute_ordered_skill_effects`에서 expression carrier 존재 확인은 유지한다. targets가 빈 경우에는 target identity lookup·Live member 준비·amount 버퍼 할당을 하지 않고 `Some(Vec::new())`를 command builder에 전달한다. 별도의 `has_effect_capture`가 해당 expression row와 비어 있지 않은 capture 구조를 검사한다. targets가 있으면 기존 첫 실제 target의 capture 검사, effect당 한 번의 member 준비, 각 target의 정확한 capture 선택과 mutation 전 전 대상 평가를 수행한다. effect command·signal·projection·local prefix/action witness settlement·step result·delivery close는 공통 후속 경로다.

`GameDataBuilder::with_resources`는 이번 SourceEmployee/TargetEmployee 제품 시험의 test/debug fixture가 resource catalog를 `build()`의 validation **전에** 제공하게 하는 작은 보조 API다. 이전 탐색 시험의 `unknown resource` 실패는 카탈로그를 `build()` 이후에 교체한 fixture 문제였고, 최종 시험은 이 순서를 교정했다. 그 외 resource runtime·projectile targeting 정책은 수정하지 않았다.

## 제품 시험 경계

- 독립 probe의 실제 `invoke_ability` → release → queued advance/impact 4조합을 제품 `core/mod.rs` 시험으로 편입했다. 각 case는 terminal empty impact log 1개, pending queue/projectile/cast 정리, settled action의 outstanding delivery·pending step 0, damage·HP loss·death 0, active action 0을 확인한다.
- inactive target과 발사 후 달아나 max travel에서 미명중 종료한 target도 같은 빈 impact 완료 경로를 확인한다. target은 존재해도 inactive/미명중이면 임의 amount를 읽지 않는다.
- 빈 DamageExpr·HealExpr 뒤 SourceEmployee Grant는 정확히 한 번 실행되어 resource balance와 fact delta가 3이다. 별도 direct step-result 시험은 빈 target/damage/kill count 0, 후속 자원 effect count 1과 release target 외 capture lookup 거부를 확인한다.
- 후속 TargetEmployee 자원 오류는 `InvalidOwner`로 전파된다. 이미 commit한 SourceEmployee +2를 유지하고, 실패 candidate의 +3과 queue/log 등은 복원한다.

이 교정은 D04-F01 한 건의 successor다. Resolved ByOwner domain 표현, 실제 unit scalar producer, D07 preview/explain, WP03/04, 서버·Unity, production publication은 이번 범위가 아니다. 자체 검증은 독립 재검토 PASS나 WP02/R06 완료 판정이 아니다.
