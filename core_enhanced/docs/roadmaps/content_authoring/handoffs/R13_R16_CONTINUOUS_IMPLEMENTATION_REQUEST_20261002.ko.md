# R13~R16 연속 구현 지시서

2026-10-02 · v3 · 구현 담당자에게 전달할 요청 문서. 이 문서 작성은 제품 구현·정식 P2G 검증·독립 감사 실행이 아니다.

## 1. 요청과 종료점

콘텐츠 제작 기반 로드맵의 **R13→R14→R15→R16을 구현하고, 최종 통합 소스에서 자체 검증한 뒤 감사 대상으로 동결**한다.
소환·설치물·진화, 유물 사본 관리와 수치 합성, 유한 활성 조건과 구조 변환, 반응형 연쇄를 실제 공개 데이터와 전투에 연결한다.
2026-10-02에 합의한 유물 정책은 §4에 명시한다. 차기 원정 전체를 구현하는 요청으로 확대하지 않는다.

전달에는 [시작 요청문](R13_R16_START_PROMPT_20261002.ko.md)을 사용한다.
기본 실행은 중간 WP마다 사용자 검토를 기다리지 않는 연속 구현이다. 필수 atomic group과 실제 Phase 선행 감사는 유지한다.
최종 종료점은 **R13~R16 구현·자체 검증 종료 / 최종 통합 target 동결 / 독립 감사 요청 대기**다.
필수 기능이나 선행 증거가 남으면 실제 부분 상태와 남은 조건을 보고한다. R13~R16 구현자가 해당 구현의 독립 PASS/Complete를 발행하거나 R17을 시작하지 않는다.

## 2. 먼저 인수할 선행 상태

[개발 현황](../ROADMAP_STATUS.md), [R08~R12 실행 PLAN](../../../goals/content_authoring_r08_r12_runtime/execution/single_goal/PLAN.md),
[최종 인계](../../../goals/content_authoring_r08_r12_runtime/execution/single_goal/REVIEW_HANDOFF.md)를 먼저 읽는다.
작성 시 확인한 기록은 다음과 같다. 아래 수치는 인계서의 보고이며 이번 문서 작성자가 재실행한 결과가 아니다.

| 항목 | 작성 시 확인한 기록 |
| --- | --- |
| R06/R07 선행 감사 | 각각 Complete / High. 해당 감사 source에 한정되며 후속 변경 전체의 감사로 확대하지 않음 |
| R08~R12 target | source `bd89f9c42191217a333d648d35f495e802d13e6fb1420caa9c08da3deb731690`, 387 files |
| R08~R12 자체 검증 | 55조건 중 54 자체 검증, X09 환경 제한 1건 |
| X09 | Unix socket fixture의 OS 권한 제한. 제외 실행의 성공을 원명령 전체 통과로 취급하지 않음 |
| R08~R12 독립 감사 | 요청 대기. 구현 인계·자체 시험 성공은 독립 감사가 아님 |

실행 시 더 최신인 감사·교정이 있으면 exact 문서, verdict, target, 미해결 finding, 현재 source와의 차이를 확인해 인수한다.
이 전체 묶음의 제품 변경 전 필요한 R08~R12 선행 감사를 확인한다. 감사가 없다면 **이번 요청에 포함된 실행 범위로
R08~R12 독립 완료 감사(GEX-20)를 먼저 수행한다.** 조사·계획 준비나 감사 요청 대기만으로 종료하지 않고 감사 시작의 재승인을 묻지 않는다.
감사 대상을 명시하고 동결 대상과 현재 source/membership, 55수용 조건, R06/R07 영향, X09 잔여 시험과 복원 증거를 확인한다.
감사 판정·근거·미해결 finding을 새로운 감사 기록에 남기며 과거 자체 검증을 독립 PASS로 옮겨 적지 않는다.
감사 pass에서는 제품 코드를 변경하지 않고 [GEX의 독립성과 단계 분리](../../../goal_execution_pipeline%281%29.md)를 지킨다.

