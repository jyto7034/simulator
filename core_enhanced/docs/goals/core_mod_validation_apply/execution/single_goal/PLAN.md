# U11 실험실 검증에서 실제 mod 적용까지

## 2026-10-11 재개 현재 상태

사용자의 “마저 진행해줘”에 따라 진행 중 GEX10을 이어간다. 아래 GEX00 및 최초 역할 문단은 당시 인수 기록이다. 제품·독립 probe·cfgtest의 workspace 전체 target 개발 구성 root_member_compile004는 exit0다. 실제 Cargo tree 준비에서 발견한 plain/측정 mode의 Core transitive feature 불일치는 Lab 전용 측정 feature로 교정했고, 네 실제 graph 대응을 확인했다. 첫 release의 실제 rustup 환경 결속 실패를 보존·교정했고, 후속 일반 release002는 actual graph 결속과 함께 exit0로 완료됐다. 일반 진단 실행과 남은 교정·측정 빌드를 진행하며 기능 PASS는 아직 없다. 선행 compile 실패 원본은 보존한다. 기존 담당자 세션은 재개 당시 live 목록에 없어 동일 지정 모델로 `/root/u11_product_resume`(gpt-6.1-sol/xhigh, 제품·sole executor)와 `/root/u11_tests_resume`(gpt-6-astra/high, 독립 시험 작성)를 명시 배정했다. 성공한 spawn의 모델·effort 인자를 배정 근거로 삼고 이전 저자 표기는 보존한다. Root는 계획·실제 코드 검토·중요 품질 판단·E 문서를 유지하며 최종 비참여 감사자는 완료 경계에서 별도 배정한다.

Root는 PLAN/NOTES, actual compile001 결과·원시 출력, dependency001 receipt 및 현재 파일 목록/변경 범위를 대조했다. 첫 컴파일의 실패 원본과 baseline은 보존돼 있고, 이후 작성물은 미검증 현재 변경으로 인수한다. 가시 프로세스 snapshot에는 관련 runner가 없었으며 이는 현재 namespace 범위다. 새 담당자는 기존 소스와 마지막 receipt의 관계를 한 번 정리한 뒤 남은 granular coverage/bulk preparation·측정 worker/criterion·정상 launcher/server·save preflight/accepted replay를 같은 GEX10 묶음에서 완성한다. 역할 교체만으로 재작성·전체 검사·과거 상태 복원을 하지 않는다. 가장 최근 공개/저장 계약 판단은 EXPERIMENT_NOTES의 cache publication 및 accepted A→B 절이다.

## 현재 실행 인수

2026-10-12 최신: build021의 동일 source release plain/instrumented pair·하네스와 integration11시험이 완료됐다. build022에서3 owner의 대표 combined/isolated와 server base/단독64의15측정이 완료됐고64조합 native거부 및 native1000의 준비 전 실패2개는 보존했다. 별도32 A/B/AB는72개 완전 sample과 coverage34/34/68행을 얻었다. Root는 정확 측정 bytes의 `CMVA-authoring-dev-v2`(authoring clone1m)를 채택했고 config source015를 추가했다. 개발 criterion `cmva-dev-cadence-v2-20261012`의 phase별6초/heap1280MiB/RSS1536MiB/native230m을 결정했으며 근거는 NOTES에 있다. 같은 identity의 fresh adoption·실제 단독 PASS/조합 거부·host/save/cache/lifetime 통합 검증은 아직 남아 있다. native1000 실패는 공개API의 유한 cap 진단으로 유효 준비 후 fault 경계를 찾고 별도 동일 profile calibration/승인으로 확인한다. 최종 독립 감사 전까지 Goal 완료가 아니다. 아래는 이 상태에 이른 workload 교정 경과다.

