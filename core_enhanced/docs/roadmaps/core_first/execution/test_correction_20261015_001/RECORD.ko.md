# 실패 테스트 교정·중복 정리

2026-10-15 사용자 승인: 유지 가치가 있는 실패 시험은 교정하고, 무가치·중복 시험은 제거한다. CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md §3–7을 따른다. 기존 전체 검사 full_tests_20261015_001의 기본26실패·추가3실패와 공유 helper 영향이 범위다. 기존 dirty 변경과 과거 동결/실패 원문을 보존한다. 이 기록의 preimages는 이번 편집 직전 실제 파일이다.

## 교정 전 설계·분류

- 공통 Defense helper와 사용 시험21개 및 feature 전용2개: 재접속·권한·timer·prepared commit/drop·IO·세션 소유·유물/소비/독립개체를 각각 검사하므로 단순 중복으로 삭제하지 않는다. 현재 공식 Expedition 시작은 B1-01의 S(완료) → 2(early_reward 고정 단순선택)다. 기존 helper는 Available Combat/Start만 찾아 이 경로를 진행하지 않았다. 현재 공개 ExitReward로 미이용 종료(노드 소비, 보상0)를 명시적으로 수행하고 지도/미리보기 상태를 검사한 뒤 Defense 미리보기까지 도달한다. seed 전수탐색 대신 사전 확인한 고정 seed의 유한 경로를 사용한다. 준비 후 ConfirmEnterNode 전 상태를 유지한다. 공식 경로를 옛맵으로 대체하지 않는다.
- 저장 IO2시험: replace_active_run=true가 현재 폐기 계약이라 목표 IO 전에 거절된다. 정상 새 시작/현재 프로필 준비로 preintent 거절과 accept 이후 recovery를 각각 유지한다. 시작할 수 없는 가짜 기존active 저장을 제거하고 실제 요청 수락/후속복구를 검사한다.
- 기존active 교체시험: 현행 continue→abandon→start 계약과 저장원본 보호를 host 명령에서 검사하도록 교정한다. 원본은 실제 생성한 run/profile이며 임의 opaque JSON으로 대체하지 않는다.
- raw duplicate vault key 시험: 정상 저장/로드는 기존 persistence 회귀와 겹치고 핵심은 외부 파일의 중복키 변조다. 범위 밖·중복으로 삭제하며 과거 증거는 보존한다.
- corrupt_local_active_run_requires_explicit_reset: 파일을 직접 덮어쓴 손상만 준비하며 이번 지시의 외부 임의 변조 제외에 해당한다. 제거한다. 정상 IO 장애·journal recovery 시험은 유지한다.
- Core 관측 실패: 기존 정상 fault 회귀가 이미 사용하고 통과한 max_work3000을 동일 초기화/Elapsed60000 관측에 사용한다. entry가 성공하고 Elapsed가 실제 work_limit으로 실패하며 first operation=Elapsed인 것을 확인한다. 제품 한도 변경/실패 기대 완화는 없다.
- 계측기 EnterOrdinaryCombat: 현재 Expedition/loan_copy_ids와 초기 보상 종료를 지원하도록 측정 도구를 교정한다. 정상 전투·authority 거절·pause/resume·retreat·none/normal/detached/full의 gameplay 동일성을 유지한다.
- 과거U07 pack 재생시험: 고정 과거 evidence를 현행콘텐츠로 덮어쓰지 않는다. 개체 actor/trap/wall 동작은 현재 Core u07 public trial/world 회귀와 중복이며 관측 retained/identity는 기존 관측시험이 담당한다. 이 큰 중복시험을 제거하고 유일한 Script→이전 공개 응답 바인딩→실행→assert 경로는 기존 정상외부전투 계측시험에 합친다.

근거: game_rules/run-and-map.md의 완료 시작노드·공개 도달 경로, relic-reward-nodes.md의 미이용 퇴장, world.rs의 replace 거절, actor handler의 continue/abandon 가드, application/fault_tests.rs의 정상 fault profile3000. results-and-save.md 일부 과거 교체문구는 현재 U12 시작수명 계약/사용자 합의의 대체근거로 사용하지 않는다.

## 실행 순서와 중단 조건

