# U16-4 비참여 최종 완료 감사004

판정: **Complete / High**. GOAL-CSCB-01의 적용 수용 조건을 충족했다. 확인한 범위의 새 제품 finding 0, 열린 완료 차단 0. 과거 IA001/IA002의 교정 확인을 유지한다. 전체 U16, U17, Unity, U21 최종 성능·밸런스의 완료 판정은 아니다.

## 권한·독립성·고정 대상

2026-10-15 사용자의 “CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md 문서를 지침 삼아 u16-4 독립 감사” 요청을 **GEX-20 stage-run**으로 인수했다. 이번 별도 실행자는 구현·교정·기존 검증 작성에 참여하지 않았다. 현행 지시서 §4/7/8, GEX Reader Contract·§5/6.4/6.7/6.8·GEX-20, 로드맵 §1.9를 적용했다. 역사적 Goal의 서브에이전트 소환 문구는 현재 단독 실행 지시로 대체하며, 비참여 조건과 기능 기준은 유지한다. 위임하지 않았다.

GS-CSCB-001 / GOAL-CSCB-01 / PHASE-CSCB-01, HA-CSCB-001 / FPF-CSCB-001 / PP-CSCB-001, Goal Quality Contract v2. [Goal 원문](../../../P2G_50_GOAL_SET.ko.md), [상세 계약](../../../RULES.ko.md), 상위 `game_rules/relic-reward-nodes.md`와 현재 runtime/loading 계약을 직접 검토했다. single_phase이므로 이 감사가 Goal Set의 최종 통합 조건까지 담당하며 별도 GEX-30은 없다.

[고정 target](../target/u164_current_001/AUDIT_TARGET_RECORD.json)은 `u164_current_001`, HEAD `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`, source920/evidence550/delta353이다. SOURCE SHA256은 `e97f62e376c2fb3902887e55283a162bb126c4ba62e9a2071e57a3c31924f7f4`이다. [입구](ENTRY.json)와 [출구](EXIT.json)에서 모든 고정 실제 파일의 일치와 source roots의 누락·추가 부재를 확인했다. **current / unchanged**다. 제품·공식 입력·canonical 시험·기존 증거·PLAN/EXPERIMENTS·동결 기록을 편집하지 않았다.

BEFORE603/SOURCE920의353개 변환을 pre/post 해시로 검증하고, 별도 메모리 사본에 정방향·역방향 변환을 적용하여 전체 파일집합과 정확한 바이트 동일성을 독립 검산했다. 실제 격리 filesystem 복원은 동일 archive/delta의 [GEX10 원시 복원](../target/u164_current_001/RESTORE.json)을 재사용한다. 작업트리 patch 재적용/reset은 없었다. [검산 코드](check_evidence.py), [결과](EVIDENCE_CHECK.json).

## 수용 조건별 판정

아래 source 경로는 `core_enhanced/src/game/` 기준이다. 구현자의 COVERAGE 표시는 탐색에만 사용하고 실제 소스·시험 오라클·raw trace를 대조했다.

