# R06 PC 전투 부하 기준·예산 적용 범위 조사

작성일: 2026-09-26 KST  
상태: **코드 조사 및 측정 계획 / 성능 측정 전 / 제품 코드 변경 없음**

## 1. 목적과 이번 작업의 경계

핵 앤 슬래시 전투의 정상적인 대규모 효과를 실행 예산이 가로막는지 검증할 기준을 만든다. 안전장치 제거, 예산 상향, 스킬 대상 수 축소를 먼저 결정하지 않는다.

이번에 수행한 일은 기획·사용자 설명의 부하 기준 정리, 현재 코드의 예산 수명·실패 경계 조사, 기존 예산 테스트 3건 실행, 후속 측정 계획 작성이다. 대규모 벤치마크·새 probe 구현·교정 구현은 수행하지 않았다. 이 문서는 새로운 Goal이나 단계검토 PASS가 아니며, 기존 동결 target/manifest/실행 기록을 대체하지 않는다.

## 2. 확정 사항과 미정 사항

2026-09-26 사용자 설명을 이번 조사 입력으로 기록한다.

| 항목 | 이번 조사 기준 | 주의 |
| --- | --- | --- |
| 플랫폼 | PC | 모바일은 이번 성능 검증 범위에서 제외. PC OS별 지원 완료를 의미하지 않음 |
| 아군 | 최대 12명 | 투사체·트랩·시각 효과를 모두 아군 유닛으로 세지 않음 |
| 적 | 수백 명 규모 | 정확한 숫자 상한은 미확정. 임의로 500명 등을 제품 제한으로 고정하지 않음 |
| 파고드는 천국 | 타일당 최대 1개체, 타일 수가 증식 한도라는 사용자 설명 | 동시 개체 수와 전투 중 누적 생성·사망 횟수를 구분 |
| 전장 | 가로×세로의 구체 수치 미정 | 테스트용 크기를 정식 전장 규격으로 승격하지 않음 |
| PC 목표 사양·시간 목표 | 미정 | 측정 가능하지만 절대 시간 기준 성능 PASS는 아직 불가 |

파고드는 천국은 생성·점유 가능한 타일 수를 `T`로 두면 동시 개체 수 `N <= T`로 모델링한다. 이를 다른 종류의 모든 적에게 일반화하지 않는다. 이동·생성이 충돌할 때의 처리, 포화된 타일 때문에 실현하지 못한 생성의 처리/대기 여부 등은 별도 설계 사항이다. 재시도 대기열이 있다면 그 양도 측정해야 하며, 이번 조사에서 그런 동작을 새로 정하지 않는다.

기존 [조우 문서](../skills/abnormality_design_notes.ko.md)의 `파고드는 천국` 절(1354행 이후)은 “개체 수에 상한이 없다”라고 서술한다. 이번 사용자 설명은 공간에 의한 한도를 명확히 한 것으로 기록한다. 원문은 이번에 수정하지 않았고, 후속 기획 정합화 시 “별도 숫자 상한 없음 / 타일 점유에 따른 유한 상한”을 구분해 반영할 필요가 있다.

## 3. 실제 기획에서 뽑은 부하 유형

설계 문서는 구현 완료 증거가 아니다. 아래 특성을 측정 시나리오로 번역하되 미구현 스킬 전체를 R06에 추가하지 않는다.

| 기획 근거 | 부하 특성 | 이미 있는 의미상 제한 / 확인할 미정 값 |
| --- | --- | --- |
| [파편 문서](../skills/abnormality_skill_fragments_design_notes.md) 1615행, 포자 감염 | source별 감염 집합, 주기 피해, 모든 숙주의 광역 기폭 | 같은 transaction 즉시 재전파 금지, 기폭의 새 감염·사망 전파 금지. 중첩 기폭 적중 제한·주기 등 미정 |
| 같은 문서 2696행, 흉탄 개화 | 처치 기반 후속 탄환·세대·충돌·이벤트 | 최대 세대, 총 탄환·적중 대상 제한을 authoring. 실제 숫자 미정 |
| [조우 문서](../skills/abnormality_design_notes.ko.md) 395행 이후, 오케스트라 | 관객 사망 → 비명 광역 피해 → 다른 관객 사망 | 관객당 한 번, 신규 진입 적은 비명 발생원 아님. 파동 중첩과 동일 시각 순서는 별도 검증 |
| 파편 문서 1305행, 나비의 장례 | 경로 타일 순회, 타일/유닛 스택 집계, 지연 폭발 | 스택 수를 폭발 수로 그대로 복제하지 않음. 전장 타일 수가 별도 비용 축 |
| 파편 문서 480행 이후, 증오의 여왕 | 끌어모으기 후 밀집된 적에게 광역 피해 | 분산 배치와 밀집 배치를 구분. 유닛 수만 같아도 실제 적중 수가 다름 |
| 파편 문서 3780행 이후, 포큐버스 | 여러 아군의 피해가 하나의 공유 보호막을 순서대로 소비 | 아군별 독립 보호막으로 복제하거나 동시 피해를 임의로 합산하면 의미가 바뀜 |
| 파편 문서 4209행 이후, 루돌타 | 평타와 독립된 반복 광역 폭격 | 평타 속도로 폭격 주기를 자동 증폭하지 않음. 한 폭탄당 한 적 한 번, 서로 다른 폭탄은 별개 |

