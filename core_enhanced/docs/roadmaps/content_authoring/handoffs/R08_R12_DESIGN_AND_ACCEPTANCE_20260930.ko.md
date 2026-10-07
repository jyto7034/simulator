# R08~R12 — 코드 기반 구현 방향과 수용 시험

2026-09-30 · v1 · [상위 구현 지시서](R08_R12_CONTINUOUS_IMPLEMENTATION_REQUEST_20260930.ko.md)의 필수 부속 문서.
아래 ‘현재’는 작성 시점에 읽은 코드, ‘권장’은 구현 방향이다. 새 타입명은 존재하는 API라고 가정하지 않는다.
정책 추천은 §7에 별도로 모았다. 구현자는 정식 계획에서 exact 결정과 실제 심볼을 확정한다.
코드 경로는 별도 표기가 없으면 `core_enhanced/` 기준이다.

## 1. 공유 경계와 구현 순서

```text
RON/외부 팩
  → raw 검증·최종 참조 검증·compiled plan·fingerprint
  → 요청 권한별 admission (플레이어 / 보스 / 기존 자동 발동)
  → 공통 action과 유한 flow
  → 기존 Instant / Projectile / Area 실행부
  → 상태·modifier·pool / 피해·생존 중재
  → 실제 fact·ActionHandle 완료·wave bookkeeping
  → attempt 결과·World commit·DTO·저장
```

1. R08에서 보스 요청을 공통 action 포트로 연결한다. 플레이어 전용 자격을 전역 완화하지 않는다.
2. R09에서 시전 대체·특수 처치·직원 대가·강제 정지·Breakdown을 종단으로 닫는다.
3. R10에서 flat step 실행의 표현력을 구조화 Flow로 확장한다. R08/R09 consumer를 새 포트에 연결해 다시 검증한다.
4. R11에서 source별 가역 효과·channel·aura·기본 공격 overlay를 추가한다.
5. R12에서 R11 membership과 R05 자원 계약을 소비하는 보호막·중재·제어를 추가한다.
6. 최종 source에서 R06~R12 공유 계약과 §8의 종단 조건을 함께 검증한다.

R별 AT는 raw/compile/runtime/consumer/test의 동반 전환 경계다. 다섯 기능을 한 거대한 runtime transaction으로 묶는 뜻이 아니다.
요청 한 건·시간 진행 한 번 등 기존 authority commit 단위 안에서 실패가 부분 변경을 남기지 않게 한다.
자료구조를 먼저 모두 만들고 마지막에 실행부를 붙이는 순서보다 각 WP에서 실제 공개 입력 한 경로를 먼저 연결한다.

## 2. R08 — 보스의 명시 스킬 실행

### 확인한 현재 코드

| 파일·심볼 | 현재 동작과 변경 지점 |
| --- | --- |
| `src/game/data/boss_runtime_data.rs` — `BossTaskData`, `CompiledBossTask`, `task_result_set`, `task_capabilities`, `compile_task` | RouteCombat/AcquireTarget/Move/Wait/Summon/Signal 등이 있으며 명시 스킬 요청 task는 없음. raw·결과 전이 coverage·capability·compiled를 함께 확장 |
| `src/game/battle/core/boss_runtime.rs` — `ActiveBossTask`, `admit_state_tasks`, `cancel_current_state`, `BossRuntimeTransitionPlan::commit` | activation identity와 후보 transition/effect 적용 구조가 있음. state/task 소유의 요청·대기·취소를 여기에 연결 |
| `src/game/battle/core/sim.rs` — `request_skill_action`, `start_manual_skill_cast_with_retention`, `resolve_manual_skill_base_request` | 공개 action 요청도 현재 Player·Manual·장착 skill·공명 만충 검사를 사용. 보스가 그대로 호출하면 거부됨 |
| 같은 파일 — `invoke_ability_with_accepted_target_tile` | 기존 cast/delivery 실행과 accepted handle 연결을 재사용할 출구. 단순 호출 성공을 task 완료로 간주하면 안 됨 |
| `src/game/battle/runtime_facts.rs` — `ActionSnapshot` | accepted/released/terminal/effects_settled 시점과 pending step/delivery가 이미 구분됨 |
| `src/game/battle/core/runtime_facts.rs` — `query_action`, result retention | 기존 handle의 유효성·보관·settlement를 재사용. 숫자 ordinal로 capability를 재생성하지 않음 |

### 구현 방향

- 권장 `StartAction` task는 skill ref, target/aim, facing, 기다릴 milestone, timeout, 취소 정책을 typed 입력으로 받는다.
  현재 catalog에 실제 있는 스킬과 지원 delivery만 허용한다. 없는 skill·맞지 않는 target·무한 대기를 publication 전에 진단한다.
- 공통 내부 action admission/실행부를 추출하고 그 앞의 player/boss/auto 권한 검사를 분리한다.
  보스를 `Side::Player`나 `Manual`로 위장하거나 모든 보스에 공명 규칙을 삭제하는 전역 변경을 하지 않는다.
  boss task의 스킬 사용 권한·자원 계약을 명시하고 기존 RouteCombat의 자동 행동과 중복 실행되지 않게 한다.
- 상태 입장 후보에서 요청 admission, 필요한 handle 보관 공간, 방향/target 변화, 취소 계획을 먼저 검증한다.
  후속 action 요청이 실패할 수 있는데 먼저 boss state를 live commit하는 순서를 만들지 않는다.
