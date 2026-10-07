# A08 — 예제의 보존 의미와 실제 검증 owner

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

원래 논리 설계와 승인 목표는 유지하고 새 R owner/현재 코드 전환 경계를 반영했다. 새 문법·타입·포트 이름은 실제 구현 존재의 증거가 아니다.

아래는 기존 E01–E06의 의미를 유지한 개정 시험 설계다. **현재 parser가 받는 새 RON이나 실행한 fixture가 아니다.** 원문 제안 문법과 숫자는 시험용이며 공식 밸런스 승인값이 아니다.

## E01 — 범위 피해 후 자기 회복 한 번

Damage(bite)의 actual HP loss 결과를 만든 뒤 조건을 충족하면 Heal(self)을 한 번 실행한다. 예제 Attack100/자기500/1000/적2명500·방어0에서 피해200씩과 자기100회복을 기대하는 손계산은 기존 산술과 대조할 fixture 입력이다. 피해 footprint만 바꾸어도 heal의 scope/source/횟수는 변하지 않아야 한다. “유효 적중” 전체 의미를 이 예제의 HP-loss 조건 하나로 확정하지 않는다. owner: R06/R10/R14/R15, 실제 공식 연결 R21/C02.

## E02 — 보스 사격의 release와 settled

시험 clock t0 접수/t100 release/t300 impact에서 Released waiter는 t100에 다음 state, EffectsSettled waiter는 t300 뒤 결과를 소비한다. t50 state exit은 미방출을 취소하고, t150 exit은 승인된 detach-released 정책에 따라 날아가는 payload를 보존한다. 이전 state generation의 늦은 결과는 새 state를 전이시키지 않는다. owner: R04/R08, 확장 R18.

## E03 — 공유 보호막의 단일 전환

하나의 effect instance가 pool을 소유하고 여러 recipient는 membership을 가진다. 단일 recipe는 기존 적격 수혜자 중 하나로 제한하고 초기량을 2S로 바꾸되 source/수명/소진 종료를 보존한다. 600 pool에 감쇄후 A400/B350 순서면 B HP 손실150이라는 값은 입력이 정해진 시험 산술이다. 수혜자 loop에서 초기량을 여러 번 키우지 않는다. S식/개인공유 우선순위/침식 비교안은 policy gate. owner: R05/R12/R15/R21-C04.

## E04 — Hello/Goodbye

일반 시전이 Hello activation과 basic PiercingDelivery override lease를 만든다. 자연 만료는 lease 제거 후 Idle이며 Goodbye가 없다. 승인된 재입력은 잔여시간을 snapshot하고 Goodbye skill action을 만든 뒤 lease를 정리한다. Hello 직접 기본공격 출처와 Goodbye 직접 skill 출처를 구별한다. old weapon 전체를 snapshot했다가 복원하지 않는다. owner: R10/R11/R21-C03.

## E05 — 태아 집중과 단일 전환

channel activation이 행동 lease와 aura membership을 소유한다. 정규 기본 공격의 피해/속도만 수혜자에게 적용하며 provider Attack 전체나 summon/fragment damage로 확대하지 않는다. 입장/이탈/종료, release snapshot, windup 경계를 시험한다. 상태별 +20/+40/+80%와 +15/+20/+30은 각 property의 교체값이지 모두 스칼라 곱이 아니다. owner: R06/R11/R15/R21-C05.

## E06 — 래티씨아와 원천별 전파

내 source가 만든 small이 있는 eligible tile에서 비용 commit과 big 형태 전환을 함께 처리한다. node ID 유지/교체는 기술 선택이지만 old edge/event가 잘못 적용되면 안 된다. 연결망은 접근/선통과 사건에서 정해진 순서로 attack→consume하고 consumed/stale node를 재발동하지 않는다. 별도 감염 fixture는 source별 살아 있는 숙주와 위력을 snapshot→consume→damage 요청하며 기폭의 재감염/사망전파 suppression을 유지한다. 감염 membership과 trap graph를 억지로 하나로 합치지 않는다. owner: R13/R16/R17/R21-C06.

## 실제 증거로 승격하는 조건

각 E를 채택된 latest RON·expected trace/state·정상/실패/취소/종료 case로 만들고 현재 parser/compiler/executor와 공개 로딩 포트에서 실행해야 한다. 원문 E01–E06과 별도로 C07의 보스 작업 신호 및 전체 COV 표현력 대조도 필요하다. 코드/게임 실행 없는 손계산이나 문서 도식은 PASS가 아니다.

원문 설계 근거: [S087](../../evidence/SOURCES.ko.md#s087). 현재 코드 근거와 각 단계의 상세 동시 전환은 [DEPENDENCIES](../DEPENDENCIES.ko.md) 및 [조사 결과](../../report/REVIEW.ko.md)를 따른다.