개념적 비용은 `시전 수 × 시전당 효과 수 × 효과별 실제 대상 수 × 대상별 수식 비용`과 프레임 구성·공간 검색·피해/사망·후속 이벤트·기록·복사 비용을 나누어 본다. 이는 정확한 runtime 비용 공식이 아니다. 예컨대 숙주 수와 폭발별 대상 수가 함께 커지면 유닛 수보다 적중 처리 수가 훨씬 커질 수 있다. 실제 중복 제한·공간 인덱스·실행 순서에 따라 달라지므로 일괄적으로 O(N²)라고 단정하지 않는다.

## 4. 현재 코드에서 확인한 예산의 층위

경로는 `core_enhanced/` 기준이며 조사 시점 행 번호다.

| 층위 | 현재 기본값의 예 | 적용·초기화 범위 | 근거 |
| --- | --- | --- | --- |
| ValueExpr 계획/평가 | depth 64, nodes/stack 1024, members/groups 4096, eval work 65,536, compile work 별도 65,536, retained 1 MiB | adapter는 평가 호출마다 새 meter 생성. 직접 kernel 호출자는 meter 수명을 소유 | `src/game/value_expr/mod.rs:328`, `:602`, `src/game/battle/core/value_expr_runtime.rs:474` |
| native 전투 실행 | work 10,000,000, clone slots 1,048,576, retained 8 MiB, actions/results 각 4096, deliveries 8192 | 외부 operation scope 하나. 중첩 호출은 남은 예산 공유. scope 종료 후 다음 작업에서 새 예산 | `src/game/battle/runtime_control.rs:60`, `:263` |
| controlled authoring 실행 | work 10,000,000, pending events 8192, bucket events 4096, effect commands 8192 등 | authoring attempt 전체. 해당 실행 안에서 native 예산도 활성화되면 두 층의 제약을 함께 받음 | `src/authoring/control.rs:27`, `:536`, `:568` |

중요한 해석:

- `65,536`은 적 수나 전투 전체 제한이 아니라 수식 평가 work의 기본값이다. work는 계측된 논리 비용이며 CPU 명령 수·밀리초가 아니다.
- members/groups는 meter 사용량에 누적된다. 중첩/반복 집계를 고려하지 않고 “적 4096명까지 항상 허용”이라고 해석할 수 없다. `value_expr/eval.rs:1172` 이후의 charge 함수 참조.
- evaluator retained는 입력 AST/frame이 이미 소유한 메모리를 제외한 보수적 신규 저장/예약 계산이며 RSS가 아니다. 실제 함수도 누적 청구한다(`eval.rs:1228`).
- native retained는 현재 owner의 extent와 operation 내 누적 growth admission을 함께 사용한다. 해제·후보 폐기가 같은 operation의 allowance를 환급하지 않는다(`runtime_control.rs:392` 이후).
- authoring의 event/entity admission 수치를 그대로 “일반 게임의 유닛/이벤트 상한”으로 옮기면 안 된다. `authoring::control::admit`는 authoring attempt에 적용되고, native 경로에는 별도의 work/retained/fact 제약이 있다. 모든 경로의 보호 범위가 동일하다고 주장하지 않는다.
- native limits의 `validate`는 현재 기본값을 상한으로 사용한다. 추후 수치 상향은 단순 RON 설정이 아니라 코드·정책·검증 변경이다.

## 5. 실행 범위와 실패 시 되돌리기 범위

