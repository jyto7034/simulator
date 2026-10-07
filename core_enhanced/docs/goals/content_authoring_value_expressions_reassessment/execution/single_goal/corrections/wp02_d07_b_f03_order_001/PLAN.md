# D07-B F03 C07 현재 위치

2026-09-29 · C07 부분 자체 검증 완료·단계검토 동결. 적용 템플릿 2026-09-27-v1.

직접 선행: [C06 부분 PASS](../../reviews/wp02_d07_b_f03_aggregate_001_review_001/REVIEW.md).
[교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md)의 C07/B06/B09 및 이번 C16만 진행한다.
실제 before345개를 baseline/entry_001에 보존하고 predecessor currentness·격리 복원 통과를 기록했다.

1. 복수 effect/target 동일 timestamp의 실제 소비·순번·최근1건.
2. 관통 projectile의 복수 impact·0값 기록·만료 후 기록 보존, 별도 D04-F01 무대상 impact의 후속 완료.
3. 공개 legacy TrialSession에서 실제 A→B 교체, Latest/Exact preview/explain 및 A not_retained.
4. 필수 검증·근거 재사용 영향 대조·source 전후 복원 후 동결, 사본 검토 대기에서 중단.

제품 동작은 먼저 보존하며 부족한 시험을 추가한다. 실행 경로 결함이 드러나면 원인을 기록하고 해당 범위만 교정한다.
C08–C15 및 C03-OBS01(RON i32::MIN)은 후속. 전체 F03/D07-B 완료를 선언하지 않는다.
[수행 이력](EXPERIMENTS.md), [현재 판단](EXPERIMENT_NOTES.md).

## 현재 결과와 미해결 의무

집중 `c07_order_007` 4개 통과. 공개7쌍 조회, 같은10ms의6회 순서 실행/36평가,
0/2 damage×expiry/hit_limit4사례, D04-F01 empty 후속 resource·완료를 확인했다.
최종347개 source에서 check·authoring113·value_expr104·전체1560(약132초)·Clippy·format 통과.
원본 영수증은 [FINAL_VALIDATION](evidence/FINAL_VALIDATION.json), 조건 판정은 [ORDER_MATRIX](evidence/ORDER_MATRIX.json)에 연결한다.

**사용자 확정:** 관통 만료 후 active cast/pending delivery가 남는 C07-OBS01은 C09 수명 교정으로 이관한다.
이번 만료 사례의 record 보존은 시전 수명 정상 판정이 아니다. F03 전체 완료 전에 교정 필수.
RON MIN C03-OBS01은 C15 의무를 유지한다. C08–C15를 이 checkpoint에서 시작하지 않는다.
[검토 인계서](REVIEW_HANDOFF.ko.md), [미해결 finding](evidence/C07_OBS01.json).

## 검토 target과 중단

[target/d07_b_f03_order_001/MANIFEST.json](target/d07_b_f03_order_001/MANIFEST.json): before345 → after347,
기존 test parent2개 변경·신규 test 모듈2개. PC36/R05는 [재사용 근거](evidence/EVIDENCE_REUSE.json)를 따른다.
`python3 freeze.py verify`로 currentness·membership 및 격리 양방향 byte 복원을 확인한다.
다음은 사본 C07 부분 범위 독립 검토. C09 교정 또는 C08 구현을 자동 착수하지 않는다.
