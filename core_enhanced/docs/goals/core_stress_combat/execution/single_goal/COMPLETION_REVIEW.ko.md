# U16-4 잔여 통합 검증·최종화 검토

2026-10-15 사용자의 잔여 검증·최종화 수행 요청으로 GEX-10의 기능·저장·수명·비용 검증을 마쳤다. 이번 pass는 제품을 수정하지 않고 시험·진행기·결과 결속을 추가했다. [집계](completion001/RESULT.json), [수용 연결](COVERAGE.json), [검사 색인](TEST_RESULTS_INDEX.json), [재사용 판단](BASELINE_REUSE.json)이 실제 범위를 구분한다. **구현·자체검증 종료이며, 필수 비참여 최종 완료감사와 Goal Complete 판정은 아직 인수하지 않았다.**

## 실제 검증

승인된 정상 host의 성공한 17개 프로세스에서 192개 연산을 관찰했다. 정상 9명·5라운드, 공개 비용 소진 후 무억제 5라운드, 첫 라운드 성공 철수, 누적 후 수동 후퇴, 실제 보호 목표 파괴, SavePoint, 재접속 및 새 프로세스 복원을 통과했다. 최종 결과 확인 전 CombatResult도 새 프로세스에서 그대로 복원했다. SavePoint 검사는 작성된 경로의 전투 전 SavePoint이며 전투 후 재진입이라고 확대하지 않는다.

정상 성공은 통화150·가루15·파츠15·출전 생존직원 각각25XP·유물1개다. 무억제는 통화300·가루30·파츠30·각각50XP·유물1개다. 무억제 적9명 모두 전5웨이브에서 HP135/공격15이며 정상90/10과 대조했다. 패배는 pending 보상을 전부 잃고 기본 생존XP10만 지급한다. 수동 후퇴는 진입 전 직원·자산·소비품 기간을 복원하고 최초 입장비용과 노드 소비를 유지한다.

선택 구간4회에서 대기시간·checkpoint가 정지하고, Continue 후 HP·유닛정체성·배치·자원·효과·시간이 유지됐다. 실제 최종tick·성공철수·수동후퇴·SavePoint의 journal IO 실패4회는 메모리와 양쪽 저장소가 불변이며 같은 요청 재시도로 완료됐다. 중복 완료/철수/Continue 거절도 확인했다.

격리 사본에만 추가한 [경계 probe](test_parts/COMPLETION_PROBES.rs)는 **4통과/0실패/0ignore, exit0**이다. 실제 미만료 태세 cooldown, proc 내부 cooldown과 전투당1회 제한, 마지막 적 사망과 같은 시각의 작성 패배 우선, 실제 무력화 후 다음 입장비용으로 breakdown 사망과 장착유물 회수를 검증했다. 마지막 사례는 전투 중 즉사와 구분한다. 실제 host의 확장전 패배 관측에서도 직원 injury1·trauma26(40에서 memory_fixative35% 완화)·XP0·소비품 기간1회 차감을 확인했다.

확장 성공은 동일 기본 적 능력치의 27명·5라운드, 지원 작성으로 넓힌 spawn 공간과 전방 배치칸, 공개 TP 자격 확인, 3인 배치와 이미 보유한 유물 추가 장착을 사용했다. 앞선 전략의 실제 패배와 준비 오류는 [EXPERIMENTS](EXPERIMENTS.md)에 보존했다. 이 실패를 성공 비용 측정으로 세거나 입력 규모·한도를 낮춰 통과시키지 않았다. 대표 작성 편대의 기능 검증이며 공식 기본편대의 최종 밸런스 판정은 아니다.

## 비용과 한도

[비용 원시 결과](completion001/cost001/RESULT.json)와 [한도 판단](COST_REVIEW.json): 정상/확장 동일 진단 binary로 각3회 순차 재생했고 host의 결과와 일치했다. 최대 공개 Core 연산시간은 정상148.93ms/확장310.37ms, native work 최대877,108/1,124,330이다. 전체 프로세스 요청 heap peak는82,909,011/73,958,188바이트, RSS peak는144,318,464/133,517,312바이트다. 입력 trace/data 상주 기준값이 다르므로 확장 프로세스 peak가 작은 것을 최적화로 해석하지 않는다.

6회 모두 application drop 후 요청 heap이 정확한 기준값으로 돌아왔다. RSS는 allocator가 확보한 페이지를 유지하므로 원복을 주장하지 않는다. Core 연산창은 trace 직렬화·disk publication을 제외한다. normal host 저장 정확성, admission pilot calibration, 전체5라운드 동적 비용 재생은 각각 별도 증거다. 기존 runtime 및 승인 한도를 유지하며 최종 프레임 SLA·서버 전체 latency·U21 성능 완료를 주장하지 않는다.

## 코드·회귀와 잔여

실제 kernel의 선택 대기 early return, 시각 bucket 정리 후 round settle/패배 우선, 단일 ActiveBattleSession의 continue/extract identity와 waiting gate, 공통 Victory 보상 누적·배율·ConfirmedCombatResult, 공통 XP와 entry checkpoint 후퇴·복원 owner를 검토했다. 새 전투를 라운드마다 만들거나 자동 회복·대체 보상 경로를 추가하지 않는다. 감사003 이후 제품·공식data·config·기존시험·계약 입력은 변하지 않았다.

기존 Core 고유2222통과/ignore6는 원래 broad 실행과 교정 재실행의 합산 결과를 비영향 근거로 재사용했다. 후속 누적보상 교정의 RON25·공개패키징 및 XP 영향49검사는 별도로 결속했다. 겹치는 focused 수를 합산하지 않는다. 서버 메시지8통과/선행 fixture1실패는 그대로 보고한다. `SelectStarterEmployees.loan_copy_ids` 누락은 착수 전부터 존재한 비영향 fixture 문제이며 서버 전체 green은 아니다.

수용10·검증차원10·통합수용5·실행node2의 자체검증 근거를 정리했다. 최종 입력은 [target 기록](target/u164_current_001/AUDIT_TARGET_RECORD.json)에서 source/evidence/변경분/정역복원을 연결한다. 원본 실패·이전 감사·교정과 사용자 dirty 작업을 보존한다. 현재 실행 지시서 §8의 “자기 검토를 비참여 감사로 표기하지 않는다”와 “서브에이전트를 소환하거나 자동 위임하지 않는다”에 따라, 비참여 최종 감사는 별도 실행자의 인수 조건으로 남긴다. U17 이후는 시작하지 않는다.