| 호출 경로 | 예산 범위 | 현재 후보/commit 경계 |
| --- | --- | --- |
| `BattleCore::apply_live_command` | 바깥 native operation | `runtime_transaction`의 candidate 전체. 성공 및 poll 이후 교체 |
| `BattleCore::step_battle_execution` | 외부 scope가 없으면 해당 작업 | timestamp candidate와 execution state를 함께 commit |
| `BattleCore::step_battle_execution_until/by` 직접 호출 | 한 호출에서 여러 timestamp가 같은 work/clone/growth allowance 공유 | timestamp별 commit. 뒤 시각에서 실패하면 이미 commit한 앞 시각은 유지 가능 |
| `GameCore::advance_active_battle_by/for_server_tick` | `execute_application`이 가진 바깥 operation | 바깥 GameCore draft에서 in-candidate 진행. 오류 시 이 application의 draft를 버리므로 해당 호출 전체가 commit되지 않음 |

근거: `src/game/battle/core/runtime_facts.rs:1594`, `:1619`; `core/sim.rs:2217`, `:2280`; `src/game/world/application.rs:174`; `src/game/world/combat.rs:1445`, `:1507`, `:1616`.

따라서 “항상 한 틱 예산”, “한 스킬이 실패하면 항상 스킬만 취소”, “오류면 전투 전체를 처음부터 rollback” 중 어느 것도 일반 규칙으로 적절하지 않다. 예산 소유자와 commit 소유자를 각각 확인해야 한다. 긴 elapsed 요청과 짧게 나눈 요청은 같은 전투 시간을 진행해도 누적 allowance·복사 비용이 달라질 수 있다. 이를 우회하기 위해 임의 분할하지 말고, 동일 성공 결과/이벤트 순서를 유지하는지와 지원할 호출 범위를 시험한다.

위 내용은 현재 공통 transaction 코드의 조사 결과다. 아직 연결되지 않은 ValueExpr skill consumer에서 모든 오류가 이 경계까지 전파된다는 완료 판정은 아니다. 실패 이후 자동 재시도·플레이어 안내·서버 복구 정책도 이번에 검증하지 않았다.

## 6. 현재 R06 상태와 선행 교정

`capture_value_expr_entry`와 `evaluate_value_expr_capture` 호출 검색상 production skill consumer 연결은 아직 없다. `core/sim.rs:2463`은 DamageExpr/HealExpr 실행을 WP02 이전 진입 불가로 취급한다. `build_value_expr_frame`은 resolved-target aggregate를 거부하고, 결과 prefix도 미연결이다. 외부 publication 차단을 우회해 벤치마크를 성공시켜서는 안 된다.

기존 [WP02 단계검토](../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_runtime_frame_capture_001_review_001/REVIEW.md)의 `CORRECTION_REQUIRED`는 유지한다.

- **F01:** 현재 adapter는 Live frame을 먼저 만들고 plan 한도로 평가한 뒤 `authoring::control::charge`로 호스트 work를 청구한다(`value_expr_runtime.rs:462–486`). 더 작은 호스트 잔여 work/메모리와 진행 중 취소를 사전에 연결하는 교정이 필요하다. 측정 수치가 좋더라도 이 순서 문제는 없어지지 않는다.
- **F02/F03:** actor/source-copy 결속과 비활성 unit의 Live resource 읽기는 부하 수치와 별개인 기존 교정 과제다. 이번 부하 조사로 해결·면제하지 않는다. 이번에는 기존 재현 probe를 다시 실행하지 않았다.
- F01 교정은 안전한 admission 순서/회계 연결 문제이고, 정상 부하 수용량은 별도의 capacity 문제다. 먼저 한도를 높여 F01을 숨기거나, 더 작은 meter만 넣고 정상 대규모 스킬이 안전하다고 끝내지 않는다.

[설계 D08](../goals/content_authoring_value_expressions_reassessment/CODE_REVIEW_003.ko.md)의 숫자는 technical safety 초기 제안이다. R05의 `9,030,188 bytes`는 보존할 특정 concurrent requested-allocation 회귀 기준이지 모든 전투의 RSS·제품 메모리 상한이 아니다. 기존 fixture/threshold 결과는 그대로 비교하고, 새로운 대규모 workload는 별도 시리즈로 기록한다. 기준 변경이 필요하면 영향·근거·승인을 별도로 남기며 역사적 실패를 소급 PASS로 바꾸지 않는다.

## 7. 후속 측정 매트릭스 — 제안, 미실행

