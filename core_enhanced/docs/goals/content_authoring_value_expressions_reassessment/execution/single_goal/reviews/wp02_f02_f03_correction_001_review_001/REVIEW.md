# R06 WP02 F02/F03 correction 001 단계 재검토

검토일: 2026-09-26 KST.

**전체 판정: CORRECTION_REQUIRED. F03는 이번 교정 범위에서 PASS, F02는 잔여 실행 정보 결속 결함 1건으로 추가 교정 필요.**

대상: `corrections/wp02_f02_f03_001/target/f02_f03_correction_001/MANIFEST.json`.
SHA-256: `84a7b1a6f4d40973c5fbc148c00457911e966a20f046e4aa6afbeac609897b23`.

## F02-R01 — 수락 이후 생성되는 cast를 다른 action에 결합할 수 있음

우선순위 P2. 위치: `core_enhanced/src/game/battle/core/runtime_facts.rs:516`의 accepted cast 비교. 소비 경계: `value_expr_runtime.rs:373` 부근의 `validate_action_source` 호출.

새 검증은 action snapshot에 원래 cast가 있었을 때만 전달 source.cast와 비교한다. 그러나 실제 `sim.rs:513`의 action 수락은 cast 없이 수행되고, 559행 이후 cast 실행 정보를 만들어 573행에서 source에 연결한다. 따라서 일반적인 이 생산 경로에서 accepted snapshot의 cast는 None이며 새 cast 검증을 건너뛴다.

실제 catalog의 Source Attack 식과 `BattleCore::invoke_ability`로 만든 두 정상 실행을 사용했다. action이나 cast를 임의 생성한 것이 아니라 생산자가 만든 source 두 개 중 첫 source의 cast Arc만 두 번째 것으로 바꿨다. actor/action/authored 정보는 첫 실행 그대로 유지했다.

| 추가 시험 | 기대 | 실제 |
| --- | --- | --- |
| 서로 다른 actor의 두 cast: A action에 B cast 결합 | 불일치 거부 | capture 성공 |
| 같은 actor의 두 cast: 첫 action에 두 번째 cast 결합 | 실행 회차 불일치 거부 | capture 성공, cast_seq 0→1 |
| 같은 생산 경로의 변경하지 않은 정상 source | 정상 허용 | 모두 성공; 예상 Damage(10)/Damage(3) |

관련 시험: [additional_probes.rs](additional_probes.rs), [실행 로그](original_and_cast_probes.log), [receipt](original_and_cast_probes.json). 두 실패는 동일한 결속 누락의 재현이며 별개의 finding 두 건이 아니다. 불일치 capture 전후 gameplay state는 그대로였다.

이 결과는 외부 pack의 public publication 성공이나 잘못된 실제 피해 적용을 재현한 것이 아니다. 현재 gate는 닫혀 있고, private adapter의 `capture_value_expr_entry`가 서로 다른 실행의 정보를 하나의 immutable capture로 받아들이는 결함을 확인했다. 원본 actor/equipment 불일치 두 사례 자체는 교정됐다. 이번 finding을 그 두 사례가 여전히 실패한다거나 이번 변경이 새로 도입한 회귀라고 표현하지 않는다.

### 수정 방향

1. action 수락과 cast 생성의 실제 순서를 전제로, cast 생성/연결 시 그 실행을 소유하는 owner가 검증한 **action–cast 결속**을 capture까지 전달·확인한다. accepted snapshot의 None을 뒤에 오는 임의 cast에 대한 허용으로 해석하지 않는다.
2. `cast.caster_instance_id == source.actor`만 추가해서 끝내지 않는다. 같은 actor의 다른 cast_seq/action 조합도 거부해야 한다. skill ID나 값 동등성만으로도 서로 다른 실행을 구별할 수 없다.
3. mutable 전역 registry를 중복 도입하거나 token 공개/직렬화 경계를 넓히지 않는다. 기존 owner의 실행 context 또는 검증된 불변 관계를 우선 활용한다.
4. 현재 active cast 목록만 조회하는 단순 수정은 제어 단계가 끝났으나 delivery가 살아 있는 정상 경로를 깨뜨릴 수 있다. released carrier와 적법한 Snapshot의 실행 정보 수명도 보존한다.
5. cast가 없는 합법적인 basic/non-cast source, 기존 child/proc/delivery, persistent copy 없는 scenario item을 일괄 거부하지 않는다. 효과별 authored metadata 전체 동등 비교로 대체하지 않는다.
6. 위 두 실패 probe, 정상 대조군, delivery 잔존·Snapshot·stale/different attempt 및 기존 actor/provider 교정 회귀를 함께 검증하고 별도 correction target으로 재검토받는다.

