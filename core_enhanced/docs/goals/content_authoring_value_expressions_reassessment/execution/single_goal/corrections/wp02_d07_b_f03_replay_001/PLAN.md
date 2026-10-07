# F03 동적 amount replay — 현재 위치

2026-09-28 · C04/C05 부분 자체 검증 종료 · successor 단계검토 요청 대기.

- 지침: [교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md) §5 C04/C05, 해당 C16 및 단계검토 템플릿 2026-09-27-v1.
- 선행: [F02 부분 PASS](../../reviews/wp02_d07_b_f02_correction_001_review_001/REVIEW.md). 실제 before/source identity는 [baseline](baseline/entry_001/MANIFEST.json)에 보존.
- 이번 동작: 실제 release carrier → ordered executor/command 소비 → 최근 record → preview/explain encoder의 수치·trace 일치.
- 입력: Source.Attack, Target.HP, typed resource balance, 완료 action result. Live 앞 effect의 HP/resource/result 변화 포함.
- 구현: 기존 fixture·private encoder 사용. 새 시험 module을 기존 cfg(test) tests의 하위 module로 분리한다.
- 기록: 이 PLAN은 현재 위치, EXPERIMENTS는 시도·실패와 원자료, EXPERIMENT_NOTES는 판단을 소유한다. 수치는 최종 판정표에 한 번 모은다.
- 중단: C04/C05 실제 동작·필수 영향 검증이 닫히는 첫 지점에서 target 고정/사본 검토 인계.
- 잔여: C03, C06–C15 및 그 밖의 F03 요구. public selector/battle 교체 검증을 private encoder 시험으로 대체하지 않는다.
- 검증: 집중 신규 시험, value_expr/authoring 및 전체 기본 병렬 회귀, check/clippy/변경 파일 format. production admission/소유권 교정이 필요하면 PC36/R05 재측정, 시험만 추가하면 무변경 근거를 대조해 재사용한다.

## 현재 target·다음 행동

- 실제 before: [baseline manifest](baseline/entry_001/MANIFEST.json), 직전 F02 target과 시작 시 currentness 대조 및 원본 보존.
- after: [successor manifest](target/d07_b_f03_replay_001/MANIFEST.json). exact source·archive·정/역 patch·기록 결속과 검증은 이 manifest 기준.
- [최종 판정표](evidence/FINAL_VALIDATION.json), [측정 재사용 근거](evidence/EVIDENCE_REUSE.json), [검토 인계](REVIEW_HANDOFF.ko.md).
- 다음: 사본 대화에서 C04/C05 successor 독립 단계검토. 수정한 구현자가 검토 PASS를 선언하지 않는다.
- F03 잔여는 C03·C06–C15이며 C16은 이번 checkpoint의 증거에만 적용했다. D07-B 전체 미완료. 후속 큰 작업은 재개 요청 후 진행한다.
