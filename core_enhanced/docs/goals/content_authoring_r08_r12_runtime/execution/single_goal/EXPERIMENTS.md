# 실행 이력

1. 2026-09-30 선행 감사 별도 종료. 두 Complete/High와 source367, fresh focused56/CLI6 및 OBS001을 인수했다.
2. P2G 작성: Parallelization 문자열을 설명으로 쓴 첫 후보 거부. object로 바꾼 두 번째 후보도 거부. 실제 validator line110의 exact `none approved`로 교정. generation003/validation001/quality2 validation001 전부 exit0 passed. 계획 validation은 제품 시험 아님.
3. 제품 변경 전 실제 source367+roadmap archive와 dirty/index/untracked 원본 보존. 실행 wrapper를 새 디렉터리로 복사했고 동결된 이전 실행 출력에 쓰지 않는다.
4. GEX-00 인수 후 WP08 caller/retention/compile 경계 조사 중. 아직 제품 변경/신규 시험 없음.

5. WP08 첫 구현: raw/compiled StartAction, 결과 coverage/capability, skill reference, 공유 cast admission과 명시 공명 reset 정책, activation waiter/예약 해제 연결. r08_check_001 exit101: import 경로, Side 실제 이름, facing 필드, borrowed index 오류8개. 로그/실패 source archive 보존. 실제 심볼 확인 후 교정하고 deadline wake/target-loss cancellation 및 builder skill validation도 연결. r08_check_002로 재검증한다. 아직 A08 PASS 없음.

6. r08_check_002 exit0. 첫 공개 RON 전투 시험 r08_tests_001은 FactSummary 필드명 오류로 compile 실패(시험0개); damage_hp_lost로 교정. 세 실제 종단/거부 시험을 r08_tests_002로 재실행.

7. r08_tests_002: 0 passed/3 failed, fixture의 MinPathCost가 명시 Reachable을 요구. fixture 교정. 추가 소스 대조에서 resident-state Acquire lock은 다음 state로 승계되지 않으므로 실제 pattern-owned Acquire→StartAction fixture로 변경했다. 계약을 완화하지 않았다.

8. r08_tests_003 3/3 통과. r08_tests_004 4/5: 공개 projectile state-exit는50ms 취소/150ms 방출 생존 모두 통과. fault test의 실패 전후 exact snapshot은 같았으나 별도 새 Core의 Debug HashMap 순서 비교가 달라 retry oracle 실패. retry는 UUID-keyed 정렬 unit 전체 상태와 boss snapshot/fact/event를 비교하도록 교정(실패 전후 exact 비교 유지).

9. r08_tests_005 5/5, r08_tests_006 7/7. 한 개 reservation cap에서5회 재진입·오래된 deadline 무효/회수, 두 번째 보스 action admission fault의 전체 timestamp rollback과 정상 retry를 증명했다. cargo fmt --all 실행(exit0).
10. r08_boss_regression_001 84 passed/1 failed. 기존 boss/omen/world 회귀는 통과; 추가 거부 시험의 fixture가 pattern target_range를 생략하여 runtime 이전에 올바르게 거부됐다. 명시 좁은 패턴을 작성하여 실제 runtime 자격 거부와 정상 Withdraw 명령 뒤 target 소실 route를 재검증한다.

11. r08_tests_007 7/8: fixture 교정 후 runtime target 소실 시험이 실제 결함을 검출했다. public Withdraw가 같은 시각 boss wake를 예약하지 않아 cast release가 target-loss poll보다 앞섰다. 현재 activation waiter에만 wake를 넣고 native work/event admission을 유지했다. Pause 중 명령 자체가 boss를 진행시키지 않고 다음 execution에서 처리한다. r08_tests_008 재검증.

12. r08_tests_008 8/8 통과. Withdraw wake 교정으로 target loss가 release 전에 명시 route로 처리된다. WP08 중간 수용 근거를 COVERAGE에 연결했다. 최종 source 재검증/전체 공개 팩 CLI 조건은 남으므로 최종 verified_count는0 유지. WP09 구현 준비로 계속한다.

