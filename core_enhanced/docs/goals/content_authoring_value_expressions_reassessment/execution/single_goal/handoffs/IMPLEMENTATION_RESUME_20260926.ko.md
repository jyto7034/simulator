# R06 후속 구현 담당자 인수인계서

작성일: 2026-09-26 KST  
대상: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`  
계획 인계: `HA-CAVER-P2G60-001`  
상태: **인수인계 작성 완료 / WP02 frame·capture 단계검토 선행 필요**

후속 상태 안내(2026-09-26): 위 상태는 이 문서 최초 작성 시점이다. 이후 WP02 감사는 `CORRECTION_REQUIRED`를 판정했고 PC 부하 baseline도 추가됐다. 현재 교정 작업의 입구는 [WP02 교정 설계·구현 인계 계획](WP02_CORRECTION_PLAN_20260926.ko.md)이다. 아래 후속 기능 구현은 교정·재검토 및 사용자의 다음 구현 요청 전에는 시작하지 않는다.

## 1. 전달할 작업 지시

> 기존 R06 ValueExpr 구현을 이어받아 남은 구현·자체 검증을 진행해줘. 처음부터 다시 구현하거나 P2G를 재작성하지 말고, 아래 최신 동결 타깃과 계획·교정·검토 이력을 인수해.
>
> 이 Goal에 이미 명시적으로 활성화한 **단계검토 옵션을 유지해줘.** 중단할 세부 지점·개수는 미리 고정하지 말고, 구현 중 의미 있는 책임이 닫히고 빌드·해당 회귀 검증이 통과하는 시점에 선택해. 다음 큰 연결 작업 전에 before/after를 고정하고 반드시 최종 응답으로 중단해.
>
> 단, 현재 최신 WP02 frame/capture 타깃은 검토 대기다. 먼저 exact target에 대한 단계검토 결과와 현재 변경분의 처리 근거를 확인해. 아직 없으면 읽기 전용 인수·비교까지만 수행하고 검토 요청 대기로 중단해. 구현 담당자의 자체 검증으로 선행 검토 PASS를 대신하거나, 이 인수인계서만으로 동결을 해제하지 마.
>
> 선행 단계검토 및 필요한 교정·재검토가 끝나고 사용자가 다음 구현을 요청하면, WP02의 순차 effect 실행·완료 prefix 결과 참조·오류 전파 및 rollback 연결을 우선 진행해. 이어지는 WP03/04는 아래 전체 방향을 따르되 각 단계검토 중단 경계를 넘지 마.
>
> 구현 전 현재 actual bytes 기준 baseline을 새로 보존하고, 종료 시 변경 source·실행 기록·시험 입력/출력·복원 자료가 결속된 새 target을 만들어줘. 과거 manifest·archive·실패 기록·감사 verdict는 덮어쓰지 마.
>
> R07 이후, wrapper/runtime 추출, game_server·Unity 변경, Windows 외부 팩 이식은 이번 범위가 아니야. GEX-20/30이나 독립 검토를 자동으로 시작하지 말고, WP/Phase/Goal 완료도 자체 선언하지 마.

이 문서는 새 담당자에게 전달할 지시서다. 작성 자체로 구현·검토를 실행한 것은 아니며, 실제 모델 선택·전환은 사용자가 한다.

## 2. 가장 먼저 읽을 문서와 권위

경로는 이 문서 기준 상대 경로다. 과거 기록의 “현재”라는 표현보다 exact identity와 실제 후속 인계를 우선 확인한다.

1. 현행 [Goal 실행 절차](../../../../../goal_execution_pipeline%281%29.md)의 인수·currentness·단계검토·종료 규칙. 이 Goal의 기록상 적용 revision은 `2026-09-13-gex-r7`; 실제 인수 시 현행 revision과 차이를 확인한다.
2. [최신 WP02 인계](../segments/wp02_runtime_frame_capture_001/REVIEW_HANDOFF.md), 같은 디렉터리의 [PLAN](../segments/wp02_runtime_frame_capture_001/PLAN.md), [기술 메모](../segments/wp02_runtime_frame_capture_001/EXPERIMENT_NOTES.md), [실행 기록](../segments/wp02_runtime_frame_capture_001/EXPERIMENTS.md).
3. [WP01 F01/F02 교정 재검토](../reviews/wp01_f01_f02_correction_001_review_001/REVIEW.md): 해당 부분 범위 PASS이며 WP02 또는 R06 전체 PASS가 아니다.
4. [계획 인계](../../../PLAN_HANDOFF.ko.md), [실제 구현 설계 D01–D10·검증 V01–V21](../../../CODE_REVIEW_003.ko.md), [Goal 및 27 acceptance/검증 행렬](../../../P2G_50_GOAL_SET_001.ko.md).
5. 필요한 exact 승인 기준: [Freeze](../../../P2G_30_FREEZE_001.json), [Phase Plan](../../../P2G_40_PHASE_PLAN_001.json), [계획 Handoff](../../../P2G_60_HANDOFF_001.json). 상세 정책 의미가 필요하면 해당 문서의 exact 참조를 따라간다.

기존 [상위 실행 PLAN](../PLAN.md)은 최초 커널 checkpoint 중심으로 남아 있다. 이를 최신 완료 현황으로 오인하지 않는다. 동결에 포함된 옛 PLAN을 편의상 갱신하지 말고 새 실행 구간의 PLAN에서 predecessor와 현재 범위를 연결한다.

본 인수인계는 위 정책·계획을 대체하지 않는다. 구현 helper 이름·borrow 구조·모듈 분할은 담당자가 정할 수 있지만 계산 의미, 처리 순서, 저장 호환, 검증 기대값과 승인 scope는 임의 변경할 수 없다.

## 3. 확인된 현재 위치

| 범위 | 확인된 상태 | 해석 시 주의 |
| --- | --- | --- |
| WP01 커널·입력·컴파일 | typed AST, bounded compiler/evaluator, 실제 RON 입력·catalog와 교정 이력 존재 | 부분 검토 결과를 전체 Goal 완료로 확대하지 않음 |
| WP01 F01/F02 | strict unknown-field 및 authored-local aggregate 교정에 부분 PASS 기록 존재 | 이후 WP02 변경은 별도 검토 대상 |
| WP02 첫 구간 | 실제 BattleCore scalar frame/capture, Snapshot/Live, exact identity, read-only resource 참조 구현·자체 검증 후 동결 | 단계검토 대기; production effect consumer는 아직 미연결 |
| WP02 나머지 | 순차 effects, 완료 prefix witness, resolved-target 집계, 실패 전파/rollback, quantitative consumer 등 잔여 | 아래 6절 및 D02–D08 확인 |
| WP03 | stat·copy enhancement·cadence·save/fingerprint 통합 잔여 | 기존 기능이 전혀 없다는 뜻이 아니라 R06 전환이 남아 있음 |
| WP04 | 실제 외부 입력부터 공개 실행까지 검증, 문서·비용·전체 회귀 잔여 | 최종 감사는 구현 요청과 별개 |

현재 authored expression publication의 `content_capability_unsupported` 차단은 의도된 안전 경계다. 타입·파서·adapter가 존재한다는 이유로 해제하지 않는다.

### Exact predecessor와 최신 타깃

- WP01 승인된 부분 검토 대상: [correction manifest](../corrections/wp01_target_001_f01_f02/target/correction_001/MANIFEST.json)
  - SHA-256: `679336ad025f9dfaa6b44345ca3e494fd694e5903d49786a91d0ff270dbd642f`
- WP02 구현 전: [baseline](../segments/wp02_runtime_frame_capture_001/baseline/pre_implementation_001/MANIFEST.json)
  - SHA-256: `26b5222aaa514b6622093429bd9994c3bab04f40791e0446bb2c96cd447fe6be`
- WP02 최신 구현 후: [target](../segments/wp02_runtime_frame_capture_001/target/target_001/MANIFEST.json)
  - Record: `R06_WP02_RUNTIME_FRAME_CAPTURE_TARGET_001`
  - SHA-256: `d3dbf3ba7c3c3be5a584db3092e7059c116d92b1ae812740f1fb68ccb05c2fc3`
  - source 314개, execution records 6개, evidence 37개, validation receipts 17개를 포함한다.

2026-09-26 인수인계 작성 조사에서 이 WP02 target의 단계검토 PASS 보고서는 찾지 못했다. 최신 인계와 manifest도 review pending을 표시한다. 뒤에 새 보고서가 생겼다면 exact target·변경 범위·판정·사용자의 다음 구현 요청을 직접 확인하고 이 관찰을 갱신해 사용한다.

## 4. 동결 이후 변경과 진입 gate

이번 문서 작성 중 manifest의 현재 source 목록을 SHA-256으로 비교했다. **314개 중 312개 일치, 2개 불일치**다. execution records 6개와 evidence 37개는 모두 일치했다. 이는 목록에 있는 파일의 비교이며 신규 파일 포함 여부·전이적 영향·archive 전체 복원·시험 재실행까지 검증한 것은 아니다.

| 현재 변경 파일 | archive와 직접 비교한 차이 | 현재 SHA-256 |
| --- | --- | --- |
| `core_enhanced/src/game/world/tests/combat.rs` | `resource_lifetimes` 모듈에 설명과 `#[cfg(unix)]` 추가 | `a2498f99b0a3e9423db0cc2655c018af8c2b24005da41ac1baa8b93a6a398eca` |
| `core_enhanced/src/game/world/tests/resource_lifetimes.rs` | 줄바꿈·들여쓰기 변경 | `7485126713f4d97db01f24da18520c5965f3462ffe9bdad5ba60bc8d1960715a` |

