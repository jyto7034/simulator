# A07 — 현재 코드 전환·재사용·유효 중간 상태

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

원래 논리 설계와 승인 목표는 유지하고 새 R owner/현재 코드 전환 경계를 반영했다. 새 문법·타입·포트 이름은 실제 구현 존재의 증거가 아니다.

## 1. 기본 원칙

이 개정은 현재 본문으로 확인한 producer/consumer·저장/종료 경계를 바탕으로 R 단위와 최소 atomic group을 제시한다. 기존 v2.1의 탐색 지도/전환 후보를 구현 완료로 간주하지 않는다. 각 R의 actual patch/test는 아직 미래 작업이다.

## 2. 주요 전환

R01은 기존 AG01 그대로 compiler/metadata/latest RON/direct consumer를 전환한다. R02는 fixed embedded source inventory를 외부 snapshot/resolver로 옮기고 run pin/save fingerprint까지 동반한다. R04는 equipment copy provenance와 receipts/handle, R07은 fragment count/progress 장부의 전체 reader/writer, R14는 모든 개인 plan consumer, R09는 침식/직원/clock/result를 묶는다.

## 3. 현재 kernel 재사용

damage/heal 산술·현재 carrier·spatial query·boss graph·SpawnKernel·wave bookkeeping·EquipmentLoadoutTransition·EconomyGrantState·world prepare/commit를 재사용한다. 같은 책임을 가진 영구 이중 시스템을 만들지 않는다. 코드 이름에 adapter가 있다는 이유로 legacy shim으로 간주하지 않는다.

## 4. 상태·표시·평가·시험

새 사본/form/resource/modifier는 도입 시 snapshot/save/entry/result와 private/public 실제 fixture를 함께 소유한다. 정의 catalog와 개인 effective projection을 구별한다. Certified 가중은 승인된 모델만 쓰고 새로운 입력의 정책대기를 숨기지 않는다. 유지할 회귀와 승인 변경으로 바뀔 의미를 따로 기록한다.

## 5. 최신 경로와 미지원

미지원 고급 op는 명시 거부한다. 아직 내용이 부족한 최소 playable profile을 공식 콘텐츠 완성으로 이름만 바꾸지 않는다. 구형 grammar/alias/이중 executor를 기본 해법으로 남기지 않는다. 최소 Goal의 저장 비변경 조건과 이후 실제 schema cutover는 서로 다르다.

## 6. 적용과 재검증

기존 dirty 변경을 보존하고 이 보고서는 새 별도 경로로만 적용한다. shared type/source 의미가 바뀌면 관련 receipt만 갱신한다. R23 최적화가 지원 의미를 바꾸는 경우 R22 독립 인수의 영향 부분을 재실행한다. 현재 문서 검사를 게임 독립 완료 감사로 세지 않는다.

원문 설계 근거: [S086](../../evidence/SOURCES.ko.md#s086). 현재 코드 근거와 각 단계의 상세 동시 전환은 [DEPENDENCIES](../DEPENDENCIES.ko.md) 및 [조사 결과](../../report/REVIEW.ko.md)를 따른다.
