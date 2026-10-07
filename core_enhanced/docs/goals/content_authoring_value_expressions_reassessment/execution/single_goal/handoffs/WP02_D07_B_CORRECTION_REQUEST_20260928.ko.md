# R06 WP02 D07-B — F01–F03 교정·검증 보강 지시서

2026-09-28 · 상태: **교정 계획 작성 완료 / 구현 시작 전**.
적용: [단계검토 구현 지시서 템플릿](../../../../../templates/IMPLEMENTATION_CHECKPOINT_REQUEST.template.ko.md)
`2026-09-27-v1`, 참고 GEX `2026-09-13-gex-r7`의 구현·기록·단계검토 원칙.

사용자 선택: 이 지시서 작성 뒤 대화를 분기한다. 원본 대화가 구현·자체 검증·동결을 맡고,
사본 대화가 구현 종료 후 읽기 전용 단계검토를 맡는다. 문서 작성만으로 제품 수정을 시작하지 않는다.
분기는 공유 설계 판단까지 독립시키지 않으므로 엄격한 독립 완료 감사와 구별한다.

## 1. 구현 시작 요청

분기 후 **원본 구현 대화**에서 다음 요청을 사용한다.

> 이 지시서의 R06 WP02 D07-B 교정을 시작해줘. F01–F03의 기존 요구와 현재 코드를 대조하고,
> 실제 before를 보존한 뒤 §7 순서로 진행해줘. PLAN·EXPERIMENTS·EXPERIMENT_NOTES를 구현 중 갱신해줘.
> 의미 있는 교정 동작이 완성되고 필수 검증을 통과하는 첫 지점에서 다음 큰 작업을 멈추고,
> successor target과 인계서를 작성해줘. 아직 처리하지 않은 finding과 시험은 미완료로 남겨줘.
> 최종 검토는 분기한 대화에서 진행하므로 자체 PASS를 독립 검토 PASS로 표시하지 마.
> D07-C/WP03, WP04, R06 전체 완료나 production expression 공개까지 진행하지 마.

일반적인 내부 구현 선택·컴파일 오류·실패한 시험은 이번 범위 안에서 조사·교정한다.
새 게임 정책이나 요구 축소가 필요할 때만 사실·선택지·영향·권고를 정리해 사용자에게 묻는다.

## 2. 시작점·범위·보존 정책

- 상위 작업: `GS-CAVER-P2G50-001 / GOAL-CAVER-01`, R06 WP02 D07-B. 새 Goal을 만들지 않는다.
- [기존 D07-B 지시서](WP02_D07_B_IMPLEMENTATION_REQUEST_20260927.ko.md)의 B01–B20과 V17은 유지한다.
- [수정 필요 검토](../reviews/wp02_d07_runtime_amount_001_review_001/REVIEW.md)의 F01–F03만 교정한다.
- [구현 인계](../segments/wp02_d07_runtime_amount_001/REVIEW_HANDOFF.ko.md),
  [실패 이력](../segments/wp02_d07_runtime_amount_001/FAILURE_HISTORY.md),
  [검토 target](../segments/wp02_d07_runtime_amount_001/target/d07_runtime_amount_001/MANIFEST.json).

| 기준 | 값 |
| --- | --- |
| Repository | `/mnt/f/work/simulator` |
| Manifest SHA-256 | `7db52e95938de08131f352f7137be8196c5099b150d1b52874ee1d2ae0315f9f` |
| Source | 340개, `ad8021359785e365c3f9560268bcef2a95205495f9e33a22314c3f97743480c4` |
| 제품 source | 336개, `6b5e0291b1b6f936c277e2138bc2c2adbbeaaa2b3df5c1b4d5bf31b5acff6807` |

작성 시 predecessor `freeze.py verify`로 currentness/양방향 복원을 다시 확인했다.
구현 시작 때도 실행하고 결과를 **새 교정 기록**에 저장한다. 차이를 reset/clean/stash로 지우지 않는다.
기존 dirty/untracked 및 다른 작업의 파일을 보존한다. 작성 시 근거 파일은
[코드 대조 기록](WP02_D07_B_CORRECTION_CODE_EVIDENCE_20260928.json)에 있다.

