# R06 WP02 D04-F01 교정 구현 지시서

작성일: 2026-09-26 KST.
대상: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`, `WP-CAVER-02`.
상위 인계: `HA-CAVER-P2G60-001`.

사용자는 계획·설계·독립 검토를 원 대화에서, 제품 구현을 별도 대화에서 진행한다.
이번 범위는 독립 검토 finding **D04-F01 하나의 교정과 자체 검증**이다.
기존 D04 설계나 Goal을 대체하지 않으며, 작성 대화에서는 제품 코드를 수정하지 않았다.

## 1. 구현 담당자에게 전달할 요청

> 이 지시서와 독립 검토 보고서를 읽고 **D04-F01을 교정**해줘.
> 현재 D04 동결본을 시작 기준으로 삼고, target-locked projectile의 대상 소실 후 빈 impact가
> 기존대로 정상 완료되도록 고쳐줘. 실제 target이 있는 경우의 capture identity 검사와 집계 의미는 유지해.
>
> mutation 전에 currentness를 확인하고 새 교정 구간에 before baseline·실행 PLAN을 남겨줘.
> 재현 4건을 제품 회귀 시험으로 옮기고, 빈 impact의 effect 처리·step 결과·delivery 종료까지 검증해.
> 기존 동결본·실패 로그·검토 기록은 보존하고, 교정 후 별도 successor target을 동결해줘.
>
> 구현·자체 검증과 재검토 인계가 끝나면 최종 응답으로 중단해.
> 독립 재검토와 다음 범위 결정은 원 대화에서 할 거야. ByOwner·D07·WP03/04로 이어서 구현하지 마.

## 2. 읽을 자료와 정확한 시작점

1. [독립 검토 보고서](../reviews/wp02_d04_resolved_target_001_review_001/REVIEW.md): **수정 필요**, finding D04-F01.
2. [재현 probe 원본](../reviews/wp02_d04_resolved_target_001_review_001/projectile_probes.rs),
   [D04 실패 로그](../reviews/wp02_d04_resolved_target_001_review_001/projectile_probes_001.log),
   [선행 상수식 통과 로그](../reviews/wp02_d04_resolved_target_001_review_001/predecessor_control/scalar_projectiles_001.log).
3. [D04 구현 인계서](../segments/wp02_d04_resolved_target_001/REVIEW_HANDOFF.ko.md),
   [D04 동결 manifest](../segments/wp02_d04_resolved_target_001/target/d04_resolved_target_001/MANIFEST.json).
4. [D04 구현 지시서](WP02_D04_IMPLEMENTATION_REQUEST_20260926.ko.md)의 설계·비용·검증 원칙,
   [기존 상세 설계](../../../CODE_REVIEW_003.ko.md)의 D02/D03/D04/D06/D08,
   [현행 GEX](../../../../../goal_execution_pipeline%281%29.md)의 교정·증거·단계검토 계약.

교정 시작 target SHA-256:

```text
24bd3b7e748b94037b2ec57822a7b7ef2b67406b9d2635a3a23a6b3eac7ba8bf
```

시작 source 333개 digest:

```text
4c63ca018cd39d50e4a896711debaaf7970dc2636841a77eb74457f83313582f
```

더 이전 action/Snapshot target `b16697bfe8c518944f0bddcf800b0f1e005ed639edd6fe915e4d96c726a6e1f4`는
상수식 target 소실의 정상 동작을 비교한 대조군이다. D04 구현을 버리고 그 버전으로 돌아가는 교정은 하지 않는다.
작성 시 D04 `freeze.py verify`의 currentness·양방향 복원 PASS를 확인했다. 구현 시작 시 다시 확인한다.

## 3. 원인과 보존할 실행 계약

현재 흐름은 다음과 같다.

1. target-locked projectile release에서 `target_at_release=Some(target)`인 capture를 만든다.
2. target이 사라지거나 유효하지 않으면 `advance_homing_skill_projectile_runtime`이 terminal empty impact를 enqueue한다.
3. `apply_skill_projectile_impact`는 active target을 필터링해 `targets=[]`로 `execute_ordered_skill_effects`를 호출한다.
4. D04에서 추가한 `sim.rs`의 `first_capture` 조회가 `capture_for(effect_index, None)`을 요구한다.
5. 고정 target capture에는 unbound row가 없으므로 `InvalidStaticData("value-expression effect has no capture for its resolved set")`가 발생한다.

이 오류는 amount를 소비할 대상이 없는데도 metadata 조회를 대상 matching에 묶어서 생긴 회귀다.
상수식 Snapshot/Live도 영향을 받는다. 선행 동결본에서는 같은 상수식 2건이 통과했고 D04에서는
상수식·resolved aggregate × Snapshot/Live 4건이 모두 실패했다.

**빈 delivery consumption과 빈 set의 식 평가는 구별한다.** 적용 대상이 없는 damage/heal occurrence는
대상별 amount를 평가하지 않고 기존 완료 경로를 진행할 수 있다. 명시적으로 empty frame의 aggregate를
평가하는 경우에는 기존 Count/Sum=0, Max 오류 의미가 그대로 적용된다. release 시 필요한 Snapshot context가
없는 경우의 거부나, 실제 Target read의 context 오류까지 0 또는 성공으로 바꾸지 않는다.

## 4. 권장 구현 구조

### 4.1 amount 준비만 대상 존재 여부에 따라 수행한다

우선 `execute_ordered_skill_effects`의 expression amount 준비 블록을 최소 범위로 교정한다.

- 유효 carrier/effect 구조의 기존 검사는 유지한다.
- `targets`가 비어 있으면 대상별 평가 결과는 **빈 amount 배열**이다. 현재 expression 경로의 `Some(empty)` 의미를 유지하고, literal 경로를 뜻하는 `None`과 혼동하지 않는다.
- 이때 존재하지 않는 첫 target으로 capture를 조회하거나 Live member frame을 준비하지 않는다. 고정 Snapshot을 임의 unbound capture로 바꾸지도 않는다.
- target이 있으면 첫 실제 target에 대응하는 capture에서 plan을 얻어 Live member payload를 **effect당 한 번** 준비한다. 기존 target별 정확한 capture 조회와 평가를 이어간다.
- 같은 effect의 모든 대상 amount 평가가 mutation보다 앞서는 순서를 유지한다. member 준비를 target별 loop 안으로 옮겨 동일 set을 N번 복사하지 않는다.

구체적인 Rust 분기·helper 이름은 구현 재량이다. 별도 effect-plan accessor가 더 적절하다면 사용할 수 있으나,
metadata 접근과 실제 target capture 선택을 분리하고 새로운 retained payload나 중복 plan을 만들 이유가 있는지 설명한다.
이번 교정만을 위해 evaluator나 schema를 넓게 재구성할 필요는 없다.

### 4.2 effect 및 delivery의 완료 경로는 그대로 통과한다

빈 target set에서도 다음 경로는 보존한다.

- 기존 ordered effect command 생성·실행 및 effect signal 처리.
- effect projection 종료, local prefix 및 action 결과 witness의 기존 완료 처리.
- `SkillStepResult`의 target/damage/kill count 의미와 step delivery 결과 전달.
- `resolve_skill_step_delivery`, 기존 action/finite delivery close·settlement·보존량 정리.
- 성공/실패 양쪽에서 source/strike 등 임시 context 복원과 transaction 오류 전파.

따라서 함수 맨 앞의 `if targets.is_empty() { return Ok(default_result) }`나 effect loop의 무조건 `continue`로
교정하지 않는다. 실제 `Resources(SourceEmployee, ...)`는 빈 targets에서도 source owner를 한 번 처리한다.
반대로 `Resources(TargetEmployee, ...)`는 현재 empty target에서 InvalidOwner를 낼 수 있다.
두 의미를 유지해야 하므로 “빈 target이면 모든 effect 성공”이라는 규칙을 만들지 않는다.

### 4.3 identity·수명·오류 검사를 약화하지 않는다

- `capture_for`에 임의 첫 capture fallback을 추가하지 않는다. 실제 consumption target이 release의 고정 target과 다르면 기존 거부를 유지한다.
- plan/site/generation/action/delivery/attempt 검증을 제거하지 않는다. Snapshot을 Live로 다시 읽거나 사라진 target을 임시 unit으로 만들지 않는다.
- `InvalidStaticData` 등 오류를 caller에서 일괄 잡아 성공으로 바꾸지 않는다. 실제 오류는 기존 candidate rollback 경계까지 전파한다.
- amount를 읽지 않는 빈 consumption에서 불필요한 할당을 하지 않는다. nonempty 경로의 work/retained admission·공유 member payload·cap은 유지한다.
- projectile의 miss/timeout/target 소실 규칙이나 targeting 정책을 새로 정의하지 않는다. 기존 runtime이 빈 impact를 생성하는 분기들을 확인해 같은 교정이 적용되는지 검증한다.

## 5. 필수 회귀 시험과 인수 기준

검토 probe는 재현 출발점이다. 실제 제품 test module로 편입하고 외부 절대경로 `#[path]` hook에 의존하지 않게 한다.
`invoke_ability` → release step → queued event 처리 경로를 유지하며, 단순 helper 호출 성공만으로 대신하지 않는다.

