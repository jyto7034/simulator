# R06 비용 판정 정책 변경 — concurrent peak 경고 전환

작성일: 2026-09-27 KST. 대상: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`의 이후 작업.

## 근거와 결정

사용자는 실행 안전장치의 중요성을 확인하면서 실제 콘텐츠 측정에 기반한 한도 설정을 요청했다.
“기존 약9MB 기준은 실패 기록을 보존하면서 개발 차단에서 경고·관찰로 전환”을 포함한 추천안에
**“응, 모두 너의 추천안대로”**라고 명시적으로 동의했고, 검토·계획 수립을 요청했다.

이에 **64-session concurrent peak_delta_bytes 9,030,188 초과만으로 R06 개발을 차단하지 않는다.**
이후 단계 판단은 해당 수치를 warning/observation으로 취급한다. baseline 값을 새 cap으로 덮어쓰거나
평균으로 실패를 지우지 않는다. 전투당3MB·전체192MB 등 대체 숫자도 확정하지 않는다.

변경 범위는 이 특정 peak 기준의 차단 효력이다. 실제 gameplay 안전 한도·parser/typed/rollback·결과 parity,
event-heavy 및 다른 allocation operations/retained 검증을 일괄 해제하지 않는다.
실제 OOM/비정상 종료·결과 불일치·측정 무효·누적 회수 결함 등 별도 문제는 이 예외로 정상 처리하지 않는다.

## 기존 문서와 증거의 관계

- [D08](../../../CODE_REVIEW_003.ko.md)의 “원 cap 침범 시 구조 교정 또는 required evidence blocker” 중
  **이 concurrent peak 숫자만으로 차단하는 부분**을 이후 실행에서 이 결정으로 대체한다.
- [이전 비용 교정 지시서](WP02_COST_REGRESSION_IMPLEMENTATION_REQUEST_20260927.ko.md)의 필수 footprint 교정,
  A/B/B/A 및 후보 추가3회/25sample 통과 요구는 더 이상 다음 필수 작업이 아니다.
- [D04-F01 독립 검토](../reviews/wp02_d04_f01_correction_001_review_001/REVIEW.md)의 당시 기능 PASS·비용 보류는 역사로 보존한다.
  R05-E01은 **원인 규명/수치 개선으로 해결된 것이 아니라 사용자가 차단 정책을 변경한 것**이다.
  이후 구현 진입에서는 이 사유를 단독 blocker로 사용하지 않는다. 기존 report/verdict/manifest를 재작성하지 않는다.
- [기존 비용 조사](../investigations/wp02_cost_regression_001/INVESTIGATION.ko.md)의 source 대조·실패·변동 관찰은 유효한 기록이다.
- [새 별도 계획](../../../../../plans/runtime_safety_observability/README.ko.md)은 R06 필수 WP 추가가 아니다.

정책 변경은 D04 전체·WP02·R06의 완료 판정이 아니다. 남은 ByOwner 명시 domain/scalar producer·D07·WP03/04와
production publication 차단은 유지한다. D04-F01 source를 되돌리거나 추가 기능을 구현하지 않는다.

## 현재 target의 적용과 다음 실행의 판정

현재 동결 target SHA: `e4f0d77b1bd3066493d605271605d7a4b6e16317a6d540d303909b076b480104`.
333개 source digest: `34ceedbebc84b9396d42f92fa31033fe0b831a90e5325c8c4463e0c88f067145`.
기존 검토의 최종본 max9,040,041 / 9,040,546 / 9,020,761 및 D04 대조9,050,125는 그대로 남긴다.
이번 문서 때문에 추가 비용 부하를 돌리지 않았다. 현재성은 새 계획의 evidence/CURRENTNESS.log에 연결한다.

원 `final_performance` 도구와 baseline은 변경하지 않았다. 따라서 실행 시 raw assertion/exit101은 여전히 발생할 수 있다.
새 실행 PLAN/summary에서 **원 guard 결과와 변경된 정책 판정을 별도 기록**한다.
도구를 실행하지 않았는데 warning 정책이 코드에 적용됐다고 주장하지 않는다.

향후 정책 판독 adapter를 구현한다면 다음을 모두 확인해야 한다.

1. exact source/tool/input/baseline, observer 적용 여부, event-heavy/concurrent 원 sample 수와 schema가 유효하다.
2. 각 sample로 원 operations·retained·peak와 결과 parity를 재계산한다. concurrent peak만 경고로 분류한다.
3. build/환경 오류·다른 panic·부족한 receipt·결과 불일치·다른 guard 실패를 경고로 낮추지 않는다.
4. exit101이면 실패 원인이 해당 concurrent assertion뿐인지 raw/log와 함께 결속한다. 모호하면 검증 미완료다.
5. raw exit·실제 수치·정책 warning 이유를 함께 저장한다. 모든 exit101을 무시하거나 raw PASS로 바꾸지 않는다.

정책 판독 adapter의 제품/tool 코드는 이번에 구현하지 않았다. 다음 관련 구현이 있을 때 이 계약을 따른다.
R06의 다음 기능 작업은 잔여 D04 domain/consumer 설계 정리로 이어갈 수 있으며, 이 계획 문서가 구현 자동 시작을 뜻하지 않는다.
