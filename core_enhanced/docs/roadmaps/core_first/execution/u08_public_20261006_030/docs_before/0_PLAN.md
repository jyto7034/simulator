# U08 실행 PLAN — GOAL-CRNV-01

U08 현재 시험 이관029: compile.rs 단일 제품 교정을 source438/f0062433으로 인수했고 scoped fmt·offline/locked lib check가 통과했다. main Astra가 승인된 5파일/9selector 및 새 공개 CLI 재현을 단독 작성·집중 실행한다. Sol은 writer0/runner0으로 반환했다. 추가 compile work는 기존 한도에 계상하며 cap·workload·기존 기대값은 유지한다. 새 최종source와 harness005를 닫은 뒤 통합007을 실행한다. 전체 기능·성능·독립 감사·U09는 아직 미완료다. 아래028은 제품 교정 전 인수 이력이다.

기록: core_first/execution/u08_public_20261006_029/RECORD.json. 시험 설계와 실제 제품 교정은 evidence/r06_test_design001 및 evidence/r06_product_correction001을 따른다.

U08 현재 교정028: R06 선행 수용 조건 검토에서 공개 콘텐츠 사전 검증 누락을 실제 재현했다. AuthoredMembers의 Magnitude/Milliseconds 혼합 값을 Sum/Max로 읽는 식에 외부 Source Attack 참조를 추가하면, 단독 정적 식과 달리 validation이 성공하고 generation identity가 발급된다. 원래 publication 전 UnitMismatch 거부 요건은 바꾸지 않는다. source438/380e에서 7개 공개 CLI 조건 중 2개가 거부 oracle을 위반했다. 정적 거부2·동일unit 정상2·원fixture 정상1은 충족했다. 초기 관측 도구의 stderr 파싱 누락은 원본을 보존하고 실제 raw를 재해석했으며 제품 재실행 없이 구분했다.

통합006은 fmt와 Core default/test-fixtures 각각1958 PASS/0FAIL/0ignored를 보존한 채 교정 전 구간을 닫았다. 다음 묶음 전 정지했던 본인 소유 loop만 종료했고 outer receipt의 실제 exit는 -9다. 이는 위 두 suite의 실패가 아니다. source438/1322입력/외부archive2·필수8member의 before/after는 동일하며 나머지46기능+8성능은 미시작이다. 전체 기능·성능·최종화·독립 감사·U09는 아직 미완료다.

root는 기존 frozen R06 의미를 유지한 교정을 승인한다. Sol은 compile.rs 단일 제품 파일에서 해당 Aggregate의 직접 AggregateMember payload read와 작성 시 확정된 unit을 연결하는 최소 검증을 구현한다. catalog의 동적 평가 gate, runtime evaluator, 공개 API/AST/plan layout/버전은 바꾸지 않는다. main Astra는 새 시험 설계를 마무리한 뒤 별도 인계로 source 시험을 보강한다. 두 역할의 실행은 겹치지 않고, 다른 작성자는 HOLD다. 추가 compile work는 기존 한도 안에서 명시적으로 계상하며 cap·workload·기존 기대값을 완화하지 않는다. 새 교정 결과와 최종 시험 source가 닫히기 전 통합007/harness005를 실행하지 않는다.

교정 설계: core_first/execution/u08_public_20261006_028/CORRECTION_DESIGN.ko.md. 아래027/역할004와 이전 실행 안내는 당시 이력이다.

U08 현재 검증 인수027: 공개 RON Normal 실제 HP 누락과 nonfinite 입력 누락을 보강했다. source438/380e7408에서 scoped fmt와 Normal/E009/nonfinite exact3시험이 PASS이며 Normal452→272=180, Corrupt422→212=210, 비정상 입력16건을 실제 확인했다. 제품 변경 없이 u08_relics.rs에 이전14줄/이번200줄을 추가했고 기존 E00936 입력은 그대로다. 현재 전체 기능 PASS나 독립 완료는 아니다.

