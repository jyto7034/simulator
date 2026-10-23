# R06 교정 전 PC 부하 측정 — baseline 001

측정일: 2026-09-26 KST  
상태: **수정 전 기준 측정 완료 / 제품 교정 없음 / 안전성·성능 합격 판정 아님**

## 결론

수식 평가만으로 대규모 전투 수용력을 판단하면 안 된다. 이번 제한된 시험에서 간단한 수식 반복/집계는 처리됐으나, **다수 대상의 실제 피해 기록과 동시 생성의 누적 allocation admission**에서 먼저 한도가 드러났다. 기존 F01(호스트 잔여 예산의 사전 연결 누락)은 그대로 남아 있으므로 현재 evaluator 성공을 안전성 증거로 사용할 수 없다.

- 36개 조건, 조건별 시간 11회·할당량 3회 측정. 각 모드별 첫 1회는 warmup으로 제외했다.
- raw 576행 = 측정 504행 + warmup 72행. 측정 504행 중 84행은 의도한 작업량을 완료하기 전에 typed limit fault로 거부됐다.
- 조건 기준으로 **30개 완료 / 6개 거부**. harness 자체의 `1 test passed`는 결과 수집 완료이지 모든 workload PASS가 아니다.
- 현재 작업 폴더 입력 294개는 진입 대비 SHA-256 불변. 임시 사본만 private 측정 모듈 연결과 read-only 예산 조회 함수를 추가했다. 제품·기존 동결 타깃·예산 숫자는 수정하지 않았다.

## 1. 환경과 측정 범위

[조사 및 계획](../R06_PC_WORKLOAD_AND_BUDGET_AUDIT_20260926.ko.md)에 따른 첫 축소 baseline이다. W01/W02/W03/W05/W07의 일부만 다뤘으며 전체 매트릭스 완료가 아니다.

- AMD Ryzen 9 7950X, x86_64 Linux/WSL2, Cargo release optimized lib-test binary. CPU affinity 미고정. 최소 PC 사양이나 Windows native 측정이 아니다.
- 실제 환경·Rust 버전: [environment.json](environment.json). 프로세스 단위 RSS/동시 live heap peak는 이번에 측정하지 않았다.
- seed 17, 25×25 타일은 fixture 가정. 아군 12명, 적 100/300/500명. 제품 전장 크기/적 상한을 정한 것이 아니다.
- 빌드·fixture 구성·결과 직렬화 시간을 hot operation 시간에 합치지 않았다. start/spawn은 `setup`, 이후 처리는 `measurement`로 따로 저장했다.
- 시간 측정 때 할당 집계는 껐고, 할당 관측은 별도 3회 실행했다. 같은 allocator wrapper의 비활성 확인 overhead는 시간 측정에도 남는다. 교정 후에도 같은 harness/조건으로 비교할 것.
- 할당량은 **현재 시험 thread의 alloc/alloc_zeroed 요청 크기와 realloc의 새 요청 크기를 합산**했다. 해제는 차감하지 않는다. 누적 요청량이며 동시에 그만큼 메모리를 점유했다는 뜻이 아니다. 반환값 drop은 각 측정 closure의 실제 수명대로 처리된다.
- 예산 조회 값은 `[admitted work, clone slots, cumulative retained admission]`. 실패를 유발한 rejected 요청 자체는 누계가 아니라 fault의 requested 필드에서 확인한다. scope 종료 직전 값이지 모든 내부 순간의 실제 메모리 peak가 아니다.
- p95는 11회 표본의 nearest-rank이며 이 표본 크기에서는 최댓값과 같다. 안정된 tail-latency 보장을 주장하지 않는다.

## 2. 기존 전투 경로 — 실제 public step, 제한된 정적 fixture

위치가 고정된 유닛, 9×9 기본 공격 범위, 높은 HP, 이동 속도 0으로 구성했다. 성공 시 실제 유닛 수가 아군+적과 같은지 검사했다. 대규모 이동 공세·사망 연쇄·장시간 기록 누적·모든 적의 동시 공격을 재현한 시험은 아니다.

100/300명 성공 fixture는 전투 시간 0→1,000 ms에 각각 **피해 60건·총 HP 감소 60**이 발생했다. 따라서 “300명이 모두 계속 공격하는 최악 부하”로 해석하면 안 된다. 동일한 실제 타격 수에서 유닛 수 증가와 상태 복사/조회 부담을 비교하는 기본선이다. 피해/시간의 실제 관측값을 기준으로 하며 fixture의 개별 설정 필드를 최종 공격 주기의 계약으로 확대하지 않는다.