| 항목 | 기대 결과 |
| --- | --- |
| target 소실 4조합 | Const Magnitude / ResolvedTargets Sum × Snapshot / Live 모두 terminal empty impact가 오류 없이 완료. 4개 조합이 실제로 실행됨을 확인 |
| 완료·수명 | 해당 delivery의 pending work와 projectile이 정리되고 step/action 완료 조건이 충족됨. no-target damage/heal의 실제 피해·회복·hit/kill count는 0. 관련 prefix/facts/retained가 기존 완료 의미를 보존 |
| inactive·miss/timeout | 실제 caller의 분기를 조사하고 대표 추가 경로를 직접 재현. 존재하지만 inactive인 target 또는 target-locked timeout의 빈 impact도 잘못된 capture 오류를 내지 않음 |
| 후속 effect 보존 | 빈 target의 expression 뒤에 유효한 SourceEmployee 자원 Grant를 둔 fixture에서 해당 효과가 정확히 한 번 적용. 함수/loop 조기 종료로 후속 effect가 생략되지 않음 |
| 실제 후속 오류 | 같은 경계 뒤의 실제 resource/typed fault는 정상 전파. transaction의 이전 commit은 보존되고 해당 candidate의 부분 변경은 복원. 빈 targets를 이유로 오류를 삼키지 않음 |
| 정상 hit | target-locked release Attack5 → impact9에서 Snapshot5/Live9 유지. actual target mismatch와 다른 action/attempt 거부 유지 |
| D04 및 선행 | WholeSet/ByProjectedTile의 canonical 결과·trace, 같은 effect의 선평가, 다음 effect/area tick의 Live 재준비, action prefix 7+3=10과 Snapshot7/Live10 유지 |
| empty 평가와 context 부재 | empty WholeSet Count/Sum=0·Max 오류, set 미제공 오류 및 미확정 release Snapshot 거부 유지. no-target consumption 분기로 이 evaluator 의미를 변경하지 않음 |

