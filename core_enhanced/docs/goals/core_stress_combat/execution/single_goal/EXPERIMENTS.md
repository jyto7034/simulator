# U16-4 실행 기록

2026-10-15 사용자는 U16-4 구현·완수 후 비참여 서브에이전트 감사를 직접 승인했다. root 단독 구현 중이며 감사자를 아직 소환하지 않았다. actual pre-mutation532파일을 BASELINE_SOURCE_001에 보관했다. 기존 dirty와 U16-3 독립 감사/동결은 보존했다.

P2G00–60을 발행하고 40/50 구조 validator를 통과했다. 정식 artifact 출력은 첫 컴파일 중에 했으며 static review 근거는 변경 전 baseline archive에서 고정했다. 40 parallelization 필드명과50 초기baseline owner를 수정하기 전 기록을 보존했다. 문서 출력 시점을 제품 검증으로 주장하지 않는다.

check001/002/003은 컴파일 진단이 나오지 않아 실행자 소유 세션을 Ctrl-C로 종료했다. 003은45/90초 프로세스 관측에서 Cargo만 진행했고 rustc가 없었다. check004 debug 로그로 Cargo가 대량 docs/goals 과거 증거를 재귀 탐색하는 원인을 확인한 뒤 종료했다. 검증 환경 교정으로 최소 입력을 /tmp/u164-build에 복사했다. 원본 문서·과거 증거·Cargo package 계약은 제거/완화하지 않았다. check005는 실제 rustc 실행까지 도달했고 action projection array/비fallible snapshot 인자 오류2개를 발견해 교정했다. 이 결과들은 기능 PASS가 아니다.

focused001은 test observation의 RuntimeUnit Serialize 가정 오류로0시험 컴파일 실패. 직렬화를 제품에 추가하지 않고 기존 kernel/queue diagnostic과 공개 DTO를 관측한다. focused002는1pass/4fail: 실제 generated setup이4·5명 웨이브의3칸 초과를 거절했다. 콘텐츠를1/1/2/2/3명과I/II/II/III/III 등급으로 조정해 증가 난이도와5라운드 요구를 유지했다.

focused003은2pass/3fail: 작성·실제 보호 대상 파괴 경로 통과, 초기TP20으로3명 배치 가능하다는 fixture 가정 실패. 배치 전략을2명으로 교정했다. focused004는2pass/3fail: 후퇴 실제 자산은 동일하고 battle UUID stream만 snapshot 시점으로 정상 복원됨을 직접 diff로 확인했다. 아이템/사본 정체성 검증은 유지하면서 unrelated battle stream을 자산 오라클에서 분리했다. 공격 범위 밖 배치로60초 안에 처치에 도달하지 못한 두pilot도 보존했다.

focused005는2pass/3fail: 임의 일반 바닥으로 tactical relocate를 허용한다고 가정한 준비 오류(OutOfBounds). 공통 전투 규칙은 배치 가능 칸 사이 이동만 허용한다. 제품 규칙을 바꾸지 않고 정상 선택 가능한 scout_runner 원거리 직원 + breach_guard, route 끝 인접 합법 배치 칸과 접근 방향을 사용하는 전략으로 교정했다. 판정 조건/적 수/HP/시간/보상/5라운드 요구는 완화하지 않았다. 다음 실행 전 유효한 적 접촉 근거는 공식 무기 범위·P/T 배치 칸·접근 route에 있다.

분리 검증 입력은 동기화 완료 후 고정하며 실행 중에는 복사본을 수정하지 않는다. 원본에서 후속 패치를 준비한 경우 최종 통합 입력 동기화 후 영향 시험을 재실행한다. 정상 host/성능 증거는 아직 준비 중이다.


## 사용자 요청에 따른 U16-4 전면 재검토 (2026-10-15)

