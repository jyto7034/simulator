# X01 수치식 게시·평가·소비 대표 조사

작성·갱신일: 2026-10-18

**선택 대표 조사·설계 유지 판단·연결 X02 확인 완료. 제품 변경과 테스트 재실행 없음.**
compile→catalog→capture/frame→evaluate→실제 효과/기록 재생의 책임과 실패 경계를 확인했다.
선택 범위에서 확정한 동작 오류나 필수 구조 개선 후보는 없다. 모든 식·유물 조합이나 X01 전체 완료를 뜻하지 않는다.

## 1 범위와 재사용 기준

[README](README.md), [리팩토링 기준](../refactoring_criteria.ko.md)의 구조 평가·후보 선정·유지 조건,
[실행 지침](../system_refactoring_execution_guide.ko.md)의 대표 흐름·근거 재사용·분리 검토를 적용했다.
[X01 실행 제어·자원 수명](X01_RUNTIME_RESOURCE_REVIEW.ko.md)의 host admission/lease,
[S10](S10_ATTACK_SKILL_EFFECT_REVIEW.ko.md)의 ordered effect·carrier 수명과 보정 통합은 재사용했다.

의미 기준은 [data loading 계약](../data_loading_contract.md#value-expression-input-and-publication-boundary)과
[runtime 계약](../core_runtime_contract.md#typed-amount-execution-and-mandatory-replay-r06)이다.
주 대상은 `game/value_expr/{compile,eval,catalog}.rs`, `core/value_expr_runtime.rs`이고,
실제 게시·소비·재생을 연결하는 data/content pack, `core/sim.rs`, `amount_observation.rs`,
`authoring/amount.rs`, `authoring/amount_query.rs`를 필요한 범위에서 확인했다.
parser는 숫자 입력·한도 경계와 기존 검증 근거를 연결했으며 모든 visitor를 새로 감사하지 않았다.

[선택 입력](evidence/x01_expr_001/baseline.json) **86개**가 기존 해당 근거와 일치한다.
`data/mod.rs`는 최초 조사 이후의 S02/S11 D1 주석 교정 hash를 사용했다. 새 실행 변경으로 취급하지 않았다.
이 목록은 전체 전이 의존성·embedded fixture·도구 환경을 동결한 manifest가 아니다.

## 2 대표 흐름과 owner

### 최종 정의에서 immutable plan까지

Content pack은 최종 winning definition을 조립·검증한 뒤 `GameDataBase`를 만든다.
`ValueExprCatalog::compile_skills_with_deadline`은 actual EffectSource를 가진 효과별로 unit·delivery capability·
resource/result reference·limits를 확인하고 EffectSite→kind/Arc plan을 보유한다.
Damage/Heal literal도 legacy provenance를 가진 plan으로 낮추며 authored 표현식과 출처를 구분한다.
Shield 계열도 같은 catalog 경계에 있으나 이번 대표 실제 HP 흐름의 전수 조사 대상으로 확장하지 않았다.

Compiler는 iterative task stack으로 op segment, 요구 reference, 비용과 fingerprint를 만든다.
상수 Add/Mul/Ratio/Clamp의 확정 오류를 게시 전에 거절하고, 정적 문맥만 필요한 aggregate는 catalog에서
공유 evaluator로 실행·최종 범위 검사한다. 동적 입력이 필요한 식을 임의의 0으로 미리 평가하지 않는다.
`ensure_runtime_publishable`은 실제 공급할 수 없는 member/owner 문맥을 별도로 거절한다.
최종 게시 poll을 통과한 generation이 catalog를 소유한다.

`amount_entry_for_source`는 causal program, effective relic program, 기본 generation catalog 순으로
현재 source에 맞는 plan owner를 선택한다. 같은 EffectSite 문자열만으로 다른 plan을 허용하지 않는다.
capture 진입에서 kind·exact Arc plan·generation·authored source·action·provider·delivery를 확인한다.

### release capture와 effect-start frame

`build_value_expr_delivery_capture`는 release 대상과 효과별 capture를 만든다.
Snapshot은 release frame을 보유하고 Live는 소비 시점에 frame을 만든다.
capture와 frame은 frozen input을 보유하며 현재 gameplay의 두 번째 변경 원본이 아니다.
소비 시에는 generation/site/fingerprint, action/delivery 유효성과 release/consume 대상·시각 관계를 검사한다.
이미 정산한 action의 권한을 숫자나 과거 record만으로 복원하지 않는다.

Live frame은 실제 unit/stat/HP/resource/time와 완료한 result prefix를 읽는다.
자원 read의 unit 가용성과 lease 수명은 앞 X01 기록의 구분을 따른다.
result는 같은 action/source의 완료된 앞부분만 읽으며 future/self/미완료·다른 occurrence를 대신 사용하지 않는다.
structural child는 그 occurrence에서 증명된 result frontier를 사용한다.

Resolved aggregate는 실제 target 집합에서 node별 filter/relation view를 준비한다.
Side/employee/equipment-copy 관계와 tile은 domain adapter가 공급하며 evaluator가 inventory나 battle을 재조회하지 않는다.
node view 누락과 유효한 빈 view를 구분한다. member identity를 정렬하고 충돌하는 중복은 거절한다.
Sum/Count의 빈 집합과 Max의 빈 집합 거절은 명시적으로 다르다.

### 계산·효과 적용·후속 실패

`evaluate_compiled_amount`는 evaluator의 typed i128 결과를 Damage/Shield u32 또는 Heal i32로 좁힌다.
계산은 checked 연산을 사용하고 Ratio는 compiler와 evaluator가 공유하는 `divide_rounded`를 사용한다.
컴파일 한도보다 큰 evaluation meter를 허용하지 않으며 work·stack·member·group·retained와 host admission을 연결한다.
오류는 site/node/unit/phase를 보유하고 literal·이전 값으로 대체되지 않는다.

`execute_ordered_skill_effects`는 한 effect의 모든 대상 값을 먼저 평가한 뒤 command를 적용한다.
그 effect의 첫 대상 변경이 같은 effect의 다음 대상 계산에 섞이지 않는다.
완료한 effect는 실제 mitigation 뒤 damage/치유/자원 결과를 prefix에 반영하므로 다음 effect는 앞선 실제 결과를 읽는다.
숫자 binding의 compose는 기존 effective/relic 연산을 호출하며, Wedge의 중간값·별도 strike 소비를
재생 경로와 단순 합칠 대상으로 취급하지 않았다. 해당 유물 조합 전체는 이번 범위 밖이다.

effect 완료 뒤 amount record를 게시하더라도 바깥 runtime candidate 안이다.
뒤 effect가 동적 0 나눗셈·문맥 오류로 실패하면 HP·queue/facts·기존 record·ordinal을 함께 보존한다.
이전 공개 operation의 성공분은 유지하며 정상 입력으로 고친 다음 operation에서 재시도한다.

### 기록과 query의 경계

`PendingAmountEvaluation`은 평가 결과와 그 입력을 모으고 `commit_amount_evaluation`은 old+new replacement peak를
검사한 뒤 최신 immutable record와 ordinal을 바꾼다. 여러 대상의 평가 개수와 보유할 마지막 record를 구분한다.
observation이 켜진 전투의 최신 record 및 마지막 terminal capsule은 event log eviction과 별도 수명을 가진다.

`amount_query::encode_record`는 frozen plan/frame에 같은 `evaluate_compiled_amount`를 사용하고,
보존한 binding의 compose와 기록된 결과를 대조한다. 현재 source를 재조회하거나 action 권한을 새로 만들지 않는다.
`authoring/amount.rs`의 static explanation은 catalog의 정적 문맥 판정 규칙을 사용하되 simulation/evaluation을 수행하지 않는다.
단순 구조 설명과 실제 실행 record 재생은 서로 다른 공개 요청이다.

## 3 설계 평가와 판정

| 항목 | 판정·이유 |
| --- | --- |
| 책임·의존 방향 | 유지. catalog/compiler는 게시 가능한 plan, battle adapter는 실제 문맥, evaluator는 typed 수치 계산, effect executor는 gameplay 변경을 소유 |
| 상태 소유권·수명 | 유지. Arc plan/capture/frame/record는 각각 generation·release·평가 시점의 immutable 근거. 현재 world와 독립적으로 변경되는 복제 원본이 아님 |
| 규칙 중복 | 유지. 반올림·narrowing·평가 kernel·정적 문맥 판정은 공유. compile constant 검증과 runtime dynamic 검증은 phase와 입력이 달라 그대로 같은 함수로 합칠 대상이 아님 |
| 결과·실패 경계 | 유지. effect 안의 평가 선행과 effect 사이의 settled prefix를 분리. record도 gameplay candidate 안에서 게시 |
| identity·결정론 | 유지. generation/plan/source/occurrence 검사와 canonical member 순서가 명시됨. 문자열 ID나 iteration 우연에 권한을 의존하지 않음 |
| 관측·검증 가능성 | 유지. 고정 record를 같은 kernel로 재생하고 실제 HP·prefix·후속 실패·재시도 시험을 연결할 수 있음 |
| 확장 비용 | 유지. opcode 추가 시 schema·compiler·evaluator·지원 capability를 바꾸는 것은 서로 다른 책임이며 닫힌 enum의 exhaustive match가 누락을 드러냄. 현재 필요 없는 plugin/registry 추가 근거 없음 |
| 성능·자원 | 현재 제한·사전 admission을 확인. 전체 hot path의 시간·최대 규모 성능을 새로 측정하지 않았으며 긴 adapter 파일이나 반복 capture만으로 분할/캐시 후보를 확정하지 않음 |

runtime 평가와 query에 binding 검증/compose 호출이 각각 존재하지만 실제 계산 규칙은 binding 메서드가 소유한다.
실행 중 action/target 가용성 및 Wedge 처리와 과거 기록의 독립적 재생은 의미가 달라, wrapper 모양만 보고 통합하지 않는다.
이번 범위에서 필수 리팩토링·오류 교정·설명 교정을 새로 만들지 않는다.

## 4 X02와 재사용한 검증

[원시 결과 연결](evidence/x01_expr_001/reused-tests.json) **30개**는 S08 전수 당시 27개와 S10 R1 이후 3개의 성공이다.
새 테스트 실행은 없고 과거 전수 결과를 현재 전수 통과로 다시 집계하지 않는다.

- 최종 winning 식만 게시, source를 보존한 정적 오류와 unsupported 문맥 거절.
- signed rounding, unit mismatch, overflow/0 나눗셈/Clamp, narrowing, aggregate 빈 집합·정렬·budget.
- Snapshot/Live 시점 차이와 exact generation/action/delivery, 한 effect의 전체 대상 평가 선행.
- mitigation 뒤 실제 damage를 다음 Heal이 소비하는 경로와 이후 오류의 candidate 복원.
- 이전 record/ordinal 보존과 정상 재시도, source 제거 뒤 재생, i128 trace, replacement peak 한도.
- node별 owner view와 실제 released projectile/fixed area, S10 변경 후 snapshot projectile·timestamp 실패·실제 HP.

낮은 수준 테스트가 직접 만든 frame/plan·runtime candidate와 실제 loader/world carrier 테스트를 구분한다.
직접 내부 문맥 오류를 만든 시험을 공개 authoring 지원 범위의 증거로 바꾸지 않는다.
특히 기존 `r06_c10_previous_record_survives_later_effect_fault_and_clean_retry`는 정상 동적 식의 0 denominator를
실제 소비에서 실패시킨 뒤 값 교정·재시도를 검증하므로 실패 원자성의 중요한 보완 근거다.

기존 실행은 기본 `game_core` feature다. `runtime-observation` feature와 authoring amount observation 옵션은
같은 설정이 아니다. amount observation 테스트가 기본 feature에서 실행됐다는 사실을 feature 조합 동등성으로
확대하지 않는다. 제품 변경·새 확인 실패가 없어 Cargo를 반복 실행하지 않았다.

## 5 분리된 조사 검토와 한계

작성 후 별도 읽기 전용 pass에서 현재 source·계약·원시 결과와 유지 판단·인계 범위를 대조했다.
[검토](evidence/x01_expr_001/review.json)와 [문서 검증](evidence/x01_expr_001/verification.json)에 결과를 기록한다.
동일 작성자 검토이며 독립 감사·새 실행 검증으로 부르지 않는다.

모든 opcode/owner/relic/Shield 조합, malformed 내부 plan 방어, parser 전체, release/feature 조합,
최대 규모 성능·외부 Unity 소비를 전수 확인하지 않았다. 공통 컨테이너·RNG/ID·선택적 관측 전체도 미완료다.
앞 X01의 중첩 store 복사 비용과 S08 comparator 24쌍/31 variant 한계는 유지한다.

## 6 다음 행동과 범위

후속 대표 조사는 [복사·결정론·관측 기록](X01_COPY_DETERMINISM_OBSERVATION_REVIEW.ko.md)에서 완료했다. 아래는 당시 인계다.

**X01 공통 상태 복사·결정론·선택적 관측의 대표 조사와 연결 X02 확인·유지/개선 판정·계획**이다.
`owned_collections.rs`, `game/determinism.rs`, `runtime_observation.rs` 및 하위 모듈,
실제 application/runtime candidate clone과 `application/runtime_observation_tests.rs`를 시작점으로 삼는다.

대표 흐름은 candidate 복사→실패/재시도에서 container 계보·RNG/ID 진행 보존,
그리고 관측을 켰을 때 gameplay 결과와 예산 정책이 바뀌지 않는 경계다.
S01/S04의 commit/복원, S08/S09의 timestamp, 앞 X01의 예산 소유권은 재사용한다.
관측 feature 근거가 부족하면 필요한 선택 실행만 배정하며 무조건 전수 재실행하지 않는다.

선택 흐름의 owner·수명·검증 근거와 남은 X01/X02 항목을 지도에 연결한 뒤 시스템 간 통합 검토 준비 여부를 판단한다.
아직 확인하지 않은 공통 기반을 이번 수치식 조사 완료에 포함하거나 전체 Core 완료로 넘어가지 않는다.
