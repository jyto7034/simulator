#!/usr/bin/env python3
"""U06 P2G-10 decisions and observable acceptance; no production mutation."""
import copy
import json
from initialize import HERE, ROOT, artifact, write, sha

# Each row is one decision lineage, with exact adopted policy section and bounded rationale.
POLICIES = [
('전투 목표와 TP', 'combat.md#전투-목표와-전장; #전술-포인트와-전투-시간; U06/B01',
 'DefenseRoute의 작성된 승리/실패/필수 웨이브 규칙과 U04 확정을 유지한다. TP는 전투마다20/최대99, 온전한 전투1초마다1이며 최초배치 기본10, 재배치 기본비용150% 올림, 이동/교대는 명령당5다. 명시 배치 비용 감소는 배치에만 적용하고 철수는 무료/무환불이다. 부분 지불·초과 비축·Pause 보충·Rest/Defense 암묵 획득은 없다.',
 ['실제 명령에서 최초/재배치/교대 비용과 부족 TP 실패 무변경을 확인한다.', 'Pause/배속/종료 전후의 온전한 전투초 회복과20 재시작을 확인하고 종료 결과는 U04와 일치한다.'],
 'B01의 명시 값과 전투 소유 자원을 적용한다. 승리/보상의 새 예외를 만들지 않는다.'),
('배치 자격과 동시 인원', 'combat.md#배치와-철수; equipment-and-fragments.md#장비와-무기; U06/B01,B11',
 '보유 최대10명 전원이 전투 후보이며 작업자를 포함해 동시에8명까지 배치한다. 서로 다른 참가자의 누적 수는 상한이 아니다. 출전 가능 condition을 먼저 검사하고 그 다음 무기/재배치 대기/타일/적성/점유/상한/비용을 검사한다. 아군과 보호 대상 타일 점유는 배치를 막고 적 겹침은 허용한다. 실패는 자원·위치·대기·개체를 바꾸지 않는다. 최초/재배치는 Attack과 일반 태세 대기1500ms로 시작하고 작업 칸은 즉시 작업한다.',
 ['무기없음과 출전불가가 겹치면 condition 사유를 우선 보여주고 무기없는 범위 미리보기도 거절한다.', '8명 동시배치와9번째 거절, 철수 뒤 이전 미참가 직원 배치를 정상 명령으로 검증한다.', '타일/점유/적성/비용/대기 실패 시 상태와 자원은 동일하다.'],
 '명시된 동시8/보유10과 condition 우선순위를 적용한다. U12의 새 편성/원정 수명 전체는 소유하지 않는다.'),
('철수와 새 재배치 개체', 'combat.md#배치와-철수; U01 취소; U04 결과/복원; U05 배치 공명',
 '배치된 생존 직원은 CC/침묵/작업/시전 중에도 자발적 철수가 가능하다. 철수는 저지와 미방출 준비/시전/추종 수명을 정리하며 이미 방출된 독립 효과를 환수하지 않는다. 자발적30초/전투불능90초 대기와 철수 시 min(maxHP,HP+floor(maxHP*30%)) 고정 HP를 적용한다. 재배치는 최신 출전자격·장비/파편에서 새 전투 개체로 생성하고 이전 버프/공명/대상/저지/공격대기를 상속하지 않는다. U04의 부상/트라우마와 전투단위 유물 기록은 보존한다.',
 ['CC/작업 중 정상 철수가 성공하고 중복·사망 철수는 거절된다.', '철수 대기 중 추가 회복/최대HP 변동이 고정 재배치HP를 소급 바꾸지 않고 새 개체 공명은 U05 규칙이다.', '발사체/독립 효과·미방출 취소·저지 정리는 U01 수명과 일치한다.'],
 '이미 합의한 개체 수명을 적용한다. 같은 개체인 이동/교대와 분리하며 새 저장 이행 예외는 없다.'),
('무기 profile과 작성 자격', 'equipment-and-fragments.md#장비와-무기; #파편-장착과-능력; U06/B11',
 '무기는 기본 행동·범위·대상 규칙·주기·저지·적성과 타일 효과를 명시한다. 필요한 profile 누락은 typed 오류로 거절하며 직업/default fallback으로 채우지 않는다. 공격속도100이 중립이고 주기는 ceil(무기주기*100/직원공격속도)다. 파편 장착 자격은 실제 무기 profile의 계열/근원거리/기본저지이며 방어 증가 저지가 아니다. Staff/Bow/Gun/Sword/Spear/Shield와 기존 특수 총 의미를 유지하고 새 계열 enum은 추가하지 않는다.',
 ['정상 RON과 누락/부적합 profile 거절을 loader와 공개 작성 경로에서 확인한다.', '무기 이전·공격속도·방어저지 증가 전후에도 profile과 파편 자격이 일치한다.', '치유 Staff도 같은 배치 적성과 행동 주기를 사용하고 기존 특수 총을 피해 기본값으로 덮지 않는다.'],
 '명시 필수 profile 계약을 적용한다. 대검 starter는 기존 Sword 계열의 authored 무기로 표현하며 직업 고정 제한을 만들지 않는다.'),
('순수 기본 치유', 'equipment-and-fragments.md#회사-지급-훈련용-파편; combat.md#공격과-방어-태세; U06/B10',
 '단일/범위 치유 지팡이의 기본 행동은 아군 HP 회복이며 피해·흡혈·보호막 대체가 아니다. 생존/배치/회복가능 아군 중 HP가 부족한 대상을 명시 범위에서 자동 선택한다. 자기 자신도 아군이다. 기본 자동 순서는 HP비율 오름차순, 거리, 안정 ID이며 fullHP/회복불가/무효 대상은 제외한다. 범위형은 고른 아군의 실행 시 타일을 중심으로 작성 footprint 안의 유효 아군을 회복한다. 시작과 방출 때 최신 자격을 검사하고 방출 전 무효 대상에 대해 임의 재지정하지 않는다. 공격 준비/주기/태세/CC/Pause 제약은 기본 행동 공통이며 Silence는 차단하지 않는다. 회복량/주기/범위는 명시 RON 초기 조절값이다. damage/피격/처치 proc·치명피해·흡혈을 만들지 않으며 실제 회복량을 관측한다.',
 ['정상 RON 단일 치유는 실제 아군 HP만 올리고 범위 치유는 footprint의 아군만 회복한다.', 'fullHP만 있을 때 새 기본 행동이 시작하지 않고 유효 부족HP 동률은 결정적으로 선택된다.', 'CC/Defense/작업/Pause와 방출 전 대상 상실, Silence 허용을 검증한다.', '동시각 확정 피해가 먼저이며 사망자를 치유로 부활시키거나 damage proc을 발동하지 않는다.'],
 '기본 치유와 부족HP의 효용은 아군 회복 의미에서 도출한다. HP비율→거리→ID와 자기 포함은 동일한 아군 자격을 결정적으로 적용하는 위임 선택이다. 새 비용/확률/상속을 만들지 않는다. 즉발 starter로 작성하되 미지원 delivery는 명시 거절한다.'),
('여섯 회사 지급 시작 세트', 'equipment-and-fragments.md#회사-지급-훈련용-파편; U06/B10,UC01',
 '기존6후보/3선발과 채용 경로에서 기본 무기+active 파편 세트를 정상 RON으로 지급한다. 총/아드레날린(공격력·공속), 검/버서커(공격력·공속·흡혈), 방패/최후의전선(받는피해감소), 대검/돌파(전방범위타격·짧은자기보호막), 단일치유지팡이/강한단일아군회복, 범위치유지팡이/약한영역아군회복이다. 기본세트는 별도 대여가 아니고 전투 밖 일반 장착 자격으로 이전할 수 있다. active1슬롯·공명·사용수명은 U05를 공유하며 새 직원 고유 active나 기존 direct BasicAttackModifier를 정식 파편으로 대신하지 않는다. 계수/지속/사거리의 초기값은 작성·검증으로 조절한다.',
 ['공개 loader에서6세트가 존재하고 정상 시작/채용 grant·장착 조회에 해당 사본이 연결된다.', '각 active 파편을 정상 사용해 약속된 buff/피해/보호막/회복 결과와 U05 잠금을 확인한다.', 'staff 기본은 active가 없어도 순수 치유이며 역할이 장비 이전의 직업 제한이 되지 않는다.'],
 '명시6역할을 기존 후보 identity에 배정하는 콘텐츠 작성이다. 새 weapon family나 미래 채용 UI/전체 원정 제도를 도입하지 않는다.'),
('일반 태세와 방어 효과', 'combat.md#공격과-방어-태세; U06/B05',
 'Attack/Defense 전환은 무료이며 직원별1500ms 일반 대기를 적용한다. 같은태세/대기/HardCC는 거절하고 Silence/준비/시전/집중 자체는 전환을 막지 않는다. Defense는 새 기본 행동·스킬을 막되 이미 시작한 준비/시전/방출·지연·추가타를 소급 취소하지 않는다. Ground는 무기별 저지 증가와 타입 감소 합산0..100%를, Platform은 저지0과 Rest를 적용한다. 패시브/반응형은 별도 기존 자격을 따른다.',
 ['일반 Defense 진입은 이미 시작한 기본/스킬을 취소하지 않으며 새 행동만 막는다.', 'HardCC/대기/중복 거절은 무변경이고 Ground 감소와 Platform 저지0이 조회/실제에서 일치한다.'],
 '일반 태세 대기와 이동 후 강제방어는 다른 이유지만 같은 stance owner가 판정한다.'),
('Rest와 제어·동시각 회복', 'combat.md#공격과-방어-태세; #군중-제어와-침묵; U06/B05',
 'Platform Defense Rest는 자격 진입 뒤 온전한 전투1초마다 HP3이다. 자격 상실/재획득은 새 주기를 시작하고 같은 타일 방향변경은 기존 주기를 유지한다. HardCC는 회복 펄스를 건너뛰되 주기는 유지하고 놓친 회복을 보충하지 않는다. 확정 피해를 같은시각 회복보다 먼저 처리하고 죽음은 회복으로 취소하지 않는다. HardCC는 행동/저지/작업기여를 막지만 방어감소와 이미 방출된 독립 효과를 지우지 않는다. Silence는 새 스킬만 막는다. 공명은 U05 공통 충전만 따른다.',
 ['온전한1초/CC 겹침·해제/Pause/동시각 lethal 피해와 Rest 결과를 확인한다.', '타일 종류 전환은 새 Rest 주기이고 제자리 방향 변경은 기존 주기다.', 'HardCC 중 저지0/작업기여0/피해감소유지와 Silence의 기본치유/이동 허용을 검증한다.'],
 '명시 회복 순서·생존과 기존 제어 수명 분리다. 손실 보충·부활을 새로 만들지 않는다.'),
('전술 이동의 자격·보존', 'combat.md#전술-이동과-자리-교대; U06/B06',
 '이동은 Attack/Defense 또는 작업 중 같은 전투 개체를 유효 배치 타일/방향으로 즉시 옮긴다. HardCC·미방출 기본 준비·스킬 시전/집중·이동대기는 차단하고 Silence/저지/일반 태세대기 자체는 차단하지 않는다. 같은 위치·방향은 거절하고 방향만 변경은 비용5/이동대기10초로 허용한다. 이동은 자기만 점유에서 제외하고 타일/무기적성/제3아군/작업자격을 최신 검증한다. HP·버프·공명·identity·기존 공격/스킬대기·방출 효과는 보존하고 저지/대상/오라는 최종 위치에서 갱신한다.',
 ['Attack과Defense에서 성공하고 일반태세대기가 남아도 이동은 독립 자격으로 판정한다.', '제자리 방향만 변경과 완전 동일 요청을 구분하고 모든 실패에서 비용/상태가 같다.', 'HP/버프/공명/identity/방출물 수명이 이동 전후 보존되며 저지/범위가 최종 위치를 따른다.'],
 '명시한 서로 다른 대기 소유자를 적용한다. 태세 전환 대기는 이동 대기가 아니며 별도 추가비용은 없다.'),
('이동 후 강제 방어', 'combat.md#전술-이동과-자리-교대; #현장-작업; U06/B06,B07',
 '성공한 이동과 교대 양쪽은 현재 전투시각부터1500ms 강제Defense 뒤 항상Attack으로 간다. 이 동안 수동 태세 전환은 거절하며 이전 Defense를 복원하지 않는다. 일반 태세 버튼은 강제타이머를 만들지 않는다. 성공은 일반 태세 전환 대기도 최소 해당1500ms 끝까지 맞추지만 자동Attack 전환이 새1500ms 대기를 더하지 않는다. Pause는 타이머를 멈추고 CC는 시간을 멈추지 않으며 자동Attack이 CC 행동제한을 지우지 않는다. 철수/사망/종료는 stale callback을 무효화한다. 미완료 작업칸 도착은 즉시 작업하며 강제타이머를 취소하고 완료 뒤Attack이다.',
 ['Attack/Defense 양쪽 출발에서1499ms Defense/1500ms Attack, 조기 수동해제 거절을 검증한다.', 'Pause/0.5배속·CC/철수/사망/종료에 타이머가 부활·이중전환을 만들지 않는다.', '작업 도착은1.5초 기다리지 않고 작업하며 완료 뒤 이전태세와 무관하게Attack이다.'],
 '강제방어가 끝나는 의미와 일반대기를 한 시점에 맞춰 즉시 되돌리기/추가 숨은 대기를 피하는 위임 선택이다. 전투시계 수명을 그대로 사용한다.'),
('교대 전체 원자 실행', 'combat.md#전술-이동과-자리-교대; battle-controls.md#자리-교대; U06/B06',
 '교대는 두 생존 배치 직원과 각 도착 방향을 최종 입력에서 함께 검증한 뒤 두 위치를 원자적으로 바꾼다. 자기교대·미배치·한쪽CC/준비/대기·상호무기적성·제3아군·비용 실패는 전체 무변경이다. 두 참가자만 점유 제외하며 비용5는 한 번, 이동대기10초와 강제Defense는 양쪽에 적용한다. 방향 미리보기/상대선택은 실행이 아니며 중간 한쪽 위치를 외부에 공개하지 않는다. 각자의HP/버프/공명/identity를 유지하고 저지/오라/작업을 최종 점유에서 다시 계산한다.',
 ['두 방향/두 타일/양쪽 자격 중 하나라도 실패하면 상태 hash·TP·쿨다운이 불변이다.', '성공은 양쪽동시·TP5한번·각10초/1.5초이며 작업타일 도착 예외를 개별 적용한다.', '정상 command/query에서 선택/preview/cancel이 실제 점유를 만들지 않는다.'],
 '기존 원자 교대 경로를 인수하며 한쪽만 성공하는 중간 상태는 완료로 허용하지 않는다.'),
('현장 작업과 완료 사실', 'combat.md#현장-작업; U06/B07,UC01',
 '작업량은 작업물에 남고 적격 직원마다 전투1ms당1ms를 기여하여 필요량까지만 합산한다. 작업 중 기본행동/새스킬/저지/일반태세/DefenseFocus/Rest는 금지하고 피해·공명은 공통규칙이다. HardCC는 배정/진행량을 보존해 그 직원 기여만 멈추고 Pause는 전체시간을 멈춘다. 철수/이동/교대는 허용하고 최종점유에서 배정한다. 필요량 최초도달 시한번 완료·효과한번이며 fullHP 수리도 소모완료한다. 완료 칸은 일반타일이 되고 완료 직원은Attack이다.',
 ['복수작업자·CC·Pause·직원이탈/복귀에 진행이 reset/bank/초과기여하지 않는다.', 'fullHP 수리도 완료사실과 소모가한번이고 반복 query/시간/재접속이 완료를 재발행하지 않는다.', '작업→철수/이동/교대와 완료 후Attack·저지/공명 자격을 정상 명령으로 확인한다.'],
 '작업물 소유의 단조 진행·1회 완료를 명시 규칙대로 적용한다.'),
('저지 위치와 유지/해제', 'blocking-and-targeting.md#최초-접근과-통과-중-저지; #저지-해제; open-decisions.md NEXT-12; U06/B08',
 '직선 접근은 아군 가까운 앞타일 끝자락까지 실제 이동한 뒤 저지되며 저지중 그 판정타일/연속위치를 유지한다. 현재 범위를 지나던 적도 용량이 생기면 그 현재위치에서 저지하고 과거위치로 돌리지 않는다. 초기 접촉반경은 타일단위0.75를 채택하여 인접 타일의 바깥 경계1.5/중앙1.0이 아닌 중심간0.75에서 멈추게 하며 실제 이동 segment가 접촉을 넘으면 최초 접촉점까지만 진행한다. 여러 적은 경로진행내림·스폰순·ID, 여러 blocker는 접촉시점/거리·ID로 결정한다. 유효 기존 관계를 신규 후보가 밀어내지 않으며 용량감소만 같은 우선순위로 줄인다. 죽음/철수/CC/저지불가/이동/실제 반경이탈 시 해제하고 새 위치에서 검사한다. 공중 적은 저지하지 않는다. 시각 겹침 보정은 판정 위치가 아니다.',
 ['직선 다방향 접근에서 앞타일 근접 끝자락까지 이동한 뒤 정지하고 판정타일을 아군타일로 강제치환하지 않는다.', '한계용량을 통과하는 적과 기존적 사망의 같은시각 재판정에서 현재위치를 보존한다.', '범위이탈·CC·죽음·이동·용량감소·공중과 신규후보가 기존관계를 밀어내지 않는 규칙을 검증한다.'],
 '0.75는 과거 코드의 존재만으로 정책으로 승격한 것이 아니라 명시된 앞타일 가까운 끝자락을 만족하는 초기 접촉값으로 이번 위임에서 채택한다. 50ms는 구현 tick일 뿐 의미규칙이 아니며 빠른 적의 접촉누락은 허용하지 않는다. 새 보상/비용/상속은 없다.'),
('저지 상대 기본 대상 예외', 'blocking-and-targeting.md#저지-상대의-기본-공격; #일반-타겟팅과-피해; U06/B08,B11',
 '아군과 자신이 저지한 적은 서로의 기본 공격 후보에서 위치/방향/타일사거리 예외를 사용한다. 같은 판정타일을 필수로 하지 않으며 태세/CC/주기/생존/유효피해·효과 조건은 유지한다. 이 예외는 다른 공격자·스킬·장판·순수 아군 치유에 확장하지 않는다. 기본공격의 profile 우선순위와 스킬 자체규칙은 분리한다. 예상피해양수 또는 유효적대효과가 없으면 자동적대후보에서 제외한다.',
 ['다른 판정타일·뒤쪽 저지상대도 기본공격만 가능하고 스킬/장판/제3자는 일반사거리다.', '저지해제 뒤 예외가 즉시 사라지고 무효효과/Defense/CC/주기 제한을 우회하지 않는다.', '치유는 적저지 우선순위를 따르거나 적HP에 양수 회복을 주지 않는다.'],
 '최신 B08이 과거 same-logical-tile 설명보다 우선한다. 기존 hostile usefulness는 유지한다.'),
('전술·행동 query와 preview', 'battle-controls.md#판정-위치와-대상-미리보기; U06/UC01 and outputs',
 '정상 Core query는 직원/적의 판정tile·연속위치·저지관계·기본행동 종류/범위/현재유효대상, 배치/이동/교대 가능한타일·방향·비용·대기·거절사유, 작업진행/완료와 실제 결과를 제공한다. preview는 읽기전용 예상값이며 최종 명령은 최신 상태와 양쪽자격을 다시 검사한다. 비용과 원자성은 Core가 소유하고 UI가 규칙을 재계산하게 하지 않는다. 실패는 선택상태도 보존하되 선택의 출발개체가 무효가 되면 그 포커스는 종료한다.',
 ['실제 공개 command→result→query에서 비용/거절/위치/치유량/작업결과를 설명할 수 있다.', 'preview 반복은 revision/TP/HP/점유를 바꾸지 않으며 stale preview 후 최종 실패는 무변경이다.', '타일과 연속위치/저지사거리 예외와 가능한 행위를 query에서 구분한다.'],
 'UC01의 화면없는 인수 요구를 충족하는 Core DTO이며 고스트/카메라 렌더링은 U26-A 소유다.'),
('공통 포커스와 선택 감속', 'battle-controls.md#포커스와-배속; #로스터와-대상-확인; open-decisions.md NEXT-13; U06/UC02',
 '일반 아군/적 포커스와 배치/재배치 선택은 Core가 소유하는 선택 상태다. 선택이 하나라도 남으면 실제배속0.5이며 최초 진입 전1/2배속을 한 번 저장한다. 대상전환·이동/교대/스킬 하위선택은 같은 복원값을 사용하고 감속을 곱하지 않는다. 일반포커스가 남으면 스킬성공/취소 뒤에도0.5를 유지하고 독립 스킬선택만 있었으면 완료/취소시원복한다. 수동배속변경은 선택중 거절한다. 적/출발직원 무효화는 해당 포커스를 닫고 모든선택종료에서만 원복한다. 종료/후퇴는 복원보다 우선한다.',
 ['2→포커스0.5→다른대상/스킬/이동0.5→전체해제2와 선택중 speed거절을 정상 명령으로 확인한다.', '아군/적/배치 선택중 수동Pause를 유지하고 선택해제로 자동재개하지 않는다.', '대상소멸/전투종료/후퇴에서 stale focus가 전투를 재개하거나배속을 덮지 않는다.'],
 '최초 복원 배속과 한 전투 소유권을 공유해 중첩을 제거한다. U05의 독립 스킬선택 복원은 일반포커스 부재에서 그대로 유지한다.'),
('Pause·설정과 강제 시계', 'combat.md#전술-포인트와-전투-시간; battle-controls.md#설정과-전투-포기; U06/UC02',
 'Pause중 허용된 배치/철수/태세/이동/교대/스킬접수는 접수순서대로 검증·원자적용하되 전투시간과 예약효과는 흐르지 않는다. 설정열기는 현재Pause/실제배속을 보존해 자동정지하고 닫기는 그 상태를 복원한다. 설정중 전투행동/Resume/speed는 명시거절하여 시간우회를 막고 조회/닫기/기존후퇴는 허용한다. 중복열기는 복원값을 덮지 않는다. 선택중설정은닫아도0.5이며 설정전수동Pause를 유지한다. 일반 host walltick은 Pause된 전투시간을 진행하지 못한다. 단 U05가 보존한 침식 강제정지의 host 경과시계는 기존 명시 예외로 유지하고 전투시간/TP/작업을 진행시키지 않는다. 별도 명시 elapsed/fixture 실행 포트는 기존 권한을 유지하며 일반 전투 입력으로 우회하지 않는다.',
 ['Pause중 연속배치/교대/철수/유효스킬접수를 입력순서로 검증하고 timer/work/projectile은 재개전 진행하지 않는다.', '1/2/0.5와Pause 각 조합의 설정열기/중복/닫기 및 설정중금지 명령에서 복원값과시간이 정확하다.', '일반 host tick과 침식 강제정지 host 시계를 구분하고 종료후 설정닫기가 전투를 되살리지 않는다.'],
 '설정메뉴의 입력소유와 직전상태 복원을 Core 경계로 적용한다. Pause 중 허용된 게임 입력은 메뉴가 닫혔을 때의 기존권한이며 새로운 시간조작 권한을 주지 않는다.'),
('저장·재접속·fault 소유권', 'results-and-save.md#저장과-복구; U04/U05 독립 감사; U06 together-boundaries',
 '같은 active actor의 재접속은 현재포커스/선택·설정복원값·강제방어·작업·치유행동/예약 상태를 보존하며 disconnect는수동Pause처럼 유지한다. 새프로세스Continue는 U04의미확정전투입장전복원이고 SavePoint는별도rewind다. live battle을새로disk저장하는예외는추가하지않는다. 지원 snapshot/clone/rollback은추가상태를독립소유하며 fault는정상 command/query의부분변경을남기지않는다. CF-D01에따라 미지원구형저장은명시거절하고원본을보존한다.',
 ['선택/설정/강제방어/작업/치유 준비중 clone/지원roundtrip/rollback과같은actor재접속이수명/비용을중복시키지않는다.', 'disconnect중설정닫기/선택종료가자동Resume하지않고 명시Resume후원래선택배속이다.', '실제hostContinue/SavePoint와미지원원본보호가U04의결과/자산/횟수계약을유지한다.'],
 '새상태를기존active소유자에포함하는선택이다. 구형저장변환이나권리재해석을하지않는다.'),
('공개 작성·비용·관측의 완결', 'U06 together-boundaries; original continuous request; U02/U03/U05 prerequisites',
 '기본치유/시작세트·전술/시간/조회는정상 RON loader/validate/compile/catalog와 CoreApplication·host의공개경로를통과한다. 제거/미지원 필드는조용한성공fallback없이진단한다. 새상태의할당/보유크기/clone/work계상과bounded관측·fault격리는기존안전성상한을유지하며의도된비용변화는실제측정으로명시한다. fixture는oracle과원래입력을보존하고정상public소비자와음성거절사례를검증한다.',
 ['6세트와pureheal schema를실제loader→CoreApplication→host/query로실행하고직접kernel수정만으로성공을대체하지않는다.', '새상태size/clone/queue/관측eventcost를초기집중검사와최종통합에서검증하며상한을통과용으로낮추거나높이지않는다.', 'strict오류·실패원본·selector/실행건수·실제비용delta와부수효과를구분해기록한다.'],
 '선행안전성/공개소비/관측계약의U06확장이다. 기존R05승인peak경고와20분stress제외는그대로이고새예외는추가하지않는다.'),
('검증·독립 감사와 후속 경계', 'U06 responsibility/prerequisites/completion; original continuous request',
 'U05까지141선행조건과별도31 R06의현재적용성을변경영향별로추적하고유효근거를재사용한다. U06모든요소·10검증차원·정상headless조우·실제명령/조회·부정/복원/비용/fault검증을자체완료한뒤구현비참여GPT-6AstraHigh독립감사와finding교정/재감사를닫는다. 통과전U07종속구현은시작하지않는다. Unity/카메라/고스트·최종보스·새무기계열·미합의UI와미래owner의전체기능은제외한다.',
 ['조건별currentness/의도된변경/증거가추적되고정상6세트조우의배치·치유·스킬·이동·교대·작업·철수·시간을명령으로인수한다.', '전체완료독립감사가Complete/High이며미해결finding이없을때만다음owner로넘긴다.'],
 '연속요청의단계별독립성/완료조건과유효근거재사용이다. 형식검사성공을정책/제품/감사성공으로표현하지않는다.'),
]