이전 구현자의 요약/집중 PASS를 결론으로 승계하지 않고 actual baseline532 대비 변경32파일, 신규 정책/World 검사/host와 공유 probe manifest, 정책·P2G·공식/외부 입력·원시 로그를 다시 대조했다. 수정 전36파일은 RE_REVIEW_INITIAL_001.tar.gz, 차이는 REVIEW_U164_DIFF.patch에 보존했다. 현재 재검토는 root의 자체 재검토이며 비참여 독립 감사가 아니다. 사용자는 원래 U16-4 완수 후 독립 감사 요청을 유지하면서 먼저 지금까지의 U16-4 전부를 재검토하도록 지시했다.

이전 focused006의 fresh retreat 재도전 결함은 continuation의 NodeConfirm 분기로 교정됐으며 focused009 6pass는 그 내부 검증까지의 증거다. focused007/008의 뒤 라운드 패배는 기본/공격40 편대 전략 결과이고, 공격80 작성 편대로 전체5라운드 양성 대조를 만들었다. 기본 편대 전체5라운드 밸런스를 검증한 것으로 쓰지 않는다. core001은 0실행 컴파일/링크 실패: 새 enum 누락된 ron_loading match, 격리복사 docs2파일 누락, tmp 공간 부족으로 linker bus error. instrumented-build001은 no-space exit101. normal-build001은 release exit0 (6m14s)이지만 정상 호스트 시나리오 실행 증거가 아니다.

/tmp 16GiB 고갈로 sandbox launch까지 실패했다. 승인된 mv로 /tmp/u15-cost-slim-target을 .cache/u164-review-dev-target으로 옮겨 삭제 없이 약8GiB를 확보했다. 이후 dev출력은 작업공간을 사용한다. 격리복사에 live_skill_catalog_manifest.ron/live_content_index.md를 포함하고 기존 RON 참조 테스트의 StressCombat 분기를 보완했다.

재검토 R01: 일반 지속 보호막은 Any 범위 delivery로 보호 대상에 도달할 수 있고, 기존 새 제한은 heal/buff/modifier만 막았다. review-focused001에서 실제50피해 흡수로 재현(기대0). sustained effect owner에 같은 아군 보호 대상 제한을 적용했다. 기존 일반 전투50흡수 양성 대조를 유지한다. 명시 RepairProtectedUnit 통과도 유지한다.

재검토 R02: 후반 라운드 u32::MAX 보상은 정상 로더가 승인하지만 무억제2배 안내/지급에서 overflow한다. review-focused001의 로더 거절 assertion이 실패해 승인 사실을 확인했다. 고정2배로 표현 불가능한 통화/가루/파츠/XP를 로딩에서 거절하도록 수정했다. 기존 런타임 checked_mul도 유지한다.

재검토 R03: GroupCleared만 미래 라운드 의존을 검사하고 ActorHealth/ActorDefeated/ActorReachedPoint의 바인딩 source wave를 검사하지 않았다. review_actor_future001/source는 정상 작성 입력이며, round1이 round5 actor HP100%를 기다리고 round5는 AtTime1000이라 기존 전역 DAG에는 cycle이 없다. 수정 전 공개 package_official_content가 exit0으로 승인한 원문은 runs/review-actor-public001.log. round gate 때문에 round1은 완료할 수 없다. 모든 actor trigger의 source wave도 현재/앞 라운드만 참조하도록 수정했다. 같은 actor가 앞 라운드이면 수용하는 대조를 추가했다.

review-focused001: 8실행/6pass/2fail (R01/R02 의도적 검출). review-focused002는 새 negative test의 expect_err가 ValidatedGameData:Debug를 요구한 시험 컴파일 오류,0실행. 제품 Debug를 추가하지 않고 err().expect로 교정했다. review-focused003: 9실행/9pass/0fail/0ignore,9.23s. R03의 정상/오류 로딩 대조 포함. 일반 지원 경로의 작성 오류/효과이며 임의 save/checksum/메모리 변조 시험이 아니다.

비용 경계 정적 재검토: 생성하는 enemy_ wave-name Vec/String에 사전 native admission을 추가하고, 미수령/다음 보상 소유 projection의 동적 문자열도 admission을 거치도록 했다. 선택 완료 중복 호출 settle_stress_round 1개는 제거했다. 실제 peak/회수·확장 비용 검증은 여전히 미실행이며 정적 보강을 비용 PASS로 표기하지 않는다.

