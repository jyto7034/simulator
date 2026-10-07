# R06 WP02 F02/F03 별도 단계검토 인계

검토 대상: `target/f02_f03_correction_001/MANIFEST.json`. 상태: **교정·자체 검증 종료 / 단계검토 타깃 동결 / 재검토 요청 대기**. 검토자는 F02와 F03을 구분해 판정해야 한다. 구현자는 PASS, WP02/Phase/Goal 완료 또는 다음 WP 진입을 선언하지 않는다.

검토 순서는 `CORRECTION_RECORD.ko.md`의 책임 구분을 확인하고, `baseline/entry_001`의 수정 전 bytes와 target의 after archive·복원 delta를 대조하는 것이다. `verification/before_review_probes_retry.log`와 `after_review_probes_final.log`는 동일한 원본 감사 probe의 실패→통과를 보여준다. `verification/all_target_test_escalated.log`, `clippy_final.log`, `focused_rustfmt_final.log`, `f01_review_regression.log`는 직접 실행 결과다. `all_target_test.log`의 Unix socket sandbox 실패와 `before_review_probes.log`의 초기 hook 컴파일 오류는 별도 실패 기록으로 남겼다.

미완료·검토 제한은 `CORRECTION_RECORD.ko.md` 말미에 기록했다. CAP-01/CAP-02, 예산 수치·누적 청구 정책 변경, 순차 effects/result-prefix, WP03, publication은 이번 타깃에 포함되지 않는다.
