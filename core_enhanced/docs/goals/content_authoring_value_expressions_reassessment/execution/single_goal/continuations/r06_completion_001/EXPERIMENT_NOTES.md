# 현재 판단

- 사용자 위임은 R06/R07 정책 판단에 적용한다. 기존 explicit 계약/저장/결정성을 우선하며 독립 감사 판정을 쓰지 않는다.
- C07-OBS01: 새 impact/effect를 만들지 않고 기존 step progress만 종료한다. zero damage gate는 unsatisfied로 닫아 후속 step을 기존 skip 경로로 보낸다.
- C07 test의 pending cast1은 의도된 계약이 아니라 기록된 결함 oracle이므로 교정 후 cleanup0으로 강화한다. 원본 시험 bytes/실패는 before 및 이전 동결에 보존한다.
- public expression은 B/C 소비와 검증 연결 후 지원 집합만 공개한다. 초기 blanket 차단은 현재 미완료 상태다.
- R07 제품 변경은 R06 경계 후 수행한다. 사본 정책/계획 검증을 수행했다고 선반영하지 않는다.

- 위임 기술 결정 D01: Certified total/model을 보존하고 fingerprint는 별도 공개 조회. semantic generation과 정렬된 직원/실제 슬롯/copy/level/growth/profile/파편 입력을 canonical encode한다. readiness/current HP/consumable 제외. 매 전투 total 계산에 관측용 전체 hash를 강제하는 대안은 불필요 비용/기존 API 파괴 때문에 폐기. 별도 조회이므로 cache 없음.

## 관측/공개 capability 결정
record는 입력 source Arc를 private immutable binding으로 공유하여 source-copy 변조 조회를 거부한다. 현재 unit/action 생존 여부를 조회하지 않는다. terminal cached extent에 mandatory/frame/trace/fact payload와 old+new coexistence를 포함하며 기존 operation metadata cap을 사용한다. 새 세션 합산 정책은 만들지 않았다.
공개 capability는 compiled context/result/unit/bound 검증과 supported finite member 집합을 따른다. resolved owner는 ByResolvedOwner, 과거 ByOwner 혼용과 미지원 memberStat/raw scalar는 거부한다. 전체 authored 차단 대신 실제 지원 expression만 공개했다. 정책 변경이 필요하면 catalog predicate를 되돌릴 수 있으나 최종 완료에는 실제 공개 지원이 필수다.
취소 검증은 cfg(test) thread-local schedule에서 실제 CancellationToken을 취소하고 기존 poll/transaction을 통과한다. sleep/race나 실패 결과 대체 없음. 큰 정수 검증은 compiler constant folding을 우회해 실제 runtime input에서2^100+1을 만들며 Value/f64를 oracle로 쓰지 않는다.


## 통합 근거 보완

기본/feature 전체 성공 source 이후 product Rust/data/config 변경은 없고, canonical 문서 세 개와 finite workload 시험만 추가했다. 새 source membership에 include_bytes 원본 V7 fixture를 포함했다. runtime 의미 검증은 이전 full evidence의 관련 bytes와 비교하여 재사용하고 새 test는 focused nonzero 실행한다. source digest 변경을 무시하지 않고 final manifest에 이 차이를 설명한다.

실제 repeated-area workload에서 casts1/8/32의 HP는 관측 off/on 각각9998/9968/9872였다. observation 기록 retained4355 bytes는64ticks까지 일정; 총 work는 off26856/215034/1026346, on28678/233950/1103870. controlled clone slots는 off447/479/671, on478/510/702였다. battle의 다른 기록 보유량 증가와 latest amount 자체를 구별한다. `finite_workload_001` compile 두 오류는 fixture 코드의 phase enum/서로 다른 error 타입 사용 오류; `002` 교정 후1test pass.

CLI `compiled_graph`는 boss graph 전용이므로 Skill에는 typed unsupported를 준다. Skill definition explain으로 실제 schema에 맞춰 성공시켰다. content parse 실패 exit1과 request 실패 exit2를 구별했다. 초기 harness의 잘못된 exit/status 기대와 잘못된 skill occurrence 편집은 실패 원자료로 보존했고, 제품 거부를 숨기거나 성공으로 바꾸지 않았다. `cli_finish_005`와 `cli_aggregate_002`는 실제 종료코드/원문을 검사한다.
