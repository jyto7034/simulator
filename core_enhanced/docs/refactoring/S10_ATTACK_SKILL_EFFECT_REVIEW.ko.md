# S10 공격·스킬·효과 대표 조사와 개선 계획

작성·갱신일: 2026-10-18

현재 단계: **대표 조사·R1/D1 구현·영향 검증·분리된 완료 검토 완료**. 구현 결과는 §7에 기록한다.
§1–5는 변경 전 조사·계획과 근거다. 당시의 제품 변경 없음·새 테스트 0회·미구현 표기는 그 시점의 기록이다.
다음은 §6의 X01 대표 조사다. 전체 효과 구현의 무결점이나 Core 완료를 뜻하지 않는다.

## 1 범위·기준과 재사용 근거

[출발점](README.md)과 [S09 인계](S09_MOVEMENT_BLOCKING_REVIEW.ko.md#6-다음-재개-위치)에 따라
기본 공격의 대상 획득→windup→release→피해, 스킬의 접수→flow/step→delivery→효과 정산,
취소·철수 뒤 이미 방출한 효과의 수명을 대표 흐름으로 선정했다.
[판단 기준](../refactoring_criteria.ko.md)의 책임·소유권·전이·중복·변경 비용과
[실행 지침](../system_refactoring_execution_guide.ko.md)의 근거 재사용·조사 종료·분리된 검토를 적용했다.

- [선택 입력](evidence/s10_001/baseline.json)은 57개다. 관련 소스·테스트·fixture·manifest·설정 등
  56개는 기존 지도와 S09까지의 승인된 변경 기록에 일치했다. 타겟팅 계약 1개는 이번에 새로 식별했다.
  전체 콘텐츠·의존성의 새 전수 동결은 아니다.
- [기존 통과 35개](evidence/s10_001/reused-tests.json)는 전수 로그의 관련 27개와 S09 수정 후 8개다.
  S08 R2/S09 R1의 한정된 변경은 기존 구조 대조·완료 검토와 최신 영향 검증으로 연결했다.
  이번에는 Cargo·테스트를 실행하지 않았다. 과거 전수 결과를 현재 전체 통과로 다시 집계하지 않는다.
- 계약은 [스킬·타겟팅](../skill_target_contract.md)의 admission/step/interrupt/projectile/area/settlement와
  [runtime](../core_runtime_contract.md)의 source·action·typed resource·취소/정산 경계를 사용했다.
- 실제 입력 연결은 `tests/fixtures/u08/carrier001/skills.ron`의 literal/projectile/GroundFixed/follow 정의와
  `world/tests/u08_carriers.rs`, `u09_structural/flow_cases*`의 RON→validated catalog→world 명령→scheduler→HP/fact 경로다.
  이 테스트들은 전투 전 catalog·소유/장착·HP 준비에 내부 seam을 사용한다. 정상 acquisition 전체나 Unity 실행의 증거는 아니다.
  `boss_action_tests.rs`는 authored catalog와 실제 kernel scheduler를 사용하지만 scenario/일부 상태는 내부 fixture다.

모든 스킬·effect variant, 모든 보호/반격·보스 task·유물 인과/구조 규칙과 공식 pack의 전수 동작은 이번 상세 조사 밖이다.
그 경계는 아래 대표 흐름의 실제 소비 지점까지만 확인했다. 수치식 평가기·자원 lease 전체는 X01과 연결할 후속 영역이다.

## 2 대표 흐름과 상태 소유권

| 단계 | 실제 owner·호출 흐름 | 유지할 의미 |
| --- | --- | --- |
| 입력·정의 | validated `SkillDef`, authored effect의 `op`·source와 compiled flow. `effective_skill.rs::skill_for_source`가 causal/effective binding 또는 catalog를 선택 | 이미 방출한 carrier는 자신의 source binding으로 정의를 해석. 현재 장착/선택을 뒤늦게 다시 읽어 대체하지 않음 |
| 기본 공격 선택 | `basic_attack.rs::select_basic_attack_target`, `targeting.rs`, `target_usefulness.rs` | 기본 공격 profile과 skill rule은 별도. block/current target/stance/range/usefulness의 순서와 결정론 유지 |
| windup·release | `handle_basic_attack_start_event`→resolve, epoch/lock/lifecycle 확인→`resolve_committed_basic_attack` | 미방출 취소와 방출된 projectile 구분. release가 최종 range gate이며 projectile은 locked target과 launch source를 보유 |
| 스킬 접수 | `sim.rs`의 manual/auto admission, pending cast와 start sequence, accepted unit/tile | player target과 automatic selection 구분. accepted target·tile, focus lock과 비용의 수명 보존; stale cast-end는 새 접수를 실행하지 않음 |
| invoke·flow | `invoke_skill` 계열→`ActiveSkillCast`/`FlowState` 생성→`drive_skill_flow`→`SkillStep` | accepted action, cast, authored node, 반복 occurrence는 다른 identity. repeat reset이 방출된 occurrence를 무효화하지 않음 |
| step·대상 | `execute_skill_step`, `resolve_skill_step_context/targets`, `target_usefulness.rs` | reuse/retarget, cast acquisition과 실제 delivery footprint를 분리. 이전 피해 gate는 Pending/Unsatisfied/Satisfied를 구분 |
| delivery | Instant는 즉시 effect 실행, Projectile은 launch snapshot·motion/hit/stop, TileArea는 geometry·tracking·tick history | 같은 effect executor를 공유하되 carrier의 이동/종료 정책은 각 모듈이 소유. S09 segment 보간을 projectile sampling에서 사용 |
| 효과·피해 | `execute_ordered_skill_effects`→effect별 target/amount 준비→command→`execute_committed_damage`→HP/fact | 한 effect의 대상별 amount를 먼저 평가하고 다음 effect는 실제 정산 prefix를 읽음. launch source와 live target 방어를 구분; actual HP loss를 후속 gate에 사용 |
| 정산·회수 | `SkillStepProgress`, deferred steps/active areas, `cleanup_finished_skill_cast`, native delivery ticket | control terminal과 finite EffectsSettled는 다름. 살아 있는 유한 작업·side flow·area가 남으면 필요한 문맥을 보존 |
| 실패·공개 | S08 timestamp candidate와 world mutation staging, control/native poll | admission/정산 실패가 후보를 폐기하고 기존 성공 prefix를 보존. `Option`/조기 반환만 보고 성공으로 해석하지 않음 |

`ActiveSkillCast`는 cast 단위 source·선택 anchor·step 진행·flow·impact 문맥을 소유한다.
`ActiveProjectileRuntime`/`AreaRuntime`은 방출된 carrier 상태와 source를, `RuntimeFactSource`의 action/delivery는
실제 권한·관찰/자원 수명을 연결한다. source의 Arc 소유만으로 새 control 권한을 부여하지 않는다.
`Sustained`의 적용된 효과 lease와 channel의 제어 수명도 동일하지 않다.

Projectile impact는 실제 결과를 적용한 뒤 kill stop과 motion terminal을 합성한다. 기존 append-only impact log를
사후 수정하지 않으며 terminal delivery는 한 번 닫힌다. Area는 tracking별 geometry와 EveryTick/OncePerArea/OnEnter
이력을 소유하고 expiry에서 cast의 area 목록과 delivery를 정리한다.

철수는 pending/unreleased 동작과 source-bound follow area를 취소하고 released projectile/GroundFixed 효과는 유지한다.
structural side branch가 있는 cast는 제어를 닫아도 시작된 delivery의 결과 범위와 source를 정산까지 유지한다.
`rv001_released_child_repeat_keeps_scoped_prefix_after_withdrawal_and_cc`는 철수 후 active cast 1개, 후속 실제 HP 감소,
최종 회수까지 확인한다. `cancel_structured_flows`와 withdrawal 전용 함수는 allow-dead·side 여부·terminal 사유가 달라
하나의 범용 cancel 함수로 합치지 않는다.

## 3 설계 평가와 판정

| 평가 항목 | 판정과 근거 |
| --- | --- |
| 책임·의존성 | cast/flow, projectile, area, damage, effective source의 경계 유지. 큰 `sim.rs`의 줄 수만으로 분리하지 않음. effect 순서와 source/정산 context가 실제 연결 책임임 |
| 상태 소유권 | pending control, cast occurrence, released carrier, native settlement의 분리를 유지. 동일한 사실의 중복 authority로 취급하지 않음 |
| 수명·전이 | 정확한 epoch/token과 별도 carrier 종료가 stale event·재실행을 제한. retained cast를 무조건 누수로 판단하지 않음. 설명 불일치 **D1** |
| 실패·commit | 후보 실행·임시 flow 소유 복원·외부 poll 경계 유지. 실패를 no-target/성공으로 바꾸는 새 fallback을 만들지 않음 |
| 규칙 중복 | whole-step ModifyDamage 합산이 production 두 구현과 test-only 한 구현에 복제. **R1** |
| 변경 비용 | R1은 admission preview와 실행 결과가 공유해야 할 한 규칙의 수동 동기화 문제. shared damage formula 자체는 이미 `DamageModifiers::merge`/damage kernel을 사용 |
| 데이터·공개 범위 | authored metadata와 `op`, cast target과 delivery shape를 구분. R1에 schema/공개 enum/DTO 변경 불필요 |
| 결정론 | target의 stable order, per-occurrence strike, source identity 기반 RNG와 event cause 구분을 유지. R1도 authored modifier 순서를 변경하지 않음 |
| 관측·결과 | action terminal/EffectsSettled·실제 HP loss와 optional log를 구분. D1은 설명 교정이며 cast control이나 cancellation 결과 변경이 아님 |
| 검증 가능성 | world 입력·실제 HP/fact 검증과 내부 fixture를 구분. test-only `build_skill_step_commands`는 production ordered executor 전체의 대체 증거가 아님 |
| 성능·자원 | 반복 skill clone, target scans, cast cleanup scan과 impact context 복사 신호가 있으나 시간/할당/최대 규모 측정 없음. 현 단계에서 최적화·성능 결함으로 확정하지 않음 |
| 저장·외부 소비 | active-battle durable resume는 적용 대상이 아님. Unity 실물·release/feature 전 조합은 이번에 확인하지 않음 |

선택 흐름에서 **확정한 gameplay 동작 오류는 없다**. 아래 후보는 동작 보존 구조 개선과 설명 교정이다.

### R1 — 스킬 step의 피해 보정 합산 규칙 통합

[정적 대조](evidence/s10_001/findings.json)의 세 식은 타입 경로·공백·Default 표기 외에 같다.

1. `target_usefulness.rs::skill_step_damage_modifiers`: 실제 delivery usefulness와 damage preview에서 사용.
2. `sim.rs::execute_ordered_skill_effects`: 실제 Instant/projectile/area 효과 실행에 사용.
3. `sim.rs::build_skill_step_commands`: `cfg(test)`의 literal command 조립 helper에 사용.

같은 step의 모든 `ModifyDamage`를 effect 순서대로 0에서 `DamageModifiers::merge`하는 규칙이다.
그 결과가 effect 위치와 무관하게 해당 step의 Damage 연산에 적용된다는 계약은 하나인데,
selector와 executor가 각각 추출/fold를 정의한다. 지원되는 여러 보정의 조합이나 보정 해석을 수정할 때
admission의 예상 유효성과 실제 피해가 어긋나지 않도록 여러 구현을 수동으로 맞춰야 한다.
현재 세 구현의 계산 결과가 다르다는 발견은 아니다.

권장 구조는 기존 `skill_runtime` 모듈의 작은 core 내부 순수 함수 하나다.
기존 helper를 그곳으로 이동하고 production 두 소비 경계와 test helper가 같은 규칙을 사용한다.
타겟 선택 모듈을 실제 damage 실행의 규칙 owner로 삼거나 별도 trait/registry/cache를 만들지 않는다.
피해 공식·effect 전체 실행기·carrier는 통합 대상이 아니다.

### D1 — 취소된 control과 남은 정산 문맥의 설명 구분

- `skill_target_contract.md::Skill Cast Interrupt`의 철수 시 active cast 전부 제거 설명과
  `core_runtime_contract.md::Native runtime facts and action observation`의 pending/cast/Follow owner 제거 설명은
  structural side cast의 retained settlement 문맥을 구분하지 않는다.
  `build.rs::cancel_active_skill_runtime_for_withdrawn_unit`와 `flow.rs::cancel_structural_side_cast_for_withdrawal`은
  side cast의 제어를 취소하고 시작된 delivery의 정산이 끝날 때까지 cast 문맥을 유지한다.
- `types.rs::ActiveSkillCast`의 `active_area_ids`와 `last_impact_context`는 아직 migration용 예약 필드라고 쓰여 있다.
  전자는 area 등록/종료·cast 회수 gate에서, 후자는 현재 저장 비용 계산·debug projection에서 사용한다.
  이전 step anchor 조회는 `impact_contexts_by_step`를 사용한다. 이 설명을 바로잡되 필드·비용·debug 계약은 유지한다.

## 4 구현 계획과 수용 조건

R1을 작은 동작 보존 변경으로 구현하면서 D1을 교정하고, 영향 검증 뒤 별도 읽기 전용 완료 검토를 수행한다.

- **제품 범위:** `skill_runtime/mod.rs`, `target_usefulness.rs`, `sim.rs`의 세 fold와 두 기존 helper 호출.
  순수 합산 함수는 core 내부로 제한한다. D1은 계약 두 곳과 `types.rs`의 해당 주석만 교정한다.
- **보존 조건:** authored 순서·모든 ModifyDamage 포함·zero default·포화 합산·non-modifier 제외,
  Literal/Expr의 기존 공통 입력, source snapshot/live target 의미, 호출부의 charge/poll 위치와 횟수.
  합산을 캐시하거나 정렬·재그룹화하지 않는다. signed saturating merge의 결합 순서를 보존한다.
- **안전망:** 기존 usefulness/실제 modifier HP·projectile/area·실패 복구 근거를 재사용한다.
  이번에 재사용한 실제 HP 시험의 ModifyDamage 예는 한 개 보정이므로, 구현 전에 여러 보정이 Damage 전후에 섞인 경우의
  admission/usefulness와 실제 HP를 함께 확인하는 작은 사례가 필요한지 최종 대조한다.
  공백이 남으면 기존 fixture로 해당 계약만 보강하고 변경 전 먼저 실행한다. fold 구현을 복제한 테스트는 만들지 않는다.
- **변경 후 검증:** 선택한 target usefulness, 실제 literal/expr damage, projectile/area와 timestamp failure 사례를
  집중 실행한다. 테스트 전용 command helper만 통과한 것을 실제 executor 검증으로 보고하지 않는다.
  공유 상태/정책 변화나 예상 밖 실패가 없으면 전수 재실행은 하지 않는다. D1 문서 수정은 소스 대조로 검증한다.
- **구조 수용:** whole-step modifier 추출/fold 구현 3곳→1곳, production 및 test helper 소비 연결을 확인한다.
  기존 formula·public API·state·schema는 유지하고, 새로운 별도 피해 계산 경로가 생기지 않아야 한다.
- **분리된 검토:** 고정 diff·원시 실행 결과·입력과 §3 문제를 다시 대조한다. R1의 무관한 effect 분기와
  cast 진행/취소 순서가 바뀌지 않았는지, D1이 이미 방출된 효과를 새 control로 설명하지 않는지 확인한다.

cast progress 생성/정산의 일부 비슷한 구문은 현재 서로 다른 pending count·sustained·terminal 처리를 가진다.
R1에 이를 함께 추출하지 않는다. test-only literal command helper 전체의 대체/삭제나 legacy 경로 전환도 별도 조사 대상이다.

## 5 검증 근거·한계와 조사 완료 검토

[재사용한 원시 로그 위치](evidence/s10_001/reused-tests.txt)는 다음을 보호한다.

- 기본/skill target authority 분리, zero damage usefulness와 실제 area 대상/뒤쪽 유효 step.
- basic projectile의 launch source·live 방어·locked target, skill projectile의 사후 사망 source·kill/hit/expiry 단일 종료.
- mixed target·projectile 후속 피해 gate·miss 정산, area 다중 피해 뒤 self heal, sustained children 대기.
- 실제 modifier·저항·유물 조합 HP, 철수 후 released area/projectile/side child 정산, 닫힌 Activation 재사용 거절.
- 최신 S09 이후 manual/auto cast·Hard CC·channel·timestamp 실패 복구·사망/철수 buff 회귀.

이 35개는 현재 선택한 대표 조사에 재사용한 과거 실행 결과다. 전체 S10·모든 effect 조합의 새 통과가 아니다.
실행 명령·기본 feature·환경은 [기존 전수 명령](evidence/s08_full_001/commands.json)과
[S09 집중 명령](evidence/s09_r1_001/commands.json)이 소유한다. 제품·테스트·fixture·설정 변경이 없어 재실행하지 않았다.
R1의 중복은 정적 분석으로 확인했고, D1은 source·실제 retained cast assertion·기존 통과 로그로 연결했다.

미확인 범위는 §1과 §3에 남겼다. 전체 effect/protection/relic graph, 최대 규모 비용, Unity·release/feature 조합을
확인했다고 확대하지 않는다. S08 queue comparator 24쌍/31 variant 한계도 이번에 재검사하지 않는다.

작성 뒤 source·계약·후보·계획과 원시 로그를 별도 시점에 대조한
[조사 완료 검토](evidence/s10_001/review.json)는 같은 실행자의 읽기 전용 검토이며 독립 감사가 아니다.
조사 당시 판정은 **선택한 대표 조사·계획 완료**였다. 이후 R1/D1 구현 완료 근거는 §7과 구분한다.

## 6 다음 재개 위치

후속 실행 제어·자원 수명 대표 조사는 [X01 기록](X01_RUNTIME_RESOURCE_REVIEW.ko.md)에서 완료했다. 아래는 당시 인계다.

**X01 공통 실행 제어·typed resource 수명 대표 조사, 연결된 X02 검증 지원 확인 → 유지/개선 판정·계획**이다.
`game/battle/runtime_control.rs`, `core/resource_runtime.rs`, `world/resource_runtime.rs`, `game/resource/**`에서
실제 action/Delivery의 자원 접근·lease 종료와 예산/취소 실패가 후보 밖으로 전파되는 대표 흐름을 선정한다.
`core/value_expr_runtime.rs`는 그 흐름의 소비 경계로 연결하고 수치식 전체 조사 범위는 근거를 보고 정한다.

S08 timestamp/실패 보존, S09 이동, S10 cast/source/carrier 수명 근거를 재사용한다.
X02는 선택 target·feature·fixture·실제 실패 도달 여부와 기존 근거의 유효성을 확인하는 범위로 함께 다룬다.
시스템 전체·모든 테스트를 다시 조사하거나 전체 Core 완료로 넘어가지 않는다.

## 7 R1/D1 구현·검증과 완료 검토

### 변경과 구조 효과

- `skill_runtime/mod.rs`의 core 내부 순수 함수 `skill_step_damage_modifiers`가 whole-step 합산을 소유한다.
  기존 `target_usefulness.rs` helper를 이동했고, usefulness의 두 호출과 `sim.rs`의 실제 ordered executor·
  test-only command builder가 이 함수를 사용한다. **동일 규칙 구현 3곳→1곳, 호출 4곳**이다.
- `ModifyDamage` 선택, authored 순서, 0 기본값과 `DamageModifiers::merge`를 그대로 유지한다.
  호출부 charge/poll, effect 순서와 source/carrier/취소/정산 코드는 변경하지 않았다.
  공개 API·상태·schema·cache를 추가하지 않았다. target selector가 실제 executor의 규칙 owner가 되는 의존도 피했다.
- D1은 [타겟팅 계약](../skill_target_contract.md)과 [runtime 계약](../core_runtime_contract.md)에서
  철수로 취소된 제어와 structural side cast의 retained settlement 문맥을 구분했다.
  `types.rs`의 두 주석도 area 수명 gate와 latest-impact의 저장 비용/debug 역할로 교정했다.
  필드·비용 계산·debug 출력·취소 정책은 유지했다.

[변경 diff](evidence/s10_r1_001/source.diff)와 [구조 대조](evidence/s10_r1_001/structure-verification.json)는
helper 본문 동일성, `sim.rs`/`target_usefulness.rs` 전체의 지정 이동·호출 교체 외 동일성,
`types.rs`의 주석만 변경, 기존 테스트 영역에 한 사례만 추가한 것을 확인한다.

### 검증과 준비 교정

[변경 전 입력](evidence/s10_r1_001/baseline.json) 57개 모두 S10 조사 입력과 일치했다.
기존 35개 통과는 변경 전 근거로 재사용했다. 복수 보정이 Damage 전후에 있는 경우의 공백을
`whole_step_damage_modifiers_match_target_usefulness_and_actual_hp` 한 테스트로 보강했다.
기존 내부 unit/skill fixture와 실제 target selector·invoke/queued step·ordered executor를 사용한다.

- true damage 100에 앞쪽 25%·뒤쪽 50% 증폭을 함께 적용해 대상 선택과 실제 HP 감소 175·cast 회수를 확인한다.
- 앞쪽 60%·뒤쪽 40% 감소로 전체 예상 피해가 0일 때 자동 target 선택이 제외되는 것을 확인한다.
  제외된 경우에는 cast를 실행하지 않는다. 이는 공개 world admission 전체를 시험했다는 주장이 아니다.

첫 빌드는 새 테스트에서 `QueuedEvent`를 원시 `BattleEvent`로 직접 비교해 E0308로 실패했다.
포장된 event의 view를 검사하도록 수정했으며 입력·기대값·제품 동작은 바꾸지 않았다.
[초기 실패](evidence/s10_r1_001/baseline-test.log)와 [명령/결과](evidence/s10_r1_001/baseline-command-result.json)를 보존했다.

| 실행 | 결과 | 시간·근거 |
| --- | --- | --- |
| 보강 테스트, 제품 변경 전 재실행 | 1개 통과·실패 0 | 38.77초(빌드 37.56초, 테스트 1.16초). [명령](evidence/s10_r1_001/baseline-command002.json), [결과](evidence/s10_r1_001/baseline-command002-result.json), [원시 로그](evidence/s10_r1_001/baseline-test002.log) |
| 변경 후 집중 회귀 | 13개 통과·실패 0·ignore 0 | 48.18초(빌드 43.58초, 테스트 4.53초). [명령](evidence/s10_r1_001/commands.json), [결과](evidence/s10_r1_001/commands-result.json), [원시 로그](evidence/s10_r1_001/regression.log) |

13개는 보강 사례, zero/area/뒤쪽 step usefulness, 실제 modifier·저항·유물 HP, literal/snapshot-expression projectile,
area 피해→self heal, sustained child 대기, 철수 후 side cast 정산, flow/timestamp 실패 복구와 test-only command helper를 포함한다.
기본 feature의 offline/locked 환경을 사용했고 기존 경고 33개가 남았다. 전수 재실행은 하지 않았다.

### 분리된 완료 검토와 한계

구현·집중 테스트 종료 뒤 같은 실행자가 고정 diff·소스·원시 로그·§4 수용 조건을 별도 읽기 전용 단계에서 대조했다.
[완료 검토](evidence/s10_r1_001/review.json)의 판정은 **선택한 R1/D1 완료**, 발견 사항은 없다.
비참여 독립 감사는 아니다. 현재 서로 다른 합산 구현의 수동 동기화 부담이 제거됐고 계산·수명 의미는 보존됐다.

실행 중인 명령과 미해결 테스트 실패는 없다. 기존 35개 재사용 결과와 이번 13개 변경 후 통과는 구분한다.
§5의 전체 effect/보스/유물·최대 규모·Unity·release/feature·S08 comparator 한계는 유지한다.
이 구현 종료 당시 X01/X02 후속 대표 조사는 미착수였으며, 이후 결과는 [X01 기록](X01_RUNTIME_RESOURCE_REVIEW.ko.md)이 소유한다.