| 적 수 | 진행 호출 | 1초 전투 진행 총 시간 p50 / p95 | 누적 요청 allocation | 결과 |
| --- | --- | --- | --- | --- |
| 100 | 100 ms씩 10회 | 3.592 / 3.649 ms | 4,018,804 bytes | 완료 |
| 100 | 1,000 ms 한 번 | 3.314 / 3.497 ms | 2,924,788 bytes | 완료 |
| 300 | 100 ms씩 10회 | 9.779 / 9.858 ms | 13,105,540 bytes | 완료 |
| 300 | 1,000 ms 한 번 | 9.090 / 10.003 ms | 9,095,740 bytes | 완료 |
| 500 | 두 방식 모두 | 진행 시간 비교 불가 | setup 중 거부 | `queued_event_slots` |

1초치 시뮬레이션을 처리한 총 CPU wall time이며 한 렌더 프레임 비용이 아니다. 같은 적 수의 성공한 두 분할 방식은 모든 반복에서 최종 event log SHA-256과 피해 결과가 일치했다. 이 시험의 반복 결과 일치이지 모든 상태/모든 호출 분할의 동등성을 증명하지 않는다.

300명 한 번 진행의 native work는 291,908, clone slots 92,403, 누적 retained admission 880,760 bytes였다. 100 ms 호출들에서는 operation별 work 최대 55,750이었으나 전체 요청 allocation 합계는 더 컸다. 호출을 쪼개면 무조건 성능이 좋아지거나 비용이 사라지는 것이 아님을 보여준다.

### 500명 동시 생성 거부

아군 12명과 적 500명을 같은 시작 시각에 생성하는 setup에서:

```text
RetainedByteLimit
site: queued_event_slots
limit: 8,388,608
requested: 8,402,680
```

네 조건(후속 step 2종, 후속 burst 2종)이 **공통 setup에서 거부**됐다. 후속 전투/타격 4개가 각각 별개 이유로 실패한 것이 아니다. setup 거부까지의 약 1 ms를 500명 전투 실행 시간으로 보고하지 않는다. 단계별 분산 생성이나 다른 queue 초기 용량에서의 수용력은 미측정이다.

## 3. 실제 피해 처리 — private transaction 합성 burst

정상 spawn 후 실제 `damage_source_snapshot_for_unit` 및 `execute_committed_damage`를 `runtime_transaction` 하나에서 호출했다. source/target ID 순서는 정렬했다. 1 또는 12명의 source가 각 적에게 True damage 1을 준다. 공개 스킬 시전·target selection·공간 검색·순차 effects·연쇄 producer를 거치지 않으므로 **실제 완성된 광역 스킬 benchmark는 아니다**. 이 조건에서 하위 피해/기록 경로의 admission 비용을 분리한다.

| 대상 수 × source 수 | 의도한 피해 처리 | p50 / p95 | 누적 요청 allocation | 결과 |
| --- | --- | --- | --- | --- |
| 100 × 1 | 100회 | 0.132 / 0.140 ms | 2,000,488 bytes | HP 감소 100 |
| 100 × 12 | 1,200회 | 0.742 / 0.761 ms | 3,426,280 bytes | HP 감소 1,200 |
| 300 × 1 | 300회 | 완료 시간 아님 | 8,525,027 bytes 후 거부 | `actor_row_reserve` |
| 300 × 12 | 3,600회 | 완료 시간 아님 | 같은 첫 source 처리 중 거부 | `actor_row_reserve` |
| 500 × 1/12 | 500/6,000회 | 측정 불가 | setup 거부 | 앞 절 참조 |

300명 거부의 fault:

```text
RetainedByteLimit
site: actor_row_reserve
limit: 8,388,608
requested: 8,423,888
```

해당 시점 마지막 admitted work 1,048,792, clone slots 3,968, retained admission 8,352,648 bytes였다. 실패 후 live core의 HP 감소/피해 fact 수는 0이고 event log hash는 직전과 일치했다. 이 fixture의 관측은 candidate 폐기와 일치한다. 전체 RNG/UUID/resource rollback 감사의 대체 증거는 아니다.

### 코드와 연결한 원인 해석