- task는 actor/behavior/pattern/state/task activation과 action을 결속한다. 새 state가 같은 task ID를 다시 사용해도 옛 완료는 무효다.
  R04 handle이 terminal 뒤 보관되지 않을 수 있으므로 실제 waiter 수명 동안 필요한 retention을 예약하고 종료 때 해제한다.
  매 tick 전역 action 목록을 스캔하지 말고 완료/wake와 필요한 조회를 연결한다.
- `Released`는 release에서, `EffectsSettled`는 실제 모든 유한 delivery 종료에서 깨운다.
  terminal 취소·실패와 이미 방출된 delivery의 settlement는 다른 축이다. 지속 효과 때문에 종료 보장이 없는 조합은
  timeout/명시 지원 계약 없이 EffectsSettled를 기다리게 하지 않는다.
- state 이탈은 그 activation이 소유한 미방출 작업만 취소한다. 이미 방출된 독립 projectile/ground area는 기존 수명을 따른다.
  target/facing snapshot과 source attribution을 임의로 최신 값으로 덮어쓰지 않는다.
- task 실패를 boss graph의 명시 결과 route에 연결한다. target 부재·요청 거부·timeout·취소를 silent success로 합치지 않는다.
  wave 생성·승패·보상은 기존 owner에 남긴다. 이 단계에서 보스 전체를 스킬 VM으로 옮기지 않는다.
- compiled 설명/descriptor, schema 검증, source 위치, budget/CloneShape를 함께 갱신한다.

| 조건 | 필수 입력과 기대 결과 |
| --- | --- |
| A08-01 | 외부 RON 보스의 stationary actor가 명시 방향으로 지원 스킬을 실행. 실제 피격 대상·출처 일치, Player-only 우회 없음 |
| A08-02 | 시험 시각 t0 접수/t100 release/t300 impact: Released waiter는100, EffectsSettled waiter는300에 진행. 타격 전 요청 접수를 완료로 취급하지 않음 |
| A08-03 | t50 state 이탈은 미방출 취소; t150 이탈은 독립 projectile 보존. 기존 보스 이동·RouteCombat 작업과 중복 발사 없음 |
| A08-04 | target 소실·잘못된 skill·자격 실패·timeout을 각각 실제 result route로 처리. 실패 시 방향/자원/상태/순번의 허용된 변화가 계약과 일치 |
| A08-05 | 이전 activation의 늦은 wake/완료·중복 완료는 새 state에 영향 없음. 반복 진입 후 handle/lease 보관량 회수 |
| A08-06 | raw/compiled 참조 오류·빠진 실패 route·capability 충돌·미지원 milestone 조합은 정확한 진단, candidate generation 미게시 |
| A08-07 | clone/fault 주입 뒤 boss state와 action 등록이 함께 rollback. 기존 wave·summon·결과 authority 회귀 |

## 3. R09 — 침식·강제 정지·Breakdown

### 기준과 현재 코드

침식은 [침식 정책 메모](../../../tmp/ego_corrosion_system_design_notes_2026-08-19.ko.md)의 승인된 방향을 인수해
P2G에서 exact 실행 의미로 닫는다. 최종 보스 파편 제외, 공통 trauma 임계·공명 만충, 일반 시전 자격,
별도 버튼/세트 조건 없음, 현재 적 특수 kill, 직원 철수·trauma100, 부가 효과 억제는 보존한다.

| 파일·심볼 | 현재 사실·변경 지점 |
| --- | --- |
| `core/sim.rs` — manual start, `AutoCastEnd`/`ManualCastEnd` match | manual 접수는 pending cast·lock·event를 생성. 두 종료 경로는 start_seq stale 검사를 하고 invoke 뒤 공명을0으로 reset. ‘입력 접수=release=침식 시작’으로 합칠 수 없음 |
| `core/build.rs` — `withdraw_unit`, 취소/정리 helper | 직원 철수와 release 이전/이후의 구분을 재사용할 경계. 실제 실행 전에 helper 본문·호출 순서를 재대조 |
| `core/death_runtime.rs` — `finalize_prepared_death`, `death_trigger_commands` | wave death epoch·death fact·graveyard·buff/이동 정리 뒤 OnDeath/OnKill/OnAllyDeath를 실행. 현재는 특수 kill suppression 인자가 없음 |
| `world/combat.rs` — `reconcile_employee_attempt`, `advance_active_battle_staged`, `handle_complete_combat_result_inner` | 전투불능을 attempt roster에 반영하고 결과 확정에서 world에 적용. 즉시 terminal에서도 침식 대가를 먼저 보존해야 함 |
| `employee.rs` — `EmployeeCondition`, `apply_incapacitation` | condition은 physical_damage/trauma. incapacitation은 신체 부상+1 및 mitigation/death prevent도 처리하므로 침식 trauma100 대가에 그대로 쓰면 틀림 |
| `world/application.rs` — `ApplicationRoute`, `execute_application` | clone 후보→실행→검증→단일 root commit. 강제 정지의 mutation guard와 rollback 적용 지점 |
| `world/state.rs` — `simulation_delta_for_tick` | paused이면 None, 아니면 speed 비율로 raw delta를 변경. 침식 연출 시간을 이 battle delta로 재면 배속·Pause에 잘못 종속됨 |
| `world/node_flow.rs` — `plan_complete_current_node_with_context_and_encounter_progression`, commit 함수들 | node 완료·recovery·층 전이의 기존 후보 경계. Breakdown 카운터는 여기서 실제 완료 한 번과 결속해야 함 |

