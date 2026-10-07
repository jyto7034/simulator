# WP12 상세 설계 — 보호·생존 중재와 제어

현재: 기능 구현·최종 자체 검증 자료 동결, 독립 감사 요청 대기. 통합55조건 중54 자체 검증/X09 환경 제한1. 최신 판정은 COVERAGE.json / REVIEW_HANDOFF.md가 소유한다. 아래 단계별 메모는 당시 이력이며 독립 감사 판정이 아니다.

상태: 보호막·생존·HP 비용·recharge·제어·패링 구현 및 집중 검증 진행 중. 이 문서는 A12 수용 완료나 독립 감사 판정이 아니다. R11의 추가 검증과 함께 구현하고 최종 source의 55개 수용 조건으로 다시 연결한다.

## 현재 접속점과 소유

`core/damage_runtime.rs::execute_committed_damage_inner`는 strike를 등록한 뒤 basic OnAttack/OnHit plan을 수집하고 기존 formula를 계산한다. `prepare_damage_fact` 이전에 보호 결과를 확정해야 HP 적용, 실제 HP loss, 공명, 사망, 관측이 같은 결과를 읽는다. 부착 효과를 차단하는 parry 판정은 trigger plan dispatch보다 앞서야 한다. 기존 감쇄를 통과한 초과 피해에 감쇄를 다시 적용하지 않는다.

`core/sustained.rs`의 finite source owner를 확장한다. shield는 activation당 capacity와 remaining이 한 번만 존재하고 membership은 UnitInstanceId 참조만 가진다. 개인 shield는 대상 한 명, shared shield는 caster 주변의 실제 직원과 caster를 대상으로 한다. Guest·소환물·방어 object는 shared 수혜자가 아니다. source 철수/사망, target 소실, expiry, depletion, terminal은 같은 exactly-once 종료 경계를 쓴다. ordinal은 조회/expiry 식별자이며 발동 권한은 private RuntimeFactSource가 소유한다.

초기량 expression은 기존 R06 catalog/compiled plan/source generation과 snapshot을 재사용한다. shared producer는 SelfUnit+Instant로 한정해 대상 수에 따른 pool 복제를 방지한다. source 생성 때 한 번 평가한 capacity를 저장하며 membership 변경·조회·일반 회복이 다시 초기량을 계산하지 않는다. recharge는 별도 operation으로 기존 capacity까지만 회복하며 소진되어 종료한 activation을 되살리지 않는다.

## 확정할 처리 순서

1. active target와 직접 적 strike eligibility 확인; parry/무적 판정.
2. 기존 피해 formula와 target 감쇄 한 번.
3. 개인 shield → shared shield. 각 분류는 priority 내림차순, creation ordinal 오름차순. 동일 시각 피격은 기존 stable event/target 순서.
4. ordinary damage의 HP1 → lethal일 때 one-use death prevention. 둘 다 명시 source record만 변경한다.
5. 최종 receipt에 calculated/mitigated/absorbed/prevented/actual HP loss/death와 사용한 protection 출처를 보존한다. 기존 formula segment는 계산 설명으로 남긴다.
6. 실제 HP 변경과 일반 죽음, 실제 loss 기준 공명·흡혈·후속 실행.

HP cost는 별도 typed operation/CostFact로 shield, parry, HP1을 우회한다. 기본은 지불 뒤 HP≥1; lethal cost는 authored opt-in이다. 미지불은 enclosing command/amount/ledger mutation 전체 실패다. R09 corrosion special kill은 이 ordinary damage pipeline에 넣지 않는다.

parry는 first eligible direct enemy strike의 damage와 그 strike의 부착 effect만 차단한다. 성공 counter는 original attacker만 대상으로 한 번 실행하며 소실 시 no-target이다. 이후 짧은 invulnerability에는 counter를 반복하지 않는다. counter/reflection/self damage/cost는 parry 입력에서 제외한다. miss는 자기 reduction record만 잠시 비활성화하고 장비·다른 source를 보존한다. black-swan형 상시 fixture의 사본 장착/배치 source와 active1 예외는 별도 제한된 정책으로 기록해야 하며, 모든 fragment를 passive로 바꾸지 않는다.

## capability 표와 actor 경계