첫 변경은 Unix에만 존재하는 테스트 helper를 비 Unix에서 import하던 편집기 오류에 대한 수정이다. Windows 외부 팩 지원 구현이 아니며, 해당 플랫폼에서의 테스트 포함 범위는 달라진다. 두 변경을 이유 확인 없이 삭제하거나 무조건 non-impacting으로 선언하지 않는다.

구현 전 순서:

1. 현재 HEAD/branch, scoped dirty/untracked, 추가·수정·삭제 파일과 겹치는 작업자를 확인한다. HEAD만으로 현재 tree를 복원할 수 있다고 가정하지 않는다.
2. 최신 manifest 및 보존 archive/delta identity를 검증한다. 현재 파일과 frozen target이 다르면 원인을 별도 기록한다. 기존 verifier의 mismatch를 숨기거나 manifest를 수정해서 PASS시키지 않는다.
3. 선행 검토 담당자가 frozen WP02 구현과 위 delta를 구분해 검토할 수 있도록 연결한다. 필요하면 기존 target을 보존한 채 current successor review target을 별도로 만든다. exact 검토 범위와 현재 tree에 적용되는 근거가 있어야 한다.
4. 검토 결과가 없으면 구현을 시작하지 않고 `선행 WP02 단계검토 요청 대기`로 반환한다. 수정 요구면 해당 finding의 교정 요청 → 별도 correction baseline/target → 재검토를 거친다.
5. 통과 후에도 자동 재개하지 않는다. 사용자의 다음 구현 요청을 확인한 뒤 새 구간의 PLAN과 **mutation 직전 implementation baseline**을 만든다.

