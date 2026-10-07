# R06 WP02 D04 실제 대상 집계 구현 지시서

작성일: 2026-09-26 KST.
대상: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`, `WP-CAVER-02`.
상위 인계: `HA-CAVER-P2G60-001`.

사용자는 계획·설계·독립 검토를 원 대화에서, 제품 구현을 별도 대화에서 진행한다.
이 문서는 기존 D04 설계를 다음 구현 구간으로 구체화한다. 새 Goal·Phase·정책 원본을 만들지 않는다.
작성 대화에서는 제품 코드를 변경하지 않았다.

## 1. 구현 담당자에게 전달할 요청

> 이 지시서에 따라 R06 WP02의 **D04 실제 resolved target 집계 연결**을 구현하고 자체 검증해줘.
> action 결과·release Snapshot 동결본과 독립 부분 PASS를 선행으로 삼아 기존 구현을 이어받아.
>
> 핵심은 executor가 이미 확정한 유한 target set을 effect-start/release frame에 연결하는 것이야.
> 기존 compiler/evaluator의 집계·단위·정렬·예산 기능을 재사용하고, 별도 집계 계산기를 만들지 마.
> 먼저 before baseline과 실제 호출·소유권·오류 전파를 기록한 실행 PLAN을 작성해.
>
> 단계검토를 유지해. 우선 WholeSet·ByProjectedTile의 실제 battle 연결을 첫 검토 단위로 잡고,
> 아래 중단 조건이 충족되면 D07이나 owner-domain 표현 확장으로 넘어가기 전에 target을 동결하고 종료해.
> 미완료 D04 조합은 구체적으로 남겨줘. 독립 검토와 WP02 전체 완료 판정은 원 대화에서 진행할 거야.

## 2. 선행과 문서 우선순위

1. [최신 독립 검토](../reviews/wp02_action_result_snapshot_001_review_001/REVIEW.md): **action 결과·release Snapshot 구간 부분 PASS**.
2. [구현 인계서](../segments/wp02_action_result_snapshot_001/REVIEW_HANDOFF.ko.md), [target manifest](../segments/wp02_action_result_snapshot_001/target/action_result_snapshot_001/MANIFEST.json), 해당 구간 `PLAN.md`·`NOTES.md`.
3. [기존 상세 설계](../../../CODE_REVIEW_003.ko.md)의 D02–D04·D06·D08–D10, 특히 V07/V08/V14/V19/V20.
4. [Phase Plan](../../../P2G_40_PHASE_PLAN_001.json), [Goal Set](../../../P2G_50_GOAL_SET_001.ko.md), [현행 GEX](../../../../../goal_execution_pipeline%281%29.md)의 재개·단계검토·증거 계약.

시작 target SHA-256:

```text
b16697bfe8c518944f0bddcf800b0f1e005ed639edd6fe915e4d96c726a6e1f4
```

333개 최종 source digest:

```text
60fe6643fa136a16773e93042d9db7d3e5b9398088c495771796abd410708717
```

작성 시 `freeze.py verify`로 currentness 및 양방향 복원 PASS를 확인했다. 구현 시작 시 다시 확인한다.
이전 [잔여 구현 지시서](WP02_REMAINING_IMPLEMENTATION_REQUEST_20260926.ko.md)의 원칙은 유지하되,
시작 target·현재 상태·다음 구간은 이 문서를 따른다. 과거의 “CurrentActionPrefix 미지원”과
“Linux Rust 도구 부재”는 최신 상태가 아니다. 최신 검토에서는 Rust 1.98.1로 focused 30개와 추가 probe 3개를 직접 실행했다.

## 3. 현재 코드에서 이어받을 경계

| 위치 | 현재 책임과 이번 연결 방향 |
| --- | --- |
| `game/value_expr/mod.rs` | `FiniteSource`, `Grouping`, `EvalMember`, `EvalFrame`, typed member references와 limits가 존재한다. 필요한 내부 frame 준비 API를 확장할 수 있다 |
| `game/value_expr/eval.rs` | canonical member 정렬·중복 검사, owner/tile grouping, inner/outer reduction, empty·overflow·unit·budget 검사가 있다. 이 evaluator를 계산의 단일 구현으로 유지한다 |
| `game/value_expr/compile.rs`, `catalog.rs` | aggregate requirements와 delivery capability를 확인한다. 실제 caller가 제공하는 context와 capability 표가 일치하도록 점검한다 |
| `battle/core/value_expr_runtime.rs` | `build_value_expr_frame_with_prefix`는 현재 `requirements().resolved_targets`를 일괄 거부한다. 실제 set을 전달받아 member frame을 채우는 것이 핵심 변경이다 |
| `battle/core/sim.rs` | `execute_ordered_skill_effects`가 `targets`를 보유하고 동일 effect의 모든 amount를 mutation 전에 평가한다. 이 순서를 유지하며 set을 adapter에 전달한다 |
| `battle/core/skill_runtime/projectile.rs`, `area.rs` | release 및 impact/tick에서 실제로 확정되는 대상의 범위를 구분해 capture/consume에 전달한다 |
| `battle/core/mod.rs`, `types.rs`, `resource_runtime.rs` | 실제 UnitInstanceId·active unit·projected tile·기존 owner resolver의 근거다. 전장 재탐색이나 임의 owner 생성에 사용하지 않는다 |

helper 이름·borrow 구조·내부 모듈 배치는 구현 재량이다. PLAN에는 실제 호출 지점, set을 확정하는 owner,
frame을 만드는 시점, 비용 admission 위치, 오류가 도달하는 transaction을 명시한다.

## 4. 구현 방향

### 4.1 실행 context가 실제 set을 전달한다

- executor가 확정한 `&[UnitInstanceId]` 또는 동등한 bounded context를 adapter로 전달한다. 식 평가가 주변 unit을 검색하거나 targeting을 다시 실행하지 않는다.
- **set 미제공**과 **확정된 빈 set**을 구별한다. context 없음은 typed fault이고, 확정된 빈 set은 Count/Sum의 정상 입력이다. `EvalFrame::default()`의 빈 vector만으로 둘을 판별하지 않는다.
- raw cardinality 검사와 준비 비용 admission 후 identity를 canonicalize한다. 같은 UnitInstanceId는 집계에서 한 번만 세고, 서로 다른 identity의 같은 값은 합치지 않는다.
- canonical 집계용 순서는 실제 damage/command delivery 순서를 바꾸지 않는다. 원 targets를 제자리 정렬하거나, 집계 dedup을 근거로 기존 gameplay target 처리를 바꾸지 않는다.
- `Target`은 현재 amount를 적용할 대상, `AggregateMember` subject는 집계에서 순회하는 구성원이다. 두 context를 덮어쓰지 않는다. Source·현재 Target·action prefix와 member별 값이 함께 필요한 식을 지원하는 구조로 연결한다.

### 4.2 member 값과 grouping key를 명시적으로 읽는다

- `ResolvedTargets` member identity는 실제 UnitInstanceId에서 결정적으로 만든다. debug 문자열·포인터·배열 순번을 unit identity로 쓰지 않는다.
- member의 Attack 및 HP Current/Max/Missing 등 **현재 producer가 있는 명시적 reference**만 read-only로 읽어 `EvalMember.values` 또는 동등한 bounded 구조에 넣는다. 필요한 ref만 준비한다.
- `Read(AggregateMember { unit })`는 unit만으로 Attack/HP/resource 중 무엇을 읽을지 정하지 않는다. authored member의 명시적 scalar와 실제 unit member를 구별한다. 실제 target용 scalar producer가 선언되지 않은 조합은 typed refusal을 유지하며 임의 0·Attack 기본값으로 통과시키지 않는다.
- Count fixture는 유효한 명시적 member expression을 사용한다. Count가 결과 값만 사용하지 않는다는 이유로 member context 검증을 우회하지 않는다. WP03의 SpellPower/cadence producer는 이번 구간에서 구현하지 않는다.
- `ByProjectedTile`은 해당 moment의 기존 `UnitBody::projected_tile`/Position으로 key를 만든다. 별도 반올림·grid·빈 tile·연결망을 만들지 않는다.
- `AuthoredMembers`는 기존 compiler의 입력 ordinal·정규화 의미를 보존한다. resolved unit identity dedup 규칙을 authored 동일 값 제거 규칙으로 확대하지 않는다.

### 4.3 Snapshot과 Live를 같은 frame 계약에 연결한다

| 경계 | 구현 기준 |
| --- | --- |
| Live | 해당 consumption의 실제 target set과 effect 시작 시 값·tile key를 읽는다. 동일 effect의 모든 대상 amount 계산이 끝난 뒤 기존 mutation을 실행한다 |
| 다음 effect / 다음 area tick | 새 effect-start frame에서 필요한 Live 값을 다시 읽는다. 앞 effect의 frame을 무조건 재사용하지 않는다 |
| release에 set이 확정된 Snapshot | release에서 member identity·필요 값·group key를 함께 고정하고 immutable capture로 소유한다. impact에서 Live 값이나 새 tile key로 재구성하지 않는다 |
| directional projectile / 미확정 area Snapshot | release에 실제 target set이 없으면 기존 D03의 UnsupportedMoment를 유지한다. 빈 set이나 나중 impact set으로 대신하지 않는다 |
| delayed Live | impact/tick에서 기존 executor가 확정한 해당 occurrence의 set을 쓴다. 서로 다른 delivery나 과거 tick의 set을 섞지 않는다 |

release/consume capability는 실제 producer를 근거로 확인한다. 특히 현재 catalog의 Instant 판정은
direct-target 여부에 묶여 있으므로, 다중 대상 Instant의 소비 직전에 실제 set이 확정되는 경로를 조사한다.
이번에 연결한 Live 경로는 해당 capability를 정확히 반영하고, 미확정 Snapshot까지 함께 허용하지 않는다.

동일 effect 안에서는 공통 member payload를 공유할 수 있지만, target별 scalar context가 다른 frame이나
다른 effect/시각의 Live frame을 하나로 합치지 않는다. Snapshot의 공유 payload는 Arc 등으로 유지해
대상 수 N에 대해 member N개를 대상마다 deep clone하는 구조를 피한다. 새 cross-frame cache는 만들지 않는다.

### 4.4 ByOwner의 첫 검토 경계

기존 설계는 선언된 Side/Employee/EquipmentCopy domain을 요구한다. 현재 `Grouping::ByOwner` 자체에는
domain 인자가 없고 `EvalMember.owner`는 하나의 optional key다. 기존 authored member는 owner를 직접 갖지만,
실제 resolved unit set에서 어느 domain을 선택할지는 별도 근거가 필요하다.

구현 전 기존 producer에 명시적 domain 선택이 있는지 확인한다. 근거가 없으면 side/employee를 기본값으로
추론하거나 source equipment를 모든 target의 owner로 복제하지 않는다. **이번 첫 checkpoint는 WholeSet과
ByProjectedTile 연결을 닫는 데 두고, resolved ByOwner의 명시적 domain 표현은 잔여 D04로 기록한다.**
기존 authored ByOwner의 동작과 mixed-domain 거부는 유지한다. 이 상태를 D04 전체 완료라고 표시하지 않는다.
domain 표현 확장은 실제 코드 조사 결과와 필요한 schema/plan 변경 범위를 원 대화에 인계한다.

### 4.5 비용·수명·오류 처리

- raw member 확인, identity 복사, sort, member별 ref 수집, group key, vector/map/Arc 구성 전에 checked 크기 계산과 해당 work/retained admission을 수행한다. evaluator 내부 meter가 나중에 검사한다는 이유로 adapter의 선행 할당을 무계상하지 않는다.
- canonical member 수·raw scan work·group 수를 구별한다. 중복 입력도 읽고 정렬하는 비용은 든다. nested aggregate의 member product·동시 buffer 상한을 checked 계산하고 loop 중에도 취소/work를 확인한다.
- 기존 ExprLimits 및 native/authoring limits 중 적용되는 더 작은 한도를 지킨다. 이미 다른 owner가 청구한 같은 allocation을 다시 청구하지 않도록 각 비용의 소유자를 기록한다.
- 새 member buffer/capture의 `CloneShape`, retained extent, preflight scan과 release/close/settlement 정리를 연결한다. 공유 payload를 반복 deep clone하지 않는다.
- typed 오류의 site/node/phase와 실제 원인을 보존한다. 모든 amount 평가 실패는 기존 candidate transaction까지 전달한다. partial command 적용, 빈 값 fallback, swallow-and-continue를 만들지 않는다.
- HP/resource/reservation/queue/RNG/UUID/log/action witness/time은 기존 rollback 경계를 유지한다. 이전 commit까지 되돌리거나 retry에서 결과를 중복 누적하지 않는다.

## 5. 필수 시험과 기대값

kernel의 `set_resolved_targets` 수동 시험만으로 실제 battle 연결을 입증하지 않는다. adapter 단위 시험과
실제 ordered execution/delivery caller 시험을 함께 둔다. 아래 값은 frame oracle이며, damage/heal 적용 시험에서는
기존 unit 검사와 final narrowing에 맞는 식을 작성한다.

| 항목 | 기대값 또는 실패 조건 |
| --- | --- |
| 실제 member read | A Attack=5, B=3, C=2의 resolved set에서 Sum=10, Max=5, Count=3. source/current Target의 별도 값과 혼동하지 않음 |
| 실제 tile grouping | A/B는 tile0, C는 tile1: inner Max→outer Sum=7, inner Count→outer Max=2. 실제 projected tile reader를 통과 |
| 중복·순열 | `[C,A,B,A]`와 `[B,C,A]`는 집계 amount·trace가 동일. plan fingerprint도 입력 순열 때문에 변하지 않음. gameplay delivery 순서는 기존대로 유지 |
| 빈 set / 미제공 | 확정된 empty WholeSet Count/Sum=0, Max=EmptyAggregate. context 미제공은 별도 typed fault이며 0으로 성공하지 않음 |
| 같은 effect / 다음 effect | 두 target의 HP가 10/10일 때 Live HP Sum의 1/4을 damage amount로 쓰면 둘 다 effect-start 합계 20에서 amount 5를 계산. mutation 후 다음 effect의 Live HP Sum은 10 |
| Snapshot / Live | release member Attack=5가 consume 전에 9가 되면 Snapshot=5, Live=9. Snapshot tile key도 release 값 유지. 미확정 target-set Snapshot은 정확한 거부 |
| 지연 caller | projectile 실제 impact 및 area 실제 tick에서 해당 consumption set을 사용. 반복 tick에 member 변화가 있으면 Live가 새 set을 읽음 |
| member·outer context | `AggregateMember` HP/Attack, current `Target`, Source, result prefix를 구분. nested aggregate의 내부 member가 바깥 문맥에 누출되지 않음 |
| 잔여 조합 | resolved ByOwner domain 미제공, scalar producer 미정, WP03 ref는 구체적인 typed refusal. 기존 authored grouping/혼합 owner-domain 오류는 보존 |
| rollback·재시도 | 앞 effect 성공 뒤 집계에서 overflow/empty Max/예산 초과 등 실제 오류를 주입. candidate 전체 복원, 이전 commit 보존, 정상 retry에서 잔여 context·중복 prefix 없음 |
| 비용·취소 | raw/canonical members, groups, nested work, retained의 해당 limit 경계와 취소. 부족한 예산에서 큰 복사/정렬 선행 없음. 반복 delivery 종료 후 retained 안정 |
| 선행 회귀 | cross-step/delivery 7+3=10, Snapshot 7/Live 10, actual damage·valid zero, delivery당 target count 1회, action settlement/clone 경계 유지 |

기존 `disconnected_dynamic_contexts_and_cancellation_are_typed_without_mutation`의 aggregate 일괄 거부 행은
새 지원 경로의 positive 시험과 실제 context 미제공/미지원 조합의 negative 시험으로 분리한다.
“아무 오류나 나면 성공”하는 oracle을 새 기능 지원 증거로 사용하지 않는다.

## 6. 실행·검증·증거

시작 시 branch/HEAD·dirty/untracked·겹치는 변경을 확인하고 선행 manifest와 actual bytes를 대조한다.
기존 수정은 reset/clean/stash로 제거하지 않는다. mutation 전 새 segment에 before archive와 source/data/test/config/lock
identity를 보존한다. 선행 동결본과 검토 기록을 덮어쓰지 않는다. 재사용 검증은 정확한 source/입력/환경 적합성을 기록한다.

기본 명령은 repository root 기준이며 실제 toolchain·selector·fixture 경로를 먼저 확인한다.

```sh
cargo check -p game_core --all-targets --offline --locked
cargo test -p game_core --lib value_expr --offline --locked
cargo test -p game_core --all-targets --offline --locked
cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op
```

새 caller 시험이 `value_expr` filter 밖에 있으면 해당 selector도 직접 실행하고 0건 실행을 성공 증거로 삼지 않는다.
변경 영역 format, 관련 battle/resource/literal 회귀를 확인한다. 최종 전체 시험 수는 실제 discovery 결과를 기록한다.
Clippy allow·기존 assertion·lockfile·지원 cap을 검증 편의를 위해 완화하지 않는다.

공유 frame/capture/clone 변경이므로 기존 PC 36조건과 R05 guard를 같은 조건으로 비교한다.
선행 PC는 성공31/typed refusal5이며, 이는 새로운 D04 workload를 대신하지 않는다.
새 실제 집계 workload로 비용·결과를 별도 측정한다. R05 원 cap은 **9,030,188 bytes**,
선행 boxed revision의 관찰 peak는 **9,019,233 bytes**, 여유는 **10,955 bytes**다.
성능 비교에서 cap 변경·RSS 혼동·미적용 latency gate를 새 보증으로 표현하는 일을 피한다.

각 실행의 command/cwd/env/toolchain/selector, stdout/stderr, exit, 실제 시험 수와 source 전후 hash를 보존한다.
격리 실행은 실제로 빌드한 격리 source와 test hook·supplemental 입력도 결속한다. workspace hash만으로 대신하지 않는다.
직접 실행·재사용·미실행·실패를 구분하며 샌드박스 오류는 정해진 승인 경로로 처리한다.
필요한 검사가 통과하면 새 변경이나 실패 근거 없이 전체 시험을 반복하지 않는다.

## 7. 중단·인계 조건

WholeSet·ByProjectedTile의 실제 target frame 연결과 직접 의존하는 capture/consume·예산·rollback 경계가 닫히고,
빌드·focused·관련 전체 회귀 및 비용 guard 결과를 확인한 첫 의미 있는 지점에서 중단한다.
단순 compile 성공만으로 멈추지 않으며 D07을 구현하기 위해 중단을 미루지도 않는다.
caller 결합 때문에 일부 지원 조합을 안전하게 분리해야 한다면 실제 경계와 미완료 항목을 PLAN·인계에 명시한다.

새 구간에 다음을 남긴다.

- predecessor target/검토, 구현 범위, 실제 owner/call/fault 경로, 중단 지점 선정 이유.
- before/after manifest·SHA-256·실제 archive, 추가/변경/삭제 목록 및 양방향 복원 가능한 delta.
- 최종 revision에 결속된 직접/재사용 검증과 실패 기록, currentness·복원 검증 결과.
- 지원 matrix: delivery×moment×finite source×grouping 및 실제 시험 근거. 미지원 owner-domain/scalar 조합은 별도 기재.
- 잔여 D04, D07 preview/explain, WP03/04와 전체 통합·감사·publication 차단 상태.

WP03/04 본 구현, 서버·Unity 변경, R07 이후, production expression publication 해제, 독립 재검토 및 GEX-20/30은
이 실행에 포함하지 않는다. 최종 응답으로 **부분 구현·검증 종료 / 단계검토 타깃 동결 / 검토 요청 대기**를 반환한다.
원 대화에서 독립 검토를 받은 후 다음 구현 범위를 정한다.
