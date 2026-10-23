# U16-1 실행 기록

P2G40 생성/검증과 P2G50 검사 exit0/result passed. 기능시험은 아직0. P2G artifact와 원시 로그는 상위 디렉터리 참조. U15 선행 증거는 상위 Goal baseline reuse 계약대로 인수하며 U16 구현 증거로 확장하지 않는다.

- check001: cargo check -p game_core --tests --offline --locked, exit101. 원문 /tmp/u16-check001.log. 새 enum의 판매 match에 금지 세 종류를 명시하지 않은 compile 오류1(일반/test 각각). 해당 판매 경계를 명시 거절로 교정. 기능시험0.

- check002 exit0 (47.23초), 기능시험0.
- focused001 exit101/기능시험0: 새 시험의 ron::de/import/ValidatedGameData 준비 타입4오류. 제품오류로 세지 않으며 원문 runs/focused001.log 보존 후 교정.

- focused002 exit101: 5실행/4PASS/1FAIL, 29.57초(build110초). 신규 구입 결과 oracle이 재료를 일반 added가 아닌 material_stacks로 제공하는 기존 계약을 잘못 기대했다. 실제 재료잔액과 전용diff를 검사하도록 교정. Continue 양분기·24seed 기본촉매/갱신·거절불변·소비품치료는 통과.

- focused003 exit0: 7실행7PASS/0FAIL/ignore0, 30.29초(build51.68초). 65번째 유물 구입 허용과 HQ가격/기본세트포함/남은채용 공개 계약 추가. 위 focused002 실패 정확사례 포함 재검증. 전체 Core 공유영향 검사를 별도 시작.

- core001 exit101: lib2108PASS/6FAIL/ignore6/socket1 제외,249.65초. integration은 lib 실패로 미실행. Resource fixture의 RON 태그3종 누락, 기존 분쇄 fixture의 reserved dust 목적Generic, G11 구semantic identity4건이 실패했다. 원문을 보존하고 기존 oracle을 유지해 fixture를 수정했다. G11은 이전 공개 공급 profile·catalog bytes를 인수한 새manifest로 옮기고 실제 package/reload receipt에서 현재identity(5f12a428…)를 확인했다.
- normal-build001 exit0/7m26s, instrumented-build001 exit0/7m10s. 위 테스트 교정 전 preliminary artifact로 보관하고 최종 승인에는 동일 최종source의 host/worker를 재빌드한다.
- authoring001/package001 exit1: 대표shop 촉매tags가 공식 동품목tags와 달라 loader가 정상 거절했다. fixture를 맞춘 package002 exit0. authoring002 exit0: 정상1+작성오류9를 package의 실제 사전loader·출력후reload에서 검증했다.

- focused004 exit0: 13실행13PASS/0FAIL/ignore0,32.70초. core001 실패6개 전부와 U16 신규7개를 함께 재검증했다. 수정은 테스트 준비/입력식별자에 한정하며 제품 runtime은 동일하다.
- integration001 exit0: 8 integration target 합계72PASS/0FAIL/ignore0. server001 exit0: persistence31PASS/0FAIL,0.34초. Core 현행 고유통과2186(2108+교정6+integration72), socket1은 이전 실제 허용환경 검사와 현재 snapshot IO source 비영향을 대조해 재사용 예정이다. 고정된 과거 원문은 변경하지 않았다.
- prepared build의 --with-worker를 추가해 같은 plain graph에서 선택 host+worker+packager를 한 Cargo 호출에 결속한다. 기존 worker-only/선택probe 동작은 유지하고 잘못된 조합은 인수 단계에서 거절한다. 최종 normal_build002와 instrumented_build002 실행 중이다.
- 실제 승인 시나리오는 기존 Event→Combat과 새 Event→SavePoint→Shop의3종구매/갱신/나가기 두 개다. 후자는 사전 공개 snapshot의 offer ID를 binding하며 가격60/60/150을 독립 기대값으로 검사한다. 정상 fixture002에는 지원상한2를 지킨 HQ6/Shop8 예약과 기존 소비품·장비 상품을 작성했다. package003 exit0.


