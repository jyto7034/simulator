# R06 WP02 CAP-01/02 correction 002 포맷 재검토

## 판정: PASS — 해당 단계검토 범위 한정

대상: `corrections/wp02_cap_01_02_001/target/cap_01_02_correction_002/MANIFEST.json`

SHA-256: `6ee8ea5a889428eef4a0986cd83037ef8c51de6c751868ef9657adb11d3ce789`

직전 검토의 **CAP-R01 [P3] 포맷 지적사항을 해소**한다. 선행 correction 001 기능 검토와 이번 포맷 delta 검토를 합쳐 CAP-01/02 교정 및 action–cast 제품 회귀 추가 구간을 수용한다. 새 기능 결함이나 추가 수정 요구는 없다. 이 판정은 WP02/R06/Phase/Goal 전체 완료나 다음 WP 진입 승인이 아니다.

## 직접 확인

- 인계서와 두 검증 스크립트를 읽고, 쓰기가 없는 `verify_format_target.py`를 실행했다. 대상 hash, source 333개/evidence 12개, 현재 bytes, 선행 archive + delta 복원이 일치했다. 기존 scope verifier는 receipt에 쓰므로 재실행하지 않았다.
- 선행 archive를 별도 `/tmp/r06-cap-format-review.BXS2CH`에 해제하고 333개를 선행 manifest의 hash와 직접 대조했다. 현재 소스와 달라진 파일은 보고된 5개뿐이다.
- 5개 파일의 전체 diff를 직접 읽었다. 변경은 CAP helper·성장 호출·추가 시험 영역의 rustfmt 정리에 한정되며 실행 분기, 상수, 시험 입력/assertion의 의미를 변경하지 않았다.
- **전후 각 파일을 같은 rustfmt 설정으로 출력한 결과가 5개 모두 byte-identical**함을 독립 스크립트로 확인했다. 단순 공백 제거 비교가 아니라 실제 Rust formatter의 출력 비교이며, 위 diff 검토와 함께 의미 보존의 근거로 사용했다.
- 현재 5개 파일의 `rustfmt --edition 2021 --check --color never --config skip_children=true`를 직접 실행했다. 잔여는 `control.rs` 4, `queue.rs` 25, `runtime_control.rs` 6, 합계 35 hunk/exit 1이다. runtime_facts/value_expr_runtime는 전체 파일도 정리됐다.
- 기존 37개보다 2개 적은 것은 CAP가 이미 교체했던 queue 성장 본문까지 포맷된 결과다. 이번 요구는 기존 포맷 부채 전체 제거가 아니라 신규 차이 제거이므로 범위 밖 35개를 이 단계의 재수정 사유로 삼지 않는다. 전체 파일 rustfmt PASS로 표현하지 않는다.

직접 생성한 증거는 `verification.json`, `focused_rustfmt.log`, 파일별 `.diff`, `verify.mjs`에 보존했다. 제품 소스와 기존 동결 자료는 변경하지 않았다.

## 검증 재사용

새 타깃에 묶인 구현자 로그에서 전체 테스트 1,475개(실패 0), all-target check, 기존 allow 조건 Clippy 완료를 확인했다. Clippy는 warning이 남은 상태의 성공이며 warning-free라는 뜻은 아니다.

이번 검토에서는 테스트·check·Clippy를 중복 실행하지 않았다. 포맷 한정 delta에 대한 diff 및 formatter 출력 동등성 확인을 수행하고, 직전 독립 429개 시험 및 새 타깃의 제품 전체 시험 기록을 재사용했다. PC 36조건과 R05 guard도 선행 검증을 재사용하며 새로 측정했다고 주장하지 않는다. R05의 작은 여유와 128 Read 5조건 거부 등 직전 보고서의 한계는 유지된다.

검토 스크립트의 최초 stdin 기반 formatter 호출은 대기 상태가 되어 중단했다. 파일 인자 + stdout 출력 방식으로 바꾸고 timeout을 설정한 최종 실행이 통과했다. 이는 검토 도구 실행 문제이며 제품 시험 실패가 아니다.

## 다음 경계

이 포맷 지적사항으로 다시 구현자에게 교정을 요구할 필요는 없다. 다음 작업은 기존 계획의 WP02 미구현 범위를 확인하고 후속 구현 구간을 지정하는 것이다. 다음 구간 구현이나 완료 선언은 이번 검토에서 수행하지 않았다.