13. WP09 start gate/host freeze/explicit death cause/attempt cost 접속. r09_check_001은 빠진 fixture 필드8개와 reason/cancel visibility 오류로 실패. 원본/로그 보존 후 수정; r09_check_002 all-target check exit0.
14. Breakdown 직원 counter·노드 완료/회복/자산 회수·엄격 V9+명시 V7/V8 변환 연결. r09_check_003은 readiness 지역 변수명과 presentation DTO exhaustive match 두 오류로 실패; 교정 후 실제 RON 침식 시험으로 재검증한다.

15. r09_tests_001 compile 실패: RON 함수 경로와 새 query error 소비자 누락. 교정 후 r09_tests_002 1/2: action release(0)가 fact 기록보다 먼저 settle되어 action 비용100이0으로 기록되는 실제 결함 검출. 대가/죽음 fact 이후 release로 이동. r09_tests_003 2/2 통과.
16. r09_tests_004 3/5: World fixture의 self_unit enum 오작성(공개 값 self), parser 정확 거부. r09_tests_005은 Auto Root cause tuple 오작성으로 compile 실패. 실제 schema에 맞춰 교정하고 stage별 cancellation/World 경로/V8-V9 변환 시험을 계속한다.

17. r09_tests_006 6/8: World self fixture의 global reach가 self와 양립 불가, 정확한 publication 거부. r09_tests_007 7/9: self-target hostile damage의 일반 유효성 거부를 확인하고 self Heal로 교정. r09_tests_008 8/9: 0.5배속 종단은 통과했으나 기본1배속 재설정의 AlreadyActive를 fixture가 성공으로 기대했다. 동일속도 재설정 생략(제품 계약 유지).
18. r09_tests_009 11/11 통과: pause접수/세 배속/disconnect/last-enemy terminal/결과/저장, mutation 경로 불변, Breakdown 회복우선/만료자산회수, V8변환/V9오류보존 및 fresh process counter 복원. r09_tests_010 13/13 통과: 일반 death OnDeath7+OnKill11+OnAllyDeath13 positive control과 침식 억제, 현재 적만 처치/500ms 미래 required wave 보존 추가. cargo fmt --all exit0. 다음 lib 통합 회귀는 중간 R09 source 검사이며 최종 통합 검증 아님.

19. r09_lib_regression_001 전체1545:1537 passed/8 failed. 5개 manual kernel fixture가 새 start gate를 처리하지 않고 첫 queue event를 end라고 가정하거나 t1 변화 전에 t0 gate를 실행하지 않았다. 원래 post-start 취소/Silence/acceptance tile 계약을 보존하도록 6개 관련 fixture의 gate를 실제 t0(태세 시험1500)에 처리한다. 저장 writer 소비자 두 시험 version8 기대는명시V9로 갱신. 1개 Unix socket acquisition fixture는 UnixListener::bind에서 EPERM(환경 제한), 제품 검증 도달 전 실패. 시험/제품을 skip-success로 수정하지 않는다. broad 원본 실패 보존.

20. r09_lib_regression_002 exit0:1544 passed/1 explicit filtered (환경 EPERM Unix socket fixture). 새 gate를 처리한 post-start/Silence/취소/target snapshot 회귀와 writer V9 소비자가 통과했다. 전체 broad PASS로 격상하지 않는다. WP10_DETAIL.ko.md를 작성하고 structured flow 구현으로 이어간다.

21. r10_check_001 실패: GameError Display 미구현을 오류 문자열 변환4곳에서 잘못 가정. Debug로 교정. r10_check_002 실패: 새 event arm의 Result 반환 누락과 cleanup method visibility2곳. 교정하며 compiled cache의 delay currentness, resolved cast target 전달, flow state의 native retained/clone owner를 연결했다. 두 실패 source/log 보존. 아직 R10 수용 통과 없음.

22. r10_tests_001 R08 8개 통과(legacy lowering 회귀). r10_tests_002 fixture heal field 오작성,003 새 출처 초기화2곳/사망 취소 enum 누락,004 새 출처를 mutable replace 인수 안에서 읽는 borrow 오류2곳으로 각각 compile 실패. source/log 보존. 필드를 먼저 capture하고 사망 사유 CasterDied를 명시했다. structured Flow의 R06 CurrentActionPrefix 선행 판정은 steps 벡터 순서에서 compiled EffectsSettled predecessor로 변경하고 compiled_graph 공개 조회에 Skill을 연결했다.