**R08~R12 감사가 통과하고 필수 선행이 충족되면 같은 요청 범위에서 R13~R16 계획·구현으로 이어서 진행한다.**
감사 종료 뒤 구현 재개 승인을 기다리지 않는다. 이미 유효하고 현재 소스에 적용 가능한 감사가 있으면 인수하고 중복 감사를 피한다.
감사 불합격·미해결 차단 finding·필수 환경 증거 부재 등으로 선행을 충족하지 못하면 근거와 필요한 교정을 보고하며,
통과로 간주해 종속 구현에 착수하지 않는다. X09만 통과시켜 전체 독립 감사를 대신하지 않는다.
이번에 포함한 감사는 R08~R12 선행 감사다. R13~R16 구현 후 최종 독립 감사는 계속 별도 요청 대기를 종료점으로 둔다.

## 3. 범위와 구현 순서

구현 기준은 현재 유효한 계약, 아래 카드, §4의 최신 합의다. 카드의 `proposed_not_implemented`는 작성 당시 상태이며
현재 코드·실행 기록을 대신하지 않는다. 과거 동결 문서를 소급 수정해 최신 정책이나 감사 결과를 끼워 넣지 않는다.

| 순서 | 책임과 필수 소비자 | 선행 / 수용 기준 |
| --- | --- | --- |
| [R13](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R13.ko.md) | 공통 생성 kernel의 actor/trap/field object adapter, 배치·점유·정지·진화·HP 표현·source 철수·수명, 비용과 생성 원자성, 보상/웨이브 권한 분리 | R04/05/07/10/12; T15/23/27; AT13 |
| [R14](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R14.ko.md) | 유물 사본/3슬롯/획득·이전·회수·분쇄·저장, 최소 수치 합성과 개인별 immutable plan, cast/queued/projectile/area/readiness/preview/평가 입력 연결 | R01/05/07/10; T12/13/18/20/27; AT14 |
| [R15](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R15.ko.md) | 같은 효과의 태그 selector, 유한 활성 조건, 생성 효과 재선택, typed Add/Scale/Replace와 구조 recipe, 최종 graph 검증·출처·설명·캐시 무효화 | R11/12/14; T13/27/28 |
| [R16](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R16.ko.md) | event 구독/queue dispatcher, 경로별 재방문, fork/join, 지연·주기·후속 생성의 인과 전달, 실행 예산·회수, 침식 부가 효과 억제 유지 | R04/05/14/15; T14/19/23/28; AT15 |

[계약 K07~K09](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/CONTRACTS.ko.md),
[의존성과 AT13~AT15](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/DEPENDENCIES.ko.md),
[원 수용표](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/ACCEPTANCE.ko.md)를 함께 인수한다.
R13은 모든 객체를 Unit 하나로 강제하지 않는다. R14는 관리 기능만 만들고 수치 합성의 실제 실행을 미루지 않는다.
R15의 정적 활성과 R16의 전투 반응은 별개다. R10의 Flow Parallel(All) 완료 합류를 R16의 인과 합류 구현으로 간주하지 않는다.

이번 유물 기반에는 침식형의 데이터 작성, 사본별 변환 상태·검증·원자적 적용, 저장과 전투 효과 연결을 포함한다.
상점/보상/Event의 전체 콘텐츠 배치·상품 구성은 제외하고 현재 정상 Core 명령과 대표 외부 RON으로 변환 경로를 입증한다.
촉매제 경로와 무료 Event 경로가 같은 대상 검증을 사용하도록 하되, 미정 가격·재고·빈도를 공식값으로 만들지 않는다.

## 4. 이번에 인수할 합의 정책

원문은 [유물 규칙](../../../game_rules_next/relics.md)과 [미결정 사항 NEXT-08/09/10](../../../game_rules_next/open-decisions.md)다.
차기 룰북 중 이 절에서 지정한 유물 정책을 이번 구현의 입력으로 채택한다. 다른 차기 원정 규칙까지 함께 적용하지 않는다.
옛 PG02의 join/root 미정 문구는 아래 합의로 보완된다. 합의가 있다는 것과 실제 구현·검증이 끝났다는 것은 구별한다.