이 문서 작성 단계에서는 코드 감사·기능 시험을 수행하지 않았고 WP02 PASS 또는 currentness 전체 PASS를 발행하지 않았다.

## 5. 작업 범위와 기존 안전성 교정의 보존

포함: 승인된 R06의 남은 Core 구현, 관련 테스트·fixture·canonical 문서, 새 실행/검증/동결 기록.

제외: R07–R24 연속 구현, 일반 Flow/VM, 공식 weight/model 재정의, wrapper/Game Runtime 추출, 서버·Unity 배포, Windows 로더 이식, 모바일 지원. wrapper/server 임시 논의는 별도 작업이다. 외부 소비자의 DTO 영향은 읽기 전용으로 조사하고 외부 수정이 필수면 별도 승인을 요청한다.

기존 교정에서 확보한 조건을 회귀시키지 않는다:

- 복제·정렬·원소 할당보다 앞선 입력/work/메모리 admission.
- compiler/evaluator 비용 계산의 일치와 checked arithmetic; static 비용과 dynamic 비용의 구분.
- 깊은 입력의 반복형 분석, 중첩 집계의 실제 동시 stack 사용량, 더 작은 runtime meter 적용.
- resource/target 시점 capability의 누적 검사.
- 실제 RON bounded parsing, strict unknown-field 거부, 실패 진단을 위한 전체 RON 재parse 금지.
- authored-local aggregate만 적법하게 정적 평가하고 동적 참조를 빈 frame/0으로 평가하지 않음.
- unit·반올림·overflow·fingerprint·legacy narrowing 의미 보존. 시험 통과를 위한 예산 확대·assertion 완화 금지.

## 6. 검토 통과 후 우선할 WP02 구현 방향

이는 다음 작업의 책임·수용 조건이지, 이 전체를 한 번에 구현해야 한다는 고정 checkpoint 목록이 아니다. 실제 코드와 의존성을 확인해 첫 의미 있는 빌드·검증 가능 경계를 선택한다.