source014의 불변 BattleEntrySnapshot 공유 교정과 독립 시험을 통합했고 build017의 선택 Core 회귀7개 및 manual003/expression002/causal005/resources005가 통과했다. 같은 funded fixture/profile의 workloads005에서 normal16/1은 통과, expanded500/8은 단일 elapsed60000ms 도중 native work10m 거부, pathological1000/16은9번째 배치에서 기존 DeploymentLimit으로 거부됐다. accepted time0은 롤백 결과이며 내부 실패 시점은 미확인이다. 기존 실패는 보존한다. 별도 양성 workload 후보는 명시적 `u11-fixed-cadence-60x1000-v1`로 roster16/500/1000과60초를 유지하고, 실제 정책상 배치 상한8을 지키는1/8/8명 및60개의 elapsed1000ms를 사용한다. 각 checkpoint의 running/정확한 시간과 개별 actor 인과·기하를 확인하고 동일 Trial 누적 자원·중복 응답 비용도 계량한다. fixture003의 자금160과 workloads005의 observation_only_clone_1000000 설정은 동일하다. 이는 CMVA-authoring-dev-v1의 work100m을 부모로 clone1m만 관찰하는 미채택 설정이며 native cap/default 변경은 없다. 이는 기존 단일60000ms/16배치 실패를 통과로 바꾸지 않는 별도 유효 입력이다. build018에서 외부 API 정적8사례와 기존8/9배치 원자성1시험이 통과했다. cadence001의 실제 이동/공격 준비 시간에 따른4명 참여 누락을 보존하고, 양쪽 합법 배치8칸으로 교정한 cadence002/v2는 세 규모 모두60회·running60000ms와1/8/8명 전원의 실제 A13+B17 인과를 실현했다. Root가 실제 RESULT/clock metrics와 대표 피해 인과를 읽었다. 같은 관찰 profile로 fresh release pair와 필요한 plain test harness를 만들고 실제 시간·heap/RSS·work 분포를 측정한다. clone1m의 정식 개발 설정 채택과 최종 criterion은 이 측정 뒤 판단한다. 최종 release pair/calibration·기준 채택·승인·실제 host/save·독립 감사가 남아 있다. 세부 증거와 영향 판단은 EXPERIMENTS/NOTES에 연결한다.

현재 직접 사용자 연속 권한에 따른 `goal-run`이다. root GS-CMVA-001, selected single member GOAL-CMVA-01, phase PHASE-CMVA-01, nodes AG-CMVA-ADMISSION/WP-CMVA-PUBLIC/WP-CMVA-VERIFY다. Core 우선·Unity 후속·개별 최종보스 제외를 유지하며 구현·독립 시험·준비된 검증·교정·비참여 전체 감사·후속 승인 로드맵까지 진행한다. 단계검토 중단 opt-in은 없다. P2G 산출 자체는 구현 권한이 아니다.

현재는 **GEX-10 전체 구현·검증 시작**이다. 아래 인수 문단의 pending은 작성 당시 상태이며 마지막 GEX00 출구와 NOTES가 현재 판단이다. P2G50 quality contract2 구조 검사 passed(exit0/failures[])와 P2G60 current chain을 Root가 인수했다. GS-CMVA001 canonical64dd01bc…, HA-CMVA0014f661439…이며 exact refs는 아래 원본에 있다. 제품/시험 mutation과 게임 검사0이고 mutation 전 recoverable baseline 및 shared contract 준비 인수 후 GEX-10에 진입한다. exact basis는 이 Goal의 [P2G-60](../../P2G_60_HANDOFF_001.json), [Goal Ref](../../P2G_50_GOAL_REF_001.json), [Goal](../../P2G_50_GOAL_SET.ko.md), [Freeze](../../P2G_30_FREEZE_001.json), [Phase Plan/receipt 결과](../../P2G_40_RESULT_001.json)를 exact accepted basis로 연결한다. 현재 baseline/구체 준비는 pending이며 후속 결과를 PASS로 간주하지 않는다.

적용 절차는 `docs/policy_to_goal_pipeline_revised_v2.md`, `docs/goal_execution_pipeline(1).md` GEX00/10/20 및 현행 직접 사용자 역할·변경 영향 검사·실측 위임이다. execution directory는 이 `execution/single_goal`이며 PLAN은 Root의 범위/준비/완료 판단, EXPERIMENTS는 실제 실행과 실패 원본 연결, EXPERIMENT_NOTES는 현재 기술 판단, STATE_INVENTORY는 실제 owner/key/수명 경계, 추후 별도 post_audit는 비참여 독립 판정이다.

## 선행과 복원 기준선

