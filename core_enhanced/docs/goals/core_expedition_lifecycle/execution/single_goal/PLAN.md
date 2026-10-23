# U12 튜토리얼·원정 수명 실행 계획

## 2026-10-13 단독 실행 재개

최신 사용자 지시와 현재 연속 지침을 적용하여 이후 구현·시험·실행·교정·검토는 한 실행자가 직접 수행하고 서브에이전트를 사용하지 않는다. 아래 기존 역할·인수 문구는 당시 이력이다. 수용 조건과 비참여 감사의 필수성은 소급 변경하지 않으며, 독립 판정이 필수인 경계에서는 가능한 구현·검증을 마친 뒤 정확한 미충족을 보고한다. G11 provenance 관련 Core 검증과 heap9회, 공개 실제 패배→명시적 포기→새 원정 분기는 완료했다. 2026-10-13 최종 지원 worker pair·서버 harness 결속, 각 실제 host의 normal calibration/adoption, 공개 Return/loan/story, 현재 사본 부재 경계, 일반·full60의 별도 프로세스 Continue까지 통과했다. 남은 GEX10 작업은 finalization001의 정확 source/coverage 결속과 target/u12_current_002의 격리 정역복원·동결이며, 성공 여부는 해당 MANIFEST/RESTORE_VERIFICATION이 소유한다. 비참여 GEX20은 미실행이고 U12 전체 완료·U13 진입은 아직 허용되지 않는다. 실제 상태는 HANDOFF_LATEST/EXPERIMENTS가 소유한다.


2026-10-12 현재 `goal-run`의 root는 GS-CEXL-001, 선택 member는 GOAL-CEXL-01, Phase는 PHASE-CEXL-01이다. AG-CEXL-LIFECYCLE / WP-CEXL-PUBLIC / WP-CEXL-VERIFY를 모두 이 single Goal이 소유한다. 사용자 직접 연속 지시와 [개정 지침](../../../../roadmaps/core_first/execution/CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md)이 구현·독립 시험 작성·준비된 실행/교정·비참여 전체 감사·다음 승인 로드맵까지 별도로 허용한다. 단계검토 opt-in이나 사용자 중간 승인 대기는 없다. P2G 작성 결과 자체를 실행 권한으로 간주하지 않는다.

현재는 **GEX-10 부분 제품 구현·독립 시험 작성의 잔여 연결과 검증 재개**다. P2G40/50과 GEX00을 완료했고 로드맵 검토의 문서·책임 보완을 반영했다. 사용자 재개 지시로 검토 중지를 해제하며 기존 Sol 제품/단독 실행과 독립 Astra 시험 작성을 유지한다. 재개 입구 compile/build/test는0회이며 실제 후속 결과는 EXPERIMENTS가 소유한다. 아래 GEX00 출구가 구현 시작 근거를 소유한다.

## 정확 입력과 문서 소유

- [HA-CEXL-001](../../P2G_60_HANDOFF_001.json), canonical `sha256:7144b612774f03d2096b9152ac62a861132445e7d144da8cdcedf9f862d19826`.
- [GS-CEXL-001 ref](../../P2G_50_GOAL_REF_001.json), canonical `sha256:92913e65bda4d5dee71ae48385b23e66bba8f9139502acffcfb11cd115f97362`; [전체 Goal](../../P2G_50_GOAL_SET.ko.md).
- FPF-CEXL-001 `sha256:4a31856008428324110bf614154f44982e2758c32b3a7cff8fe08c13bbddb705`; PP-CEXL-001 `sha256:70669c818fcec1ccba714a22ad66106f5efbf063add00ef1e017e97c486d3aba`. 정확 상위 chain은 Handoff Result Composite Basis가 소유한다.
- [P2G40 결과/receipt](../../P2G_40_RESULT_001.json), [P2G50 quality2 결과](../../P2G_50_RESULT_001.json). 두 validator의 actual raw·exit0·failures[]는 별도 실행 기록에 있다. 구조 PASS는 제품 PASS가 아니다.
- 적용 절차는 `docs/policy_to_goal_pipeline_revised_v2.md` 및 `docs/goal_execution_pipeline(1).md` Reader/§5–6/GEX00/10/20, 직접 사용자 현재 역할/검증 지시다.