| 수용 | 직접 검토·실행·재사용 근거 | 판정 |
| --- | --- | --- |
| A01 입장/첫 보상 | `live_ron→StressCombatPolicy::validate`, `facility_rooms/map_content`, `expedition_lifecycle::node_entry_preview`에서 지원 RON의 전용 Defense 조우·첫 보상·배율을 공개한다. 기존 정상 package/calibration/adoption 입력을 해시 확인했고 이번 새 host 준비/입장에서도 해당 경로를 실행했다. | Pass |
| A02 선택/안내 | `battle_capability`의 waiting gate, `stress_combat_view`, `handle_stress_combat_action`의 UUID/완료round 검사를 확인했다. 이번 host에서 잘못된 UUID/round·대기 중 retreat 거절, 대기 tick/reconnect의 checkpoint 불변, 성공 철수·중복 거절을 직접 재현했다. 기존 무억제 pending/next 조회도 재사용한다. | Pass |
| A03 웨이브/5라운드 | `wave_activation_cause`는 현재까지의 round만 허용하고 `settle_stress_round`는 해당 group 전부 clear 후 전이한다. 기존 delayed multiwave 및 미래wave/actor 의존 작성 거절을 이번53검사에서 재실행했다. 정상9명·확장27명5라운드의 기록을 실제 Core로 각각 새 재생했다. | Pass |
| A04 상태/시간 | 단일 ActiveBattleSession/BattleCore의 current/completed/waiting만 전환한다. timestamp bucket을 정리하고 terminal 판정을 수행한다. 기존4회 Continue 전후 units/deployment/TP/sustained/time 전체 동등성을 raw에서 독립 검산했다. 실제 미만료 stance cooldown, 내부cooldown, 전투당1회와 같은시각 패배우선의4 completion probe를 재실행했다. 일반 보호효과 차단/명시수리 대조도 통과했다. | Pass |
| A05 누적 단일 지급 | `combat_battle_state_from_live_result`는 성공시에만 completed 누적분을 ClaimAll 공통 grant에 전달한다. 기존 raw에서 중간 자원/XP 미지급, 정상150/가루15/파츠15/각25XP·무억제300/30/30/각50XP를 독립 산술 검산했다. 이번 host 철수에서10/1/1/출전각11XP와 fresh결과·중복거절을 확인했다. 유물1개 유지는 기존host 및 재실행 probe로 확인했다. | Pass |
| A06 패배/직원 손실 | 같은시각 terminal 우선과 공통 participant 결과/생존XP owner를 검토했다. 원래 실제 보호목표 파괴 trace의 누적상실·무억제패배 기본XP10, 실제 무력화 injury1/trauma26·XP0·소비품 차감을 대조했다. 지원작성으로 실제 무력화→다음입장비용 사망→장착유물 회수 probe도 재실행했다. 이를 전투 중 즉사 사례로 확대하지 않는다. | Pass |
| A07 수동 후퇴 | `BattleEntrySnapshot::restore_shared→handle_complete_node`, continuation의 안정상태 발행 분기를 확인했다. pending 후 Continue→후퇴의 자산/기간 복원·입장비용 유지·노드소비·XP0·재도전 거절은 기존 raw와 이번 Core probe로 검증했다. 후퇴의 실제IO/동일요청재시도 및 fresh저장은 원래 trace를 독립 검산하여 재사용했다. | Pass |
| A08 무억제/수량 | 정상 비용소진→공개 Continue 선택의 원래host trace와 전5wave9명 HP135/공격15를 정상90/10과 대조했다. 성공보상만2배, 유물수량 유지, 패배기본XP10을 확인했다. 누적잔고·가루alias·재료분리 경계와 정상/무억제 대량XP 원래 실패 probe를 재실행했다. XP4,294,967,314가 두 출전자에게 정확히 반영된다. | Pass |
| A09 복원/IO | 기존17process/192연산의 same-process reconnect·fresh미완료·완료결과확인 전후·SavePoint, finaltick/extract/retreat/SavePoint의4개 OS21 및 동일요청재시도를 독립 검산했다. 이번 새5process/41연산에서 SavePoint와extract의 실제IO2회, 새 프로세스의 미확인결과 유지/확인, 미완료 입장전 rollback을 직접 재현했다. host가 메모리 및 양쪽 저장소 byte 불변을 assert한다. | Pass |
| A10 작성/공개경로/회귀/비용 | 최종host와 cost binary의 해시, fixed pack·runtime settings, build/dependency 기록을 대조했다. 기존 RON25·공개패키징 오류거절/경계승인과 Core고유2222/ignore6를 조건별 재사용했다. 새53검사, approvedhost41연산, 정상52/확장33 Core연산 재생이 통과했다. 요청heap 정확회수와 실제시간/RSS를 내부work와 분리한다. | Pass |

## 차원·통합·구조

10차원 모두 applicable/Pass이며 N/A는 없다. success=A01/03/05, invalid_input=A02/10, failure_rollback=A06/07/09, retry_idempotency=A02/05/09, ordering_concurrency=A04/09, boundary_empty_max=A03/05/10, disconnect_restart_persistence=A09, live_data=A01/03/10, producer_consumer=A01/09/10, legacy_fallback_absence=A04/05/07/10이다. 직렬 application candidate/publication과 동시 timestamp 처리를 검증하며 병렬 gameplay 지원을 새로 주장하지 않는다.

IVA-CSCB-FLOW/GAME/LOAN/SHOP/STORE는 동결 매핑대로 각각 A01–02/A03–04/A05–06/A07–08/A09–10에 대응한다. AG-CSCB-RUN과 WP-CSCB-VERIFY의 제품 연결·지속 가능한 테스트/host seam·비용·보관 책임을 모두 충족했다. 재현용 입력과 진행기는 `test_parts`, 원시 결과는 `runs`와 `completion001`, 이번 추가증거는 이 디렉터리에 유지한다.