| 제어 | 이동 | basic/새 skill | 진행 cast | 저지 | target | 만료 |
| --- | --- | --- | --- | --- | --- | --- |
| Slow | source별 속도 modifier 합성 | 유지 | 유지 | 유지 | 유지 | 자기 modifier만 제거 |
| Root | 이동만 정지/시작 금지 | 유지 | 유지 | 유지 | 유지 | 자기 이동 lease만 제거 |
| Taunt | 기존 적 이동 정책 유지 | eligible source로 basic target 선택 | 유지 | 유지 | source 생존/도달 가능성 검사 | 자기 강제 target만 제거 |
| Pull | 적 combatant를 검증한 유효 위치로 이동 | 현재 windup/경로 상태와 합치도록 무효화 | interrupt 정책 명시 | 이전 저지 연결 해제·재계산 | 갱신 | instant 완료 |
| Stun/Freeze | 기존 hard CC | 기존 시작 금지 | 기존 interrupt | 기존 정책 | 기존 정책 | 기존 buff source 계약 |
| Silence | 유지 | skill 시작 금지, basic 유지 | 기존 시작된 channel 유지 | 유지 | 유지 | 기존 buff source 계약 |

신규 강제 이동은 적 combatant만 지원하며 플레이어의 자유 이동 AI를 열지 않는다. pull은 authored 범위, map valid tiles, boundary, occupancy를 검증하고 unsupported actor/target 조합을 publication 또는 명시 runtime 오류로 진단한다. Root/Taunt를 Stun으로 낮추지 않는다.

## 검증과 계량

A12-01의 실제 shared600/A400/B350 oracle부터 개인+공유·동시 타격·priority·증폭1회·재입장·재시전·stale expiry·철수로 넓힌다. Heal/Rest/recharge/cost 경계를 서로 다른 fact로 확인한다. HP1/death prevention/corrosion 조합과 parry success/miss/counter recursion을 정상 RON/명령에서 실행한다. shield absorbed와 실제 HP loss0인 타격의 R06 gate·공명·흡혈·조회도 독립 oracle로 둔다.

새 pool/member/protection receipt/control lease에는 allocation 전 native admission, authoring entity/work/clone, transaction rollback, source currentness를 연결한다. 여러 피해 중간 fault에서도 HP·잔량·사망·queue·관측이 함께 rollback하며 정상 retry와 결과가 같아야 한다. battle-local 상태는 attempt 밖으로 저장하지 않고 기존 V9 durable save와 독립시킨다. 최종 World 결과/save/restore, public CLI, R05 peak 및 전 수용표는 통합 단계에서 재실행한다.


## 구현 중 계약 확정 및 증거 (2026-09-30)

- `r12_tests_006`: 보호막 6개 집중 시험 통과. shared600/400/350, personal→shared priority, HP1/one-use 우선순위, 비용 지불 불가/허용 사망, Heal과 recharge 구분.
- `r12_tests_008`: 위 보호와 실제 Slow/Root/Taunt/Pull을 합친 9개 통과. Pull은 ground opponent combatant, radius≤0.5를 지원하며 cardinal tile step을 최대 64번 검사한다. 장애물/움직이는 body와 겹치는 segment 앞에서 멈추고 결과는 tile center다. unsupported actor/air/body는 typed runtime 오류다.
- `r12_tests_010`: 13개 통과. Parry의 성공/다단 무적/반격 1회/부착 억제, 실패 시 자기 감면만 제거 후 복구, counter와 binding publication 거부, 5개 fault 지점 rollback을 포함한다.
- `attached_to`는 동일 action의 앞선 Damage/DamageExpr id만 참조한다. 지원 attachment는 ApplyBuff, ModifyStats, InterruptCast, Pull, Sustained::Control이다. 부착 효과를 앞선 damage로 옮기거나 지연 재정렬하지 않는다. 표현식/후속 effect 순서는 그대로다. 동일 대상의 action strike가 deflect되면 결속된 attachment만 대상 필터에서 빠진다.
- Parry의 window/recovery는 `[start, end)`다. 양수 base damage를 가진 직접 적 BasicAttack/Ability만 eligible이다. Environment 반사, BuffTick, runtime reaction counter, self damage, HP cost는 제외한다. 성공 record는 다시 arm하지 않는다. 반격은 별도 zero-focus/zero-delay Instant CastTarget damage leaf로 original attacker만 실행한다. 입력 공격자의 소실은 `AttackerUnavailable`이며 새 대상을 찾지 않는다. 반격 오류는 enclosing timestamp와 함께 rollback해야 한다.
- Parry의 반격 출처에는 private reaction 표식을 부여하고 equipment/fragment provider를 상속한다. public receipt는 activation/kind/counter outcome과 protection source를 보유한다. 기본 OnAttack/OnHit dispatch는 deflect 시 실행하지 않고 current-hit formula modifier만 읽는다. Shield가 막는 경우 기존 OnHit 의미는 유지한다.
- BasicLifesteal은 `r12_tests_014`의 16개 집중 시험에 포함되어 통과했다. Weapon의 OnAttack/SelfUnit에 명시한 percent와 max_heal을 release snapshot으로 고정한다. 순간 광역은 전체 actual HP loss 합계, 관통은 유한 projectile 종료 시 전체 합계를 한 번 정산한다. 반올림은 합산 후 내림, cap도 한 번이다. Source가 이미 비활성이면 회복하지 않는다. ordinary HealFact에 실제 equipment provider를 남기고 skill/Goodbye/반사/overkill/shield absorbed는 합계에 넣지 않는다. 실제 콘텐츠 cap 값은 fixture 값으로 확정하지 않는다.

