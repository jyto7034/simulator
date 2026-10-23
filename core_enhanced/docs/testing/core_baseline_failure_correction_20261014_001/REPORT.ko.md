# 기존 Core 실패 138개 교정 결과

2026-10-14 사용자가 요청한 **기존 138개 교정은 완료**했다. 이름 변경·중복 통합·환경 분리를 반영한 현재 139개 대응 사례가 모두 실제 통과했다. [개별 대응표](CORRECTION_MATRIX.json), [실행 명령·환경·종료 코드·원문 색인](EXECUTION_INDEX.json)을 함께 확인한다.

전체 저장소 테스트가 모두 통과한 것은 아니다. 범위를 넓힌 통합 검사에서 기존 138개에 없던 **17개 실패**를 발견했다. 구형 `Standard` 시작 입력 16개와 대검을 기대하는 시작 장비 검사 1개이며, [원문과 목록](INTEGRATION_REMAINDER.json)에 분리했다. 이 17개는 이번 138개 교정 완료 수에 넣지 않았다.

## 변경 내용

| 기존 판정 | 실제 처리 |
| --- | --- |
| 유지·교정 135개 | 현재 준비 계약과 기대값으로 보정하고 본래 assertion까지 실행 |
| 시작 장비 중복 1개 | 고유한 전투 자격·inventory 정합성·snapshot assertion을 U06의 6역할/사본/Continue 검사에 통합한 뒤 중복 본문 제거 |
| Continue preview 1개 | 정상 저장·복원·입장 재계산 경로 유지. 외부 저장 위조 분기는 제거. 정상 fixture에 배율110을 작성해, 생략된 배율의 기본값100과 실제 재계산110을 구별 |
| 환경 의존 1개 | UTF-8/디렉터리와 Unix socket을 분리. socket은 허용된 실행 환경에서 실제 bind와 loader 거절 확인 |

변경은 테스트와 fixture **35개 파일**에 한정된다. 제품 실행 로직·공식 콘텐츠·기본 자원 한도는 변경하지 않았다. [최초 원본](preimages/), [최종 변경 목록과 해시](final_focus_018.inputs.json), [작업 시작 시점 대비 전체 diff](final_focus_018.diff)를 보존했다. 파일명이 제품 모듈인 변경도 모두 `cfg(test)` 안의 assertion/helper다. 통합 빌드를 위해 `tests/common/mod.rs`의 기존 PVE fixture 3곳에 현재 필수 필드의 기본값을 추가했다.

- 외부 RON/Trial은 현재 Expedition과 지원 FacilityLayout을 사용한다. 역할별 시작 무기·파편을 유지하고, 시나리오에 필요한 전투·보상·이벤트를 정상 작성 콘텐츠로 구성했다. 구형 모드로 공식 경로를 대체하지 않았다.
- 고정 조우의 입장 전 은폐를 존중하여 실제 입장 결과에서 배치 지형을 읽는다. 공개 roster의 직원 정체성, Ground 자격, 실제 경로 끝과 사거리에 따라 위치·방향을 준비한다. 원래 수치 및 실제 소비·피해·회복 판정은 유지했다.
- resource lifetime은 역할에 맞는 무기/active 호환성을 준비하고 실제 획득·해제·장착 경로로 사본 소유자를 구분한다. Gate와 실제 방 이동도 각각 실행한다.
- 새 프로세스 Continue는 별도 저장된 권위 있는 플레이어 프로필을 공급한다. 저장 재작성 없이 원래의 사본·이벤트 이력·generation·소유권 경계를 확인한다.
- checkpoint15, schema6, 명령80개, 정의46종 등 현재 명시 계약을 반영했다. unknown-field 시험은 정상 입력 대조군에 잘못된 필드 하나만 추가한다. 본부 고용은 첫 고용 후 세션 유지와 명시적인 종료를 검사한다.
- bounded 전투 로그에서 초기 spawn이 퇴출되는 경우 실제 공개 event delta를 관측한다. fault 시험은 현재 입장을 수용한 뒤 원래 긴 Elapsed 작업에서 실제 fault와 모든 mutation gate를 검증한다. 보호/반격 시험은 자연 종료를 관측할 수 있는 유한 시간 동안 같은 oracle을 유지한다.

## 검증 결과

