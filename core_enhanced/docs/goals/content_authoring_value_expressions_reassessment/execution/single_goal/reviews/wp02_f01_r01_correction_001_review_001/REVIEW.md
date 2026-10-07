# WP02 F01-R01 correction 001 단계 재검토

검토일: 2026-09-26. 판정: **PASS — F01-R01 교정 범위 한정**.

동결 대상: `corrections/wp02_f01_r01_001/target/f01_r01_correction_001/MANIFEST.json`.
SHA-256: `ba25a8fbedfeb4b43c13d8f3bce88340e3ffafc4eb6dba2424169028bad7be98`.

이전 `f01_admission_001`의 F01-R01(Live frame의 첫 BTreeMap 할당 과소 청구)은 이번 타깃에서 교정됐다고 판단한다. 검토 범위에서 새로운 수정 요구 사항은 발견하지 못했다. 이는 F01 전체 또는 WP02/Phase/Goal 완료, public publication, 독립 GEX 최종 감사 PASS를 의미하지 않는다.

## 코드 검토 결과

- `src/game/battle/core/value_expr_runtime.rs`의 `frame_storage_bound`가 member-scoped 참조를 제외한 개수와 동적 payload를 checked arithmetic으로 계산한다. Live는 전체 금액을 먼저 승인한 뒤 `with_value_capacity`를 호출한다. 기존의 참조별 청구 뒤 BTreeMap 노드가 임의 크기로 할당되던 경로는 없어졌다.
- `src/game/value_expr/mod.rs`의 frame은 정렬된 Vec다. compiler가 BTreeSet에서 만든 정렬·중복 제거된 requirements를 순서대로 삽입하므로 현재 BattleCore 생성 경로에서는 뒤쪽 추가가 된다. 일반적인 역순 삽입도 정렬을 보존한다. 조회는 이진 탐색이고, 중복 key는 값을 교체한다. logical limit를 넘는 새 key는 삽입 전에 거부한다. 현재 caller는 같은 count로 용량을 확보하고 limit를 넘겨주므로 숨은 성장 할당이 발생하지 않는다.
- `Vec::with_capacity(n)`의 allocator 요청은 n개 요소에 필요한 크기이며, 충분한 capacity 내 insert는 재할당하지 않는다는 표준 보증과 맞는다. allocator가 내부적으로 더 큰 공간을 제공할 가능성까지 RSS 상한으로 주장하는 것은 아니다. [Rust Vec Guarantees](https://doc.rust-lang.org/std/vec/struct.Vec.html#guarantees).
- resource 문자열 경로의 lease key, value unit, frame key, unit 검증용 복제가 동적 문자열 4배 항목에 포함된다. 고정 여유 256과 기존 전역 예산은 유지됐다. 합리적 상한 근거와 allocator 요청량을 구분한다.
- Snapshot의 `capture_extent`에는 frame buffer 및 문자열 몫이 포함된다. 평가 시 frame을 다시 만들거나 청구하지 않고 evaluator 신규 저장 비용만 청구한다. 반복 Snapshot 평가의 정확한 잔여 예산 시험으로도 확인했다.
- evaluator 변경은 frame 조회 API 교체이며, 계산 순서·rounding·narrowing·fingerprint 의미를 바꾸지 않는다. pure kernel의 외부 입력 frame 생성 API와 BattleCore의 bounded 생성 경계를 구분한다.

## 직접 실행한 검증

동결 source archive를 `/tmp/r06-f01-r01-review.d7WUvr`에 복원하고 기존 runtime tests 내부에 reviewer 모듈 hook 1개만 추가했다. 별도 target 디렉터리의 offline release 빌드로 검증했다. 제품 workspace에는 변경하지 않았다.

구현자가 보강한 4개 probe를 그대로 보존한 뒤 reviewer 시험 3개를 추가했다. 합계 7/7 통과:

| 검증 | 선청구/허용량 | 실제 신규 allocation 요청 합계 | 결과 |
| --- | ---: | ---: | --- |
| 단일 Live frame | 352 | 96 | 성공; 한 byte 부족하면 frame 버퍼 할당 전 거부 |
| 전체 Live 평가 | 1,120 | 736 | Damage(10), frame 예산 부족 시 버퍼 할당 전 거부 |
| 문자열 key Live frame | 776 | 520 | 성공 |
| 단일 stat Snapshot capture | 3,755 | 538 | 성공 |
| 위 Snapshot 평가 3회, 하나의 operation | 2,304 | 1,920 | evaluator 비용만으로 모두 성공 |
| 문자열 resource 6참조 Snapshot capture | 19,067 | 3,634 | capture 성공, frame 값 합계 28 |

추가로 0/1/2/11/12/32/128개 참조를 역순 삽입하고 모든 값을 조회했다. 각 크기에서 실제 요청은 상한 이내였고, 승인 개수를 넘는 새 참조 삽입 거부 시 allocation은 0이었다. 128개 조건은 99,328 청구/52,992 요청이었다. 기존 두 참조 반복 시험도 통과했다. 이는 컨테이너의 cardinality 검증이지 수백 명 전투 부하 시험은 아니다.

- ValueExpr 기존 회귀: 37/37 통과.
- BattleCore ValueExpr adapter 기존 회귀: 10/10 통과. Snapshot/Live 분리, unit/context/resource, native work, 취소 및 narrowing 포함.
- 서로 다른 직접 실행 테스트는 총 54개(기존 47 + probe 7)다. adapter selector에서 probe 7개가 다시 실행됐지만 중복 집계하지 않는다.
- 로그/명령/실행 시각/exit: `allocation_and_snapshot.{log,json}`, `kernel_regressions.{log,json}`, `adapter_regressions.{log,json}`.
- release 빌드의 기존 `ResourceStore::apply_operations` dead-code warning은 보존했다. 검증 실패는 없었다.

## 재사용한 증거와 판정 제한

구현자의 full all-target 로그는 lib 1,411 + integration 52 = 1,463건 통과이며 이번에 전체 재실행하지 않았다. PC 36조건도 원본 보존 결과·비교 기록을 검토했으며 직접 재측정하지 않았다. 기록은 이전 F01과 같은 25완료/11거부, 같은 거부 site/native work를 보인다. Live adapter 조건의 allocation 감소는 개선이나, 누적 예산 정책의 제품 적합성을 증명하지 않는다.

R05 별도 allocation guard와 다른 OS/allocator 실측은 이번에 수행하지 않았다. 기본/최대 예산, 누적 청구·미환급 정책을 완화하지 않았다. F02/F03/CAP-01/CAP-02 및 이후 WP는 이번 판정으로 닫지 않는다. 다음 작업은 기존 교정 계획의 미해결 항목에 대한 별도 구현 요청이다.

검토자는 앞선 수정 방향에 참여한 같은 대화의 reviewer다. 구현자 자체 검증과 별도로 코드 검토·추가 시험을 수행했지만, blind/fresh 최종 감사라고 주장하지 않는다.

## 무결성과 종료 상태

328개 source를 manifest와 대조하고 격리 소스는 hook을 제외하면 모두 일치함을 확인한다. 새 baseline→delta→after 복원, 현재 source/evidence, 이전 동결 타깃 source/evidence archive 해시도 검증한다. 종료 receipt와 새 reviewer 증거 해시는 `verification.json`에 보존한다. 기존 baseline·target·제품 파일은 수정하지 않는다.

종료: **F01-R01 단계 재검토 PASS / 나머지 교정 미완료 / 후속 구현 미착수**.
