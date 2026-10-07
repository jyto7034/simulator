# WP11 상세 설계 — source가 소유하는 지속 효과

현재: 기능 구현·최종 자체 검증 자료 동결, 독립 감사 요청 대기. 통합55조건 중54 자체 검증/X09 환경 제한1. 최신 판정은 COVERAGE.json / REVIEW_HANDOFF.md가 소유한다. 아래 단계별 메모는 당시 이력이며 독립 감사 판정이 아니다.

상태: 구현 준비. WP10의 RON/flow/amount/runtime 연결과 회귀1552개(환경 socket1개 제외)를 바탕으로 조사했다. WP10 최종 통합 수용 및 R11 구현 완료를 뜻하지 않는다.

## 실제 접속점

- `SkillEffectDef`, raw skill validation, `value_expr/catalog.rs`와 `authoring::effect`의 exhaustive consumer를 함께 변경한다. 지속 효과는 기존 `ModifyStats`를 소급 재정의하지 않는 별도 producer다.
- `BattleCore::process_commands_with_modifier_source`의 `ApplyModifier`는 현재 `UnitStats`를 영구 변경한다. 임시 ledger가 활성인 대상은 영구 기준값을 먼저 변경하고 활성 record를 다시 합성해야 한다.
- `ActionLocks`는 Copy의 until 합성이다. source별 활성 제한은 ledger가 소유하고 기존 until을 보존한 파생 capability mask로 연결한다. 종료 시 역수/무조건 초기화하지 않는다.
- 실제 기본 공격은 `handle_basic_attack_start_inner`에서 windup snapshot, `handle_basic_attack_resolve_inner`에서 release한다. aura의 regular basic damage와 속도는 별도 property로 두며 skill damage의 Attack을 증폭하지 않는다.
- `BattleCore::process_event`, live-command candidate 완료, death/withdraw/interrupt/hard-CC/terminal이 ledger 동기화·종료 경계다. membership의 변경은 다음 실제 소비 및 공개 조회 전에 반영한다.
- `reserve_finite_source`/delivery ticket으로 지속 source의 R04/R05 자원·출처를 유지하고 exactly-once 종료 때 닫는다. 진단 ordinal 자체를 capability로 사용하지 않는다. compiled flow와 이미 방출된 payload 출처는 보존한다.

## 작성 계약과 정책

별도 sustained effect 정의에 가역 modifier, channel aura, 기본 공격 overlay를 둔다. source/activation/대상/property/operation/expiry의 record를 단일 battle-local owner가 보관한다. 속성별 flat 합산 후 percent 합성으로 기준값에서 재계산한다. 지원 속성과 시간/참조/조합은 publication에서 검증하며 암묵적 fallback은 두지 않는다.

태아 계열은 P08대로 자기 자신·Guest·소환물·방어 object를 제외한 실제 직원 수혜자를 range에서 선택한다. aura 동일 family는 priority 내림차순/creation ordinal 오름차순으로 한 source만 적용한다. 일반 피격과 Silence는 이미 시작한 channel을 종료하지 않는다. 공격·새 스킬·태세·전술 이동/교대·작업은 source lease로 제한하고, 명시 중지는 허용한다. 기존 저지는 유지한다.

Hello는 P09의 [start,end)와 별도 재입력 경로를 사용한다. 정확한 현재 activation을 소비하고 추가 공명을 청구하지 않으며, Goodbye용 남은 시간은 재입력 commit 시 확정해 기존 ValueExpr RemainingTime 경계에 전달한다. 자연 expiry는 복귀만 한다. 재입력 실패는 효과 제거와 후속 실행 전체를 enclosing transaction으로 되돌린다. 다음 기본 공격1회 overlay는 유효 release에만 소비한다. 원래 공격 주기를 reset하거나 두 발을 만들지 않는다. windup/expiry/release의 적용 경계를 구현 시 실제 source에 재대조해 기록한다.

## 검증 및 계량

A11-01~09의 공개 RON 정상 명령부터 전투·결과·저장까지 검사한다. A/B 중첩·종료·입출입 반복, aura exact eligibility와 overlap, capability별 명령 거부, 각 종료 원인과 다른 source 보존, Hello natural/reinput/same-time/stale/rollback, 기본 공격 출처·흡혈·다른 무기·windup·release를 독립 oracle로 둔다.

새 ledger/member/lease/expiry event는 allocation 이전 admission, native retained/clone shape, timestamp reuse, 관측 DTO/compiled 설명/fingerprint를 함께 연결한다. 영속 사본 개화에는 쓰지 않으며 attempt 종료 시 제거한다. R12 pool·방어/제어는 다음 WP에서 같은 수명 owner에 추가하되 이 문서만으로 구현 완료를 표시하지 않는다.

## 구현 중간 업데이트 (r11_tests_016)