harness004의1322입력(기존122+실제 읽히는 추가1200), archive78+44+1200, 외부archive2개/필수8member, 현재source438·정확archive를 root가 검토했다. 49기능+8성능의57필수 역할을 새006으로 실행한다. source/input writer0이며 Sol /root/u08_product_runner_resume가 실제 before 결속 후 단독runner를 맡는다. 두 Astra 작성자의 후속 문서 작업은 실행 입력 밖에서만 허용한다. 현재 fullaccess/network enabled/approval never 지시에 따라 sandbox_permissions 인자는 사용하지 않는다. 과거 EPERM·승인 요청 방식과005 부분 실행은 원본 이력으로 보존한다.

정확 실행 인계는 evidence/verify_harness_handoff004/HANDOFF.json, root 인수는 core_first/execution/u08_public_20261006_027/RECORD.json, 역할은 u08_roles_20261006_004/RECORD.json이다. 소스·도구 before/after 실제UTC, 모든 비성능49receipt를 포함한 PC/R05 각각의 실제 quiescence, fresh release/debug 선행 빌드와 binary 결속을 유지한다. 최종 순서는 selection→191+31 impact→adaptation→32문장/10차원/3node/16seam coverage→자체검증→PLAN/NOTES→동결/격리복원→전체U08 비참여 Astra 감사다. U17 실제 부활촉매 교차는 미구현 후속 경계이며 16.2 독립 감사는 대기다.

2026-10-06 역할·환경 현재화004: 현재 환경은 danger-full-access/network enabled/approval never이며 sandbox_permissions 인자는 사용하지 않는다. 이전 승인 요청 방식은 당시 이력이다. /root/u08_product_runner_resume(gpt-6.1-sol/xhigh)는 제품·통합·성능 실행 담당이며 dependency_fetch001 환경 준비와 docs_checkpoint026을 닫고 현재 HOLD다. /root/u08_test_author_resume(gpt-6-astra/high)는 공개 Normal 실제 HP 검증을 u08_relics.rs 단일 파일에 추가하고 승인된 scoped fmt/집중 실행·일반 시험 교정을 맡는다. /root/u08_harness_author_resume(gpt-6-astra/high)는1322입력 보완과 supplement archive 준비를 닫고 최종 source 결속을 HOLD한다. root는 계획·읽기·문서·조정만 맡고 독립 감사자는 아직 미배정이다.

통합005는 이전 source873의 fmt/Core default1957 PASS까지만 유효하며 후속 검증 누락을 보강하기 위해 실행 interval을 중단·보존했다. 새 source9af에서 nonfinite16case 집중 검증이 통과했고, 현재 PE09.2의 full-loader Normal actual-HP 누락을 별도 시험으로 보강 중이다. 새 시험 종료 후 최종 source를 확인하기 전에는 harness004/통합006을 실행하지 않는다. 기존 E009 export와 실패·중단 원본은 보존한다. source 입력 편집/집중 실행 담당은 main Astra 한 명이고 Sol 통합/성능 runner는0이다. 상세 역할 기록은 core_first/execution/u08_roles_20261006_004/RECORD.json을 따른다.

