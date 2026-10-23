# U14 독립 완료 감사 — Not Complete

2026-10-14, GEX-20 stage-run / `GS-CEDR-001` · `GOAL-CEDR-01` · `PHASE-CEDR-01`.

**판정: Not Complete. 열린 완료 차단 finding 1건: F-U14-001 (P2).** 시설 전체 지도에서 다른 방문 방의 미회수 봉인 보스가 봉인 없음·HP/ATK 100%로 표시된다. 정상 제작·공개 이동 명령으로 재현했다. 제품 교정과 U15 착수는 수행하지 않았다.

## 권한·대상·독립성

사용자가 `CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md`를 읽고 U14 독립 감사를 시작하도록 요청했다. 이전 구현 인계의 감사 대기는 이 요청으로 해제되었다. 이번 대화의 실행자는 U14 구현·교정에 참여하지 않았으며, 서브에이전트·위임 없이 source review와 감사용 probe를 직접 수행했다. 감사 과정에서 구현 코드를 수정하지 않았다.

원문 기준은 `FPF-CEDR-001`, `PP-CEDR-001`, [Goal Set의 31수용·10차원·10IVA](../../P2G_50_GOAL_SET.ko.md), 현행 실행 지시서 §4·§7·§8 및 GEX-20이다. 외부 임의변조만으로 발생하는 사례는 완료 차단에 사용하지 않는다.

- 정확한 대상: [u14_current_001](target/u14_current_001/AUDIT_TARGET_RECORD.json).
- HEAD: `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`.
- SOURCE SHA256: `49e2263de43ce65589a414277eb16b69f9a8cdcd6543896936aa284b2c33e456`.
- [입구](audit/u14_independent_001/ENTRY_CURRENTNESS.json)와 [출구](audit/u14_independent_001/EXIT_CURRENTNESS.json): source 704 / evidence 980의 모든 member 일치. source archive·manifest·restore record identity도 일치한다.
- 실제 production 3개 root의 추가 파일 누락 없음. [증거 재확인](audit/u14_independent_001/EVIDENCE_RECHECK.json).
- source/before archive의 117개 delta를 정·역으로 독립 재적용하여 전체 member/byte 일치를 확인했다. 기존 격리 filesystem 복원 PASS는 재사용했다. 작업 트리에 patch/reset/clean을 적용하지 않았다.
- PLAN·EXPERIMENTS·자체 검토·기존 실패 원문·동결 target은 그대로 보존했다. 이번 감사와 probe는 `audit/u14_independent_001`에 분리했다. HANDOFF·이력·ROADMAP_STATUS는 target의 명시적 제외 locator다.

## F-U14-001 — 다른 방 봉인·배율 preview의 현재 방 의존

**P2 / 범위 안 — 지원 경로 / U14 GEX-10 교정 owner.** A13의 진입 전 조우별 배율 공개와 public producer-consumer 계약을 충족하지 못한다. [finding 상세](audit/u14_independent_001/FINDINGS.json), [실제 관측](audit/u14_independent_001/visited_probe001/RESULT.json).

`facility_rooms.rs:182`는 현재 방과 방문 방의 노드를 모두 `node_entry_preview`에 넘긴다. `expedition_lifecycle.rs:197–205`는 봉인 여부를 `sealed_golden_bough(node.id)`로 조회한다. 그런데 `facility_rooms.rs:6–14`의 해당 함수는 `run.map`과 `state.current_room_id`만 검색한다. 다른 방의 정상 node는 검색되지 않아 봉인 없음과 일반 배율로 투영된다.

감사용 fixture는 정상 작성 경로에 B1-01의 선택적 일반보스 slot 1/3과 두 배치안을 추가했다. 두 안 모두 slot 1을 포함하며, 기존 SavePoint 2 → 정비 5 → 상점 8 → Gate 경로로 보스를 건드리지 않고 방을 이동할 수 있다. 일반 `package_official`과 `content_pack::load`를 통과한 뒤 공개 starter/노드/방 이동 명령만 실행했다. 저장·checksum·메모리 상태를 변조하지 않았다.

