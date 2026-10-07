# 차기 룰북 재구성·후속 합의 통합 기록

날짜: 2026-09-26. 사용자 요청: 짧은 게임 설계서와 주제별 규칙을 구성하고 후속 합의까지 통합.
문서 편집·목표 정책 정리의 기록이며 현재 canonical 교체·코드 구현·밸런스 검증을 선언하지 않는다.

## 입력과 문서 상태

이후 정책 변경: [2026-09-27 회수·대여와 강화·개화 보존 합의](game_rulebook_next_recovery_2026-09-27.md).
추가 변경: [2026-09-27 시설 지도·원정 흐름·황금가지·흉탄 분류 합의](game_rulebook_next_expedition_2026-09-27.md).
고정 관문표와 흉탄의 일반 보스 분류 등 아래의 당시 상태는 이 후속 합의로 대체됐다.
아래 재구성 기록은 당시 통합 이력이며 이후 변경의 근거는 해당 결정 기록을 따른다.

| 입력 | 사용 범위 |
| --- | --- |
| [9월 10일 초안 보존본](../archive/game_rulebook_next_2026-09-10.md) | 기존 규칙·합의·미정의 재구성 출발점. 본문을 삭제해 승인 경위를 잃지 않도록 보존 |
| [현행 룰북](../archive/game_rulebook_legacy_2026-10-04.md) | 현재 gameplay authority와 구현 세부·기존 모드/반복 정책의 비교 기준 |
| [후속 정책 합의](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md) | 9월 10~13일 직접 합의·정정·후속 수락·명시 위임에 따른 기본값 |
| [유물 시스템 노트](../tmp/run_relic_system_design_notes_2026-08-21.ko.md), [진행·성장 기록](../tmp/run_progression_growth_policy_notes_2026-09-07.ko.md) | 이전 초안이 통합한 결정의 원래 출처 |
| [런 구조 통합 설계안](../tmp/run_structure_integrated_design_2026-09-12.ko.md) | 합의·추천·시험값의 경계 확인. 사용자 합의로 표시된 범위만 사용 |
| [초기 재설계안](../tmp/run_progression_relic_redesign_proposal_2026-09-07.ko.md), [9월 12일 추가안](../tmp/run_structure_design_proposal_2026-09-12.ko.md) | 비교 이력. 신규 정책 승인 근거로 사용하지 않음 |

재구성 전 `game_rulebook.md` SHA-256:
`bc8f4196624e9e67fb31eaff9265b0829c6a08ddeb86cc98097cd9b9548e9f75`.
현행 룰북 기준 SHA-256:
`482e964cc7015945dec29cf28293d58d9e315c8db47ef60e0f4765e3f1a36c6d`.
보존본은 안내 문단 추가와 상대 링크 이동만 수행했다. 그 안의 ‘미정/미반영’은 당시 상태다.

## 대체 관계

| 이전 표현 | 통합한 최신 상태 |
| --- | --- |
| 6층/8층 미정 | 8층 배치 확정. 모드·종료·연장 연결은 여전히 미정 |
| 아종·특수·숨겨진·준최종을 별도 보스 등급으로 취급 | 지정 14개 항목은 일반 보스, 준최종 폐기. 등장·승패·보상 조건은 유지 |
| 파편 가루를 개화 재료로 확정 유지 | 파츠 사용 확정, 가루 동반/대체는 9월 12일 후속 검토로 미정 |
| 연속 위치를 단순 연출로 설명 | 실제 거리·이동·충돌 판정 상태로 정정. 저지 때 판정 타일 유지 |
| 이동·교대에 방어 태세 선행 요구 | B01: 태세 무관 요청·성공 후 강제 방어 |
| B01의 방어 유지·자동 공격 없음 | B20: 강제 방어 1.5초 뒤 자동 공격. 이전 태세 복원 아님 |
| 작업을 Defense + FieldWorking으로 차기에도 확정 | B21: 즉시 작업 연출만 확정, 내부 모드·자동 공격 타이머 관계 유보 |
| 일반 포커스·로스터 선택 감속 미정 | B02·03: 0.5배속·이전 속도 복원, 위임된 정지/입력 기본값 채택 |
| 카메라 보완·휠·방향키 미정 | 후속 수락·B23·방향키 합의 적용 |

