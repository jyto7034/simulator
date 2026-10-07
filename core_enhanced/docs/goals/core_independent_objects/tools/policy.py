#!/usr/bin/env python3
"""U07 P2G-10: adopted meaning and bounded delegated decisions."""
import copy
import json
import tarfile
from initialize import HERE, ROOT, artifact, write, sha

# Name, exact policy locator, decision, observable acceptance, delegated rationale.
POLICIES = [
('생성 능력과 진행 권한의 분리', 'ROADMAP §5 U07/B13,C01; combat.md#웨이브와-보스; R13 §§1,3–5; K09',
 '정상 작성 입력으로 아군/적 actor, 정지 trap, 유한 field와 HP를 가진 wall을 생성한다. 종류별 이동·공격·피격·점유·HP 능력을 구분하며 모든 object를 직원/일반 전투 Unit으로 강제하지 않는다. 생성물의 진영은 실제 생성 source의 진영을 따른다. 생성이 actor slot·required wave·보상·파편·도감 권한을 새로 만들지 않으며 생성물의 생존만으로 승리를 막지 않는다. 기존 보스 summon은 기존 의미와 정확한 provenance를 유지한다.',
 ['같은 공개 생성 능력을 아군과 적 source에서 실행하여 올바른 진영/대상/종류별 능력을 확인한다.', '생성·진화·파괴·TTL 뒤에도 wave/보상/도감 원장과 required victory가 임의로 늘지 않으며 기존 boss summon 회귀가 유지된다.'],
 'U07의 명시한 actor/object 분리와 K09 권한 경계를 적용한다. 기존 boss caller의 Side::Opponent는 범용 생성의 정책이 아니다.'),
('생성 요청·identity·원자성', 'R13 §§1,3,6; T15/T23; U07 completion',
 '생성 요청은 occurrence에 귀속된 안정된 identity를 갖는다. 동일 요청/동일 입력의 재처리는 같은 settlement를 돌려주며 비용·ID·생성을 다시 수행하지 않는다. 같은 요청 identity에 다른 입력은 명시 거절한다. 전체 배치와 정의·타일·점유·owner·작성 상한·HP/분절 비용을 검증한 후 비용·객체·ID·source·queue·실제 결과를 함께 commit한다. 도메인 거절은 성공0개와 구별하며 budget/control fault는 cached gameplay 거절로 바꾸지 않는다. 기존 성공 request 원장과 공통 SpawnKernel의 의미를 재사용한다.',
 ['여러 생성 중 후반 타일/정의/예산 실패에 비용·HP·pool·객체·ID·fact가 부분 반영되지 않는다.', '정상 재시도/중복은 한 번만 생성하며 payload 충돌·fault 재시도·clone 분기에서 identity와 settlement가 일관된다.'],
 '기존 정규화·idempotence·candidate 경계를 적용한다. 새 자동 환불이나 오류를 정상 성공으로 바꾸는 경로를 만들지 않는다.'),
('타일 자격·점유와 작성 상한', '래티씨아 파편 기본/개화형·미결정 사항; 고기벽 배치 제한; U07 placement',
 '생성 타일은 현재 battlefield와 authored 배치 범위에서 재검증한다. 함정과 벽은 유효한 지상 path tile을 사용하며 spawn/goal/portal/고정 object/현재 점유 타일을 거절한다. 일반 actor/field는 종류에 맞는 명시적 occupancy를 사용하고 빈 타일과 중첩 허용을 조용히 바꾸지 않는다. 래티씨아의 같은 source/copy가 만든 작은 친구 한 개만 같은 타일 진화 예외이며 다른 직원·적·다른 생성물은 예외가 아니다. authored 생존/총생성 상한은 해당 source와 계열에 귀속하고 진화는 기존 한 개체의 같은 생존 슬롯이다. 보편적인 새 gameplay 상한을 만들지 않고 host 안전 한도와 구별한다.',
 ['경계/범위 밖·무효 타일·복수 점유·다른 source 작은 친구·큰 친구 재중첩은 입력 거절과 비용 불변이다.', '작성 상한의 exact/one-over, source A/B 독립과 소멸 뒤 생존 슬롯 회수, 진화의1슬롯 유지가 정상 명령/query에서 일치한다.'],
 '타일형 함정과 path 방해물의 명시 계약에서 eligibility를 도출한다. 콘텐츠별 초기 범위/상한은 승인된 미정 수치의 조절값으로 기록하며 사용자 확정값이나 보편 규칙으로 승격하지 않는다.'),
('독립 actor의 실제 행동', 'R13 actor adapter/정지매개체; U07 common creation; existing summon contract',
 'actor는 작성된 전투 profile과 명시된 이동 방식에 따라 생성 진영의 적을 대상으로 실제 행동한다. 경로 이동과 제자리 행동은 서로 다른 capability이며 정지를 이동속도0 검증 우회로 표현하지 않는다. 직원 roster/배치8/TP/수동 직원 명령 권한을 자동 상속하지 않고 생성 동작의 공명 사용 수명과 분리한다. actor의 공격·스킬은 실제 actor/source identity를 가지며 부모의 만료된 action 또는 자원 lease를 임의로 되살리지 않는다.',
 ['아군/적 actor의 대상 선택·공격·정지/이동과 실제 HP 결과를 외부 작성 입력으로 실행한다.', '부모 철수 뒤 독립 actor의 정상 수명과 자신의 source attribution이 유지되며 직원 배치 상한/공명/명령을 대리하지 않는다.'],
 '독립 actor와 직원·보스 adapter의 소유권을 구별한다. 향후 관계망·유물 인과 root를 이 단계에서 임의 발명하지 않는다.'),
('정지 field와 별도 object 능력', 'R13 actor/trap/field object; U07 장판/정지; R10/R11 released lifetime',
 'field는 작성된 위치·범위·유한 기간·양의 주기에 따라 실제 효과를 실행한다. 이동·HP·저지·직원 명령을 명시하지 않은 field에 추가하지 않는다. 기존 고정 TileArea로 의미를 완전히 표현할 수 있으면 기존 executor/source/TTL을 재사용한다. overlap과 target filter는 명시 입력을 따르고 임의 합치기·조용한 tick 생략은 없다. 부모 동작이 끝난 뒤 남은 독립 field는 자기 battle-time 수명과 source를 유지한다.',
 ['정상 RON의 독립 field가 실제 주기/범위/대상에 효과를 내고 Pause·부모 철수·TTL 경계에서 정해진 횟수만 실행한다.', 'HP/점유/이동이 없는 field는 해당 능력을 가진 것으로 query되거나 타격·치유·배치 blocker가 되지 않는다.'],
 '이미 있는 독립 영역의 의미를 재사용하며 같은 기능을 두 런타임에서 이중 실행하지 않는다. 새 객체가 필요한 부분만 별도 능력으로 분리한다.'),
('래티씨아 함정의 발동·공격·소모', 'abnormality_skill_fragments_design_notes.md#래티씨아 기본형/개화형; ROADMAP USER-07/U07',
 '래티씨아 기본형은 빈 eligible 타일에 작은 친구를 배치한다. 작은 친구는 이동/지속 기본 공격/저지가 없는 함정이며 유효한 적이 trigger 범위에 들어오면 공격 가능한 대상을 향해 한 번 공격한 뒤 사라진다. 큰 친구는 더 넓은 trigger/피해 범위와 높은 authored 피해를 가진다. 적이 없거나 현재 공격할 수 없는 친구는 소비하지 않는다. 현재 trigger와 공격 대상 자격을 확인한 성공한 자기 공격만 친구를 소비하며, 같은 시각 여러 후보는 결정적 순서로 한 번 처리한다. 피해는 생성/진화 시 실제 장착 무기 Attack을 포함한 source 값을 snapshot해 계산하고 기본공격 전용 proc을 상속하지 않는다. 기존 래티씨아 조우의 감염/숙주/거리변이는 이 파편에 복제하지 않는다.',
 ['정상 기본·개화 파편의 실제 명령으로 작은/큰 친구가 생성되고 작은 단일·큰 광역 공격과 소멸 결과가 관측된다.', '대상 없음/공중 등 부적격/대상 소실·동시 trigger에서 공격 불가능한 친구가 남고 성공한 친구만1회 공격·소모된다.', '생성 후 caster 장비/능력치·철수 변화가 이미 소유한 피해 snapshot을 고치지 않고 피해 출처와 파편 copy가 보존된다.'],
 '공격 가능한 친구만 소모하는 최신 합의를 우선한다. 무기값 고정 시점은 독립 방출물의 기존 snapshot 소유권에서 도출한 위임 선택이며 연결 거미줄은 U18이다.'),
('같은 타일 진화와 상태 보존', '래티씨아 같은 타일 중첩/개화형; R13 §5; U07 evolution carry',
 '기본형의 두 번째 완료된 같은 타일 시전은 자기 작은 친구를 큰 친구 하나로 바꾼다. 공명 만충과 시전은 각각 정상 소비하며 별도 두 개체를 겹쳐 남기지 않는다. 큰 친구 위 세 번째와 다른 source/copy의 진화는 거절한다. 개화형은 빈 타일에 큰 친구를 한 번에 생성하며 전투 중 파편 교체/개화 migration을 추가하지 않는다. 진화는 논리 identity·owner/source 계열·생존 슬롯을 유지하고 새 형태의 값과 새 생성 시전 source를 결속한다. 이전 형태의 trigger/expiry/공격 대기·target/link epoch는 무효화한다. HP를 가진 공통 형태 전환은 현재 HP를 유지한 뒤 새 최대HP로 clamp하며 공짜 회복하지 않는다. HP 유무가 다른 형태 전환은 명시 지원 없이 거절한다. 새 형태의 authored TTL은 진화 시점부터 시작한다.',
 ['작은→큰에 객체1개·같은 타일·생존1슬롯, 정확한 두 시전 비용과 개화 한 시전의 차이를 확인한다.', '진화 직후 이전 timer/trigger가 새 형태를 삭제·공격시키지 않으며 clone의 형태 전환은 원본과 독립이다.', 'HP형 진화의 증가/감소에서 no-free-heal/clamp가 적용되고 변환 실패는 이전 상태/비용을 보존한다.'],
 '논리적 형태 변환과 최신 생성 시전의 독립 payload를 결합하는 위임 선택이다. 최대HP 정책을 그대로 적용하고 형태의 유한 수명 재시작은 두 번째 정상 생성 동작에 귀속한다.'),
('source·사본·철수 후 독립 수명', 'R04/R05/R07 source; combat.md#배치와-철수; R13 삭제/철수/source binding',
 '각 생성물은 생성 actor·실제 파편/장비 copy·형태·authored site와 필요한 값 snapshot을 독립 소유한다. source가 철수·사망·삭제돼도 이미 방출된 독립 생성물은 authored Persist 수명을 유지한다. 명시 Despawn-on-source-removal 정의는 일회 제거한다. 공식 친구/벽은 독립 Persist를 사용한다. 살아 있는 source로 가장하거나 만료된 action/delivery/resource 권한을 복제하지 않는다. 관측 provenance와 후속 실행에 필요한 실제 private lease를 구분하고 cleanup은 자신이 소유한 권한만 회수한다.',
 ['source 삭제/철수/사망·다른 copy 재배치 뒤 Persist/Despawn 두 계약과 실제 결과 provenance를 확인한다.', '독립 생성물이 살아 있어도 생성 skill의 사용 잠금이 끝나고 양수 공명 획득이 재개되며 오래된 actor/copy를 새 source로 잘못 연결하지 않는다.'],
 'U01/U05의 미방출 취소·독립 방출물 보존을 새 객체에 적용한다. U10 인과 연쇄를 선구현하거나 새 root 권한을 만들지 않는다.'),
('TTL·파괴·종료와 자원 회수', 'U07 TTL/memory회수; R13 source lifetime; U03 fault; U04 restore',
 '유한 생성물은 battle time의 authored TTL을 사용하고 Pause에서 만료 시계가 멈춘다. TTL/명시 despawn/source 제거/공격 소모/HP0 중 처음 성립한 원인만 객체를 제거하며 후속 stale callback은 아무 결과도 추가하지 않는다. Despawn/소모/TTL을 죽음·처치·시체로 가장하지 않는다. 실제 HP0 파괴는 해당 종류의 실제 결과를 한 번 기록하되 보상/시체 흡수 권한을 만들지 않는다. 전투 종료/후퇴는 객체·timer·실행 lease와 retained state를 정리하며 무한히 누적되는 완료 객체 목록을 새로 만들지 않는다.',
 ['동시 TTL/피격/소모와 source 제거의 중복 cleanup에서 실제 결과1회와 stale 무해를 확인한다.', '종료·반복 생성/소멸 뒤 활성 객체·queue/source 보유와 allocator 비용이 수명에 맞게 회수되고 원래 안전 한도를 통과한다.'],
 '기존 death와 despawn 구별 및 first-cause 수명 규칙을 적용한다. 완료 관측은 bounded witness/정상 로그의 기존 소유자를 사용한다.'),
('분절HP의 단일 truth·피해·구간 분리', '고기방패 HP와 스택/피해 전달; 고기벽 구간 소비; U07 분절HP; R12 pool',
 '분절HP는 양의 구간HP H와 구간 수 한도의 단일 HP pool이다. 스택은 현재HP P에서 ceil(P/H)로 계산하고 별도로 저장하지 않는다. 소유 unit의 분절 보호 pool과 독립 벽 HP를 직원 자신의 HP와 구별한다. pool은 source와 명시한 pool identity에 귀속하며 여러 pool을 암묵적으로 합치거나 장착 순서로 고르지 않는다. 감쇄된 eligible 피해를 한 번 적용하고 여러 구간 파괴/잔여 초과 HP피해를 처리한다. 구간별 저지 보너스가 정의된 pool은 실제 잔량에서 계산하고 초과 blocker만 결정적으로 해제한다. 흡수량은 직원 actual HP-loss·일반 피격/흡혈/반사 source가 아니다. 분리할 최상단 HP는 P-(ceil(P/H)-1)*H이며 비용과 벽 생성이 함께 commit한다. 일반 Heal/흡혈은 pool을 수복하지 않는다. U07은 공통 공개 pool 생산·피해·구간 분리 consumer를 실제 실행하고, 녹아내리는 사랑의 감염·시체 소유권 producer는 U18/U21에 둔다.',
 ['H100/P150에서 스택2·분리50·잔량100·벽HP50이며 P100/P1/0와 한 번의 여러 구간 피해를 확인한다.', 'pool 흡수/초과 피해·저지 감소가 중복 감쇄·가짜 HP-loss/proc 없이 실제 query와 일치한다.', '다른 source/pool·일반 회복은 잔량을 임의 변경하지 않고 실패 생성은 pool/저지/객체 전체를 복원한다.'],
 '합의된 segmented truth와 detach 공식을 공통 입력 계약으로 채택한다. 전체 감염 무기나 시체 소유권을 새로 구현했다고 주장하지 않으며 U07 직접 작성 pool fixture와 후행 공식 생산자를 구분한다.'),
('HP fallback과 비용+벽 생성', '고기벽 다른 Shield HP fallback; R12 DR-CARX-E05-001 P11; U07 HP비용 생성',
 '고기벽형 생성은 사용할 자기 분절 pool의 실제 구간이 있으면 그것을 먼저 분리하고, 없으면 caster HP를 authored 양의 비용으로 소비한다. 비율 비용은 현재 최대HP와 authored 비율을 정수 계산하며 실제 소비 HP와 authored 변환 계수로 벽 HP를 정한다. 비용 후 HP가0이 되는 fallback 입력은 접수 전에 거절하고 공명을 소비하지 않는다. HP 비용은 Damage/Heal이 아니며 보호막·방어/저항·피해감소·parry/HP1을 우회한다. 독립 일반 HealthCost의 기존 명시 lethal opt-in은 유지하지만 고기벽 fallback에는 적용하지 않는다. 지급 불가/점유/정의/상한/예산 실패는 비용·pool·ID·생성을 부분 반영하지 않는다.',
 ['일반 Shield와 빈 pool에서 실제 비용/벽HP, shield/parry/HP1 비영향 및 lethal 경계 거절을 정상 명령으로 확인한다.', 'HP 비용 후 의도된 생성 failure/fault와 여러 placement 실패에서 원래 HP/pool/facts/IDs를 되돌리고 정상 재시도는 한 번 소비한다.'],
 'R12 HP 비용 계약과 고기벽의 명시 nonlethal fallback을 결합한다. 정확한 초기 비율/변환계수는 승인된 콘텐츠 조절값으로 기록하고 무료 생성이나 환불 예외를 만들지 않는다.'),
('벽의 경로 접촉·피격·공식 얼음벽 사례', '고기벽 battlefield object/배치 제한; U07 얼음벽; 눈의 여왕 조우 세 행동',
 '벽은 이동/공격하지 않는 HP object다. 작성된 path를 다시 짜거나 우회시키지 않는다. 지상 적이 원래 경로에서 벽에 닿으면 멈추고 적대 벽을 공격해 제거한 뒤 원래 경로를 계속한다. 공중 actor는 무시한다. 벽은 caster의 방어/저항을 생성 시 snapshot할 수 있고 해당 벽 피해는 caster HP/피격/공명/시체 사실이 아니다. 일반 회복·흡혈·시체 흡수와 파괴 환불은 없다. U07의 얼음벽은 공통 작성 포트의 실제 공식 소비 정의/시험으로 연결하며 최신 눈의 여왕 조우에 네 번째 행동·소환 기믹을 추가하지 않는다. 같은 HP/TTL 벽 계약을 고기벽 fallback에도 사용한다.',
 ['정상 작성 얼음벽/고기벽에 지상 적이 실제 이동 segment에서 정지·공격·파괴·경로 재개하고 공중 적은 통과한다.', '벽을 치유/직원 명령/시체/보상 대상으로 오인하지 않으며 snapshot 방어와 피격 결과를 query할 수 있다.', 'source 철수 뒤 벽이 남고 실제 정상 CoreApplication/authoring trial에서 생성→피격→제거를 소비한다.'],
 '일반 방해물의 합의된 동작과 최신 Queen의 별도 object 금지를 함께 유지한다. 얼음벽 요구를 기존 엘리트 기믹으로 잘못 해석하지 않는 범위 결정이다.'),
('최대HP 변경·처형·실제 결과', 'combat.md#전투-능력치와-HP; B13; R04/R12 actual results',
 '최대HP 증가는 현재HP를 회복하지 않고 감소는 새 상한으로 clamp한다. HP0이면 사망/파괴 경계를 한 번 거치며 살아 있는0HP를 남기지 않는다. 능력치 변경→HP변경→사망 순서와 실제 원인 attribution을 유지한다. 직접 Kill/특수 kill·HP비용·일반 피해는 기존 결과 종류를 유지하고 Despawn으로 대체하지 않는다. 생성물/분절 잔량/직원 HP를 각각 해당 owner에서 처리하며 구간 수·query·저지 파생값도 같은 실제 잔량을 따른다.',
 ['직원/HP형 생성물의 maxHP 증가·감소·0과 원인 있음/없음에서 순서·중복 없는 종료·attribution을 확인한다.', '기존 직접 Kill/특수 kill/비용/보호/actual HP-loss 회귀가 새 객체 도입 뒤 유지된다.'],
 'B13의 기존 명시 정책과 선행 R04/R12를 유지한다. object에는 직원 사망 보상·부상 규칙을 자동 추가하지 않는다.'),
('접수·취소·재검증과 생성 동작 종료', 'U01 cancel; U05 actual use; combat.md#공명; U07 공명생성동작종료',
 '직원은 유효한 스킬 접수에서 공명을 한 번 전량 소비한다. 현재 타일/상한/비용 부적격을 접수 전 거절하고, 접수한 타일/대상은 실행 시 필요한 상태를 재검증한다. 접수 뒤 다른 명령이 공간·HP를 바꾸어 생성이 실패해도 새로운 비용/부분 객체는 남기지 않으며 이미 접수 때 소비한 공명은 기존 U05대로 환불하지 않는다. 미방출 제어 취소는 미래 생성을 막고 이미 생성된 Persist 객체는 남긴다. 사용 잠금은 생성/진화 동작이 끝나면 종료하며 child TTL 전체를 사용 시간으로 묶지 않는다. 일반 HP-cost effect의 fault 의미는 새 생성의 도메인 거절과 구분한다.',
 ['Pause 중 유효 접수/뒤의 타일점유 또는 HP변화/재개를 실행해 후발 생성 실패와 접수 공명·새 비용 무반영을 구분한다.', 'CC/철수/취소 전후 미방출0개·방출 Persist 유지, child 생존 중 U05 충전 재개를 확인한다.'],
 '생성 원자성을 기존 공명 무환불 계약과 충돌시키지 않는다. 미래 시전의 실패를 이유로 이미 성공한 앞 명령을 되돌리지 않는다.'),
('조회·미리보기·실제 결과의 단일 권위', 'U07 query/normal ports; U06 shared query; CF-D07',
 '공개 조회는 생성 종류/definition/owner/copy·위치·현재 HP/분절 잔량·파생 stack·남은 TTL·형태/epoch·실제 종료 원인을 제공한다. 배치/진화 선택은 비용·거절 사유와 현재 eligible 타일을 Core의 공통 자격에서 설명한다. preview는 HP/공명/pool/ID/RNG/queue/revision을 바꾸지 않으며 실행 권한이 아니다. 정상 명령은 같은 규칙을 현재 상태에 다시 적용하고 실제 생성/진화/공격/파괴 결과를 제공한다. UI가 타일/비용/잔량을 재계산하는 구조는 도입하지 않는다.',
 ['반복 preview/query는 완전 무변경이고 실제 명령의 비용/위치/형태/거절과 일치한다.', 'stale preview 후 변경된 타일·HP·pool·source 상태에서 명시 거절하며 같은 actor 재접속으로 정확한 객체 상태를 조회한다.'],
 'U06의 실제 query 권위와 CF-D07을 새 도메인에 적용한다. Unity 화면은 후속이며 외부 소비 검증은 지금 닫는다.'),
('정지·동시각·결정론', 'U06 Pause/settings; R13 T23; U07 정지/TTL/중복',
 '생성 명령은 Pause 자체로 금지하지 않고 기존 입력 자격/설정 모드 gate를 따른다. 시전·TTL·field tick·함정 탐지와 actor 이동은 battle clock을 사용한다. 같은 timestamp의 피해/사망·객체 수명·생성 settlement·승리 판단은 기존 Core ordering을 보존하며 새 callback의 stable sequence/epoch로 중복을 막는다. container 순회나 pack 장착 순서로 gameplay 결과를 바꾸지 않는다. U05 강제 침식 host clock의 명시 예외가 새 생성 시계를 진행시키지 않는다.',
 ['Pause/0.5/설정/재접속과 같은시각 TTL·타격·진화·출처삭제·승리에서 순서와 결과가 결정적이다.', '같은 seed/input을 다른 container/pack 순서와 log/observation 모드에서 실행해 동일한 실제 결과를 확인한다.'],
 '현재 clock/ordering/candidate 소유자를 유지한다. 타이머를 임의 조정해 경쟁을 피하지 않는다.'),
('clone·재접속·저장/복원 경계', 'U04/U06; results-and-save; R13 새 상태 저장 책임',
 '객체·형태 epoch·source/copy·pool/분절잔량·spawn settlement·timer와 action 연결은 현재 live clone/rollback에서 독립 복사한다. 같은 process 재접속은 현재 객체를 투영하고 명시 Resume 전 시간을 진행하지 않는다. process Continue는 미확정 전투 입장 전 복원이며 U07 live battle disk 저장을 새로 만들지 않는다. 새 definition/compiled 입력이 지원 checkpoint/fixture에 포함되는 경계는 reader/writer를 함께 전환한다. 미지원 구형 입력은 명시 거절하고 원본을 보존하며 save/결과/되감기 자산 의미를 바꾸지 않는다.',
 ['생성/진화/부상 pool/expiry 대기 clone·rollback·같은actor reconnect의 독립성과 exactly-once를 검증한다.', '실제 host Continue/SavePoint/미지원 원본보호 회귀와 새 정상 정의의 저장/복원 consumer를 통과한다.'],
 '새 상태의 저장 책임을 도입 owner가 닫되 현재 미지원인 전투 중 process 복원을 자동 약속하지 않는다.'),
('정상 작성·공식 소비·지원 범위', 'U07 raw→compile→executor/Lab; R13 §6; latest content notes',
 '새 생성/진화/pool/cost 정의는 strict RON parse·definition reference·범위/수치/종류별 validation·compiled executor·정상 CoreApplication·Lab capability를 함께 제공한다. 실행되지 않는 opcode나 없는 능력을 성공 등록하지 않는다. 래티씨아 기본/개화 친구와 공통 얼음벽, 고기벽 fallback을 실제 작성된 소비 사례로 검증하며 감염/거미줄·전체 조우/보상 공급·전용 장비 전체는 U18/U21 경계를 유지한다. 이름/수치/수명/상한의 미정 초기값은 위임 선택과 조절 가능성을 기록한다. 외부 pack이 엔진 수정 없이 공개 가능한 actor/trap/field/wall·진화·분절/비용을 실행하고 미지원 조합을 진단받아야 한다.',
 ['정상 외부 RON→validate→compile→trial/API→실제 결과/query에 각 capability의 nonzero 성공과 strict negative 사례가 있다.', '기본/개화 친구·얼음벽·HPfallback/분절분리 consumer의 실제 명령과 종단 결과를 검증하고 지원 목록/문서가 실제 포트와 일치한다.'],
 '공식 기본 소비와 후속 전체 콘텐츠 통합을 구분한다. 초기 밸런스 값은 사용자의 확정값이 아니며 공개 runtime 기능의 미완성을 future owner로 넘기지 않는다.'),
('native 비용·fault·관측과 구현 품질', 'U02/U03/R05; T23/T27; current continuous request',
 '새 객체/pool/source/queue의 allocation 전 admission·retained/clone/work accounting과 실제 회수를 구현 시작부터 연결한다. positive retained-growth·operation fuel·bounded facts/log/observer 한도를 유지한다. 실패는 기존 typed sticky fault/원자성으로 전파하고 success0개·silent normalization·fixture별 production 분기·catch-all 복구로 숨기지 않는다. PC의 원래 숫자와 입력을 보존하며 의도된 layout/실행비용 변화는 측정과 명시된 비교로 구분한다. 기존 R05 peak-only 경고와20분stress 제외 외 새 예외나 상한 완화는 없다.',
 ['작은 새상태 크기/clone/queue/원자 fault 검증 후 native/full suite·public bounds·5모드 fault·observer/allocator와 PC/R05를 현재 영향에 맞춰 검증한다.', '예산 exact/one-under·실패원본·회수·숫자oracle이 추적되며 구현 guardrail8개를 독립 확인한다.'],
 '선행 안전 계약과 위험에 따른 검증 순서를 유지한다. 비용이 늘면 실제 원인과 영향을 기록하며 예전 실행을 새 실행으로 표시하지 않는다.'),
('전체 검증·독립 감사·후속 인수', 'current resume request; U07 prerequisites and completion; GEX r8',
 'U07 전체 정책과10검증차원·161선행 조건·별도31 R06의 현재 적용성을 추적한다. 유효 증거는 exact input/도구/source/claim 영향이 분리될 때만 재사용한다. 필수 최종 Core/host 통합·공개 생성 소비·복원·수치/성능/관측 검증을 자체 완료한 뒤 정확한 target을 동결한다. 구현 비참여 GPT-6 Astra High 독립 감사와 finding 교정/재감사를 Complete/High로 닫기 전 U08 종속 구현을 시작하지 않는다. 단계/감사 인계에서 재승인을 요구하지 않으며 승인 범위 밖의 새로운 선택 또는 실제 환경 장애만 질문한다.',
 ['각 조건/차원의 실제 command·selector·nonzero 결과·raw/currentness와 범위 제외가 추적되고 current integrated target이 정역 복원된다.', '전체 목표의 독립 Complete/High와 open finding0 이후에만 U08 정상 진입으로 인수한다.'],
 '현재 연속 요청의 종료·독립성·재사용 조건을 그대로 적용한다. 단계 형식 통과를 제품 또는 감사 완료로 표현하지 않는다.'),
]


