# R08~R12 준비 판단과 한계

상태: 조사·계획 초안. 독립 감사나 정식 P2G 정책 동결이 아니다. 현재 위치는 [PLAN](PLAN.md).

## 실제 source에서 확인한 내용

- `BossTaskData`에는 RouteCombat/AcquireTarget/Move/Wait/Summon/Signal 등이 있고 명시 skill task는 없다. `request_skill_action`은 transaction 안에서 manual admission을 호출한다. `resolve_manual_skill_base_request`는 Player/Manual/active/동일 장착 skill/공명 만충을 검사한다. boss 전용 권한 adapter를 만들되 기존 검사를 전역 제거하지 않는 방향을 채택한다.
- manual 접수는 ActionHandle accept, focus 종료 시각, action locks, PendingSkillCast/start_seq와 ManualCastEnd를 만든다. 따라서 입력 접수/실행 gate/release를 구분해야 한다. 현재 일반 비용 정책 및 auto 경로 전체 비교 전 start hook의 exact 위치를 확정하지 않는다.
- `finalize_prepared_death`는 deployment 종료·movement·wave death epoch·fact·buff·graveyard 처리 후 death_trigger_commands를 실행한다. optional dispatch 억제를 신규 typed cause로 전달할 필요가 있고, source=None만으로 억제했다고 판정할 수 없다.
- `EmployeeCondition`은 physical_damage/trauma이고 Breakdown mental state는 trauma100에서 파생한다. 현행 룰북은 별도 run/checkpoint 카운터·node완료·회복·일반 사망 회수 및 의도값2를 요구한다. 새로운 영구 사망 정책을 발명할 필요는 없다. `apply_incapacitation`의 부상 대가를 침식에 재사용하지 않는다.
- `simulation_delta_for_tick`은 Pause에서 None, 아니면 배속/잔여 delta를 적용한다. 강제 연출은 이 battle delta와 분리한 host elapsed가 필요하다. `ApplicationRoute`에 elapsed/tick/bucket/disconnect/admin/restore 등이 있어 Behavior guard 하나로 우회를 닫았다고 할 수 없다.
- `SkillDef`의 `Vec<SkillStepDef>`는 delay/target/when/repeat/delivery/effects를 가진다. Flow 확장은 compiled 단일 경로가 되어야 하며 새 AST enum만으로 표현력 완료를 주장하지 않는다.
- `ActionLocks`는 until의 max 합성이다. source별 lease 제거가 가능한 ledger가 필요하다. 기존 one-shot ModifyStats의 의미는 보존한다.
- `apply_damage_result_and_record`는 final_damage로 lethal 준비 후 HP/fact/actual loss 관측/공명을 처리한다. shield 적용은 이 이후 숫자 패치가 아니라 확정 receipt 이전 중재여야 한다.

위 항목은 선택 본문 직접 열람 범위다. 모든 호출자·저장·예산 경계 검토는 PLAN의 후속 조사로 남겼다.

## 정책 준비표

기존 승인 방향과 작성자 추천을 구분한다. 아래 선택은 **위임에 의한 계획 후보**로 기록하며 정식 P2G-10/20에서 exact 계약/영향/시험을 완성한다.
숫자0·무한·자동 성공으로 빈 정책을 메우지 않는다. 현재 사용자에게 추가 정책 결정을 요구할 상황은 발견하지 못했다.