2026-10-06 · 아래 첫 부분은 초기 E1/E2 검증 인수 이력이다. 실제 진행 상태는 “현재 우선 작업”과 최신 기록을 따른다.
현재 마지막 E1 성공 source435 / `984f8e93da698a55baa2f4c41f53fd9934dc9f3a884e4e522669c552f1bbf53b`에서 E009 전체1/1 PASS다.
정상 보상·장착·SavePoint/Event/Load·fresh OS Continue 이후 실제 Snapshot Const100 스킬 피해210이 HP422→212로 소비됐다. Original/정상grant fragmentcopy/employee/generation/segment raw·calculated210을 명시 검사하고 trace on/off의 full DamageFact·query를 대조했다.
E007의 clamp92→0 성공과 E008의4초 무대상 실패는 보존한다. E009는 외부 fixture의 대상maxHP170→500, 준비8초이며 공식 콘텐츠/제품/cap은 그대로다. 최종 public_trial_cli009의35개 산출물 해시를 확인했고 실제 release CLI 실행은 후속 Sol 범위다.
Continue owner2·allocation2·durable2/UUID1/Eventowner4·행동소유75 검사1 PASS를 회수했다. 새 codec fixture의 두 nonempty set/current·SavePoint canonical 동일성과 누적4,518,759 exact/6단계 one-short 거부·gameplay roots 불변을 검증했다. 이는 실제 heap 계측을 대신하지 않는다.
E2 literal002는 관측 off 실제 두 직원144/210의 1/1 PASS다. 이어 stage2001은 literal 1 PASS / 실제 carrier 4 FAIL이었다. delivery 재결속에서 private relic_numeric이 누락되는 제품 원인을 확인했다.
Sol의 두 파일 교정 후 Astra private004 2/2, 동일 입력 world stage2002 5/5, 관련 relic_amount12/12, 기존 resource projectile002와 fixed001 각1/1 PASS다. source438/939a61e13b381c034c5ecf1efcb80a79091038592e46df090107aa60ddbf47a4의 검증 경계이며 이후 live 편집 성공을 뜻하지 않는다.
basic001 parameterized1/1은 근접/투사체 × Normal/Corrupt의 Auto·ExtraAttack과 base12→38/78 once를 검증했다. epoch0051/1 및 carrier complete0017/7은 source438/0df25480ef7cb5c787f37488f85c9c5d6069b156637132a19c703343d6d15d25에서 통과했다. 정상 channel 만료1601ms 뒤 원 Auto2500ms는 취소되고 복제 대조군은38피해를 냈다. epoch001–004 fixture 실패 원본은 보존한다.
초기 Astra의45개 검증 도구·의존 입력 인수 당시 모든 입력 writer를 HOLD로 전환했다. 첫 integrated002는 제품2파일 import/module 정렬 차이로 fmt 실패·시험0이었다. scoped형식 교정 후 source438/fa734235cef0c4b9ab73c9acac1785d0cdc9e9dd6123d70ff36efd0fe42b18a0에서 fmt003이 통과했고 당시 Sol에 integrated_matrix003→verify_remaining003 기능 묶음을 위임했다. integrated003의 이후 실패·교정과 현재 재개 조건은 아래 최신 기록을 따른다. [형식 비영향과 새 실행](../../../../roadmaps/core_first/execution/u08_public_20261006_014/RECORD.ko.md)을 따른다. fresh release와 debug measurement의 source/binary 결속을 선행하며 실제 playback26/lifecycle10/host4×2도 포함한다. [도구 인수·실행 계약](../../../../roadmaps/core_first/execution/u08_public_20261006_013/RECORD.ko.md)을 따른다. 상세 [carrier 교정 회수](../../../../roadmaps/core_first/execution/u08_public_20261006_011/RECORD.ko.md)와 [basic·epoch 회수/다음 범위](../../../../roadmaps/core_first/execution/u08_public_20261006_012/RECORD.ko.md)를 따른다. root는 제품·시험 코드를 대신 작성하지 않는다.
전체 U08 완료가 아니다. 공개 release CLI, 최종 기능·성능 및 evidence→impact→adaptation→coverage→자체검증→동결/복원→독립 감사가 남았다. native exact-cap 검사는 admission enforcement 증거이며 독립 heap 상계를 대신하지 않는다.
Core StartNewGame은 NotStarted만 허용하고 replace_active_run은 host 확인과 구분한다. 실제 사망 정산은 terminal이고 CompleteCombatResult는 acknowledgement다.
이전 numeric on33/off31·behavior11·초기 PC는 과거 경계의 유효 범위에서 보존하며 현재 변경 영향에 따라 결속한다.
상세 계약은 [공개 API](../../../../roadmaps/core_first/execution/u08_public_20261006_001/RECORD.ko.md),
[OwnedHashMap 정정](../../../../roadmaps/core_first/execution/u08_public_20261006_002/EVENT_OWNER_BOUNDARY_002_CORRECTION.ko.md),
[Event 현재 층 수명](../../../../roadmaps/core_first/execution/u08_public_20261006_003/EVENT_LIFETIME_DECISION_001.ko.md),
[역할·terminal 계약 정정](../../../../roadmaps/core_first/execution/u08_roles_20261006_002/RECORD.ko.md),
[E1·Continue 회수 및 E2 범위](../../../../roadmaps/core_first/execution/u08_public_20261006_009/RECORD.ko.md)를 따른다.
과거 실패·원 작성 이력·감사·target·HANDOFF는 보존한다. 프로세스 가시성은 sandbox namespace에 한정된다.