### 구현 방향

1. **실제 시작 지점:** 일반 명령 접수와 실제 침식 판정용 단일 start gate를 구분한다. Pause 중 접수만으로 침식하지 않는다.
   manual/auto가 같은 gate를 지나며, resume 후 첫 적격 실행 시점과 준비 중 취소의 관계를 §7 정책으로 닫는다.
   단순히 현재 `ManualCastEnd`를 ‘실제 시작’이라고 이름만 바꾸지 않는다. 공명 소비 전 판단에 필요한 실제 입력을 보존하되
   과거 snapshot과 현재 trauma/자격을 혼용하지 않는다. 같은 activation은 한 번만 대체된다.
2. **대체 transaction:** 유효한 적 집합을 deterministic 순서로 capture, 필요한 fact/event/clone 공간을 admission한 뒤
   시전자 철수, 공명/행동 종료, attempt-local trauma100, 적 처치, 연출 상태를 같은 논리적 성공 경계로 묶는다.
   적 처치로 terminal이 먼저 관찰돼 직원 대가가 사라지는 흐름을 금지한다. 일반 시전과 침식 양쪽을 실행하지 않는다.
3. **특수 kill:** damage UINT_MAX나 Despawn이 아니라 명시 death cause/dispatch policy를 둔다.
   필수 lifecycle·wave death epoch·죽음 fact·결과 bookkeeping은 수행하고 선택적 OnKill/OnDeath/OnAllyDeath와
   장비·파편·미래 유물의 반응 dispatch는 억제한다. ‘source=None’만으로 억제를 흉내내지 않는다.
   현재 적만 capture하며 처치 중 후속 wave가 생성되더라도 새 적을 같은 침식에 추가하지 않는다.
4. **강제 연출 정지:** 사용자 Pause와 별도 상태/원인을 둔다. Core에 host elapsed를 명시적으로 전달하는 포트를 사용하며
   raw elapsed는 배속 계산 전에 소비한다. 직접 wall clock/Instant를 gameplay kernel에서 읽지 않는다.
   게임 명령·직접 elapsed/event-bucket advance·trial·admin 등 실제 공개 mutation 우회 경로를 분류해 검사한다.
   연출 시간 갱신과 disconnect 제어는 허용된 system route로 구분하고 Resume은 해제할 수 없다.
   read-only query는 revision/RNG/state를 바꾸지 않는다. 만료 직후 남은 delta 처리도 §7에서 고정한다.
5. **terminal과 host:** 마지막 적이 죽어도 연출 시간이 사라져 Resume/결과 확인으로 우회되지 않게 한다.
   권장안은 terminal 결과를 준비해 보존하고 강제 정지 만료 후 기존 결과 전이를 게시하는 방식이다.
   다른 상태 구조를 택해도 대가·결과·연출 종료 각각 exactly once를 입증한다. 새 game_server 기능을 만들지 않고
   현재 Core 공개 host 포트에 fake clock으로 검증한다. 필요한 host 호출 계약 변경은 문서화한다.
6. **Breakdown:** employee identity에 속하는 명시 잔여 node 수를 두고 psychology memory TTL과 분리한다.
   trauma100 진입·회복·재진입, 노드 완료, SavePoint/Gate 회복, 일반 사망 자산 회수, checkpoint restore를 연결한다.
   retreat/미완료 재시도는 차감하지 않는다. 전투 밖 만료 사망을 combat DeathPrevent로 막지 않는다.
   R07 exact copy 회수와 역결속도 함께 처리한다. 새 카운터를 넣는 모든 clone/DTO/save 검증을 갱신한다.
7. **위험 표시:** selected fragment copy/form의 실제 대상 여부·trauma·공명·자격에서 파생하는 DTO를 제공한다.
   UI 색상·연출 asset은 만들지 않는다. authored 임계·연출 시간은 typed 검증하고 미작성 값을0초 성공으로 처리하지 않는다.

| 조건 | 필수 입력과 기대 결과 |
| --- | --- |
| A09-01 | 임계 아래/경계/위, 공명 부족/만충, 일반/최종 보스 파편, 잘못된 사본·자격의 조합. 일반 시전 또는 침식 또는 typed 거부가 명시 정책과 일치 |
| A09-02 | Pause 중 manual 입력→query→resume와 auto 준비: 접수만으로 침식 없음, 실제 start gate에서 한 번 판정. 준비 중 자격 변화·취소·stale event 포함 |
| A09-03 | 침식 대체는 일반 피해를 추가 실행하지 않고 공명 비용을 한 번만 처리. 현재 적만 사망, 미출현 required wave는 유지 |
| A09-04 | OnKill/OnDeath/OnAllyDeath에 관찰 가능한 효과를 달아 미발동 확인. wave death epoch/승리 검사/필수 정리는 정상. HP1·shield 도입 뒤도 재검증 |
| A09-05 | 마지막 적 처치로 즉시 terminal인 경우도 직원 Withdrawn·attempt trauma100·결과 확정 한 번. 신체 부상 증가 없음, 같은 전투 재배치 거부 |
| A09-06 | 0.5×/1×/2×·Pause·disconnect/resync의 fake host elapsed: 동일 연출 길이, battle time 정지, 강제 Resume 거부. 만료 후 사용자 Pause는 별도 유지 |
| A09-07 | 모든 공개 mutation route를 분류한 행렬. 거부 시 state/resource/event/revision/RNG 불변. query/preview 반복은 무변경, 직접 step으로 우회 불가 |
| A09-08 | Breakdown 진입→완료 node→회복/재진입/만료. 치료와 만료 동시, retreat·실패 재시도·중복 완료·checkpoint 잔여 수 복원 |
| A09-09 | 만료 사망의 장비·R07 사본 회수 exactly once, 소비 효과 제거·사망 방지 경계. 저장 decode 오류는 원본/현재 state 보존 |
| A09-10 | 적 capture/fact/직원 대가/연출 출력 각 단계 fault 주입: 부분 kill·부분 대가 없음. retry가 처음 성공과 같은 결과 |