23. r10_tests_005 compile 실패: LiveRonError의 generic Option에 .into()를 중복 사용. 정확한 &str로 교정. r10_tests_006 3/3 통과. 선행 scope는 전체 index 목록 대신 step당 boolean으로 보관해 불필요한 제곱 크기 표를 제거했다.
24. r10_tests_007 5/6: CastTarget은 활성 대상의 진영을 다시 필터하지 않는 기존 계약이므로 private owner 변경을 무대상으로 기대한 fixture가 잘못됐다. 테스트를 public Withdraw→빈 회차→250ms scenario SpawnGroup + RetargetOnStep으로 교정. r10_tests_008 7/7: 실제 광역3명300피해→self Heal15 한 번, interval0/100/200의3회, 빈 회차 소모, Parallel wait/join, release 뒤 Interrupt/투사체 보존, stale wake, 4개 fault 지점 timestamp rollback과 retry 통과. 아직 depth/큰 반복 예산·CLI·최종 통합 증거 미완료.

25. r10_lib_regression_001 compile 실패: DataValidationError generic 인수의 불필요한 .into() 교정. r10_lib_regression_002는1552개 시험 시작 뒤 daemon restart로 중단, 최종 요약/exit receipt 없음. 부분 로그의 실패5건을 보존하고 recovery.json에 미완료로 기록했다. 보존 tar와 복구 현재 source 전체가 정확히 일치함을 확인했다. 동일 source를 새 이름003으로 재실행한다.

26. r10_lib_regression_003 완료:1547 passed/5 failed/1환경 제외. 실제 결함은 이미 검증한 Skill index 준비에서 charged flow compile을 expect로 반복하여 budget refusal이 panic으로 변환된 것(2시험). index의 기존 구조 검증과 fallible public/raw/amount compile 경계를 분리했다. 신규 flow admission을 skill copy 앞에 넣어 기존 first-fault site가 바뀐 경로도 source copy 뒤/identity 할당 전으로 복구. 나머지2시험은 Skill compiled_graph 미지원의 옛 기대와 시작 후 native 한도 변경(정상 InvalidAction)을 교정했다. 원본 실패와 source는 보존.

27. r10_lib_regression_004:1552 passed/0 failed/1환경 제외. 제곱 크기 선행표를 없애고 종료 repeat occurrence를 회수하며 마지막 impact anchor 하나를 유지했다. amount preview/explain에 실제 flow occurrence를 노출하고 Skill compiled_graph/정적 effect flow 설명 및 capability를 연결했다.
28. r10_tests_009 9/10: 일반 trial은 SavePoint를 방문하지 않아 export checkpoint None인 것이 정상인데 존재를 기대한 fixture 오류. 공개 amount/CLI oracle와 실제 World 결과→현재 checkpoint roundtrip oracle를 분리했다. r10_tests_010 10/11: 정상 배치/스킬 cost2건을 비용0으로 기대한 오류를 before+1(스킬 공명 비용)로 교정. r10_tests_011 10/11: 침식용 stationary 전투 fixture에 자기 Heal만 넣으면 적이 사라지지 않아 결과가 생기지 않음. public RON에 별도 적 광역 finisher leaf를 작성하여 실제 죽음/결과로 이어지게 교정한다. 사망/결과를 fixture 내부에서 주입하지 않는다.

29. r10_tests_012 11/11 통과. fmt 적용 뒤 r10_release_001 release bins exit0, Snapshot/Live 각각 trace off/on CLI4회 exit0. stdout 전체 byte equality는 evidence/r10_cli_parity_001.json에 보존. WP10 중간 기능 구현 완료, 최종55조건 수용 판정은 미완료. WP11 source별 지속 효과 구현 착수.

30. CLI stdout byte equality는 false: 각 전투 usage.metadata_bytes_peak 두 값만 trace 저장 때문에 증가했다. 다른 모든 JSON 필드(게임 진행·결과·amount query)는 exact equality. 원본001 false 보존, 구조 대조002에 제외한 실제 peak 값과 일치 결과 기록.