### 4.1 사본 관리와 침식

- 직원마다 유물 공용 3슬롯. 같은 사본의 이중 장착 금지, 중복/비호환 제한은 직원별이며 비활성 사본에도 적용한다.
  조건이 없어도 슬롯에 남고 조건 회복 시 활성화한다. 전투 밖 장착·해제·교체·직원 간 이전은 무료이며 전투 중 변경은 금지다.
- 사망 시 유물을 공용 보관함으로 회수한다. 유물은 판매하지 않고 확인 후 분쇄하면 기존 장비 가루를 얻는다.
  기본 분쇄량은 미정이다. 무기 강화·파편 개화의 투자 재료 50% 환급을 유물 분쇄량에 자동 적용하지 않는다.
- 해당 유물 RON에 효과와 패널티를 갖춘 침식형이 작성되어 있어야 침식할 수 있다. 런타임이 공통 무작위 패널티를 발명하지 않는다.
- 한 사본은 일반→침식으로 한 번만 변환한다. 촉매제와 Event 모두 같은 제한을 적용한다. 이미 침식한 사본의 재적용은
  추가 소비·중첩·단계 상승 없이 거절한다. 같은 종류의 다른 사본은 각각 변환할 수 있다.
- 적용 전 대상·효과·패널티·소비·원상복구 불가를 조회할 수 있어야 하고, 확정 시 재검증한다. 취소·부적격·실패는 소비나 변형을 남기지 않는다.
  촉매제 사용은 해당 소비와 변환을 함께 처리한다. 침식 Event 변환에는 촉매제·추가 재화 소비를 붙이지 않는다.
- 원정 중 정화/원상복구는 없다. 해제·이전·사망 회수로 침식이 사라지지 않는다. 상태는 사본에 귀속한다.
  기존 체크포인트/후퇴 계약에서 소비와 변형 결과를 같은 시점으로 복원한다. 이를 현재 미지원인 전투 중 프로세스 저장 신설로 확대하지 않는다.
- 침식된 유물 상태와 R09 파편 침식의 특수 kill·suppression은 다른 기능이다. 희귀도와 침식 상태도 같은 축으로 합치지 않는다.

### 4.2 수치 합성과 활성 조건

- **같은 효과의 피해량**에 대한 유물 고정 증가를 먼저 합산하고, **동일 종류**의 비율 증폭을 합산 적용한다.
  예: `(100 + 20) × (1 + 0.2 + 0.3) = 180`. 예시값은 시험용이다. 다른 속성·저항·전체 피해 계산식에 일괄 적용하지 않는다.
  공통 피해 증폭 gameplay 상한은 두지 않는다. 실행 자원 안전 한도와 혼동하지 않는다.
- 무기·파편 등 실제 시작 효과와 조건 없이 부여되는 효과에서 출발해, 조건을 만족한 효과를 추가하고 더 추가할 것이 없을 때까지 판정한다.
  시작 효과 없이 서로의 조건만 만족하는 A/B는 비활성이다. 시작 효과를 제거하면 현재 구성에서 다시 계산하며 과거 활성으로 자립하지 않는다.
- 같은 장착 집합은 장착 순서와 무관하게 같은 결과를 낸다. 전용 파편·장비·형태 조건은 일반 태그로 대체하지 않는다.
  여러 효과의 태그를 합쳐 하나의 효과가 조건을 만족한 것처럼 처리하지 않는다. 생성 효과에도 출처·태그를 보존한다.
