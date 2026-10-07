# R06 WP02 CAP-01/02 correction 001 단계검토

## 판정

- 검토 대상: `corrections/wp02_cap_01_02_001/target/cap_01_02_correction_001/MANIFEST.json`
- SHA-256: `179bff1593608283c82ed51a2923f183d47312b8df51ad3dcb7556fdca8a4a5f`
- **CORRECTION_REQUIRED — P3 포맷 정리 1건.** 이번 변경 범위에서 기능 교정을 요구할 새 결함은 발견하지 않았다. CAP 기능의 재설계나 예산 상향을 요구하는 판정이 아니다.
- 독립 검토에서 CAP-01/02 성장·선청구와 action–cast 제품 회귀의 구현 방향은 수용 가능하다. 다만 새 포맷 차이가 남아 있으므로 현재 동결 타깃을 무조건 PASS 처리하지 않는다.
- WP02/R06/Phase/Goal 완료나 다음 WP 진입 승인이 아니다. 제품 소스와 기존 동결 자료는 변경하지 않았다.

## CAP-R01 [P3] 새 코드의 rustfmt 차이를 정리할 것

위치 예시: `core_enhanced/src/game/battle/runtime_control.rs:456`, `core_enhanced/src/game/battle/core/value_expr_runtime.rs:2255`, `core_enhanced/src/game/battle/core/runtime_facts.rs:2357`.

동일한 5개 변경 파일에 `rustfmt --edition 2021 --check --color never --config skip_children=true`를 독립 실행했다. 보존된 정확한 before는 37개 diff hunk/exit 1, 현재 after는 57개/exit 1이다. 신규 helper와 추가 회귀 시험에도 포맷 차이가 있다. `format_before.log`, `format_after.log`에 결과를 남겼다. 차이의 개수는 순증 20 hunk이며, 이를 20개의 별도 기능 결함으로 해석하지 않는다.

수정 방향:

1. 이번에 추가·수정한 영역의 포맷을 정리한다. 기존의 관련 없는 37개 차이를 전부 청소할 의무로 범위를 넓히지 않는다.
2. before와 대조하여 신규 포맷 회귀가 없어졌음을 검증한다. 기존 차이가 남아 전체 명령이 exit 1이면 이를 PASS로 쓰지 말고, baseline 잔존과 신규 차이 제거를 나누어 기록한다.
3. 의미 변경 없는 포맷 교정인지 diff로 확인하고 관련 회귀/check를 검증한 다음 별도 correction target으로 동결한다. 기존 target/archive/로그는 덮어쓰지 않는다.
4. 재검토는 포맷 delta·의미 보존·타깃 현재성에 집중할 수 있다. 이번 사유만으로 CAP 기능 재구현이나 PC 예산 재조정을 요구하지 않는다.

## 직접 코드 검토

5개 변경 제품 파일과 연결된 비용 청구·COW·queue dispatch 경계를 읽었다.

- `admitted_buffer_capacity`: checked 최소 원소 수, 약 1.5배 preferred capacity, 원소 상한, retained/native work/authoring work headroom을 함께 적용한다. 최소 용량도 불가능하면 실제 admission이 실패한다. 선호 크기를 먼저 청구해 fault를 만든 후 무시하는 방식이 아니다.
- `admit_retained_growth` → authoring `charge` → native `charge` 연결을 확인했다. 전체 새 요청량과 64 bytes overhead를 reserve 전에 청구하고, 기존 버퍼 해제는 operation 예산을 환급하지 않는다.
- ActorSummaries: 정렬된 ID, insert shift 비용, actor 개수 제한을 유지한다. live extent는 실제 capacity를 포함하고, COW는 실제 복제되는 live rows를 기준으로 청구한다.
- EventQueue/timestamp bucket: `reserve_exact(capacity - len)`의 인자를 확인했다. comparator를 바꾸거나 원소를 재정렬하지 않는다. payload/source/완료 metadata의 기존 경계는 보존된다. heap의 남은 capacity와 clone의 live len을 구분한다.
- action–cast 회귀는 실제 cast 생성 경로를 사용하며 다른 actor·동일 actor의 다른 cast 치환 거부, 정상 cast/castless, controller 정리 뒤 delivery, source 퇴장 뒤 Snapshot을 검증한다.

## 직접 실행