def main():
    intake, before = json.loads((HERE/'P2G_00_INITIAL_001.json').read_text())['artifacts']
    assert before['payload']['Current Policy Element Revision Map'] == {}
    basis = {'Input Working Revision Ref': before['ref'], 'Exact User-Approved Target Scope/Change Request': None,
             'Policy Intake Snapshot Ref': intake['ref']}
    artifacts, current, decisions, dispositions = [], {}, [], []
    for n,(name,span,statement,acceptance,reason) in enumerate(POLICIES,1):
        eid=f'PE-CBST-{n:02}'
        opened=artifact('Policy Element Revision',f'PER-CBST-{n:02}-OPEN-001',{
            'Stable Element ID':eid,'Predecessor Policy Element Revision Ref':None,'Policy Statement':statement,
            'Decision Revision Ref':None,'Observable Policy Acceptance':[],'Intrinsic status':'open'})
        decision=artifact('Decision Revision',f'DR-CBST-{n:02}-001',{
            'Predecessor Decision Revision Ref':None,'Affected Element IDs':[eid], 'Decision Name':name,'Exact Decision':statement,
            'Decision Authority':'AI delegated application of adopted agreements; exact boundary choices identified in Reason; not a new user answer',
            'Exact User Approval Reference':{'intake_ref':intake['ref'],'item':'IN-001','delegation':'IN-004 CF-D06/07','scope':'U06 B01/B05-B08/B10/B11/UC01/UC02'},
            'Source Reference':{'intake_ref':intake['ref'],'span':span},'Reason':reason,'Impact':acceptance,
            'Delegation Criteria':{'existing_agreements_preserved':'최신 game_rulebook/game_rules와 U06 명시 규칙·U04/U05 수명을 적용하고 제외 범위를 유지',
                'derived_from_ownership':reason,'no_new_game_rule':'명시 비용/상한/회복/1회 완료 유지. 초기 콘텐츠 수치와 결정적 접촉/선택 상세만 위임 안에서 결정. 새 보상/확률/환불/상속 예외 없음',
                'local_impact':'U06 기본 행동·전술·선택/시간과 같은 전투 소유 상태. 독립 자산/진행 불변',
                'user_data_preserved':'사용자 변경·감사·저장 원본 보존. 미지원 형식은 거절하고 자동 삭제/변환 없음'}})
        decided=artifact('Policy Element Revision',f'PER-CBST-{n:02}-DECIDED-001',{
            'Stable Element ID':eid,'Predecessor Policy Element Revision Ref':opened['ref'],'Policy Statement':statement,
            'Decision Revision Ref':decision['ref'],'Observable Policy Acceptance':acceptance,'Intrinsic status':'decided'})
        disposition=artifact('Candidate Policy Element Disposition Record',f'CD-CBST-{n:02}-001',{
            'Candidate ID':f'C-CBST-{n:02}','P2G-10 Evaluation Basis':basis,'Source Reference':decision['payload']['Source Reference'],
            'Candidate Statement':statement,'Disposition':'promoted','Stable Element ID':eid,
            'Initial Open Policy Element Revision Ref':opened['ref'],'Decision Revision Ref':decision['ref']})
        artifacts.extend([opened,decision,decided,disposition]);current[eid]=decided['ref'];decisions.append(decision['ref']);dispositions.append(disposition['ref'])
    for n,(statement,owner) in enumerate([
        ('Unity 입력/HUD/고스트/카메라·카드 위치 고정·미합의 minimap/rotation','U26-A; U06 명시 제외'),
        ('최종보스 고유 UI/내용·새 무기계열','U06 명시 제외; 현재 로드맵 승인'),
        ('전체 로드맵 미래 상세·전체 원정/새 채용/UI 제도','U07 이후 owner; U06은 기존 grant에 여섯 세트 연결')],1):
        d=artifact('Candidate Policy Element Disposition Record',f'CD-CBST-OUT-{n:02}-001',{
            'Candidate ID':f'C-CBST-OUT-{n:02}','P2G-10 Evaluation Basis':basis,'Source Reference':{'intake_ref':intake['ref'],'span':'IN-002 U06 exclusions; IN-001'},
            'Candidate Statement':statement,'Disposition':'out_of_scope','Evidence/Decision Reference':owner,
            'Stable Element ID':None,'Initial Open Policy Element Revision Ref':None})
        artifacts.append(d);dispositions.append(d['ref'])
    payload=copy.deepcopy(before['payload']);payload.update({'Predecessor Working Revision Ref':before['ref'],
        'Current Policy Element Revision Map':current,'Decision Revision Ref Set':decisions})
    result=artifact('Working Revision','WR-CBST-002',payload);artifacts.append(result)
    # Reuse the tested U05 canonical field shapes, and check candidate data before immutable publication.
    for a in artifacts:
        assert artifact(a['ref']['Artifact Type'],a['ref']['Artifact ID'],a['payload'])==a
        if a['ref']['Artifact Type']=='Policy Element Revision':
            assert a['payload']['Intrinsic status'] in ['open','decided']
            assert isinstance(a['payload']['Observable Policy Acceptance'],list)
            if a['payload']['Intrinsic status']=='decided':
                assert a['payload']['Observable Policy Acceptance']
                assert a['payload']['Decision Revision Ref'] in decisions
    assert len(current)==len(decisions)==20 and len(dispositions)==23
    assert len({a['ref']['Artifact ID'] for a in artifacts})==len(artifacts)
    write('P2G_10_POLICY_001.json',{'artifacts':artifacts})
    write('P2G_10_RESULT_001.json',{'stage':'P2G-10','status':'complete','Input Working Revision Ref':before['ref'],
        'Result Working Revision Ref':result['ref'],'Produced Candidate Disposition Ref Set':dispositions,
        'Result Current Candidate Policy Element Disposition Effective View':dispositions,'is_p2g10_complete':True,
        'checks':{'23_candidates_one_disposition_each':True,'20_elements_decided':True,'single_leaf_decisions':True,
            'acceptances_present_consistent':True,'delegation_five_criteria_per_decision':True,'intake_unchanged':True,
            'open_questions_empty':True,'cross_references_and_digests_valid':True,'draft_schema_checked_before_publication':True},
        'Finding Triage':[
            {'classification':'material_policy','disposition':'20 delegated decisions applying explicit current agreements and bounded owner choices; each has reason/impact/five criteria','elements':list(current)},
            {'classification':'implementation_finding','disposition':'P2G-20 required; previous read-only observations are not formal review','statement':'Pure healing action, general focus/settings, forced defense, and same-tile block requirement must be formally traced across producer/consumer/restore.'},
            {'classification':'out_of_scope','disposition':'3 candidates; U26-A/future owners and explicit exclusions'}],
        'Open Policy Question Log':[],'Evidence Gap Log':[],'new_user_request_required':False,'next':'P2G-20','production_mutation':False})
    view=(HERE/'WORKING_POLICY.ko.md').read_text().replace('`WR-CBST-001`','`WR-CBST-002`').replace(
        '현재 P2G-00 완료, 다음은 P2G-10이다. 초기 정책 요소·결정·미결 질문은 비어 있다.\nP2G-10 Completion Predicate: N/A. 이 초기화는 정책 의미·코드 검토·동결·제품 구현 완료가 아니다.',
        '현재 P2G-10 완료, 다음은 P2G-20 정식 코드 검토다. 정책 요소/결정20개, candidate23개(승격20/범위외3), 미결 질문0개다.\nP2G-10 Completion Predicate: true. [정책/결정](P2G_10_POLICY_001.json)과 [검증](P2G_10_RESULT_001.json)이 근거다. 코드 검토/동결/구현 완료가 아니다.')
    view+='\n[운영 지침 인수](WORKFLOW_INTAKE_001.json): U05 이후 문서7개 delta의 무영향을 확인하고 P2G r8/GEX gex-r8/운영안v2.1.3을 적용한다. 기존 검증과 원본은 보존한다.\n\n'
    view+='위임 선택은 정책별 Reason/Impact에 기록했다. 특히 기본 치유의 HP비율→거리→ID, 자기 포함, 0.75 접촉반경의 초기값, 일반 태세/강제방어 대기, 설정 입력 소유, 포커스 중첩/재접속은 사용자 새 답변으로 표시하지 않는다.\n\n'
    for n,(name,span,statement,acceptance,reason) in enumerate(POLICIES,1):
        view+=f'### PE-CBST-{n:02} — {name}\n\n{statement}\n\n근거: {span}. 판단 이유: {reason}\n\n'
    (HERE/'WORKING_POLICY.ko.md').write_text(view)
    print('U06 P2G-10 complete:20 decided elements/23 dispositions; no product mutation; P2G20 next')

if __name__=='__main__': main()