직접 선행은 `core_relic_causal_chains/post_audit/u10_current_002_pass001/AUDIT_CLOSURE.json`의 Complete/High/open0와 `core_first/execution/u10_closure_20261010_001/PUBLICATION_RECORD.json`이다. Root가 전체 감사 원문/closure와 발행을 이미 읽고 인수했다. 이전 U10 PLAN의 감사 대기 문구는 동결 당시 기록이며 최신 완료를 대체하지 않는다. U10의 모든 target/감사/실패 원본은 수정하지 않는다.

U10 target002 MANIFEST의543개 source/document와 SOURCE_AFTER archive를 가능한 exact 전이미지로 재사용한다. Sol은 제품 mutation 전에 현재 membership/bytes와 실제 archive member를 대조하고 격리 복원 가능성, HEAD/dirty/toolchain/lock 및 가시 runner snapshot을 `baseline/BASELINE_001.json`으로 보존한다. U11 추가 영향의 game_server/tools/새 canonical 문서 등 선행 archive 밖 기존 파일은 현재 원본만 새 보관하고 신규 파일 부재를 기록한다. P2G20 source archive의 exact member도 재사용할 수 있다. 관계없는 dirty를 성과로 귀속하거나 되돌리지 않는다. 전체 reset/clean/stash 및 archive 재귀 중첩을 하지 않는다. 가시 process0은 해당 namespace 범위로 한정한다.

초기 전체 suite 재실행은 요구하지 않는다. Goal의 Shared Validation Baseline Reuse Contract에 따라 U02 strict loader/identity, U03 first-fault/transaction, U04 strict persistence/paired intent, U07M 계량, U10 actual producer/causal owner의 유효 근거와 현재 관계를 인수한다. 새 delta의 동작과 hot-path 비용은 과거 PASS로 대신하지 않는다.

## 역할과 파일 소유

기존 확인된 역할·모델을 재사용한다. Root Astra/high는 actual source 검토·계획·공개 계약/저장/수명·품질 및 실측 기준 채택만 소유한다. `/root/u09_execution_continue` Sol6.1/xhigh는 제품 source/data/구현 문서와 **모든 준비된 검사 sole runner**다. `/root/u09_tests_continue` Astra/high는 독립 test/fixture/probe/oracle/comparator/판정 도구만 작성한다. 최종 `/root/u09_independent_audit029` Astra/high는 U11 계획/제품/시험 비참여 상태로 전체 감사를 맡는다. 과거 U10 감사 참여는 U11 독립성을 훼손하지 않는다.

Root는 P2G와 E의 PLAN/NOTES/STATE/EXPERIMENTS를, Sol은 product_parts와 baseline/실행 evidence 및 허용된 제품 파일을, author는 test_parts 및 독립 시험 파일을 소유한다. 제품 파일 안 cfg(test)/등록부도 whole-file 순차 소유로 바꾼다. 다른 영역이라도 같은 파일 동시편집은 없다. 실행 전에 관련 writer0를 확인하고 실행 중 제품/test/fixture/config/tool 입력을 고정한다. 성능 실행은 기능 검사와 분리한다.

## 구현 묶음과 구체 준비

[CODE_REVIEW](../../CODE_REVIEW.ko.md) T01–T07과 [Phase Plan](../../P2G_40_PHASE_PLAN_001.json)의 다섯 atomic WP를 따른다. registry/candidate→actual normal Lab 준비/coverage/calibrated metrics→issuer job/authenticated cache→actual local/server Apply→required-mod 저장 preflight가 하나의 admission 계약이다. 내부 순서로 구현하되 미교정 진단이나 일부 연결을 정상 Apply 완료라고 하지 않는다. 별도 PUBLIC과 VERIFY는 실제 구현을 소비한다.

Sol은 시작 전 실제 API와 기존 사용 예를 확인하고 product_parts에 필요한 만큼 구체 schema/API·상태 key/수명·auth/private-root/build identity·metadata version 전환·정상 preparation 및 test-facing API를 명시한다. 제안 타입 이름을 실제 존재로 간주하지 않는다. 새로운 public/authority/save 형식의 의미는 Root가 검토하고, 내부 자료구조/모듈/일반 compile/명확한 제품 오류는 자율 처리한다. 작은 교정마다 새 승인/계획을 만들지 않는다. 독립 author는 같은 frozen requirement와 actual API에 따라 입력/기대값을 독립 도출한다.