- `src/game/battle/core/runtime_facts.rs:85`의 `ActorSummaries::insert`는 용량이 가득 찰 때 `reserve_exact(1)`로 확장하고, **새 배열 전체 크기**를 `admit_retained_growth(0, bytes, "actor_row_reserve")`에 전달한다.
- `src/game/battle/core/queue.rs:444` 이후도 heap 확장 때 `reserve_exact(1)`와 새 전체 배열 크기를 같은 방식으로 청구한다.
- `src/game/battle/runtime_control.rs`는 operation 안에서 누적 growth admission을 환급하지 않는다. 한 칸씩 용량을 늘리면 요청/보수적 청구 합계가 `1+2+…+N` 형태로 커질 수 있다. 이는 **동시 live 메모리가 같은 형태로 증가한다는 뜻이 아니다**. realloc이 실제로 매번 물리적 복사를 하는지도 이번 측정으로 입증하지 않았다.
- 따라서 현재 거부를 곧바로 PC RAM 부족, 안전장치 불필요, 적 300명 자체가 불가능하다는 결론으로 바꾸지 않는다. 기록 구조의 성장 전략과 admission 의미를 먼저 검토할 근거다.

## 4. R06 Live adapter 반복 평가 — 합성 수식 부하

기존 private fixture의 두 유닛과 유효 action/capture를 사용했다. **동일 source의 같은 Live capture를 반복 소비**하며 대상 수/동시 source 수에 해당하는 평가 횟수만 모사했다. 실제 500개 target frame 또는 12개 source capture를 생성한 시험은 아니다. compile/capture 생성은 hot loop 밖이고 매 소비의 Live frame 생성·evaluate·trace·narrowing은 포함한다. public expression publication gate는 열지 않았다.

| 식 / 반복 수 | p50 / p95 | 누적 요청 allocation | native work |
| --- | --- | --- | --- |
| Source Attack 단일 Read × 500 | 0.118 / 0.121 ms | 856,000 bytes | 1,000 |
| 단일 Read × 6,000 (=12×500) | 1.288 / 1.345 ms | 10,272,000 bytes | 12,000 |
| Source Attack 128 Read의 Add × 500 | 2.973 / 3.767 ms | 26,968,000 bytes | 65,000 |
| 같은 128 Read 식 × 6,000 | 36.278 / 42.171 ms | 323,616,000 bytes | 780,000 |

128 Read 식은 동일 값을 반복 읽는 의도적인 비용 스트레스 식이며 일반 스킬의 평균 복잡도로 가정하지 않는다. 결과는 각각 10/1,280인지 매번 assertion으로 검사했다. 표의 큰 allocation 값은 반복 중 해제되는 요청의 합이며 324 MB 동시 점유라는 뜻이 아니다.

단일 Read의 evaluator 회계상 retained는 평가당 768 bytes, 128 Read는 66,112 bytes였다. 반면 native retained admission은 두 6,000회 사례 모두 초기값 **9,099 bytes**에 남아 있었다. 이는 기존 **F01의 평가 메모리 회계 미연결**과 일치한다. 현재 성공률·속도를 보고 호스트 메모리 한도를 안전하게 지킨다고 결론 내릴 수 없다. frame 준비 work·중간 취소의 누락 역시 별도 교정 대상이다.

## 5. 순수 kernel의 resolved 집계 — 수동 frame

runtime resolver가 아직 연결되지 않은 `ResolvedTargets`는 `EvalFrame::set_resolved_targets`로 명시적으로 구성한 합성 입력만 사용했다. 프레임 생성·월드 탐색·피해 적용은 hot loop 밖이다. member 값은 각각 10, 합계가 `10×N`인지 평가마다 확인했다.

| 500 members 집계 | p50 / p95 | 누적 요청 allocation | 평가당 meter work / retained |
| --- | --- | --- | --- |
| WholeSet 1회 | 0.066 / 0.068 ms | 248,640 bytes | 4,258 / 284,672 bytes |
| WholeSet 12회 | 0.768 / 0.792 ms | 2,983,680 bytes | 동일 |
| 타일당 1 member, 500 groups 1회 | 0.129 / 0.153 ms | 354,696 bytes | 4,758 / 883,472 bytes |
| 같은 tile grouping 12회 | 1.446 / 1.488 ms | 4,256,352 bytes | 동일 |