첫 탐색용 숫자: 적 `100 / 300 / 500`, 타일 `16×16 / 20×20 / 25×25`. **사용자가 확정한 상한이나 실제 맵 규격이 아닌 임시 측정 좌표**다. 각 스킬의 합법적인 대상·장착·중첩 조건이 확인되지 않은 조합은 “실제 가능 전투”가 아니라 합성 부하로 표시한다. 타일당 1개체 규칙은 파고드는 천국 포화 모델에만 적용한다.

| ID | 입력과 비교 | 지금 가능한 범위 / 뒤에 필요한 검증 |
| --- | --- | --- |
| W01 일반 지속전 | 아군 12, 적 수 변화, 분산 배치, 반복 전투 | 기존 literal runtime으로 측정 가능. 숫자 표현식 경로 성능과 혼동 금지 |
| W02 밀집 광역 | 같은 적 수로 분산/밀집 비교, 적중 대상 수 기록 | 현재 AoE 경로와 kernel 반복 평가를 분리 측정. 연결 후 같은 public 시나리오 재검증 |
| W03 집계·기폭 | source별 숙주 수, 폭발별 대상 수, grouping 변화 | kernel authored/resolved frame 합성 시험. 실제 감염·기폭 의미 및 resolver 성능은 해당 구현 후 검증 |
| W04 유한 연쇄 | 비명 발생원 수, 흉탄 branch/세대/총량을 독립 축으로 변화 | 모사된 event/평가 부하임을 명시. 정확한 스킬 처리 순서·의미의 PASS로 확대 금지 |
| W05 동시 발동 | 같은 시각 1/6/12개 source 작업, 주기 피해와 겹침 | source 수는 합성 축. 실제 동시 장착/발동 가능성 검증 전 정상 최대 조합으로 확정 금지 |
| W06 타일 포화·교체 | 파고드는 천국 N=T, 이어 처치/재생성 반복 | 동시 peak와 누적 생성·죽음·기록·대기량을 따로 측정. 공간 포화 시 규칙은 추가 결정 필요 |
| W07 호출 묶음 | 같은 event/seed/time을 작은 진행 요청 vs 큰 요청으로 처리 | BattleCore 직접 경로와 GameCore 경로를 나누고 결과·commit prefix·fault 경계를 비교 |
| W08 경계 실패 | work/retained 등의 limit−1/limit/limit+1, 잔여 0, 취소, 실패 후 재시도 | 각 카운터 의미에 맞는 fixture 필요. 설정 0 자체가 invalid인 경우, 양수 예산을 먼저 소진한 잔여 0과 구분 |

반복 주기·지속 시간은 기획 확정 값과 실험 변수를 구분한다. 기본 공격 속도가 독립 스킬/폭격의 주기까지 증폭한다고 가정하지 않는다. 긴 전투에서 메모리/기록이 계속 쌓이는지 확인할 soak 구간도 따로 둔다.

### 계측 항목과 시험 조건

- kernel: compile 비용과 runtime을 분리; 평가 횟수, work, raw/canonical members, groups, retained admission, stack/task peak, frame 생성 비용, mandatory trace 비용.
- native operation: work/clone 누적, 시작 retained extent·누적 growth·종료 extent, action/delivery/result peak. 현재 공개 통계로 얻을 수 없는 항목은 격리 계측 hook이 필요하며 0으로 대신하지 않는다.
- 전투: 실제 적중·피해·사망 수, queue/bucket/event peak, source별 capture/witness 생존량, 공간 검색, rollback·복사, 기록/전송용 결과 준비 비용.
- 메모리: 누적 요청 allocation, 동시 outstanding 요청량, 회계상 retained, 프로세스 RSS를 서로 다른 열로 기록. instrumentation 비용도 별도 명시.
- 시간: 컴파일을 제외한 release 실행의 준비/프레임/평가/전체 operation 구간, 반복 샘플 수·워밍업·p50/p95/p99/max. CPU/OS/Rust 버전/빌드 flags/seed/input hash/trace 조건을 기록. PC 최소 사양이 미정이므로 로컬 수치를 전체 PC 지원 보장으로 표시하지 않는다.
- 의미 검증: 결과 checksum만 아니라 피해/회복·자원·RNG/identity·이벤트 순서·실패 시 state/queue/log/prefix를 비교. target 생략·피해 0 fallback·후속 효과 누락으로 성능을 맞추지 않는다.
- 동결 source를 사용하는 private probe는 별도 임시 사본과 **분리된 CARGO_TARGET_DIR**에서 수행. 지원되지 않는 public capability는 열지 않는다. 합성 kernel, 실제 adapter, public runtime 결과를 구별해 기록한다.

## 8. 권장 진행 순서와 판단 기준