## 현재 우선 작업 — 2026-10-06/026 · 승인 실행으로 통합 재개

2026-10-06 승인 실행 담당 후속 배정: 기존 /root/u07_implementation에는 실제 approval never 지시가 남아 있어 읽기 preflight 후 HOLD로 반환했다. 새 **/root/u08_integration_runner (gpt-6.1-sol/xhigh)**를 지정모델로 실제 배정했고, 새 developer 지시에 require_escalated/justification 승인 요청이 허용됨을 확인했다. 새 Sol이 통합/빌드/성능 단독runner를 인수한다. 기존 제품 작성 이력은 옛 Sol에 귀속하며 역할 변경만으로 증거를 재작성하지 않는다. 두 Astra작성자는HOLD, 전체 제품/test/harness 입력writer0을 유지한다. 상세 역할 원본은 core_first/execution/u08_roles_20261006_003/RECORD.json이다.

사용자 권한 변경 후 root의 require_escalated 실행에서 AF_UNIX·AF_INET 생성/bind가 둘다 PASS했다. 기본 sandbox 제한은 유지되지만 승인된 외부 실행으로 필수 소켓 검증을 진행할 수 있다. [재개 인수026](../../../../roadmaps/core_first/execution/u08_public_20261006_026/RECORD.ko.md)을 따른다. source438/8734111c와 harness003 입력122개는 변경0이다. Sol이 실제 before/currentness·writer0·외부archive를 결속한 뒤 승인 요청 경로로 통합005→remaining005를 단독 실행한다. Astra 작성자둘은 HOLD이며 모든 입력writer0이다. 전체 기능·PC/R05·최종화·독립감사는 아직 남아 있다. 아래025는 권한 변경 전 실제 장애 이력이다.

## 2026-10-06/025 · 권한 변경 전 환경 장애 인계 이력

통합004의 Core default는 1,872 PASS / 12 FAIL로 종료했다. catalog·definition fixture 6건과 capability 기대 5건은 Astra가 7개 테스트 파일에서 교정했고 최종 source438/8734111ce349bc36de5534aff831f89ccf8c9034b7e802e120d87cee5d0e248b에서 11개 exact selector가 각각1/1 PASS다. Sol의 post_correction_fmt001도 PASS/입력 변경0이다. 남은 UnixListener bind 실패는 환경 진단에서 AF_UNIX bind EPERM·AF_INET socket EPERM으로 확인됐으며, 현재 approval never 환경에서 필수 소켓/host 검증을 완료할 수 없다.
[교정·장애·현재 source·도구 인수025](../../../../roadmaps/core_first/execution/u08_public_20261006_025/RECORD.ko.md)와 [최신 인수인계](../../../../roadmaps/core_first/execution/HANDOFF_LATEST.ko.md)를 따른다. fresh harness003의122입력·source438 및 외부 archive를 인수했으나 미래005 실행은0이다. G7 CLI·전체 기능/성능·최종화·동결/복원·독립감사는 미완료이며 U09는 미진입이다. 모든 제품/test/harness writer와 runner는0/HOLD다. 사용자 중지나 완료 판정이 아니라 실제 환경 장애 인계이며 기존 연속 권한은 유지된다. 합법적으로 소켓 검증이 가능한 환경이 확보된 뒤 현재성/가용성을 다시 확인하여 새label로 이어간다. 아래024 실행 예정 안내는 당시 이력이다.

## 2026-10-06/024 통합 실행 인수 당시 이력

R07 기본stack 교정과 G1/G4/G2/G3/G5 성공을 보존한다. G6는 최종source438/ce0e0f2f44a1f9ab6665acbc78f3f8903f0da46c01d9afe56b8b230e716ec79a에서 finite3회 lifetime0021/1 및 carrier12/12 PASS다. G7 공개정의검사도구와새harness122파일/외부archive입력을인수했고실제G7 CLI성공은아직대기다. [G6/G7 인수·통합004 실행 계약](../../../../roadmaps/core_first/execution/u08_public_20261006_024/RECORD.ko.md)을 따른다.
모든입력writer/runner0에서Sol의실제before기록후 integrated_matrix004→verify_remaining004를시작한다. Astra작성자둘은HOLD이며finalizerprep002는준비상태로유지한다. 기능종료뒤PC/R05순차실행과실제quiescence/전후입력동일성기록을남긴다. 실패003·기존준비원본은보존하고새label을사용한다. 이후현재evidence선정/impact/adaptation/coverage·자체검증·동결/복원·별도독립감사가남았다. U17 실제부활촉매교차시험과전체U08완료는아직주장하지않는다.