보존할 정책: host 관측 기본 off, active 전투와 기존 last-terminal 각각 최근 1건, 오래된 Exact는 not-retained,
기록되지 않은 수치를 0으로 대체하지 않음, Snapshot/Live 실제 소비 입력 사용, 기존 gameplay·순서·RNG·오류 의미 보존.
authored DamageExpr/HealExpr production 차단, D07-A 정적 `performed=false`, 미래 usefulness Indeterminate를 유지한다.
9MB concurrent peak는 [비용 정책](R06_COST_POLICY_UPDATE_20260927.ko.md)에 따라 경고이며 다른 hard guard는 유지한다.
20분 시험, 자동 스킬 취소, 새 메모리 상한·설정 RON·편집기·실험실 구현은 제외한다.

## 3. 실제 코드 연결점과 수정 방향

경로는 `core_enhanced/` 기준이며 줄 번호보다 심볼을 따른다.

| 파일·심볼 | 현재 확인 사실 | 교정/확인 방향 |
| --- | --- | --- |
| `src/bin/author_content.rs::Arguments::parse` | simulate는 필수 4개 positional 길이 검사를 건너뛰고 `args[3]` 접근 | 필수 길이를 먼저 검사. 기존 bounded 입력/옵션 규칙 유지 |
| `battle/core/sim.rs::execute_ordered_skill_effects` | expression에는 values Vec 사전 admission이 있고 관측 on legacy에는 없음 | 두 분기의 checked 크기 계산·사전 admission을 동일하게 적용 |
| `battle/runtime_control.rs::admit_retained_growth` | operation 내 누적 admission이며 release/rollback로 allowance가 회복되지 않음 | 기존 guard 재사용. 테스트에서 cap을 바꿀 때 새 operation을 사용 |
| `battle/core/value_expr_runtime.rs::evaluate_value_expr_capture_with_resolved` | identity 확인 후 Snapshot Arc 또는 새 Live frame 선택 | 관측 활성 actual executor에서 생성된 record로 두 query까지 검증 |
| 같은 파일 `evaluate_legacy_amount_for_observation` | catalog context-free plan의 결과를 command에 전달 | Damage/Heal/zero/음수 경계와 관측 on/off 소비 동등성 확인 |
| `battle/core/amount_observation.rs::PendingAmountEvaluation::from_runtime`, `commit_amount_evaluation` | 마지막 후보 1건, prefix settle 뒤 record/ordinal 교체 | 기존 record가 있는 실패·취소·다중 대상 교체에서 원상복구 확인 |
| `game/value_expr/mod.rs::EvalFrame::retained_extent` | scalar와 member/view를 순회해 extent 계산 | 실제 payload와 관계를 확인. 시험 때문에 값 축소·한도 확대 금지 |
| `battle/core/runtime_facts.rs::retained_runtime_fact_extent`, `preflight_runtime_fact_clone` | active observer의 record extent 포함 | 공유 Arc retained와 copy 비용을 구분해서 경계 시험 |
| `world/authoring_observation.rs::capture_authoring_terminal`, `stored_amount` | record가 terminal capsule로 이동 | action 정리·unit 제거·다음 전투·capsule 교체 후 기록과 retained 수명 확인 |
| `authoring/clone_shape.rs::CloneShape for Arc<T>` | payload 재귀 검사 없이 핸들만 가격 책정 | 이를 retained payload 검증으로 오인하지 않음. generic Arc visitor를 deep clone 방식으로 바꾸지 않음 |
| `authoring/amount_query.rs::select`, `validate_record`, `encode_record` | 선택→결속 확인→공통 evaluator→결과 비교→bounded 직렬화 | 실제 보관 기록에 대한 실패/경계/큰 정수 검증 |
| `authoring/trial.rs::inspect_amount` | 별도 Inspection scope와 native provenance 사용 | query가 accepted session을 변경하지 않음을 검증 |
| `authoring/output.rs::encode_result`, `author_content.rs::line` | typed Serialize 뒤 원문 bytes 출력 | i128 중간 trace가 Value/f64를 거치지 않음을 실제 경로로 입증 |