31. WP11 Sustained::Modifiers producer와 battle-local source record, flat/percent 합성, 기존 영구 ApplyModifier의 base 변경, finite delivery 소유/만료/terminal/철수, clone/native retained/entity 경계 연결. r11_check_001 package명 오작성(제품 compile 전 거부),002 StatModifier CloneShape 누락, r11_tests_001 fixture 인수 순서 오류를 각각 원본 보존 후 교정. r11_tests_002 3/3 통과: 공개 RON 두 source225→영구 변경240→A종료143→B종료110,10회 수명 drift 없음,3개 fault site 전체 상태·queue 복원/retry.
32. ChannelAura의 정확 직원 membership, family 비중첩, 기본 공격 전용 damage/speed, source별 movement/basic/cast/stance/tactical/work lease, reinput 중지 연결 중. check003은 members 초기값/ActionState import2오류; tests003은 constructor 치환이 CloneShape macro에도 들어간 오류로 compile 실패. 교정하고 tests004 진행. 전체 A11 수용 완료 아님.

33. r11_tests_004 3/5: aura fixture의 필수 delay_ms 누락. 005 3/5: Employee source에 실제 무기 binding이 없어 WeaponRequired가 정상 거부. fixture에 실제 EquipmentMetadata/무기 binding을 넣고 데이터 radius 단위1e6을 교정. 006은 EquipmentType import 경로 오류; 007 5/5 통과. 실제 Employee source, 자기 제외, skill Attack 보존/basic120 vsAbility100, 속도점수15, 중지 추가공명 없음, 무수혜 시작 거부·withdraw membership 회수 확인.
34. source/activation/member/authoring/fragment 공개 sustained_effects DTO와 typed 종료 reason/count, query/native allocation 및 fallible World projection 연결. check004 all-target exit0. tests008은 checkpoint byte배열을 문자열로 오인한 fixture compile 오류; 009 7/7 통과. normal World aura→Pause수명 보존→실제 finisher→결과→V9 save/freshCore 복원, 공개 조회 출처 확인. 일반 damage/Silence 유지, HardCC/interrupt는 channel만 종료하며 별도 movement lock9000 보존.
35. R10 EffectsSettled를 sustained finite-child 종료에 연결했다. source가 보유하는 structured occurrence에 pending delivery를 등록하고 마지막 종료 후 continuation을 깨운다. legacy 동작 유지. expiry/activation/end count overflow를 native CounterOverflow로 변환; direct tactical relocation/swap과 death-before-trigger 경계 membership도 연결. r11_tests_010은 r1 필터로 R10/R11 함께 검증 중. Hello/overlay 및 R12는 아직 미구현.

36. r11_tests_010 R10/R11 19/19 통과. ChannelAura의 Instant SelfUnit/EffectsSettled publication 계약 후 fmt, r11_lib_regression_001 1563 passed/0 failed/1환경 socket 제외. R11 전체 완료는 아니며 Hello/overlay 구현을 계속한다.
37. BasicAttackOverlay 공개 schema, source별 원본 profile 보존/복원, 신규 instant tile area 및 실제 directional projectile basic adapter, 다음 release 소비를 연결. r11_check_005는 기존 test ProjectileLaunch 3개 필드 누락으로 compile 실패; r11_tests_011은 새 fixture의 RuntimeUnit position 필드 오작성으로 compile 실패. 원본 source/log 보존 후 교정.
38. r11_tests_012 9/11: 새 area/관통 시험2개가 기본 무기 interval1500을1000으로 잘못 가정하여 overlay 종료 전에 release하지 못했다. fixture 무기의 interval1000/windup1을 명시하고 다음 기본 공격 시각을 현재 scheduler로 검증한다. r11_tests_013으로 재실행 중. 아직 overlay/Hello 수용 판정 없음.
39. r11_tests_013 2/3: 실제 directional release→overlay expiry 뒤 owner 보존→2명 관통은 통과. area fixture가 기본 include_anchor_tile=false로 조준 대상 타일을 제외했으므로 명시 true로 교정했다. BasicAttackOverlay의 기본 무기 profile/주기는 유지한다.
40. typed ReinputSustainedSkill 정상 World/Battle 명령, current activation 검사, fixed RemainingTime context, 별도 Goodbye skill 출처/파편 provider 전달, 단일 immediate child의 원자적 실행 연결. r11_check_006 compile 실패는 기존 명시 RuntimeFactSource constructor 한 곳의 reinput 필드 누락. 교정. r11_tests_014 11/13: Goodbye footprint가 facing 회전 후 가로폭7이 되어 먼 적을 포함하지 못한 fixture 오류. 15x15 명시 정사각형으로 교정했다.
41. r11_tests_015 13/15: actual child DivideByZero 및3 fault site rollback/retry 통과. 두 실패는 helper가 첫 skill만 복사해 Goodbye를 누락한 것과 amount observation key를 bind하지 않은 것. helper에서 공개 catalog 전체를 전달하고 battle key를 명시했다. r11_tests_016 15/15 통과: Snapshot/Live remaining1500ms→150HP, 공명 추가비용 없음, duplicate/stale/time-equality, actual error 원자 복원, normal World exact fragment 출처→Goodbye900ms→결과/save/fresh restore까지 검증.
42. channel 중 triggered windup release/triggered 새 시전 경계도 차단하고, directional 영거리 aim은 launch facing으로 정의했다. windup 중 overlay 만료와 새 activation에 대한 과거 입력 거부 두 시험을 추가하여 r11_tests_017 실행 중. R11 final acceptance 아님.