### 순차 effect 실행과 frame

- 실제 호출 경로를 조사한다: `core_enhanced/src/game/battle/core/`의 `sim.rs`, `value_expr_runtime.rs`, `runtime_facts.rs`, `resource_runtime.rs` 및 instant/projectile/area callers.
- 현행 전체 effect command 일괄 생성 지점에서 모든 Live 식을 먼저 평가하지 않는다.
- authored effect 순서대로 **effect-start frame 고정 → 해당 effect의 모든 target 값 계산·범위 검사 → 기존 damage/heal authority로 실행 → actual facts 기반 완료 witness 확정 → 다음 effect** 순서를 구현한다.
- 같은 effect의 첫 target mutation이 다음 target의 Live 입력을 바꾸지 않게 한다. 기존 target/delivery 순서와 ModifyDamage fold 의미는 보존한다.
- release Snapshot은 다시 읽지 않고, Live는 consumption/effect-start state를 사용한다. exact source-copy/action/delivery/generation과 시각을 유지한다.
- `ResolvedTargets`는 실제 확정된 유한 target 집합에 연결하고 owner/tile grouping을 명시적으로 처리한다. 미확정 Snapshot target을 임의 생성하지 않는다.

### 완료 prefix 결과와 resource

- 단순 active `query_action`이나 마지막 settled action을 현재 effect의 선행 결과로 사용하지 않는다.
- exact action capability, effect index, delivery occurrence, fact frontier에 묶인 **완료된 선행 effect witness**만 참조한다. same/future effect, 다른 attempt, 미완료·evicted 결과는 typed fault다.
- 표현식의 원래 damage amount가 아닌 실제 HP loss 등 승인된 fact field를 읽는다. 7 피해가 방어 후 실제 4라면 후속 heal 참조도 4다.
- 존재하는 완료 witness의 0과 witness 부재를 구분한다. resource read는 실제 lease의 read-only `bind/value` 경로를 유지한다.
- witness/carrier를 bounded 보존하고 종료·취소·rollback·retention 경계에 연결한다. full fact history 또는 source-copy 간 공유 mutable 평가 cache를 만들지 않는다.

### 오류 전파와 전체 candidate rollback

- `with_instant_resource_delivery` 등 void callback과 조기 return/오류 무시 경로를 확인하고 typed `Result`를 최상위 transaction까지 전달한다.
- instant/projectile/area 및 native/controlled 실행 경로를 모두 다룬다. context 복원·delivery close 오류도 성공으로 숨기지 않는다.
- effect 0 성공 후 effect 1 실패 시 HP뿐 아니라 resources/reservations, queue, RNG, UUID, 로그, prefix, state/time을 candidate 시작 상태로 되돌린다. 이전에 commit된 timestamp prefix는 보존한다.
- capture/witness의 retained·clone 비용을 기존 admission/clone-shape/native scan에 포함한다. immutable plan 공유를 유지한다.

### 남은 quantitative consumers

- runtime/preview/explain은 같은 evaluator와 명시적 frame을 사용한다.
- context 없는 static explain은 `needs_runtime_context`, 사전 usefulness의 미확정 값은 적절한 미확정 상태로 남긴다. 0/false로 바꾸지 않는다.
- prospective preview와 실제 committed Snapshot, expression amount와 RNG/mitigation 이후 damage를 구분한다.
- WP03 producer가 없는 stat에는 임의 값을 넣지 않는다. 공개 capability는 아래 atomic 통합 조건이 충족되기 전까지 차단한다.

## 7. 후속 WP03/04의 전체 방향

WP03은 D05/D07을 따른다. Attack/SpellPower 단일 합성, weapon 수치의 한 번만 적용, actual owned-copy 기반 enhancement, 기존 AttackSpeedPoints/cadence 순서, V7 zero omission과 nonzero 저장·복원, content/Certified input fingerprint를 함께 연결한다. definition-wide 강화 합산·저장 checksum 우회·미정 weight 발명은 금지다.

WP04는 D09 V01–V21과 Goal의 27 acceptance를 실제 구현·시험에 연결한다. 외부 RON → compiler → production consumer → preview/explain/CLI 경로, literal/V7 호환, 비용·취소·회귀와 canonical 문서를 닫는다. private builder·단위 시험만으로 공개 지원 완료를 주장하지 않는다.