| 정책 | 준비 판단 / 근거·대안 | 영향·oracle / 복구 방식 |
| --- | --- | --- |
| P01/P02 | 승인된 Pause 접수≠침식 시작, 일반 자격과 공명 만충 유지. 첫 unpaused 공통 gate 후보; release 시점 단순 치환은 기각. 일반 cast 비용 시점 대조는 미완료 | A09-01~03/10. exact hook은 P2G에서 확정, 감사 후 필요하면 후보 수정 |
| P03 | 공통 trauma 임계·양수 유한 연출은 데이터. 80/1000ms는 fixture 예시이며 공식 밸런스로 채택하지 않음 | A09-01/06; 범위/overflow 검증. 데이터 변경으로 되돌림 |
| P04 | 룰북의 node완료/trauma회복/재진입/저장 카운터 보존. 회복 먼저·새 진입은 당해 node 소모 제외 추천은 node flow 대조 후 확정 | A09-08/09,X05/06. live limit2 의도와 미작성 사실 구별 |
| P05 | 강제정지 해제 호출의 초과elapsed 폐기 후보. catch-up은 의도하지 않은 즉시 전투 진행을 유발. 사용자 Pause/disconnect는 독립 | A09-06/07. fake clock 경계 시험과 host 계약 문서로 검증 |
| P06 | state 소유 미방출 action만 취소, 방출 payload 보존. timeout과 action 취소는 schema로 구별 | A08-02~05. task candidate 실패는 함께 rollback |
| P07 | stable authored 순서, Parallel All join, 유한 continuation 후보. 대량 선할당·시간+1 회피·무대상 기회 적립 기각 | A10-02~08. compiler/runtime 동반 변경, unsupported 진단 |
| P08 | 승인된 태아 기본 공격만 강화/겹침 비중첩/타 source 보존. 저지는 기존 유지 후보: 원문은 open design. 강도 tie-break는 source order와 함께 P2G에 명시할 것 | A11-01~04. 일반 Attack 증폭 대안 기각(파편/proc 오증폭), source별 종료 |
| P09 | 승인된 Hello 자연 만료 복귀/재입력 Goodbye1회/추가 공명 없음. [start,end)와 commit 잔여시간 후보 | A11-05~08. 같은 시각 입력·실패 transaction을 oracle로 결정 |
| P10 | 개인→공유→authored priority→생성ordinal 후보. HashMap/UUID 임의 순회 기각 | A12-01~03. source별 중재 순서만 변경 가능, receipt와 같이 검증 |
| P11 | ordinary HP1, nonlethal 기본 HP cost/명시 lethal opt-in 후보. shield/parry로 비용 대납 금지, 침식 special kill 별도 | A12-04/05/09. cost fact와 실패 무변경 검증 |
| P12 | strike에 결속된 피해/부착효과만 패링 억제, counter1회, 소실 공격자 no-target 후보. unrelated self effect 일괄 제거 기각 | A12-06/07. 반사/자해/cost 재귀 금지. 흑조 상시 효과는 아래 별도 계약 대조 |
| P13/P14 | 전투 pool/channel/overlay의 attempt 수명, R07 copy/form과 Breakdown의 durable owner 분리. V8/명시 V7 원본 보존 인수 | X05/06/08. 새 schema 필요 시 version/explicit 변환 결정, silent default 금지 |

흑조 상시 효과와 룰북 active1/passive 아님 계약은 구체적 예외 범위 대조가 남아 있다. 상세 지시서는 흑조 개별 예외를 P2G에 명시하도록 요청한다.
전체 파편의 passive화를 허용하지 않으며, 기존 확정 계약을 바꾸는 예외로 드러나면 근거 없이 확정하지 않는다.
태아의 기본 수치+20%/+15는 원문에 후속 합의가 있지만, 이번 범위 밖 유물 증폭·전체 콘텐츠 이식을 포함하지 않는다.

## 채택한 기술·진행 선택

- 별도 최종 감사 부재를 자체 회귀로 대체하지 않는다. 이 요청의 준비 경계에서 제품 테스트/감사를 자동 시작하지 않는다.
- 무관한 대규모 untracked cache까지 열거/크기 합산하던 조사를 중단하고 R07 runner의 실제 source membership을 재현했다. 복원 가능한 before는 그 source와 조사 입력·수정 예정 roadmap 원문을 포함한다. 저장소 전체 backup이라는 주장은 하지 않는다.
- 기존 frozen runner를 import하지 않고 순수 source 목록 규칙을 새 script에 복사했다. import cache/로그가 이전 target에 생기지 않게 한다.
- Single Goal 후보로 공유 final regression을 한 번 소유한다. 각 WP는 기능이 유효해진 경계를 남기며 빈 API scaffold만 인계하지 않는다.
- 최종 검증 비용 계획은 기본 병렬/focused와 최종 broad/PC36/R05, 20분 시험 제외. allocation/work/rollback budget은 기존 typed fault 유지, 새 자동 취소 정책 없음.

## 미래 전환 영향

현재 node 완료·SavePoint/Gate·retreat/checkpoint·R07 copy 소유를 기준으로 조사했다. 차기 원정의 지도/깊이/억제력/귀환/대여로 대체하지 않는다.
미래 원정 전환 시 Breakdown node 카운터, host/result 게시, attempt rollback 및 사본 소유·반출 의미를 재검토해야 한다.
이 메모는 차기 정책을 확정하지 않는다.