특히 (1) changed definition/site 자동 required coverage와 same-bucket actual producer 기록, (2) base+dependency individual 검증과 final-combination actual 검증 분리, (3) runtime safety와 Lab criterion·heap/RSS/native 분리, (4) exact issuer/engine/build/source/writer/profile/suite/seed/환경 cache key 및 authenticated restart HIT, (5) 정상 server의 raw loader 우회 제거와 active A/future B staged gate, (6) snapshot/backup/intent/profile/start 전에 원본 save preflight를 구현해야 한다. 상세 근거와 정책 원문은 upstream이 소유한다.

실제 기준은 아직 선택하지 않았다. GEX에서 normal/expanded/pathological 유효 workload를 격리 실행해 raw 분포·peak·drop·환경·여유 근거를 확보한다. Root가 현행 실측 위임 안에서 concrete versioned criterion을 채택하고, 독립 경계/one-over 및 실제 normal Lab→Apply까지 확인해야 한다. 500/1000은 유효 준비/실제 발동을 확인할 후보이며 gameplay cap이 아니다. 기존 16MiB native admission을 process heap 한도로 혼동하거나 old R05 수치를 복사하지 않는다.

## 검증·완료·반환

정확15요소/30수용 항목·20통합·10필수차원·3node가 대상이다. author는 mod/dependency/writer, normal/bulk coverage, calibrated metrics/parity, issuer/cache/revision/IO/restart, actual local/server/fault/generation, same-save 보호/복원과 lifetime/resource를 의미 있는 묶음으로 준비한다. 검사는 Sol이 단독 실행하고 nonzero/PASS/FAIL/skip/incomplete와 raw 위치를 남긴다. 실패 원인을 제품/시험/환경/의미로 구별해 같은 묶음에서 해당 담당자가 교정한다. 테스트를 통과시키기 위해 기대값·한도·workload를 약화하지 않는다.

최종 source 영향에 따라 관련 U02/U03/U04/U07M/U10 회귀와 공개/default-release/feature 계측/외부 프로세스를 선정한다. changed producer/clone/layout의 실제 비용은 해당 hot-path 검증으로 연결한다. 동일 입력 또는 명시 비영향 근거는 재사용하고, source 변경을 시험 파일 불변만으로 무시하지 않는다. 전체 테스트 미실행 자체는 실패가 아니며 필수 coverage 누락은 보완한다.

최종 actual evidence selection→predecessor impact→adaptation current 결속→coverage→Root 자체검증→PLAN/NOTES 최신화→exact target와 격리 정역 복원→nonparticipant whole Goal audit를 이어서 수행한다. Complete/High/open0 이전에 U11 완료를 발행하지 않는다. Root/Sol/author는 최종 감사자를 겸하지 않는다. 다음 승인 로드맵은 해당 독립 완료 뒤 계속한다.

현재 required policy/source blocker는 없다. 새로운 정책·공개 의미·state authority/lifetime·save compatibility·완료 조건 변화는 earliest P2G owner에 반환하며 기존 위임 안에서는 Root가 결정한다. 일반 교정·담당자 결과 대기·단계 전환은 사용자 승인 대기/종료 사유가 아니다. 실제 권한 밖 선택, 접근 불가능한 필수 consumer, 복원자료 손실 또는 안전하게 분리할 수 없는 소유 충돌만 해소 전 dependent 작업을 멈춘다.

## GEX00 출구와 GEX10 시작 — 2026-10-10

Root는 [baseline](baseline/BASELINE_001.json)과 실제 복원/currentness·raw intake 및 [구체 준비](product_parts/PREPARATION_001.ko.md)를 인수했다. 선행543와 추가16의559개 전이미지가 복원 가능하고 preparation 전후 source 불변이다. 정책/범위·필수 consumer·dirty/소유에 남은 blocker가 없으며 current P2G60와 U10 Complete/High/open0 prerequisite가 성립한다. 원본 source/test/build0에서 GEX00을 완료하고 승인된 GOAL-CMVA01 전체 GEX10에 진입한다.