새 query 전용 계산식, 현재 state에서 과거 frame 재구성, 테스트만 사용하는 production 분기를 만들지 않는다.
검증 보강으로 결함이 드러나면 이 표의 관련 계약 안에서 수정하며, 실제 수정 위치와 이유를 기록한다.

## 4. F01·F02의 구체적인 교정

### F01: CLI 필수 인자 확인

`needs_request`와 `expected`를 정한 뒤, simulate에도 **최소 expected 길이**를 요구한다.
비simulate의 정확한 길이 제한과 simulate의 후행 옵션 허용을 구별한다.
모든 인덱싱과 옵션 소비가 길이 확인 후 일어나게 한다. CLI 의존 라이브러리 전환은 필요 없다.
시험 편의를 위해 iterator를 받는 private parser helper를 분리할 수 있지만 실제 `parse`가 같은 helper를 써야 한다.
panic을 잡아 exit 2로 바꾸거나 REQUEST 기본값을 넣지 않는다.

필수 실제 재현: `author_content simulate unused-root unused-selection`은 파일 읽기 전에
typed `authoring_cli_input`, exit 2이며 panic/backtrace가 없어야 한다.
현재 재현 원자료는 [CLI_MISSING_ARGUMENT_REPRO](../reviews/wp02_d07_runtime_amount_001_review_001/CLI_MISSING_ARGUMENT_REPRO.json)다.
REQUEST가 필요한 explain과 필요 없는 validate/describe의 기존 동작도 보존한다.

### F02: legacy 대상별 amount 배열 사전 admission

`targets.len().checked_mul(size_of::<RuntimeAmount>())`로 계산하고
`admit_retained_growth(0, bytes, site)` 성공 뒤 `Vec::with_capacity`를 호출한다.
expression과 관측 on legacy가 동일 규칙을 사용하게 하며, 작은 내부 helper 재사용은 가능하다.
site는 계량/진단용으로 일관되게 유지하고 테스트 이름에 따른 production 특례는 두지 않는다.
관측 off 또는 catalog 없는 programmatic legacy에 불필요한 새 Vec/평가를 만들지 않는다.

시험은 두 층으로 나눈다. (1) 실제 사용되는 allocation helper의 checked overflow/정확 bytes/1 byte 부족 경계,
(2) actual legacy 다중 대상 executor를 낮은 native allowance에서 실행해 같은 admission site에서 거부되고
accepted gameplay·이전 record·ordinal이 보존됨을 확인한다. helper만 시험하고 executor 연결을 생략하지 않는다.
누적 admission은 현재 retained 크기와 다른 값이므로 성공 1회에서 관찰한 해당 operation의 요구량과 site를 근거로 삼는다.
단순히 다른 사전 검사에서 먼저 실패한 결과를 F02 경계 PASS로 쓰지 않는다.

## 5. F03 시험 보강 방법

### 공통 fixture와 query 경계

기존 `value_expr_runtime.rs`의 `ordered_fixture`, `owner_fixture`, `build_value_expr_delivery_capture`와
`execute_ordered_skill_effects`를 우선 사용한다. 관측을 켜고 실제 `ObservationKey`를 bind한 뒤 실행한다.
Snapshot은 **release 때 만든 carrier를 actual 소비 경로로 전달**한다. 소비 직전에 새 capture를 만들면
release 이후 변경을 시험하지 못한다. `DamageSourceSnapshot.value_expr_capture`와 기존 deferred-delivery 시험을 참고한다.