| 실행 | 실제 결과와 해석 |
| --- | --- |
| `full_core_016` 전체 기본 lib | 2,092 통과 / 4 실패 / 기존 ignore6 / 별도 socket1. 실패4는 G11 구형 기본 manifest 입력 |
| `final_focus_019` | U14에서 이미 사용하던 현재 G11 manifest를 지정한4개와 강화한 Continue1개: **5/5 통과** |
| `socket_supported_017` | Unix socket **1/1 통과**. 샌드박스 EPERM과 기능 검증을 분리 |
| 기본 lib의 고유 사례 합산 | **2,097 통과 / 미해결 실패0 / 기존 ignore6**. 단일 실행 결과로 바꿔 쓰지 않음 |
| `observation_024`, `observation_resources_025` | `runtime-observation`의 G11 **4/4**, 자원 수명 **19/19 통과** |
| 외부 `author_content` CLI | R07/R12 기본프로필6개, U09 기존 작성용 개발프로필6개: **12/12 시나리오 전체 step 실행** 및 amount preview/explain·trace 소비 확인 |
| `integration_021`, `integration_remaining_023` | 8개 통합 target 합계 **56 통과 / 17 실패**. bin target3개는0시험이며 기능 PASS로 계산하지 않음 |

기존 ignore6은 실제 승인 cache·별도 관리 실행을 요구하는 U11 시험이다. 이번 작업에서 ignore를 추가하지 않았다. G11 구형 manifest 실패, 중간 컴파일/명령 실패, 진단 실패는 원문 그대로 남겼으며 통과로 계산하지 않았다. `observation_024`의 잘못된 자원 모듈 필터는 실제로 G11 4개만 실행했으므로, 정확한 모듈을 지정한 `observation_resources_025`의19개 결과를 별도로 사용했다.

U09 확장 콘텐츠는 CLI 기본 작성 작업량10,000,000을 초과했다(`requested=10,000,026`). 기존 `config/runtime_safety_authoring_dev.ron`을 명시한 성공을 기본프로필 성공이나 운영 한도 승인으로 해석하지 않는다. [CLI 검산 결과](CLI_VERIFICATION.json)는 거절 원본1회와 성공12회를 구분하고, 각 입력의 전체 step 수·연속 index·amount 문서 대조·trace 출력을 확인한다.

## 재현과 잔여 범위

작업 디렉터리는 저장소 root, Cargo cache는 `/tmp/wp02-cargo-home`, target은 `.cache/u12-vault-cost-target`이다. 정확한 argv는 `EXECUTION_INDEX.json`, G11 환경은 `final_focus_019.environment.json` 및 `observation_024.environment.json`에 있다. 기본 lib를 한 번에 재현하려면 현재 G11 입력을 지정한다.

```bash
U12_G11_INPUTS="$PWD/core_enhanced/docs/goals/core_expedition_deterrence_rewards/execution/single_goal/test_parts/g11_inputs001/manifest.json" \
CARGO_HOME=/tmp/wp02-cargo-home \
/home/blast/.cargo/bin/cargo test -p game_core --lib --offline --locked \
  --target-dir .cache/u12-vault-cost-target -- \
  --skip game::data::content_pack::snapshot::tests::unix::socket_inputs_return_errors
```

socket은 Unix socket 생성이 허용된 환경에서 해당 exact selector로 별도 실행한다. 전체 통합 결과를 재현할 때는 `--no-fail-fast`를 사용하여 첫 실패 뒤의 target도 실행한다. CLI 입력은 `cli_inputs_016`, 원문 결과와 trace는 실행별 파일에 보존했다.

최종 소스에서 필요한 관련 검증과 자체 코드 검토를 완료했다. 마지막 전체 lib 뒤의 변경은 Continue 단일 시험의 준비/판정 강화와 U10 시험의 들여쓰기이며, 전자는019에서 재검증했다. 나머지 전체 결과는 비영향 재사용한다. 추가 통합17개는 아직 최초 거절 뒤의 assertion까지 검증되지 않았으며 제품 결함으로 단정하지 않는다.

기존 사용자 dirty 변경과 U14 동결·독립 감사·진단 기록은 보존했다. 이번 결과는 정식 U14 재감사나 U15 착수를 뜻하지 않는다.
