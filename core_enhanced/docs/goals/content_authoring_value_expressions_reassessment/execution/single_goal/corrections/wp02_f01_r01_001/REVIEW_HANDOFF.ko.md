# R06 WP02 F01-R01 별도 단계검토 인계

검토 대상: `target/f01_r01_correction_001/MANIFEST.json`. 상태는 **교정·자체 검증 완료 / 재검토 요청 대기**다. 구현자가 재검토 PASS, F01 전체 종료, WP02 완료 또는 다음 WP 진입을 선언하지 않는다.

검토 범위는 [F01-R01 보고서](../../reviews/wp02_f01_admission_001_review_001/REVIEW.md)의 Live frame 선청구 결함과 직접 관련된 네 소스 파일이다. `baseline/entry_001`은 수정 전 실제 bytes를 담고, target은 수정 후 source 전체 archive·delta·검증 증거를 담는다. 이전 `f01_admission_001`은 그대로 보존했다.

권장 검토 순서: `CORRECTION_RECORD.ko.md`의 비용 소유권을 코드와 대조하고, `verification/probes.rs`가 원본 두 실패 probe를 어떤 경계 assertion으로 강화했는지 확인한다. `verification/corrected_allocation_probes.log`, `full_tests.log`, `pc_summary.json`, `pc_comparison.json`, `archive_integrity.json`은 각각 allocator 경계, 관련 회귀, 원본 PC 부하, 이전 동결 자료 보존의 증거다. `target/f01_r01_correction_001`의 read-only verifier로 archive와 현재 source의 일치 여부를 다시 검사할 수 있다.

남은 항목: 다른 플랫폼 allocation 실측과 제품 규모 부하 적합성 판단, F02/F03/CAP-01/CAP-02 및 이후 WP. 현재 PC의 추가 5건 거부는 보존된 누적 retained 정책의 결과이며 이번 타깃에서 완화하지 않았다.