`snapshot_preserves_release_values_while_live_reads_effect_start_state`,
`released_member_values_and_tiles_remain_snapshot_while_live_advances`,
`release_result_snapshot_keeps_earlier_frontier_while_live_advances`,
`ordered_effect_uses_one_live_member_set_before_mutation_then_rebuilds_it`의 기존 runtime 증거를 출발점으로 삼는다.
관측 켜기·실제 소비 확인·record 획득·preview/explain assertion을 더해야 신규 관측 증거가 된다.

staged dynamic 수치 대조에는 기존 crate-private `encode_record_for_test`를 사용할 수 있다.
단, 그 helper는 selector를 검증하지 않고 직접 record를 받으므로 Latest/Exact·battle 교체·권한 검사까지
통과했다고 표시하면 안 된다. 그 항목은 `TrialSession`의 실제 선택 경로나 동일 private selector를 시험한다.
public legacy `d07_b15_public_trial_amount_preview_explain_and_exact_use_actual_legacy_execution`과 외부 CLI fixture는 유지한다.
동적 fixture를 공개 로더로 통과시키기 위해 production 차단을 해제하지 않는다.

### 필요한 관측 사례와 기대 결과

아래 C 번호는 이 지시서의 시험 추적용이다. 새 roadmap 단계가 아니다. 한 시험이 여러 조건을 입증해도 된다.

| ID / 원 요구 | 입력·절차 | 필수 기대 결과 |
| --- | --- | --- |
| C01 / F01, B18 | simulate REQUEST 누락, trace path 누락, 옵션 중복/unknown/과다, 다른 operation에 옵션 사용 | typed 입력 거부·exit 2·panic 없음. 정상 amount/trace 각각 및 양 순서 조합은 성공 |
| C02 / F02, B11 | 실제 공유 helper와 관측 on legacy 다중 대상, 정확 allowance와 1 byte 부족, checked 곱 overflow | allocation 전 지정 site의 typed 거부, accepted 상태 보존. 관측 off gameplay 불변 |
| C03 / B01 | catalog legacy Damage 음수/0/i32 최소, Heal 음수/0/i32 경계. 같은 입력의 관측 on/off | literal 기존 unsigned_abs/narrowing/zero 정책과 실제 command 소비 동일. HP 제한으로 최종 HP가 달라질 수 있어 command 입력도 확인 |
| C04 / B02–B04 | Snapshot에서 Source.Attack=10 release→20으로 변경→실제 소비→30으로 변경→두 query 반복 | 실제 소비와 기록·두 query는 10. frame/trace가 release 입력을 가리킴. target HP·resource·완료 result 입력도 각각 지원 fixture로 변화 전후 대조 |
| C05 / B03–B04 | Live release 당시 Attack=10→소비 전20→실제 소비→30→조회. 별도로 앞 effect가 바꾼 HP/result/resource 읽기 | 첫 사례 실제 소비/조회20. 앞 effect 결과를 읽는 사례는 변경 후 값, query는 이후 변경에 영향받지 않음 |
| C06 / B05 | Side/실제 Employee/지정 EquippedSlot, 명시적 제외, 서로 다른 grouping의 복수 aggregate node. release/consume 이후 member 상태 변경 | 실제 소비와 두 query의 value·narrowed amount·node/member trace 순서·내용 동일. node별 view 혼합 없음. 각 관계의 유효·부적격 입력 보존 |
| C07 / B06, B09 | 복수 effect·복수 target·같은 timestamp·piercing delivery. 기록 A 후 실제 B로 교체하고 A Exact 조회. zero와 무대상 impact 구별 | 기존 실행 순서대로 ordinal 증가/최근1건. A는 not-retained, B를 A로 반환하지 않음. zero는 기록, D04-F01 miss는 새 기록/ordinal 없이 후속 완료 |
| C08 / B07–B08 | 다른 battle/ordinal 선택, private 시험에서 generation/site/fingerprint/source-copy 결속 손상, action/delivery 정리와 unit 제거 | 정상 과거 record는 replay 가능. 잘못된 선택은 정해진 unavailable/invalid request, 손상된 record는 typed invariant. 현재 live action 재발급 없음 |
| C09 / B08, B13 | active→terminal→다음 전투→다음 terminal. controlled clone/timestamp scratch 및 반복 평가 | 소유 범위가 active와 마지막 terminal로 제한. 교체된 terminal Exact는 not-retained. clone은 immutable Arc 공유, record/trace 이력이 계속 쌓이지 않음 |
| C10 / B10 | 성공 record A 보존 후 뒤 target/뒤 effect 실패, timestamp/receipt 실패 또는 취소. 별도 operation에서 정상 재시도 | accepted gameplay·A의 identity/amount/trace·ordinal 모두 보존. 실패 중 B가 query에 노출되지 않음. 재시도는 정상 순서 |
| C11 / B11 | 새 record·old+new 교체 peak·frame/views/trace·active/terminal retained·clone의 실제 소유 경계 | 정확 한도와 부족 한도 대조, clone copy와 retained를 구별. source-only 수동 합계나 Arc 핸들 수만으로 PASS 불가 |
| C12 / B11, B14 | 동일 실제 record를 숫자 query의 work/metadata/result 경계 및 긴 member trace 중간 취소로 조회 | typed 제한/취소. accepted snapshot/RNG/queue/resources/identity/work/index/receipt 불변. 반복 성공 문서와 usage 결정적 |
| C13 / B12 | 동일 입력에서 optional trace off/on/포화, 일반 event log off/eviction | 필수 amount/node/member trace 동일. 작은 trace 한도로 포화를 만들며 긴 전투 시험 불필요 |
| C14 / B18 | staged 실제 실행의 중간 값이 `2^100 + 1` 등 큰 i128이고 마지막 값은 Damage/Heal narrowing 범위 안 | 중간 trace의 정확한 십진수와 순서가 typed query/출력 원문에 유지. Value/f64를 oracle로 사용하지 않음 |
| C15 / B15–B17 | 실제 외부 legacy TrialSession·CLI, authored 공개 거부, static explain, DamageExpr/mixed usefulness | CLI 최대4문서와 kind/selector 유지, DamageExpr/HealExpr는 공개 거부, static performed=false, 미래량 Indeterminate·기존 eligibility 보존 |
| C16 / B19–B20 | 선택 target의 회귀/비용·전후 identity·복원 | 실행/재사용 구별, 무효화된 근거 재검증, 검토 가능한 target 고정 |

