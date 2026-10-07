# R06 WP02 F02-R01 correction 002 단계 재검토

검토일: 2026-09-26 KST. 판정: **PASS — F02-R01 교정 범위 한정**.

대상: `corrections/wp02_f02_r01_001/target/f02_r01_correction_002/MANIFEST.json`.
SHA-256: `ae2c540157c9caa769febb56e2664da7be8b5807b917e4ecfcb685103aa5f0f7`.

이전 검토에서 확인한 늦게 생성된 cast의 action 결속 누락은 교정됐다. 이번 검토 범위에서 새로운 수정 요구 사항은 발견하지 못했다. 기존 F03 부분 교정 PASS를 유지한다. WP02/Phase/Goal 완료나 publication·GEX 최종 감사 PASS를 선언하는 것은 아니다.

## 코드 검토

- `sim.rs`는 실제 cast 실행 객체를 생성한 직후, source/carrier에 전달하기 전에 `runtime_facts.bind_cast`를 호출한다. 결속 실패는 기존 native fault 경계로 전파된다.
- `ActiveAction.bound_cast`는 기존 action owner 내부의 private 관계다. 별도 전역 registry, 공개 token 또는 직렬화 필드를 만들지 않았다. 유효 action, actor, skill 및 미결속 상태를 확인한 뒤 한 번 설정한다.
- `validate_action_source`는 bound cast와 전달 cast의 Arc 동일성을 검사한다. 다른 actor의 cast, 같은 actor의 다른 실행, 같은 데이터만 복제한 새 객체, 이미 결속된 source에서 cast를 누락하는 경우를 거부한다. 합법적인 cast 없는 source는 유지된다.
- 관계가 active cast controller가 아니라 action에 보관되므로 controller 정리 후에도 살아 있는 delivery의 capture가 가능하다. action 정리 시 소유 관계가 함께 정리되는 경로와 기존 extent 감소를 확인했다.
- 새 필드가 CloneShape에 포함되며 COW 시 Arc 관계를 유지한다. bind 보유 extent가 기존 retained ledger에 포함된다. 부족한 예산의 실패에서는 결속이 설정되지 않고 이후 정상 operation에서 재시도된다.
- 기존 actor/provider 검사, effect별 authored metadata 차이, F03의 frame 읽기 가용성 검사와 공통 resource 수명 정책은 변경하지 않았다.

## 직접 실행한 시험

동결 after archive를 `/tmp/r06-f02-r01-audit.SDwxhw`에 복원하고 `value_expr_runtime.rs`의 기존 private tests에 reviewer 모듈 hook 1개만 추가했다. 제품 로직은 변경하지 않았으며 별도 target 디렉터리의 offline/locked release 빌드를 사용했다.

| 검증 | 결과 | 로그 |
| --- | --- | --- |
| 원본 reviewer probe 9 + 구현자 추가 probe 3 + 이번 추가 probe 3 | 15/15 PASS | binding_probes.log |
| 기존 ValueExpr adapter | 16/16 PASS | adapter_regressions.log |
| kernel/catalog/parser | 37/37 PASS | kernel_regressions.log |
| runtime facts | 11/11 PASS | facts_regressions.log |
| resource store/lifetime | 27/27 PASS | resource_regressions.log |
| released 관련 실제 projectile/area/withdraw 회귀 | 16/16 PASS | released_regressions.log |

released selector의 3건은 앞선 adapter/facts/resource selector와 중복된다. 중복 제외 직접 실행 테스트는 **119개 모두 PASS**다. 기존 `ResourceStore::apply_operations` dead-code warning은 로그에 보존했다.

이번 추가 probe가 확인한 사항:

1. cast 누락은 거부한다. 기존 action map을 공유하는 후보를 수정해 COW를 발생시킨 뒤에도 정상 capture는 성공하고, 원본 후보 상태는 변하지 않는다.
2. 잘못된 actor/skill 결속과 재결속은 거부한다. 후보에 설정한 결속은 원본 owner에 새로 생기지 않는다.
3. 잔여 retained 1 byte에서 결속을 거부하고 owner 상태를 유지한다. 정상 예산의 다음 operation에서는 결속 및 capture가 성공한다.

원본 두 cast 바꿔치기 assertion은 수정하지 않고 재실행해 모두 거부됨을 확인했다. 구현자 probe의 controller 제거는 직접 구성한 경계시험이라는 제한을 유지한다. 별도로 실제 released area/projectile 수명 회귀를 실행했지만 모든 조합의 expression publication을 검증한 것은 아니다.

## 증거·복원 검증

333개 source의 현재 bytes와 after archive, baseline→두 파일 delta→after 복원을 확인한다. 격리 사본은 hook을 제외하면 333개 모두 manifest와 일치한다. 제품 source/test/data 경로의 manifest 외 파일 여부도 확인한다. 재사용 probe 3개 파일은 각각 원본과 byte-identical이다.

before archive는 338항목/337고유 경로이며 중복된 계획 문서의 bytes가 같다. 최종 verifier는 before archive의 동일 bytes 중복만 허용하고 상이한 중복은 거부한다. 첫 후보 manifest와 실패 기록을 보존한 상태에서 최종 002만 검토 대상으로 사용했다. 이전 타깃 source/evidence archive 해시도 대조한다. 종료 currentness와 새 reviewer 산출물 해시는 `verification.json`에 기록한다.

구현자의 full all-target 1,469건(lib 1,417 + integration 52), all-target check/Clippy는 기록을 검토했으며 전체를 이번에 다시 실행한 것은 아니다. focused rustfmt는 runtime_facts PASS, sim의 기존 13개 차이 유지로 보고돼 있으며 전체 포맷 PASS로 확대하지 않는다. 최초 release 시험 구성 오류와 sandbox Unix socket 실패는 성공 기록과 분리돼 보존돼 있다.

PC 36조건, CAP-01/02, R05 별도 allocation guard, 다른 플랫폼, 외부 pack publication은 이번에 검증하지 않았다. 메모리 한도의 제품 적합성이나 전체 수백 적 공세 성능을 이번 PASS로 승인하지 않는다.

## 종료와 다음 범위

제품 코드·기존 baseline·target·증거를 수정하지 않았다. 새 reviewer 디렉터리와 격리 테스트 사본만 만들었다. 검토자는 앞선 설계 방향에 참여한 동일 대화의 reviewer이므로 blind/fresh GEX 최종 감사라고 주장하지 않는다.

**F02-R01 부분 단계검토 PASS / F03 교정 유지 / 후속 구현 미착수.** 다음 권장 작업은 기존 계획에 따른 CAP-01·CAP-02의 bounded 용량 성장 교정과 PC 부하 재측정이다. 실행은 사용자의 별도 요청으로 진행하며 WP02 완료나 다음 WP 진입을 자동 승인하지 않는다.