scoped delta와 실제 호출을 검토했다. 정책은 validated layout, round gate는 기존 wave runtime, 보상은 공통 grant/XP, 후퇴는 entry snapshot, 저장은 기존 paired owner가 소유한다. 라운드마다 전투를 재생성하거나 자동회복/대체보상/새 저장권한을 추가하지 않는다. optional serde default는 기존 layout 호환을 유지하며 active battle의 미완료 fresh Continue 계약도 유지한다. 전용 class는 일반 조우 선택에 섞이지 않는다. 보호대상에 대한 직접 지속효과 gate와 공유보호막의 직원 전용 membership을 확인했다.

fixture 이름에 따른 제품 분기, 오류를 성공으로 바꾸는 catch-all, client 재추론, 임의 시간/수량 한도 완화, 중복 소유권, 새 legacy fallback을 발견하지 못했다. runtime/data-loading 문서는 현재 명령·조회·보상·복원 의미와 일치한다. 공통 XP의 u64 임시합계는 기존 공개필드/성장곡선/대상을 유지하며 작은값·경계·큰값·실패원자성 영향검사가 존재한다.

선행 U16-3의 [비참여 Complete/High](../../../../core_dealer_minigames/execution/single_goal/COMPLETION_AUDIT.md)와 U16-4 착수전 baseline을 인수했다. 이후 StressCombat 전용 분기와 공통 command vocabulary 증가는 기존 기능을 대체하지 않는다. 기존 broad의 U16-1/2/3·일반전투·저장 통과를 재사용하고, 공유 XP 교정은49개 영향검사 및 이번53개에 포함된 경로로 재검증했다. 선행의 과거 target을 새 소스에 대해 unchanged라고 소급 표기하지 않는다. 관련 의미의 비영향 및 필요한 영향검증을 이 target의 [BASELINE_REUSE](../BASELINE_REUSE.json)와 결속한다.

## 새 실행·재사용·제한

- 새 격리 Cargo 실행: [replay001.log](replay001.log), **53pass/0fail/0ignore, exit0**, 실행20.50초. 기존 correction/감사 probe와 completion4개를 한 번에 선택했다. 원본시험을 수정하지 않고 archive 사본의 시험 파일에 추가했으며 절대 include 경로만 사본에 맞췄다.
- 새 승인host: [HOST_REPLAY](HOST_REPLAY.json), **5process/41연산, 전부exit0**. 새 store·output을 사용했다. 실제OS21 2회와 동일명령재시도, 잘못된명령/중복의 무변경을 포함한다. 원본issuer의 기존승인을 사용했으며 새 calibration을 수행했다고 주장하지 않는다.
- 새 비용 재생: [COST_REPLAY](COST_REPLAY.json), 정상52/확장33연산 각1회, 둘 다exit0. 최대 Core연산 **90.03ms/256.29ms**, 둘 다 drop후 요청heap 정확회수. 동일binary의 기존각3회와 함께 해석하며 새 최적화/최종SLO/RSS원복을 주장하지 않는다. 시간창은 disk publication·trace직렬화를 제외한다.
- 독립 재검산: [EVIDENCE_CHECK](EVIDENCE_CHECK.json)의11묶음은 archive/delta·기존raw 성공산술·4회 상태유지·4IO재시도·fresh결과·비용6회와 고유회귀수를 확인한다. 첫 검산기의 regex가 `- should panic` 성공12개를 누락하여 실패했다. 원시결과를 확인한 뒤 parser만 교정했고 실제고유2150+72=2222를 확인했다. 이는 제품실패가 아니다.
- 유효 재사용: 과거 Core고유2222/ignore6, RON25/공개pack 양·음성, 기존host17process/192연산과 실제IO4경로, 정상/확장비용6회. 중첩된53/49/25를 더해 새 고유시험수로 보고하지 않는다. 현재전체 일괄green도 주장하지 않는다.
- 기존 서버 fixture1실패(`SelectStarterEmployees.loan_copy_ids` 누락)는 착수전 불일치이며 이번제품delta가 원인이 아니다. 서버 메시지8pass는 재사용하지만 서버전체green은 아니다. 공식 기본편대5round의 최종밸런스, U21/Unity/전체U16 완료도 이 판정에 포함하지 않는다.

모든 발생 근거는 정상 플레이·지원작성·지원명령·실제IO에 연결했다. 외부 save/checksum/메모리 임의변조만으로 생기는 사례를 finding이나 완료차단으로 추가하지 않았다. 모든 소유 실행이 종료되었고 제품교정·U17 이후를 시작하지 않았다. 감사 종료 이후 HANDOFF/이력/상태 색인만 별도 기록 작업으로 갱신한다. frozen PLAN/COVERAGE의 당시 “감사 대기”는 역사적 입력으로 보존하며 최신 판단은 이 문서와 RESULT.json이 소유한다.
