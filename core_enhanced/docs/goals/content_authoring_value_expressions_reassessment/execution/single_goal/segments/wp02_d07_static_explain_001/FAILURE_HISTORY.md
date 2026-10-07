# R06 WP02 D07-A 실패·환경 이력

2026-09-27 기준이다. 성공 결과만 남기지 않고 구현·검증 중 관찰한 실패를 함께 보존한다.

## 구현 시험 실패와 교정

1. `evidence/authoring_focused.json`은 source digest
   `82892b94058e364a98bc8aadb5ce1d3926c14f9c8a342f8a4cdf60028467388f`에서
   109 passed / 1 failed였다. 새 `TrialSession::explain` 위임 검사가 외부 resolution metadata가 없는
   embedded token으로 `ExplainQuery::Effect`를 호출해 `authoring_source_unavailable`을 받았다.
   실제 외부 official fixture를 사용하도록 시험을 고쳤다. 최종 digest
   `5279b8333c6be8f90992ac94a47f09beb083bf82bfcdf02ebbe3e0d179c8f530`에서
   `trial_delegation_002` 1/1 및 `authoring_focused_final_001` 110/110이 통과했다.
2. `trial_delegation_001`은 임시 Cargo cache가 정리된 뒤 offline `rand` package를 찾지 못해
   시험 실행 전 exit 101이었다. `cargo fetch --locked`로 같은 lock의 의존성을 복구한 뒤 재실행했다.
3. `cli_effect_explain_final_001`은 예시로 지정했던 오래된 `CLI_EXTERNAL_004` fixture의 contract가
   `catalog_schema=2, execution=1`이고 현재 요구가 `3, 2`여서 exit 1이었다. 현재 소스의
   `package_official_content`로 contract `3,2` fixture를 만들고
   `cli_effect_explain_current_final_001`을 실행해 exit 0을 확인했다.

## 실행 도구와 환경 실패

1. 중단된 최초 `--all-targets -- --test-threads=1` 실행은 래퍼가 child output을 종료 시점까지
   보관했기 때문에 완료 영수증과 부분 로그가 남지 않았다. SIGBUS 메시지는 관찰되지 않았다.
   `reviews/wp02_d07_test_timing_001/README.ko.md`가 같은 source digest의 기본 병렬 실행을 재현해
   1,533 passed / 0 failed를 확인했다. 래퍼를 즉시 log 생성·streaming·20초 상태 출력·중단 시
   process group 종료와 `.interrupted.json` 기록 방식으로 바꿨고, 최종 자체 실행도 1,533/1,533이다.
2. 위 시간 확인의 첫 sandbox 시도는 `UnixListener::bind`의 EPERM으로 1개가 실패했다. 승인 환경의
   재실행은 통과했으며 두 실행 모두 제품 source digest가 같았다. 원자료는 해당 review 디렉터리에 있다.
3. `format.json`, `format_final_003`은 `check_format.py`의 cwd를 잘못 지정해 exit 2였다. 소스 검사가
   실행되기 전 실패했으며, 올바른 cwd의 `format_final_004`가 최종 source에서 통과했다.
4. 격리 cost probe 준비 중 복사한 별도 lock이 현재 offline cache와 달라
   `probe_static_explain_usage_final_001`은 `zmij` 부재, `_002`는 lock 갱신 금지로 시험 실행 전
   exit 101이었다. 제품 workspace `Cargo.lock`의 해를 격리 probe lock에 반영한 뒤 `_003`이 통과했고,
   probe source를 rustfmt한 동일 의미 최종 `_004`도 통과했다. probe target은 제품 target과 분리했다.

## 관찰 결론

- 이번 기록 어디에도 SIGBUS 또는 bus error가 없다.
- 최종 제품 source digest의 필수 check/test/clippy/CLI는 모두 exit 0이다.
- 실패를 숨기기 위한 0 반환, 자동 제외, 첫 장비 선택, publication 차단 해제는 넣지 않았다.