원문에서 FinalBoss의 ‘결과 확정 전 후퇴’와 공통 BattleEnd 전 제한이 혼재한 것은 NEXT-15로,
시도 소진으로 방을 완료하는 경우와 Breakdown 카운터의 관계는 NEXT-03으로 기록했다.
이 편집에서 새 예외를 결정하지 않았다.

## 기존 초안의 이동 위치

규칙을 간결하게 다시 쓰면서 게임플레이 조건·수치·예외를 새 페이지에 보존했다.
표에서 현행 정본·계약으로 연결한 항목은 구현 상세 또는 비교 이력이며 차기 본문에 중복하지 않는다.
보존본 링크에서 수정 전 문장을 확인할 수 있다. 이 heading 표는 문장별 의미 검토를 대체하지 않는다.

| 이전 절 | 현재 위치 | 처리 |
| --- | --- | --- |
| [문서 책임과 읽는 법](../archive/game_rulebook_next_2026-09-10.md#문서-책임과-읽는-법) | [읽기](../game_rulebook.md) | 설계서의 흐름·탐색으로 재작성 |
| [통합 입력과 결정 출처](../archive/game_rulebook_next_2026-09-10.md#통합-입력과-결정-출처) | [읽기](#입력과-문서-상태) | 출처·문서 이행 기록 |
| [통합된 주요 변경](../archive/game_rulebook_next_2026-09-10.md#통합된-주요-변경) | [읽기](../game_rulebook.md) | 설계서의 흐름·탐색으로 재작성 |
| [탐색](../archive/game_rulebook_next_2026-09-10.md#탐색) | [읽기](../game_rulebook.md) | 설계서의 흐름·탐색으로 재작성 |
| [게임 개요](../archive/game_rulebook_next_2026-09-10.md#게임-개요) | [읽기](../game_rulebook.md) | 설계서의 흐름·탐색으로 재작성 |
| [게임 정체성](../archive/game_rulebook_next_2026-09-10.md#게임-정체성) | [읽기](../game_rulebook.md) | 설계서의 흐름·탐색으로 재작성 |
| [게임 모드](../archive/game_rulebook_next_2026-09-10.md#게임-모드) | [읽기](../game_rules/run-and-map.md#현행-모드와의-연결) | 8층 합의 반영, 현행 모드와 연결은 미정 |
| [핵심 플레이 루프](../archive/game_rulebook_next_2026-09-10.md#핵심-플레이-루프) | [읽기](../game_rulebook.md) | 설계서의 흐름·탐색으로 재작성 |
| [시작 직원 선발](../archive/game_rulebook_next_2026-09-10.md#시작-직원-선발) | [읽기](../game_rules/run-and-map.md#직원-선발과-채용) | 공통 규칙 유지, 차기 인원 미정 |
| [탐사와 노드](../archive/game_rulebook_next_2026-09-10.md#탐사와-노드) | [읽기](../game_rules/run-and-map.md#한-런의-구성) | 8층·분류·연속 조우 합의 통합 |
| [필수 관문과 선택 전투](../archive/game_rulebook_next_2026-09-10.md#필수-관문과-선택-전투) | [읽기](../game_rules/run-and-map.md#한-런의-구성) | 8층·분류·연속 조우 합의 통합 |
| [Floor 맵과 노드 선택](../archive/game_rulebook_next_2026-09-10.md#floor-맵과-노드-선택) | [읽기](../game_rules/run-and-map.md#시설-지도와-방-선택) | 게임 규칙 유지, 배정/좌표 구현은 계약 참조 |
| [Certified 전투력과 Floor 난이도](../archive/game_rulebook_next_2026-09-10.md#certified-전투력과-floor-난이도) | [읽기](../game_rules/growth-and-rewards.md#전투력과-난이도) | 기존 모델과 유물 목표·미정 분리 |
| [Safe Node와 Safezone](../archive/game_rulebook_next_2026-09-10.md#safe-node와-safezone) | [읽기](../game_rules/run-and-map.md#gate와-비전투-준비) | 노드 역할과 준비 행동 유지 |
| [Event Node](../archive/game_rulebook_next_2026-09-10.md#event-node) | [읽기](../game_rules/run-and-map.md#event) | 판정·진행·전투 예외 유지 |
| [일반 인벤토리](../archive/game_rulebook_next_2026-09-10.md#일반-인벤토리) | [읽기](../game_rules/run-and-map.md#물품과-소비-아이템) | 게임 규칙 유지, wire selector는 계약 참조 |
| [소비 아이템](../archive/game_rulebook_next_2026-09-10.md#소비-아이템) | [읽기](../game_rules/run-and-map.md#물품과-소비-아이템) | 게임 규칙 유지, wire selector는 계약 참조 |
| [상점](../archive/game_rulebook_next_2026-09-10.md#상점) | [읽기](../game_rules/run-and-map.md#상점) | 엔케팔린·세 상점 후속 합의 통합 |
| [노드 진입 전 브리핑](../archive/game_rulebook_next_2026-09-10.md#노드-진입-전-브리핑) | [읽기](../game_rules/run-and-map.md#진입-전-브리핑) | 무료 정보 유지 |
| [전투](../archive/game_rulebook_next_2026-09-10.md#전투) | [읽기](../game_rules/combat.md#전투-목표와-전장) | 게임 규칙 유지, 설정 DTO는 계약 참조 |
| [전투 모드](../archive/game_rulebook_next_2026-09-10.md#전투-모드) | [읽기](../game_rules/combat.md#전투-목표와-전장) | 게임 규칙 유지, 설정 DTO는 계약 참조 |
| [저지 판정과 적의 타일](../archive/game_rulebook_next_2026-09-10.md#저지-판정과-적의-타일) | [읽기](../game_rules/blocking-and-targeting.md#타일과-연속-위치) | 연속 좌표의 실제 판정 역할 정정 |
| [판정용 타일과 연출 위치](../archive/game_rulebook_next_2026-09-10.md#판정용-타일과-연출-위치) | [읽기](../game_rules/blocking-and-targeting.md#타일과-연속-위치) | 연속 좌표의 실제 판정 역할 정정 |
| [최초 접근과 통과 중 새 저지](../archive/game_rulebook_next_2026-09-10.md#최초-접근과-통과-중-새-저지) | [읽기](../game_rules/blocking-and-targeting.md#최초-접근과-통과-중-저지) | 앞 타일 끝자락 접근 후 저지 통합 |
| [저지 상대의 기본 공격](../archive/game_rulebook_next_2026-09-10.md#저지-상대의-기본-공격) | [읽기](../game_rules/blocking-and-targeting.md#저지-상대의-기본-공격) | 기본 공격 예외 유지 |
| [저지 해제와 남은 결정](../archive/game_rulebook_next_2026-09-10.md#저지-해제와-남은-결정) | [읽기](../game_rules/blocking-and-targeting.md#저지-해제) | NEXT-12 유지, 조사 수치는 미채택 |
| [전술 포인트](../archive/game_rulebook_next_2026-09-10.md#전술-포인트) | [읽기](../game_rules/combat.md#전술-포인트와-전투-시간) | 수치·시간·실패 경계 유지 |
| [Hard CC와 Silence](../archive/game_rulebook_next_2026-09-10.md#hard-cc와-silence) | [읽기](../game_rules/combat.md#군중-제어와-침묵) | 행동 제한 유지, 거절 enum은 계약 참조 |
| [철수와 재배치](../archive/game_rulebook_next_2026-09-10.md#철수와-재배치) | [읽기](../game_rules/combat.md#배치와-철수) | 회복·대기·초기화 유지, 유물 수명 연결 |
| [Attack/Defense 태세와 타일 효과](../archive/game_rulebook_next_2026-09-10.md#attackdefense-태세와-타일-효과) | [읽기](../game_rules/combat.md#공격과-방어-태세) | 일반 태세 유지, 강제 방어는 이동 규칙 소유 |
| [전술 이동과 아군 점유](../archive/game_rulebook_next_2026-09-10.md#전술-이동과-아군-점유) | [읽기](../game_rules/combat.md#전술-이동과-자리-교대) | B01·B20 최신 합의로 태세/자동 전환 갱신 |
| [수동 지정 스킬](../archive/game_rulebook_next_2026-09-10.md#수동-지정-스킬) | [읽기](../game_rules/combat.md#수동-스킬) | 대상 규칙 유지, 공통 포커스 연결 |
| [현장 작업](../archive/game_rulebook_next_2026-09-10.md#현장-작업) | [읽기](../game_rules/combat.md#현장-작업) | B21 즉시 연출 통합, 내부 상태 NEXT-14 유보 |
| [Pause와 직렬 전투 명령](../archive/game_rulebook_next_2026-09-10.md#pause와-직렬-전투-명령) | [읽기](../game_rules/combat.md#정지와-명령-처리) | 플레이 의미 유지, 생산자·revision·wire는 계약 참조 |
| [전투 시작과 Unity 경계](../archive/game_rulebook_next_2026-09-10.md#전투-시작과-unity-경계) | [읽기](../game_rules/combat.md#정지와-명령-처리) | 플레이 의미 유지, 생산자·revision·wire는 계약 참조 |
| [전장과 배치](../archive/game_rulebook_next_2026-09-10.md#전장과-배치) | [읽기](../game_rules/combat.md#전투-목표와-전장) | 게임 규칙 유지, 설정 DTO는 계약 참조 |
| [적 구성과 웨이브](../archive/game_rulebook_next_2026-09-10.md#적-구성과-웨이브) | [읽기](../game_rules/combat.md#웨이브와-적) | 적 구성·웨이브 의미 유지 |
| [Live Boss Runtime Reference](../archive/game_rulebook_next_2026-09-10.md#live-boss-runtime-reference) | [읽기](../archive/game_rulebook_legacy_2026-10-04.md#live-boss-runtime-reference) | 기준 시점 reference 세부는 현행 정본에 보존 |
| [기본 공격과 스킬 타겟팅](../archive/game_rulebook_next_2026-09-10.md#기본-공격과-스킬-타겟팅) | [읽기](../game_rules/blocking-and-targeting.md#일반-타겟팅과-피해) | 유효 적대 대상·피해 정책 유지 |
| [피해 타입과 피드백](../archive/game_rulebook_next_2026-09-10.md#피해-타입과-피드백) | [읽기](../game_rules/blocking-and-targeting.md#일반-타겟팅과-피해) | 유효 적대 대상·피해 정책 유지 |
| [Runtime 스탯·HP·사망 전이](../archive/game_rulebook_next_2026-09-10.md#runtime-스탯hp사망-전이) | [읽기](../game_rules/combat.md#전투-능력치와-hp) | 게임플레이 HP·사망 의미 보존, event 인과는 현행 정본·계약 참조 |
| [성장, 스킬 파편과 도감](../archive/game_rulebook_next_2026-09-10.md#성장-스킬-파편과-도감) | [읽기](../game_rules/growth-and-rewards.md#성장의-역할) | 차기 성장 역할·개념식 유지 |
| [성장 체계](../archive/game_rulebook_next_2026-09-10.md#성장-체계) | [읽기](../game_rules/growth-and-rewards.md#성장의-역할) | 차기 성장 역할·개념식 유지 |
| [공격력·주문력과 파편 피해](../archive/game_rulebook_next_2026-09-10.md#공격력주문력과-파편-피해) | [읽기](../game_rules/growth-and-rewards.md#성장의-역할) | 차기 성장 역할·개념식 유지 |
| [보상과 무기 강화 재료](../archive/game_rulebook_next_2026-09-10.md#보상과-무기-강화-재료) | [읽기](../game_rules/growth-and-rewards.md#재료와-엔케팔린) | 9월 12일 공통 비용·파츠와 가루 미정 통합 |
| [직원 XP와 성장](../archive/game_rulebook_next_2026-09-10.md#직원-xp와-성장) | [읽기](../game_rules/growth-and-rewards.md#xp와-보상-확정) | 직원별 지급·성장 유지 |
| [연구도 폐기와 스킬 파편 획득](../archive/game_rulebook_next_2026-09-10.md#연구도-폐기와-스킬-파편-획득) | [읽기](../game_rules/growth-and-rewards.md#파편-획득의-현행-비교-기준) | 8층 연결 전의 획득 비교 기준으로 보존 |
| [영구 도감, 튜토리얼 해금과 개인 기록](../archive/game_rulebook_next_2026-09-10.md#영구-도감-튜토리얼-해금과-개인-기록) | [읽기](../game_rules/results-and-save.md#영구-도감과-기록) | 영구/런 경계 유지, 새 모드 집계는 NEXT-01 |
| [Endless 환상체 격파와 반복 조우](../archive/game_rulebook_next_2026-09-10.md#endless-환상체-격파와-반복-조우) | [읽기](../game_rules/growth-and-rewards.md#파편-획득의-현행-비교-기준) | 8층 연결 전의 획득 비교 기준으로 보존 |
| [Elite assignment pool](../archive/game_rulebook_next_2026-09-10.md#elite-assignment-pool) | [읽기](../game_rules/growth-and-rewards.md#엘리트와-모노리스의-비교-기준) | 기존 pool·stage·잔재 보존, 보너스 목표는 별도 절 |
| [NormalBoss assignment pool](../archive/game_rulebook_next_2026-09-10.md#normalboss-assignment-pool) | [읽기](../game_rules/growth-and-rewards.md#보스-징조와-반복-조우) | 비교 기준 보존, 새 등급/8층 연결은 NEXT-01·05 |
| [Elite 모노리스와 강화 재현](../archive/game_rulebook_next_2026-09-10.md#elite-모노리스와-강화-재현) | [읽기](../game_rules/growth-and-rewards.md#엘리트와-모노리스의-비교-기준) | 기존 pool·stage·잔재 보존, 보너스 목표는 별도 절 |
| [숙달 Elite 소탕](../archive/game_rulebook_next_2026-09-10.md#숙달-elite-소탕) | [읽기](../game_rules/growth-and-rewards.md#숙달-소탕의-비교-기준) | 소탕·재획득·XP·아이템 기간 유지 |
| [장비](../archive/game_rulebook_next_2026-09-10.md#장비) | [읽기](../game_rules/equipment-and-fragments.md#장비와-무기) | 무기 역할·강화·타일 profile 유지 |
| [장비 전투 트리거](../archive/game_rulebook_next_2026-09-10.md#장비-전투-트리거) | [읽기](../game_rules/equipment-and-fragments.md#장비-효과의-발동) | 발동 의미·반격 경계 유지, 정확한 순서는 계약 참조 |
| [스킬 파편](../archive/game_rulebook_next_2026-09-10.md#스킬-파편) | [읽기](../game_rules/equipment-and-fragments.md#파편-장착과-능력) | active 파편·장착 조건 유지 |
| [개화 형태와 재료](../archive/game_rulebook_next_2026-09-10.md#개화-형태와-재료) | [읽기](../game_rules/equipment-and-fragments.md#개화와-형태-선택) | 형태 선택 유지, 가루/파츠 관계는 후속 미정 반영 |
| [유물](../archive/game_rulebook_next_2026-09-10.md#유물) | [읽기](../game_rules/relics.md#효과와-희귀도) | 합의 유지 |
| [목적과 효과 범위](../archive/game_rulebook_next_2026-09-10.md#목적과-효과-범위) | [읽기](../game_rules/relics.md#효과와-희귀도) | 합의 유지 |
| [성장과 희귀도](../archive/game_rulebook_next_2026-09-10.md#성장과-희귀도) | [읽기](../game_rules/relics.md#효과와-희귀도) | 합의 유지 |
| [슬롯·사본·소유](../archive/game_rulebook_next_2026-09-10.md#슬롯사본소유) | [읽기](../game_rules/relics.md#소유와-장착) | 합의 유지 |
| [장착 변경과 비활성](../archive/game_rulebook_next_2026-09-10.md#장착-변경과-비활성) | [읽기](../game_rules/relics.md#조건과-비활성) | 조건·대상·비호환 유지 |
| [태그·새 효과·증폭](../archive/game_rulebook_next_2026-09-10.md#태그새-효과증폭) | [읽기](../game_rules/relics.md#효과의-결합) | 합산·순환 제한·직원 단위 비호환 유지 |
| [연쇄 발동과 비호환](../archive/game_rulebook_next_2026-09-10.md#연쇄-발동과-비호환) | [읽기](../game_rules/relics.md#효과의-결합) | 합산·순환 제한·직원 단위 비호환 유지 |
| [판매·분쇄](../archive/game_rulebook_next_2026-09-10.md#판매분쇄) | [읽기](../game_rules/relics.md#판매와-분쇄) | 판매 금지·장비 가루 유지 |
| [체크포인트·전투 상태 수명](../archive/game_rulebook_next_2026-09-10.md#체크포인트전투-상태-수명) | [읽기](../game_rules/relics.md#상태-보존) | 스택 초기화와 전투당 사용량 분리 유지 |
| [침식된 유물](../archive/game_rulebook_next_2026-09-10.md#침식된-유물) | [읽기](../game_rules/relics.md#침식된-유물) | 합의·패널티 후보·미정 분리 유지 |
| [파편 침식형과의 경계](../archive/game_rulebook_next_2026-09-10.md#파편-침식형과의-경계) | [읽기](../game_rules/relics.md#에고-침식과의-경계) | 증폭/연쇄 제외·정상 웨이브 정리 유지 |
| [획득 정책](../archive/game_rulebook_next_2026-09-10.md#획득-정책) | [읽기](../game_rules/relics.md#획득과-표시) | 공급 보류·활성 효과 평가 유지 |
| [전투력과 표시](../archive/game_rulebook_next_2026-09-10.md#전투력과-표시) | [읽기](../game_rules/relics.md#획득과-표시) | 공급 보류·활성 효과 평가 유지 |
| [개별 콘텐츠의 합의 범위](../archive/game_rulebook_next_2026-09-10.md#개별-콘텐츠의-합의-범위) | [읽기](../game_rules/relics.md#개별-콘텐츠에서-합의한-범위) | 개별 합의/예시/미정 구분 보존 |
| [비전투 노드](../archive/game_rulebook_next_2026-09-10.md#비전투-노드) | [읽기](../game_rules/run-and-map.md#gate와-비전투-준비) | 노드 역할과 준비 행동 유지 |
| [SavePoint 노드](../archive/game_rulebook_next_2026-09-10.md#savepoint-노드) | [읽기](../game_rules/results-and-save.md#저장과-복구) | 회복·사용 횟수 유지, 저장 형식은 계약 참조 |
| [Maintenance 노드](../archive/game_rulebook_next_2026-09-10.md#maintenance-노드) | [읽기](../game_rules/equipment-and-fragments.md#정비) | 차기 작업 범위 유지 |
| [본사 연락 노드](../archive/game_rulebook_next_2026-09-10.md#본사-연락-노드) | [읽기](../game_rules/run-and-map.md#gate와-비전투-준비) | 노드 역할과 준비 행동 유지 |
| [결과와 실패](../archive/game_rulebook_next_2026-09-10.md#결과와-실패) | [읽기](../game_rules/results-and-save.md#결과-확인과-재도전) | 결과 확정 유지, 시도는 관문 연결 비교 기준 |
| [노드 결과 처리](../archive/game_rulebook_next_2026-09-10.md#노드-결과-처리) | [읽기](../game_rules/results-and-save.md#결과-확인과-재도전) | 결과 확정 유지, 시도는 관문 연결 비교 기준 |
| [후퇴](../archive/game_rulebook_next_2026-09-10.md#후퇴) | [읽기](../game_rules/results-and-save.md#전투-후퇴) | 복원 유지, 원문 판정/확정 경계 NEXT-15 기록 |
| [직원 condition, 전투 HP, 전투불능, 런 실패](../archive/game_rulebook_next_2026-09-10.md#직원-condition-전투-hp-전투불능-런-실패) | [읽기](../game_rules/results-and-save.md#직원-상태) | 부상·Breakdown·복구·사망 회수 보존 |
| [직원 Psychology](../archive/game_rulebook_next_2026-09-10.md#직원-psychology) | [읽기](../game_rules/results-and-save.md#psychology) | 성격/신뢰·기억 경계 보존 |
| [후속 콘텐츠·구현 authoring](../archive/game_rulebook_next_2026-09-10.md#후속-콘텐츠구현-authoring) | [읽기](../game_rules/open-decisions.md) | 이미 해결된 질문을 갱신하고 남은 경계 연결 |
| [고급 정책과 레거시](../archive/game_rulebook_next_2026-09-10.md#고급-정책과-레거시) | [읽기](../game_rules/growth-and-rewards.md#보스-징조와-반복-조우) | 비교 기준 보존, 새 등급/8층 연결은 NEXT-01·05 |
| [FinalBoss pool과 BossOmen Chain](../archive/game_rulebook_next_2026-09-10.md#finalboss-pool과-bossomen-chain) | [읽기](../game_rules/growth-and-rewards.md#보스-징조와-반복-조우) | 비교 기준 보존, 새 등급/8층 연결은 NEXT-01·05 |
| [레거시 제거 원칙](../archive/game_rulebook_next_2026-09-10.md#레거시-제거-원칙) | [읽기](../archive/game_rulebook_legacy_2026-10-04.md#레거시-제거-원칙) | 현행 구현·레거시 금지 원칙의 원래 위치로 연결 |
| [미정 사항과 미승인 제안](../archive/game_rulebook_next_2026-09-10.md#미정-사항과-미승인-제안) | [읽기](../game_rules/open-decisions.md) | 이미 해결된 질문을 갱신하고 남은 경계 연결 |
| [다음 결정이 필요한 경계](../archive/game_rulebook_next_2026-09-10.md#다음-결정이-필요한-경계) | [읽기](../game_rules/open-decisions.md) | 이미 해결된 질문을 갱신하고 남은 경계 연결 |
| [승인된 규칙으로 사용하지 않을 초기안](../archive/game_rulebook_next_2026-09-10.md#승인된-규칙으로-사용하지-않을-초기안) | [읽기](../game_rules/open-decisions.md#확정값으로-쓰지-않을-제안) | 후속 채택/대체를 반영한 제안 목록 |
| [이후 문서 갱신](../archive/game_rulebook_next_2026-09-10.md#이후-문서-갱신) | [읽기](#입력과-문서-상태) | 출처·문서 이행 기록 |

## 구현 상세와 콘텐츠의 소유 위치

- 명령 field·typed 거절·revision·원자적 적용·난수/UUID·이벤트·HP clamp/사망 순서·저장 검증은
  [런타임 계약](../core_runtime_contract.md)과 [현행 룰북의 해당 절](../archive/game_rulebook_legacy_2026-10-04.md)에 남긴다.
- 대상 필터·tie-break·발사체·단계별 재지정은 [타겟팅 계약](../skill_target_contract.md)이 소유한다.
  차기 저지와의 충돌은 NEXT-12로 연결한다.
- RON schema·fixture·배정·웨이브 검증·정확한 live ID는 [데이터 계약](../data_loading_contract.md)과
  [live 콘텐츠 색인](../skills/live_content_index.md)이 소유한다. placeholder/reference를 최종 콘텐츠로 승격하지 않는다.
- 개별 유물·파편·보스의 합의는 해당 콘텐츠 노트가 소유한다. 차기 룰북은 효과 범위와 출처를 연결한다.
- `timeline_delta`/`battle_delta`, 제거된 전투 mode·RON wrapper·Support/Rest support-choice의
  부활 금지는 [현행 레거시 원칙](../archive/game_rulebook_legacy_2026-10-04.md#레거시-제거-원칙)에 보존한다.
  `engineering_workflow.md`는 이번 작업 트리에서 찾지 못했다. 기존 보존본의 역사적 링크를 복구
  목적으로 새 계약 파일로 만들지 않았으며 새 차기 규칙의 탐색 경로에서도 사용하지 않는다.

## 후속 합의별 반영

모든 행은 차기 문서 통합 상태다. 원문에 기록된 UI 문서 반영 이외의 코드·서버·Unity 구현 상태를
이번 작업에서 검증하지 않았다. 이전 합의가 대체된 경우 마지막 답을 적용한다.

| 후속 기록 | 반영 위치 | 승인·대체 경계 |
| --- | --- | --- |
| [2026-09-10 / 최초 저지 위치와 두 좌표의 역할](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-10--최초-저지-위치와-두-좌표의-역할) | [규칙](../game_rules/blocking-and-targeting.md#타일과-연속-위치) | 직접 정정. 하위 코드 조사 0.75·50ms는 정책 수치로 미채택 |
| [2026-09-12 · 엔케팔린 공통 비용과 개화 재료·상점 구분](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-12--엔케팔린-공통-비용과-개화-재료상점-구분) | [규칙](../game_rules/growth-and-rewards.md#재료와-엔케팔린) | 직접 합의. 파편 가루 동반/대체는 미정 |
| [2026-09-12 · 일반 보스 분류 통일](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-12--일반-보스-분류-통일) | [규칙](../game_rules/run-and-map.md#보스-분류와-연속-조우) | 직접 합의. 개별 조우 조건·보상 유지 |
| [2026-09-12 · 8층 배치와 준최종 폐기·연속 조우의 시작 순서](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-12--8층-배치와-준최종-폐기연속-조우의-시작-순서) | [규칙](../game_rules/run-and-map.md#한-런의-구성) | 직접 합의·2층 확인. 예산표는 시험값 |
| [2026-09-13 · 전투 고스트 배치·방향 설정과 교대 자동 방어 전환](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--전투-고스트-배치방향-설정과-교대-자동-방어-전환) | [규칙](../game_rules/battle-controls.md#자리-교대) | 고스트·두 방향·최종 실행 유지. 태세 제한/방어 유지는 B01·B20으로 대체 |
| [2026-09-13 · 유닛 포커스 진입 시 자동 0.5배속](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--유닛-포커스-진입-시-자동-05배속) | [규칙](../game_rules/battle-controls.md#포커스와-배속) | 직접 0.5배속. 초기 기본안은 B02·03으로 확정/위임 범위 구분 |
| [2026-09-13 · 전투 로스터의 배치·미배치 두 섹션과 초상화 포커스](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--전투-로스터의-배치미배치-두-섹션과-초상화-포커스) | [규칙](../game_rules/battle-controls.md#로스터와-대상-확인) | 직접 합의와 수락한 표시 동작. 교대 초상화는 B11로 확정 |
| [2026-09-13 · B01 — 이동·교대는 태세 무관, 성공 후 강제 방어](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--b01--이동교대는-태세-무관-성공-후-강제-방어) | [규칙](../game_rules/combat.md#전술-이동과-자리-교대) | 태세 무관 이동/교대. 자동 공격 없음은 B20으로 대체 |
| [2026-09-13 · B02 — 포커스 해제 시 진입 전 배속 복원](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--b02--포커스-해제-시-진입-전-배속-복원) | [규칙](../game_rules/battle-controls.md#포커스와-배속) | 직접 확인: 전체 해제 시 이전 배속 |
| [2026-09-13 · B03 — 로스터 배치 감속과 일반 조작의 자율 적용](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--b03--로스터-배치-감속과-일반-조작의-자율-적용) | [규칙](../game_rules/battle-controls.md#포커스와-배속) | 로스터 감속 직접 확인, 정지·메뉴·입력 등은 명시 위임된 기본값 |
| [2026-09-13 · B20 — 이동·교대 후 강제 방어 1.5초와 자동 공격](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--b20--이동교대-후-강제-방어-15초와-자동-공격) | [규칙](../game_rules/combat.md#전술-이동과-자리-교대) | 직접 정정: 전투 시간 1.5초 후 자동 공격. 일반 대기/작업 관계 유보 |
| [2026-09-13 · B21 — 작업 칸 도착 즉시 작업 연출, 내부 모드 유보](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--b21--작업-칸-도착-즉시-작업-연출-내부-모드-유보) | [규칙](../game_rules/battle-controls.md#도착과-작업-연출) | 직접 정정: 즉시 작업 연출. 내부 모드·타이머 유보 |
| [2026-09-13 · B11 — 로스터 초상화로 교대 상대 선택과 아군 중첩 정정](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--b11--로스터-초상화로-교대-상대-선택과-아군-중첩-정정) | [규칙](../game_rules/battle-controls.md#자리-교대) | 직접 확정: 교대 초상화 입력·아군 중첩 금지 |
| [2026-09-13 · B22 — 공중에서 살짝 내려앉는 공통 배치 애니메이션](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--b22--공중에서-살짝-내려앉는-공통-배치-애니메이션) | [규칙](../game_rules/battle-controls.md#도착과-작업-연출) | 공통 착지 연출. 무적·공중 판정·추가 대기 승인 아님 |
| [2026-09-13 · B16 — 적 정보 확인용 포커스](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--b16--적-정보-확인용-포커스) | [규칙](../game_rules/battle-controls.md#로스터와-대상-확인) | 직접 수락: 적 정보 포커스와 공통 감속 |
| [2026-09-13 · B18 — 맵별 카메라 이동과 로스터 더블클릭 위치 찾기](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--b18--맵별-카메라-이동과-로스터-더블클릭-위치-찾기) | [규칙](../game_rules/battle-controls.md#카메라) | 직접 합의: 맵별 카메라·초상화 더블클릭 |
| [2026-09-13 · B18 후속 — 카메라 보완 규칙 채택](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--b18-후속--카메라-보완-규칙-채택) | [규칙](../game_rules/battle-controls.md#카메라) | 직접 수락: 입력 우선순위·경계·화면 밖 경고 등 보완 |
| [2026-09-13 · B23 — 이동 가능한 전투 맵의 제한된 휠 확대·축소](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--b23--이동-가능한-전투-맵의-제한된-휠-확대축소) | [규칙](../game_rules/battle-controls.md#카메라) | 휠 확대 직접 확인·UI 입력 기본값 위임 |
| [2026-09-13 · B17/B18 후속 — 방향키로 전투 카메라 이동](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md#2026-09-13--b17b18-후속--방향키로-전투-카메라-이동) | [규칙](../game_rules/battle-controls.md#카메라) | 방향키 직접 확정·연속 이동과 우선순위 기본값 위임 |