## 요청·역할·현재 인수

2026-10-05 현 사용자 연속 재개 요청에 따라 Core-first 전체를 계속한다. U07과 삽입 U07-M을 독립 완료했고 다음 U08이다.
root는 계획·설계·수용 기준·검증 목적·전체 조정만 담당하며 제품·테스트 코드를 대신 작성하거나 최종 독립 감사를 겸하지 않는다.
기존 `/root/u07_implementation`(**gpt-6.1-sol/xhigh**)은 제품 코드·제품 데이터·구현 문서·제품 및 통합 포맷/빌드·통합/회귀/성능 실행·원시 결과 수집·제품 교정을 담당한다.
새 `/root/u08_test_author`(**gpt-6-astra/high**)는 테스트 설계·코드·fixture·공개 probe·기대값·판정 도구·comparator와 테스트 교정을 담당한다. 승인 묶음의 scoped 포맷·작은 컴파일·집중 실행과 일반 교정은 고유 label/입력고정 아래 자율 반복하며 매번 재승인을 받지 않는다. 제품 변경은 구현자에게 반환한다.
구현자는 이제 테스트/fixture/oracle/comparator를 직접 작성하거나 교정하지 않는다. 과거 Sol 작성 테스트는 원 작성 이력으로 보존하며 역할 변경만으로 전부 다시 작성하거나 재실행하지 않는다.
동일 파일은 테스트 모듈 등 영역이 달라도 동시에 편집하지 않는다. 실행 전 모든 입력 writer가 편집을 멈추고 종료/receipt 회수 뒤 다음 변경을 한다.
기존 감사자 `/root/u07_independent_audit`는 U08 계획·설계·구현·테스트 작성에 참여하지 않았다. 최종 동결 시 실제 가용성과 독립성을 재확인한 별도 **gpt-6-astra/high**에게 전체 GEX20을 맡긴다.
테스트 담당자는 자신이 작성한 검증의 결과와 한계를 정리하지만 최종 독립 감사자가 될 수 없다.

사용자는 자체검증·독립감사·finding교정·재감사와 후속 roadmap까지 명시 위임했다. 단계별 사용자 재승인을 요구하지 않는다.
중간 단계검토 중단은 이번 U08에서 선택되지 않았다. 일반 내부 교정은 구현자가 자율 처리한다.
정책/공개계약/상태소유/완료기준/핵심설계전제 변경은 코드 근거·영향·대안을 root에 반환하며 자동 사용자 승인대기로 취급하지 않는다.
사용자중지·전체승인범위완료·현재권한환경에서 해결불가능한 실제장애 외에는 milestone에서 최종 종료하지 않는다.

[HA-CRNV-001](../../P2G_60_HANDOFF_001.json), [GS-CRNV-001](../../P2G_50_GOAL_SET.ko.md),
[FPF-CRNV-001](../../P2G_30_FREEZE_001.json), [PP-CRNV-001](../../P2G_40_PHASE_PLAN_001.json),
[PVR-CRNV-001](../../P2G_40_VALIDATION_RECEIPT_001.json)이 exact basis다.
[CODE_REVIEW TD01–06](../../CODE_REVIEW_001.ko.md)가 실제 owner·interface·lifetime·실패·저장·권한·산술·검증 설계를 소유한다.
코드근거121파일/source003이고 정책16군/검증10차원/단일phase/최종3실행node다. 모든 frozenrefs digest/currentness를 GEX00에서 재확인한다.
적용 지침은 P2G2026-10-04-r8/GEX2026-10-04-gex-r8/usagev2.1.3/quality2.