host harness는 fail:true에서 임의 거절을 실제 IO로 오인할 수 있어 os error21 확인을 추가했고, reject:true의 disk 무변경도 비교한다. stop만으로 passed/fresh_process:true를 내보내던 표시는 protocol_completed로 바꿨다. 상위 시나리오 오라클이 기능 합격/새 프로세스 복원을 입증해야 한다. 아직 실행된 host 검증은 없다. 외부 normal_suite에 이전 딜러 거래가 남아 StressCombat 배정과 충돌했으므로 원본을 before_review에 보존하고 pilot 시나리오만 남겼다. 이 pilot은 최종5라운드/확장 비용 증거가 아니다.

ACCEPTANCE.json의 A02–04 순서가 frozen P2G30/P2G50과 달랐다. 원본을 ACCEPTANCE.before_review.json에 보존하고 frozen10조건의 ID/문구로 파생 색인을 정렬했다. 동결 정책·Goal의 의미/문서는 변경하지 않았다. 과거 LOAN/SHOP 식별자는 의미 없는 잔재지만 내용은 XP/후퇴·무억제이며 고정 참조를 새로 만들기 위해 고치지 않는다.

rustfmt host 실행이 참조 mod의 기존 normal_host_support.rs까지 정렬한 것을 확인했다. U16-3 SOURCE_MANIFEST와 해시가 일치하는 기존 isolated restore의 원문으로 해당 부수 변경만 즉시 복구했다. 새 Rust 포맷은 이후 skip_children=true로 실행한다.

review-core001은 동일 격리 입력으로 전체 영향 검사를 진행 중. ApplicationCommandKind/ActionKind의 기존85개 count assertion 누락이 발견되어 원본 실패를 보존하고 root에서86개로 수정 준비했다. 실행 중인 copy는 바꾸지 않았다. 종료 후 이 한 시험과 아직 실행되지 않은 integration/server 범위를 분리해 검사한다. 정상host/실제IO/SavePoint/소비품수명/재료·유물·XP 무억제와 비용·최종고정·독립감사는 아직 미완료다.


재검토 회귀 종료: review-core001은lib2143pass/7fail/6ignore(216.75s). 두 count assertion을86으로 갱신하고 실제 G11 fixture/원문receipt/구 콘텐츠를 격리복사에 보충했다. review-regression-fixes001의6검사(2count+4G11)는6pass(4.91s). socket은 실제EPERM 확인 후 승인된 제한 밖 exact1검사로1pass(0.08s). 따라서 lib고유2150pass/ignore6이며 단일fullgreen 아님.

review-integration001은 앞4개 integration target13pass 뒤 ron_loading24pass/1fail. 기존 “모든 PVE는 reward_uuids에 보상” 오라클이 라운드 보상 owner를 반영하지 못했다. 스트레스는 일반 grant가 비어 있고 실제 layout의5개 fixed round reward가 존재하는지 검사하는 분기를 추가했다(단순 skip 아님). review-integration002에서 ron25+skill runtime15+skill suite4+public relic15=59pass. 앞13과 합한 integration고유72pass, lib와 합하면 Core고유2222pass/ignore6. 집중9는 이2222에 포함되며 더하지 않는다.

review-actor-public002는 같은 미래 actor 입력을 교정한 dev package_official_content에서 실행해 exit1/FinalValidation/invalid stress combat policy로 거절했다. 수정전exit0과 수정후거절의 공개도구 대조를 확보했다.

review-server001은 메시지9검사 중8pass/1fail. 새 StressCombatAct를 포함한 live battle parsing과 setup/update/resync 직렬화는 통과했다. 실패한 deserializes_snake_case_node_flow_requests의 첫 SelectStarterEmployees fixture는 필수loan_copy_ids를 생략한다. U16-4 BASELINE_SOURCE의 behavior.rs에도 같은 필수필드, messages.rs에도 같은 누락fixture가 있어 이번 작업 이전부터 존재한 불일치임을 원문으로 확인했다. U16-4 한정 재검토 요청에 따라 선행 fixture/제품 계약을 임의 변경하지 않고 기록한다. server전체green이나 선행31개 재실행으로 주장하지 않는다.