1. 이 조사 기준으로 작은 측정 harness를 구성하고 **변경 전 baseline**을 얻는다. 기존 literal 전투와 합성 expression을 구분하며, 미연결 부분 때문에 얻을 수 없는 수치는 미측정으로 둔다.
2. F01/F02/F03 수정 계획을 확정하고 별도 승인·baseline 아래 교정한다. 계측은 admission 전 작업량과 취소 응답도 검증해야 한다.
3. 동일 입력·동일 측정 방식으로 교정 후 비교하고 단계검토한다. 의미·회귀·안전성 검증 없이 속도만으로 PASS하지 않는다.
4. 정상 workload가 거부되면 비용 누락/중복, 불필요한 복사·trace·재집계, 실행 묶음의 계약, 실제 허용 조합을 먼저 분류한다. 필요한 capacity 변경은 근거를 제시해 별도 판단한다.
5. 재검토 이후 사용자 요청으로 WP02 consumer·prefix·rollback 구현을 재개하고, 연결된 public 경로에서 다시 측정한다. 후속 스킬 전체 구현은 각 소유 작업에서 검증한다.

이번 단계의 결론: **안전장치의 세밀함 자체를 줄이거나 상한을 확정할 근거는 아직 없다. 예산 범위는 파악됐고, 남은 것은 실제 부하 측정과 기존 admission 결함의 교정이다.** 맵의 정확한 크기와 최소 PC 사양이 없어도 상대 비교·안전 경계 시험은 시작할 수 있다. 최종 지원 규모/시간 합격선은 그 정보와 측정 이후 결정한다.

## 9. 직접 검증과 조사 기준점

실행 명령:

```sh
cargo test --manifest-path core_enhanced/Cargo.toml --lib game::battle::runtime_control::tests -- --nocapture
```

결과: **3 passed / 0 failed / 1404 filtered out**, exit 0. 기존 `ResourceStore::apply_operations` dead-code warning 1건. clone 누적·다음 작업 초기화, deferred scope 공유/첫 fault 유지, retained growth 미환급을 검사하는 기존 테스트다. 대규모 성능 측정이나 기존 F01 재현 재실행이 아니다. 전체 테스트·Clippy는 이번 문서 조사에서 재실행하지 않았다.

HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`. 작업 트리에 다수의 기존 변경/untracked 파일이 있어 HEAD만으로 현재 구현을 식별하지 않는다. 아래는 조사 핵심 파일의 실제 bytes SHA-256이며 전체 source manifest/currentness 증명은 아니다. 이 문서 외 제품·기획·동결 자료는 편집하지 않았다.

| 파일 (`core_enhanced/` 기준) | SHA-256 |
| --- | --- |
| `src/game/value_expr/mod.rs` | `fb1a52229e6a1d872a122320272f8b26092065ec0bb578d7a9be1a2592cb24f1` |
| `src/game/value_expr/eval.rs` | `961abbe0119a053c419edfae20550cfbfa45a6f63a385669c38c6a6574e56582` |
| `src/game/battle/runtime_control.rs` | `5a4039fc9f24738ae20f3c6c2b63ba473e561adabb5dfb2cc3a538a290c86d0f` |
| `src/game/battle/core/value_expr_runtime.rs` | `bbe0227f8843334c1f4ee20bce4d13bae47e0621fb4ed246cc1d750a504f3b25` |
| `src/game/battle/core/runtime_facts.rs` | `f1b5399087da9c7d970f6d7d103861c2d029fe9543abb558670e1af8ed9f6ae7` |
| `src/game/battle/core/sim.rs` | `5068b3a1bb17fe54f6d7dc0c8cf3560767f767dfa1bb25c456910587ae599b91` |
| `src/game/world/application.rs` | `36ba3058aacfd27980ea3211e2c0fd6be088367139808487e018a9076965c057` |
| `src/game/world/combat.rs` | `96bc69bad5bc802a96840a36a9b56468c2969c928204a9700699db43791d467a` |
| `src/authoring/control.rs` | `5e8638daa77594ecac9bee432996dbeea0d32605e000c354d3a84dd16e7559e2` |
| `docs/skills/abnormality_skill_fragments_design_notes.md` | `dfbfbe4f52d7b08fb51777586c3c9b5c3e6f8f9774c7479354e337c0584e2ff6` |
| `docs/skills/abnormality_design_notes.ko.md` | `58700cc50105941303198e82992972e5cc0b382ef4e82013f6fdbdeab5c6ac80` |
