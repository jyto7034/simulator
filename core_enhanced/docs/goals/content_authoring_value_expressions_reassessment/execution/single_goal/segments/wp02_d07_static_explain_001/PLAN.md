# R06 WP02 D07-A 정적 compiled amount 설명 PLAN

- 시작점은 D04-OWNER-F01 successor manifest SHA-256 `58505bbcd34089b9c1ec3d70f4d9df4ca32a2397e1115c68386364e44514e549`, 확장 337개 digest `8aa03d6bd04f8d524fa61b465a9bd7a4171062a1190f615b794e20f915190f04`다. 구현 직전 predecessor currentness와 격리 양방향 복원을 통과한 실제 바이트를 before로 보존한다.
- D07-A만 구현한다. 기존 `ExplainQuery::Effect`에 catalog의 winning compiled amount plan을 borrowed descriptor로 연결하고, 구조·단위·moment·requirements·source·fingerprint·cost·publication을 출력한다.
- context-free와 runtime context 필요를 catalog validation과 같은 분류 함수로 판단한다. 두 경우 모두 정적 explain에서는 `performed=false`이며 amount·trace·final damage를 만들지 않는다.
- 금액 effect의 plan 부재와 site/kind/source 불일치는 typed request failure로 거부한다. 정상 비금액 effect만 `value_expression:null`이다.
- 기존 query Inspection scope, work/metadata/result bytes/cancellation을 유지하고 lookup·검사·segment/fingerprint 설명 비용을 admission한다. 전체 catalog 순회나 무제한 JSON 중간값을 만들지 않는다.
- A01–A10과 기존 authoring/value_expr/D04 회귀를 최종 source에서 검증하고 실제 CLI `author_content explain`을 실행한다. runtime 평가 경로를 바꾸지 않으므로 PC36/R05는 재측정하지 않는다.
- 새 target `target/d07_static_explain_001`에 before/after, 신규 파일 부재→존재 delta, 양방향 복원, 실행 원자료와 인계서를 동결한다. D07-B/C·WP03/04나 전체 완료를 선언하지 않는다.
