# F03 C03 legacy amount — 현재 위치

2026-09-28 · C03 부분 자체 검증 종료·단계검토 요청 대기. 적용 템플릿 2026-09-27-v1.

- [교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md)의 C03/B01 및 해당 C16.
- [직접 선행 C04/C05 부분 PASS](../../reviews/wp02_d07_b_f03_replay_001_review_001/REVIEW.md). [before](baseline/entry_001/MANIFEST.json)는 시작 verify 뒤 byte 보존.
- 기존 typed authoring API의 Damage/Heal 경계값을 정상 catalog compilation→actual ordered command 소비→관측 record로 대조한다. 같은 core clone에서 관측 on/off의 command·gameplay·facts·queue 동등성을 검증한다.
- 변경: 시험 module, command 소비 직전 cfg(test) scoped observer, 음수 Damage의 test-only catalog staging. 음수 Damage 공개 거부를 유지하며 해당 runtime 호환 사례만 정상 공개 경로와 분리한다. 정책/수치식을 바꾸지 않는다. 결함 발견 시 관련 계약 안에서 교정한다.
- 신규 집중 시험, authoring/value_expr, 기본 병렬 전체회귀, check/Clippy/변경 format. cfg(test) 밖 무변경이면 PC36/R05 재사용 근거 대조.
- 계획/현재 위치는 이 PLAN, 이력은 EXPERIMENTS, 현재 판단은 EXPERIMENT_NOTES. 최종 수치·identity는 판정표/manifest에 연결한다.
- C03의 의미 있는 부분이 검증되면 동결·사본 검토 인계에서 중단. C06–C15 및 F03/D07-B 전체는 후속.

## 현재 target·다음 행동

- [최종 판정표](evidence/FINAL_VALIDATION.json), [경계값 실행표](evidence/BOUNDARY_MATRIX.json), [재사용 판정](evidence/EVIDENCE_REUSE.json).
- [검토 인계](REVIEW_HANDOFF.ko.md), [successor manifest](target/d07_b_f03_legacy_001/MANIFEST.json), [격리 복원](target/d07_b_f03_legacy_001/RESTORATION_VERIFICATION.json).
- 다음은 사본의 C03 단계검토. 수정한 구현자가 PASS를 선언하지 않는다.
- C03-OBS01: 원문 RON MIN 파싱 실패는 해결되지 않았으며 C15 통합 경계에서 재대조한다. C03 runtime와 public RON 지원을 구분한다.
- C06–C15 및 F03/D07-B 전체 미완료. 후속 큰 작업은 사용자 재개 요청 뒤 진행한다.