R11 집중15개 통과. 이 결과는 전체 A11/통합 수용 완료가 아니다. `Sustained::BasicAttackOverlay`는 acquisition range/footprint와 delivery만 교체하고, 원래 무기 damage type·archetype·affinity·block·interval·windup·trigger binding은 유지한다. 같은 대상의 overlay는 생성 순서상 최신 record 하나가 유효하며 종료 시 남은 record/원본에서 다시 합성한다. next_attack_only는 유효 release에서만 소비한다. 원래 준비 중인 공격의 delivery/range는 release 시점 유효 profile로 선택하고, 이미 release한 projectile은 당시 immutable profile을 유지한다. 기본 공격 자동 스케줄은 overlay 생성/종료에서 reset하지 않는다.

공간 basic adapter는 기존 타일 footprint projection과 이동 원 충돌 kernel을 사용한다. directional은 유한 range/lifetime, 고정 launch aim, stable impact 순서와 중복 hit set, max hits/kills/pierces를 갖는다. BasicAttack damage policy와 weapon triggers를 유지하며 가짜 skill source로 변환하지 않는다. target-locked 기본 projectile의 기존 publication 계약은 유지한다. persistent basic area/targetless basic launch는 명시 publication 오류다.

재입력은 `PlayerBehavior::ReinputSustainedSkill` 및 동일 BattleLiveCommand의 caster+skill+activation을 현재 owner ledger에서 검증한다. 정상 ActivateSkill은 활성 Hello를 새 Hello로 갱신할 수 없다. 만료 구간은 [start,end)이다. 재입력 child Goodbye는 별도 zero-focus skill의 zero-delay immediate leaf 하나로 publication을 제한하여, Hello 제거→새 skill 출처 생성→amount 평가/실제 피해까지 한 command candidate에서 실행한다. 실제 DivideByZero와 세 fault site에서 원상 복원이 검증됐다. 해당 제약 밖의 authored continuation은 거부한다.

`CommittedReinputContext`는 검증한 private source에 activation/commit time/remaining milliseconds를 저장한다. Goodbye의 R06 RemainingTime은 Snapshot/Live 모두 이 고정 duration을 읽는다. 자연 만료에는 context 생성/Goodbye가 없다. 실제 fragment provider는 새 Goodbye cast로 전달하고 amount query/observation은 두 출처를 함께 노출한다. command는 공명 검사/추가 reset 없이 실행한다. 실제 World fixture는 exact fragment 장착→Hello→typed reinput→Goodbye→결과→저장·fresh Core 복원을 거쳤다.

남은 R11 검증: aura 다중 source overlap/직원 외 명시 제외/새 배치·이동, 기본 무기 trigger의 실제 출처/흡혈 비교, next-one·windup·다른 무기/다른 source 조합, retreat 및 추가 native fault matrix. 전체 broad 재검증과 최종 CLI/55조건 currentness는 최종 통합에서 수행한다.

## 추가 수용 실행 (r11_tests_023)

집중23개 통과(`60901b6435c16b4217082bb944873ff0bc131116d950677d97f37671434664f6`). aura 같은 family의 priority/생성 동률, 다른 family 합산, 한 source 제거 후 남은 source 보존을 3직원으로 검증했다. 정상 Deploy/Withdraw/재배치, Guest의 employee condition 부재, 실제 scenario spawn의 SummonedUnit/DefenseObject 제외, aura_membership admission 실패 시 배치/queue/source 원복을 추가했다. 정상 tactical relocation을 cooldown 경계까지 진행해 범위 이탈/재진입을 세 번 반복했고 channel caster의 태세/이동/교대 거부 시 state 불변을 확인했다. World의 channel/Hello 모두 외부 transaction의 철수 후 실패에서 복원되고, 정상 철수 후 재진입은 빈 ledger로 시작한다.

최초 추가 시험의 RON 합치기, live deployment salt 재사용, DefenseObject를 직원 배치 명령에 보낸 fixture, Position 좌표 타입, scenario 초기 직원의 tactical state 부재, 실제 배치 stance cooldown 미반영 오류는 r11_tests_018~022에 그대로 보존했다. 이를 production 정책을 완화하지 않고 실제 지원 spawn/deploy/cooldown 경로로 수정했다. r11_lib_regression_002의 1572개 회귀는 앞선 source에 대한 결과로 유지하며 최신 source broad 통과로 대체 표기하지 않는다.

남은 검증에는 overlay 무기 trigger/실제 HP-loss 기반 흡혈 연결, 호환 무기 비교, channel 추가 basic/작업/사망 경계와 기본 공간 projectile 한계 fault 등이 있다. R12의 protection receipt와 같은 actual-HP 소비 경계에서 흡혈 fixture를 닫고 최종 A11/A12에 함께 연결한다.