수정 후 제품 source로 필요한 영향검사를 종료했으며 실행 중 세션은 없다. /tmp/u164-build는 현재 코드 사본(계약문서 포함), devtarget은 .cache/u164-review-dev-target이다. normal-build001 release는 교정 전이므로 최종호스트 증거로 재사용할 수 없다. instrumented-build001은실패이며미재실행. 정상호스트/IO/SavePoint/비용 및 최종동결/비참여감사는 잔여다. 원시수치/해시는 RE_REVIEW_RESULTS_001.json, 해석은 IMPLEMENTATION_REVIEW.ko.md.


## 2026-10-15 현재 사용자 요청에 따른 독립 감사 종료

사용자가 지금 U16-4 독립 감사를 직접 요청하고 현재 지시 우선을 명시하여, 기존 구현·교정에 참여하지 않은 이번 별도 실행이 현재 구현을 읽기 전용으로 감사했다. [감사 결과](independent_audit001/COMPLETION_AUDIT.md)는 **Not Complete / 제품 finding F-U164-IA001 1건**이다. 개별 수량 검사만으로 누적5,000,000,000 보상표를 승인하여 실제5라운드 정산에서 InvalidAction·동일tick 재시도 실패를 재현했다. 공개 패키징도 오류입력을 승인했다. 부분지급은 없고 수동후퇴는 가능하다. 원본 제품·기존 시험·공식입력은 수정하지 않았다.

감사 격리 사본의 기존9+신규4 검사에서12pass/1fail, 후속 exact 오류재현진단1pass이며 이 진단PASS를 제품합격으로 계산하지 않는다. 다중wave·정상소진/Continue 무억제진입·XP/가루 누적과 pending뒤후퇴의 Core 대조는 추가 확인했다. 기존2222pass/ignore6는 원시로그해시와 고유성공이름으로 재검산했다. 실제 host/reconnect/fresh process/SavePoint/pairedIO·파츠/유물/소비품·동시각패배·비용·최종동결/정역복원은 여전히 잔여다.

현재 감사는 최종GEX10/target이 준비됐다는 판정이나 완료 후 GEX20을 대체하지 않는다. 다음 owner 작업은 해당수량경계 교정과 남은 검증/최종화다. 이번 요청에서는 교정을 수행하지 않았고 U17 이후로 진행하지 않았다. 감사입력653파일 출구불변 및 archive일치 확인 후 이 인계 절만 추가했으며 비영향 문서변경은 independent_audit001/POST_AUDIT_DOCUMENTATION.json에 기록했다. 모든 실행 세션은 종료·회수했다.


## F-U164-IA001 누적 수량 교정 (2026-10-15)

사용자가 교정을 요청하여 별도 기술교정 pass를 시작했다. audit input과 원본3파일의 일치를 확인하고 correction001/before에 보존했다. 기존 독립감사 결과/실패 로그는 변경하지 않았다. 이번 실행자는 이제 구현교정에 참여했으므로 후속 자체검토를 비참여감사라고 표기하지 않는다.

잔고 owner를 직접 확인했다: 엔케팔린 checked_add, 가루 RunCraftingResources, 재료별 equipment_materials.add. GrantEquipmentMaterial(equipment_dust)은 동일 가루 owner다. XP는 각각의 grant 후 레벨 비용을 소비하므로 단순 누적지갑 검사를 추가하면 유효한 grant연쇄를 거절한다. 따라서 보존되는 잔고만 합산하고 개별 XP배율 검사·실제25/50XP 대조는 유지한다. 고정 보상표 총량과 이미 가진 잔고를 더할 때의 공통 runtime 거절·원자성을 구분했다.

수정전 red001은 새3검사1pass/2fail(12.22s,exit101). 서로다른owner/XP분리는 허용됐고, 초과통화와 mixed가루alias의 거절 assertion이 실패해 기존 결함을 검출했다. 실패사본은 실행 중 고정했으며 root에서 준비한 제품교정은 종료·회수 후에만 사본으로 동기화했다.