- normal_build002 exit0/6m18, instrumented_build002 exit0/5m51. 초기 정상 승인 normal_calibration002/limit_review002/adoption002 PASS. public_inputs003의 장비64개 정상 지급 pack004도 actual package/reload PASS.
- authoring003 exit0: 추가 제작오류3건을 정확 진단으로 거절. authoring0029건과 합계12거절/정상1. 기존 monolith+base_price 복합오류와 전용 monolith 금지를 구분한다.
- normal_host002의 정상 상점/HQ/SavePoint 새프로세스8단계, 실제OS21 세 번은 통과했다. expanded 구매까지 통과했으나 completed_continue003은 retained19939869>16777216으로 실패. PRE_FIX_STORES.tar.gz와 요청/오류/원본을 보존했다.
- cost001: 기존 paid profile 동일bytes/동일 public12단계, normal3+expanded3 fresh processes PASS/정확회수. 복원 교정 전 비교 자료로 유지한다.
- expanded-regression001 exit0/1PASS7.40초: 실제 Core grant64장비+신규상품3구매/완료 후 default Continue. restore-regression001 exit0/44PASS88.06초: U08 관련기존 경로, 누적3예약과 각1byte부족 거절 포함. final-focused001 exit0/28PASS40.20초: U04/U16/shape scanner. final-public001 exit0/15PASS19.81초: 외부 content/public application의 무료 Event·거절·후퇴/Continue·유물 판매 금지.
- restore_cost001: 제품 allocation.rs 그대로 include한 JSON Value30형태 실제allocator 상계/정확회수 PASS. 실패 당시 실제171980bytes checkpoint의 public Continue3fresh process 모두PASS, native14151702/heap증가max1859234bytes/해제후동일baseline. dev opt0/debug0이며 정상호스트 승인을 대신하지 않는다.
- 최종 normal_build003 exit0/7m15, instrumented_build003 exit0/6m43. host/worker 공통 source_compiler7568d061…; 정상호스트 default release artifacts/normal002, 계측worker instrumented002로 보존했다. final normal/expanded calibration003·review003·adoption003 전부PASS/기존수치한도유지.
- cost002는 잘못 선택한 초기 probe가 provenance에서 거절(exit1), 원문보존. paid_cost001의 원본 소스를 정확히 선택한 cost003 exit0/6fresh process72stage PASS. 동일workload의 prepare/Continue 실제heap peak는 전후 동일, 전량회수.
- 최종 normal_host003 exit0: pilot포함12fresh process72명령/실제OS21 세 번, 추가Event3fresh process12명령/실제OS21 한 번. 합계15process84명령/실제저장실패4회 모두불변+재시도. 상점/HQ 미완료 입장전 되돌림과 완료유지, SavePoint, Event raw20 고정+정확재요청+완료유지, 장비71/occupied73 확장Continue를 확인했다. 동일프로세스 반복을 fresh-process로 세지 않는다.
- bound_inputs001 준비는 원본 RON 공백 형태를 잘못 가정한 assert 실패, 제품 검사0. 정상 minified source의 해당 bound:false만 true로 작성한 bound_inputs002는 final packager exit0. public_edges001은 정상 공개원정12명령으로 준비하여 귀속3/장착3/0수량/미존재/재료3 sale 합계11건을 정확사유+모든public roots 불변으로 검사했다. 이 제작fixture/application 증거를 정상host 저장 증거와 구분한다.

- consumable001은 무배치 상태에서 300 clock 호출 상한에 도달했다. consumable002는 실제 단독 배치의 패배25tick과 효과만료/후퇴유지/중복확인 거절을 통과했다. consumable003/004는 배치 구역 밖 좌표, consumable005는 플랫폼 전용 직원을 지상에 배치한 준비 오류로 거절됐다. consumable006의 교대 전술은 300호출 내 종결하지 못했다. 오류·미완료 관측과 입력 원문을 보존하며 승리 증거로 세지 않는다.
- consumable-build008 exit0. 실제 공개 미리보기의 허용 지상 구역과 배치 적성에 맞춘 consumable007 exit0/5.58초: 동일 HQ 구매·사용 저장본으로 패배25tick/승리18tick 두 사례 모두 통과. 각 사례에서 후퇴는 기간1 유지, 결과확정은 효과제거, 결과확인 뒤 중복확인은 정확 거절·run 불변. 내부 강제 승패·저장 변조 없이 공개 CoreApplication 명령과 clock만 사용했다. 이 진단은 dev build이며 정상 host 프로세스15개에 합산하지 않는다.
- summary001 exit0: 실행된 정확 test name과 실패6개 closure, 비영향 socket 원문/동일 source를 검산했다. 고유 Core2188, 기존ignore6, 알려진 현재실패0. native4/server31, 정상호스트15process84명령/실제OS21 네 번, 작성오류12와 판매거절11, 실제 소비품 승패2, 비용·회수 증거를 COVERAGE/TEST_RESULTS_INDEX/BASELINE_REUSE/COST_REVIEW에 연결했다. 수용18/차원10/통합5이며 단일 all-targets 전체녹색 실행으로 표기하지 않는다.
- 최종 고정은 test_parts/finalize.py로 SOURCE/BEFORE byte equality와 격리된 정방향·역방향 복원을 확인하고 AuditTargetRecord를 만든다. 실행 결과는 target/u16_1_current_001/RESTORE.json 및 고정 뒤 GEX_10_RESULT_001.json에 기록한다. 이 절차는 작업 트리를 되돌리지 않는다.