## EXP-43 — R11 membership/source 중첩과 정상 배치·철수

- 기존 회귀 `r11_lib_regression_002`: 1572 PASS, source `81146a83bd44b7a9eec1682d610ebdcd787a39850f734475ff084797d18fa5bf`.
- 추가 집중 `r11_tests_018`~`r11_tests_023`: 최종23 PASS, source `60901b6435c16b4217082bb944873ff0bc131116d950677d97f37671434664f6`.
- 다중 aura의 priority/tie/family 및 하나만 제거, 신규 배치·재배치·Guest/summon/object 제외, 실제 전술 이동3회 이탈/복귀, membership fault와 World retreat rollback/reentry를 추가했다.
- 실패는 추가 fixture의 RON 연결/식별자/actor 경로/좌표 타입/tactical 초기화·cooldown 문제였다. 원본 raw log와 exact source를 유지한다. final A11 판정과 최신 broad/CLI는 아직 미완료.


44. R12 보호·생존·비용·제어 구현 및 중간 검증.
- `r12_check_001`~`007`, `r12_tests_001`~`010`의 각 exact source/archive/log/exit는 evidence에 보존한다. 테스트005의 3개 실패는 parallel legacy step의 절대 delay/이미 대기 중인 t0 gate를 fixture가 잘못 가정한 문제였다. 006 6/6 통과.
- 007의 Pull 좌표 실패는 시작 body가 이동 tick으로 cell center에서 벗어났는데 연속 거리 2를 기대한 fixture 오류였다. 셀 이동 endpoint로 교정한 008 9/9 통과.
- check005의 새 source 필드/constructor 오류는 수정 후006 compile 통과. tests009 12/13에서 miss fixture의 후속 단계가 counter skill로 삽입되어 publication이 거부했다. 보조 함수의 skill 경계를 교정한010 13/13 통과.
- `r12_check_007`는 release-owned BasicLifesteal과 실제 instant/area/projectile settlement 연결의 lib compile 통과이며 흡혈 수용 판정은 아직 아니다. 기존 실패 기록을 덮어쓰지 않았다.


45. BasicLifesteal 실제 경로 집중 검증.
- tests011은 RON parser namespace 오작성으로 compile 실패. 012는 whitespace를 고정한 test-only step 삽입 helper 때문에 신규 3 fixture 구성에서 실패했다. helper를 quote-aware bracket 경계로 변경한013은 15/16 통과, 마지막 fault 시험의 고정 1001ms가 실제 instant 공격 시각과 맞지 않았다.
- 같은 fixture의 정상 실행에서 첫 HealFact 시각을 얻어 그 timestamp만 fault/retry한014는 16/16 통과(`2241a44f34746b657539d2803f7801cf5f5502af76aeafeb198f8f897e671b5e`). actual HP 3+3의25%=1, 100+100의25%=50에 cap30 한 번, projectile 종료 후 cap35, Goodbye 흡혈 배제와 healing 후 rollback을 확인했다.
- 다음 `r12_lib_regression_001`은 이 중간 source의 전체 lib 회귀다. 기존 Unix socket EPERM fixture 1개만 명시 제외하며 전체 환경 PASS로 표시하지 않는다. Black-swan passive/최종 수용표/통합 검증은 남아 있다.

46. `r12_lib_regression_001` 완료: 1594 passed/0 failed/1 명시 환경 제외, exit0. exact source는 tests014와 동일 `2241a44f34746b657539d2803f7801cf5f5502af76aeafeb198f8f897e671b5e`다. 새 보호/패링/흡혈의 기존 library 회귀를 확인했으며 최종 통합 완료 판정은 아니다.