수정은 StressCombatPolicy.validate 내부의 엔케팔린/가루/재료ID별 누적 checked_add·checked_mul(2)이다.5라운드 및 같은라운드 반복효과를 함께 계산하고 가루alias는 합친다. 유물수량·XP지급·런타임정산은 변경하지 않았다. 새정책/Goal/완료조건 변경이 아닌 기존 누적지급 계약의 기술교정이다.

수정후 green001은16pass/0fail/0ignore(16.61s,exit0): 기존9, 영구회귀3, 교정사본에만 추가한 양성replay4다. 과거 감사의 양성3경로는 기대값 그대로 재사용했고, 추가 허용경계경로는 정상 작성 초기잔고0/각라운드429496729·마지막429496731로 총2147483647, 무억제4294967294를 실제5라운드 완료 및 Core export/import까지 확인했다. 배치·처치는 실제 전투진행이며 내부적삭제/보상state조작이 아니다. probe의 independent라는 시험명은 과거 파일에서 가져온 이름일 뿐 이번 교정을 독립감사로 의미하지 않는다.

론로딩회귀/공개패키징 후속 결과와최종입력 identity는 correction001/RESULT.json 및 후속기록에 연결한다. 기존2222전체시험 재실행이나 실제pairedhost/freshprocess/SavePoint/비용 PASS로 확대하지 않는다.


교정 종료: loading001은25pass/0fail/0ignore(10.95s,exit0). 새 공개 패키징은 감사 때 승인된 동일10억×5 입력을 FinalValidation/cumulative enkephalin으로 거절(exit1)했고, 정상누적2147483647의경계입력은 승인(exit0)했다. green001의 실제정산2147483647/4294967294와 함께 거절·허용 양성대조를 확보했다. canonical3파일은 시험입력hash와같고 AFTER.tar.gz 보관검증도통과했다. 모든세션종료. [교정검토](correction001/CORRECTION_REVIEW.ko.md), [결과/원시로그identity](correction001/RESULT.json). F-U164-IA001은교정·영향검증완료이나별도재감사미인수이며, 기존Goal의host/저장/비용/최종화잔여로U16-4는Not Complete다. 이전독립감사원문을완료로변경하지않았다.


## 2026-10-15 독립 재감사002 종료

현재 사용자 요청에 따라 이전 구현·교정 비참여 실행자가 읽기 전용 감사했다. [독립 재감사](independent_audit002/COMPLETION_AUDIT.md)는 **Not Complete**다. F-U164-IA001은 교정 확인으로 닫았으나, 공통 XP의 레벨업 전 saturating_add가 지원 작성 무억제 성공 보상에서19XP를 소실시키는 새 P2 F-U164-IA002를 재현했다. 공통 employee.rs는 U16-4 착수 전과 같으므로 correction001의 회귀로 귀속하지 않는다. 실제 기대4,294,967,314/반영4,294,967,295, 정상 대조2,147,483,657일치다.

기존16개 재실행 통과, 신규 가루alias/환상체파츠 경계·유물1개 유지·노드소비·중복거절·Core복원 통과. 원시 probes001은16pass/1감사fixture실패, 준비 교정 후 probes002는1pass/1XP제품실패로 각각exit101이다. RON25/공개패키징 양·음성은유효증거재사용. [재현](independent_audit002/REPRODUCE.md)/RESULT.json 참조. 기존 host/실제저장/소비품·직원손실/동시각/비용/최종화 잔여는 유지한다. 제품·기존시험·공식입력은미수정,724파일감사출구불변/archive검증뒤인계링크만추가했다. 다음은 공통XP owner 교정과 영향검증 및 기존 Goal잔여다. U17이후로진행하지않았다.


## 2026-10-15 F-U164-IA002 교정 pass 착수

사용자의 “그럼 교정해줄래?” 요청에 따라 공통 XP 소실 결함을 별도 GEX-10에서 교정한다. 이전 independent_audit002와 실패원문은 보존한다. 이번 실행자는 이제 교정에 참여하므로 이후 검토를 비참여 독립 감사로 부르지 않는다.