## 4. R10 — 구조화 Flow와 대상·전달

### 현재와 전환 지점

`ability.rs::SkillDef`는 `steps: Vec<SkillStepDef>`이며 step은 delay/target/when/repeat/delivery/effects를 가진다.
`SkillStepRepeat`는 Once/Times/ByBuffStacks, 조건은 Always/IfPreviousStepDealtDamage/IfCasterHasBuff다.
`data/skill_data.rs::RawSkillDef`, RawSkillStepDef와 database validation이 작성 입력을 소유한다.
`core/skill_runtime/cast.rs`는 step 결과·pending delivery·deferred step·cleanup을 이미 누적한다.
`core/sim.rs::execute_skill_step`와 `build_ordered_skill_effect_commands`, `skill_runtime/projectile.rs`·`area.rs`가
실제 전달/효과를 수행한다. 기존 R06 amount 평가·source·Snapshot/Live 연결을 이 경로에서 유지해야 한다.

### 구현 방향

- 작성 AST와 compiled Flow를 분리한다. 최소 구조는 Sequence/Branch/Parallel/유한 Repeat/Wait/기존 action leaf다.
  사용자 함수·재귀·임의 코드 실행은 추가하지 않는다. legacy steps를 계속 지원한다면 compile-time에 한 실행 계획으로
  낮추며 별도 두 runtime을 유지하지 않는다. 공개 schema 전환은 현재 승인 호환 계약과 fixture 영향부터 확인한다.
- node/action/effect 주소를 안정적으로 구분한다. repeat occurrence와 branch identity는 runtime key이고 작성 주소를 덮어쓰지 않는다.
  action carrier 변경으로 같은 action의 무관한 self-heal effect footprint가 확대되지 않게 한다.
- typed result slot은 생성 node, 읽을 수 있는 시점, 단위, scope를 검증한다. 아직 실행하지 않은 분기 결과·미완료 delivery를
  0으로 바꿔 읽지 않는다. R06 실제 HP 손실 gate와 새로운 hit/absorption 결과를 구분한다.
- Sequence는 작성된 milestone을 기다린다. Parallel은 OS thread가 아니라 deterministic scheduler의 분기이며
  join 완료·실패·취소 정책을 명시한다. 같은 시각의 순서는 stable node/occurrence·기존 event priority로 결정한다.
- `Times(n)`은 기회 n번, 시간/간격 기반 반복은 별도 의미다. interval만 바꿨을 때 count가 암묵적으로 변하지 않는다.
  무대상 회차도 소비하고 다음 적에게 적립하지 않는다. zero interval 허용 여부·count0·상한·checked 시간 overflow를 검증한다.
  대량 반복을 미리 Vec로 펼치지 말고 유한 continuation으로 실행하며 max work/queue/retained를 charge한다.
- target query, 대상 선택, effect 적용 대상, aim/anchor/origin/facing, retarget 시점을 서로 구분한다.
  cast snapshot tile과 각 occurrence의 actual resolved units는 같은 값이 아니다. 공중/지상·직원/소환물 자격을 명시한다.
- Instant/Projectile/Area는 기존 커널에 adapter로 연결한다. 없는 target에서 후속 self effect·step·delivery 정리를 건너뛰지 않는다.
  R06의 empty impact, piercing expiry, actual group aggregation, release Snapshot vs Live, 최근 관측1건 수명을 보존한다.
- cancel은 미방출 continuation을 닫고 방출된 자식의 계약상 수명을 보존한다. 부모 종료가 자식의 출처·자원 lease를 조기 파괴하지 않게 한다.
- public compiler/descriptor/fingerprint/preview/explain/CloneShape·예산을 함께 전환한다.
  실행 가능한 RON이 없는 enum 추가, private typed fixture만의 성공은 완료가 아니다.

| 조건 | 필수 입력과 기대 결과 |
| --- | --- |
| A10-01 | 광역 피해로 적3명 피격 후 self-heal1회. 대상 수만큼 자기 회복하지 않음. 0 실제 피해/빗나감의 결과 조건은 기존 HP-loss 의미 유지 |
| A10-02 | 나무꾼형3타: strike/segment/step/occurrence 주소와 누적 결과 분리. 병행 branch의 완료 순서가 달라도 authored result order 유지 |
| A10-03 | 루돌타형 유한 n회·중간 무대상: 기회 소모·고정 종료, 나중 적에게 적립 없음. interval-only 변경 시 n 보존 |
| A10-04 | Branch의 미선택 경로 미실행, Sequence wait, Parallel join·실패·부분 취소·중복/stale wake의 실제 결과 검증 |
| A10-05 | self/unit/tile·고정 anchor/추적·step retarget·명시 facing을 Instant/Projectile/Area에서 검증. 대상 소실과 empty set 후속 완료 포함 |
| A10-06 | release 전 취소와 후 취소, R08 waiter, R09 시작 gate·강제정지가 새 Flow에서도 동일 의미 |
| A10-07 | 잘못된 result slot/type/unit/forward reference, 미지원 조합·무한 구조·반복 상한·시간 overflow를 publication 전에 거부 |
| A10-08 | 큰 유한 반복/깊이/분기에서 allocation 전 admission·work fault·rollback, 순번/RNG/queue/최근 관측 보존 |
| A10-09 | 공개 RON→compiled→CLI/실제 전투와 R06 Snapshot/Live·집계·trace·D04-F01/관통 만료 회귀 |