## F03 판정 — PASS, 교정 범위 한정

Resource frame 읽기에서 SourceEmployee/SourceEquipment는 Source, TargetEmployee는 Target으로 해석하고 `value_expr_unit` active 검사를 lease 조회보다 먼저 수행한다. 공통 resource mutation resolver와 저장 수명 정책은 변경하지 않았다.

원본 실제 death producer 재현이 이제 MissingContext로 거부되고, 제품 회귀의 active/dead/withdrawn/missing, 세 owner selector, Run/Attempt/Deployment target 차이, Snapshot 값 보존 및 source withdrawal 이후 released Snapshot 소비가 통과했다. resource store/finite lifetime 회귀도 통과했다.

SourceEquipment의 Run/Deployment 전체 조합과 제품 producer에 의한 missing unit 제거는 별도 실측하지 않았다. missing 테스트의 내부 unit 제거를 제품 lifecycle producer 검증으로 확대하지 않는다. 이는 이번 코드 경계의 검토 PASS이지 모든 lifecycle 조합/후속 consumer의 완료 판정이 아니다.

## 직접 실행과 재사용

동결 archive를 `/tmp/r06-f02-f03-review.5nvNWT`에 복원했다. 제품 코드 로직은 바꾸지 않고 기존 runtime tests 안에 reviewer 모듈 hook 하나만 추가했으며, 별도 target 디렉터리에서 offline release로 빌드했다.

| 직접 실행 | 결과 | 증거 |
| --- | --- | --- |
| 원본 감사 probe 6개, bytes 그대로 | 6 PASS | original_and_cast_probes.log |
| 새 실제 cast 생산자 probe 3개 | 정상 대조군 1 PASS / 불일치 2 FAIL | 같은 로그 |
| 기존 adapter 회귀 | 16 PASS | adapter_regressions.log |
| ValueExpr kernel/catalog/parser 회귀 | 37 PASS | kernel_regressions.log |
| runtime facts 회귀 | 11 PASS | facts_regressions.log |
| resource store/lifetime 회귀 | 27 PASS | resource_regressions.log |

총 100개 중 98 PASS / 2 FAIL이며 실패 둘은 F02-R01 하나다. 모든 명령·선택자·시각·exit·probe 해시는 각 JSON receipt에 있다. 기존 `ResourceStore::apply_operations` dead-code warning을 보존했다.

구현자의 36/36은 `review_` selector 전체 결과로, 원본 이번 WP02 probe는 6개다. 나머지 review 이름의 기존 테스트를 새 F02/F03 재현 30개로 오인하지 않는다. 구현자의 full 1,469 tests, check/rustfmt/Clippy, F01-R01 allocator 7개는 보존된 자체 검증 증거이며 이번에 모두 다시 실행한 것은 아니다. PC 36조건, R05 별도 allocation guard, 다른 플랫폼 실측은 이번에 실행하지 않았다.

## 자료 보존과 권한

333개 source의 currentness, evidence archive, before→delta→after 복원과 이전 F01-R01 archive 해시를 검증한다. 격리 소스는 reviewer hook을 제거하면 333개 모두 manifest와 일치한다. 원본 감사 probe를 복사한 파일의 byte 동일성도 검증한다. 종료 검증 및 새 review 파일 해시는 `verification.json`에 기록한다.

제품 코드·기존 baseline·target·증거는 수정하지 않았다. 검토자는 앞선 교정 방향에 참여한 같은 대화의 reviewer이며 blind/fresh GEX 최종 감사라고 주장하지 않는다.

종료 상태: **F03 부분 교정 PASS / F02-R01 교정 필요 / CAP-01·CAP-02 및 다음 WP 미착수 유지**. 이번 검토는 후속 제품 변경이나 WP/Phase/Goal 완료를 자동 승인하지 않는다.
