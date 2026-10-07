# D07-B 실패·재시도 기록

모든 실패 원본은 `logs/`와 `evidence/cli_fixture/`에 그대로 보존한다. 성공하지 않은 실행을 최종 영수증으로 사용하지 않았다.

1. `d07_b_focused_001`은 staged fixture의 잘못된 effect index와 legacy 음수 소비 assertion 때문에 2/4 실패했다. index를 실제 ordered effect 주소에 맞추고 legacy `unsigned_abs` 소비를 직접 확인하도록 고쳤다. `focused_002`의 남은 예상 HP 차이를 실제 두 effect 합계로 고친 뒤 `focused_004`가 당시 5/5 통과했다.
2. 공개 `TrialSession` 경로는 `d07_b_public_trial_001`–`025`에서 현재 실제 게임 흐름을 단계적으로 발견했다. 자동 basic attack에는 authored catalog record가 없었고, baseline fragment 직접 소유는 persistent loadout 검증에 거부됐다. combat 종료 시각·배치 자원·map 연결·reward 공개 shape·다음 combat의 적 생존·resonance 준비 상태도 고정 가정할 수 없었다. 최종 fixture는 별도 정상 fragment와 reward를 외부 pack에 추가하고, 공개 result binding으로 reward/combat node와 유닛을 선택하며 실제 두 번째 전투·수동 시전·projectile impact를 수행한다. `public_trial_026`이 1/1 통과했고 이 시험은 이후 집중/전체 회귀에 포함됐다.
3. V17 Live 두 query 원자료를 추가한 `d07_b_focused_005`는 `control::run`의 중첩 Result 해체와 JSON/typed 값 비교 타입 오류로 컴파일되지 않았다. Result를 명시적으로 해체하고 typed 값을 bounded serde 투영으로 비교해 `focused_006` 5/5를 통과시켰다.
4. 첫 CLI fixture는 선행 구형 contract pack을 사용해 `content_contract_unsupported`로 exit 1이었다(`output.ndjson`, `final_cli_001.stderr`). 현재 contract v3/execution2 pack을 별도 fixture로 만들었고 최종 `output_004`/`trace_004`와 trace-off `output_005`가 exit 0이다. 중복·unknown·validate 전용 옵션은 최종 source에서 모두 exit 2를 유지했다.
5. 첫 PC36 명령은 보존 fixture의 private hook을 주입하지 않은 작업 트리에서 selector 0건으로 exit 0이었다(`final_pc36_001`). 이를 증거로 사용하지 않았다. exact source를 격리 복사하고 보존 hook 두 개와 probe만 넣은 `final_pc36_002`가 pre-final source에서, `final_pc36_003`이 최종 340개 source에서 각각 1/1 통과했다. 제품 작업 트리는 계측하지 않았다.
6. 첫 R05 실행은 offline cache에 잠긴 `zmij`가 없어 workload 전에 exit 101이었다(`final_r05_001`). 동일 tool manifest를 `cargo fetch --locked`로 채운 뒤 offline 재실행했다. pre-final source `final_r05_002`는 원 guard까지 통과했으나 Live 증거 추가 뒤 최종 source를 다시 측정했다. `final_r05_003`은 concurrent peak만 원 cap을 30,846 bytes 초과해 raw exit 101이며, 사용자 승인 정책에 따라 이 항목만 warning으로 분류했다. event-heavy, parity, repeatability, retained, latency와 다른 hard failure는 통과했다.
7. 공개 시험 fixture 탐색 과정의 panic과 0건 selector는 제품 panic 또는 최종 회귀 실패로 분류하지 않았다. 최종 source 명령은 format/check, authoring 112, value_expr 91, 기본 병렬 전체 1,538, clippy가 모두 exit 0이다.