격리 소스: `/tmp/r06-cap-review.2Wuy7u`. frozen after archive를 해제하고, queue 테스트 모듈 한 곳에만 검토용 hook과 `probes.rs`를 추가했다. hook을 제외한 333개 소스의 SHA-256 일치를 확인했다.

| 검증 | 결과 |
| --- | --- |
| frozen target/baseline/predecessor, before+delta 복원, 현재 source/evidence | 333 source / 48 evidence 검증 통과 |
| 추가 독립 probe | 3/3 |
| battle core 관련 시험 | 388/388, 위 probe 3개 포함 |
| native runtime control | 4/4 |
| ValueExpr kernel/catalog/parser | 37/37 |
| 고유 시험 합계 | **429/429**: 제품 426 + 검토자 추가 3 |
| focused rustfmt | before 37 / after 57 hunk, 모두 exit 1 |

독립 probe 내용:

- 작은 len/원소 상한/byte/work 조합 122,640건을 최대 합법 용량 oracle과 대조.
- timestamp bucket 원소 상한 1, 최소 비용보다 1 byte 부족한 경우 reserve 전 거부, 다음 operation의 성공을 확인.
- 300개 이벤트의 동일 priority metadata 순서를 이전 한 칸씩 reserve하는 heap과 대조. pop/clear 후 원본 capacity 유지와 빈 clone의 capacity 0도 확인.

최초 테스트 호출은 Cargo cache 쓰기 제한으로 실패했고 승인된 재실행에서는 검토 runner의 package 이름 오기로 실패했다. 이를 `game_core`로 정정한 `independent_build`부터 정상 실행했다. 실패 로그도 남겼으며 제품 실패로 분류하지 않는다. 기존 `apply_operations` dead_code warning은 남아 있다.

## 기존 증거 재사용 및 독립 재분석

전체 all-target 1,475개 시험(1,423 lib + 52 integration), all-target check, 기존 allow Clippy, PC 측정, R05 allocation guard는 **구현자 동결 증거를 재사용**했다. 이 검토에서 해당 전체 명령이나 PC/R05 부하 측정을 새로 실행한 것으로 표현하지 않는다.

`check_receipts.mjs`로 PC raw before/after 각 576행을 다시 검사했다. 36조건마다 timing 11회, allocation 3회, 각 모드 warmup 총 2회이며, 각 조건의 성공/실패가 일관됐다.

- 거부 11 → 5. 남은 5조건은 기존 `value_expr_evaluation_retained` 거부다.
- 300명 × 1 source는 300 damage mutations와 HP 감소 300을 확인했다. 500명 setup은 총 512 units이며 1/12 source 피해량도 500/6,000과 일치한다.
- 전후 모두 성공한 전투 6조건은 모든 after sample의 log hash와 HP 감소가 before와 같았다.
- R05 final-exact receipt의 concurrent peak delta 최대는 9,010,868 bytes, 기존 cap 9,030,188 bytes 이내다. 여유는 **19,320 bytes**로 작다. 이는 고정 fixture에서의 관측이며 플랫폼·스케줄에 독립적인 여유 보장은 아니다. 최종 통합에서 같은 guard를 유지해 재확인할 사항이지, 현재 새 기능 결함 판정은 아니다.

PC 수치는 누적 allocation 요청 합계이며 동시에 보유한 메모리 peak/RSS가 아니다. 거부되던 workload가 완료된 것이 핵심 개선이며 모든 경로의 실행 시간 개선이나 실게임 대규모 공세 성능 합격을 뜻하지 않는다. 원시 receipt 재분석기의 초기 field mapping 오류는 실제 raw schema에 맞춰 수정한 뒤 검증했으며 제품 오류가 아니었다.

## 남은 범위와 다음 단계

우선 CAP-R01 포맷 교정만 수행하고 별도 타깃을 제시한다. 기능 의미를 바꾸지 않았다면 작은 후속 검토로 마무리할 수 있다. 기존 128 Read 거부를 이번 CAP 교정의 새 결함으로 재분류하거나 무조건 성공시키기 위해 예산을 올리지 않는다.

실제 전체 공세, 다른 플랫폼, PC peak/RSS와 WP02의 남은 sequential effect/consumer 통합 등은 이번 검토의 완료 대상이 아니다. 향후 구현 진행과 전체 완료 판정은 분리한다.