## 5. R11 — source별 가역 효과·channel·기본 공격 교체

### 현재와 전환 지점

`SkillEffectDef::ModifyStats`와 `core/sim.rs`의 effect-command 변환에는 일회성 stat 변경 경로가 있다.
`core/types.rs::ActionLocks`는 movement/basic_attack/resonance_gain의 until 값을 max로 합친다.
이 구조만으로 ‘source A가 종료되면 A의 lock만 제거’를 표현할 수 없다.
`battle/buffs.rs`의 BuffKind는 PeriodicDamage/Stun/Freeze/Silence이고 metadata·reapply 규칙이 있다.
`core/basic_attack.rs`는 기본 공격 target 선택을, 기존 start/release와 projectile 경로는 실제 공격을 소유한다.
`resource/runtime.rs`는 activation/deployment/delivery 수명 및 hold를 이미 구분한다.

### 구현 방향

- 권장 `ModifierLedger`의 record는 source capability, instance/activation, 대상, property·operation, lifetime을 갖는다.
  immutable base + 활성 record로 유효 값을 계산한다. `+20%` 적용 후 역수 곱셈으로 복구하거나 모든 modifier를 초기화하지 않는다.
  일회성 ModifyStats의 승인 의미를 가역 효과로 몰래 바꾸지 말고 producer를 구분해 단일 stat 합성에 연결한다.
- action/이동/저지 제한도 source-local lease로 합성한다. A 종료 후 B의 hard CC·focus·다른 lock이 유지되어야 한다.
  기존 until 시간만 덮어써 소유권을 잃지 않는다. lifetime capability를 diagnostic ID로 위조하지 않는다.
- aura membership는 effect source와 현재 eligibility/range에서 파생한다. 입장/퇴장·새 배치·철수·시전자 종료 때 diff로 적용한다.
  범위 재계산마다 buff를 새로 누적하지 않는다. regular 기본 공격 전용 증폭을 범용 Attack으로 구현하지 않는다.
  태아는 자신·Guest·소환물·방어 object 제외, 겹침 비중첩, 유효 수혜자 없는 시작 거부를 대표 oracle로 사용한다.
- channel은 targeting focus나 준비 focus와 별개다. 자연 종료/재입력/명시 중지/Hard CC/철수/사망/terminal/retreat를 typed 원인으로 처리한다.
  태아의 일반 피해는 channel을 끊지 않고 Silence는 새 시작만 막는다. 종료와 aura 제거는 함께 commit한다.
  시전 중지 후 새 기본 공격은 새 windup으로 시작하고 밀린 공격을 catch-up하지 않는다.
- Hello overlay는 기본 공격 delivery/range/profile을 임시 교체하되 무기 출처·archetype·배치 affinity·block capacity를 바꾸지 않는다.
  overlay 부여로 공격 주기를 즉시 reset하거나 원본과 overlay를 함께 발사하지 않는다.
  이미 release한 공격은 당시 snapshot을 유지한다. 준비 중인 공격에 대한 적용 지점도 명시하고 시험한다.
- Hello 자연 만료는 원래 기본 공격 복귀만 한다. 재입력은 exact activation을 소비해 Goodbye를 한 번 실행하고 복귀한다.
  공명 만충·추가 소비·새 Hello 갱신을 요구하지 않는다. 남은 시간을 ValueExpr 입력으로 연결해 확정 시점을 기록한다.
  Goodbye를 basic attack으로 표시하지 않으며 무기 기본 공격 흡혈을 잘못 적용하지 않는다.
- ‘다음 기본 공격1회’ record는 새 정상 기본 공격에 한 번 결속한다. preview/취소/잘못된 요청에 소비되지 않는다.
  commit/release 중 정확 소비 시점과 취소 시 처리는 정책 기록과 시험으로 닫는다.
- 종료 후 모든 source record/membership/hold를 회수하되 사본 영속 개화·다른 source를 지우지 않는다.
  이번 source별 record를 R11의 조회 기반으로 노출해 R17이 사용할 수 있게 하되, 전체 효과 집계 언어·감염 연결망은 만들지 않는다.

| 조건 | 필수 입력과 기대 결과 |
| --- | --- |
| A11-01 | source A/B의 additive/scale modifier를 겹치고 각각 제거. base·남은 B 보존, 반복 입출입에 수치 drift 없음 |
| A11-02 | 태아 aura 입장/이탈·새 배치·재배치·종료와 겹침: exact 수혜자, 비중첩, 기본 공격만 강화, 기존 released snapshot 보존 |
| A11-03 | channel 중 공격·시전·태세·전술 이동/교대·작업 시작 거부, 명시 중지 허용. 다른 source lock은 종료 후에도 유지 |
| A11-04 | 일반 피해/Silence는 유지, Hard CC/interrupt/철수/사망/terminal/retreat는 자기 channel·aura 정리. Pause 중 duration 보존 |
| A11-05 | Hello 자연 만료는 Goodbye 없이 복귀, 재입력은 남은 시간 기반 Goodbye 한 번·추가 공명 소비 없음 |
| A11-06 | timeout과 재입력 동시·중복 입력·stale activation·후속 실행 오류의 정책/rollback. 효과가 반쯤 제거된 상태 없음 |
| A11-07 | overlay 기본 공격의 관통·범위·방향과 무기 기본 출처/흡혈, Goodbye skill 출처 구분. 다른 호환 무기도 특례 없이 실행 |
| A11-08 | 다음 공격1회·windup 중 overlay 종료·발사 후 종료에서 정확 적용/소비. 기본 공격 주기 reset·이중 발사 없음 |
| A11-09 | ledger/member/lease의 admission·clone·반복 생성/종료·retreat rollback 및 공개 RON/조회/fingerprint 연결 |