- 충돌하는 구조 변경은 기존 비호환 정책과 typed write set으로 판정한다. 서로 다른 Replace를 UUID/장착 순서의 마지막 값으로 해결하지 않는다.
  부정 조건 순환 등 유한성을 증명하지 못한 authoring은 명시적으로 거절한다. 정상적인 긍정 상호 조건을 전부 순환 오류로 거절하지 않는다.
  비피해 속성의 산술·개별 recipe는 기존 계약과 대조해 구현자가 근거와 시험을 기록한다.

### 4.3 전투 연쇄

- 방문 키는 **유물 사본 + 그 사본의 효과**다. 같은 경로의 같은 키 재방문만 막는다. 다른 사본·다른 효과·독립 기본 공격은 허용한다.
- fork는 공통 조상 기록을 이어받고 이후 형제 갈래의 기록은 독립적으로 확장한다. 전투 전체 visited 집합이나 cooldown으로 대체하지 않는다.
- 실제 여러 경로의 결과로 하나의 효과를 만드는 join은 **실제 참여 경로 전체의 조상 기록을 합집합으로 계승**한다.
  어느 참여 경로에서든 거친 같은 키는 합류 이후 재발동하지 않는다. 새로운 효과의 발동과 합류 결과 자체의 피해는 정상 처리한다.
  동시 적중·피해량 표시 합산만으로 join을 만들지 않으며, 참여하지 않은 경로의 기록을 섞지 않는다.
- 독립적으로 반복하는 주기 효과는 매 occurrence 새 root를 시작한다. 적중/반응으로 생긴 추가 공격·지연 폭발·지속 피해는 원인 경로를 계승한다.
  타이머 실행·시간 경과·투사체 도착만으로 root를 초기화하지 않는다. 새 root 권한은 producer의 실제 발생 원인으로 판정한다.
  임의 authoring flag로 기록을 지우지 않는다. 소환물의 모든 공격을 자동으로 독립 root 또는 부모 root로 일괄 분류하지 말고 producer 계약을 확인한다.
- 로그를 지워도 인과 제한은 유지한다. 전투 종료·취소·TTL 등 실제 수명에 맞춰 회수한다. 직접 재귀 호출 대신 정상 queue dispatch를 사용한다.
- R09 특수 kill이 억제한 부가 효과를 R16 구독으로 재활성화하지 않는다. 기존 OnAttack/OnHit·실제 HP-loss 인정 계약을 보존한다.

## 5. 계획과 코드 조사

[P2G](../../../policy_to_goal_pipeline_revised_v2.md)와 [GEX](../../../goal_execution_pipeline%281%29.md)의 Reader Contract를 따른다.
동일 범위의 유효한 정식 계획이 있으면 재사용한다. 없다면 권장 위치는 `core_enhanced/docs/goals/content_authoring_r13_r16_runtime/`다.
이 경로는 제안이며 이 문서로 Goal/Handoff가 생성되거나 검증된 것은 아니다.

권장 구성은 **Single Goal 안의 WP13~WP16과 최종 통합 검증**이다. 실제 P2G에서 크기·의존성·원자적 전환을 검토하고
계획/Goal/Handoff 및 기존 validator를 실제로 생성·실행한다. 카드·링크 검사만으로 P2G PASS를 기록하지 않는다.
유효한 계획과 선행이 준비되면 GEX-00/10을 이어간다. 통상적인 내부 WP마다 새 승인을 요구하지 않는다.
실제 multi-phase가 필요하면 필수 Phase 감사를 생략할 수 없으므로 그 경계를 명시한다.

현재 조사 시작점은 `core_enhanced/src/` 아래 다음 경로다. 최종 수정 목록이 아니며 구현 전 생산자와 모든 소비자를 추적한다.