def main():
    intake, before = json.loads((HERE / 'P2G_00_INITIAL_001.json').read_text())['artifacts']
    assert before['payload']['Current Policy Element Revision Map'] == {}
    assert json.loads((HERE / 'P2G_00_RESULT_001.json').read_text())['status'] == 'complete'
    basis = {'Input Working Revision Ref': before['ref'], 'Exact User-Approved Target Scope/Change Request': None,
             'Policy Intake Snapshot Ref': intake['ref']}
    support_paths = ['core_enhanced/docs/goals/content_authoring_r08_r12_runtime/P2G_POLICY_AND_REVIEW.json',
                     'core_enhanced/docs/goals/core_independent_objects/P2G_10_SUPPORT_001.json']
    with tarfile.open(HERE / 'POLICY_SUPPORT_SOURCE_001.tar.gz', 'x:gz') as archive:
        for name in support_paths:
            archive.add(ROOT / name, arcname=name, recursive=False)
    sources = [{'path': name, 'sha256': sha(ROOT / name)} for name in support_paths]
    artifacts, current, decisions, dispositions = [], {}, [], []
    for n, (name, span, statement, acceptance, reason) in enumerate(POLICIES, 1):
        eid = f'PE-CIOB-{n:02}'
        opened = artifact('Policy Element Revision', f'PER-CIOB-{n:02}-OPEN-001', {
            'Stable Element ID': eid, 'Predecessor Policy Element Revision Ref': None, 'Policy Statement': statement,
            'Decision Revision Ref': None, 'Observable Policy Acceptance': [], 'Intrinsic status': 'open'})
        decision = artifact('Decision Revision', f'DR-CIOB-{n:02}-001', {
            'Predecessor Decision Revision Ref': None, 'Affected Element IDs': [eid], 'Decision Name': name,
            'Exact Decision': statement,
            'Decision Authority': 'AI delegated application/detail decision under current explicit continuous request and CF-D06; quoted adopted contracts are not new user answers',
            'Exact User Approval Reference': {'intake_ref': intake['ref'], 'item': 'IN-001', 'delegation': 'IN-004 CF-D06/07', 'scope': 'IN-002 U07/B13,C01'},
            'Source Reference': {'intake_ref': intake['ref'], 'span': span, 'additional_exact_policy_sources': sources},
            'Reason': reason, 'Impact': acceptance,
            'Alternatives not selected': 'boss-only SideOpponent replication; all objects as employee Unit; implicit stack truth; cost first/partial create; source resurrection; premature full U18/U21 content; fabricated success or numerical oracle relaxation',
            'Delegation Criteria': {'existing_agreements_preserved': 'Current rulebook/U07 and latest Laetitia/Queen/meatwall adopted meanings prevail over old placeholders.',
                'derived_from_ownership': reason,
                'no_unapproved_global_rule': 'No new reward/wave/compendium authority, universal gameplay cap, refund, process-save or historical asset reinterpretation. Named content initial values stay authored tuning under expressly delegated in-scope decisions.',
                'local_impact': 'U07 creation/evolution/HP resource/actual consumers; infection, relic causality and full content acquisition remain assigned later.',
                'user_data_preserved': 'No reset/stash/delete or old-save rewriting; frozen evidence and prior failures preserved.'},
            'Revisability': 'Gameplay contract changes require the earliest P2G successor owner; exact initial tuning recorded with normal validation and affected regression.'})
        decided = artifact('Policy Element Revision', f'PER-CIOB-{n:02}-DECIDED-001', {
            'Stable Element ID': eid, 'Predecessor Policy Element Revision Ref': opened['ref'],
            'Policy Statement': statement, 'Decision Revision Ref': decision['ref'],
            'Observable Policy Acceptance': acceptance, 'Intrinsic status': 'decided'})
        disposition = artifact('Candidate Policy Element Disposition Record', f'CD-CIOB-{n:02}-001', {
            'Candidate ID': f'C-CIOB-{n:02}', 'P2G-10 Evaluation Basis': basis,
            'Source Reference': decision['payload']['Source Reference'], 'Candidate Statement': statement,
            'Disposition': 'promoted', 'Stable Element ID': eid,
            'Initial Open Policy Element Revision Ref': opened['ref'], 'Decision Revision Ref': decision['ref']})
        artifacts.extend([opened, decision, decided, disposition])
        current[eid] = decided['ref']; decisions.append(decision['ref']); dispositions.append(disposition['ref'])
    for n, (statement, owner) in enumerate([
        ('전종 관계/감염/시체소유권·붉은 실타래 거미줄 연결과 연쇄', 'U07 explicit exclusion; U18, U21'),
        ('전체 엘리트 조우/전용장비/획득보상·유물 연쇄를 U07 생성 소비와 함께 완성', 'ROADMAP U08–10/U18/U19/U21 owners; U07 official consuming subset only'),
        ('Unity UI·개별 최종보스 고유 구현·전체 미래 상세 설계', 'IN-001 explicit constraints'),
    ], 1):
        item = artifact('Candidate Policy Element Disposition Record', f'CD-CIOB-OUT-{n:02}-001', {
            'Candidate ID': f'C-CIOB-OUT-{n:02}', 'P2G-10 Evaluation Basis': basis,
            'Source Reference': {'intake_ref': intake['ref'], 'span': 'IN-002 U07 exclusions and assigned later owners; IN-001'},
            'Candidate Statement': statement, 'Disposition': 'out_of_scope', 'Evidence/Decision Reference': owner,
            'Stable Element ID': None, 'Initial Open Policy Element Revision Ref': None})
        artifacts.append(item); dispositions.append(item['ref'])
    payload = copy.deepcopy(before['payload'])
    payload.update({'Predecessor Working Revision Ref': before['ref'], 'Current Policy Element Revision Map': current,
                    'Decision Revision Ref Set': decisions})
    result = artifact('Working Revision', 'WR-CIOB-002', payload); artifacts.append(result)
    for item in artifacts:
        assert artifact(item['ref']['Artifact Type'], item['ref']['Artifact ID'], item['payload']) == item
        if item['ref']['Artifact Type'] == 'Policy Element Revision':
            p = item['payload']; assert p['Intrinsic status'] in ['open', 'decided']
            assert isinstance(p['Observable Policy Acceptance'], list)
            if p['Intrinsic status'] == 'decided':
                assert p['Observable Policy Acceptance'] and p['Decision Revision Ref'] in decisions
    assert len(current) == len(decisions) == 20 and len(dispositions) == 23
    assert len({item['ref']['Artifact ID'] for item in artifacts}) == len(artifacts)
    write('P2G_10_POLICY_001.json', {'artifacts': artifacts})
    write('P2G_10_RESULT_001.json', {
        'stage': 'P2G-10', 'status': 'complete', 'Input Working Revision Ref': before['ref'],
        'Result Working Revision Ref': result['ref'], 'Produced Candidate Disposition Ref Set': dispositions,
        'Result Current Candidate Policy Element Disposition Effective View': dispositions, 'is_p2g10_complete': True,
        'checks': {'all23_candidates_dispositioned': True, 'all20_elements_decided': True,
                   'single_leaf_decisions': True, 'observable_acceptances_consistent': True,
                   'explicit_delegation_per_decision': True, 'raw_intake_unchanged': True,
                   'open_questions_empty': True, 'refs_digests_fields_checked_before_publication': True},
        'Finding Triage': [
            {'classification': 'material_policy', 'disposition': '20 exact delegated decisions applying accepted U07/R13/current contracts; source/TTL/evolution/segmentedHP/cost/query boundaries explicit', 'elements': list(current)},
            {'classification': 'material_policy', 'disposition': 'Ice wall is a common official authored consumer; latest Queen encounter remains three actions without new objects. Full infection/corpse producer and relic web remain U18/U21.'},
            {'classification': 'implementation_finding', 'disposition': 'Limited source support only: existing actor schema requires motion/boss behavior, boss adapter stamps Opponent, R12 pool is nonsegmented and HealthCost has existing typed facts. Formal full review next in P2G20.', 'evidence': 'P2G_10_SUPPORT_001.json'},
            {'classification': 'out_of_scope', 'disposition': '3 exact out-of-scope candidates above; no scope expansion'},
        ],
        'Open Policy Question Log': [], 'Evidence Gap Log': [], 'additional_policy_sources': sources,
        'source_support_archive_sha256': sha(HERE / 'POLICY_SUPPORT_SOURCE_001.tar.gz'),
        'new_user_request_required': False, 'next': 'P2G-20', 'production_mutation': False})
    view = (HERE / 'WORKING_POLICY.ko.md').read_text().replace('`WR-CIOB-001`', '`WR-CIOB-002`')
    view = view.replace('현재 P2G-00 완료, 다음은 P2G-10이다. 초기 정책 요소·결정·미결 질문은 비어 있다.\nP2G-10 Completion Predicate: N/A. 초기화는 정책 의미·코드 검토·동결·제품 구현 완료가 아니다.',
                        '현재 P2G-10 완료, 다음은 P2G-20 정식 코드 검토다.20개 정책·결정과23개 candidate(승격20/범위외3), 미결질문0개다.\nP2G-10 Completion Predicate: true. [정책/결정](P2G_10_POLICY_001.json)·[검증](P2G_10_RESULT_001.json)이 근거이며 코드 검토/동결/구현은 아직이다.')
    view += '\n정책 선택은 현재 사용자 위임에 따른 AI 결정이다. 원문에 없는 source snapshot 시점·형태 TTL·HP carry·pool 분리·얼음벽 소비 경계를 사용자 직접 답변으로 표시하지 않는다.\n[제한적 사실 확인](P2G_10_SUPPORT_001.json)은 정식 P2G20 code review가 아니다.\n\n'
    for n, (name, span, statement, acceptance, reason) in enumerate(POLICIES, 1):
        view += f'### PE-CIOB-{n:02} — {name}\n\n{statement}\n\n근거: {span}. 결정 이유: {reason}\n\n'
        view += '\n'.join('- ' + value for value in acceptance) + '\n\n'
    (HERE / 'WORKING_POLICY.ko.md').write_text(view)
    print('U07 P2G-10 complete:20 decided elements/23 dispositions; no product mutation; P2G20 next')


if __name__ == '__main__':
    main()
