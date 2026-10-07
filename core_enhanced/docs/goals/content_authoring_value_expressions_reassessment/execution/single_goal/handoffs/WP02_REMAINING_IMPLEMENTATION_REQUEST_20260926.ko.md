# R06 WP02 잔여 구현 지시서

작성일: 2026-09-26 KST. 대상: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`.
상위 인계: `HA-CAVER-P2G60-001`. 적용 범위: WP02 잔여 구현·자체 검증과 다음 단계검토 인계.

사용자는 **계획·검토는 원 대화에서, 구현은 다른 대화에서** 진행하기로 했다.
이 문서는 기존 R06 계획과 최신 교정 검토를 연결한 후속 구현 지시서다.
기존 정책·Phase Plan·Goal을 대체하거나 새로운 P2G 계획을 만들지 않는다.
작성 대화에서는 제품 코드를 변경하지 않았다. 사용자가 이 문서를 전달하며 구현을 요청하면 아래 범위로 실행한다.

## 1. 구현 담당자에게 전달할 요청

> 이 문서를 읽고 R06의 **WP02 잔여 구현**을 진행해줘. 기존 상세 설계를 따르고,
> 최신 교정본 및 부분 PASS를 시작 기준으로 삼아 이미 구현된 코드를 이어받아.
>
> 먼저 current tree와 선행 target을 대조하고, 변경 전 실제 파일을 보존한 baseline과
> 새 실행 구간의 PLAN을 작성해. 기존 계획의 의미는 유지하고 실제 함수 연결·데이터 소유권을 확인해.
> 상세 설계에 이미 정해진 내용을 새 정책 질문으로 다시 제기하지 마.
>
> 기존에 활성화한 **단계검토 과정을 유지해줘.** 아래 잔여 범위를 모두 한 번에 끝내도록 강제하지 말고,
> 의미 있는 하나의 책임이 구현되고 빌드·관련 회귀 검증이 통과한 첫 지점에서
> 다음 큰 연결 작업 전에 before/after·증거·남은 범위를 고정하고 최종 응답으로 중단해.
>
> 이번 대화의 역할은 구현·자체 검증이야. 독립 재검토나 GEX-20/30을 자동 시작하지 말고,
> 결과를 원 계획·검토 대화로 가져갈 수 있게 인계해. WP03/04 본 구현, R07 이후, 서버·Unity 변경,
> production expression publication 해제는 이번 실행 범위에 포함하지 마.

## 2. 읽을 원본과 최신 시작점

다음 순서로 읽고, 과거 기록의 “현재”보다 exact target과 후속 검토 관계를 우선한다.

1. [EEP-F01~F03 교정 재검토 002](../reviews/wp02_eep_f01_f03_correction_001_review_002/REVIEW.md): **교정 구간 부분 PASS**, 추가 수정 요구 없음.
2. [교정 인계](../corrections/wp02_eep_f01_f03_001/REVIEW_HANDOFF.ko.md)와
   [교정 target manifest](../corrections/wp02_eep_f01_f03_001/target/correction_001/MANIFEST.json).
3. [상세 설계 CODE_REVIEW_003](../../../CODE_REVIEW_003.ko.md): D02–D04, D06–D10과 해당 V01–V21 행.
4. [정식 Phase Plan](../../../P2G_40_PHASE_PLAN_001.json)의 `WP-CAVER-02`,
   [Goal Set](../../../P2G_50_GOAL_SET_001.ko.md)의 요구·불변식·Command Matrix.
5. [현행 GEX](../../../../../goal_execution_pipeline%281%29.md)의 재개·기준선·검증 재사용·단계검토 절.
   기존 적용 revision은 `2026-09-13-gex-r7`이다. 실제 실행 시 변경 여부를 확인한다.

시작 target SHA-256:

```text
abe5a96830d8090f77e63fde7c6d7ff79675f701a382437d2492e0c66e9aa60a
```

작성 시 target 해시와 현재 source 333개 해시의 일치를 확인했다. 이 관찰을 미래 실행 시점의 검사로 대신하지 않는다.
기존 [계획 인계](../../../PLAN_HANDOFF.ko.md)는 상위 계획 입구이며, 옛
[구현 인계서](IMPLEMENTATION_RESUME_20260926.ko.md)와 상위 실행 PLAN의 frame/capture 또는 최초 커널 상태는 역사다.
이번 구현 재개의 상태·선행 locator는 이 지시서와 위 최신 검토를 따른다.

## 3. 이미 구현된 부분과 유지할 교정

WP01의 typed AST·bounded parser/compiler/evaluator·catalog,
WP02의 scalar Snapshot/Live frame·capture·resource read adapter를 재사용한다.
`execute_ordered_skill_effects`의 **동일 effect 대상 전체 평가 → 기존 command 실행 → 실제 결과 확정 → 다음 effect**
경로와 immediate/projectile/area의 오류 전파·candidate rollback을 이어받는다.
한 delivery consumption 내부의 `EarlierEffect` 읽기는 이미 연결되어 있다.

최신 교정에서 확보한 조건:

- 대상 임시 복사 제거, resource ID·carrier·prefix·projection·평가 결과 배열의 할당 전 비용 청구.
- resolved target 수는 delivery consumption당 한 번. 효과별 피해·사망 및 다른 delivery의 합산 의미는 유지.
- action 전체 결과를 보증하지 못하는 `CurrentActionPrefix`는 typed fault로 거부.
- source/action/delivery/effect frontier 결속과 유효 0 / 부재 구분, 기존 예산 상한 및 누적 미환급 유지.

이전 검토 PASS가 새 변경의 회귀 검증을 면제하지 않는다. 새 코드가 직접 영향을 주는 부분을 재검증한다.

## 4. 이번에 이어갈 WP02 범위

아래는 기존 계획의 미완료 책임이다. **우선 연결할 후보는 action-wide 결과와 release 결과 Snapshot**이며,
나머지 범위의 실제 순서는 코드 의존성을 확인해 새 실행 PLAN에 기록한다.
이 후보 목록은 고정된 checkpoint 수나 새로운 Phase 분할이 아니다.

| 책임 | 기존 설계와 구현 기준 |
| --- | --- |
| action 전체 완료 결과 | D02/D06. 실제 ActionHandle·ActionSite·effect·delivery occurrence·완료 frontier를 결속해 현재 effect 시작 전에 끝난 동일 action의 결과만 읽는다. active `query_action` 또는 마지막 action 결과를 완료 witness로 대신하지 않는다. 다른 step/delivery의 완료 결과를 누락하거나 같은 occurrence를 중복 합산하지 않는다 |
| release 결과 Snapshot | D03/D06. 지연 delivery 생성 전에 완료된 prefix를 고정해 소유한다. 이후 Live 결과로 대체하지 않는다. 아직 완료되지 않았거나 다른 attempt의 결과는 거부하며 release/consume 시각을 구분한다 |
| 실제 대상 집계 | D04. executor가 확정한 유한 target set을 frame으로 전달한다. identity 중복 제거·명시적 owner/tile grouping과 결정적인 reduction 순서를 적용한다. 새 전장 탐색·targeting 정책이나 연결망을 만들지 않는다. aggregate 정렬 때문에 실제 damage delivery 순서를 바꾸지 않는다 |
| 수치 preview/explain | D07. 동일 evaluator와 명시적 frame을 사용한다. context 없는 explain은 `needs_runtime_context`, 미확정 usefulness는 미확정 상태를 유지한다. prospective preview와 committed Snapshot, 식의 amount와 mitigation/RNG 이후 피해를 구분한다 |

결과 witness는 승인된 actual HP loss/healing/resource/count projection만 bounded 보존한다.
full fact history를 늘려 쌓거나 action·result handle을 저장 데이터에 넣지 않는다.
기존 R04/R05 종료·취소·retention과 연동하고, 실패 시 witness와 facts도 같은 candidate에서 복원한다.
새 저장소·carrier는 clone shape와 native retained/preflight 비용에 포함한다.

변경 전 실제 연결 위치를 확인할 최소 범위는 `src/game/battle/core/`의 `value_expr_runtime.rs`
(`EffectResultPrefix`, `ValueExprDeliveryCapture`, capture/evaluation/frame 함수), `sim.rs`
(`execute_ordered_skill_effects`), `runtime_facts.rs`, `resource_runtime.rs`, 해당 `skill_runtime` callers다.
집계는 `game/value_expr`의 기존 frame/compiler/evaluator를, 관찰 소비자는 `authoring/query.rs`,
`authoring/preview.rs`, `authoring/compiled.rs`, `battle/core/target_usefulness.rs`를 대조한다.

새 실행 PLAN에는 **어느 owner가 완료 결과를 갱신·조회·해제하는지, 어느 호출이 release capture와
effect-start read를 수행하는지, 오류가 어느 transaction까지 전달되는지**를 실제 코드에 연결한다.
이는 D10 준비 점검이다. helper 이름·borrow 구조·모듈 내부 배치는 구현 재량이지만
정책·수명·결과 의미를 바꾸거나 새 gameplay 규칙이 필요한 경우에는 해당 쟁점과 코드 근거를 원 계획 대화로 반환한다.

WP03의 새 stat/copy producer가 필요한 소비자는 그 의존을 명시한다.
이번 범위를 넘어 WP03을 구현하거나 임의의 stat 값으로 성공을 만들지 않는다.
기존 미지원 조합은 필요한 실제 경로가 닫힐 때까지 명시적으로 거부한다.
public capability는 원 계획의 atomic 통합 조건이 충족되어야 하므로 이번에 차단을 해제하지 않는다.

## 5. 직접 시험할 기대값

이번에 실제로 구현한 책임과 영향받는 회귀를 아래에 매핑한다. 아직 손대지 않은 책임의 시험을
PASS로 기재하거나 모든 WP02 시험 완료를 첫 checkpoint의 필수 조건으로 삼지 않는다.

| 경계 | 기대값·오류·복원 |
| --- | --- |
| cross-step / cross-delivery 결과 | 동일 action에서 실제 피해 7 후 3이면 그 이후 `CurrentActionPrefix(DamageHpLost)`는 10. 같은 delivery 내부 값 3으로 축소하지 않음. 다른 action·attempt 결과는 섞이지 않음 |
| 실제 결과와 0 | 요청 피해 7이 방어 후 실제 4이면 참조값 4. 완료된 0은 읽을 수 있고, missing/unsettled/self/future/evicted는 정확한 typed fault |
| 결과 Snapshot / Live | 완료 결과 7을 release에서 고정한 뒤 동일 action의 완료 결과가 10이 되어도 Snapshot은 7, 해당 완료 frontier를 읽는 Live는 10. release 이전 선행 결과가 없으면 거부 |
| 대상 집계 | tile0:A5/B3, tile1:C2에서 flat Sum=10/Max=5, 그룹별 Max 후 Sum=7, 그룹별 Count 후 Max=2. 같은 target identity 중복은 집계에서 한 번, 순열은 동일 결과. empty Count/Sum=0, Max는 오류. 서로 다른 owner domain 혼합은 거부 |
| 같은 effect와 다음 effect | 동일 effect의 모든 대상 값은 어떤 대상 mutation보다 먼저 계산. 다음 effect의 Live는 앞 effect 적용 후 상태를 읽음 |
| 실패·재시도 | 앞 effect 성공 후 뒤 effect 오류 시 HP/resource/reservation/queue/RNG/UUID/log/prefix/state/time은 해당 candidate 시작 상태. 이전에 commit된 진행은 보존. 이후 정상 재시도에서 결과 중복 누적·잔여 context 없음 |
| 지연 delivery와 수명 | immediate/projectile/area의 실제 변경 경로에서 release/consume 결속, 취소·사망·철수, Live owner 부재 및 Snapshot 존속의 기존 규칙 유지 |
| 관찰 소비자 | 동일 고정 frame의 runtime/preview/explain amount·trace 동등. context 부재를 0 damage 또는 false usefulness로 숨기지 않음 |
| 비용 | work·retained·members 등 변경한 비용 축의 정확한 경계와 부족한 예산, 취소를 확인. 할당/복사/정렬 전에 admission. 반복 delivery 종료 후 보존량과 clone 비용 확인 |

**기존 미지원 시험을 새 지원의 성공 증거로 쓰지 않는다.** action-wide 구현 후에는 현재의
`action_prefix_refusal_rolls_back_only_the_current_candidate` 등 거부 전용 시험을 새 계약에 맞게 전환한다.
정상 prefix=10의 성공 시험과 실제 오류를 유발하는 candidate rollback 시험을 각각 유지한다.
아직 미지원인 조합의 typed refusal도 남긴다. “어떤 오류든 발생하면 통과”하는 옛 cross-step probe는
새 지원 완료의 oracle로 충분하지 않다. 기존 frozen 시험·실패 기록은 보존하고 새 테스트 변경 이유를 기록한다.

## 6. 구현 전 환경과 기준선

1. branch/HEAD, scoped/global dirty·untracked, 추가·삭제 파일과 겹치는 작업을 확인한다.
   최신 target과 현재 파일 차이를 분류하고 기존 변경을 reset/clean/stash로 없애지 않는다.
2. [교정 verifier](../corrections/wp02_eep_f01_f03_001/freeze.py)의 `verify`를 읽고 실행한다.
   drift가 있으면 원인과 영향, 별도 successor 검토 필요 여부를 기록하며 과거 manifest를 고치지 않는다.
3. mutation 직전 actual bytes, source/data/test/config/lock 및 필요한 입력의 hash·원본 archive를
   새 구간 경로에 보존한다. 기존 dirty 원본은 Git HEAD/diff만으로 대체할 수 없다.
4. 실제 toolchain과 test selector, fixture·trace 출력 경로를 확인한다. 관련 baseline의 재사용 적합성을
   판단하고 부족하거나 달라진 부분을 직접 검사한다. 격리 실행은 별도 `CARGO_TARGET_DIR`와
   쓰기 가능한 임시 fixture/trace 위치를 사용한다.

최신 검토 환경에는 Linux cargo/rustc가 없어 Rust 시험을 직접 재실행하지 못했다.
검토자는 동일 고정본의 기존 제품 1,487건·probe 7건 증거를 재사용했다.
이 한계를 구현 환경의 검증 완료로 가져오지 않는다. 이전 독립 실행의 64건 ReadOnlyFilesystem와
trace 파일 부재 1건, 별도의 Unix socket 권한 실패 이력도 보존되어 있다.
샌드박스 실패는 정해진 승인 절차로 해결하며 시험 skip·cfg 변경·assertion 완화로 통과시키지 않는다.
실행 도구를 확보할 수 없으면 그 상태를 보고하고 검증 완료 checkpoint를 선언하지 않는다.

## 7. 구현 후 검증과 기록

실제 selector를 확인한 뒤 해당 focused tests와 영향받는 기존 회귀를 실행한다.
runtime/source/result/clone 변경은 관련 battle/resource/literal 회귀를 포함한다.
공유 타입·transaction·consumer 영향이 넓으면 all-target을 포함한 검증 범위를 넓힌다.
원 Goal의 최종 전체 Command Matrix는 계속 유효하지만 WP02 부분 검토를 전체 완료로 확대하지 않는다.

기본 빌드 확인은 repository root에서 다음 명령이다.

```sh
cargo check -p game_core --all-targets --offline --locked
```

필요한 전체 제품 회귀 명령은 `cargo test -p game_core --all-targets --offline --locked`다.
기존 Clippy 조건은 `cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op`이며
allow를 추가하지 않는다. 변경 영역 format을 확인하고 기존 범위 밖 포맷 차이는 구분한다.
의존성/cache 부족을 숨기기 위해 lockfile을 임의 변경하지 않는다.

새 witness/carrier/clone 비용에 영향이 있으므로 관련 구간에서 기존 PC fixture 및 R05 allocation guard를
동일 조건으로 비교한다. R05 원 cap 9,030,188 bytes와 과거 실패는 유지한다.
36조건의 기존 PC fixture 결과는 31완료/5거부였으며, 새 기능은 필요한 workload를 별도로 보완한다.
기존 성공 fixture가 새 prefix/집계 조합을 검증했다고 주장하지 않는다. 성능 수치는 RSS·공식 PC 상한과 구분한다.

각 실행의 command/cwd/env/toolchain/selector, fixture·source identity, stdout/stderr, exit 및 실제 시험 수를 기록한다.
원 작업공간에서 실행하면 그 source, 격리 실행이면 **실제로 빌드한 격리 source**의 시작/종료 hash를 기록한다.
test-only hook과 supplemental 입력도 보존한다. 원 작업공간 hash만으로 격리 실행의 identity를 증명하지 않는다.
직접 실행·재사용·미실행·실패를 구분하고, 충분한 검사 통과 후 근거 없이 같은 전체 검사를 반복하지 않는다.

## 8. 중단과 원 대화에 가져올 결과

의미 있는 책임이 닫히고 실제 빌드·관련 시험을 통과했으며 남은 범위를 설명할 수 있는 첫 지점에서 중단한다.
단순 컴파일 성공만으로 사소한 수정마다 멈추거나, 다음 기능이 남았다는 이유로 중단을 미루지 않는다.
안전하게 나눌 수 없는 직접 의존은 최소 범위로 함께 연결하고 그 이유를 PLAN에 기록한다.

새 구간의 실행 문서와 after target에 다음을 포함한다.

- exact Goal·선행 교정 target/검토·이번 구현 범위와 실제 중단 지점 선정 이유.
- before/after manifest·SHA-256, 실제 archive, 추가/변경/삭제 목록과 복원 가능한 delta.
- 직접 실행 및 재사용 검증, 성공·실패 로그, 실제 입력/소스 결속과 환경 제한.
- 기존 상세 설계의 어느 항목이 닫혔는지, 미지원 기능·남은 WP02 및 WP03/04 의존.
- 검토 입구 `REVIEW_HANDOFF`와 읽기 전용 currentness/복원 verifier.

최종 응답은 **`부분 구현·검증 종료 / 단계검토 타깃 동결 / 검토 요청 대기`**로 마친다.
빌드나 필수 검증을 끝내지 못했다면 그 사실과 재개 위치를 보고하고 위 완료 문구를 사용하지 않는다.
이후 검토는 원 대화에서 요청한다. 구현자가 자신의 결과에 독립 PASS 또는 WP02/R06 전체 완료를 발행하지 않는다.