정확 execution directory는 `core_enhanced/docs/goals/core_expedition_lifecycle/execution/single_goal`이다. 메인 Astra high는 PLAN·EXPERIMENT_NOTES·STATE_INVENTORY 및 actual product/test code·raw 결과 품질 판단을 소유한다. EXPERIMENTS는 승인된 실행의 실제 결과를 연결하며 Sol의 product_parts 실행 원문을 복제하지 않는다. Sol6.1 xhigh는 제품·공식 live data·일반 사용 계약 문서와 준비된 build/test/runtime의 단독 실행, 별도 Astra high는 시험/fixture/public probe/oracle/comparator 작성이다. 기존 독립 시험 작성자 `/root/core_main_astra/u12_tests`의 원본·작성 이력을 보존하고, 재개 스레드 제한으로 같은 지정 `gpt-6-astra/high`인 `/root/u12_tests_resume`가 미완료 시험 작성을 인수했다. 최종 감사는 현재 계획/제품/시험 비참여 Astra high가 별도 post_audit 경로에서 수행한다. 같은 모델이라도 참여자는 final auditor가 될 수 없다.

첫 제한 검사 뒤 기존 제품 thread도 재개 제한으로 사용할 수 없어 같은 지정 `gpt-6.1-sol/xhigh`인 `/root/u12_product_resume`가 부분 소스·작성 이력·실행 원본과 남은 제품/runner 역할을 인수했다. 모델이나 책임을 바꾸지 않는다. 하위 agent 승인 UI 미노출에 대한 한정 운영 중계는 [실행 기록](EXPERIMENTS.md)에 실제 호출자를 구분해 남기며, 명령·검사 선택과 raw 인수는 Sol이 유지한다.

새 test_parts 파일은 test author가 소유한다. 기존 product source의 embedded test/등록부는 파일 전체를 writer0에서 순차 이관한다. 실행 중에는 관련 source/data/test/fixture/config/tool writer0을 확인하고 동일 입력에 runner 하나만 둔다. doc-only 진행은 실행 입력에 영향을 주지 않는 범위에서 가능하다. 제품/시험 양쪽이 같은 파일을 동시에 쓰지 않는다.

## 선행과 dirty/복원 기준

직접 선행은 U11 `core_mod_validation_apply/post_audit/u11_current_001_pass001`의 Complete/High/open0다. 메인이 원문 감사와 current closure를 이미 인수했고, [한 번의 resume 확인](../../RESUME_20261012_001.json)에서 runtime manifest 4,678파일이 현재 source와 모두 일치했다. HEAD는 `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`. 가시 프로세스 조회는 sandbox namespace 범위이며 OS 전체 실행 부재 주장이 아니다.

GEX00에서는 기존 U11 exact SOURCE_AFTER archive/manifest를 재사용하고 실제 영향 파일의 원본 복원 가능성을 확인한다. 그 archive 밖 기존 source/data/contract/test/config를 바꾸기 전에는 원본 bytes를 추가 보관하고 신규 파일 부재를 기록한다. baseline의 path/hash/dirty/추가·삭제·이동 및 archive member는 `baseline/BASELINE_001.json`이 소유한다. 관련 없는 누적 dirty와 과거 실패/감사/증거를 유지하며 reset/clean/stash/자동 commit을 하지 않는다. 계획 단계 P2G source archive만으로 mutation baseline을 대신하지 않는다.

initial 새 full-suite는 요구하지 않는다. Goal Set Shared Validation Baseline Reuse Contract에 따라 유효 U03 draft/fault, U04 strict store/paired intent/CAS, U07M clone/비용, U11 approved identity/Apply 계약을 인수하고 실제 변경 영향과 누락 seam을 실행한다. 역할/날짜 변경을 재실행 이유로 삼지 않는다. 새 product state/codec/identity를 과거 PASS로 대신하지 않는다.

## 구현 의미와 준비