| 관측 | 현재 방 | 억제력 | B1-01 slot 1의 공개 봉인 | 공개 HP/ATK |
| --- | --- | --- | --- | --- |
| 원정 시작 | B1-01 | 12 | 있음 | 150% / 150% |
| B1-01의 지원 경로 완료 후 수평 이동 | B1-02 | 9 | **없음** | **100% / 100%** |
| B1-02의 지원 경로 완료 후 복귀 | B1-01 | 6 | 있음 | 150% / 150% |

전체 지도 marker는 중간에도 같은 가지를 미회수·도달가능으로 표시한다. 전투·봉인승리·가지 회수는 한 번도 없고, 모든 관측에서 억제력이 양수이므로 무억제 전환으로 설명할 수 없다. 잘못된 값은 `activities[].entry_preview`와 `current_expedition_map.nodes[].entry_preview` 양쪽에 나타난다. 중간 조회의 모든 공개 root 무변경도 확인했다.

따라서 플레이어/후속 consumer는 방문 방을 비교할 때 봉인 전투의 위험을 낮게 안내받는다. 이 반례가 실제 전투 스탯·정산·보상 중복 또는 저장 손상을 입증하는 것은 아니다. 별도 bough marker, 비용·유예·사망 forecast는 이 반례에서 정상이다.

필요한 후속 검증은 현재/방문 방의 미회수·회수 및 억제/무억제 projection, 방 왕복·Continue 후 일치, 기존 hidden encounter 공개 경계와 원래 slot/부활 제외의 영향 범위다. 정산 owner·입장 receipt와 공개 권위를 중복시키지 않는 U14 교정 후 필요한 successor와 별도 독립 재감사로 반환한다.

## 수용 조건·구현 검토

[조건별 판정](audit/u14_independent_001/CONDITION_REVIEW.json)에 31수용 조건 각각의 source·원시 시험·재사용/신규 근거를 연결했다. **수용 30 pass / A13 fail, 차원 8 pass / success·producer_consumer fail, IVA 9 pass / IVA-CEDR-PWR02 fail**이다. 하나의 finding이 여러 추적 항목에 반영된 것이며 결함 건수는 1건이다.

직접 source review 결과:

- 입장은 activity→kind→run authored 비용, 기존 receipt 한 곳을 소비한다. 잔량 saturating 차감과 직원 규정비용 소비를 분리하고 사망·자산회수 뒤 소진선택/치료/활동을 처리한다. 기존 완료노드 유예 감소 호출은 제거되었다. SavePoint의 내부 캡처 전 Breakdown reconcile과 사망 후 기존 recovery 판정을 확인했다.
- `RunPowerAuthorityState`는 authored anchor만 선택한다. 전투 snapshot은 비용 후 자격을 고정하며 cache에는 독립 scale 필드를 대입한다. 실제/지연 적과 보스 소환 경로가 같은 draft scale을 소비한다. 부활·무억제 곱셈, exact override 이후 HP/ATK 적용, 봉인·무억제 비중첩을 확인했다.
- 실제 피해 mutation 뒤의 실효 HP 손실과 최대HP 분수만 관측하고 BasicAttack/Ability를 허용한다. DOT/여러 타격 합산을 제외하며 Victory에서 목표별 한 번 효과를 생성한다. 결과 지급의 whitelist와 기본 생존XP의 고유 참가자·생존 조건, 기존 target별 전액 지급을 확인했다.
- golden-bough plan은 원정 state에 한 번 선택하여 유지한다. 작성 검증은 후보·중복·양수 가중치/보충·wing·지원 도달성을 검사한다. 원래 slot의 미회수 봉인승리만 collection/refill을 확정하며 부활은 제외한다. 후퇴 snapshot과 미확정 Continue가 서로 다른 경계를 유지한다.
- writer 15의 명시적 거절, 기존 SavePoint/paired persistence와 terminal candidate의 원자성 경로를 확인했다. 구저장의 opaque 보관을 게임 호환 지원으로 해석하지 않았다.
- 새 상태는 기존 run/employee/battle/result owner와 clone shape에 연결되어 있다. 제품의 시험명 분기·새 중복 보상권위·미래 전용 abstraction은 확인하지 못했다. 공개 조회의 잘못된 현재 방 의존은 F-U14-001로 남긴다.

U13 최종 감사 `u13_independent_002`는 당시 target의 판정으로 인수했다. U14에서 변경한 U13 공개/방 이동 경계는 focused011 및 현재 host/public 실행으로 별도 검증되어 있으며, 이번 발견은 U14가 추가한 projection delta에 귀속한다. U04/U12 기존 메커니즘의 재사용과 변경된 입장/Continue 회귀를 구분했다.

