# R06 WP02 EEP-F01~F03 교정 인계

상태: **교정·자체 검증 종료 / correction_001 동결 / 독립 재검토 요청 대기**.

검토 대상은 `target/correction_001/MANIFEST.json`이다. 구현자가 자체 검증했으며, 독립 재검토 PASS나 WP02/R06/Phase/Goal 완료를 선언하지 않는다. 다음 WP 및 publication 해제는 수행하지 않았다.

## 교정 범위

- EEP-F01: release target 임시 복사 제거. resource ID의 slots·문자열을 clone 전에 선청구하고 in-place 정렬·중복 제거. effect ID slots, action-site 문자열, prefix 및 평가 결과 버퍼의 비용 소유권 교정. 예산 상한 유지.
- EEP-F02: `resolved_target_count`는 delivery consumption당 한 번 기록. effect 결과 및 별도 delivery의 합산은 기존 의미를 유지.
- EEP-F03: delivery-local 결과를 `CurrentActionPrefix`로 반환하지 않는다. 앞 frontier가 없으면 `NotSettled`, 완료된 local frontier만 있으면 `Unavailable`. 지원 가능한 `EarlierEffect`는 exact preceding frontier를 검증한다. action-wide witness 구현은 이번 범위 밖이다.

제품 변경은 `src/game/battle/core/value_expr_runtime.rs`, `src/game/battle/core/sim.rs` 두 파일뿐이다. 상세 판단과 비용 표는 [교정 기록](CORRECTION_RECORD.ko.md)에 있다.

## 검증 결과

| 직접 실행 | 결과 |
| --- | --- |
| all-target 제품 시험 | **1,487/1,487**: lib 1,435 + integration 52 |
| 기존 감사 probe | 5/5 |
| 추가 buffer/한도 경계 probe | 2/2 |
| all-target check | exit 0 |
| Clippy | 기존 `never_loop`, `eq_op` allow만 적용, exit 0; 기존 warnings 존재 |
| 변경 영역 rustfmt | PASS; sim의 기존 범위 밖 포맷 차이는 유지 |
| PC 동일 fixture | **31완료 / 기존 128-Read 거부 5개**, 거부 조건·오류 동일 |
| R05 allocation guard | PASS; concurrent peak **8,945,386 / 9,030,188 bytes**, 여유 84,802 bytes |

512명 capture의 추가 allowance 6,000 bytes에서 최대 요청은 이전 8,704 → **4,096 bytes**, 관측 요청 합계는 13,296 → **4,568 bytes**였다. 이는 작은 예산 경계의 선청구 시험이지 PC 메모리 상한 권고가 아니다. prefix 시험은 필요 2,296 bytes에서 성공, 2,295 bytes에서 **0 allocation**으로 거부했다. capture 전체 청구 경계도 12,249 성공 / 12,248 거부를 확인했다.

실행·실패·재사용 구분은 [EXPERIMENTS.md](EXPERIMENTS.md). 성공 receipt의 시작/종료 source hashes가 최종 333개 source와 일치함을 확인했다. 격리 시험은 정확히 두 test-only hook 외 제품 bytes가 일치한다. 기존 감사 다섯 시험의 본문은 유지했고 allocator 계측만 PC observer와 공유했다.

## 재검토 절차와 남은 범위

1. `python3 .../corrections/wp02_eep_f01_f03_001/freeze.py verify`로 currentness, evidence hashes, 양방향 delta 복원을 확인한다. 이 명령은 제품·기존 동결 자료를 수정하지 않고 임시 디렉터리에서 복원한다.
2. `SOURCE_DELTA.patch`와 위 두 제품 파일을 직접 검토한다. 비용 소유권 표와 `evidence/review_probes.rs`의 경계 시험을 함께 확인한다.
3. 필요하면 새 격리 복사에서 probe를 재실행한다. 기존 baseline/target/실행 로그는 덮어쓰지 않는다.

cross-step/cross-delivery action-wide witness, release 이전 완료 결과 Snapshot 연결, ResolvedTargets aggregate는 여전히 미구현이다. `CurrentActionPrefix`는 단일 delivery처럼 보이는 경우도 완전성 보증이 없어 거부한다. 기존 유효 0의 EarlierEffect, 실제 HP 결과, source/action/delivery 결속, candidate rollback은 유지한다.

PC 측정은 기존 36조건 fixture 비교이지 새로운 순차 expression 조합 전체의 성능 보증이나 Windows 실측이 아니다. 많은 resource ID의 prefix 버퍼는 별도 시험했으나, 실제 외부 pack에서 많은 resource ID를 수집·중복 제거하는 조합 전체를 allocator probe로 별도 재현하지는 않았다. 해당 수집 경로는 코드·checked 상계로 검토했다. 타이밍은 공유 실행 환경 및 allocator hook 영향이 있으므로 PC 최소사양/프레임 예산 합격선으로 해석하지 않는다.