목표는 실제 독립3층 tutorial 및 첫 profile에서 시작 가능한 Expedition, 역할별2/총6중3 직원과 기본 세트/예산, 같은 사본의0–2대여,실제 paid2hire·회복·Gate/exhaustion 귀환,정확 terminal/story 및 durable profile+active 수명이다. memory/trust는 새 model/default/advance/query/codec/capability에서 완전 제거하고 personality/condition/진행 기록은 보호한다. 전체 원문은 FPF의38acceptance가 소유한다.

[CODE_REVIEW](../../CODE_REVIEW.ko.md) T01–T07와 CRA002가 실제 현재 코드와 권장 전환을 구분한다. PreparedApplication exclusive draft,existing grant/copy inventory,seed queue,고정 튜토리얼 실제 executor,U04 accepted journal,U11 normal approved host를 재사용한다. 핵심은 scenario seed/원정 instance 분리,profile knowledge merge와 exact vault/story/active replacement 분리,terminal 한 owner,실제 현재 eligibility와 strict reader/writer 동시 전환이다. 실제 세부 API/타입/숫자/version은 Sol의 PREPARATION에 명시하고 정책 의미가 변하지 않으면 추가 사용자 승인 없이 진행한다.

추가 소비 지점은 terminal 뒤 StartNewGame capability/phase guard, UUID와 progression/checkpoint의 seed-only assertion, fixed3층 terminal과 새 원정 depth≥3 실제 Gate authored schedule이다. 이름만 바꿔 공개 route를 끊거나 원정 귀환을 가짜 terminal로 대신하지 않는다. 사본 single ownership은 vault와 active의 합집합 및 dormant/key-embedded/cross-kind 충돌까지 포함한다. 같은 instance에서 knowledge/story ack 이후 옛 SavePoint를 읽는 정상 복원은 유지하되 최신 영구 상태를 되감지 않는다.

범위 밖은 U13 전체graph/U14 완전 억제력·Breakdown·황금가지, U15 미래 투자/재료 vault, 고유 최종보스,실제 Unity UI/VN,출시/배포 migration이다. 최소 실제 개발 원정과 normal local/server persistence는 범위 안이다. old mode/저장/제거 전용 입력의 silent alias/drop,history 기반 사본 재발급,임의 gameplay vault cap은 허용하지 않는다.

## 검증과 완료

Goal Quality Contract2의10차원을 모두 required로 인수한다. 독립 test author는 `test_parts/TEST_PLAN_001.ko.md`에38개 정확 수용 항목과19IVA의 actual input/oracle/consumer/파일 소유를 연결한다. API-dependent fixture는 제품 공유 계약 뒤 구체화한다. 실패·구현 교정·시험 전제 교정은 각각 원문을 보존하며 shared preparation 오류면 그 owner까지 교정한다.

새 schema/상태는 작은 실제 왕복/독립 소유/CAS/clone 비용 검증으로 먼저 위험을 확인한다. 그 후 필요한 actual tutorial3층 및 normal approved fresh6중3→실제조우→귀환/실패→next expedition,server fresh restart/intent,role/loan/recruit/depth2/3/exhaustion0/invalidcopy/legacy nested input을 실행한다. final owner는 이 Goal의 VERIFY다. no-test/build-only/skip/환경 실패를 functional PASS로 쓰지 않으며 metrics는 기능 실행과 격리한다. 구체 명령/수/원시 결과는 EXPERIMENTS와 Sol raw record가 소유한다.

관련 변경 map: mode/role/start→PS01/02·PE01–06/13; vault/copy/terminal→PE07–12; Gate/exhaustion/recruit/recovery→PE04/08/10/11; profile/Continue/journal→PE01/07/09/11–13 및 U03/U04; data identity/normal prefix→PE02/03/13 및 U11;clone/layout/new profile bytes→U07M. shared reader/writer나 영향 분리 불확실성은 실제 소비 범위를 넓혀 검증하며 무관한 전체 suite를 자동 반복하지 않는다.