C04/C05의 예시 수치는 Attack 사례의 기대값이다. HP/resource/result는 실제 계약과 단위에 맞는 별도 식/상태로
기대값을 명시한다. 지원되지 않는 WP03 stat을 가짜 입력으로 채우거나 단위 규칙을 우회하지 않는다.

### 비용·손상·큰 정수 시험에서 지킬 경계

- retained admission은 **할당 전**, record commit은 **실제 effect/settlement 성공 후**라는 두 순서를 함께 지킨다.
  보관 중인 record만 계산해 후보/평가 scratch의 공존 peak를 생략하지 않는다.
- terminal retained 요구는 아직 입증되지 않았다. `capture_authoring_terminal`의 소유권 이전부터 다음 clone/query까지
  실제 계량 경로를 먼저 적고, 기존 한도 계약에서 빠진 payload가 있으면 닫힌 owner의 cached extent와 preflight로 교정한다.
  Arc 복사는 핸들 비용으로 유지하되 마지막 owner가 보존하는 payload를 누락하지 않는다.
  전투/세션 전체의 새 합산 상한이나 거부 정책을 임의로 만들지 않는다. 기존 계약으로 결정할 수 없으면 근거와 선택지를 묻는다.
- invariant 손상은 private 테스트 안에서만 만든다. 테스트용 변조 생성자를 public API로 내보내지 않는다.
  source-copy는 당시 고정된 결속과 비교하며 이미 제거된 현재 unit/item을 재조회해 정상 과거 기록을 거부하지 않는다.
- 취소는 잠들기/벽시계 경합에 의존하지 않고 기존 제어 경계 또는 test-only 관찰 지점에서 결정적으로 발생시킨다.
  production에 특정 시험 이름/fixture를 인식하는 분기를 넣지 않는다.