## 6. R12 — 보호막·방어 중재·제어·HP 비용

### 현재와 전환 지점

| 파일·심볼 | 현재 사실·주의 |
| --- | --- |
| `battle/damage.rs` — `DamageResult`, damage calculation | raw/final damage·component breakdown을 유지할 계산 계층 |
| `core/damage_runtime.rs` — `execute_committed_damage_inner` | basic OnAttack/OnHit plan 수집→calculate_damage→prepare_damage_fact→HP 적용. 보호막·패링 도입 시 기존 계산 뒤 HP 숫자만 바꾸면 fact/공명/trigger가 틀어짐 |
| 같은 파일 — `apply_damage_result_and_record` | lethal death를 준비한 뒤 HP·damage fact·관측·공명·death를 처리. 최종 HP loss가 확정되기 전에 lethal을 정하면 안 됨 |
| `resource/mod.rs`, `store.rs` — ResourceOwner/ResourceLifetime/ResourceStore | Employee/EquipmentCopy/FragmentCopy owner와 Run/Attempt/Deployment/Activation/Delivery/Chain. 일반 effect pool owner는 아직 없음 |
| `core/types.rs::UnitControlState`, `battle/buffs.rs::BuffControlCapability` | hard CC와 Silence 중심. slow/root/taunt/pull을 모두 Stun으로 낮추면 행동 의미를 잃음 |
| `employee.rs::apply_incapacitation` | 런의 부상 기반 사망 방지이며 전투 HP1/패링과 별개. 전투 피해에 그대로 사용하지 않음 |

### 구현 방향

- 보호막은 `(source effect instance, activation)`에 속하는 단일 pool이며 대상은 membership 참조만 가진다.
  개인 보호막도 같은 중재 구조의 수혜자1명 사례로 표현한다. 수혜자마다 shared 잔량을 복제하지 않는다.
  기존 resource transaction/lease/계량을 재사용하되 effect pool을 가짜 직원 UUID로 저장하지 않는다.
  source owner 추가 또는 전투 전용 pool adapter 중 실제 수명에 맞는 최소 확장을 선택해 validation/clone/query를 닫는다.
- initial amount 계산·초기량 modifier는 source 생성 때 한 번만 적용한다. 새 수혜자 추가·재진입·조회로 재적용하지 않는다.
  R06 ValueExpr의 caster snapshot과 실제 사본 출처를 연결한다. 이미 확정한 잔량과 capacity를 UI가 별도로 계산하지 않는다.
- 처리 단계는 typed eligibility/패링·무적 판정, 기존 감쇄, 흡수, HP·생존 중재, 최종 receipt/죽음/후속 효과로 명시한다.
  선택한 정확 우선순위는 §7에서 닫는다. **감쇄 후 초과분에 방어력/저항을 재적용하지 않는다.**
  parry가 피해와 부착 효과를 막아야 한다면 현재 trigger/effect batch 생성·실행도 중재 결과를 소비해야 한다.
- receipt에는 calculated/mitigated/absorbed/actual HP loss/prevented/death를 구분해 보존한다.
  R06 actual HP-loss gate와 흡혈·공명에 absorbed 값을 더하지 않는다. 기존 OnHit 발동 의미를 바꿀 필요가 있으면
  R04 의미 영향과 정책 변경을 기록한다. 계산 설명의 amount를 final damage라고 표시하지 않는다.
- 일반 HP 회복과 shield recharge는 다른 operation이다. HP 비용은 별도 cost로 처리하며 shield·parry로 대납하지 않는다.
  비용으로 사망 가능 여부·HP1과의 관계는 authored policy/검증으로 드러내고 negative Heal로 위장하지 않는다.
- pool 만료·소진·교체·source 철수/사망·terminal은 exactly once 종료다. 만료 wake에 generation을 넣어
  같은 이름의 새 pool을 제거하지 않는다. 여러 피격·pool의 소비 순서는 stable하게 고정한다.
- 흑조형 parry fixture는 첫 eligible 직접 적 공격을 무효화하고 counter, 짧은 무적, miss 시 자기 감면만 일시 해제까지 검증한다.
  반사/받아치기/자해/HP 비용은 반사 입력으로 재사용하지 않는다. 이것은 R16 범용 유물 연쇄 전체를 구현하라는 뜻이 아니다.
  상시 효과가 필요한 fixture는 사본 장착/배치 source와 연결한다. active1 슬롯 정책에 대한 흑조의 개별 예외를
  P2G에 명시하며 모든 파편이 passive 장비가 되도록 일반 규칙을 바꾸지 않는다.