현재 선행 product398파일 aggregate `f2144e9a4c5ddeb41245799294a75b41de9d843018cb2166f711b7f07821cda8`.
[PREDECESSOR_INTAKE](../../PREDECESSOR_INTAKE_001.json)는 U07-M target/u07m_001의4463members와27감사artifact를 다시 검사하고
완료publication의현황문서2개만 비영향 delta로 인수했다. 이번 U08착수 publication은 별도 신규기록으로 인수해야 한다.
원역사/실패/target/감사는 그대로 보존한다. 새로운 문서가 생겼다는 사실은 과거 product verdict의 소급 수정이 아니다.

## GEX-00 — 구현 전 실제 기준

1. 관련 AGENTS/current processes/actual source·dirty/untracked/index/worktree를 확인한다. 이전 수행label을 이어쓰거나 도구가 있다는 이유로 완료를 가정하지 않는다.
2. 현재 product/source·필요 config/data/test/tools·top-level lock/dependency와 input identity를 명시적 목록으로 보존한다.
   exact bytes/archive,변경/추가/삭제membership,git status/index/worktree차이를 기록하고 사용자파일을 reset/clean/stash하지 않는다.
3. U07-M immediate independent closure `post_audit/u07m_20261006_001/AUDIT_CLOSURE.json` SHA
   `a9fac11bb702a2e484daeb8c34ee04e39d3c30d553cb1c996611c1b7aca4dbb8` 및 manifest SHA
   `50c3c228d2c8f0915ab6133014bcf7783bb44d3b8880a8ba7c784ffe130a5733`를 인수한다.
   current191선행조건(181+U07M10)과별도31R06을 exact owner·receipt·baseline oracle로 연결한다.
4. initial 전체baseline은 유효한U07M 선정54receipts를 exact reuse한다. phase시작만으로 전체테스트를 되풀이하지 않는다.
   source변경 영향이 생긴 seam은 기존성공으로 자동충족하지 않는다. 정책 의미는 유지해도 layout/capture/source 변경은 비용회귀를 일으킬 수 있다.
5. GEX00 actual record와 experiment/notes/state inventory를 만든 뒤 같은연속권한으로 GEX10을 시작한다.

## GEX-10 — 구현 순서와 먼저 확인할 위험

최종node: **AG-CRNV-RUNTIME → WP-CRNV-PUBLIC → WP-CRNV-VERIFY**. RUNTIME 안의 DEFINITIONS/OWNERS/NUMERIC/PERSISTENCE는
하나의 정상 공개경로 cutover다. 내부 작업 순서를 정의→copy/transaction→numeric/authority→lifecycle/save로 나눌 수 있으나
schema만 등록한 중간 상태를 지원완료로 게시하지 않는다. 별도 Master/GEX30은 없다.

- TD01: relic immutable catalog·copy/3slot owner와 normalgrant/equip/transfer/death를 기존 paired/staged owner에 연결한다.
- TD02: cost+variant 한번 commit,실제 currentEvent target선택 권한/decline,촉매1 vs무료0,분쇄확인/가루/판매거부.
  새equipmentdust는 향후U15의단일run자산이고 `fragment_dust` legacy를 환산하지 않는다. 전체 상점/경제는 선구현하지 않는다.
- TD03/04의 조기 verticalslice: 두직원같은SkillId,100+20+(20%+30%)=180,관측offliteral/Expr/basic/queued/projectile.
  catalog ptr_eq와generation/action/provider를 보존하는 immutable overlay/binding;기본공격 별도 typedaddress.
  모든피해 helper에 일반증폭을 붙여 basicHeal/BuffTick/Environment/반사/bonus/R09에 잘못 적용하거나 이중 합성하지 않는다.
  기본aura반영후base→reliconce,원 source_attack 의미보존. base typed/rangeguard·effectivechecked합성·최종narrow와queryreplay를같이연결한다.
- **성능 조기확인:** 유물없는common source inline/size/retained/clone path를 작게 유지한다. 새PCisolatedlabel로 현재소스를 비교한다.
  실패를8MiB상향·workload축소·oracle/comparator완화로 해결하지 않는다. 비싼 최종 통합 전에 구조원인을 줄인다.
