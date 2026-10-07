# U01 실행 기록

2026-10-04. 기존 실행/감사 원본을 유지한 새 교정 pass다.

- 인수: U00에서 과거 감사 source 387개 중 386개 동일·현황만 변경, records 1435와 archives/patches 9개 hash 일치.
- 조사: pending cast 준비 종료 후 active Flow가 생존하는 실제 분기 확인. 정확 action에 한정한 취소와 방출 child 보존을 설계.
- 이후 실제 명령은 `run.py`의 불변 receipt/log에 기록하고 아래 결과를 갱신한다.

## 실제 교정 전 실행

- `before_release_001`: cargo build -p game_core --release --bins --offline --locked, exit0. 기존 경고 15개; rustc 1.98.1. 제품 source 무변경.
- `public_oracle_before_001`: 원 감사의 공개 팩/시나리오 5개를 현재 release CLI로 실행. CLI 실행은 모두 성공했고 계약 oracle은 **2 PASS/3 FAIL, exit1**. 조기 취소/Detach 대조군 PASS, Cancel1500/Cancel500/state-exit의 후속 area6885ms가 남아 FAIL.
- 새 원자료: `evidence/public_before/SUMMARY.json` 및 각 jsonl/stderr. 기존 감사 원자료를 덮어쓰지 않았다. 각 입력 및 binary/probe SHA-256을 연결했다.
- mutation 전 제품 baseline: `baseline/SOURCE_BEFORE.json`의 387개·digest36284fada33057409f61fe201fe0eddca1560136e5f4be026994fbad77c101cb 및 실제 archive.

## 교정 후 실제 검증

| 원자료 | exit | 시험 통과/실패 | 초 |
|---|---:|---:|---:|
| [all_default_002](evidence/all_default_002.json) | 0 | 1684/0 | 111.71 |
| [all_feature_001](evidence/all_feature_001.json) | 0 | 1684/0 | 165.41 |
| [check_001](evidence/check_001.json) | 0 | 0/0 | 19.45 |
| [event_log_check_001](evidence/event_log_check_001.json) | 0 | 0/0 | 19.00 |
| [clippy_001](evidence/clippy_001.json) | 0 | 0/0 | 31.22 |
| [fmt_001](evidence/fmt_001.json) | 0 | 0/0 | 4.37 |
| [release_001](evidence/release_001.json) | 0 | 0/0 | 48.88 |
| [public_oracle_after_001](evidence/public_oracle_after_001.json) | 0 | 0/0 | 5.67 |
| [public_cli_001](evidence/public_cli_001.json) | 0 | 0/0 | 20.53 |
| [public_trace_001](evidence/public_trace_001.json) | 0 | 0/0 | 4.94 |
| [pc_current_001](evidence/pc_current_001.json) | 0 | 1/0 | 185.79 |
| [pc_compare_001](evidence/pc_compare_001.json) | 0 | 0/0 | 1.95 |
| [r05_current_002](evidence/r05_current_002.json) | 101 | 0/0 | 57.69 |
| [r05_compare_002](evidence/r05_compare_002.json) | 0 | 0/0 | 1.95 |

모든 위 명령의 실행 전후 source는 388개·`5b05f2454b83b9b1dc4a4db50b7d4fc066efeca84d319c6095212c5c505ac876`로 동일하다.
기본/feature 각 1684 PASS는 기존 1677개와 추가 7개를 포함하며 제외/ignored는 없다.
중단된 `all_default_001`은 완료 receipt가 없어 PASS로 사용하지 않았고 별도 interruption 기록을 남겼다.
`focused_002`의 1실패는 초기화 이전에 분기한 Debug HashMap 표시 순서 시험을 정정한 것으로, 원 실패를 보존한다.

공개 F01 입력은 before 2 PASS/3 FAIL에서 after **5 PASS/0 FAIL**로 바뀌었다.
다섯 case 모두 marker 도착·정확 projectile HP 시간·후속 area 허용/차단 oracle을 확인했고 trace on/off event가 같다.
공개 9팩45명령은 모두 exit0이며 팩별 trace on/off gameplay·amount 동일성을 확인했다.
R06 31수용→21 integration과 R07 F01–F12의 지정 selector는 양쪽 전체 실행에서 모두 발견되고 통과했다.
55개 Goal 조건과 새 회귀의 연결은 [COVERAGE_CURRENT](COVERAGE_CURRENT.json)이며 의미 판정은 독립 감사에 남긴다.

PC36은 현재 제품 source의 격리 사본에 기존 probe만 부착하여 다시 실행했다. 사본 일치, 계측 변경 2파일과
보조 probe 원자료는 `pc_isolate_current_001.json` 및 hooks archive에 있다. 기존 비교기의 원 기준을 유지했다.
R05 현재 raw 결과는 `warning_concurrent_peak_only`, concurrent peak 초과 `343587` bytes다.
기존 합의의 concurrent peak 단독 경고만 적용하며 allocation count/retained/event-heavy peak/parity/repeatability
기준을 완화하지 않았다. latency gate는 원 계약상 적용하지 않는다. 20분 시험은 수행하지 않았다.
clippy는 기존의 never_loop/eq_op 허용 옵션을 유지했고, GameError 크기 등 경고를 0개라고 주장하지 않는다.

독립 감사는 아직 미수행이다. 이 자체 검증으로 후속 작업의 선행 통과를 선언하지 않는다.