WP01/02/03은 단일 atomic 통합 계약이다. 부분 checkpoint는 독립 Phase 완료가 아니다. literal과 expression의 실제 소비자, 필요한 stat/save/projection 및 공개 진단이 연결되고 검증되기 전에는 publication gate를 제거하지 않는다.

## 8. 검증 및 before/after 증거

### 구현 전

- 새 구간 경로 아래 PLAN에 승인 요청, 단계검토 유지, predecessor 검토 결과, 포함/제외 범위, drift 처리와 종료 조건을 적는다.
- 수정할 source뿐 아니라 필요한 schema/data/test/config/lock/실행 기록의 실제 bytes와 SHA-256을 보존한다. Git diff는 clean HEAD 외 dirty/untracked 원본의 대체물이 아니다.
- 기존 old-build V7/literal fixture를 재사용하되 원본 hash를 확인한다. 새 코드로 다시 생성해서 old-build 증거를 대체하지 않는다.
- 실제 test selector를 발견하고 비어 있지 않은 baseline을 실행한다. “계획된 test 이름”과 “실제로 존재하는 test”를 구분한다.

### 구현 중·후

- 변경 seam의 focused tests와 영향받은 기존 회귀를 실행한다. WP02라면 Snapshot/Live, target/grouping, result frontier, resource 무변경, 실패→정상 retry와 native/controlled rollback을 포함한다.
- `cargo check -p game_core --all-targets --offline --locked` 및 변경 파일 rustfmt를 확인한다. lock/cache가 부족하면 원인을 보고하고 의존성을 몰래 바꾸지 않는다.
- Clippy의 기존 기록은 command-line allow `clippy::never_loop`, `clippy::eq_op` 조건이었다. 실제 baseline·명령·새 warning을 구분하고 allow를 확대해 결함을 숨기지 않는다.
- 공유 schema/transaction/save 변경이면 검증 범위를 넓힌다. Goal 전체 구현 종료 전에는 Goal Command Matrix의 all-target/feature/release/public probes 및 V01–V21 전체 증거가 필요하다. focused green만으로 대체하지 않는다.
- raw command/cwd/flags, source·fixture·toolchain identity, stdout/stderr, exit, 실제 건수/ignored/실패를 보존한다. 0 tests, sandbox 실패, 직접 재실행하지 않은 과거 성공을 현재 PASS로 합산하지 않는다.
- before/after actual-byte archive, added/modified/deleted 목록, binary를 포함해 복원 가능한 delta, manifest 및 detached hash를 새 이름으로 고정한다. 필요한 입력을 빼거나 이전 target을 수정해 currentness를 맞추지 않는다.
- 종료 시 read-only verifier로 source·records·evidence와 필요한 파일 목록을 확인하고, source 변경이 검증 이후 생겼다면 영향을 받는 검증을 다시 수행한다. currentness 통과와 코드 검토 PASS는 별개다.

## 9. 언제 멈추고 무엇을 넘길 것인가

단계검토 opt-in은 같은 Goal의 재개·교정에 유지한다. 실제 지점은 구현 중 다음 조건을 처음 만족하는 의미 있는 경계에서 정한다.

1. 한 책임·처리 경로가 검토 가능하게 닫혔다.
2. 빌드와 해당 기능·영향 회귀 검증이 통과했다.
3. 구현된 범위와 미연결 범위를 구분할 수 있다.
4. before/after·실행 증거·현재 identity를 동결할 수 있다.

그 시점에 다음 큰 producer/consumer 연결을 시작하지 않고 mutation을 멈춘다. 종료 보고에는 다음을 포함한다.

- 이번 구현 책임과 잔여 범위; WP 전체 완료 여부를 과장하지 않음.
- 변경 파일 요약, 직접 실행/재사용/미실행 검증의 구분.
- before/after manifest 경로·SHA-256, archive/delta, 검토 입구.
- 실패 기록·제약·검토가 필요한 위험과 다음 구현 후보.
- 종료 문구: **`부분 구현·검증 종료 / 단계검토 타깃 동결 / 검토 요청 대기`**.

교정이면 별도 correction target을 만들고 `교정·자체 검증 종료 / 재검토 요청 대기`로 끝낸다. 전체 GEX-10 구현 조건을 충족한 경우에도 Audit Target 고정 후 `구현·자체 검증 종료 / 감사 요청 대기`에서 멈춘다. 검토·교정·다음 구현은 각각 사용자의 요청을 받으며, 자동 감사·자동 다음 구간 진입은 하지 않는다.