| 범위 | 코드 조사 시작점 |
| --- | --- |
| R13 | `game/battle/core/{spawn_kernel,summon_runtime}.rs`, `game/data/summoned_unit_data.rs`, battlefield/점유, R12 HP cost |
| R14 | `game/resources/{inventory,equipment_loadout,economy}.rs`, `game/world/grant_state.rs`, authoring compiler, combat setup, `game/battle/core/skill_runtime/{cast,flow,projectile,area}.rs`, preview/readiness, snapshot |
| R15 | authoring selector/compiled effect/SourceMap, R11/R12 `game/battle/core/{sustained,resource_runtime}.rs`, R14 effective plan과 캐시 |
| R16 | `game/battle/core/{triggers,queue,sim,types,corrosion}.rs`, `game/battle/{recording,skill_source}.rs`, projectile/area/timer/후속 생성 |

입력 RON→parser→validation→compiled plan→정상 Core 명령→전투 결과→저장/조회 표를 작성한다.
새 schema는 parse만 하고 실행하지 않는 상태로 게시하지 않는다. 미지원 조합은 명시적으로 진단한다.
source/copy/form/owner 수명, 개인 plan의 지연 payload 보존, 새 상태의 allocation 전 admission·retained·clone·rollback을 처음부터 설계한다.
R13 진화 HP 보존·source 철수 세부는 기존 콘텐츠 계약을 먼저 찾고, 모든 소환물에 하나의 임의 규칙을 강제하지 않는다.
R12의 이미 승인된 HP 비용·보호·생존 정책을 옛 PG 미정 문구만 보고 다시 미정으로 되돌리지 않는다.

### 구현 중 정책 결정 권한

사용자는 이번 R13~R16 구현에서 새로 결정해야 하는 정책을 **구현자가 재량으로 결정하고, 문서에 남긴 뒤 구현을 계속**하도록 위임했다.
이 위임은 통상적인 기술·세부 정책에만 한정하지 않는다. 이번 범위 안의 미정 gameplay 정책도 포함하며,
플레이 의미가 크다는 이유만으로 질문하거나 승인 대기로 중단하지 않는다. 기존의 '큰 정책 선택은 하나씩 질문' 지침은 이번 구현에 적용하지 않는다.

기존 사용자 합의와 명시된 구현 범위를 우선하면서 미정 항목·공백·모호한 해석을 결정한다.
결정마다 항목, 선택한 정책, 근거와 대안, 플레이·데이터·저장·호환성 영향, 검증 조건과 변경 가능성을
`EXPERIMENT_NOTES.md` 및 해당 정책/설계 문서에 기록한다. 사용자 직접 결정과 구현자 위임 결정을 구별하고,
`PLAN.md`에서 기록 위치와 구현 상태를 연결한다. 별도 결정 로그를 중복 생성하지 않는다.
정식 P2G 계약에 영향이 있으면 필요한 산출물과 수용 조건도 갱신·검증한다. 동결된 계약은 원문을 덮어쓰지 않고 successor로 반영한다.
미정 정책을 구현에서 조용히 기본값으로 처리하거나, 미지원으로 남긴 채 완료하지 않는다. 결정·문서화·검증 후 남은 구현을 이어간다.
밸런스 예시는 플레이 검증 대상임을 유지하며 사용자 확정값으로 소급하지 않는다.

이는 구현 범위 안의 정책 결정 위임이다. §2의 선행 감사·증거 확인과 최종 독립 감사 인계 경계는 그대로 적용한다.

## 6. 필수 수용 조건

아래는 이번 범위의 최소 추가 수용표다. §3의 원 T 조건을 대체하거나 축소하지 않는다.
구현 계획에서 각 ID에 실제 입력·공개 실행 경로·oracle·selector·증거 위치를 연결한다.