47. 흑조 예외 metadata/runtime/typed receipt/query 구현. `r12_cargo_check_008` compile PASS; `r12_focused_tests_015` 21 PASS, source bf503d40a8510f82e7da87fea9382e88ab70f90b0c343965a60e1806eb5f761b. Reflection observation 출처 분리와 counter invariant 실패를 추가 보강했다.
48. `r12_focused_tests_016` 22/25: 새 테스트 RON 괄호 누락, 금지된 self damage의 public admission 기대 오류, 복수 적 fixture의 HashMap 첫 적 선택 오류. 교정한017은26/27이며 recast fixture가 기본 공명 gain lock을 누락했다. 테스트 profile의 gain_lock_ms0과 authored ModifyResonance로 실제 재시전하도록 교정했다.
49. `r12_focused_tests_018` 28/29: 공유 잔량 신규 배치/철수/재배치 불변, 초기 source amount 고정, recast 새 capacity, 과거 expiry, 같은 시각 광역 stable order, R06 actual HP result와 hard native rollback PASS. OnHit 시험은 가까운 적 fixture의 획득 profile이 유효하지 않아 피격이 없었다. 이미 검증된 whole-field ranged enemy profile을 사용하도록 교정했다. 원본 실패/source/로그는 유지한다.

50. R12 집중019~022 및 실제 World 보호 통합.
- 019의 신규 두 fixture 실패(시작 직전의 기본 공격 OnHit를 패링 이전에 적용한 점, Finished 뒤 정상 finalize 경계 누락)를 교정한020은31/31 PASS, source `cb6459476bbf3445752c6cc24676a4c8d657f91e3d81079a98af138975d1491a`.
- 021/022는 shield의 actual HP loss 99/0→흡혈24/0 및 공명19/10 oracle를 통과했다. World 신규 fixture만 실패했으며 원본을 보존한다.
- World probe001~005는 test-only target/range/window 설정을 교정하는 증거다. Self cast + global은 publication이 거부했고, 허용된 TileArea 도발로 실제 적의 target을 변경했다. 도발이 기존 windup을 취소하므로 실제 첫 타격3000ms에 맞춰 유효 window3200ms/duration4500ms를 저작했다. 이전 로그의 적 미피격을 production 실패로 오판하지 않는다.
- `r12_world_probe_006` PASS, source `5d46769e191e75b592efd760884d59351f5d82acc19036b7778cec46e36d9bf1`: 정상 Grant/Equip/Deploy 2명, shared600+personal100, HP cost10, 적 기본 공격→counter1→보호막 소비, finite flow 종료→CombatResult→Complete→checkpoint→새 Core restore와 exact fragment identity를 검증했다.
- `r12_public_probe_001`은 시험 팩 생성에 generic RON serialization을 사용해 i128 unsupported로 실패했다. live loader와 런타임 실행 전의 fixture 오류다. 공식 원문에 별도 authored definitions를 추가하는 방식으로 교정한다. CLI/최종 수용은 아직 대기다.

51. 공개 팩과 최종 cross-WP 집중 검증.
- `r12_public_probe_002`는 Hello를 일반 ActivateSkill로 다시 요청하여 정확히 resonance_not_full 거부를 받았다. public checkpoint의 actual activation을 binding하는 ReinputSustainedSkill로 교정한003 PASS(`0888285208acb04556456e53e42d495caff4f17b8f61e14b6c6c12dd9612b6d2`). 이후 amount observation off를 포함한 public result 동등성, exact site/actual fragment 확인도 추가했다.
- channel 작업 시험001 compile namespace 오류,002 occupied position,003 FieldWorking 거부와 integrated001 DefenseStance/002 charge 미설정은 전부 신규 fixture 오류다. 실제 normal deploy의 starting resonance를 scenario profile에 설정하고, 작업 칸 밖 Assault에서 channel→진입 거부→중지→Defense 이동→실제 작업 기여로 교정했다. production admission 완화 없음.
- 보스 공개 팩 첫 시도는 marker task failure route가 불완전하여 publication 거부. exhaustive5개 route를 저작했다. 이후 긴8000ms 단일 clock은 native work10,000,000 cap에서 정확히 거부했다. 기본 cap/측정 workload를 변경하지 않고 이 기능 fixture의 정상 host tick을500ms로 나눴다. PC36/R05 측정 입력과 별개다.
- World 통합 보스 fixture의 entry 변경으로 남은 unreachable state는 compile 거부되어 그 미사용 state를 제거했다.
- `r12_integrated_probe_003` 72/72 PASS(`d24404796d943c67edebd352386af4ebde3597aabbbc510441644258899f43e3`): 공개 외부 보스 Flow projectile→area, Released/EffectsSettled 후 marker action의 실제 시각 차이, World boss guard+Flow+channel+shared pool 공존 중 침식→보호 무시 mandatory death→host freeze→결과/save/fresh Core의 Breakdown/actual copy 보존을 포함한다.
- `r11_final_weapon_001` PASS(`4f4eb618b7c117a256feb41e41fdfbac86d5c14d27d56d7e305fabc8d19378d9`): 서로 다른 실제 장비 UUID3개, Physical/Magic/True, overlay 원복/주기/actual HP100→흡혈25/source 장비 identity 일치. 최종 fmt 후 INT 전체 행렬을 시작했다.

