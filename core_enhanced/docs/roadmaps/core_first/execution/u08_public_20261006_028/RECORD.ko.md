# U08 R06 교정 인수028

2026-10-06T13:14:30.753024+00:00

U08 현재 교정028: R06 선행 수용 조건 검토에서 공개 콘텐츠 사전 검증 누락을 실제 재현했다. AuthoredMembers의 Magnitude/Milliseconds 혼합 값을 Sum/Max로 읽는 식에 외부 Source Attack 참조를 추가하면, 단독 정적 식과 달리 validation이 성공하고 generation identity가 발급된다. 원래 publication 전 UnitMismatch 거부 요건은 바꾸지 않는다. source438/380e에서 7개 공개 CLI 조건 중 2개가 거부 oracle을 위반했다. 정적 거부2·동일unit 정상2·원fixture 정상1은 충족했다. 초기 관측 도구의 stderr 파싱 누락은 원본을 보존하고 실제 raw를 재해석했으며 제품 재실행 없이 구분했다.

통합006은 fmt와 Core default/test-fixtures 각각1958 PASS/0FAIL/0ignored를 보존한 채 교정 전 구간을 닫았다. 다음 묶음 전 정지했던 본인 소유 loop만 종료했고 outer receipt의 실제 exit는 -9다. 이는 위 두 suite의 실패가 아니다. source438/1322입력/외부archive2·필수8member의 before/after는 동일하며 나머지46기능+8성능은 미시작이다. 전체 기능·성능·최종화·독립 감사·U09는 아직 미완료다.

root는 기존 frozen R06 의미를 유지한 교정을 승인한다. Sol은 compile.rs 단일 제품 파일에서 해당 Aggregate의 직접 AggregateMember payload read와 작성 시 확정된 unit을 연결하는 최소 검증을 구현한다. catalog의 동적 평가 gate, runtime evaluator, 공개 API/AST/plan layout/버전은 바꾸지 않는다. main Astra는 새 시험 설계를 마무리한 뒤 별도 인계로 source 시험을 보강한다. 두 역할의 실행은 겹치지 않고, 다른 작성자는 HOLD다. 추가 compile work는 기존 한도 안에서 명시적으로 계상하며 cap·workload·기존 기대값을 완화하지 않는다. 새 교정 결과와 최종 시험 source가 닫히기 전 통합007/harness005를 실행하지 않는다.

[교정 설계](CORRECTION_DESIGN.ko.md)와 [실제 재현](../../../../goals/core_relic_numeric_variants/evidence/r06_publication_probe001/RESULT.json), [006 구간 종료](../../../../goals/core_relic_numeric_variants/evidence/execution_harness_binding006/ABORTED_INTERVAL.json)를 따른다.
