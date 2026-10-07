# D07-B F01 수행 이력

## 2026-09-28 · 인수/기준선

교정 지시서, D07-B 리뷰 F01–F03, 선행 실패 이력, 실제 parser/diagnostic 종료 경로를 확인했다.
새 실행 디렉터리가 없음을 확인한 뒤 생성했다. predecessor freeze.py verify 통과.
340개 실제 원본 archive, Git HEAD/branch/status와 새 시험 경로 부재를 baseline에 보존했다.
기존 streaming run.py를 이번 디렉터리로 복사해 사용한다. 시험 추가 전 파일 부재도 identity로 기록하도록 조정했다.
이 시점에는 제품 코드를 수정하지 않았다.

## 교정 전 회귀와 최소 수정

실제 CLI integration 시험 4개를 추가하고 f01_before_regression_001을 실행했다.
3개 통과/1개 실패, exit 101. 실패는 simulate의 REQUEST 누락에서 기대 exit 2 대신 실제 panic/101이었다.
원자료는 evidence/f01_before_regression_001.json 및 .log, 당시 시험 source는 BEFORE_REGRESSION_TEST.rs다.
첫 복합 patch는 기록 파일의 잘못된 문맥 때문에 적용 전 거부됐다. source 무변경을 확인하고 올바른 문맥으로 다시 적용했다.
제품 교정은 필수 positional 최소 개수 검사 한 조건 추가로 한정한다. 비simulate exact 길이와 옵션 처리는 유지한다.

## 교정 후 회귀·정상 CLI

f01_after_regression_001: 4/4 통과, exit 0. 네 시험은 실제 subprocess 30조건을 검사한다.
f01_cli_matrix_001: 실제 simulate plain/trace/amount/양 순서 조합과 validate/describe/explain 총8명령 exit 0.
plain과 trace-only 출력 동일, amount의 trace off/on·옵션 순서 출력 동일, D07-B 동결 amount 출력과 byte 동일.
f01_check_001 및 f01_format_001: exit 0. 각 명령/환경/로그 SHA/source 전후는 evidence의 동명 JSON에 있다.
현재 baseline 340개 중 변경은 author_content.rs 1개뿐이며 core src/tests의 추가 membership은 신규 integration test 1개다.
이 결과를 근거로 F01을 첫 단계검토 지점으로 선택한다. F02/F03 코드에는 착수하지 않는다.

## 최종 검증과 재사용 판정

f01_clippy_001 exit 0. 기존 경고 원문은 보존했으며 자동 fix는 수행하지 않았다.
최종 fresh 검증5종의 source 전후 digest와 log SHA를 대조했다.
predecessor 340개 중 CLI 한 파일만 변경, 나머지339개가 byte 동일하다. 전체/집중 runtime 시험·PC36/R05
원자료 SHA를 다시 확인하고 해당 라이브러리 근거로만 재사용했다.
evidence/EVIDENCE_REUSE.json과 FINAL_VALIDATION.json에 새 실행/재사용·미완료 F02/F03을 구별했다.
최종 기록 갱신의 복합 patch도 잘못된 문맥 한 줄 때문에 적용 전 거부돼, 그 줄을 제거하고 다시 적용했다.
두 patch 도구 오류는 제품 검증 실패와 구분하며 제품 소스 변경을 만들지 않았다.
최종 인계 준비 후 freeze.py freeze/verify로 source/test membership과 격리 양방향 복원을 검증한다.

## 동결 결과 파일 자기 포함 오류와 재시도

첫 freeze 내부 currentness/양방향 복원은 통과했지만, 직후 별도 verify에서 FREEZE_RESULT.json의 SHA 불일치를 검출했다.
shell stdout redirection이 freeze 전에 빈 결과 파일을 만들었고 수집 뒤 그 파일에 출력을 썼기 때문이다.
VERIFY_RESULT.json은 빈 파일로 남았으며 뒤이은 결과 표시도 JSON 파싱에 실패했다.
해당 후보 target 전체, 결과 파일, 교정 전 freeze.py를 evidence/failed_freeze_001에 이동·보존했다.
기존 후보를 덮어쓰지 않았고 제품 소스와 시험 결과는 변하지 않았다.
freeze.py의 local record 수집에서 루트의 FREEZE_RESULT.json/VERIFY_RESULT.json을 제외하고,
재시도는 stdout을 직접 도구에 출력한다. 실패 후보도 최종 evidence archive에 원본 그대로 결속한다.
최종 target 유효성은 새 freeze와 별도 verify 결과를 기준으로 판단한다.