Black-swan passive fragment lifecycle, 추가 lifecycle/native-fault/World·CLI 시험과 최종 55조건 통합 검증은 아직 미완료다. 위 집중 통과를 A11/A12 전체 완료로 읽지 않는다.


## 흑조 상시 예외의 확정 계약

`ActiveSkill.passive_guard: Option<ParryGuardDef>`만 허용한다. 세 percent는 0..100이며 모든 form의 skill에 SelfUnit/Instant/유한 reduction0 패링이 있어야 한다. 별도 passive 슬롯·자산·배치 record를 만들지 않고 실제 deployed unit의 R07 `FragmentSourceSnapshot`에서 파생한다. 공명0에서도 적용되며 미배치/철수/사망 시 비활성, 재배치 시 같은 소유 사본을 다시 연결한다. field-work 효과는 추가하지 않는다.

직접 적 BasicAttack/Ability의 기존 damage-reduction 합성에 자체 percent를 더한다. 같은 사본의 실패 record가 가리키는 `[window_end,recovery_end)`에서 자체 percent만0이며 반사 확률은 유지한다. 다른 장비 stat/감쇄는 유지한다. 파편 피해 기여를 반사량이나 총감쇄량으로 대신 계산하는 별도 유물 축적은 없다.

반사 base는 보호/HP1 뒤 해당 strike의 첫 양수 actual HP-loss segment × percent를 내림한 값이다. 대상마다 한 번이며 후속 segment를 재추첨하지 않는다. seed+target+strike ordinal의 고정 namespace로 0..99를 만들고 `roll < chance`를 사용한다. 원래 타격은 취소하지 않는다. 원래 damage type으로 반격 대상의 ordinary 방어를 한 번 거치며 최소 피해는0이다. 원래 공격자가 없거나 보호받은 직원이 사망하면 실행하지 않고 typed outcome에 남긴다. 반사는 `reaction` source와 실제 copy/form을 유지하며 incoming skill의 authored observation을 상속하지 않는다. 피해원/부착 억제/비용 계약은 앞 절과 같다.

`ProtectionReceipt.passive_guard`는 policy, 자체 경감 억제 여부, roll, reflected base, outcome, 실제 사본 출처를 남긴다. `sustained_effects.passive_guards`는 활성 배치와 policy 및 실패 가능한 half-open 구간 목록을 제공한다. 조회는 시간을 진행하지 않으며 구간은 현재 battle clock과 함께 해석한다. 성공 record는 실패 구간 목록에서 빠진다. 이 query는 새 active effect나 skill slot으로 집계하지 않는다.

`r12_focused_tests_015` 21/21: 상시0공명, 다단 1회 반사, miss 자기경감, 성공/무적 반사 배제, reflection fault 원자복원, 실제 Grant→Equip→Deploy→Withdraw→Redeploy→Retreat 사본 lifecycle을 확인했다. 추가 경계 시험과 최종 55조건·CLI·native matrix는 계속 진행한다.