| ID | 확인할 동작과 결과 |
| --- | --- |
| A13-01 | 아군/적 actor, 정지 trap/field object를 공개 RON에서 생성하고 각각의 이동·피격·점유 capability를 확인 |
| A13-02 | 생성 성공과 HP 부족·점유 실패·중복 요청·한도 실패에서 비용/ID/객체의 원자성 및 idempotency 확인 |
| A13-03 | small→big 진화, segmented HP/HP fallback, 삭제·source 철수·TTL 후 stale 참조/예약 event의 무효화 확인 |
| A13-04 | actor 생성이 보상·wave 완료 권한을 임의 취득하지 않으며 기존 보스/웨이브 생성 회귀가 유지됨 |
| A14-01 | 3슬롯·사본 독점·직원별 중복/비호환·순서 변경·조건 없는 장착/휴면·무기 해제·무료 이전·사망 회수 |
| A14-02 | 판매 거부, 분쇄 확인/취소/실패, 기존 가루 지급과 사본 제거의 원자성·저장/복구 |
| A14-03 | 고정 증가와 동일 종류 증폭의 180 예시, 서로 다른 직원이 같은 SkillId를 써도 개인별 실제 피해·preview·평가 입력이 분리됨 |
| A14-04 | cast/queued/projectile/area/readiness의 개인 plan 사용, source 문맥 종료 뒤 지연 payload가 전역 원형으로 되돌아가지 않음 |
| C-01 | 침식형 없는 유물 거부, 작성된 효과/패널티 조회, 촉매제·Event 모두 같은 대상 검증, 취소/실패 시 무소비 |
| C-02 | 사본당 1회 변환, 재적용 무소비·무중첩, 같은 종류 다른 사본의 독립 변환, 일반/침식형 실제 전투 효과 차이 |
| C-03 | 해제/이전/사망 회수 후 침식 유지, 현재 지원 저장·후퇴/체크포인트 경계에서 소비와 변형을 함께 복원 |
| A15-01 | 새 폭발 부여→다른 유물 강화, 효과 단위 태그와 전용 identity/형태 AND 조건, 장착 순서 permutation 동일 plan |
| A15-02 | 시작점 없는 A/B 상호 조건은 휴면, 시작점 추가 시 활성, 제거 시 재휴면; 자기 부트스트랩 금지와 유한성 |
| A15-03 | NOT cycle·충돌 Replace 명시 거부, typed recipe의 write/preserve set, 피해 범위만 변경할 때 회복·carrier·출처·채널 수명 보존 |
| A15-04 | 카드의 C04/C05 및 포큐버스/태아 대표 recipe를 해당 계약으로 검증; 특정 예시의 ×2를 모든 전환에 일반화하지 않음 |
| A16-01 | A1.e→B1.e→A1.e만 차단, A2·A1의 다른 효과·독립 기본 공격 허용 |
| A16-02 | 공통 조상은 계승하되 형제 갈래의 이후 방문은 독립; 실제 join은 참여 경로 합집합, 비참여 경로는 제외 |
| A16-03 | 동일 시각 별도 적중은 join 아님; 진짜 join 결과 피해 정상, 이미 거친 효과 재발동 차단, 미방문 효과 허용 |
| A16-04 | 지연 폭발·지속 피해의 각 tick·projectile/area/후속 생성이 원인 제한 보존; 독립 주기 producer는 매회 새 연쇄 |
| A16-05 | 로그 eviction/trace off에서도 제한 유지, 수명 종료 후 회수, fanout/instance/event 한도에서 기존 typed fault·원자성 유지 |
| A16-06 | 일반 kill과 R09 침식 kill 비교로 부가 효과 suppression 보존, 기존 OnHit/HP-loss 계약 회귀 |
| X-01 | 공개 외부 RON→정상 명령→실제 전투→결과/조회까지 R13~R16이 함께 작동하는 통합 시나리오 |
| X-02 | 새 런 귀속 상태의 현행 durable 저장과 fresh Core 복원, attempt entry/result/retreat, 실패·취소·재시도의 무부분 commit |
| X-03 | R06/R07/R08~R12 영향 회귀, Snapshot/Live·source/copy/form·R11 channel·R12 비용/보호/실제 HP 손실 계약 보존 |
| X-04 | 전체 검사·공개 CLI·PC36/R05, 최종 source currentness와 격리 정/역 복원, 실제 수용표·manifest·인계서 일치 |