group 수가 많을 때 회계상 retained가 기본 1 MiB에 가까워진다. 이번 500 groups 성공을 더 큰 전장의 임의 집계 성공으로 확대하지 않는다. tile grouping은 나비의 장례 전체나 파고드는 천국 구현을 대신하지 않는다.

## 6. 다음 교정에 넘길 판단

1. 기존 F01/F02/F03 교정이 여전히 선행 과제다. 특히 F01 교정 후에는 이와 같은 부하가 새로 거부되는지 반드시 비교해야 한다.
2. `actor_row_reserve`/`queued_event_slots`의 누적 확장 비용은 별도 capacity 관측으로 인계한다. bounded chunk/geometric growth, 합법적인 사전 용량 산정, 회계의 누적/동시 보유 의미를 검토할 수 있으나 **이번에 수정안을 확정하거나 구현하지 않았다**. 단순히 이전 bytes를 빼면 되는 회계 버그라고 단정하지 않는다.
3. 상한을 올리거나 요청을 임의로 잘라서 성공시킨 baseline은 만들지 않았다. 동결된 R05 회귀 cap도 변경하지 않았다. 정상 지원 범위를 바꿀 때는 별도 근거와 승인이 필요하다.
4. 현재 기본 전투 시험은 60타격의 정적 배치다. 실제 수백 적의 이동·공세, 밀집 광역·다중 source 상태, 연쇄, 장시간 생성/죽음, GameCore application·서버/Unity 전송의 전체 비용은 미측정이다.
5. 다음 구현자는 source identity 차이를 확인하고, 동일 fixture/계측기를 유지해 before/after를 비교해야 한다. 측정 입력 변경이 필요하면 별도 버전으로 두고 이전 실패를 지우지 않는다.

## 7. 증거·재현과 검증 한계

- 원본 입력과 실제 bytes: [source_identity.json](source_identity.json), [source_before.tar.gz](source_before.tar.gz). 작업 트리 변경을 포함해 294개 입력을 고정했다. 과거 단계검토의 전체 manifest와 동일 범위라고 주장하지 않는다.
- 최종 probe: [probe.rs](probe.rs), 빌드/실행: [run.mjs](run.mjs), 집계: [summarize.mjs](summarize.mjs).
- 최종 결과: [measure_002_raw.json](measure_002_raw.json), [measure_002_summary.json](measure_002_summary.json), [실행 로그](measure_002.log), [실행 receipt](measure_002_receipt.json).
- 첫 실행은 단일 Read fixture를 1항 Add로 감싸 `Add requires at least two operands`로 중단했다. 제품이 정상 거부한 **probe 구성 오류**다. [최초 코드](probe_initial_invalid_add.rs), [실패 로그](measure_001.log), [receipt](measure_001_receipt.json)를 보존했고 capacity 실패 6개에 포함하지 않았다. 입력만 Read로 고쳤다.
- 결과 파일 기록 완료 전에 집계기를 한 번 실행해 incomplete JSON parse 오류가 났다. 원본을 수정하지 않고 측정 프로세스 exit 0 후 완성 파일로 집계를 수행했다. 이는 workload 실패가 아니다.
- 임시 경로 `/tmp/r06-load-baseline.ZZys4Z`, 독립 target `/tmp/r06-load-baseline.ZZys4Z/target`. 제품 workspace의 빌드 캐시를 공유하지 않았다.
- 저장된 [instrumentation.patch](instrumentation.patch)는 사본에만 적용한 두 진단 추가를 담는다. 이 두 부분을 제거한 사본의 모든 입력 hash가 원본과 일치하는지 [finalize.mjs](finalize.mjs)로 확인했다. [ARTIFACTS.json](ARTIFACTS.json)은 결과·코드·바이너리 identity를 연결한다.
- 재현은 source archive를 새 임시 디렉터리에 복원하고 instrumentation patch의 include 경로와 runner의 임시 경로를 새 위치에 맞춘 뒤, 새로운 별도 CARGO_TARGET_DIR/output 이름으로 수행한다. 기존 원본·결과 파일에 덮어쓰지 않는다. `run.mjs measure <새 태그>`는 결과를 새 파일로 만든다.
- 새 private test 하나만 실행했다. 전체 회귀 테스트·Clippy·재감사를 수행한 결과가 아니며, 기존 단계검토 판정과 WP/Phase/Goal 상태는 변경하지 않는다.