## 실행·재사용 근거

**이번 새 실행:** 감사용 release probe build exit 0, 81초. 지원 authoring·공개 이동 반례 1개 실행, exit 0 / 1.150초. 여기서 exit 0은 시나리오와 관측 완료이며 제품 계약 PASS가 아니다. `seal_projection_matches=false`, `battle_projection_matches=false`가 실제 결과다. 독립 일반 소비자 진단이며 새 Normal host 승인 주장은 없다.

**이번 독립 재계산:** [원시 결과 검산](audit/u14_independent_001/INDEPENDENT_EVIDENCE_CALCULATIONS.json). 실제 12승 각각의 난이도별 재화/장비가루 및 직원별 XP를 재계산했다. 4×10 + 8×20 = 200, 각 직원 레벨 3/잔여 0과 일치한다. 공식/확장 6개 원시 표본의 각 단계 성공과 종료 후 requested live bytes의 baseline 복귀를 확인했다. archive 정·역 delta도 독립 검산했다. 이 활동은 게임플레이 재실행과 구분한다.

**유효성 확인 후 재사용:**

- focused011: 82 실행/82 통과. G11 기본 4/4, runtime-observation 4/4. 서버 persistence 29/29. 모든 coverage test 이름을 해당 PASS 원문에서 찾았다.
- 전체 Core002: 1,940 통과/154 실패/6 ignored. baseline의 동일 기존 실패 138개는 실제 원시 실패 집합의 부분집합이다. 나머지 16개(기존 회귀 15 + 신규 fixture 1)는 최종 focused/G11 PASS 이름에 전부 연결된다. 전체 green은 주장하지 않는다. 후속 제품 변경은 Gate/입장사망 recovery/지도 projection/정렬에 국한되며 관련 회귀를 재실행한 근거를 검토했다.
- normal_route002: 실제 승인 host의 fresh-process 30단계. sealed_host001: 10단계와 실제 terminal tick의 paired journal IO 실패, 메모리·active/profile 원본 유지, 같은 tick 재시도, XP 34/14/14·재료 14·성과 가루 16·가지 1·보충 8 및 재접속/ack 중복 없음. 해당 host source·driver·calibration/adoption·binary receipt를 대조했다.
- 실제 공개 봉인승리·12전 성장, 공식/확장 계량 6표본, 보존된 초기 실패/준비 교정 기록. frozen source/evidence identity와 최종 실행 뒤 runtime/data 무변경 근거를 확인했다. 기존 시험 통과가 새 공개 반례를 닫는 것으로 취급하지 않는다.

재현은 저장소 root에서 다음과 같다. 마지막 인자는 존재하지 않는 새 output 경로로 지정한다.

```bash
core_enhanced/docs/goals/core_expedition_deterrence_rewards/execution/single_goal/audit/u14_independent_001/u14_independent_probe audit-visited game_resources /tmp/u14-audit-visited-repro
```

정확한 argv·binary hash·exit·시간은 [PROBE_RECEIPT_001](audit/u14_independent_001/PROBE_RECEIPT_001.json), source/fixture와 결과는 [probe](audit/u14_independent_001/probe/src/main.rs) 및 [보존 출력](audit/u14_independent_001/visited_probe001/RESULT.json)에 있다. 원본 결과는 덮어쓰지 않는다.

## 종료 경계

이번 감사 작업은 종료했지만 Goal은 완료되지 않았다. 다음 단계는 F-U14-001의 **U14 GEX-10 교정 → 영향 검증/필요한 successor → 별도 독립 GEX-20**이다. 감사 stage-run 권한으로 제품을 고치거나 U15를 시작하지 않았다.

공식 네 고유보스/최종보스 본체·U16 전체 스트레스 활동·Unity·배포·전체 공식 균형 플레이·최종 SLO는 기존 제외/후속 범위를 유지한다. 성장은 공식 적/무기/성장 수치를 유지한 OpenHall 작성 경로이고, 계량은 한 방 현현과 전체 정적 지도 조회다. U11의 FacilityLayout/Abnormality changed-mod 계측 미지원과 private issuer key 제외도 유지한다. 추가 외부 임의변조 방어·필수시험을 만들지 않았다.