기존 잔여XP와 이번 지급량은 각각u32이므로 임시합계를u64에 정확히 보존하고, 기존 authored 레벨비용을 차감한 뒤 u32범위의 잔여만 저장한다. 공개 필드/저장 형식/보상량/대상/기본생존XP/레벨최댓값의 기존 saturating 의미는 유지한다. 런타임 계약의 saturating-add 설명은 이번 기술교정에 맞춰 갱신한다. 상위규칙의 남은XP유지·지급총량을 구현하는 교정이며 새로운 성장정책을 만들지 않는다.

실행 전 수작업 기대값: 비용10억일 때20+(u32::MAX-1)=4,294,967,314 → 레벨5/잔여294,967,314. 비용u32::MAX에서(u32::MAX-1)+u32::MAX → 레벨2/잔여u32::MAX-1. 가변비용[7,11,13]에서0/경계/여러레벨/마지막비용반복은6→7→44→56→57누적XP에 대해(1,6)/(2,0)/(5,0)/(5,12)/(6,0)이다. 내부불가능상태를만들지않고신규직원에순서대로지급한다.

영구회귀3개를수정전실행해검출력을확인한뒤제품을교정한다. 현재감사의무억제XP실패probe는양성대조/목표량그대로사본에서재실행하고, 공통XP대상/결과diff·실패원자성·생존XP·U14/U16-4를선택실행한다. 정상host/전체회귀/비용검증으로확대하지않는다. 별도사본 `/tmp/u164-correction002`, 기존cache/오프라인locked/devdebug0. 기존빌드49–125초/집중17초,첫60초로그관측및5분초과진단. 실행중입력고정,종료뒤동기화한다.


IA002 red001 종료: 영구회귀3개중1통과/2실패,exit101. 작은가변곡선/0/경계는통과하고 생존XP뒤대량보상 및 두대량보상에서기존포화손실을검출했다. 실행종료후합계를u64로교정하고동일시험입력에반영했다. 레벨증가·성장곡선·저장필드는유지하며잔여는양수u32비용보다작아진뒤저장한다. 다음단일빌드선택실행은공통experience/생존XP/U14/U16-4/정산실패원자성을묶는다.


## 2026-10-15 F-U164-IA002 교정 종료

공통 Employee XP를 u64임시합계→기존레벨비용차감→u32잔여저장으로교정했다. [교정검토](correction002/CORRECTION_REVIEW.ko.md)/[원시결과](correction002/RESULT.json). 수정전영구회귀3개1pass/2fail→수정후영향49pass/0fail/0ignore(exit0). 감사의동일시나리오에서정상2,147,483,657/무억제4,294,967,314를출전직원두명모두정확히반영한다. 기존소량XP/대상별지급/기본생존XP/level최댓값/실패원자성/U14/U16-4도확인했다. canonical변경은employee.rs(제품및영구회귀3)와core_runtime_contract.md이며,저장형식·수량·대상·성장곡선은유지한다. 원문보존·시험입력일치·2파일archive와patch정역복원검증완료. 전체2222재실행/host/저장/비용PASS가아니다.

F-U164-IA002는교정·영향검증완료/별도독립재감사미인수다. 이번실행자는교정참여자로후속자체검토를비참여감사로표기하지않는다. 기존Goal잔여와Not Complete를유지하고모든실행을종료했다. 기존감사/실패/dirty/사본은보존했다.


## 2026-10-15 독립 재감사003 종료

사용자의 현재 지시서에 따른 독립 감사 요청으로 구현·교정 비참여 실행자가 correction002 이후 현재 구현을 감사했다. [감사 결과](independent_audit003/COMPLETION_AUDIT.md)는 **Not Complete**다. F-U164-IA002 XP 소실은 교정 확인으로 닫고 F-U164-IA001 종료도 유지한다. 확인한 범위에서 새 제품 결함은 없다.