GEX10 종료에는 actual source로 guardrail/legacy absence,38항목/10차원/3node/19IVA와 영향을 받은 선행의 current evidence를 닫고 격리 exact 정역 복원·source/evidence manifests·Audit Target을 고정한다. 제품·시험 작성자가 Complete/High 감사 verdict를 선발행하지 않는다. 비참여 whole-Goal GEX20에서 Complete/High/open0를 받은 뒤만 U12 완료를 갱신하고 다음 승인 roadmap으로 진행한다.

현재 unresolved policy/source gap은 없다. 새로운 정책 의미/필수 소비자 접근 불가/unsafe dirty overlap/복원자료 손실은 정확 owner와 영향으로 반환한다. 일반 API/compile/fixture/실제 제품 결함,담당자 대기,단계 전환은 종료나 사용자 재승인 사유가 아니다.

## GEX00 출구와 GEX10 시작 — 2026-10-12

메인은 [BASELINE001](baseline/BASELINE_001.json)의 actual archive4732 member 검증/영향633path와 원본 재사용591·추가11·신규부재31,관련 runtime 불일치0 및 raw/toolchain/dirty 한계를 인수했다. PREPARATION001의 구체 선택과 독립 TEST_PLAN001의38수용·19통합·10차원을 검토했고 계약/소유 blocker는 없다. exact 참조 해시도 일치했다. [GEX00 RESULT001](GEX_00_RESULT_001.json)에서 제품·시험·runtime0의 출구를 고정하고 승인된 GEX10 전체 구현/검증을 시작한다. 앞의 대기 문장은 인수 당시 기록이다.

Sol은 actual product source/data/canonical 문서, Astra author는 신규 test_parts와 world/tests 및 live fixture producer를 작성한다. 기존 product 파일의 cfg(test)는 whole-file 이관 뒤만 Astra가 수정한다. 준비된 build/test는 writer0 뒤 Sol만 실행한다. archive 밖 path는 편집 전 baseline extension을 먼저 결속한다. U11 frozen evidence는 수정하지 않는다.

## 로드맵 검토 반영 계획 — 2026-10-12

최초 요청에 따라 [FINDINGS F01–F06](../../../../roadmaps/core_first/reviews/2026-10-12-roadmap-review/FINDINGS.ko.md)의 수정 계획을 이 섹션에 작성했다. 이후 사용자 지시로 문서·책임 보완을 적용하고 GEX10 남은 구현·관련 검증을 재개한다. 아래 안전중지 상태는 재개 입구 확인 기록이며 원 검토·동결 P2G/감사는 보존한다.

상태 확인에는 이 PLAN, [GEX00 출구](GEX_00_RESULT_001.json), [준비 계약](product_parts/PREPARATION_001.ko.md), [시험 계획](test_parts/TEST_PLAN_001.ko.md), [기술 판단](EXPERIMENT_NOTES.md), [실행 기록](EXPERIMENTS.md)과 마지막 두 작업자의 안전중지 보고를 사용했다. 현재 [원정 수명 코드](../../../../../src/game/world/expedition_lifecycle.rs)의 출발/대여/terminal·first-entry receipt·Gate/소진 선택 및 [독립 소진 시험](../../../../../src/game/world/tests/u12_lifecycle/return_exhaustion.rs)을 다시 읽어 부분 작성 사실을 확인했다. 제품에는 mode/profile/UUID·시작/tutorial·대여/귀환·저장/정확 발행 연결이, 시험에는 신규 lifecycle 30개와 public/normal host probe가 작성돼 있다. **U12 compile/build/test 실행은 0회이고 기능 검증·후속 인수는 미완료다.** 작성된 시험 수는 통과 수가 아니다. 두 담당은 writer0·진행 실행 없음으로 중지했으며 OS 전체 프로세스 부재를 주장하지 않는다. EXPERIMENTS와 Working Policy의 과거 ‘변경0’ 문구는 GEX00 등 당시 사실로 구분할 수정 대상이다.

### 수정 대상과 책임