- C14는 예를 들어 큰 i128 Const에서 큰 값을 빼 최종 1을 만드는 유효 식으로 실제 소비를 만든다.
  동적 expression은 public CLI에서 차단돼 있으므로 공개 차단을 풀어 CLI 시험을 만들지 않는다.
  staged actual record→실제 query encoder의 큰 정수 원문과 CLI `line`의 byte 보존을 각각 확인하고,
  외부 CLI의 실제 실행은 legacy로 검증한다. 이 조합을 ‘public dynamic CLI 지원’으로 보고하지 않는다.

## 6. 실행 중 기록과 근거 재사용

첫 교정의 실행 기록 위치는 `execution/single_goal/corrections/wp02_d07_b_review_001/`을 권장한다.
존재하면 내용을 먼저 확인한다. predecessor segment·root의 과거 동결 PLAN/NOTES를 덮어쓰지 않는다.

| 역할 | 이번 경로 | 기록 방식 |
| --- | --- | --- |
| 현재 계획/위치 | `corrections/wp02_d07_b_review_001/PLAN.md` | 이 지시서 링크, 이번 범위·before·선택 checkpoint·최신 target·남은 F/C 번호·다음 행동 |
| 수행 이력 | 같은 디렉터리 `EXPERIMENTS.md` | 시도/가설, 명령·source·환경, 관찰·실패·교정·재검증과 원자료 |
| 현재 판단 | 같은 디렉터리 `EXPERIMENT_NOTES.md` | 채택/폐기 이유, 미검증 가정, 재발 방지, 정책 질문 |

시작·의미 있는 실패·설계 변경·검증 결과·인계 때 갱신한다. 실패를 삭제하거나 종료 직전에 기억으로 몰아서 쓰지 않는다.
같은 시도를 반복하면 달라진 가설/입력을 적는다. 이력의 과거 추정을 NOTES에서 현재 사실로 복제하지 않는다.
세 역할의 동등한 기존 교정 문서가 있다면 경로를 PLAN에 지정하고 재사용한다.
작업 전체 진입점은 기존 ROADMAP_STATUS이며, 새 전역 registry는 만들지 않는다.

조건표에는 기존 B01–B20과 C01–C16을 연결하고 아래 네 종류를 구별한다.
`기존 runtime 회귀 / 신규 관측 시험 / source 근거 / 미검증`.
각 시험은 실제 이름·입력·기대 결과·관측 설정·명령·nonzero 실행 수·원자료·target을 연결한다.
기존 source 동일 부분의 증거는 영향 판단 후 재사용하되 새 기능의 시험을 대신하지 않는다.

## 7. 구현 순서와 첫 단계검토

1. 시작 target 대조와 실제 before 보존, 세 기록 역할 준비. 기존 실패 이력과 직접 관련 시험부터 읽는다.
2. F01과 C01을 처리한다. 실제 CLI 정상/비정상 경로가 닫히면 이것만으로 의미 있는 검토 단위가 된다.
3. 다음 요청으로 재개할 때 F02/C02 및 연결된 기존 record 보존을 처리한다.
4. 이후 F03의 동적 입력/조회(C03–C08), 수명·실패·계량(C09–C13), 출력·통합(C14–C16)을 실제 결합에 맞춰 진행한다.

2–4는 후보 순서이며 미래 checkpoint 개수를 고정하지 않는다. **처음으로** 다음 조건이 함께 성립하면 다음 큰 작업 전에 멈춘다.
검토 가능한 책임/동작이 완성됨, 빌드·그 부분의 필수 검증과 영향 회귀가 통과함,
구현/미구현과 남은 의존을 분리해 안정적으로 동결할 수 있음.
이번 source에서 F01은 독립적인 CLI 교정이므로 첫 후보로 적합하다. F02/F03이 남았다고 이 경계를 무시하지 않는다.
후보를 합쳐야 할 실제 의존이 있으면 PLAN에 이유를 적되, 단순 편의로 전체 R까지 범위를 넓히지 않는다.