제품 설계·시험 기대 결과는 EXPERIMENT_NOTES의 이번 구체 판단과 TEST_PLAN001을 따른다. hostV6/journalV2+legacy exact-approved reader 전환을 수용했고 Core11/12는 유지한다. actual backup/intent precedence, cache durable accept와 private-root, base authority/build identity는 해당 판단대로 구현한다. Sol은 product source/data/문서 및 준비된 실행을, Astra author는 별도 신규 test_parts/test파일/fixture/판정도구를 작성한다. shared 등록/inline tests 포함 기존 파일은 Sol writer0 후 author에게 whole-file로 이관한다. 각 실행은 모든 관련 입력 writer0에서 Sol 한 명만 수행한다. baseline 밖 기존 파일을 새로 바꾸기 전에 preimage 또는 기존 exact archive를 추가 결속한다.

## 2026-10-12 현재 교정·검증 결속

개발용 runtime v2 및 새 criterion v3(233m/6s/1280MiB heap/1536MiB RSS)의 실측 근거는 EXPERIMENT_NOTES에 있다. 이전230m 기준과 실패는 보존한다. build027의26명령은 관리9성공·기능4성공·8실패·5의존 미실행으로 닫혔으며 아직 Goal 완료가 아니다. Root가 실제 코드/원시 실패를 읽고 승인한 source016의 host non-Start generation 교정, cache/lifetime 시험 준비 교정, fresh context·Resource·literal entry 도구 및 배치 후 native 진단을 통합했다.

`u11_release_matrix003`와 `u11_calibration_observation002`는 완전 종료했다. 현재 own-context 관측과 명시적 v3를 별도 reviewed-inputs로 결속해 main22(관리9+실패·보류 기능13) 및 literal entry3 후속을 실행한다. 기존4기능 성공과 관련 유효 회귀는 비영향 근거로 재사용한다. Resource-only base의230m 초과는 실제 비용 판단으로 반환·처리했으며 새 v3에서 basePassed/changeIncomplete와32 A/B 양성·AB 초과를 실제 확인한다. 낮은 native-cap 진단은3회 모두 한정 no-gap으로 보존하고 기존 native1000 dependent 명령은 출시하지 않는다. 별도 dynamic-expression 콘텐츠의 실제 양성 Lab coverage→fresh 측정·승인→정상 actor fault/격리 검증이 남는다. shared actor 시험 변경으로 server harness가 달라지면 해당 owner 권한만 새로 결속하며 worker source 비영향이면 worker 재빌드를 반복하지 않는다. canonical workflow 문서와 전체 Goal 최종화·독립 감사도 남아 있으며 단계 보고나 비동기 대기를 종료로 취급하지 않는다.

현재25의 닫힌 결과는 관리11성공·기능8성공·시험 입력 실패3(13/14/18)·의존 미실행3(19–21)이다. Outer 수집/입력 봉인 완료 후 수용된 시험7파일(expression4/save1/Core lifecycle2)과 좁은 빌드 옵션을 통합한다. 같은 Core graph를 확인한 server lib 및 worker_boundaries/approved_lifecycle만 다시 빌드하고 Engine/worker identity 비영향을 실제 확인한다. Expression2 suite의4관측과 기존 default-cadence 새 server owner의1관측을 독점 실행한 뒤 각 suite의 수치를 판단·채택한다. 교정된 저장2개·실제 fault actor/worker·lifecycle와 dependent cache3를 필요한 기능 묶음으로 수행한다. 기존 standalone/Core/literal owner의 권한·실행 결과는 비영향 범위에서 재사용하며 canonical 문서 변경만으로 재빌드하지 않는다.

현재 상태는 build033의 완전 종료 결과를 따른다. 좁은 빌드와 identity 확인은 build032에서 끝났고, Core lifecycle와 cache3은 실제 통과했다. Expression 준비 오류 두 곳을 교정한 새 fixture/context로 worker의 실제 양성·typed 실패 전제를 먼저 확인한 뒤 영향받은 A/B/base 관측3개만 다시 실행한다. 기존 policy/default 관측과 수명·캐시 성공은 비영향 근거로 재사용한다. Default 새 server owner에는 실측을 검토해 같은 v3 기준을 유지했으며 fresh adoption→A/B/C→저장2개를 측정 구간 뒤 실행한다. Expression 기준과 actor 검증은 새 완전한 측정 뒤 진행한다. 최종 currentness·복원·전체 독립 감사는 여전히 남아 있다.