| 대상 / owner | 보완 계약과 문서 완료 조건 |
| --- | --- |
| [단일 현황](../../../../roadmaps/content_authoring/ROADMAP_STATUS.md), [최신 인계](../../../../roadmaps/core_first/execution/HANDOFF_LATEST.ko.md), [로드맵](../../../../roadmaps/core_first/ROADMAP.ko.md)의 상태 안내 / U00·메인 | 기존 단일 현황을 유지하고 이 PLAN의 현재 상태와 실제 실행 기록을 직접 가리킨다. 최신 인계는 부분 구현·미실행·잔여 작업을 구분한다. 로드맵의 과거 U08 현재/U07-M 대기 등은 당시 이력으로 표시하고 본문 작업 범위·순서를 보존한다. Working Policy·EXPERIMENTS·EXPERIMENT_NOTES의 현재/과거 표현도 같은 상태로 맞춘다. 별도 현황 장부를 만들지 않는다. |
| 로드맵 U12→U13/U14/U16/U17 및 U21의 인계 / U12 메인·각 후속 owner | 아래 인계 의무를 지금 명시한다. U13은 지도 확장·재방문 identity, U14는 유예/황금가지/무억제 확장, U16은 유료 채용과 지원 활동, U17은 실제 활동 소비, U21은 납치 등 행동 가능성 변화가 기존 계약을 어떻게 소비·갱신하는지 적는다. 완료 코드/시험 근거는 실제 검증 뒤 결속하며 미완료 산출물에 ‘인수 가능’ 표시를 하지 않는다. |
| 로드맵 U13/U20 및 [지도 규칙](../../../../game_rules/run-and-map.md)의 배정 경계 / U13 보장·U20 소비 | U13은 유효 authored 데이터에서 **진입 가능한 모든 첫 방 경우**에 예약 보호와 미완료 필수 전조 후보 수용 조건을 보장하고 불가능한 입력은 초기 검증에서 거절한다. U20은 후보/첫 진입/동층 중복 방지에 따른 실제 배정·저장·복원으로 소비하고 도입 시 재검증한다. 후보 유무·여러 첫 방·예약 포화·재방문·U17/U21 부화 대체와 결정론의 검증 책임을 적는다. 모든 방 전조 사전배치 방식으로 확정하지 않는다. |
| 로드맵 §8.4와 U13–U17/U20/U21의 기존 도입 계획, U27/U29 인수 / 각 도입 owner | 단계별 해당 노드의 authored 항목, fixed 규칙, invalid 입력, 실제 command consumer, save/세대, public 지원 범위를 짧게 명시하도록 한다. 지금은 도입 의무와 책임을 배정하고 세부 목록은 해당 단계에서 좁힌다. U27은 실제 구현 근거를 합성하고 U29는 허용/거절을 검증한다. 전체 미래 schema 동결·범용 노드 엔진 선구현으로 확대하지 않는다. |
| 로드맵 U21/U27→U26-A/B/C, §3 Unity 후속 열·§7.4·부록 A 대응 / Core 표현 owner와 해당 Unity owner | U26-A는 starter 이후 활성 전장 내 소환/진화·빙벽/개인 방어막·원천별 감염/연결망·형태변화·특수 작업의 판단에 필요한 표시와 조작을 닫는다. **전투 중 고유 도구/대상 선택과 라운드 사이 전장 선택도 A**, 편성·정비·비전투 활동 및 전투 전후 고유 선택/설명은 B로 배정한다. 같은 기능의 단계별 화면 경계와 query/event·자산 참조를 연결하고 U26-C는 기존 화면 통합·재접속/복구를 맡는다. Core 계약 부족은 U21/U27의 해당 producer에 반환한다. 기능군마다 최초 구현 owner와 실제 장면 검증으로 추적 가능해야 한다. |
| [전투 조작](../../../../game_rules/battle-controls.md)↔[유물 보상 노드](../../../../game_rules/relic-reward-nodes.md), U16-4/U26-A / 규칙·활동·화면 owner | 일반전/전조의 2회 후퇴, 스트레스 전투 중 무보상 종료·노드 소비/재도전 불가, 라운드 사이 보상 받고 철수, 고유 후퇴 금지의 자격·결과를 연결한다. 메뉴는 해당 Core 조회를 소비하며 공통 2회 문구로 예외를 덮지 않는다. 규칙 링크·담당 카드·검증 조건이 일치하면 문서 보완을 닫는다. |