새 격리 사본에서 공통XP/생존XP/U14/U16-4/실패 원자성49개가49통과·0실패·0ignore, exit0이었다. 원래 실패 probe의 입력·행동·기대값 그대로 정상2,147,483,657/무억제4,294,967,314를 출전 두 직원 각각 확인했다. 기존 작은XP/통화·가루·파츠 경계/유물1개/후퇴·패배도 통과했다. [재현](independent_audit003/REPRODUCE.md), [원시결과](independent_audit003/RESULT.json). RON25·공개패키징과 과거 회귀는 입력 비영향 및 원시해시 확인 범위만 재사용했고 전체2222 재실행·host/저장/비용 통과로 확대하지 않았다.

감사 입력753파일과 archive의 출구 일치를 확인한 뒤 이 요약·인계만 별도로 갱신했다. 제품·원본시험·공식콘텐츠·과거감사는 불변이다. 정상host/reconnect/freshContinue/SavePoint/pairedIO 실패·동일명령 재시도, 실제 직원손실·소비품수명, 쿨다운/전투당 기록·동시각패배, 전라운드 무억제 적능력치, 정상/확장 시간·heap/RSS/회수·한도판단과 coverage/최종target/정역복원/GEX10 종료가 남는다. 이번 감사가 그 최종 완료 감사를 대신하지 않는다. 모든 실행은 종료했으며 다음 구현·U17 이후는 시작하지 않았다.


## completion001 정상 host 준비

normal-build002는 현재 source에서 성공(exit0,6분16초). completion-package001은 destination의 parent content 디렉터리가 없는 준비 오류로 Acquisition/os error2, 제품 validation 전 실패했다. 원시로그/receipt 보존 후 부모디렉터리 생성으로 교정하고 package002를 실행한다. 제품·기대값은 수정하지 않았다.

2026-10-15 completion 준비/진행기 교정: cost-probe build001은 새 진단 workspace lock 부재로 offline yanked dependency 선택에서 exit101/0시험이었다. 기존 pinned 진단 lock을 사용한 build002는6m07s/exit0. 승인host pilot001은 준비14연산과 첫라운드26연산을 통과했다. unfinished001은 동시 admission의 cache.lock 경합으로 준비단계 exit1/0연산; 별도 저장소만으로 승인캐시가 격리되지 않으므로 이후 순차 실행한다. full001은9연산 뒤 대기tick에 battle_state_revision만4→5가 된 것을 전체context 동일 assertion이 거절했다. 실제 전투시간·배치·자원·stress상태는 불변이다. command revision을 게임 진행으로 취급한 관측기 오류이며 후속 full002는 revision을 분리하고 checkpoint 전체와 게임context의 정지를 계속 검사한다. 세 실패 원문은 runs 아래 그대로 보존한다.

completion host 결과: full002의5라운드/실제최종IO 실패재시도/정산/중복거절/fresh복원 PASS. storage001의unfinished002·extract001(실제IO재시도)·retreat001(누적후퇴/실제IO재시도)와각fresh PASS. unrestrained001 준비는 slot9가 앞줄 slot8 완료 전 Unavailable이므로 중단했다. slot8 정상입장→수동후퇴로소비→slot9 cost12→공개Continue로교정한unrestrained002는5라운드/2배정산/fresh PASS. 정상/무억제 final eventlog의 적9명 모두HP90/135,공격10/15이며5wave전체대조다.

격리 경계검사 준비실패: probes-build001은 cwd가root였던no-run build/0실행이므로 경계PASS가 아니다. probes002는Side private import compile오류/0실행, import교정후probes003은0pass/4fail로고정map의Event→SavePoint→slot8 선행을 빠뜨렸다. 네사례공통준비를실제host에서검증된동일순서로교정한다. savepoint002 역시Event직후slot8을선택한같은선행오류이므로SavePoint자체IO/fresh검증을올바른slot5에서수행한다. 해당Goal의SavePoint검증을전투후같은SavePoint재진입이라고확대하지않는다.

확장pack001은27명수량에대해zone3칸밖에없어정상loader가GeneratedSetup에서거절했다(제품정상거절/준비실패). OpenHall두template의상단3행에기존spawn표식을확장하여각zone9칸을지원입력으로작성한package002와calibration/review/adoption은동일한도에서PASS. HP/공격/플레이어배치/시간한도는변경하지않았다. 실패원본과현재source차이는normal_inputs002의동일원본및expanded PREPARATION에연결한다.