52. 첫 최종 전체 행렬(001), source `5aa876932df24508e23fb81f2327a7c5ceb8c0021ead33b7f17936bccd337ebd`.
- default 및 test-fixtures 원명령은 library1617 PASS/Unix socket fixture1 EPERM 실패. 해당 환경 시험만 명시 제외한 별도 all-target 실행은 각각1674 PASS/0 FAIL. 원명령 실패를 전체 PASS로 치환하지 않는다.
- check/event_log_checks/release/clippy/fmt PASS. Clippy에는 기존 대량 경고가 있고 새 allow를 추가하지 않았다. 공개 R12 export3개 selector PASS,4팩 생성.
- 수용표 대조에서 A11-02 released aura snapshot과 A11-03 실제 periodic basic 차단을 더 직접적으로 확인하기 위해2개 oracle를 보강한다. 기존001 기록을 보존하고 최종002 행렬로 갱신한다. 제품 동작을 바꾸기 위한 재실행이 아니며 해당 경계의 증거를 보완한다.

53. A11의 직접 oracle 보강.
- `r11_final_oracles_001` 28/28 PASS, source `980f94763da3d5b7f774769ff8ab1f27ae32b27360360a31b6d7ab3ba3a83b70`.
- 동일 scenario 대조군의2500ms 내 실제 periodic AttackStart≥2에 대해 channel caster는0, 명시 중지2500ms 뒤 한 번만 재개하고 실제HP100 피해를 확인했다.
- 실제 ranged weapon projectile의1001ms release 당시 aura20%를 고정하고1101ms aura 종료 뒤 도착 시HP120을 확인했다. source 만료 후 aura 조회는0이다.
- cargo fmt 뒤 소스는 같고 최종002 전체 행렬을 수행 중이다. 이후 제품 변경이 없으면 이 source를 CLI/PC36/R05 및 byte 복원의 기준으로 사용한다.

54. 최종002 행렬 source980f9476: default/test-fixtures 원명령 각1619 PASS/Unix socket EPERM1 FAIL. 해당 정확 selector만 제외한 각각1676 PASS/0 FAIL. check/event/release/clippy/fmt/public export PASS.
55. PC36 final001은 기존31success/5refusal 대비30success/6refusal. 신규 ProtectionReceipt의 inline 선택 필드가 모든 일반 피해 fact를 키워12×500 burst가 RetainedByteLimit에 도달했다. 공개 일반 피해 JSON에 reaction=false/빈 protection을 추가하여 이전 fact/log hash도 달랐다. R05 final001은 P01 parity에서 exit101, raw receipt 출력 이전 실패이며 log/소스 보존. 실행 중 제품 변경 없음. 선택 보호 필드를 Box로 옮기되 별도 메모리 admission/retained/CloneShape 비용을 포함하고, 효과 없는 일반 피해의 기존 wire shape를 유지하도록 교정한다. PC/R05 workload와 cap/비교 기준은 그대로다. 최초 shell의 python 미설치 exit127도 cost_matrix_001.console.log에 보존하고 python3로 실제 실행했다.