첫 CLI 구간에는 실제 바이너리 빌드, parser/실제 CLI 시험, amount/trace 정상 fixture와 오류 행렬,
변경 target의 check/clippy·format을 확인한다. CLI가 호출하는 core 라이브러리의 byte가 같다면
그 라이브러리 전체 시험과 PC36/R05는 선행 근거의 source/환경 적용성을 기록해 재사용할 수 있다.
선택한 부분의 필수 검증 실패를 남긴 채 ‘검토 준비 완료’로 중단하지 않는다.

native executor/관측 소유권/한도까지 교정한 뒤 F01–F03 전체를 닫는 target에서는 다음 명령을 최종 source에서 수행한다.

```bash
cargo check -p game_core --all-targets --offline --locked
cargo test -p game_core --lib authoring --offline --locked
cargo test -p game_core --lib value_expr --offline --locked
cargo test -p game_core --all-targets --offline --locked
cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op
```

작업 디렉터리는 repository root, Cargo는 `/home/blast/.cargo/bin/cargo`를 사용할 수 있다.
검증된 offline cache는 `CARGO_HOME=/tmp/wp02-cargo-home`이다. build output은 별도 경로를 쓰고 실제 환경을 기록한다.
제품 전체 시험은 기본 병렬로 실행한다. 전체에 `--test-threads=1`을 붙이지 않는다.
변경 파일의 format을 검사하며, 무관한 파일을 전체 formatting한 diff를 교정에 섞지 않는다.
추가 C 조건은 실제 selector/CLI 명령으로 수행하고 0건 성공은 인정하지 않는다.

native admission/소유권이 바뀐 target은 PC36과 R05를 재측정한다. 기존 workload의 관측 off 회귀와
관측 on/off 실제 gameplay 대조를 구별한다. PC36 off 성공만으로 on 비용/동작까지 같다고 주장하지 않는다.
20분 시험을 추가하지 않는다. 로그는 실행 시작부터 저장하고 진행을 표시하며 중단·실패·재시도도 보존한다.

## 8. successor 고정과 검토 인계

각 checkpoint의 실제 before/after와 신규 파일 부재↔존재를 보존한다. 첫 before는 위 340개 대상이다.
새 test/source/config 파일을 빠뜨리지 말고 필요한 파일 집합을 확장한다. manifest 해시만으로 원본 보존을 대신하지 않는다.
기존 freeze/run 도구의 방법을 재사용하되 새 교정 디렉터리에서 실행한다.
predecessor의 하드코딩된 성공 건수/파일 수/기대 digest를 그대로 베껴 성공 판정을 만들지 않는다.

기록 소유 위치와 target identity는 PLAN 한 곳에서 연결한다. 각 인계에는 다음을 넣는다.

- 실제 교정한 F/C 범위와 재현 before→after, 나머지 항목의 미완료 표시.
- 구현 방향에서 달라진 부분·이유, 실패와 재검증 이력, 기존 시험 재사용/새 실행의 구분.
- 실제 runtime 입력→command 소비→record→두 query의 동적 사례 원자료와 해당 조건표(이번 범위일 때).
- 전후 archive/diff, manifest SHA-256, currentness와 격리 정·역 복원 결과.
- 바로 이전 target 대비 변경과 최초 D07-B target 대비 누적 변경, 후속 검토에 필요한 문서·소스·로그 경로.

마지막에는 **부분 교정·자체 검증 종료 / successor target 동결 / 단계검토 요청 대기**로 종료한다.
사본 대화에 인계서·manifest 경로를 전달한 뒤 검토를 시작한다. 같은 작업 디렉터리의 제품 수정은 구현 대화 한쪽에서만 한다.
수정한 구현자가 F01–F03 검토 PASS를 선언하지 않는다. 부분 PASS와 전체 D07-B/R06 완료를 구별하며,
검토 뒤 다음 교정 구간은 사용자의 재개 요청에 따라 수행한다.