- timed slow/root/taunt/pull 등 R12/COV가 요구하는 제어를 실제 행동·movement/block 시스템에 연결한다.
  각 제어마다 이동/기본 공격/skill 시작/진행 중 cast/저지/target/면역/겹침/종료 표를 작성한다.
  플레이어 직원의 자유 이동 AI를 추가하지 않는다. 밀기/당기기의 적용 actor와 타일/점유/경계 검증을 명시하고
  승인되지 않은 직원 이동 의미는 자동으로 열지 않는다. 미지원 actor 조합은 진단하되 필수 적 제어 기능 자체를 누락하지 않는다.
- R09 침식은 ordinary protection과 별도 특수 kill을 유지한다. shield나 HP1이 침식을 일반 damage로 되돌리지 않는다.

| 조건 | 필수 입력과 기대 결과 |
| --- | --- |
| A12-01 | shared pool600, 감쇄 후 A400/B350: A HP 손실0, B150, 잔량0. 신규 수혜자·이탈/재진입은 새 잔량을 만들지 않음 |
| A12-02 | 개인+공유·복수 source·광역 같은 시각 피격의 결정적 순서. 대상별 실제 감쇄는 한 번, 피해가 다른 직원 HP로 이전되지 않음 |
| A12-03 | source 초기량 증폭 한 번·재시전·expiry/소진 동시·stale expiry·source 철수. 종료/발동 기록 exactly once |
| A12-04 | 일반 Heal/Platform Rest는 shield 불변, recharge만 잔량 변경. HP 비용은 shield를 우회하고 별도 CostFact, 부족/사망 허용 경계 |
| A12-05 | 비치명/치명·HP1·death-prevent·cost·특수 kill 조합: 정책 우선순위, 실제 HP loss/죽음 fact/trigger/결과가 모두 일치 |
| A12-06 | parry 성공 첫 eligible 타격의 damage와 부착 효과 억제, counter 한 번; 후속 무적은 추가 counter/반사 없음 |
| A12-07 | parry miss는 자기 감면만 잠시 해제, 장비 방어/다른 source 유지. 반사·자해·cost 재귀 없음, 공격자 소실·다단 경계 |
| A12-08 | slow/root/taunt/pull 및 기존 Stun/Freeze/Silence의 capability별 표를 실제 이동·저지·시전·target 변화로 검증. 만료 뒤 자기 source만 해제 |
| A12-09 | shield로 HP loss0인 타격의 R06 gate·공명·흡혈·조회 oracle. absorbed와 actual HP loss 구분, R09 suppression 보존 |
| A12-10 | pool/member/receipt admission 실패·중간 fault에서 HP/잔량/사망/queue/관측 모두 rollback, 정상 retry 동일 |

## 7. 정책 결정표

아래는 **작성자의 권장안**이며 사용자 직접 확정 답변이 아니다. 구현자는 기존 최신 계약을 먼저 찾고,
부재할 때 상위 지시서의 위임과 P2G-10 절차로 채택/수정 근거를 남긴다. 시험 수치는 공식 밸런스가 아니다.

| ID | 결정할 문제 | 권장 출발점·보존 조건 |
| --- | --- | --- |
| P01 | 침식 실제 start | manual은 접수 후 첫 unpaused 실행 gate, auto는 같은 의미의 실제 start gate. 준비 종료/release와 구분하고 조건은 그 gate에서 한 번 확정. 기존 일반 시전 접수·비용 계약 차이는 반드시 기록 |
| P02 | 공명과 준비 취소 | 공명 만충 검사에 필요한 값을 소비 전에 판정. 일반/침식 양쪽 비용을 중복 처리하지 않음. 준비 중 취소의 기존 비용 정책 보존, 새 환불 규칙 금지 |
| P03 | trauma/연출 수치 | 임계는 유효 범위의 명시 데이터, 연출은 유한 양수 duration. 시험 예시 trauma80/duration1000ms는 test-only. 공식값이 필요하면 위임 잠정값으로 기록 |
| P04 | Breakdown 동시 회복/완료 | 해당 완료의 회복을 먼저 적용하고 여전히100인 기존 Breakdown만 차감. 새로100이 된 node는 유예 카운터를 즉시 소모하지 않는 안. limit는 기존 의도2와 실제 live 계약 대조 |
| P05 | 강제 정지 만료의 초과 elapsed | 연출 해제 호출에서는 남은 delta를 battle catch-up에 쓰지 않고 다음 호출부터 재개하는 안. disconnect/manual Pause는 별도 유지. 클라이언트 완료보고 불필요 |
| P06 | 보스 task 취소/timeout | 기다림 종료와 action 취소를 구분해 작성. state-owned 미방출은 취소, 이미 release된 독립 payload 보존. timeout이 action 취소를 뜻하는지 task schema에 명시 |
| P07 | Flow 동시/결과/반복 | stable authored 순서·명시 milestone·parallel All join을 최소 정상 모델로. 실패 시 미방출 sibling 중단/방출분 보존 안. no-target 반복은 기회 소비. 임의 시간+1로 충돌 회피 금지 |
| P08 | channel 저지·aura 중첩 | 태아는 기존 저지를 유지하는 안을 우선 검토. 같은 aura family는 비중첩, 서로 다른 강도는 명시 우선순위/강도 정책으로 선택. 무관한 source lock은 유지 |
| P09 | Hello 재입력과 만료 | 유효구간 `[start,end)`; end와 같은 시각 재입력은 종료된 activation으로 거부. 성공 재입력 commit 시 남은 시간 고정, typed 단위로 계산. 정확 소비와 rollback을 함께 보장 |
| P10 | 개인/공유 보호막 소비 | 개인 먼저→공유, 같은 분류는 작성 priority→생성 ordinal의 안정 순서를 추천. UUID/HashMap 순회에 gameplay를 맡기지 않음 |
| P11 | HP1·HP 비용·특수 kill | HP1은 ordinary damage에 적용. 기본 HP cost는 지불 뒤 HP≥1인 경우만 허용, lethal cost는 명시 opt-in. cost는 shield/패링 우회, 침식 특수 kill은 ordinary 방어 우회 |
| P12 | 패링/부착 효과·반사 | 첫 eligible 직접 적 strike 단위로 damage와 그 strike에 결속된 부착 효과를 억제. unrelated 후속 self effect를 지우지 않음. 사라진 공격자 counter는 명시 no-target, 새 자동 재타겟 없음 |
| P13 | effect/pool 수명 | attempt 밖으로 전투 pool/overlay/channel을 저장하지 않음. 사본 개화/소유와 Breakdown 카운터는 각 durable owner. 기존 전투 중 process-save 미지원 정책 보존 |
| P14 | 저장 전환 | 현재 R07 V8·명시 V7 변환을 인수. 새 필드 필요 시 schema 검증/정상 restore/구형 처리 정책을 기록. 원본 파일 변경·손실·묵시 default 성공 금지 |