실제 join과 독립 주기/파생 효과는 이번 필수 대상이다. 정책이 닫혔어도 실행 경로를 미지원으로 남기면 R16 전체 완료가 아니다.
저장 책임은 각 새 상태를 도입한 WP에서 닫는다. 마지막 통합 단계나 R20으로 일괄 미루지 않는다.

## 7. 검증·기록·동결

작업 루트는 `/mnt/f/work/simulator`, Rust 패키지는 `game_core`다. 기존 환경을 확인하고 다음 최종 검사 명령을 기준으로 한다.

```bash
cargo check -p game_core --all-targets --offline --locked
cargo test -p game_core --all-targets --offline --locked
cargo test -p game_core --all-targets --features test-fixtures --offline --locked
cargo check -p game_core --all-targets --features event_log_checks --offline --locked
cargo build -p game_core --release --bins --offline --locked
cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op
cargo fmt --all -- --check
```

WP별 focused 시험과 영향 회귀를 수행하고, 최종 통합 source에서 broad 검증을 실행한다. 기본 병렬 시험을 유지한다.
원명령·선택된 시험·실제 건수·exit code·source를 기록한다. 0개 시험·중단·환경 실패를 PASS로 기록하지 않는다.
기존 fmt/clippy 문제는 baseline과 구별하며 무관한 변경이나 새 allow로 감추지 않는다.
공개 release CLI의 실제 validate/describe/explain/simulate 경로와 trace on/off 결과 일치를 검증한다.
PC36/R05는 기존 workload/cap/hard guard를 보존한다. R05 concurrent peak의 원 cap **9,030,188 bytes 단독 초과만 경고**이며
다른 비용·결정론·parity 실패를 완화하지 않는다. **20분 시험은 제외**하고 유한 입력·fake clock·경계 부하를 사용한다.

제품 변경 전 실제 dirty/untracked bytes와 membership, worktree/index 차이를 보존한다. 사용자 작업을 reset/clean/stash하지 않는다.
현재 위치는 `PLAN.md`, 실행 원자료는 `EXPERIMENTS.md`, 기술 판단은 `EXPERIMENT_NOTES.md`로 연결해 유지한다.
새 로그는 신규 실행 위치에 기록한다. 선행 동결 target과 기존 runner의 고정 출력 위치를 덮어쓰지 않는다.
내부 WP 경계 source와 실패/중단/교정 이력을 남기고, 최종 source에 모든 조건의 증거를 다시 연결한다.

before/after archive 또는 동등한 byte 보존, 추가·삭제를 포함한 정/역 patch, SHA-256 manifest와 currentness를 만든다.
격리 디렉터리에서 양방향 복원을 확인한다. 원본 작업 트리에 검증용 역 patch를 적용하지 않는다.
동결 후 source가 바뀌면 successor를 만들고 영향 검증을 다시 연결한다. 과거 target을 수정하지 않는다.
인계에는 수용표, 공개 입력, 명령/원자료, 미지원/한계, 이전 감사 적용 범위와 새 감사 필요 범위를 포함한다.
종료 시 R08~R12 선행 감사 인수/실행 결과와 R13~R16 최종 독립 감사 미수행을 구별해 명시한다.
R13~R16이 Single Goal이면 GEX-20 대상이며 존재하지 않는 Master/GEX-30을 만들지 않는다.

## 8. 이번 범위 밖

차기 원정의 지도·억제력·황금가지·귀환·대여·창고 경제 전체, 전투 도중 프로세스 종료 후 durable 재개 신설,
R17 연결망/감염/공간 집계 전체, R18 보스 전체 개편, R19~R24 전체 완료는 제외한다.
GUI/편집기·Unity/game_server 확장·배포·실험실 stress/metrics·안전값 RON 분리와 범용 스크립트 언어도 제외한다.
대표 외부 RON으로 기반 기능을 증명하는 것과 공식 콘텐츠 전종·밸런스 완료를 구별한다.
예산 초과의 새 자동 스킬 취소 정책을 만들지 않고 기존 typed fault/rollback을 유지한다.
