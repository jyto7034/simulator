# U09 실행 경계

현재 동결은 `FPF-CRSA-001`, 계획은 [PP-CRSA-001](P2G_40_PHASE_PLAN_001.json)이다. exact prerequisite과 repair basis는 JSON이 소유하며 change/obligation/resolution/supersession 집합은 모두 explicit empty다. 정책을 재결정하지 않는다.

한 Phase에 세 실행 node를 둔다. `AG-CRSA-RUNTIME`은 strict schema, 개인 effective plan, 실제 전투 consumer, world/save/query/계량의 네 작업 묶음을 함께 전환한다. plan은 원 source와 generated effect 주소를 공급하고 consumer는 실제 target·Flow·occurrence·release 의미를 정하며, configuration/restore는 그 결과를 commit 전에 검증해야 한다. 이 순환을 끊어 중간 제품 완료로 선언하면 parser-only 지원, 원형 재조회 또는 부분 commit이 남는다. schema도 실제 지원 descriptor를 게시하므로 같은 atomic group이다.

`WP-CRSA-PUBLIC`은 완성된 runtime 위의 정상 외부 RON·획득·장착·시전·설명 사례와 작성 문서다. `WP-CRSA-VERIFY`는 독립 oracle, durable probe/comparator, 실제 통합·회귀·성능 결과와 whole-goal 감사 인계를 소유한다. 두 작업은 runtime 내부 타입과 구분할 수 있으나 모두 완료돼야 유일 Phase의 출구가 유효하다. Phase 병렬화는 승인하지 않는다. 별도 파일의 제품/시험 준비는 소유를 정해 병행할 수 있으며 실행 담당자는 한 명이다.

공유 계약과 실제 재사용 지점은 [코드 검토 TD01–06](CODE_REVIEW_001.ko.md), [runtime 검토](review_parts/RUNTIME_REVIEW_001.json), [시험 seam·34문장 mapping](review_parts/TEST_SEAMS_REVIEW_001.json)을 인계한다. 구체 API 이름·enum·cache·effect-local 또는 동등 lowering은 아직 구현이 아니며 Sol이 상세화한다. Root는 의미·권한·원자성·수명을 확인하고 Astra는 정책에서 독립 기대값을 도출한다.

특히 target invalid는 recipient 연결만 종료하고 원 source channel/cost/expiry를 보존한다. single2S는 shared protection stage, source 계산 한 번과 pool 한 개를 유지한다. regular-only 새 coefficient는 Triggered/ExtraAttack을 포함하지 않으며 비변형 legacy aura의 묵시 변경을 허용하지 않는다. Monk는 해당 damage의 settled HpLost가 양수일 때 Self heal 한 번이다.

17개 integration acceptance에 durable 검증 owner를 모두 배정했다. 실제 공개 입력·관찰된 결과·입력/source/toolchain/원시 receipt가 필요하며 private fixture, 컴파일 또는 조회 설명만으로 완료하지 않는다. 마지막 상태는 U10·공식 미정 밸런스·Unity 없이 유효해야 하고, 223 선행 조건과 별도 R06 31행의 영향 및 기존 PC/R05 제약을 보존한다.

기계 검증은 6 work packages, 18 edges, cyclic SCC 1, atomic group 1, 최종 node 3, Phase 1, audit row 1, integration row 17을 산출했다. 표준 생성과 기본 검증 모두 exit0/passed/failures0이다. 이는 계획 구조 검증이며 제품·시험 완료 판정은 아니다.