상충하는 선행 계약을 찾으면 추천안을 우선하지 않는다. 더 큰 정책 변경이 필요한 경우 exact 영향과 해결 가능 범위를 기록한다.
이 표는 부족한 질문을 조용히0·무한·항상 성공으로 채우는 대신 구현 전에 결정하도록 하는 체크리스트다.

## 8. 전체 교차 수용표

각 조건을 `COVERAGE.json`의 actual test selector·실행 receipt·코드 위치와 연결한다. 여러 조건이 같은 종단 시험을 공유할 수 있다.
아래 숫자는 시험 개수가 아니라 수용 조건 ID다. 표만 채우거나 private state에 결과를 주입해서 통과시키지 않는다.

| 조건 | 최종 source에서 필요한 증거 |
| --- | --- |
| X01 | 공개 외부 팩의 보스가 Flow 스킬 요청→projectile/area 실제 피해→Released/Settled 전이. direction/target/오류 route·wave 진행을 함께 관찰 |
| X02 | 정상 직원 지급·R07 exact 파편 장착→Hello 또는 channel→실제 기본 공격·명시 재입력/중지→종료. copy/form/source와 R06 amount/trace 일치 |
| X03 | 2명 이상 배치·개인/공유 shield·적 공격·HP/cost/parry·잔량/종료를 GameCore 정상 명령/시간 진행으로 검증 |
| X04 | 침식이 기존 보스·새 Flow·channel/pool 존재 중 발생: 필수 death/wave 유지·선택 효과 억제·대가·host freeze·결과 확정 |
| X05 | 결과 확정→node 완료/회복/Breakdown→SavePoint→새 Core restore. R07 identity·개화·장착/역결속·typed resource와 새 durable 상태 보존 |
| X06 | 실제 Retreat는 진입 전 상태·RNG·자원/카운터를 복원. 재시도/같은 요청 중복/오류 후 retry에 자산/효과 복제 없음 |
| X07 | trace/관측 on/off, preview/explain 반복이 gameplay·amount·revision·RNG·완료 순서를 바꾸지 않음. 보관되지 않은 과거 평가를 재구성해 진짜 기록처럼 반환하지 않음 |
| X08 | allocation/retained/work/output/event 한도의 exact/1-short/overflow, parser 깊이/중복/잘못된 ref, 임의 순회 순서 변화·stale capability·clone 격리 |
| X09 | 최종 기본/feature 전체 시험·check·release CLI·clippy·fmt와 PC36/R05 원자료. warning과 hard failure 구분, 20분 시험 제외 |
| X10 | 실제 before/after와 added/deleted membership·정역 복원·currentness. 모든 필수 A08~A12 조건 완료, 미정/미구현을 별도 명시 |

## 9. 구현자가 반드시 재대조할 부분

이 문서는 전체 코드 감사나 실제 실행 결과가 아니다. 소스 identity와 읽은 근거는 [조사 기록](R08_R12_CODE_EVIDENCE_20260930.json)에 있다.
지시서 작성자는 위 호출 경계의 본문을 읽었으나 다음 연결은 구현 직전 실제 consumer까지 추가 추적해야 한다.

- R08 catalog compile의 cross-database skill 참조 검사와 모든 boss transition admission 호출자.
- R09 world command/read-only 분류, terminal publication·disconnect·trial bypass, 모든 node 완료/회복 경로.
- R10 source map/effect address·fingerprint·descriptor·amount 관측의 전체 producer와 기존 step lowering 방식.
- R11 modifier mutation·stat 재계산과 현재 attack start/release snapshot, 이동·저지 lock 전체 consumer.
- R12 damage component/attached effect group의 실제 단위, 모든 HP 변경/비용/생존 경로와 field-work·movement 영향.
- 새 상태를 가진 DTO/native clone/retained/save loader/restore 검증. 검색 hit만으로 종단 연결 완료라고 기록하지 않기.

위 확인은 구현 지시를 처음부터 다시 발명하라는 뜻이 아니다. 구체적인 타입·함수·오류·상태 key와 실제 수정 파일을
PLAN의 상세 설계에 보완한 뒤 구현한다. 설계 추천과 다른 선택은 이유·의미 보존·시험 근거를 남긴다.