추가경계최종probes005:4pass/0fail/0ignore,10.80초,exit0. 태세쿨다운·실제proc내부쿨다운/전투당횟수·동시각작성패배우선·실제무력화→다음입장사망/장착유물회수. probes004의두실패는gun의첫투사체미적중으로proc0회였고,보호목표옆경비가적차단을하지않아무력화되지않은준비전략이었다. 즉시타격shieldproc와지원전방배치칸으로교정했으며제품변경없음. savepoint003은실제IO실패재시도/fresh복원PASS. defeat001은공개소진/Continue의무억제pending후실제보호목표파괴·보상전부상실·기본XP10·소비품1회차감·중복Continue거절/freshPASS.

expanded003은3번째배치TP가9인데10을가정해거절됐다. exit1원문과BrokenPipe때남은/tmp trace를PROCESS_RECOVERY.json에연결해보존했다. 공개배치eligibility를유한20회내TP회복으로확인하는expanded004는26/27처치까지완료했으나마지막라운드누수적이목표를파괴했다. 경비/의무병/정찰은모두생존했고,마지막차단후경비의전방공격범위밖에목표타일적이남아있었다. expanded005는같은27명·동일pack·동일한도에서5라운드6초후경비를공개TacticalRelocate로목표옆(5,6)왼쪽방향으로옮긴다. 준비실패를성공측정으로세지않으며,직전4라운드성공과실제잔존대상/사거리/재배치비용근거를사용한다.

expanded005도26처치에서마지막적HP12를남기고보호목표가파괴됐다. 원시피해90−78=12에서정찰이두타격을필요로한것이관측됐다. Event에서이미보유한미장착u162_common_0의공개효과(flat20,percent20)를정찰에게장착하면한타격120−방어>90이므로,입력수량/기본능력치/한도를바꾸지않고이기존정상자산을사용한다. expanded006은이장착과3인배치를사용하며전방경비를유지한다. 최종라운드재배치전략은채택하지않았다. 최종비용프로필의추가유물1개도명시한다.

2026-10-15 최종 통합 집계: expanded006은기존보유유물추가장착·3인실배치로27명전원처치/5라운드38,311ms/정산PASS. expanded_fresh006은결과확인명령전에새프로세스로CombatResult상태·보상·맵소비·결과summary를그대로복원했다. 정상/확장공개Core재생각3회(총6회)는모든실행에서기록된승패/직원/인벤토리/라운드와일치하고drop후요청heap이정확히baseline으로복귀했다. 최대연산정상0.148930363초/확장0.310367126초, native work최대877,108/1,124,330. 과정전체RSSpeak는기록입력과allocator보유공간도포함하며각144,318,464/133,517,312바이트였다. Runtime 설정과승인한도는변경하지않았다. 연산측정은trace쓰기·disk publication·배포프레임SLA가아니며일반host의저장검증과구분한다. raw: completion001/cost001/RESULT.json 및각6개json+trace.

completion_analyze.py exit0:23개직접집계확인,승인host17프로세스/192연산,격리경계4pass,비용6회. canonical intake753중변경은PLAN/EXPERIMENTS두문서뿐이며제품·공식data·기존test·config·contract는불변이다. 실제4개journal IO실패동일요청재시도,4번Continue의시간/HP/UUID/배치/자원/효과일치,5wave전체무억제HP/공격력,직원손실·소비품완화·SavePoint·결과미확인fresh복원까지연결했다. 원시실패/준비교정은보존한다. 새독립감사는아직수행하지않았다.


최종화 준비: COVERAGE.json(수용10/차원10/IVA5/node2), BASELINE_REUSE.json, TEST_RESULTS_INDEX.json, COST_REVIEW.json, COMPLETION_REVIEW.ko.md와 재현 안내를 작성했다. 새 제품수정 없이 현재 결과의 증거를 결속한다. 최종 archive/정역복원 실행결과는 post-freeze GEX_10_RESULT_001.json에 기록하여 고정한 source 문서를 다시 변경하지 않는다. 비참여 완료감사는 미수행이며 기존독립감사003은 교정검토 이력으로 유지한다.