작은 공통준비/IO 사전검사 후 기본 서버lib 영향 묶음, Core fault 관측과 서버 runtime-measurement 관련 전체 모듈·공유helper feature2를 검사한다. 의미 있는 입력변경이 있는 범위만 재실행하고 이전 Core2228/probe11 등 비영향 pass는 재사용한다. 첫 빌드 예상1–4분, 공통준비 시험 목표수초; 실행단계60초 초과 시 로그/CPU와 준비전제 진단, 준비전제 실패면 원문보존 후 교정한다. 입력은 실행 중 고정한다. 기본과 feature구성은 순차 실행한다. 전용ignored calibration/관리/child helpers는 기존과 같이 미실행으로 구분한다.


## 사전 관측

preflight001은 checkpoint adapter 변환 누락으로 compile 실패(시험0); 기존 StoredCheckpoint::from 변환으로 교정했다. preflight002는 공통 고정seed0 준비와 preintent 검사가 통과(2pass), IO후복구와 saved-start guard2fail이었다. 전투 준비는 초반보상 공개퇴장 후 정상 Defense preview에 도달하며 1000seed 반복을 제거했다.

IO후복구의 남은 실패는 Attach가 authoritative durable profile을 설치하는데 fixture가 그전에 Core에만 넣은 compendium을 유지한다고 가정한 것이다. 지식은 실제 저장소에 정상 publish하여 준비하고 recovery에서 보존을 검사한다. 실제 유효 active/profile 조합에서는 Start capability가 막히므로 replace=true 거절은 context_irrelevant다. 원본보호와 continue→abandon→start 성공을 함께 검사하며 거절만으로 PASS하지 않는다. 같은 entry/Elapsed 선행을 공유하는 host fault profile도 기존 Core fault 회귀3000을 사용하도록 맞추고 무시되던 profile publish 결과를 확인한다.


server001은90pass/6fail/22ignore, 시험20.91초다. 원래1000seed 장시간준비는 사라졌다. 노출된6개 중3개는 current Expedition ExitReward의 continue_encode 작업이 작은native한도보다 크며,3개는 이전fixture가 active와짝인 lifecycle profile을 저장/설치하지 않았다. 한도를 계속 올리거나 runtime profile을 도중에 바꾸지 않는다. 작은 entry/tick 한도검사만 현재 지원 Tutorial의 직접 전투 경로로 준비한다(현재 공식Expedition 경로시험21개를 옛맵으로 바꾼 것이 아님). 일반 helper와 수명/복구시험은 공식Expedition 유지. 실제 paired profile을 저장하고 fresh Continue에도 함께 설치한다. 후속은 이6개와 준비영향만 묶어 확인한다.


focused003은 4통과/2실패다. 실제 작업량 초과·새 owner Continue·준비 drop/실제 IO·자동 terminal 재시도가 통과했다. 남은 두 개는 서로 다른 플레이어가 하나의 authoritative profile을 공유한 fixture와, 새 lifecycle profile 검증보다 저장 revision 검증이 먼저 발생한다고 가정한 오래된 기대값이다.

플레이어 격리는 각 플레이어의 실제 profile을 각 저장소에 정상 publish하여 준비했다. A의 fault가 고정된 동안 B의 tick/명령/실제 save가 진행되고 A 종료가 B 저장을 바꾸지 않는 검증은 유지한다. stale actor는 A의 Abandon으로 authoritative active_expedition이 None이 되어 B의 profile 설치가 world.rs의 expedition identity 검사에서 먼저 거절된다. 시험명을 이 경계에 맞추고 profile identity, durable bytes, B 전체 probe 불변, pending intent 부재를 검사한다. 저장 revision 자체는 server001에서 통과한 u04_conditional_intent_rejects_stale_choices_and_terminal_missing_aba가 계속 담당한다. 임의 중간상태로 더 늦은 경계까지 강제하지 않는다.

focused004은 2통과/0실패다. 기본 서버의 server001 90pass + focused003 4pass + focused004 2pass로 현재 남은 고유96시험의 기본 구성 통과를 확보했다. 각 수정은 그 실패 사례 또는 전용 helper에 한정되며 이전90pass의 입력·공유 준비를 다시 바꾸지 않았다. 22ignore는 기존 전용 실행 경계를 유지한다. 한 번의 최종 전체서버 실행으로 표기하지 않는다.

