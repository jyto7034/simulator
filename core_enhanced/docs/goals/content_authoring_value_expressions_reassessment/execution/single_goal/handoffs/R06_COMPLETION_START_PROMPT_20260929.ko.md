# R06 잔여 전체 구현 시작 요청

**이전 R06 단독 요청문이다. 현재는 [R06→R07 시작 요청문](R06_R07_CONTINUOUS_START_PROMPT_20260929.ko.md)을 전달한다.** 아래 원문은 이전 범위 기록으로 보존하며 최신 시작 요청으로 사용하지 않는다. 이미 제품을 수정 중인 대화가 있다면 그 대화에서 재개하고 같은 workspace의 두 대화가 동시에 수정하지 않게 한다.

```text
R06의 C07 이후 남은 전체 구현을 수행해줘.

먼저 다음 지시서를 읽어줘:
core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/R06_COMPLETION_IMPLEMENTATION_REQUEST_20260929.ko.md

이 요청은 GS-CAVER-SCOPE50-001 / GOAL-CAVER-01 / PHASE-CAVER-01의 GEX-00 인수와 GEX-10 잔여 전체 구현·자체 검증을 승인한다.
기존의 항목별 중간 단계검토 중단은 이번 실행에서 해제한다. C08 하나나 의미 있는 부분을 끝냈다는 이유로 멈추지 말고, C08–C15 및 OBS 교정, WP03, D07-C, 공개 실행, WP04 최종 검증까지 이어가줘.

C07은 자체 검증·동결 완료이며 독립 검토 PASS는 아니다. 최신 347개 source와 manifest를 대조하고 실제 before를 보존해줘. 기존 frozen 기록은 수정하지 말고 지시서의 기존 execution directory/세 기록 역할을 이어 사용해줘.

지시서의 실제 코드·설계·기대값을 확인한 뒤 구현하고, 시도·실패·교정·판단과 원자료를 진행 중 기록해줘. 관통 투사체 만료 시 cast 잔존(C07-OBS01)과 공개 RON i32::MIN 파싱(C03-OBS01)도 완료 전에 교정해줘.

작업 시작 후 나는 자리를 비워 중간 질문에 답할 수 없어. routine 구현 선택뿐 아니라 R06 구현에 필요한 미정 정책도 네 재량으로 결정하고 그대로 구현을 이어가줘. 기존 명시적 합의와 데이터·호환성 보존을 우선하고, 질문 답변 대기로 멈추지 말아줘.
결정한 정책·대안·선택 이유·영향·되돌리는 방법과 검증을 세 기록 역할에 남기고 최종 인계에서 모아 보여줘. 수용 기준을 줄이거나 실패를 숨겨 완료 처리하지 말고, 최종 독립 감사 대기 경계는 유지해줘.

전체 자체 검증과 최종 target 동결까지 마친 뒤 '구현·자체 검증 종료 / 최종 target 동결 / 감사 요청 대기'로 종료해줘. GEX-20 독립 감사나 R07은 시작하지 말아줘.
```

정확한 기준·명령·종료 조건은 [구현 지시서](R06_COMPLETION_IMPLEMENTATION_REQUEST_20260929.ko.md)가 소유한다.