### U12 인계 의무와 변경 영향

U12의 후속 인계에는 first-entry identity/receipt·재입장 중복 방지, 실제 소진 전이와 선택 시점, 깊이별 귀환 cause와 terminal owner, 유료 채용 가격·횟수·queue/offer identity, eligibility와 실제 명령/회복 판정의 공유 지점, 이 상태들의 Continue/SavePoint/profile 수명을 포함한다. 후속 지도·활동 변경은 같은 의미를 재사용하거나 명시적인 이행과 중복 소모/채용 방지 검증을 소유해야 한다. Rust 타입명을 영구 고정하거나 별도 입장·귀환·채용 판정기를 만드는 의무가 아니다. 검증 전에는 준비 계약과 부분 소스를 참고 위치로만 연결하고, 검증 완료 뒤 기존 EXPERIMENTS/인계에 정확 코드·시험 입력·raw 결과·적용 범위를 붙인다. U12 완료 인수는 기존 독립 감사 통과 뒤에만 갱신한다.

이번 문서·책임 보완 자체는 U12의 38수용 항목·19IVA·10차원, 단일 Goal/Phase와 최소 원정 범위를 바꾸지 않는다. F01/F02는 현재 안내와 산출물 인계에 직접 적용하고 F03–F06의 후속 구현은 해당 owner에 남긴다. 이 지적만으로 전체 U12 계획이나 완료 선행을 재개방하지 않는다. 실제 U12 schema/저장/승인 경로 변경의 선행 영향 검증은 이미 정한 절차대로 필요한 범위만 수행한다.

현재 제품 잔여에는 pending 선택의 NodeConfirm/session/선택 노드 및 current/nested SavePoint 결속, HQ query와 회복 evaluator, corrupt-active/profile reset 원자 처리, 공식 소비자/문서와 구버전 거절의 완결이 있다. 시험 잔여에는 제품 파일 cfg(test) 순차 인수, 일부 Continue 양성의 profile pairing, reset·pending 음성, 전체 수용/통합 연결과 실행 근거가 있다. 이는 기존 GEX10의 미완료이며 이번 문서 검토로 새로 생긴 전체 재계획 사유가 아니다.

### 적용 순서와 종료 기준

1. 메인이 위 현황·로드맵·규칙·책임 문서를 보완하고 현재/이력 구분, 상대 링크, 단계 간 producer/consumer와 제외 범위를 대조한다. 모든 F 항목에 대상·owner·문서 완료 조건이 연결되면 **문서 보완만** 완료로 표시한다.
2. 기존 U12 PLAN·준비/시험 계획에서 직접 영향과 후속 단계 의무를 확인한다. 필요한 실행 인계만 갱신하며 동결 P2G·과거 감사·실패 원본은 수정하지 않는다. 새 정책 변화가 없는 이번 보완을 재동결이나 전체 재검증으로 확대하지 않는다.
3. 기존 Sol 제품/단독 runner와 독립 Astra 시험 작성 역할로 남은 구현·관련 시험을 이어간다. 메인은 actual code/oracle를 검토하고 관련 writer0 뒤 Sol이 준비된 검사를 실행한다.
4. 검증된 계약에 실제 근거를 결속하고 기존 최종화·격리 복원·비참여 Astra 전체 독립 감사로 종료한다. 감사 전에는 U12 완료·후속 인수 가능을 표시하지 않으며, 승인된 후속 로드맵 순서는 유지한다.

문서 적용 결과: 위 1–2의 현황/이력·단계별 책임·후퇴 안내를 해당 mutable 문서에 반영하고 기존 최신 인계를 원본 보존 후 갱신했다. U13/U20·U26 실제 기능과 장면 검증은 후속 단계에 남는다. 문서 의미·상호 링크 검토를 마치고 3의 기존 미완료를 계속하며 U12 정책/수용/Goal은 재발행하지 않는다. 재개 점검과 실제 실행 구분은 [EXPERIMENTS](EXPERIMENTS.md), 판단은 [EXPERIMENT_NOTES](EXPERIMENT_NOTES.md)가 소유한다.
