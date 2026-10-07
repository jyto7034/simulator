# R06 WP02 F01 admission 부분 교정 단계검토 입구

대상 Goal: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`. 선행 감사: `reviews/wp02_runtime_frame_capture_001_review_001/REVIEW.md`의 `CORRECTION_REQUIRED`. 이번 target은 `target/f01_admission_001/MANIFEST.json`; detached hash는 같은 디렉터리의 `MANIFEST.sha256`이다. 검토 전 manifest와 current source·evidence를 read-only verifier로 확인한다.

검토 범위는 F01의 현재 private BattleCore frame/evaluator host work·retained·취소 admission과 기존 kernel 계산 의미다. 수정된 제품 파일은 `src/game/value_expr/{mod,eval}.rs`, `src/game/battle/core/value_expr_runtime.rs` 3개다. `baseline/entry_001`이 직접 before이며, 선행 WP02 target의 동결 source 대비 기존 Unix 테스트 2개 delta는 별도 보존했다. `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, `evidence/EXECUTION_RECORDS.json`, 원본 감사 probe 결과와 PC 원본 fixture 재측정이 판단 입구다.

검토자는 각 변경의 비용 소유권, local/host 선청구와 오류 후 미환급, Snapshot 중복 청구 여부, 정상 production scope의 결속, work=1의 조기 거부, 잔여 retained 1 byte, 평가 도중 취소, 값/trace/rounding 회귀를 확인해 달라. Clippy warning과 초기 sandbox 실패·재실행 로그도 보존했다.

F02/F03/CAP-01/CAP-02는 미교정이며 기존 finding의 재검토 PASS를 요청하지 않는다. 순차 effects/prefix/WP03/publication도 이 target 범위 밖이다. 이 target의 부분 검토 결과와 별도의 다음 구현 요청 전에는 후속 제품 변경을 시작하지 않는다.