source 자원 fixture는 skill validation 전에 해당 resource catalog를 올바르게 구성한다.
후속 effect·완료 상태는 관측 가능한 실제 resource/fact/step/delivery 결과로 확인한다.
단순히 `process_event`가 Ok이거나 projectile map이 비었다는 assertion만으로 완료·정리 검증을 끝내지 않는다.
원래 실패 로그는 보존하고, 교정 전 실패와 교정 후 성공을 각각 해당 source identity에 결속한다.

## 6. baseline과 검증 실행

branch/HEAD·dirty/untracked·겹치는 변경을 확인한다. 기존 수정은 reset/clean/stash로 제거하지 않는다.
새 correction 구간에 actual before bytes·manifest·복원 archive를 보존하고, PLAN에 D04-F01과 수정 범위·실제 호출 경계를 연결한다.
필요한 변경은 ordered executor와 관련 product tests를 중심으로 한정하고 추가 변경이 필요하면 직접 의존 이유를 기록한다.

최종 revision에서 아래 기본 검증과 신규/기존 projectile·area selector를 실행한다.

```sh
cargo check -p game_core --all-targets --offline --locked
cargo test -p game_core --lib value_expr --offline --locked
cargo test -p game_core --all-targets --offline --locked
cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op
```

`value_expr` filter에는 일부 projectile 시험 이름이 포함되지 않으므로 신규 target 소실 4건과 완료·후속 effect 시험,
기존 `target_locked_projectile_uses_release_members_for_snapshot_and_impact_members_for_live`,
`resolved_area_tick_rebuilds_live_members_for_each_occurrence`를 실제 selector로 확인한다. 0건 실행은 증거가 아니다.
전체 시험의 기준은 고정된 1,500이라는 숫자가 아니라 최종 discovery의 모든 제품 시험이다.
Clippy allow는 추가하지 않고 변경 영역 format을 확인한다.

공유 executor 교정이므로 최종 source에서 기존 PC 36조건 및 R05 allocation guard를 확인한다.
PC 기준은 성공31/typed refusal5, R05 원 cap은 **9,030,188 bytes**다.
D04의 관찰 peak **8,997,016 bytes**는 비교값이며 새 cap이 아니다. allocation을 RSS와 혼동하거나
미적용 latency gate를 통과 보증으로 표시하지 않는다. 동일 최종 revision의 충분한 검증 후 근거 없이 전체 시험을 반복하지 않는다.

명령/cwd/env/toolchain/selector·stdout/stderr·exit·실제 시험 수·전후 source hash를 보존한다.
격리 실행이면 실제 built source와 hook/supplemental 입력까지 기록한다. 직접 실행·재사용·실패를 구별하고,
재사용한 결과를 교정 source의 직접 실행처럼 표시하지 않는다. sandbox 실패는 정해진 승인 절차로 해결한다.

## 7. 동결·재검토 인계와 종료

교정 구현과 위 관련 검증이 닫히면 별도 successor target에 다음을 남긴다.

- predecessor D04 manifest와 독립 검토, D04-F01의 원인·수정 경계·인수 시험 매핑.
- before/after manifest·SHA-256·실제 archive, 변경/추가/삭제 목록과 양방향 복원 가능한 delta.
- 교정 전 재현 실패와 교정 후 성공, 최종 source에 결속된 회귀·비용 기록 및 환경 제한.
- 읽기 전용 currentness·양방향 복원 검증, 검토자가 따라갈 `REVIEW_HANDOFF.ko.md`.
- 잔여 resolved ByOwner·scalar producer·D07·WP03/04·publication 차단 상태.

기존 D04 target과 독립 검토 파일은 덮어쓰지 않는다. 자체 검증 성공을 독립 PASS나 WP02/R06 완료로 선언하지 않는다.
**교정 구현·자체 검증 종료 / successor target 동결 / 독립 재검토 요청 대기**로 최종 응답하고 중단한다.
이번 실행에는 추가 D04 기능, D07, WP03/04, 서버·Unity, publication 해제 및 GEX-20/30을 포함하지 않는다.