- TD04 penalty는시험용 신규RON의typedflatDefense/MR 사례로 실제 소비를검증한다. persistent/certifiedstats가아닌
  finalbattle scale/exact_override 뒤단계에적용하고certifiedpower불변·permutation동일·actualdamage위험을확인한다.
- TD05: GameCoreState/EconomyGrant/checkpoint/entry/active_run/Continue/death/runreset/clone의모든capture/restore/validator를함께수정한다.
  실제durable경로는VERSION11 continuation이며defaultfield추가만으로canonicalchecksum호환이보장되지않는다.
  oldfield없음정상·malformedpresent·checksum위조·generationmismatch를별도검증한다.
- PUBLIC: 외부newIDRON/제3자재조합→정상CoreApplication/trial/CLI→grant/equip/variant/전투/query/save. privateprobe만으로완료하지않는다.

구현자는 모듈명·private타입·함수분리·컨테이너를 자율선택하고, 테스트 담당자는 승인된 검증 목적에 맞는 입력·기대값·시험 배치를 정한다. 상세impl preflight가P2G전제를깨면root에반환한다.
정책의미를그대로구현할내부수정마다승인을요구하지않는다. 신규실행도구는현Goalnamespace/newlabel만사용한다.

## 검증과 증거

[Element Coverage01–16](../../CODE_REVIEW_001.ko.md)와Goal10차원을 실제nonzero test/공개입력/expected-observed에매핑한다.
16새군+191선행조건=207주조건과별도31R06이다. 요소별2개acceptance 문장을묶음ID만으로누락하지않는다.

필수마지막통합은변경영향에따른 current소스에서수행한다. 기존지침commandset: Corecheck/alltargets/test/test-fixtures,
event_log_checks,releasebins,clippy의기존allow2개만,fmtcheck. U07M runtime-observation과hostdefault/observation/measurement/heap,
publicreleaseCLI·traceon/off/nativefault/값/source/save/clone/계량·PC/R05의영향소비를추가한다.
전체functionalworkers가끝난뒤성능을재측정하며공유호스트 timing한계를밝힌다. 20분stress 제외는유지한다.

PC원576/36조건의31성공5명시거절과각completion/code/site/cap/gameplayoracle·hardguard를보존한다.
requested bytes의설명가능차이는원시전후/원인으로공개하되새예산거절과구별한다.
R05는원cap9,030,188의concurrentpeak 단독초과만기존경고다. U07M raw101 peak9,669,160/+638,972를새PASS로소급하지않는다.
다른hardguard/동작/결정론실패는예외가아니다. 새로운성능결과는새exactinput/receipt로선정한다.

- runner는원command·selector·count·exit·source/input/toolchain/환경과raw를보존한다.0시험/skip/중단/환경실패는PASS아님.
- 과거artifact생성스크립트를그대로재실행하지않는다. hardcodedlabel/output/source/adaptation참조를실행전에검토한다.
- 검증입력에mutation하지않는다.수정이필요하면원run을닫고sourcecheckpoint와새label로이어간다.
- current evidence선정→predecessorimpact→adaptation새결속→coverage→GEX10종료→PLAN/NOTES최신화→target동결순서.
  coverage 전에impact/adaptation이current여야한다. 소비도구는실제successor파일을읽게한다.
- before/afterarchive와정/역patch를격리경로에서복원검증한다. 실제workspace에역patch를적용하지않는다.
  이전targetarchive는exactexternalref로보존하고불필요재귀중첩하지않는다.
- 전체목표16군/10차원/191선행+31R06·3node/16integrationseam와sourcecurrentness를동결한다.

## 감사·후속

GEX10자체검증완료는GoalComplete가아니다. root가frozen정확대상을인수한뒤비참여Astra/High전체GEX20을가동한다.
감사중target/source/input/요구mutation은멈춘다. finding은기존Sol/xHigh교정→새증거/target→독립Astra재감사를반복한다.
Complete/High/open0와exitcurrentness뒤root가roadmap을새publication으로갱신한다. 그뒤U09를현재코드로상세화한다.

초기 국소 owner/world owner·catalog 및 수치 검증 증거는 실행 이력으로 보존한다. 현재 검증 상태와 남은 범위는 문서 상단 및 최신 실행 기록을 따른다.