feature_preflight001은 Core fault 관측1개와 계측기 정상외부전투1개 모두 통과했다. 후자는 none/normal/detached/full 네 관측모드에서 정상 전투의 최종 상태가 같고, 이전 reconnect의 공개 authority를 Script 바인딩으로 가져와 resume한 결과 paused=false까지 확인한다. Core는 entry 이후 실제 Elapsed work-limit 실패의 work/lease와 비commit/반복거절을 확인했다.

최종 feature 영향검사는 Core·서버 all-targets를 빌드하되 actor handlers, Core fault/관측, server observability/runtime_measurement 모듈만 선택한다. 계측기 최대 sessions/repeats는 이전 유효 실행165초 수준의 경계검사로 유지하며 규모를 줄이지 않는다. exporter 단독 heap(ignore)은 코드·입력 비영향으로 이전 단독 통과를 재사용한다. 기본 전체workspace 재실행은 하지 않고 Core2228/독립probe11 등 비영향 결과를 재사용한다.


## 최종 결과·검토

요청된 교정 완료. 기존 실패 **29개 중26개 교정·통과,3개 제거**다. 이름별 처분과 증거는 [dispositions.json](dispositions.json), 명령·환경·종료코드·타깃별 실제 수는 [RESULT.json](RESULT.json), 이번 수정만의 차이는 [changes.patch](changes.patch), 변경6파일의 실제 전후 식별은 [changes.json](changes.json)에 있다. u06.rs는 조사 전 원본만 보존했으며 이번 수정은 없다.

- 기본 구성: 서버 고유96pass/잔여실패0/22ignore의 합성 근거(server001 90 + focused003 4 + focused004 2). 전체를 최종 한 번에 재실행했다고 주장하지 않는다.
- 최종 runtime-measurement 영향 묶음: **96pass/0fail/1ignore, exit0, 총416.58초**. Core32pass(10.25초), 서버64pass/1ignore(356.25초). all-targets 빌드 완료, 0시험 타깃은 기능PASS에 더하지 않았다. 수정한26개 모두 이 최종 묶음에서 직접 통과했다. 이전 미실행 feature 전용2개도 통과했다.
- 서버 계측 실행에서60초 초과를 확인하여 실제 프로세스를 진단했다. 경과149초에 CPU 약2928%, RSS 약4.84GB로 연산 중이었고 이후 통과 로그가 계속 증가했다. 기본구성과 계측구성의 workload/비용을 같다고 해석하지 않는다. 최대1024 session/repeat 경계도 규모 변경 없이 완료했다.
- exporter heap의1ignore는 이전 단독 실행1pass를 비영향 재사용했다. 기본 Core2228pass·probe11pass와 기본28ignore 구분도 재사용하며 이번 영향검사를 전체feature 검증으로 확대하지 않는다.

검토는 현재 공식 시작의 보상→전투 도달, 작은 한도에서의 Tutorial battle-entry/실제 tick fault 도달, paired profile 소유권, continue→abandon→start, preintent/accept 이후 복구, stale profile의 초기 거절과 durable 불변을 대조했다. 작은 한도 시험의 entry 성공/Elapsed 실패를 유지했으며 제품 기본 한도를 올리지 않았다. 게임 엔진·서버 제품 동작은 변경하지 않았고, 선택 계측기의 공개 전투 진입만 현행 Expedition/loan/reward 계약으로 고쳤다.

삭제한 과거U07 재생의 개체 동작은 현재 Core u07_public_external_ron_trial_creates_actor_traps_walls_and_detaches_pool, u07_public_official_owned_forms_preview_cast_paused_reconnect_clone_and_continue 및 battle u07_official/u07_edges 회귀가 검증한다. 계측 Script→공개 응답 바인딩→명령→assert라는 고유 연결은 정상외부전투 모드동일성 시험에 합쳐 최종 통과했다. 외부 파일 변조 전용2개는 연속 지시서§4의 제외 범위이며 실제 IO 실패·재시도 시험을 대체하지 않는다.

수정6파일의 diff 공백 검사exit0, 보존 preimage의 최초 기록hash 일치, 기존 실패29개 처분 누락0을 확인했다. 원래 전체 실패로그와 중간 실패로그·과거 pack/감사 원문은 보존했다. 소유 실행은 모두 종료했으며 이 요청 범위의 잔여 교정은 없다. U17 등 후속 구현은 시작하지 않았다. U16-4 감사004의 과거 Complete/High는 그대로 보존하며 이번 선택 계측도구 변경을 과거 감사의 무변경 입력으로 주장하지 않는다.