사용자가 지정한 `core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md`를 적용한다. 기존 역할·입력 고정·진행 중 실행은 유지한다. 모든 제품 변경과 검증 주장은 실제 코드/증거에 대한 Astra 검토를 거치고, 반복된 fixture 전제 오류는 공유 준비 경로까지 함께 교정한다. 장시간 실행의 예상 소요·진단/중단 신호는 기존 실행 기록에 남기며 운영 중단 기준과 제품 합격 기준을 구분한다. 같은 묶음의 일반 교정·재실행에 새 승인 절차를 만들지 않는다.

현재 후속 상태는 build035/036을 따른다. Default fresh 채택·A/B/C와 저장2개는 실제 통과하고 Root의 코드/결과 검토를 마쳤다. 이후 expression 시험만 바뀌므로 이 default 결과는 비영향 근거로 재사용한다. SelfUnit 피해가 유효 표적이라는 잘못된 시험 전제와 Activate 즉시 오류라는 시점 전제를 폐기했다. 합법 hostile target·정상 preview·성공한 Activate 다음 clock/HostTick의 typed 오류로 교정했다. Worker 최소 재빌드 후 직접 전제 검증을 먼저 수행하고 성공하면 server harness를 빌드한다. 새 server owner에서는 expression A/B/base/policy 네 관측을 새로 결속한다. Expression 기준 채택·정상 actor·최종화/독립 감사는 여전히 남아 있다.

후속 build039/040에서 hostile expression의 직접 전제와 전체 worker 경계·네 관측을 완료했고 Root가 실제 결과를 검토했다. 새 두 suite에는 기존 개발용 v3 수치를 유지하되 각 suite의 현재 server owner 관측으로 fresh 채택한다. 다음 묶음은 이 두 채택→A/B 실제 승인→normal expression actor다. Default036은 재사용하며 추가 Rust 빌드나 성능 재관측은 요구하지 않는다. 이후 현재성·coverage·격리 복원·전체 독립 감사를 계속한다.

041의 두 실제 채택과 A/B 승인은 통과했으나 actor는 배치 직후 공명0의 정상 준비 상태에서 멈췄다. 수식 fault 시도 전 실패이며 actor-only 정상 pulse 대기/동기 Pause 교정을 수용했다. 현재 후속은 server type check·한정 rebuild→새 owner004 context→실측을 포함하는 fresh 채택2/A-B/actor다. 기존040 worker/수치 검토와036 default/save는 비영향으로 재사용한다.

## 2026-10-12 build043 완료·Root 검토 및 최종화 진입

build042의 server 한정 빌드와 actual Engine/owner 비영향 대조가 완료됐다.004의9 reviewed 입력과 새 context를 runtime 시작 전에 strict binding했고 build043은 source `638eab584d31b2a98890a44bd9292126cf6cb1f0c03a812dd36060d535b89746` 불변·outer0으로 닫혔다. 실제 fresh calibration/adoption2개와 A/B 승인2개, normal actor 시험1개가 모두 통과했다. Root는 관리 결과의 현재 server identity/각 suite/기준/완전한 관측과 A/B 각 individual 및 최종 조합 passed를 확인했고, actor의 실제 공명0→1·다음HostTick typed 오류·first-fault gate·공개 원자성·B/다른 actor 격리·원본 저장/stop/recreate/Continue 의미를 코드와 원시 공개 증거로 검토했다. 과거041 readiness 실패는 보존한다.

남은 작업은 최종 source 현재성·정확 coverage/선행316항목 결속·실행 문서 정리·격리 정역 복원·비참여 전체 독립 감사다. 최종화 도구는 outside staged 상태에서043을 결속 중이며 아직 collector/freeze를 실행하지 않았다. 기존040 worker와036 default/save를 비영향으로 재사용하고 추가 게임/성능 검사를 임의 반복하지 않는다. Complete/High/open0 이전에는 U11 Goal 완료를 발행하지 않는다.