56. 희소 보호 Box 교정 source3600cd6d에서 R12 집중36개 통과. PC final002는 일반 fact JSON의 의미 차이를 없앴지만 출처의 빈 flow/reinput inline 크기로12×500 burst가 여전히 RetainedByteLimit이다. 로그192행 차이는 version31→32 변경을 추적 중이다. source의 선택 context를 Arc 소유로 바꾸고 allocation 전 admission/retained를 추가한다. 집중002의3 compile 오류는 amount query DTO가 여전히 value type인 점과 테스트의 Option move이며 구현 비용 개선과 별도로 교정한다. 버전 비교는 옛 source의65개 원본 로그를 실제 원 workload에서 추출해 모든 원 hash를 확인하고 envelope의31→32만 바꾼 별도 baseline을 만든다. 원 cap·반복·workload·공식 runner를 바꾸지 않는다.

57. source d8fe7c9b에서 R10/R11/R12 집중75개 통과. PC final003은 버전 envelope 외 비교를 엄격히 유지하며 원 이벤트 전체 내용의 동일성을 확인했으나12×500 burst의 retained 한도는 아직 초과했다. 모든 일반 피해에 남는64-byte 보호 receipt를24-byte 값+희소 Box payload로 축소한다. 실제 보호 receipt allocation 전 admission 및 retained/CloneShape 계산을 함께 유지한다. 버전31의 원본65개 로그는 R07/P01 hash와 전부 일치했고, 내용이 같은 버전32 baseline에서 바뀐 필드는 두 workload의 log hash뿐이다. 원본과 파생 파일·실제 추출 소스·측정 로그를 보존했다.

58-pre. source bd89f9c4에서 집중75 PASS. PC final004는 모든 측정 뒤 wrapper의 이전 tmp final003출력 이름 잔존으로 create_new AlreadyExists가 발생했다. 기존 파일 덮어쓰기는 없었다. 실패를 보존하고 동일 source/동일 instrumented isolate를 재사용해 새 final005 출력으로 재실행한다. 제품 변경 없음.

58-cli. 최종 CLI001의 첫 R06 validate가 필수 RunPolicy.corrosion 누락을 ron_parse로 거부했다. R09의 명시 정책 계약에 따른 정상 진단이다. 과거 원본은 수정하지 않고3개 선행 팩을 별도 복사한 뒤 현행 공식 policy의 corrosion 한 줄만 추가했다. 모든 나머지 파일·scenario·skill·recipe가 동일함을 CLI_PREDECESSOR_MIGRATION_001에 기록하고 새 CLI002 명령 이름으로 전체45명령을 실행한다. 제품 source 변경 없음.

58-currentness. CLI002 45명령/9팩 모두 성공하고 trace off/on 결과·amount 동등성을 확인했다. currentness 첫 검사에서 명령57개는 최신 source와 일치했으나 검사 스크립트의 PC isolate locator가 과거final001을 가리켜 assert가 발생했다. 출력 currentness 파일 생성 전이며 최종final005 locator로 교정한다. 제품·측정 데이터 변경 없음.

58. 최종 source `bd89f9c42191217a333d648d35f495e802d13e6fb1420caa9c08da3deb731690`. 희소 저장 교정 후 집중75개와 PC final005 통과:576행/36조건/31success/5refusal, envelope 버전 외 모든 gameplay/fact/status/error-code 동일. R05 final005 concurrent peak9372846/cap9030188(+342658) 단독 경고, 나머지 hard guard0. 원 workload와 cap·공식 runner 유지. LOG_VERSION_MIGRATION_001의65개 원 byte/hash와 버전만 바꾼 baseline으로 schema 이행을 입증했다.
59. 최종003 default/feature는 환경 socket1개 명시 제외 후 각각1676 PASS. 원명령의1619 PASS/1 EPERM FAIL 각각 보존. check/event/release/clippy/fmt/export PASS. 최종 CLI9팩45명령 성공, trace gameplay/amount4문서 일치. 57 receipts currentness와 instrumented PC copy identity 검증 PASS.
60. 수용표55개:54self_verified+X09 environment_limited1, 독립 감사 미수행. R06/R07 영향표 갱신. 격리 정역 patch fuzz0 및 membership/bytes PASS. REVIEW_HANDOFF와 최종 기록을 seal하고 FINAL_TARGET 포인터로 인계한다. seal 후 별도 manifest/source/record hash 검사는 포인터에 연결한다.
