# 실제 코드 기반 의존성 판정과 전환 경계

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

## 1. 관계 종류

| 종류 | 판정 |
| --- | --- |
| 기능상필수 | 후행의 실제 소비 지점이 선행의 특정 계약/상태/결과를 필요로 한다. |
| 동시전환 | 분리하면 reader/writer·authority·transaction·장착/저장/실행 의미가 불일치한다. |
| 검증선행 | 후행 완료를 증명할 실제 데이터·관찰·실행 증거가 필요하다. |
| 진행상권장 | 독립 구현은 가능하지만 재작업·공용 파일 충돌을 줄이는 순서다. |
| 조건부 | 특정 정책·지원 capability·도입 순서가 정해질 때만 관계가 활성화된다. |

현행 call/data dependency와 **목표 구조로 전환하기 위한 설계 dependency**는 같지 않다. 아래 판단은 관찰된 source와 승인 목표에서 도출한 설계이며 실제 새 구현의 실행 결과가 아니다.

## 2. 기존 49개 선행 관계의 전수 disposition

원본 `GOALS.json`의 CA별 dependency 목록을 기준으로 49쌍을 모두 남겼다. 삭제/축소된 화살표도 이유·현행 재사용·조건이 있다.

### OLD-00-01: CA00 → CA01

**판정:** 유지·검증 관계로 명료화. **관계:** 검증선행.

**선행 결과 → 실제 후행 소비:** CA00 승인/미정 구별→R01 최소 schema와 V01–06.

**이유:** 코드 기능의 호출 선행이 아니라 범위·정책 출처의 기준선. **잘못 해석/분리하면:** 초안을 최소 Goal의 승인 의미로 승격.

**기존 충족/재사용:** 현재 CA00와 formal P2G-60 성과 존재. **남은 조건:** 기존 성과를 다시 생성하지 않음.

근거: [S005](../evidence/SOURCES.ko.md#s005) · [S006](../evidence/SOURCES.ko.md#s006) · [S007](../evidence/SOURCES.ko.md#s007) · [S009](../evidence/SOURCES.ko.md#s009).

### OLD-01-02: CA01 → CA02

**판정:** 세분화. **관계:** 기능상필수, 진행상권장.

**선행 결과 → 실제 후행 소비:** R01 최소 typed schema/컴파일→R02 후보 게시; 미래 전체 opcode 불필요.

**이유:** 기존 embedded loader에 외부 source를 붙이는 최소 slice가 독립 유효. **잘못 해석/분리하면:** R02가 미지원 schema를 게시하거나 임시 성공 사용.

**기존 충족/재사용:** load_live_sources validate→Arc 게시. **남은 조건:** stable owned source locator·fingerprint는 R02에서 확장.

근거: [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S017](../evidence/SOURCES.ko.md#s017) · [S010](../evidence/SOURCES.ko.md#s010).

### OLD-02-03: CA02 → CA03

**판정:** 세분화. **관계:** 기능상필수, 진행상권장.

**선행 결과 → 실제 후행 소비:** R02 external source→R03 pack CLI; Describe 자체는 R01로 가능.

**이유:** 외부팩 지원 CLI와 일반 descriptor query를 구분. **잘못 해석/분리하면:** public pack workflow를 private builder로 대체.

**기존 충족/재사용:** CoreApplication construction/기존 test seam. **남은 조건:** R03 전체 완료는 kernel 구현의 선행 아님.

근거: [S012](../evidence/SOURCES.ko.md#s012) · [S014](../evidence/SOURCES.ko.md#s014) · [S017](../evidence/SOURCES.ko.md#s017).

### OLD-03-04: CA03 → CA04

**판정:** 권장으로 재분류. **관계:** 진행상권장, 검증선행.

**선행 결과 → 실제 후행 소비:** R03 공개 관찰포트→R05 외부 RON 인수; ResourceStore 자체와 별개.

**이유:** 기존 BattleCore/transaction seam으로 상태 시험 가능. **잘못 해석/분리하면:** R03 없이 kernel 작성은 가능하나 public authoring 완료 증거는 부족.

**기존 충족/재사용:** 현재 unit/integration fixture와 snapshot. **남은 조건:** 기존 seam으로 증거 충족 시 kernel 병행 가능; 기본안은 순차.

근거: [S014](../evidence/SOURCES.ko.md#s014) · [S027](../evidence/SOURCES.ko.md#s027) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038).

### OLD-04-05: CA04 → CA05

**판정:** 세분화. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R05 ResourceRef/수명→R06 자원 참조 식.

**이유:** 상수/stat 식 전체가 generic state store를 요구하지 않음. **잘못 해석/분리하면:** resource evaluator가 해제된 owner/임의 global값을 읽음.

**기존 충족/재사용:** 현재 UnitStats scalar와 stat_pipeline. **남은 조건:** ResourceRef를 약속하는 R06 slice부터 필요.

근거: [S030](../evidence/SOURCES.ko.md#s030) · [S043](../evidence/SOURCES.ko.md#s043) · [S027](../evidence/SOURCES.ko.md#s027).

### OLD-04-06: CA04 → CA06

**판정:** 제거·계약만 분리. **관계:** 진행상권장.

**선행 결과 → 실제 후행 소비:** K05 owner/수명 설계→R04 receipt/cast handle; R05 full runtime은 후행 가능.

**이유:** 현재 damage/step/source snapshot은 generic ResourceStore 없이 실행. **잘못 해석/분리하면:** owner/수명 계약을 생략하면 새 receipt가 영구 로그에 결속.

**기존 충족/재사용:** cast_seq/step_progress/current source snapshot. **남은 조건:** 기능상 CA04 전체 선행 삭제. R04 local receipt 수명을 먼저 구현.

근거: [S019](../evidence/SOURCES.ko.md#s019) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S027](../evidence/SOURCES.ko.md#s027).

### OLD-05-06: CA05 → CA06

**판정:** 기능상 선행 제거. **관계:** 진행상권장.

**선행 결과 → 실제 후행 소비:** 기존 literal 피해/회복→R04 결과; 새 ValueExpr는 R06.

**이유:** 결과 계측에 SpellPower/임의 식 compiler가 필요하지 않음. **잘못 해석/분리하면:** 새 계산과 사건 인식을 혼합하면 기존 HP-loss 의미 변경.

**기존 충족/재사용:** 기존 actual_damage_target_count/ApplyHeal. **남은 조건:** 새 식이 결과 참조할 때는 반대로 R04→R06 필요.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024).

### OLD-05-07: CA05 → CA07

**판정:** 유지·필수 계약 축소. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R06 typed ValueExpr와 결과참조→R10 Branch/Repeat/amount.

**이유:** Flow가 요구하는 typed predicate/count 입력에 한정. **잘못 해석/분리하면:** 없는/미완료 result slot·단위 오류 실행.

**기존 충족/재사용:** 현재 step delay/repeat/when 기본 경로. **남은 조건:** 단순 Sequence는 독립 가능하나 R10 완료범위에는 typed 식 포함.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019).

### OLD-06-07: CA06 → CA07

**판정:** 유지. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R04 ResultReceipt/ActionHandle→R10 wait/join.

**이유:** 현재 hit/terminal 차이가 있으므로 완료 기준 필수. **잘못 해석/분리하면:** 첫 타격과 전체 전달 종료를 혼동.

**기존 충족/재사용:** 현재 pending delivery·deferred gate·cast finish. **남은 조건:** 전체 reaction chain은 필요 없음.

근거: [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021).

### OLD-07-08: CA07 → CA08

**판정:** 유지·세분화. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R10 request/continuation/cancel→R11 channel/reinput.

**이유:** 유한 flow와 지속 activation 종료를 연결. **잘못 해석/분리하면:** 만료·재입력 이중 발동 또는 lease 누수.

**기존 충족/재사용:** 현재 action locks와 기본 공격 경계. **남은 조건:** 원형 모든 Flow construct가 channel마다 필요한 것은 아님.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S029](../evidence/SOURCES.ko.md#s029).

### OLD-06-09: CA06 → CA09

**판정:** 유지. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R04 damage/cost/kill receipt→R12 흡수/생존 중재.

**이유:** HP-loss만으로 shield hit와 실제 잔량 소비를 설명할 수 없음. **잘못 해석/분리하면:** 중복 흡수·OnHit 의미 변조·HP비용을 damage로 처리.

**기존 충족/재사용:** 기존 감쇄/죽음 경계. **남은 조건:** HP1/처형/개인공유 우선순위 PG04.

근거: [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S028](../evidence/SOURCES.ko.md#s028).

### OLD-08-09: CA08 → CA09

**판정:** 세분화. **관계:** 기능상필수, 진행상권장.

**선행 결과 → 실제 후행 소비:** R11 Membership/lease→R12 shared pool 참가/이탈; Hello 불필요.

**이유:** 수혜자별 pool 복제 대신 원천 pool과 관계 필요. **잘못 해석/분리하면:** 이탈 대상 보호 잔존/초기량 중복 적용.

**기존 충족/재사용:** 기존 unit lifecycle cleanup. **남은 조건:** membership만 먼저 안정화하면 다른 sustained 기능 전체를 기다릴 필요 없음.

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S029](../evidence/SOURCES.ko.md#s029).

### OLD-05-10: CA05 → CA10

**판정:** 권장으로 재분류. **관계:** 진행상권장, 조건부.

**선행 결과 → 실제 후행 소비:** R06 성장 입력→R07 구형 파편 강화 제거와 profile 동시성.

**이유:** copy identity/형태 선택 자체는 현재 SkillId로 유효. **잘못 해석/분리하면:** 성장 재설계를 정책 없이 정상 강화 제거에 혼합.

**기존 충족/재사용:** 장비 enhancement·현재 base/awakened skill 참조. **남은 조건:** 공식 성장 수식 변경만 PG05/08; R07 impl dependency에서 R06 제외.

근거: [S030](../evidence/SOURCES.ko.md#s030) · [S031](../evidence/SOURCES.ko.md#s031) · [S034](../evidence/SOURCES.ko.md#s034) · [S042](../evidence/SOURCES.ko.md#s042).

### OLD-08-10: CA08 → CA10

**판정:** 기능상 선행 제거. **관계:** 진행상권장.

**선행 결과 → 실제 후행 소비:** R07 copy/form→현재 profile; 고급 form 행동은 R11 이후.

**이유:** 사본화·개화 선택은 channel/basic override 완성과 독립. **잘못 해석/분리하면:** 후행 form 기믹을 copy ledger 기본 완료에 끼워넣으면 불필요 지연.

**기존 충족/재사용:** 현재 base/awakened SkillId 프로필. **남은 조건:** 고급 형태 내용은 해당 실행 owner가 추가.

근거: [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S042](../evidence/SOURCES.ko.md#s042).

### OLD-04-11: CA04 → CA11

**판정:** 세분화. **관계:** 기능상필수, 진행상권장.

**선행 결과 → 실제 후행 소비:** R05 source/lifetime→R14 battle-use relic state; 관리 aggregate는 독립.

**이유:** Run 자산 관리와 battle proc 사용 기록을 구별. **잘못 해석/분리하면:** 재배치마다 battle 사용횟수 복구/사본간 cooldown 공유.

**기존 충족/재사용:** Equipment ownership·EconomyGrantState. **남은 조건:** 숫자/관리만 먼저 만드는 별도 분할도 가능하나 기본안은 R14 통합.

근거: [S027](../evidence/SOURCES.ko.md#s027) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S045](../evidence/SOURCES.ko.md#s045).

### OLD-10-11: CA10 → CA11

**판정:** 권장으로 재분류·일부 동시. **관계:** 진행상권장, 동시전환.

**선행 결과 → 실제 후행 소비:** R07 copy/profile binding→R14 fragment별 effective plan; RelicCopy 관리 자체는 독립.

**이유:** 서로 다른 자산 aggregate이나 roster/inventory/save 소비 경계 공유. **잘못 해석/분리하면:** 두 장부/장착표/저장 shape를 따로 바꿔 상태 불일치.

**기존 충족/재사용:** 기존 장비 copy와 양방향 transition. **남은 조건:** 관리만 분리할 수 있지만 개인 합성까지 닫는 R14는 R07 binding 사용.

근거: [S031](../evidence/SOURCES.ko.md#s031) · [S033](../evidence/SOURCES.ko.md#s033) · [S035](../evidence/SOURCES.ko.md#s035) · [S038](../evidence/SOURCES.ko.md#s038) · [S045](../evidence/SOURCES.ko.md#s045).

### OLD-09-12: CA09 → CA12

**판정:** 세분화. **관계:** 기능상필수, 진행상권장.

**선행 결과 → 실제 후행 소비:** R12 pool→R15 shared-to-single recipe; R14 scalar composer에는 불필요.

**이유:** 유물 수치 변경과 구조 변경의 실행 요구가 다름. **잘못 해석/분리하면:** 미구현 shield opcode를 지원으로 공개.

**기존 충족/재사용:** 기존 literal Damage/Heal로 최소 수치 합성 가능. **남은 조건:** S 계산식/개인공유 순서 PG04/06.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S028](../evidence/SOURCES.ko.md#s028) · [S045](../evidence/SOURCES.ko.md#s045).

### OLD-10-12: CA10 → CA12

**판정:** 세분화. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R07 copy/form identity→R14/R15 source binding·cache key.

**이유:** compiled template와 실제 소유 사본을 분리해야 함. **잘못 해석/분리하면:** 같은 SkillId/사본 캐시 공유로 자원·출처 혼합.

**기존 충족/재사용:** 현재 SkillId·직원 owned UUID. **남은 조건:** 전체 파편 콘텐츠·개화 기믹이 아니라 copy/form 계약.

근거: [S027](../evidence/SOURCES.ko.md#s027) · [S030](../evidence/SOURCES.ko.md#s030) · [S031](../evidence/SOURCES.ko.md#s031) · [S042](../evidence/SOURCES.ko.md#s042).

### OLD-11-12: CA11 → CA12

**판정:** 최소 수치 slice와 동시 전환. **관계:** 동시전환.

**선행 결과 → 실제 후행 소비:** R14 relic grant/loadout→effective plan→all runtime/preview consumers.

**이유:** 관리 완료와 빈 합성 executor의 연결을 피하는 기본 분해. **잘못 해석/분리하면:** 장착 성공하지만 실행은 전역 원형 사용.

**기존 충족/재사용:** 기존 EquipmentLoadoutTransition·직접 skill consumers. **남은 조건:** 관리-only 독립 단위도 유효하나 그 경우 실행 효과 완료라고 하지 않아야 함.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S033](../evidence/SOURCES.ko.md#s033) · [S035](../evidence/SOURCES.ko.md#s035) · [S045](../evidence/SOURCES.ko.md#s045).

### OLD-06-13: CA06 → CA13

**판정:** 유지. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R04 committed fact/source→R16 event dispatcher.

**이유:** 기존 OnAttack/OnHit는 전체 damage event 아님. **잘못 해석/분리하면:** reactive event를 기존 basic trigger로 뭉개거나 죽음 bookkeeping 억제.

**기존 충족/재사용:** 현재 source snapshots/trigger 순서. **남은 조건:** 미정 accepted-hit 의미는 해당 predicate gate.

근거: [S023](../evidence/SOURCES.ko.md#s023) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027).

### OLD-12-13: CA12 → CA13

**판정:** 세분화. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R14 source-bound plan + R15 유한 grant closure→R16 reaction definition.

**이유:** 개인 plan/생성 effect identity는 필요하나 모든 구조 recipe는 불필요. **잘못 해석/분리하면:** generated effect provenance·재방문 키 손실.

**기존 충족/재사용:** definition/cast/proc identity 일부 존재. **남은 조건:** runtime chain과 compile closure를 같은 visited로 만들지 않음.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S026](../evidence/SOURCES.ko.md#s026) · [S045](../evidence/SOURCES.ko.md#s045).

### OLD-04-14: CA04 → CA14

**판정:** 유지·계약 축소. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R05 owner/lifetime/transaction→R13 object 생성/회수.

**이유:** source 철수 뒤 존속과 수명 종료를 별도로 판단. **잘못 해석/분리하면:** owner 종료와 객체 종료 혼동·늦은 timer의 새 객체 삭제.

**기존 충족/재사용:** 현재 summon lifetime/count release. **남은 조건:** 범용 모든 resource 종류가 필요한 것은 아님.

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051).

### OLD-07-14: CA07 → CA14

**판정:** 세분화. **관계:** 기능상필수, 진행상권장.

**선행 결과 → 실제 후행 소비:** R10 Spawn Invoke/완료 결과→R13 typed action adapter.

**이유:** 기본 SpawnKernel 자체는 이미 존재하고 full Flow와 독립. **잘못 해석/분리하면:** spawn 실패를 정상 효과 성공으로 처리.

**기존 충족/재사용:** 현재 원자적 SpawnKernel·boss pending task. **남은 조건:** 공통 action adapter에 필요한 handle과 request만 선행.

근거: [S047](../evidence/SOURCES.ko.md#s047) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050).

### OLD-09-14: CA09 → CA14

**판정:** 세분화. **관계:** 조건부, 기능상필수.

**선행 결과 → 실제 후행 소비:** R12 HP-cost mediation→R13 meatwall; actor/trap 기본은 독립.

**이유:** 비용을 지불하는 생성의 상태·실패 원자성에만 필수. **잘못 해석/분리하면:** HP 차감 후 생성 실패/반대로 무료 object 생성.

**기존 충족/재사용:** SpawnKernel preflight·clone rollback. **남은 조건:** HP cost 없는 actor slice는 CA09 전체 선행 불필요.

근거: [S024](../evidence/SOURCES.ko.md#s024) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051).

### OLD-10-14: CA10 → CA14

**판정:** 세분화. **관계:** 조건부, 기능상필수.

**선행 결과 → 실제 후행 소비:** R07 source fragment copy/form→R13 Laetitia 진화/소유.

**이유:** 보스 소환은 이미 copy 없는 owner도 처리. **잘못 해석/분리하면:** 내 source/남의 source 함정 합치거나 form 선택 오염.

**기존 충족/재사용:** boss state/task activation owner. **남은 조건:** 기본 보스 actor는 독립; 파편 생성 범위부터 copy binding.

근거: [S031](../evidence/SOURCES.ko.md#s031) · [S042](../evidence/SOURCES.ko.md#s042) · [S047](../evidence/SOURCES.ko.md#s047) · [S050](../evidence/SOURCES.ko.md#s050).

### OLD-13-15: CA13 → CA15

**판정:** 세분화. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R16 inherited ancestry/suppression→R17 전파·기폭.

**이유:** 정적 선/멤버십만은 독립이나 반응 전파는 인과가 필수. **잘못 해석/분리하면:** 기폭이 재감염/재전파를 무한 생성.

**기존 충족/재사용:** 현재 pre-death/area owner 일부. **남은 조건:** join·전파 순서 PG02/03; 순수 geometry는 별도 개발 가능.

근거: [S021](../evidence/SOURCES.ko.md#s021) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S045](../evidence/SOURCES.ko.md#s045).

### OLD-14-15: CA14 → CA15

**판정:** 유지·계약 축소. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R13 object lifecycle/identity→R17 link invalidation.

**이유:** 진화/삭제 후 관계가 실제 존재 객체를 참조해야 함. **잘못 해석/분리하면:** stale edge·consume 재발동.

**기존 충족/재사용:** 기존 summon/스케줄 generation 처리. **남은 조건:** 감염 membership을 trap graph와 같은 구조로 강제하지 않음.

근거: [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052).

### OLD-07-16: CA07 → CA16

**판정:** 세분화·조기 포트 분리. **관계:** 기능상필수, 진행상권장.

**선행 결과 → 실제 후행 소비:** R04 ActionHandle→R08 boss Invoke; R10 full Flow는 후행.

**이유:** 현재 executor를 공통 port로 감싸 최소 보스 명시 요청이 가능. **잘못 해석/분리하면:** Accepted=Completed, silent invoke bypass.

**기존 충족/재사용:** compiled state/task result와 current skills. **남은 조건:** 명시 시전 authority/facing/cancel은 R08 동시 소유.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S022](../evidence/SOURCES.ko.md#s022) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048).

### OLD-08-16: CA08 → CA16

**판정:** 세분화. **관계:** 기능상필수, 진행상권장.

**선행 결과 → 실제 후행 소비:** R11 profile/lease→R18 state effects; R08 기본 호출과 독립.

**이유:** 형태 modifier의 가역성은 필요하나 channel 전체 불필요. **잘못 해석/분리하면:** phase 이탈 후 profile 잔존/타 owner 해제.

**기존 충족/재사용:** 현재 boss state activation/cancel owner. **남은 조건:** R18 profile output만 필수.

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S029](../evidence/SOURCES.ko.md#s029) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048).

### OLD-09-16: CA09 → CA16

**판정:** 권장·조건부로 재분류. **관계:** 진행상권장, 조건부.

**선행 결과 → 실제 후행 소비:** R18 damage predicate hook; shared shield 전체와 독립.

**이유:** 현재 incoming damage modifier 경계 재사용 가능; R18은 거리조건 일반 predicate를 자체 소유 가능. **잘못 해석/분리하면:** 미정 shield 기능 전체를 boss 호출 gate로 만들면 불필요 대기.

**기존 충족/재사용:** 현재 damage pipeline·state graph. **남은 조건:** 공유 shield 사용하는 특정 boss만 R12 pool의 조건부 소비자.

근거: [S023](../evidence/SOURCES.ko.md#s023) · [S028](../evidence/SOURCES.ko.md#s028) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048).

### OLD-14-16: CA14 → CA16

**판정:** 세분화. **관계:** 기능상필수, 진행상권장.

**선행 결과 → 실제 후행 소비:** R13 actor adapter→R18 다중 actor 확장; R08 기본 호출 독립.

**이유:** 현재 보스 소환 공용 kernel 이미 있음; 새 actor capability만 추가 소비. **잘못 해석/분리하면:** 소환 batch/death/wave authority 혼합.

**기존 충족/재사용:** SpawnKernel·boss-bound existing summon. **남은 조건:** 모든 trap/meatwall을 R18 기본 호출의 선행으로 삼지 않음.

근거: [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S053](../evidence/SOURCES.ko.md#s053).

### OLD-10-17: CA10 → CA17

**판정:** 세분화. **관계:** 기능상필수, 동시전환.

**선행 결과 → 실제 후행 소비:** R07 grant/profile/save→R19 복수 파편 선택.

**이유:** 기본 지급은 R07부터, 복수 보상 policy graph는 R19. **잘못 해석/분리하면:** 아이템만 정의되고 정상 획득/재획득/저장 경로 부재.

**기존 충족/재사용:** 현재 RewardEffect::GrantSkillFragment/양방향 장착. **남은 조건:** 원문 일반/퍼펙트 보상 의미 PG06.

근거: [S031](../evidence/SOURCES.ko.md#s031) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042).

### OLD-11-17: CA11 → CA17

**판정:** 세분화. **관계:** 기능상필수, 동시전환.

**선행 결과 → 실제 후행 소비:** R14 relic grant→R19 shop/selection/all-domain registry.

**이유:** 유물 기본 획득은 R14와 같이 유효해야 함. **잘못 해석/분리하면:** R19까지 grant를 미루면 앞 관리 단계 정상 사용 불가.

**기존 충족/재사용:** 현재 common GrantExecutor/preview transaction. **남은 조건:** 유물 drop/가격은 승인값만.

근거: [S033](../evidence/SOURCES.ko.md#s033) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S045](../evidence/SOURCES.ko.md#s045).

### OLD-12-17: CA12 → CA17

**판정:** 세분화. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R14/15 effective summary→R19 Certified 모델.

**이유:** 현재 active_skill_flat은 모든 합성 효과 평가가 아님. **잘못 해석/분리하면:** 원형 skill 기준 power/preview와 실제 plan 불일치.

**기존 충족/재사용:** 현재 certified profile/고정 weights/버전. **남은 조건:** 숫자 가중 PG08 gate; 입력 연결은 도입 단위부터.

근거: [S030](../evidence/SOURCES.ko.md#s030) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056).

### OLD-16-17: CA16 → CA17

**판정:** 세분화. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R08/18 boss contracts→R19 encounter 생성/등록 소비.

**이유:** 모든 보스 콘텐츠 완성이 아니라 등록할 task capability가 필요. **잘못 해석/분리하면:** 정의는 있으나 encounter setup/결과권한 검증 실패.

**기존 충족/재사용:** 현재 pve/boss catalog setup 경계. **남은 조건:** 미지원 신규 기믹은 publication reject.

근거: [S013](../evidence/SOURCES.ko.md#s013) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S053](../evidence/SOURCES.ko.md#s053).

### OLD-06-18: CA06 → CA18

**판정:** 유지. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R04 mandatory DeathReceipt/suppression→R09 침식.

**이유:** 일반 큰 damage/전역 death callback 중단으로 구현 불가. **잘못 해석/분리하면:** 미래 wave 멈춤 또는 OnKill/OnDeath 추가 발동.

**기존 충족/재사용:** finalize death의 wave bookkeeping. **남은 조건:** 일반 처치와 통계/도감 세부는 승인 계약만.

근거: [S023](../evidence/SOURCES.ko.md#s023) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S046](../evidence/SOURCES.ko.md#s046).

### OLD-09-18: CA09 → CA18

**판정:** 기능상 선행 제거. **관계:** 진행상권장.

**선행 결과 → 실제 후행 소비:** 현재 kill/lifecycle→R09; shield 우회는 특수명시 kill.

**이유:** 침식은 일반 damage가 아니므로 full shield/parry 불필요. **잘못 해석/분리하면:** shield에 막히는 거대damage로 침식 대체.

**기존 충족/재사용:** 현재 death/withdraw 경로. **남은 조건:** 후속 R12는 침식 regression 재인수.

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S025](../evidence/SOURCES.ko.md#s025) · [S046](../evidence/SOURCES.ko.md#s046).

### OLD-10-18: CA10 → CA18

**판정:** 세분화. **관계:** 기능상필수.

**선행 결과 → 실제 후행 소비:** R07 선택 파편·사본·form→R09 실제 시전 자격.

**이유:** 최종 보스 파편 제외와 일반 파편 시전 binding이 필요. **잘못 해석/분리하면:** 구형 세트4종/별도입력 조건을 되살리거나 잘못된 파편 분류.

**기존 충족/재사용:** 현재 SkillFragment origin/form/직원 eligibility. **남은 조건:** 최종 보스 전투 제외가 아니라 파편 제외.

근거: [S031](../evidence/SOURCES.ko.md#s031) · [S042](../evidence/SOURCES.ko.md#s042) · [S046](../evidence/SOURCES.ko.md#s046).

### OLD-13-18: CA13 → CA18

**판정:** 선행 제거·후행 회귀로 전환. **관계:** 검증선행, 진행상권장.

**선행 결과 → 실제 후행 소비:** R04 suppression→R09; R16 chain 추가 시 침식 회귀.

**이유:** chain 전체 없이 optional dispatch를 끊을 수 있음. **잘못 해석/분리하면:** 후속 subscriber가 suppressed fact를 다시 정상발동.

**기존 충족/재사용:** 현재 triggers와 death 분리 지점. **남은 조건:** R16 완료 증거에 R09 규칙 재검증 의무.

근거: [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S045](../evidence/SOURCES.ko.md#s045) · [S046](../evidence/SOURCES.ko.md#s046).

### OLD-16-18: CA16 → CA18

**판정:** 기능상 선행 제거. **관계:** 진행상권장.

**선행 결과 → 실제 후행 소비:** R09 special action/presentation→현재 field kill.

**이유:** 원본 환상체 등장은 보스 AI/보상권한을 가진 actor일 필요 없음. **잘못 해석/분리하면:** 연출 actor가 wave 승리를 막거나 별도 보상 생성.

**기존 충족/재사용:** 현재 death/withdraw/field 분리. **남은 조건:** UI 구현은 비범위; typed presentation 사실만.

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S025](../evidence/SOURCES.ko.md#s025) · [S046](../evidence/SOURCES.ko.md#s046) · [S053](../evidence/SOURCES.ko.md#s053).

### OLD-15-19: CA15 → CA19

**판정:** 유지·통합 검증으로 축소. **관계:** 검증선행.

**선행 결과 → 실제 후행 소비:** R17 관계 수명→R20 복합 결과/복원 검증.

**이유:** 새 상태 자체 복원은 R17의 책임; R20에서 처음 도입 아님. **잘못 해석/분리하면:** R17 전 단계가 후퇴/재시도에서 이미 깨짐.

**기존 충족/재사용:** 현재 entry/attempt/checkpoint 경계. **남은 조건:** live battle whole-process persistence는 비범위.

근거: [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044).

### OLD-17-19: CA17 → CA19

**판정:** 유지·통합 검증으로 축소. **관계:** 검증선행.

**선행 결과 → 실제 후행 소비:** R19 정상 획득/평가→R20 cross-domain commit.

**이유:** 등록/보상과 상태 결과의 통합 확인. **잘못 해석/분리하면:** 현재 profile/receipt/save가 다른 정의/선택을 참조.

**기존 충족/재사용:** 기존 prepare/commit validation. **남은 조건:** 각 R의 새 save는 이미 단독 유효해야 함.

근거: [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S037](../evidence/SOURCES.ko.md#s037) · [S039](../evidence/SOURCES.ko.md#s039) · [S054](../evidence/SOURCES.ko.md#s054).

### OLD-18-19: CA18 → CA19

**판정:** 유지·통합 검증으로 축소. **관계:** 검증선행.

**선행 결과 → 실제 후행 소비:** R09 직원 대가/forced clock→R20 복원/재접속.

**이유:** 침식의 단독 result commit도 R09에서 끝내야 함. **잘못 해석/분리하면:** 전투 즉시 종료 대가 누락/중복.

**기존 충족/재사용:** attempt roster→result→node 처리. **남은 조건:** R09 policy gate가 미해결이면 해당 전체통합 완료는 불가.

근거: [S038](../evidence/SOURCES.ko.md#s038) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S046](../evidence/SOURCES.ko.md#s046).

### OLD-15-20: CA15 → CA20

**판정:** 유지·대표 인수로 한정. **관계:** 검증선행.

**선행 결과 → 실제 후행 소비:** R17 관계/감염→R21 C06와 whole-COV 대조.

**이유:** 대표 공식 복합 구조가 요구하는 구현증거. **잘못 해석/분리하면:** C06를 단순 spawn/RON 이름으로 대체.

**기존 충족/재사용:** 각 단계 실제 prototype RON. **남은 조건:** 공식 모든 콘텐츠 전량이식은 목표 아님.

근거: [S054](../evidence/SOURCES.ko.md#s054) · [S057](../evidence/SOURCES.ko.md#s057) · [S058](../evidence/SOURCES.ko.md#s058).

### OLD-16-20: CA16 → CA20

**판정:** 유지·대표 인수로 한정. **관계:** 검증선행.

**선행 결과 → 실제 후행 소비:** R08/R18→R21 C03/C07 boss.

**이유:** 보스는 플레이어 파편 예제로 대체 불가. **잘못 해석/분리하면:** task 호출/작업억제/연속HP 검증 누락.

**기존 충족/재사용:** 현재 boss/field/wave 기반. **남은 조건:** 전체 boss-COV 별도 상태표 보존.

근거: [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S058](../evidence/SOURCES.ko.md#s058).

### OLD-19-20: CA19 → CA20

**판정:** 유지. **관계:** 검증선행.

**선행 결과 → 실제 후행 소비:** R20 결과·복원 검증→R21 공식 pack 인수.

**이유:** live source로 실행한 결과까지 연결. **잘못 해석/분리하면:** private fixture만 맞고 정상런/저장 실패.

**기존 충족/재사용:** 각 owner의 조기 RON/저장 tests. **남은 조건:** 최종 통합 이전 prototype은 각 R에 이미 배정.

근거: [S039](../evidence/SOURCES.ko.md#s039) · [S044](../evidence/SOURCES.ko.md#s044) · [S054](../evidence/SOURCES.ko.md#s054) · [S057](../evidence/SOURCES.ko.md#s057).

### OLD-20-21: CA20 → CA21

**판정:** 유지. **관계:** 검증선행.

**선행 결과 → 실제 후행 소비:** R21 대표 공식pack/공개SDK→R22 독립재조합.

**이유:** 지원범위·문서·schema를 동결해 작성자 한계와구분. **잘못 해석/분리하면:** 공식 hardcode를 일반 제작 지원으로 오인.

**기존 충족/재사용:** 기존 acceptance C01–07/T22. **남은 조건:** 별도 작성자 실제참여 필요.

근거: [S054](../evidence/SOURCES.ko.md#s054) · [S057](../evidence/SOURCES.ko.md#s057) · [S058](../evidence/SOURCES.ko.md#s058).

### OLD-21-22: CA21 → CA22

**판정:** 유지하되 초기 안전 의무 분산. **관계:** 검증선행, 진행상권장.

**선행 결과 → 실제 후행 소비:** R22 대표 workload→R23 부하회귀; 안전 예산은 R02/04/10/13/16부터.

**이유:** 최종 현실 부하와 최초 안전성 도입은 다름. **잘못 해석/분리하면:** 예산없는 primitive를 먼저 production지원으로공개.

**기존 충족/재사용:** 기존 bounded history/bossmicrostep/SpawnKernel. **남은 조건:** 최적화가 의미/지원범위를바꾸면 R22관련인수재수행.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S057](../evidence/SOURCES.ko.md#s057).

### OLD-22-23: CA22 → CA23

**판정:** 유지. **관계:** 검증선행.

**선행 결과 → 실제 후행 소비:** R23 현재 revision의 회귀/측정 증거→R24 독립완료감사.

**이유:** 보고서 자체와 independent runtime evidence 구분. **잘못 해석/분리하면:** 미실행/정책대기/낡은receipt를 완료로승격.

**기존 충족/재사용:** 기존 P2G/GEX 판정 경계. **남은 조건:** 이번 연구는 독립감사 미수행.

근거: [S001](../evidence/SOURCES.ko.md#s001) · [S009](../evidence/SOURCES.ko.md#s009) · [S054](../evidence/SOURCES.ko.md#s054) · [S057](../evidence/SOURCES.ko.md#s057).

## 3. 새 단위 관계

각 R의 모든 기능이 아니라 producer_output의 계약만 선행한다. 코드/저장/fixture의 동시 경계는 다음 절에서 따로 다룬다. 기본 순차 순서의 인접 단위가 모두 기능상 선행인 것은 아니다.

### REL-001: R00 → R01 / 검증선행

**산출:** 승인/미정이 분리된 요구·정식 최소 Goal 기준선.

**소비:** 현재 SkillDatabase 검증→ValidatedGameData→sim/build_skill_step_commands와 runtime 검사.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 승인/미정이 분리된 요구·정식 최소 Goal 기준선 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: CA00 요구·COV와 CA01 검토·Freeze·Plan·Goal·Handoff.

남은 정책 gate: PG00.

후행이 약속한 유효 상태: 기존 embedded 검증 로딩과 Damage/Heal 실행에 효과별 주소·지원 정보·진단이 붙는다. 개인별 합성·외부 팩·새 저장은 아직 약속하지 않음.

근거: [S001](../evidence/SOURCES.ko.md#s001) · [S002](../evidence/SOURCES.ko.md#s002) · [S005](../evidence/SOURCES.ko.md#s005) · [S006](../evidence/SOURCES.ko.md#s006) · [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S024](../evidence/SOURCES.ko.md#s024) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S070](../evidence/SOURCES.ko.md#s070) · [S078](../evidence/SOURCES.ko.md#s078) · [S079](../evidence/SOURCES.ko.md#s079) · [S088](../evidence/SOURCES.ko.md#s088) · [S089](../evidence/SOURCES.ko.md#s089) · [S091](../evidence/SOURCES.ko.md#s091) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093) · [S094](../evidence/SOURCES.ko.md#s094) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-002: R01 → R02 / 기능상필수

**산출:** 검증된 최신 최소 schema·effect 주소·descriptor·SourceMap.

**소비:** live_ron source 확보→data/mod publication→CoreApplication/new-run briefing→checkpoint restore.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [live_ron source 확보→data/mod publication→CoreApplication/new-run briefing→checkpoint restore]에서 필요한 산출물 [검증된 최신 최소 schema·effect 주소·descriptor·SourceMap]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 외부 A로 시작한 기존 런은 A를 유지하고 새 런은 검증된 B를 선택. 실패 B는 미게시·기존 session/profile 무변경. 새 format만 읽는 전환은 이 단위 범위에서 소유.

기존 충족/재사용: 기존 SkillId·계산·Instant/Projectile/Area kernel·ValidatedGameData 게시·현재 P2G 성과.

남은 정책 gate: PG07.

후행이 약속한 유효 상태: 외부 A로 시작한 기존 런은 A를 유지하고 새 런은 검증된 B를 선택. 실패 B는 미게시·기존 session/profile 무변경. 새 format만 읽는 전환은 이 단위 범위에서 소유.

근거: [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S015](../evidence/SOURCES.ko.md#s015) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S024](../evidence/SOURCES.ko.md#s024) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S068](../evidence/SOURCES.ko.md#s068) · [S069](../evidence/SOURCES.ko.md#s069) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S079](../evidence/SOURCES.ko.md#s079) · [S085](../evidence/SOURCES.ko.md#s085) · [S091](../evidence/SOURCES.ko.md#s091) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093) · [S094](../evidence/SOURCES.ko.md#s094) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-003: R01 → R03 / 기능상필수

**산출:** 검증된 최신 최소 schema·effect 주소·descriptor·SourceMap.

**소비:** Describe/Validate/Explain/Simulate와 Cargo 명시 binary/외부 RON 실행.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [Describe/Validate/Explain/Simulate와 Cargo 명시 binary/외부 RON 실행]에서 필요한 산출물 [검증된 최신 최소 schema·effect 주소·descriptor·SourceMap]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 독립 외부 RON 경로에서 production과 같은 검증·실행을 사용할 수 있다. 현재 지원 어휘만 공개하고 후속 primitive는 각 owner가 즉시 등록.

기존 충족/재사용: 기존 SkillId·계산·Instant/Projectile/Area kernel·ValidatedGameData 게시·현재 P2G 성과.

남은 정책 gate: PG00.

후행이 약속한 유효 상태: 독립 외부 RON 경로에서 production과 같은 검증·실행을 사용할 수 있다. 현재 지원 어휘만 공개하고 후속 primitive는 각 owner가 즉시 등록.

근거: [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S024](../evidence/SOURCES.ko.md#s024) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S068](../evidence/SOURCES.ko.md#s068) · [S069](../evidence/SOURCES.ko.md#s069) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S079](../evidence/SOURCES.ko.md#s079) · [S085](../evidence/SOURCES.ko.md#s085) · [S091](../evidence/SOURCES.ko.md#s091) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093) · [S094](../evidence/SOURCES.ko.md#s094) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-004: R02 → R03 / 기능상필수

**산출:** 외부 source selection·검증된 immutable generation과 fingerprint.

**소비:** Describe/Validate/Explain/Simulate와 Cargo 명시 binary/외부 RON 실행.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [Describe/Validate/Explain/Simulate와 Cargo 명시 binary/외부 RON 실행]에서 필요한 산출물 [외부 source selection·검증된 immutable generation과 fingerprint]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 독립 외부 RON 경로에서 production과 같은 검증·실행을 사용할 수 있다. 현재 지원 어휘만 공개하고 후속 primitive는 각 owner가 즉시 등록.

기존 충족/재사용: 현재 load_live_sources 검증 후 게시·Arc 데이터·generated setup 검증·state transaction의 원리.

남은 정책 gate: PG00.

후행이 약속한 유효 상태: 독립 외부 RON 경로에서 production과 같은 검증·실행을 사용할 수 있다. 현재 지원 어휘만 공개하고 후속 primitive는 각 owner가 즉시 등록.

근거: [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S015](../evidence/SOURCES.ko.md#s015) · [S017](../evidence/SOURCES.ko.md#s017) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S068](../evidence/SOURCES.ko.md#s068) · [S069](../evidence/SOURCES.ko.md#s069) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S085](../evidence/SOURCES.ko.md#s085) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094).

### REL-005: R01 → R04 / 기능상필수

**산출:** 검증된 최신 최소 schema·effect 주소·descriptor·SourceMap.

**소비:** damage_runtime/commands/death_runtime/triggers 및 skill_runtime의 결과·수명 포트.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [damage_runtime/commands/death_runtime/triggers 및 skill_runtime의 결과·수명 포트]에서 필요한 산출물 [검증된 최신 최소 schema·effect 주소·descriptor·SourceMap]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 기존 스킬/전달체를 그대로 쓰면서 Accepted/Released/FlowFinished/EffectsSettled와 실제 HP/회복/처치·비용을 관찰한다. 보스 adapter나 새 Flow 전체는 아직 필요 없음.

기존 충족/재사용: 기존 SkillId·계산·Instant/Projectile/Area kernel·ValidatedGameData 게시·현재 P2G 성과.

남은 정책 gate: PG02, PG03, PG04.

후행이 약속한 유효 상태: 기존 스킬/전달체를 그대로 쓰면서 Accepted/Released/FlowFinished/EffectsSettled와 실제 HP/회복/처치·비용을 관찰한다. 보스 adapter나 새 Flow 전체는 아직 필요 없음.

근거: [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S064](../evidence/SOURCES.ko.md#s064) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S079](../evidence/SOURCES.ko.md#s079) · [S082](../evidence/SOURCES.ko.md#s082) · [S091](../evidence/SOURCES.ko.md#s091) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093) · [S094](../evidence/SOURCES.ko.md#s094) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-006: R04 → R05 / 기능상필수

**산출:** typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트.

**소비:** resource request owner/cost receipt→ResourceStore.

자원 transaction에 필요한 것은 CostReceipt·source owner identity다. 모든 ActionHandle waiter 구현을 전제로 하지 않는다.

무시/오해 시: 후행 소비 지점 [resource request owner/cost receipt→ResourceStore]에서 필요한 산출물 [typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 두 소유 사본의 자원이 분리되고 배치 stack만 재배치에 초기화된다. attempt 사용 횟수와 Run 자원은 지정된 수명을 따른다.

기존 충족/재사용: DamageSourceSnapshot·cast_seq·pending start_seq·step progress·death/wave 흐름.

남은 정책 gate: PG05.

후행이 약속한 유효 상태: 두 소유 사본의 자원이 분리되고 배치 stack만 재배치에 초기화된다. attempt 사용 횟수와 Run 자원은 지정된 수명을 따른다.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-007: R01 → R06 / 기능상필수

**산출:** 검증된 최신 최소 schema·effect 주소·descriptor·SourceMap.

**소비:** stat_pipeline/typed evaluator의 live/snapshot/resource/result 입력.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [stat_pipeline/typed evaluator의 live/snapshot/resource/result 입력]에서 필요한 산출물 [검증된 최신 최소 schema·effect 주소·descriptor·SourceMap]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 상수/stat/HP/시간/resource/result 식과 유한 count/sum/max를 실행한다. 무기 성장의 각 입력은 한 번만 합산된다. 파편 구형 rank 제거는 R07까지 기존 동작을 보존.

기존 충족/재사용: 기존 SkillId·계산·Instant/Projectile/Area kernel·ValidatedGameData 게시·현재 P2G 성과.

남은 정책 gate: PG01, PG03, PG05, PG08.

후행이 약속한 유효 상태: 상수/stat/HP/시간/resource/result 식과 유한 count/sum/max를 실행한다. 무기 성장의 각 입력은 한 번만 합산된다. 파편 구형 rank 제거는 R07까지 기존 동작을 보존.

근거: [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S024](../evidence/SOURCES.ko.md#s024) · [S028](../evidence/SOURCES.ko.md#s028) · [S030](../evidence/SOURCES.ko.md#s030) · [S034](../evidence/SOURCES.ko.md#s034) · [S043](../evidence/SOURCES.ko.md#s043) · [S061](../evidence/SOURCES.ko.md#s061) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S070](../evidence/SOURCES.ko.md#s070) · [S079](../evidence/SOURCES.ko.md#s079) · [S091](../evidence/SOURCES.ko.md#s091) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093) · [S094](../evidence/SOURCES.ko.md#s094) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-008: R04 → R06 / 기능상필수

**산출:** typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트.

**소비:** stat_pipeline/typed evaluator의 live/snapshot/resource/result 입력.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [stat_pipeline/typed evaluator의 live/snapshot/resource/result 입력]에서 필요한 산출물 [typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 상수/stat/HP/시간/resource/result 식과 유한 count/sum/max를 실행한다. 무기 성장의 각 입력은 한 번만 합산된다. 파편 구형 rank 제거는 R07까지 기존 동작을 보존.

기존 충족/재사용: DamageSourceSnapshot·cast_seq·pending start_seq·step progress·death/wave 흐름.

남은 정책 gate: PG01, PG03, PG05, PG08.

후행이 약속한 유효 상태: 상수/stat/HP/시간/resource/result 식과 유한 count/sum/max를 실행한다. 무기 성장의 각 입력은 한 번만 합산된다. 파편 구형 rank 제거는 R07까지 기존 동작을 보존.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S030](../evidence/SOURCES.ko.md#s030) · [S034](../evidence/SOURCES.ko.md#s034) · [S043](../evidence/SOURCES.ko.md#s043) · [S061](../evidence/SOURCES.ko.md#s061) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082) · [S091](../evidence/SOURCES.ko.md#s091) · [S095](../evidence/SOURCES.ko.md#s095).

### REL-009: R05 → R06 / 기능상필수

**산출:** owner×lifetime resource/lease·원자적 소비/복원 포트.

**소비:** stat_pipeline/typed evaluator의 live/snapshot/resource/result 입력.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [stat_pipeline/typed evaluator의 live/snapshot/resource/result 입력]에서 필요한 산출물 [owner×lifetime resource/lease·원자적 소비/복원 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 상수/stat/HP/시간/resource/result 식과 유한 count/sum/max를 실행한다. 무기 성장의 각 입력은 한 번만 합산된다. 파편 구형 rank 제거는 R07까지 기존 동작을 보존.

기존 충족/재사용: 기존 battle attempt/entry/checkpoint transaction·장비 독립 소유.

남은 정책 gate: PG01, PG03, PG05, PG08.

후행이 약속한 유효 상태: 상수/stat/HP/시간/resource/result 식과 유한 count/sum/max를 실행한다. 무기 성장의 각 입력은 한 번만 합산된다. 파편 구형 rank 제거는 R07까지 기존 동작을 보존.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S030](../evidence/SOURCES.ko.md#s030) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S043](../evidence/SOURCES.ko.md#s043) · [S044](../evidence/SOURCES.ko.md#s044) · [S061](../evidence/SOURCES.ko.md#s061) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S082](../evidence/SOURCES.ko.md#s082) · [S091](../evidence/SOURCES.ko.md#s091) · [S095](../evidence/SOURCES.ko.md#s095).

### REL-010: R01 → R07 / 기능상필수

**산출:** 검증된 최신 최소 schema·effect 주소·descriptor·SourceMap.

**소비:** SkillFragmentInventory/Loadout→Maintenance/grant→player_spawns/profile→save/표시.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [SkillFragmentInventory/Loadout→Maintenance/grant→player_spawns/profile→save/표시]에서 필요한 산출물 [검증된 최신 최소 schema·effect 주소·descriptor·SourceMap]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 같은 정의 A만 개화·형태 선택, B 미개화; 이전 후에도 A identity 유지. 현재 지원 기능의 기본/개화 form이 실제 전투에 적용되고 저장된다.

기존 충족/재사용: 기존 SkillId·계산·Instant/Projectile/Area kernel·ValidatedGameData 게시·현재 P2G 성과.

남은 정책 gate: PG05, PG06.

후행이 약속한 유효 상태: 같은 정의 A만 개화·형태 선택, B 미개화; 이전 후에도 A identity 유지. 현재 지원 기능의 기본/개화 form이 실제 전투에 적용되고 저장된다.

근거: [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S024](../evidence/SOURCES.ko.md#s024) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S064](../evidence/SOURCES.ko.md#s064) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S070](../evidence/SOURCES.ko.md#s070) · [S075](../evidence/SOURCES.ko.md#s075) · [S079](../evidence/SOURCES.ko.md#s079) · [S091](../evidence/SOURCES.ko.md#s091) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093) · [S094](../evidence/SOURCES.ko.md#s094) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-011: R05 → R07 / 기능상필수

**산출:** owner×lifetime resource/lease·원자적 소비/복원 포트.

**소비:** SkillFragmentInventory/Loadout→Maintenance/grant→player_spawns/profile→save/표시.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [SkillFragmentInventory/Loadout→Maintenance/grant→player_spawns/profile→save/표시]에서 필요한 산출물 [owner×lifetime resource/lease·원자적 소비/복원 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 같은 정의 A만 개화·형태 선택, B 미개화; 이전 후에도 A identity 유지. 현재 지원 기능의 기본/개화 form이 실제 전투에 적용되고 저장된다.

기존 충족/재사용: 기존 battle attempt/entry/checkpoint transaction·장비 독립 소유.

남은 정책 gate: PG05, PG06.

후행이 약속한 유효 상태: 같은 정의 A만 개화·형태 선택, B 미개화; 이전 후에도 A identity 유지. 현재 지원 기능의 기본/개화 form이 실제 전투에 적용되고 저장된다.

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S044](../evidence/SOURCES.ko.md#s044) · [S064](../evidence/SOURCES.ko.md#s064) · [S075](../evidence/SOURCES.ko.md#s075) · [S082](../evidence/SOURCES.ko.md#s082) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-012: R01 → R08 / 기능상필수

**산출:** 검증된 최신 최소 schema·effect 주소·descriptor·SourceMap.

**소비:** BossTaskData/compiled task→공통 내부 action admission→state activation result.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [BossTaskData/compiled task→공통 내부 action admission→state activation result]에서 필요한 산출물 [검증된 최신 최소 schema·effect 주소·descriptor·SourceMap]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 기존 보스 graph가 현재 지원 스킬을 명시적으로 요청하고 release/settled를 선택해 기다린다. 설치물·공유 shield·전체 Flow 없이도 정상·실패 route가 유효.

기존 충족/재사용: 기존 SkillId·계산·Instant/Projectile/Area kernel·ValidatedGameData 게시·현재 P2G 성과.

남은 정책 gate: PG03.

후행이 약속한 유효 상태: 기존 보스 graph가 현재 지원 스킬을 명시적으로 요청하고 release/settled를 선택해 기다린다. 설치물·공유 shield·전체 Flow 없이도 정상·실패 route가 유효.

근거: [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S024](../evidence/SOURCES.ko.md#s024) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S070](../evidence/SOURCES.ko.md#s070) · [S079](../evidence/SOURCES.ko.md#s079) · [S084](../evidence/SOURCES.ko.md#s084) · [S091](../evidence/SOURCES.ko.md#s091) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093) · [S094](../evidence/SOURCES.ko.md#s094) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-013: R04 → R08 / 기능상필수

**산출:** typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트.

**소비:** BossTask의 Accepted/Released/EffectsSettled waiter.

기존 Instant/Projectile/Area의 ActionHandle과 실패/취소 포트만 필요하다. full Flow·shield·설치물을 기다리지 않는다.

무시/오해 시: 후행 소비 지점 [BossTask의 Accepted/Released/EffectsSettled waiter]에서 필요한 산출물 [typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 기존 보스 graph가 현재 지원 스킬을 명시적으로 요청하고 release/settled를 선택해 기다린다. 설치물·공유 shield·전체 Flow 없이도 정상·실패 route가 유효.

기존 충족/재사용: DamageSourceSnapshot·cast_seq·pending start_seq·step progress·death/wave 흐름.

남은 정책 gate: PG03.

후행이 약속한 유효 상태: 기존 보스 graph가 현재 지원 스킬을 명시적으로 요청하고 release/settled를 선택해 기다린다. 설치물·공유 shield·전체 Flow 없이도 정상·실패 route가 유효.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082) · [S084](../evidence/SOURCES.ko.md#s084) · [S094](../evidence/SOURCES.ko.md#s094).

### REL-014: R04 → R09 / 기능상필수

**산출:** typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트.

**소비:** manual/ordinary cast 접수와 실제 실행 연결→withdraw/kill→attempt roster→node/save/clock.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [manual/ordinary cast 접수와 실제 실행 연결→withdraw/kill→attempt roster→node/save/clock]에서 필요한 산출물 [typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 시전 자격과 실제 실행이 연결되고 현재 적만 처치하며 미출현 wave 유지. 대가가 즉시 종료 전투에서도 한 번 반영되고 강제 정지 중 읽기만 가능.

기존 충족/재사용: DamageSourceSnapshot·cast_seq·pending start_seq·step progress·death/wave 흐름.

남은 정책 gate: PG03, PG04, PG05, PG06, PG09.

후행이 약속한 유효 상태: 시전 자격과 실제 실행이 연결되고 현재 적만 처치하며 미출현 wave 유지. 대가가 즉시 종료 전투에서도 한 번 반영되고 강제 정지 중 읽기만 가능.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S046](../evidence/SOURCES.ko.md#s046) · [S059](../evidence/SOURCES.ko.md#s059) · [S060](../evidence/SOURCES.ko.md#s060) · [S061](../evidence/SOURCES.ko.md#s061) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S075](../evidence/SOURCES.ko.md#s075) · [S077](../evidence/SOURCES.ko.md#s077) · [S082](../evidence/SOURCES.ko.md#s082) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-015: R05 → R09 / 기능상필수

**산출:** owner×lifetime resource/lease·원자적 소비/복원 포트.

**소비:** manual/ordinary cast 접수와 실제 실행 연결→withdraw/kill→attempt roster→node/save/clock.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [manual/ordinary cast 접수와 실제 실행 연결→withdraw/kill→attempt roster→node/save/clock]에서 필요한 산출물 [owner×lifetime resource/lease·원자적 소비/복원 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 시전 자격과 실제 실행이 연결되고 현재 적만 처치하며 미출현 wave 유지. 대가가 즉시 종료 전투에서도 한 번 반영되고 강제 정지 중 읽기만 가능.

기존 충족/재사용: 기존 battle attempt/entry/checkpoint transaction·장비 독립 소유.

남은 정책 gate: PG03, PG04, PG05, PG06, PG09.

후행이 약속한 유효 상태: 시전 자격과 실제 실행이 연결되고 현재 적만 처치하며 미출현 wave 유지. 대가가 즉시 종료 전투에서도 한 번 반영되고 강제 정지 중 읽기만 가능.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S022](../evidence/SOURCES.ko.md#s022) · [S025](../evidence/SOURCES.ko.md#s025) · [S027](../evidence/SOURCES.ko.md#s027) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S046](../evidence/SOURCES.ko.md#s046) · [S059](../evidence/SOURCES.ko.md#s059) · [S060](../evidence/SOURCES.ko.md#s060) · [S061](../evidence/SOURCES.ko.md#s061) · [S075](../evidence/SOURCES.ko.md#s075) · [S077](../evidence/SOURCES.ko.md#s077) · [S082](../evidence/SOURCES.ko.md#s082) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-016: R07 → R09 / 기능상필수

**산출:** 실제 파편 사본/형태·장착 binding·정상 지급/저장.

**소비:** SkillFragmentLoadout→실제 시전 침식 적격 판정.

실제로 선택된 파편의 definition/origin/form binding이 최종 보스 파편 제외와 cast 대체의 입력이다.

무시/오해 시: 후행 소비 지점 [SkillFragmentLoadout→실제 시전 침식 적격 판정]에서 필요한 산출물 [실제 파편 사본/형태·장착 binding·정상 지급/저장]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 시전 자격과 실제 실행이 연결되고 현재 적만 처치하며 미출현 wave 유지. 대가가 즉시 종료 전투에서도 한 번 반영되고 강제 정지 중 읽기만 가능.

기존 충족/재사용: EquipmentLoadoutTransition의 clone/검증/commit 패턴, EconomyGrantState 및 UUID ownership.

남은 정책 gate: PG03, PG04, PG05, PG06, PG09.

후행이 약속한 유효 상태: 시전 자격과 실제 실행이 연결되고 현재 적만 처치하며 미출현 wave 유지. 대가가 즉시 종료 전투에서도 한 번 반영되고 강제 정지 중 읽기만 가능.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S022](../evidence/SOURCES.ko.md#s022) · [S025](../evidence/SOURCES.ko.md#s025) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S044](../evidence/SOURCES.ko.md#s044) · [S046](../evidence/SOURCES.ko.md#s046) · [S059](../evidence/SOURCES.ko.md#s059) · [S060](../evidence/SOURCES.ko.md#s060) · [S061](../evidence/SOURCES.ko.md#s061) · [S064](../evidence/SOURCES.ko.md#s064) · [S075](../evidence/SOURCES.ko.md#s075) · [S077](../evidence/SOURCES.ko.md#s077) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-017: R01 → R10 / 기능상필수

**산출:** 검증된 최신 최소 schema·effect 주소·descriptor·SourceMap.

**소비:** sim/skill_runtime의 continuation/result slot·반복·전달 분리.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [sim/skill_runtime의 continuation/result slot·반복·전달 분리]에서 필요한 산출물 [검증된 최신 최소 schema·effect 주소·descriptor·SourceMap]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: AoE 피해 뒤 자기 회복 한 번, 결과 조건·유한 기회 반복·병행 join이 실제 실행된다. 기존 최소 주소와 보스 adapter가 같은 service를 소비.

기존 충족/재사용: 기존 SkillId·계산·Instant/Projectile/Area kernel·ValidatedGameData 게시·현재 P2G 성과.

남은 정책 gate: PG03, PG04.

후행이 약속한 유효 상태: AoE 피해 뒤 자기 회복 한 번, 결과 조건·유한 기회 반복·병행 join이 실제 실행된다. 기존 최소 주소와 보스 adapter가 같은 service를 소비.

근거: [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S024](../evidence/SOURCES.ko.md#s024) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S070](../evidence/SOURCES.ko.md#s070) · [S079](../evidence/SOURCES.ko.md#s079) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S091](../evidence/SOURCES.ko.md#s091) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093) · [S094](../evidence/SOURCES.ko.md#s094) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-018: R04 → R10 / 기능상필수

**산출:** typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트.

**소비:** sim/skill_runtime의 continuation/result slot·반복·전달 분리.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [sim/skill_runtime의 continuation/result slot·반복·전달 분리]에서 필요한 산출물 [typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: AoE 피해 뒤 자기 회복 한 번, 결과 조건·유한 기회 반복·병행 join이 실제 실행된다. 기존 최소 주소와 보스 adapter가 같은 service를 소비.

기존 충족/재사용: DamageSourceSnapshot·cast_seq·pending start_seq·step progress·death/wave 흐름.

남은 정책 gate: PG03, PG04.

후행이 약속한 유효 상태: AoE 피해 뒤 자기 회복 한 번, 결과 조건·유한 기회 반복·병행 join이 실제 실행된다. 기존 최소 주소와 보스 adapter가 같은 service를 소비.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S079](../evidence/SOURCES.ko.md#s079) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-019: R05 → R10 / 기능상필수

**산출:** owner×lifetime resource/lease·원자적 소비/복원 포트.

**소비:** sim/skill_runtime의 continuation/result slot·반복·전달 분리.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [sim/skill_runtime의 continuation/result slot·반복·전달 분리]에서 필요한 산출물 [owner×lifetime resource/lease·원자적 소비/복원 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: AoE 피해 뒤 자기 회복 한 번, 결과 조건·유한 기회 반복·병행 join이 실제 실행된다. 기존 최소 주소와 보스 adapter가 같은 service를 소비.

기존 충족/재사용: 기존 battle attempt/entry/checkpoint transaction·장비 독립 소유.

남은 정책 gate: PG03, PG04.

후행이 약속한 유효 상태: AoE 피해 뒤 자기 회복 한 번, 결과 조건·유한 기회 반복·병행 join이 실제 실행된다. 기존 최소 주소와 보스 adapter가 같은 service를 소비.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S070](../evidence/SOURCES.ko.md#s070) · [S079](../evidence/SOURCES.ko.md#s079) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S091](../evidence/SOURCES.ko.md#s091) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-020: R06 → R10 / 기능상필수

**산출:** typed value/result/resource 식·집계·스탯 입력.

**소비:** sim/skill_runtime의 continuation/result slot·반복·전달 분리.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [sim/skill_runtime의 continuation/result slot·반복·전달 분리]에서 필요한 산출물 [typed value/result/resource 식·집계·스탯 입력]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: AoE 피해 뒤 자기 회복 한 번, 결과 조건·유한 기회 반복·병행 join이 실제 실행된다. 기존 최소 주소와 보스 adapter가 같은 service를 소비.

기존 충족/재사용: stat_pipeline·scale_attack_interval_ms·기존 enhancement 소유/저장.

남은 정책 gate: PG03, PG04.

후행이 약속한 유효 상태: AoE 피해 뒤 자기 회복 한 번, 결과 조건·유한 기회 반복·병행 join이 실제 실행된다. 기존 최소 주소와 보스 adapter가 같은 service를 소비.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S028](../evidence/SOURCES.ko.md#s028) · [S030](../evidence/SOURCES.ko.md#s030) · [S034](../evidence/SOURCES.ko.md#s034) · [S043](../evidence/SOURCES.ko.md#s043) · [S061](../evidence/SOURCES.ko.md#s061) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S079](../evidence/SOURCES.ko.md#s079) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S091](../evidence/SOURCES.ko.md#s091) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-021: R05 → R11 / 기능상필수

**산출:** owner×lifetime resource/lease·원자적 소비/복원 포트.

**소비:** buff/stat/basic profile와 activation/membership/lease 해제.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [buff/stat/basic profile와 activation/membership/lease 해제]에서 필요한 산출물 [owner×lifetime resource/lease·원자적 소비/복원 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 오라 진입/이탈·집중 종료가 자기 modifier만 제거하고, Hello natural expiry는 복귀만 하며 재입력은 Goodbye 후 복귀.

기존 충족/재사용: 기존 battle attempt/entry/checkpoint transaction·장비 독립 소유.

남은 정책 gate: PG01, PG03, PG04, PG05.

후행이 약속한 유효 상태: 오라 진입/이탈·집중 종료가 자기 modifier만 제거하고, Hello natural expiry는 복귀만 하며 재입력은 Goodbye 후 복귀.

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S029](../evidence/SOURCES.ko.md#s029) · [S030](../evidence/SOURCES.ko.md#s030) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S043](../evidence/SOURCES.ko.md#s043) · [S044](../evidence/SOURCES.ko.md#s044) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-022: R06 → R11 / 기능상필수

**산출:** typed value/result/resource 식·집계·스탯 입력.

**소비:** buff/stat/basic profile와 activation/membership/lease 해제.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [buff/stat/basic profile와 activation/membership/lease 해제]에서 필요한 산출물 [typed value/result/resource 식·집계·스탯 입력]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 오라 진입/이탈·집중 종료가 자기 modifier만 제거하고, Hello natural expiry는 복귀만 하며 재입력은 Goodbye 후 복귀.

기존 충족/재사용: stat_pipeline·scale_attack_interval_ms·기존 enhancement 소유/저장.

남은 정책 gate: PG01, PG03, PG04, PG05.

후행이 약속한 유효 상태: 오라 진입/이탈·집중 종료가 자기 modifier만 제거하고, Hello natural expiry는 복귀만 하며 재입력은 Goodbye 후 복귀.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S029](../evidence/SOURCES.ko.md#s029) · [S030](../evidence/SOURCES.ko.md#s030) · [S034](../evidence/SOURCES.ko.md#s034) · [S043](../evidence/SOURCES.ko.md#s043) · [S061](../evidence/SOURCES.ko.md#s061) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091) · [S095](../evidence/SOURCES.ko.md#s095).

### REL-023: R10 → R11 / 기능상필수

**산출:** 실행 가능한 typed Flow/target/carrier와 immutable plan 참조.

**소비:** buff/stat/basic profile와 activation/membership/lease 해제.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [buff/stat/basic profile와 activation/membership/lease 해제]에서 필요한 산출물 [실행 가능한 typed Flow/target/carrier와 immutable plan 참조]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 오라 진입/이탈·집중 종료가 자기 modifier만 제거하고, Hello natural expiry는 복귀만 하며 재입력은 Goodbye 후 복귀.

기존 충족/재사용: 현재 current cast/delivery kernels·R04 handle·R06 식.

남은 정책 gate: PG01, PG03, PG04, PG05.

후행이 약속한 유효 상태: 오라 진입/이탈·집중 종료가 자기 modifier만 제거하고, Hello natural expiry는 복귀만 하며 재입력은 Goodbye 후 복귀.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S029](../evidence/SOURCES.ko.md#s029) · [S030](../evidence/SOURCES.ko.md#s030) · [S043](../evidence/SOURCES.ko.md#s043) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S076](../evidence/SOURCES.ko.md#s076) · [S079](../evidence/SOURCES.ko.md#s079) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-024: R04 → R12 / 기능상필수

**산출:** typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트.

**소비:** incoming damage→pool/HP/survival atomic commit 및 수혜자 membership.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [incoming damage→pool/HP/survival atomic commit 및 수혜자 membership]에서 필요한 산출물 [typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 여러 대상은 하나의 잔량만 공유하며 피해·회복·충전·HP 비용의 사건 의미가 분리됨.

기존 충족/재사용: DamageSourceSnapshot·cast_seq·pending start_seq·step progress·death/wave 흐름.

남은 정책 gate: PG03, PG04, PG06.

후행이 약속한 유효 상태: 여러 대상은 하나의 잔량만 공유하며 피해·회복·충전·HP 비용의 사건 의미가 분리됨.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S029](../evidence/SOURCES.ko.md#s029) · [S043](../evidence/SOURCES.ko.md#s043) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087).

### REL-025: R05 → R12 / 기능상필수

**산출:** owner×lifetime resource/lease·원자적 소비/복원 포트.

**소비:** incoming damage→pool/HP/survival atomic commit 및 수혜자 membership.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [incoming damage→pool/HP/survival atomic commit 및 수혜자 membership]에서 필요한 산출물 [owner×lifetime resource/lease·원자적 소비/복원 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 여러 대상은 하나의 잔량만 공유하며 피해·회복·충전·HP 비용의 사건 의미가 분리됨.

기존 충족/재사용: 기존 battle attempt/entry/checkpoint transaction·장비 독립 소유.

남은 정책 gate: PG03, PG04, PG06.

후행이 약속한 유효 상태: 여러 대상은 하나의 잔량만 공유하며 피해·회복·충전·HP 비용의 사건 의미가 분리됨.

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S029](../evidence/SOURCES.ko.md#s029) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S043](../evidence/SOURCES.ko.md#s043) · [S044](../evidence/SOURCES.ko.md#s044) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-026: R06 → R12 / 기능상필수

**산출:** typed value/result/resource 식·집계·스탯 입력.

**소비:** incoming damage→pool/HP/survival atomic commit 및 수혜자 membership.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [incoming damage→pool/HP/survival atomic commit 및 수혜자 membership]에서 필요한 산출물 [typed value/result/resource 식·집계·스탯 입력]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 여러 대상은 하나의 잔량만 공유하며 피해·회복·충전·HP 비용의 사건 의미가 분리됨.

기존 충족/재사용: stat_pipeline·scale_attack_interval_ms·기존 enhancement 소유/저장.

남은 정책 gate: PG03, PG04, PG06.

후행이 약속한 유효 상태: 여러 대상은 하나의 잔량만 공유하며 피해·회복·충전·HP 비용의 사건 의미가 분리됨.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S028](../evidence/SOURCES.ko.md#s028) · [S029](../evidence/SOURCES.ko.md#s029) · [S030](../evidence/SOURCES.ko.md#s030) · [S034](../evidence/SOURCES.ko.md#s034) · [S043](../evidence/SOURCES.ko.md#s043) · [S061](../evidence/SOURCES.ko.md#s061) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091) · [S095](../evidence/SOURCES.ko.md#s095).

### REL-027: R11 → R12 / 기능상필수

**산출:** source별 reversible profile/lease·membership·종료 포트.

**소비:** pool membership와 source activation lease.

공유 보호막의 수혜자 add/remove와 source 해제 포트만 필요하다. Hello·재입력 전체는 선행이 아니다.

무시/오해 시: 후행 소비 지점 [pool membership와 source activation lease]에서 필요한 산출물 [source별 reversible profile/lease·membership·종료 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 여러 대상은 하나의 잔량만 공유하며 피해·회복·충전·HP 비용의 사건 의미가 분리됨.

기존 충족/재사용: 기본 공격 start/release/recovery·기존 stat pipeline·R05 lease.

남은 정책 gate: PG03, PG04, PG06.

후행이 약속한 유효 상태: 여러 대상은 하나의 잔량만 공유하며 피해·회복·충전·HP 비용의 사건 의미가 분리됨.

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S029](../evidence/SOURCES.ko.md#s029) · [S030](../evidence/SOURCES.ko.md#s030) · [S043](../evidence/SOURCES.ko.md#s043) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087).

### REL-028: R04 → R13 / 기능상필수

**산출:** typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트.

**소비:** SpawnKernel adapter→Trap/Actor/FieldObject 생성·HP비용·진화·source별 수명.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [SpawnKernel adapter→Trap/Actor/FieldObject 생성·HP비용·진화·source별 수명]에서 필요한 산출물 [typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 아군/적 actor와 정지 함정·벽이 각 기능에 맞게 생성된다. 작은→큰 진화는 논리적 형태 전환이다. node ID 유지/교체는 기술 설계에서 선택하되 source 권한과 stale link/event 무효화를 함께 보장한다..

기존 충족/재사용: DamageSourceSnapshot·cast_seq·pending start_seq·step progress·death/wave 흐름.

남은 정책 gate: PG04, PG05, PG06.

후행이 약속한 유효 상태: 아군/적 actor와 정지 함정·벽이 각 기능에 맞게 생성된다. 작은→큰 진화는 논리적 형태 전환이다. node ID 유지/교체는 기술 설계에서 선택하되 source 권한과 stale link/event 무효화를 함께 보장한다..

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S047](../evidence/SOURCES.ko.md#s047) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087).

### REL-029: R05 → R13 / 기능상필수

**산출:** owner×lifetime resource/lease·원자적 소비/복원 포트.

**소비:** SpawnKernel adapter→Trap/Actor/FieldObject 생성·HP비용·진화·source별 수명.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [SpawnKernel adapter→Trap/Actor/FieldObject 생성·HP비용·진화·source별 수명]에서 필요한 산출물 [owner×lifetime resource/lease·원자적 소비/복원 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 아군/적 actor와 정지 함정·벽이 각 기능에 맞게 생성된다. 작은→큰 진화는 논리적 형태 전환이다. node ID 유지/교체는 기술 설계에서 선택하되 source 권한과 stale link/event 무효화를 함께 보장한다..

기존 충족/재사용: 기존 battle attempt/entry/checkpoint transaction·장비 독립 소유.

남은 정책 gate: PG04, PG05, PG06.

후행이 약속한 유효 상태: 아군/적 actor와 정지 함정·벽이 각 기능에 맞게 생성된다. 작은→큰 진화는 논리적 형태 전환이다. node ID 유지/교체는 기술 설계에서 선택하되 source 권한과 stale link/event 무효화를 함께 보장한다..

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S047](../evidence/SOURCES.ko.md#s047) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-030: R07 → R13 / 기능상필수

**산출:** 실제 파편 사본/형태·장착 binding·정상 지급/저장.

**소비:** Laetitia 원천 사본→small/big 진화 자격.

파편별 trap 소유·진화에 copy binding이 필요하다. 기존 보스 actor에는 필수 아님.

무시/오해 시: 후행 소비 지점 [Laetitia 원천 사본→small/big 진화 자격]에서 필요한 산출물 [실제 파편 사본/형태·장착 binding·정상 지급/저장]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 아군/적 actor와 정지 함정·벽이 각 기능에 맞게 생성된다. 작은→큰 진화는 논리적 형태 전환이다. node ID 유지/교체는 기술 설계에서 선택하되 source 권한과 stale link/event 무효화를 함께 보장한다..

기존 충족/재사용: EquipmentLoadoutTransition의 clone/검증/commit 패턴, EconomyGrantState 및 UUID ownership.

남은 정책 gate: PG04, PG05, PG06.

후행이 약속한 유효 상태: 아군/적 actor와 정지 함정·벽이 각 기능에 맞게 생성된다. 작은→큰 진화는 논리적 형태 전환이다. node ID 유지/교체는 기술 설계에서 선택하되 source 권한과 stale link/event 무효화를 함께 보장한다..

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S047](../evidence/SOURCES.ko.md#s047) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S064](../evidence/SOURCES.ko.md#s064) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-031: R10 → R13 / 기능상필수

**산출:** 실행 가능한 typed Flow/target/carrier와 immutable plan 참조.

**소비:** SpawnKernel adapter→Trap/Actor/FieldObject 생성·HP비용·진화·source별 수명.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [SpawnKernel adapter→Trap/Actor/FieldObject 생성·HP비용·진화·source별 수명]에서 필요한 산출물 [실행 가능한 typed Flow/target/carrier와 immutable plan 참조]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 아군/적 actor와 정지 함정·벽이 각 기능에 맞게 생성된다. 작은→큰 진화는 논리적 형태 전환이다. node ID 유지/교체는 기술 설계에서 선택하되 source 권한과 stale link/event 무효화를 함께 보장한다..

기존 충족/재사용: 현재 current cast/delivery kernels·R04 handle·R06 식.

남은 정책 gate: PG04, PG05, PG06.

후행이 약속한 유효 상태: 아군/적 actor와 정지 함정·벽이 각 기능에 맞게 생성된다. 작은→큰 진화는 논리적 형태 전환이다. node ID 유지/교체는 기술 설계에서 선택하되 source 권한과 stale link/event 무효화를 함께 보장한다..

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S047](../evidence/SOURCES.ko.md#s047) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S070](../evidence/SOURCES.ko.md#s070) · [S076](../evidence/SOURCES.ko.md#s076) · [S079](../evidence/SOURCES.ko.md#s079) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-032: R12 → R13 / 기능상필수

**산출:** 공용 pool/HP비용·방어 중재의 필요한 최소 포트.

**소비:** cost reserve/commit/abort→SpawnKernel 생성 transaction.

R13에 포함한 meatwall의 HP비용/생성 원자성만 선행이다. 비용 없는 actor/trap은 독립이다.

무시/오해 시: 후행 소비 지점 [cost reserve/commit/abort→SpawnKernel 생성 transaction]에서 필요한 산출물 [공용 pool/HP비용·방어 중재의 필요한 최소 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 아군/적 actor와 정지 함정·벽이 각 기능에 맞게 생성된다. 작은→큰 진화는 논리적 형태 전환이다. node ID 유지/교체는 기술 설계에서 선택하되 source 권한과 stale link/event 무효화를 함께 보장한다..

기존 충족/재사용: 기존 damage reduction·control/movement·R05 resource/R11 membership.

남은 정책 gate: PG04, PG05, PG06.

후행이 약속한 유효 상태: 아군/적 actor와 정지 함정·벽이 각 기능에 맞게 생성된다. 작은→큰 진화는 논리적 형태 전환이다. node ID 유지/교체는 기술 설계에서 선택하되 source 권한과 stale link/event 무효화를 함께 보장한다..

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S028](../evidence/SOURCES.ko.md#s028) · [S029](../evidence/SOURCES.ko.md#s029) · [S043](../evidence/SOURCES.ko.md#s043) · [S047](../evidence/SOURCES.ko.md#s047) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087).

### REL-033: R01 → R14 / 기능상필수

**산출:** 검증된 최신 최소 schema·effect 주소·descriptor·SourceMap.

**소비:** loadout Composer→cast/queued step/projectile/area/개인 preview·평가 입력.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [loadout Composer→cast/queued step/projectile/area/개인 preview·평가 입력]에서 필요한 산출물 [검증된 최신 최소 schema·effect 주소·descriptor·SourceMap]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 유물을 정상 획득·장착·저장하고 지원한 수치 변형이 즉시/지연 실행에 실제 반영된다. 요구 효과가 없는 유물은 슬롯 유지·휴면.

기존 충족/재사용: 기존 SkillId·계산·Instant/Projectile/Area kernel·ValidatedGameData 게시·현재 P2G 성과.

남은 정책 gate: PG01, PG05, PG06, PG08.

후행이 약속한 유효 상태: 유물을 정상 획득·장착·저장하고 지원한 수치 변형이 즉시/지연 실행에 실제 반영된다. 요구 효과가 없는 유물은 슬롯 유지·휴면.

근거: [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S024](../evidence/SOURCES.ko.md#s024) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S041](../evidence/SOURCES.ko.md#s041) · [S045](../evidence/SOURCES.ko.md#s045) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S070](../evidence/SOURCES.ko.md#s070) · [S076](../evidence/SOURCES.ko.md#s076) · [S079](../evidence/SOURCES.ko.md#s079) · [S083](../evidence/SOURCES.ko.md#s083) · [S091](../evidence/SOURCES.ko.md#s091) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093) · [S094](../evidence/SOURCES.ko.md#s094) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-034: R05 → R14 / 기능상필수

**산출:** owner×lifetime resource/lease·원자적 소비/복원 포트.

**소비:** loadout Composer→cast/queued step/projectile/area/개인 preview·평가 입력.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [loadout Composer→cast/queued step/projectile/area/개인 preview·평가 입력]에서 필요한 산출물 [owner×lifetime resource/lease·원자적 소비/복원 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 유물을 정상 획득·장착·저장하고 지원한 수치 변형이 즉시/지연 실행에 실제 반영된다. 요구 효과가 없는 유물은 슬롯 유지·휴면.

기존 충족/재사용: 기존 battle attempt/entry/checkpoint transaction·장비 독립 소유.

남은 정책 gate: PG01, PG05, PG06, PG08.

후행이 약속한 유효 상태: 유물을 정상 획득·장착·저장하고 지원한 수치 변형이 즉시/지연 실행에 실제 반영된다. 요구 효과가 없는 유물은 슬롯 유지·휴면.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S041](../evidence/SOURCES.ko.md#s041) · [S044](../evidence/SOURCES.ko.md#s044) · [S045](../evidence/SOURCES.ko.md#s045) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-035: R07 → R14 / 기능상필수

**산출:** 실제 파편 사본/형태·장착 binding·정상 지급/저장.

**소비:** loadout Composer→cast/queued step/projectile/area/개인 preview·평가 입력.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [loadout Composer→cast/queued step/projectile/area/개인 preview·평가 입력]에서 필요한 산출물 [실제 파편 사본/형태·장착 binding·정상 지급/저장]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 유물을 정상 획득·장착·저장하고 지원한 수치 변형이 즉시/지연 실행에 실제 반영된다. 요구 효과가 없는 유물은 슬롯 유지·휴면.

기존 충족/재사용: EquipmentLoadoutTransition의 clone/검증/commit 패턴, EconomyGrantState 및 UUID ownership.

남은 정책 gate: PG01, PG05, PG06, PG08.

후행이 약속한 유효 상태: 유물을 정상 획득·장착·저장하고 지원한 수치 변형이 즉시/지연 실행에 실제 반영된다. 요구 효과가 없는 유물은 슬롯 유지·휴면.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S045](../evidence/SOURCES.ko.md#s045) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S083](../evidence/SOURCES.ko.md#s083) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-036: R10 → R14 / 기능상필수

**산출:** 실행 가능한 typed Flow/target/carrier와 immutable plan 참조.

**소비:** cast/queued step/impact consumer의 effective plan 참조.

선택한 기본안에서 current/structured Flow 모두의 개인별 불변 plan을 소비한다. 관리 aggregate 자체는 독립이다.

무시/오해 시: 후행 소비 지점 [cast/queued step/impact consumer의 effective plan 참조]에서 필요한 산출물 [실행 가능한 typed Flow/target/carrier와 immutable plan 참조]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 유물을 정상 획득·장착·저장하고 지원한 수치 변형이 즉시/지연 실행에 실제 반영된다. 요구 효과가 없는 유물은 슬롯 유지·휴면.

기존 충족/재사용: 현재 current cast/delivery kernels·R04 handle·R06 식.

남은 정책 gate: PG01, PG05, PG06, PG08.

후행이 약속한 유효 상태: 유물을 정상 획득·장착·저장하고 지원한 수치 변형이 즉시/지연 실행에 실제 반영된다. 요구 효과가 없는 유물은 슬롯 유지·휴면.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S041](../evidence/SOURCES.ko.md#s041) · [S045](../evidence/SOURCES.ko.md#s045) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S076](../evidence/SOURCES.ko.md#s076) · [S079](../evidence/SOURCES.ko.md#s079) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S091](../evidence/SOURCES.ko.md#s091) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-037: R11 → R15 / 기능상필수

**산출:** source별 reversible profile/lease·membership·종료 포트.

**소비:** typed selector/closure/recipe→기존 지원 primitive의 변형된 plan 재검증.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [typed selector/closure/recipe→기존 지원 primitive의 변형된 plan 재검증]에서 필요한 산출물 [source별 reversible profile/lease·membership·종료 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 새로 부여한 효과도 다른 유물 선택 대상이 되며 같은 장착 집합은 같은 plan. 포큐버스 단일2S·태아 상태별 전환이 각 계약대로 작동.

기존 충족/재사용: 기본 공격 start/release/recovery·기존 stat pipeline·R05 lease.

남은 정책 gate: PG01, PG02, PG03, PG06.

후행이 약속한 유효 상태: 새로 부여한 효과도 다른 유물 선택 대상이 되며 같은 장착 집합은 같은 plan. 포큐버스 단일2S·태아 상태별 전환이 각 계약대로 작동.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S029](../evidence/SOURCES.ko.md#s029) · [S030](../evidence/SOURCES.ko.md#s030) · [S043](../evidence/SOURCES.ko.md#s043) · [S045](../evidence/SOURCES.ko.md#s045) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S087](../evidence/SOURCES.ko.md#s087).

### REL-038: R12 → R15 / 기능상필수

**산출:** 공용 pool/HP비용·방어 중재의 필요한 최소 포트.

**소비:** typed selector/closure/recipe→기존 지원 primitive의 변형된 plan 재검증.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [typed selector/closure/recipe→기존 지원 primitive의 변형된 plan 재검증]에서 필요한 산출물 [공용 pool/HP비용·방어 중재의 필요한 최소 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 새로 부여한 효과도 다른 유물 선택 대상이 되며 같은 장착 집합은 같은 plan. 포큐버스 단일2S·태아 상태별 전환이 각 계약대로 작동.

기존 충족/재사용: 기존 damage reduction·control/movement·R05 resource/R11 membership.

남은 정책 gate: PG01, PG02, PG03, PG06.

후행이 약속한 유효 상태: 새로 부여한 효과도 다른 유물 선택 대상이 되며 같은 장착 집합은 같은 plan. 포큐버스 단일2S·태아 상태별 전환이 각 계약대로 작동.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S028](../evidence/SOURCES.ko.md#s028) · [S029](../evidence/SOURCES.ko.md#s029) · [S043](../evidence/SOURCES.ko.md#s043) · [S045](../evidence/SOURCES.ko.md#s045) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S076](../evidence/SOURCES.ko.md#s076) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S087](../evidence/SOURCES.ko.md#s087).

### REL-039: R14 → R15 / 기능상필수

**산출:** 유물 사본 binding·실제 numeric effective plan/설명/저장.

**소비:** typed selector/closure/recipe→기존 지원 primitive의 변형된 plan 재검증.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [typed selector/closure/recipe→기존 지원 primitive의 변형된 plan 재검증]에서 필요한 산출물 [유물 사본 binding·실제 numeric effective plan/설명/저장]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 새로 부여한 효과도 다른 유물 선택 대상이 되며 같은 장착 집합은 같은 plan. 포큐버스 단일2S·태아 상태별 전환이 각 계약대로 작동.

기존 충족/재사용: 장비 loadout transaction·EconomyGrantState·save/entry·R01주소/R10plan.

남은 정책 gate: PG01, PG02, PG03, PG06.

후행이 약속한 유효 상태: 새로 부여한 효과도 다른 유물 선택 대상이 되며 같은 장착 집합은 같은 plan. 포큐버스 단일2S·태아 상태별 전환이 각 계약대로 작동.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S029](../evidence/SOURCES.ko.md#s029) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S041](../evidence/SOURCES.ko.md#s041) · [S045](../evidence/SOURCES.ko.md#s045) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S081](../evidence/SOURCES.ko.md#s081) · [S083](../evidence/SOURCES.ko.md#s083) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-040: R04 → R16 / 기능상필수

**산출:** typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트.

**소비:** trigger dispatcher→proc/extra attack→projectile/area/timer→후속 reaction.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [trigger dispatcher→proc/extra attack→projectile/area/timer→후속 reaction]에서 필요한 산출물 [typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: A1.e→B1.e→A1.e만 차단하고 다른 사본/효과/독립공격은 허용. 지연시켜도 동일 인과 경로는 유지.

기존 충족/재사용: DamageSourceSnapshot·cast_seq·pending start_seq·step progress·death/wave 흐름.

남은 정책 gate: PG02, PG03.

후행이 약속한 유효 상태: A1.e→B1.e→A1.e만 차단하고 다른 사본/효과/독립공격은 허용. 지연시켜도 동일 인과 경로는 유지.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S045](../evidence/SOURCES.ko.md#s045) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083).

### REL-041: R05 → R16 / 기능상필수

**산출:** owner×lifetime resource/lease·원자적 소비/복원 포트.

**소비:** trigger dispatcher→proc/extra attack→projectile/area/timer→후속 reaction.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [trigger dispatcher→proc/extra attack→projectile/area/timer→후속 reaction]에서 필요한 산출물 [owner×lifetime resource/lease·원자적 소비/복원 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: A1.e→B1.e→A1.e만 차단하고 다른 사본/효과/독립공격은 허용. 지연시켜도 동일 인과 경로는 유지.

기존 충족/재사용: 기존 battle attempt/entry/checkpoint transaction·장비 독립 소유.

남은 정책 gate: PG02, PG03.

후행이 약속한 유효 상태: A1.e→B1.e→A1.e만 차단하고 다른 사본/효과/독립공격은 허용. 지연시켜도 동일 인과 경로는 유지.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S045](../evidence/SOURCES.ko.md#s045) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-042: R14 → R16 / 기능상필수

**산출:** 유물 사본 binding·실제 numeric effective plan/설명/저장.

**소비:** trigger dispatcher→proc/extra attack→projectile/area/timer→후속 reaction.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [trigger dispatcher→proc/extra attack→projectile/area/timer→후속 reaction]에서 필요한 산출물 [유물 사본 binding·실제 numeric effective plan/설명/저장]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: A1.e→B1.e→A1.e만 차단하고 다른 사본/효과/독립공격은 허용. 지연시켜도 동일 인과 경로는 유지.

기존 충족/재사용: 장비 loadout transaction·EconomyGrantState·save/entry·R01주소/R10plan.

남은 정책 gate: PG02, PG03.

후행이 약속한 유효 상태: A1.e→B1.e→A1.e만 차단하고 다른 사본/효과/독립공격은 허용. 지연시켜도 동일 인과 경로는 유지.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S041](../evidence/SOURCES.ko.md#s041) · [S045](../evidence/SOURCES.ko.md#s045) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-043: R15 → R16 / 기능상필수

**산출:** 유한 grant/활성 closure·typed structural recipe 증거.

**소비:** reaction definition과 (copy,effect) 재방문 key.

필요한 것은 생성 effect의 안정 주소·유한 closure다. 모든 shield/channel recipe 완료를 요구하지 않는다.

무시/오해 시: 후행 소비 지점 [reaction definition과 (copy,effect) 재방문 key]에서 필요한 산출물 [유한 grant/활성 closure·typed structural recipe 증거]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: A1.e→B1.e→A1.e만 차단하고 다른 사본/효과/독립공격은 허용. 지연시켜도 동일 인과 경로는 유지.

기존 충족/재사용: R14 immutable plan/selector·R11membership/R12pool.

남은 정책 gate: PG02, PG03.

후행이 약속한 유효 상태: A1.e→B1.e→A1.e만 차단하고 다른 사본/효과/독립공격은 허용. 지연시켜도 동일 인과 경로는 유지.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S029](../evidence/SOURCES.ko.md#s029) · [S045](../evidence/SOURCES.ko.md#s045) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S076](../evidence/SOURCES.ko.md#s076) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S087](../evidence/SOURCES.ko.md#s087).

### REL-044: R06 → R17 / 기능상필수

**산출:** typed value/result/resource 식·집계·스탯 입력.

**소비:** object/status source membership→relation/GroupByTile→detonation/consume.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [object/status source membership→relation/GroupByTile→detonation/consume]에서 필요한 산출물 [typed value/result/resource 식·집계·스탯 입력]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 연결 node와 감염 원천이 분리되고 타일별 상태·유닛값을 명시 단위로 집계한다.

기존 충족/재사용: stat_pipeline·scale_attack_interval_ms·기존 enhancement 소유/저장.

남은 정책 gate: PG02, PG03, PG04, PG05, PG06.

후행이 약속한 유효 상태: 연결 node와 감염 원천이 분리되고 타일별 상태·유닛값을 명시 단위로 집계한다.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S021](../evidence/SOURCES.ko.md#s021) · [S025](../evidence/SOURCES.ko.md#s025) · [S028](../evidence/SOURCES.ko.md#s028) · [S030](../evidence/SOURCES.ko.md#s030) · [S034](../evidence/SOURCES.ko.md#s034) · [S043](../evidence/SOURCES.ko.md#s043) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S054](../evidence/SOURCES.ko.md#s054) · [S058](../evidence/SOURCES.ko.md#s058) · [S061](../evidence/SOURCES.ko.md#s061) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091) · [S095](../evidence/SOURCES.ko.md#s095).

### REL-045: R13 → R17 / 기능상필수

**산출:** actor/object identity·원자적 생성·진화/삭제 수명.

**소비:** object/status source membership→relation/GroupByTile→detonation/consume.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [object/status source membership→relation/GroupByTile→detonation/consume]에서 필요한 산출물 [actor/object identity·원자적 생성·진화/삭제 수명]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 연결 node와 감염 원천이 분리되고 타일별 상태·유닛값을 명시 단위로 집계한다.

기존 충족/재사용: SpawnKernel의 정규화·idempotent request·원자적 commit·기존 actor summon.

남은 정책 gate: PG02, PG03, PG04, PG05, PG06.

후행이 약속한 유효 상태: 연결 node와 감염 원천이 분리되고 타일별 상태·유닛값을 명시 단위로 집계한다.

근거: [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S025](../evidence/SOURCES.ko.md#s025) · [S047](../evidence/SOURCES.ko.md#s047) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S054](../evidence/SOURCES.ko.md#s054) · [S058](../evidence/SOURCES.ko.md#s058) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087).

### REL-046: R16 → R17 / 기능상필수

**산출:** branch-local ancestry·지연 전달·suppression dispatcher.

**소비:** object/status source membership→relation/GroupByTile→detonation/consume.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [object/status source membership→relation/GroupByTile→detonation/consume]에서 필요한 산출물 [branch-local ancestry·지연 전달·suppression dispatcher]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 연결 node와 감염 원천이 분리되고 타일별 상태·유닛값을 명시 단위로 집계한다.

기존 충족/재사용: 현재 occurrence/proc-source 정렬·R04receipt·R14소유출처.

남은 정책 gate: PG02, PG03, PG04, PG05, PG06.

후행이 약속한 유효 상태: 연결 node와 감염 원천이 분리되고 타일별 상태·유닛값을 명시 단위로 집계한다.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S045](../evidence/SOURCES.ko.md#s045) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S054](../evidence/SOURCES.ko.md#s054) · [S058](../evidence/SOURCES.ko.md#s058) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S087](../evidence/SOURCES.ko.md#s087).

### REL-047: R08 → R18 / 기능상필수

**산출:** BossTask→Action request/result/cancel/방향 adapter.

**소비:** boss state profile/selector→field-work signal→multi-actor/wave 관찰.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [boss state profile/selector→field-work signal→multi-actor/wave 관찰]에서 필요한 산출물 [BossTask→Action request/result/cancel/방향 adapter]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: C03 HP연속/phase skip와 C07거리조건·잠복·작업억제가 data-only graph로 성립. 새 selector는 실제 executor와 validator가 함께 존재.

기존 충족/재사용: 기존 compiled phase/behavior/pattern/state/task graph·task activation·boss boundary clone/commit.

남은 정책 gate: PG03, PG04, PG06.

후행이 약속한 유효 상태: C03 HP연속/phase skip와 C07거리조건·잠복·작업억제가 data-only graph로 성립. 새 selector는 실제 executor와 validator가 함께 존재.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S058](../evidence/SOURCES.ko.md#s058) · [S084](../evidence/SOURCES.ko.md#s084) · [S094](../evidence/SOURCES.ko.md#s094) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-048: R11 → R18 / 기능상필수

**산출:** source별 reversible profile/lease·membership·종료 포트.

**소비:** state profile overlay와 state exit cleanup.

보스 형태에 쓰는 reversible profile/lease만 필요하다. 전체 channel 기믹을 기다리지 않는다.

무시/오해 시: 후행 소비 지점 [state profile overlay와 state exit cleanup]에서 필요한 산출물 [source별 reversible profile/lease·membership·종료 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: C03 HP연속/phase skip와 C07거리조건·잠복·작업억제가 data-only graph로 성립. 새 selector는 실제 executor와 validator가 함께 존재.

기존 충족/재사용: 기본 공격 start/release/recovery·기존 stat pipeline·R05 lease.

남은 정책 gate: PG03, PG04, PG06.

후행이 약속한 유효 상태: C03 HP연속/phase skip와 C07거리조건·잠복·작업억제가 data-only graph로 성립. 새 selector는 실제 executor와 validator가 함께 존재.

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S029](../evidence/SOURCES.ko.md#s029) · [S030](../evidence/SOURCES.ko.md#s030) · [S043](../evidence/SOURCES.ko.md#s043) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S058](../evidence/SOURCES.ko.md#s058) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S084](../evidence/SOURCES.ko.md#s084) · [S087](../evidence/SOURCES.ko.md#s087) · [S094](../evidence/SOURCES.ko.md#s094) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-049: R13 → R18 / 기능상필수

**산출:** actor/object identity·원자적 생성·진화/삭제 수명.

**소비:** multi-actor binding·소환 batch 수명.

필요한 것은 사용하는 actor의 생성/수명 adapter다. 모든 trap/wall recipe가 보스 기본 호출의 선행은 아니다.

무시/오해 시: 후행 소비 지점 [multi-actor binding·소환 batch 수명]에서 필요한 산출물 [actor/object identity·원자적 생성·진화/삭제 수명]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: C03 HP연속/phase skip와 C07거리조건·잠복·작업억제가 data-only graph로 성립. 새 selector는 실제 executor와 validator가 함께 존재.

기존 충족/재사용: SpawnKernel의 정규화·idempotent request·원자적 commit·기존 actor summon.

남은 정책 gate: PG03, PG04, PG06.

후행이 약속한 유효 상태: C03 HP연속/phase skip와 C07거리조건·잠복·작업억제가 data-only graph로 성립. 새 selector는 실제 executor와 validator가 함께 존재.

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S058](../evidence/SOURCES.ko.md#s058) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S084](../evidence/SOURCES.ko.md#s084) · [S087](../evidence/SOURCES.ko.md#s087) · [S094](../evidence/SOURCES.ko.md#s094) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-050: R02 → R19 / 기능상필수

**산출:** 외부 source selection·검증된 immutable generation과 fingerprint.

**소비:** pve reward-source/encounter plan·commit→grant/save→Certified 모델.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [pve reward-source/encounter plan·commit→grant/save→Certified 모델]에서 필요한 산출물 [외부 source selection·검증된 immutable generation과 fingerprint]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 새 보스 등장→전투→정상보상→장착/저장과 공식교체가 연결. 단일source fragment 가정을 최신 승인정책으로 전환한다.

기존 충족/재사용: 현재 load_live_sources 검증 후 게시·Arc 데이터·generated setup 검증·state transaction의 원리.

남은 정책 gate: PG06, PG07, PG08.

후행이 약속한 유효 상태: 새 보스 등장→전투→정상보상→장착/저장과 공식교체가 연결. 단일source fragment 가정을 최신 승인정책으로 전환한다.

근거: [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S015](../evidence/SOURCES.ko.md#s015) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S042](../evidence/SOURCES.ko.md#s042) · [S054](../evidence/SOURCES.ko.md#s054) · [S055](../evidence/SOURCES.ko.md#s055) · [S056](../evidence/SOURCES.ko.md#s056) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S068](../evidence/SOURCES.ko.md#s068) · [S069](../evidence/SOURCES.ko.md#s069) · [S071](../evidence/SOURCES.ko.md#s071) · [S073](../evidence/SOURCES.ko.md#s073) · [S074](../evidence/SOURCES.ko.md#s074) · [S085](../evidence/SOURCES.ko.md#s085) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094).

### REL-051: R07 → R19 / 기능상필수

**산출:** 실제 파편 사본/형태·장착 binding·정상 지급/저장.

**소비:** pve reward-source/encounter plan·commit→grant/save→Certified 모델.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [pve reward-source/encounter plan·commit→grant/save→Certified 모델]에서 필요한 산출물 [실제 파편 사본/형태·장착 binding·정상 지급/저장]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 새 보스 등장→전투→정상보상→장착/저장과 공식교체가 연결. 단일source fragment 가정을 최신 승인정책으로 전환한다.

기존 충족/재사용: EquipmentLoadoutTransition의 clone/검증/commit 패턴, EconomyGrantState 및 UUID ownership.

남은 정책 gate: PG06, PG07, PG08.

후행이 약속한 유효 상태: 새 보스 등장→전투→정상보상→장착/저장과 공식교체가 연결. 단일source fragment 가정을 최신 승인정책으로 전환한다.

근거: [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S054](../evidence/SOURCES.ko.md#s054) · [S055](../evidence/SOURCES.ko.md#s055) · [S056](../evidence/SOURCES.ko.md#s056) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S073](../evidence/SOURCES.ko.md#s073) · [S074](../evidence/SOURCES.ko.md#s074) · [S075](../evidence/SOURCES.ko.md#s075) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094).

### REL-052: R14 → R19 / 기능상필수

**산출:** 유물 사본 binding·실제 numeric effective plan/설명/저장.

**소비:** pve reward-source/encounter plan·commit→grant/save→Certified 모델.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [pve reward-source/encounter plan·commit→grant/save→Certified 모델]에서 필요한 산출물 [유물 사본 binding·실제 numeric effective plan/설명/저장]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 새 보스 등장→전투→정상보상→장착/저장과 공식교체가 연결. 단일source fragment 가정을 최신 승인정책으로 전환한다.

기존 충족/재사용: 장비 loadout transaction·EconomyGrantState·save/entry·R01주소/R10plan.

남은 정책 gate: PG06, PG07, PG08.

후행이 약속한 유효 상태: 새 보스 등장→전투→정상보상→장착/저장과 공식교체가 연결. 단일source fragment 가정을 최신 승인정책으로 전환한다.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S045](../evidence/SOURCES.ko.md#s045) · [S054](../evidence/SOURCES.ko.md#s054) · [S055](../evidence/SOURCES.ko.md#s055) · [S056](../evidence/SOURCES.ko.md#s056) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S073](../evidence/SOURCES.ko.md#s073) · [S074](../evidence/SOURCES.ko.md#s074) · [S076](../evidence/SOURCES.ko.md#s076) · [S083](../evidence/SOURCES.ko.md#s083) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094).

### REL-053: R15 → R19 / 기능상필수

**산출:** 유한 grant/활성 closure·typed structural recipe 증거.

**소비:** pve reward-source/encounter plan·commit→grant/save→Certified 모델.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [pve reward-source/encounter plan·commit→grant/save→Certified 모델]에서 필요한 산출물 [유한 grant/활성 closure·typed structural recipe 증거]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 새 보스 등장→전투→정상보상→장착/저장과 공식교체가 연결. 단일source fragment 가정을 최신 승인정책으로 전환한다.

기존 충족/재사용: R14 immutable plan/selector·R11membership/R12pool.

남은 정책 gate: PG06, PG07, PG08.

후행이 약속한 유효 상태: 새 보스 등장→전투→정상보상→장착/저장과 공식교체가 연결. 단일source fragment 가정을 최신 승인정책으로 전환한다.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S029](../evidence/SOURCES.ko.md#s029) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S042](../evidence/SOURCES.ko.md#s042) · [S045](../evidence/SOURCES.ko.md#s045) · [S054](../evidence/SOURCES.ko.md#s054) · [S055](../evidence/SOURCES.ko.md#s055) · [S056](../evidence/SOURCES.ko.md#s056) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S073](../evidence/SOURCES.ko.md#s073) · [S074](../evidence/SOURCES.ko.md#s074) · [S076](../evidence/SOURCES.ko.md#s076) · [S081](../evidence/SOURCES.ko.md#s081) · [S083](../evidence/SOURCES.ko.md#s083) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094).

### REL-054: R18 → R19 / 기능상필수

**산출:** 보스 형태·신호·다중 actor의 통합 증거.

**소비:** pve reward-source/encounter plan·commit→grant/save→Certified 모델.

후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.

무시/오해 시: 후행 소비 지점 [pve reward-source/encounter plan·commit→grant/save→Certified 모델]에서 필요한 산출물 [보스 형태·신호·다중 actor의 통합 증거]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 새 보스 등장→전투→정상보상→장착/저장과 공식교체가 연결. 단일source fragment 가정을 최신 승인정책으로 전환한다.

기존 충족/재사용: 기존 boss compiledgraph·microsteptransaction·SpawnKernel·wave death epoch.

남은 정책 gate: PG06, PG07, PG08.

후행이 약속한 유효 상태: 새 보스 등장→전투→정상보상→장착/저장과 공식교체가 연결. 단일source fragment 가정을 최신 승인정책으로 전환한다.

근거: [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S042](../evidence/SOURCES.ko.md#s042) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S054](../evidence/SOURCES.ko.md#s054) · [S055](../evidence/SOURCES.ko.md#s055) · [S056](../evidence/SOURCES.ko.md#s056) · [S058](../evidence/SOURCES.ko.md#s058) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S073](../evidence/SOURCES.ko.md#s073) · [S074](../evidence/SOURCES.ko.md#s074) · [S084](../evidence/SOURCES.ko.md#s084) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-055: R09 → R20 / 검증선행

**산출:** 침식 suppression·강제 시계·직원 대가/Breakdown의 단독 검증.

**소비:** 복합 run/battle entry/attempt 결과와 restore/reconnect 검증.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 침식 suppression·강제 시계·직원 대가/Breakdown의 단독 검증 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: withdraw·finalize death bookkeeping·attempt roster/result commit·entry restore의 기존 경계.

남은 정책 gate: PG05, PG07.

후행이 약속한 유효 상태: 이미 각 기능단계에서 유효했던 저장·복원 계약이 복합조합에서도 유효함.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S022](../evidence/SOURCES.ko.md#s022) · [S025](../evidence/SOURCES.ko.md#s025) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S046](../evidence/SOURCES.ko.md#s046) · [S059](../evidence/SOURCES.ko.md#s059) · [S060](../evidence/SOURCES.ko.md#s060) · [S061](../evidence/SOURCES.ko.md#s061) · [S075](../evidence/SOURCES.ko.md#s075) · [S077](../evidence/SOURCES.ko.md#s077) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094).

### REL-056: R17 → R20 / 검증선행

**산출:** 관계/원천별 감염·집계·consume/cleanup 증거.

**소비:** 복합 run/battle entry/attempt 결과와 restore/reconnect 검증.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 관계/원천별 감염·집계·consume/cleanup 증거 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: 현재 spatial/area query와 R13 object/R16chain 기반.

남은 정책 gate: PG05, PG07.

후행이 약속한 유효 상태: 이미 각 기능단계에서 유효했던 저장·복원 계약이 복합조합에서도 유효함.

근거: [S021](../evidence/SOURCES.ko.md#s021) · [S025](../evidence/SOURCES.ko.md#s025) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S054](../evidence/SOURCES.ko.md#s054) · [S058](../evidence/SOURCES.ko.md#s058) · [S076](../evidence/SOURCES.ko.md#s076) · [S077](../evidence/SOURCES.ko.md#s077) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094).

### REL-057: R18 → R20 / 검증선행

**산출:** 보스 형태·신호·다중 actor의 통합 증거.

**소비:** 복합 run/battle entry/attempt 결과와 restore/reconnect 검증.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 보스 형태·신호·다중 actor의 통합 증거 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: 기존 boss compiledgraph·microsteptransaction·SpawnKernel·wave death epoch.

남은 정책 gate: PG05, PG07.

후행이 약속한 유효 상태: 이미 각 기능단계에서 유효했던 저장·복원 계약이 복합조합에서도 유효함.

근거: [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S058](../evidence/SOURCES.ko.md#s058) · [S077](../evidence/SOURCES.ko.md#s077) · [S084](../evidence/SOURCES.ko.md#s084) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-058: R19 → R20 / 검증선행

**산출:** 정상 등장/획득 선택/평가/교체의 통합 증거.

**소비:** 복합 run/battle entry/attempt 결과와 restore/reconnect 검증.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 정상 등장/획득 선택/평가/교체의 통합 증거 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: 기존 EconomyGrantState·encounter progression·Certified sort/profile.

남은 정책 gate: PG05, PG07.

후행이 약속한 유효 상태: 이미 각 기능단계에서 유효했던 저장·복원 계약이 복합조합에서도 유효함.

근거: [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S042](../evidence/SOURCES.ko.md#s042) · [S044](../evidence/SOURCES.ko.md#s044) · [S054](../evidence/SOURCES.ko.md#s054) · [S055](../evidence/SOURCES.ko.md#s055) · [S056](../evidence/SOURCES.ko.md#s056) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S073](../evidence/SOURCES.ko.md#s073) · [S074](../evidence/SOURCES.ko.md#s074) · [S077](../evidence/SOURCES.ko.md#s077) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094).

### REL-059: R03 → R21 / 기능상필수

**산출:** 공개 로딩/API/CLI·readonly 설명·headless 실행 포트.

**소비:** 공식 pack 로딩·실행·receipt.

최종 공식 인수는 private builder로 대신하지 못하므로 공개 source→실행 포트가 필요하다.

무시/오해 시: 후행 소비 지점 [공식 pack 로딩·실행·receipt]에서 필요한 산출물 [공개 로딩/API/CLI·readonly 설명·headless 실행 포트]을 공급하지 못해 다음 완료 상태를 보장할 수 없다: 정해진 대표구조가 실제RON으로 실행되고 전체baseline의 누락/정책대기 상태도 별도로 드러남.

기존 충족/재사용: CoreApplication/ValidatedGameData·기존 headless 시험 seam.

남은 정책 gate: PG00, PG01, PG02, PG03, PG04, PG05, PG06, PG09.

후행이 약속한 유효 상태: 정해진 대표구조가 실제RON으로 실행되고 전체baseline의 누락/정책대기 상태도 별도로 드러남.

근거: [S006](../evidence/SOURCES.ko.md#s006) · [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S017](../evidence/SOURCES.ko.md#s017) · [S045](../evidence/SOURCES.ko.md#s045) · [S046](../evidence/SOURCES.ko.md#s046) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S058](../evidence/SOURCES.ko.md#s058) · [S068](../evidence/SOURCES.ko.md#s068) · [S069](../evidence/SOURCES.ko.md#s069) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S078](../evidence/SOURCES.ko.md#s078) · [S079](../evidence/SOURCES.ko.md#s079) · [S085](../evidence/SOURCES.ko.md#s085) · [S087](../evidence/SOURCES.ko.md#s087) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-060: R15 → R21 / 검증선행

**산출:** 유한 grant/활성 closure·typed structural recipe 증거.

**소비:** 공개 최신 RON/production loader를 통한 C01–C07 및 전체 COV 인수.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 유한 grant/활성 closure·typed structural recipe 증거 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: R14 immutable plan/selector·R11membership/R12pool.

남은 정책 gate: PG00, PG01, PG02, PG03, PG04, PG05, PG06, PG09.

후행이 약속한 유효 상태: 정해진 대표구조가 실제RON으로 실행되고 전체baseline의 누락/정책대기 상태도 별도로 드러남.

근거: [S006](../evidence/SOURCES.ko.md#s006) · [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S029](../evidence/SOURCES.ko.md#s029) · [S045](../evidence/SOURCES.ko.md#s045) · [S046](../evidence/SOURCES.ko.md#s046) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S058](../evidence/SOURCES.ko.md#s058) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S078](../evidence/SOURCES.ko.md#s078) · [S079](../evidence/SOURCES.ko.md#s079) · [S081](../evidence/SOURCES.ko.md#s081) · [S083](../evidence/SOURCES.ko.md#s083) · [S087](../evidence/SOURCES.ko.md#s087) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-061: R16 → R21 / 검증선행

**산출:** branch-local ancestry·지연 전달·suppression dispatcher.

**소비:** 공개 최신 RON/production loader를 통한 C01–C07 및 전체 COV 인수.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 branch-local ancestry·지연 전달·suppression dispatcher 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: 현재 occurrence/proc-source 정렬·R04receipt·R14소유출처.

남은 정책 gate: PG00, PG01, PG02, PG03, PG04, PG05, PG06, PG09.

후행이 약속한 유효 상태: 정해진 대표구조가 실제RON으로 실행되고 전체baseline의 누락/정책대기 상태도 별도로 드러남.

근거: [S006](../evidence/SOURCES.ko.md#s006) · [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S045](../evidence/SOURCES.ko.md#s045) · [S046](../evidence/SOURCES.ko.md#s046) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S058](../evidence/SOURCES.ko.md#s058) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S078](../evidence/SOURCES.ko.md#s078) · [S079](../evidence/SOURCES.ko.md#s079) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S087](../evidence/SOURCES.ko.md#s087) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-062: R17 → R21 / 검증선행

**산출:** 관계/원천별 감염·집계·consume/cleanup 증거.

**소비:** 공개 최신 RON/production loader를 통한 C01–C07 및 전체 COV 인수.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 관계/원천별 감염·집계·consume/cleanup 증거 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: 현재 spatial/area query와 R13 object/R16chain 기반.

남은 정책 gate: PG00, PG01, PG02, PG03, PG04, PG05, PG06, PG09.

후행이 약속한 유효 상태: 정해진 대표구조가 실제RON으로 실행되고 전체baseline의 누락/정책대기 상태도 별도로 드러남.

근거: [S006](../evidence/SOURCES.ko.md#s006) · [S021](../evidence/SOURCES.ko.md#s021) · [S025](../evidence/SOURCES.ko.md#s025) · [S045](../evidence/SOURCES.ko.md#s045) · [S046](../evidence/SOURCES.ko.md#s046) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S058](../evidence/SOURCES.ko.md#s058) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S078](../evidence/SOURCES.ko.md#s078) · [S079](../evidence/SOURCES.ko.md#s079) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-063: R18 → R21 / 검증선행

**산출:** 보스 형태·신호·다중 actor의 통합 증거.

**소비:** 공개 최신 RON/production loader를 통한 C01–C07 및 전체 COV 인수.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 보스 형태·신호·다중 actor의 통합 증거 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: 기존 boss compiledgraph·microsteptransaction·SpawnKernel·wave death epoch.

남은 정책 gate: PG00, PG01, PG02, PG03, PG04, PG05, PG06, PG09.

후행이 약속한 유효 상태: 정해진 대표구조가 실제RON으로 실행되고 전체baseline의 누락/정책대기 상태도 별도로 드러남.

근거: [S006](../evidence/SOURCES.ko.md#s006) · [S045](../evidence/SOURCES.ko.md#s045) · [S046](../evidence/SOURCES.ko.md#s046) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S058](../evidence/SOURCES.ko.md#s058) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S078](../evidence/SOURCES.ko.md#s078) · [S079](../evidence/SOURCES.ko.md#s079) · [S084](../evidence/SOURCES.ko.md#s084) · [S087](../evidence/SOURCES.ko.md#s087) · [S094](../evidence/SOURCES.ko.md#s094) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-064: R20 → R21 / 검증선행

**산출:** 복합 결과·후퇴·저장·재접속의 현재 revision 증거.

**소비:** 공개 최신 RON/production loader를 통한 C01–C07 및 전체 COV 인수.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 복합 결과·후퇴·저장·재접속의 현재 revision 증거 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: 현 capture/restore 및 result transaction.

남은 정책 gate: PG00, PG01, PG02, PG03, PG04, PG05, PG06, PG09.

후행이 약속한 유효 상태: 정해진 대표구조가 실제RON으로 실행되고 전체baseline의 누락/정책대기 상태도 별도로 드러남.

근거: [S006](../evidence/SOURCES.ko.md#s006) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S045](../evidence/SOURCES.ko.md#s045) · [S046](../evidence/SOURCES.ko.md#s046) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S058](../evidence/SOURCES.ko.md#s058) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S077](../evidence/SOURCES.ko.md#s077) · [S078](../evidence/SOURCES.ko.md#s078) · [S079](../evidence/SOURCES.ko.md#s079) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-065: R21 → R22 / 검증선행

**산출:** 대표 공식 실제 RON·전체 baseline disposition·지원 문서.

**소비:** 별도 작성자의 frozen-engine 외부 pack 제작·검증·실행.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 대표 공식 실제 RON·전체 baseline disposition·지원 문서 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: 각primitive단계의진짜RON/trace.

남은 정책 gate: .

후행이 약속한 유효 상태: 지원 어휘를 다른 작성자가 새 조합에 재사용했음이 분리된 증거로 확인됨.

근거: [S006](../evidence/SOURCES.ko.md#s006) · [S045](../evidence/SOURCES.ko.md#s045) · [S046](../evidence/SOURCES.ko.md#s046) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S057](../evidence/SOURCES.ko.md#s057) · [S058](../evidence/SOURCES.ko.md#s058) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S078](../evidence/SOURCES.ko.md#s078) · [S079](../evidence/SOURCES.ko.md#s079) · [S084](../evidence/SOURCES.ko.md#s084) · [S085](../evidence/SOURCES.ko.md#s085) · [S087](../evidence/SOURCES.ko.md#s087) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-066: R22 → R23 / 검증선행

**산출:** 엔진 동결 후 독립 작성/실행 및 data-only 증거.

**소비:** 독립 제작 workload를 포함한 fault/결정론/성능 통합 시험.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 엔진 동결 후 독립 작성/실행 및 data-only 증거 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: R03공개port·R15/16합성연쇄·R19등록.

남은 정책 gate: PG08.

후행이 약속한 유효 상태: 초기부터배정한안전장치가복합부하에서유효하고측정된범위의성능만판정.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S026](../evidence/SOURCES.ko.md#s026) · [S037](../evidence/SOURCES.ko.md#s037) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S054](../evidence/SOURCES.ko.md#s054) · [S057](../evidence/SOURCES.ko.md#s057) · [S058](../evidence/SOURCES.ko.md#s058) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082) · [S084](../evidence/SOURCES.ko.md#s084) · [S085](../evidence/SOURCES.ko.md#s085).

### REL-067: R23 → R24 / 검증선행

**산출:** 현재 revision의 결정론·fault·성능·회귀 증거.

**소비:** 독립 감사자의 coverage/currentness/실제 결과 판정.

후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.

무시/오해 시: 후행 완료 주장에 필요한 현재 revision의 결정론·fault·성능·회귀 증거 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다..

기존 충족/재사용: 기존bossmicrostep/cloneboundary 및각R예산계측.

남은 정책 gate: PG00.

후행이 약속한 유효 상태: 필수미해결이없을때만제작기반완료. 공식전체이식/조합기/클라이언트출시와별개.

근거: [S006](../evidence/SOURCES.ko.md#s006) · [S018](../evidence/SOURCES.ko.md#s018) · [S026](../evidence/SOURCES.ko.md#s026) · [S037](../evidence/SOURCES.ko.md#s037) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S054](../evidence/SOURCES.ko.md#s054) · [S057](../evidence/SOURCES.ko.md#s057) · [S058](../evidence/SOURCES.ko.md#s058) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082) · [S085](../evidence/SOURCES.ko.md#s085) · [S088](../evidence/SOURCES.ko.md#s088) · [S089](../evidence/SOURCES.ko.md#s089).

### REL-068: R03 → R04 / 진행상권장

**산출:** 공개 로딩/API/CLI·readonly 설명·headless 실행 포트.

**소비:** damage_runtime/commands/death_runtime/triggers 및 skill_runtime의 결과·수명 포트.

초기 공개 RON 관찰 경로를 준비해 재작업을 줄인다. 기존 skill_runtime_contract 등의 kernel seam으로 기능 개발은 가능하다.

무시/오해 시: 기능이 반드시 깨진다는 뜻은 아니다. 초기 공개 RON 관찰 경로를 준비해 재작업을 줄인다. 기존 skill_runtime_contract 등의 kernel seam으로 기능 개발은 가능하다..

기존 충족/재사용: CoreApplication/ValidatedGameData·기존 headless 시험 seam.

남은 정책 gate: PG02, PG03, PG04.

후행이 약속한 유효 상태: 기존 스킬/전달체를 그대로 쓰면서 Accepted/Released/FlowFinished/EffectsSettled와 실제 HP/회복/처치·비용을 관찰한다. 보스 adapter나 새 Flow 전체는 아직 필요 없음.

근거: [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S064](../evidence/SOURCES.ko.md#s064) · [S068](../evidence/SOURCES.ko.md#s068) · [S069](../evidence/SOURCES.ko.md#s069) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082) · [S085](../evidence/SOURCES.ko.md#s085).

### REL-069: R03 → R05 / 진행상권장

**산출:** 공개 로딩/API/CLI·readonly 설명·headless 실행 포트.

**소비:** resource owner key·cost request→attempt/entry/checkpoint/result commit.

ResourceStore 자체의 코드 선행이 아니라 외부 제작 인수의 편의·일관성이다.

무시/오해 시: 기능이 반드시 깨진다는 뜻은 아니다. ResourceStore 자체의 코드 선행이 아니라 외부 제작 인수의 편의·일관성이다..

기존 충족/재사용: CoreApplication/ValidatedGameData·기존 headless 시험 seam.

남은 정책 gate: PG05.

후행이 약속한 유효 상태: 두 소유 사본의 자원이 분리되고 배치 stack만 재배치에 초기화된다. attempt 사용 횟수와 Run 자원은 지정된 수명을 따른다.

근거: [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S017](../evidence/SOURCES.ko.md#s017) · [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S068](../evidence/SOURCES.ko.md#s068) · [S069](../evidence/SOURCES.ko.md#s069) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082) · [S085](../evidence/SOURCES.ko.md#s085) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-070: R06 → R07 / 진행상권장

**산출:** typed value/result/resource 식·집계·스탯 입력.

**소비:** SkillFragmentInventory/Loadout→Maintenance/grant→player_spawns/profile→save/표시.

profile 파일 충돌을 줄이려고 성장 입력 뒤에 사본 전환한다. 사본 저장/선택 자체는 Expr 전체와 독립이다.

무시/오해 시: 기능이 반드시 깨진다는 뜻은 아니다. profile 파일 충돌을 줄이려고 성장 입력 뒤에 사본 전환한다. 사본 저장/선택 자체는 Expr 전체와 독립이다..

기존 충족/재사용: stat_pipeline·scale_attack_interval_ms·기존 enhancement 소유/저장.

남은 정책 gate: PG05, PG06.

후행이 약속한 유효 상태: 같은 정의 A만 개화·형태 선택, B 미개화; 이전 후에도 A identity 유지. 현재 지원 기능의 기본/개화 form이 실제 전투에 적용되고 저장된다.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S028](../evidence/SOURCES.ko.md#s028) · [S030](../evidence/SOURCES.ko.md#s030) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S043](../evidence/SOURCES.ko.md#s043) · [S061](../evidence/SOURCES.ko.md#s061) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S075](../evidence/SOURCES.ko.md#s075) · [S091](../evidence/SOURCES.ko.md#s091) · [S095](../evidence/SOURCES.ko.md#s095).

### REL-071: R07 → R08 / 진행상권장

**산출:** 실제 파편 사본/형태·장착 binding·정상 지급/저장.

**소비:** BossTaskData/compiled task→공통 내부 action admission→state activation result.

보수적 순차 기본안이다. R08의 실제 필수는 R01/R04뿐이다.

무시/오해 시: 기능이 반드시 깨진다는 뜻은 아니다. 보수적 순차 기본안이다. R08의 실제 필수는 R01/R04뿐이다..

기존 충족/재사용: EquipmentLoadoutTransition의 clone/검증/commit 패턴, EconomyGrantState 및 UUID ownership.

남은 정책 gate: PG03.

후행이 약속한 유효 상태: 기존 보스 graph가 현재 지원 스킬을 명시적으로 요청하고 release/settled를 선택해 기다린다. 설치물·공유 shield·전체 Flow 없이도 정상·실패 route가 유효.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S064](../evidence/SOURCES.ko.md#s064) · [S075](../evidence/SOURCES.ko.md#s075) · [S084](../evidence/SOURCES.ko.md#s084) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094).

### REL-072: R09 → R10 / 진행상권장

**산출:** 침식 suppression·강제 시계·직원 대가/Breakdown의 단독 검증.

**소비:** sim/skill_runtime의 continuation/result slot·반복·전달 분리.

침식 suppression/clock 경계를 먼저 닫아 후속 기능 추가가 이를 회귀 인수하게 한다. 미정이면 지정 분기만 유예한다.

무시/오해 시: 기능이 반드시 깨진다는 뜻은 아니다. 침식 suppression/clock 경계를 먼저 닫아 후속 기능 추가가 이를 회귀 인수하게 한다. 미정이면 지정 분기만 유예한다..

기존 충족/재사용: withdraw·finalize death bookkeeping·attempt roster/result commit·entry restore의 기존 경계.

남은 정책 gate: PG03, PG04.

후행이 약속한 유효 상태: AoE 피해 뒤 자기 회복 한 번, 결과 조건·유한 기회 반복·병행 join이 실제 실행된다. 기존 최소 주소와 보스 adapter가 같은 service를 소비.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S025](../evidence/SOURCES.ko.md#s025) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S046](../evidence/SOURCES.ko.md#s046) · [S059](../evidence/SOURCES.ko.md#s059) · [S060](../evidence/SOURCES.ko.md#s060) · [S061](../evidence/SOURCES.ko.md#s061) · [S070](../evidence/SOURCES.ko.md#s070) · [S075](../evidence/SOURCES.ko.md#s075) · [S077](../evidence/SOURCES.ko.md#s077) · [S079](../evidence/SOURCES.ko.md#s079) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S091](../evidence/SOURCES.ko.md#s091) · [S097](../evidence/SOURCES.ko.md#s097).

### REL-073: R13 → R14 / 진행상권장

**산출:** actor/object identity·원자적 생성·진화/삭제 수명.

**소비:** loadout Composer→cast/queued step/projectile/area/개인 preview·평가 입력.

공용 types/skill runtime/owner-state 변경을 순차 통합해 개인 plan 전환 충돌을 줄인다.

무시/오해 시: 기능이 반드시 깨진다는 뜻은 아니다. 공용 types/skill runtime/owner-state 변경을 순차 통합해 개인 plan 전환 충돌을 줄인다..

기존 충족/재사용: SpawnKernel의 정규화·idempotent request·원자적 commit·기존 actor summon.

남은 정책 gate: PG01, PG05, PG06, PG08.

후행이 약속한 유효 상태: 유물을 정상 획득·장착·저장하고 지원한 수치 변형이 즉시/지연 실행에 실제 반영된다. 요구 효과가 없는 유물은 슬롯 유지·휴면.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S041](../evidence/SOURCES.ko.md#s041) · [S045](../evidence/SOURCES.ko.md#s045) · [S047](../evidence/SOURCES.ko.md#s047) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-074: R12 → R18 / 조건부

**산출:** 공용 pool/HP비용·방어 중재의 필요한 최소 포트.

**소비:** boss state profile/selector→field-work signal→multi-actor/wave 관찰.

공유 보호막을 직접 사용하는 보스에 한해 pool 계약이 필요하다. 거리 무효화 predicate와 R08 기본 호출은 전체 shield의 선행을 요구하지 않는다.

조건: 해당 boss가 shared shield capability를 채택한 경우.

무시/오해 시: 공유 보호막을 사용하는 보스가 원천 pool과 membership을 제공받지 못하면 잔량·흡수·종료의 authority가 없어 그 기믹이 성립하지 않는다. 공유 보호막을 사용하지 않는 보스의 요청/방향/상태 전환은 이 관계로 차단하지 않는다..

기존 충족/재사용: 기존 damage reduction·control/movement·R05 resource/R11 membership.

남은 정책 gate: PG03, PG04, PG06.

후행이 약속한 유효 상태: C03 HP연속/phase skip와 C07거리조건·잠복·작업억제가 data-only graph로 성립. 새 selector는 실제 executor와 validator가 함께 존재.

근거: [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S028](../evidence/SOURCES.ko.md#s028) · [S029](../evidence/SOURCES.ko.md#s029) · [S043](../evidence/SOURCES.ko.md#s043) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S058](../evidence/SOURCES.ko.md#s058) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S084](../evidence/SOURCES.ko.md#s084) · [S087](../evidence/SOURCES.ko.md#s087) · [S094](../evidence/SOURCES.ko.md#s094) · [S096](../evidence/SOURCES.ko.md#s096).

### REL-075: R09 → R16 / 조건부

**산출:** 침식 suppression·강제 시계·직원 대가/Breakdown의 단독 검증.

**소비:** trigger dispatcher→proc/extra attack→projectile/area/timer→후속 reaction.

R09가 먼저 구현됐다면 새 모든 subscriber가 침식 suppression 회귀를 인수한다. R09가 유예되면 나중 R09가 새 subscriber를 인수한다.

조건: 침식 또는 새 chain의 실제 도입 순서에 따른 재검증 owner.

무시/오해 시: 이미 구현된 침식에 새 chain subscriber가 억제 계약을 적용하지 않으면 침식 kill에서 금지된 유물/장비/파편 연쇄가 새로 발생한다. R09가 뒤에 도입되면 R09가 이 결합 검증을 소유한다..

기존 충족/재사용: withdraw·finalize death bookkeeping·attempt roster/result commit·entry restore의 기존 경계.

남은 정책 gate: PG02, PG03.

후행이 약속한 유효 상태: A1.e→B1.e→A1.e만 차단하고 다른 사본/효과/독립공격은 허용. 지연시켜도 동일 인과 경로는 유지.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S045](../evidence/SOURCES.ko.md#s045) · [S046](../evidence/SOURCES.ko.md#s046) · [S059](../evidence/SOURCES.ko.md#s059) · [S060](../evidence/SOURCES.ko.md#s060) · [S061](../evidence/SOURCES.ko.md#s061) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S075](../evidence/SOURCES.ko.md#s075) · [S077](../evidence/SOURCES.ko.md#s077) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S091](../evidence/SOURCES.ko.md#s091).

### REL-076: R23 → R22 / 검증선행

**산출:** 현재 revision의 결정론·fault·성능·회귀 증거.

**소비:** 별도 작성자의 frozen-engine 외부 pack 제작·검증·실행.

최적화가 의미·지원 범위·API를 바꾼 경우 영향받은 독립 제작 증거를 다시 획득한다. 구현 순환/항상 실행 선행이 아니다.

조건: 변경 영향이 있는 경우에만 재검증.

무시/오해 시: 독립 제작 시험 뒤 의미·지원 범위·API를 바꾼 최적화가 들어가면 이전 동결 revision의 PASS는 변경된 엔진을 증명하지 못한다. 해당 변경 영향이 없으면 이 재검증 관계는 적용하지 않는다..

기존 충족/재사용: 기존bossmicrostep/cloneboundary 및각R예산계측.

남은 정책 gate: .

후행이 약속한 유효 상태: 지원 어휘를 다른 작성자가 새 조합에 재사용했음이 분리된 증거로 확인됨.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S026](../evidence/SOURCES.ko.md#s026) · [S037](../evidence/SOURCES.ko.md#s037) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S054](../evidence/SOURCES.ko.md#s054) · [S057](../evidence/SOURCES.ko.md#s057) · [S058](../evidence/SOURCES.ko.md#s058) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082) · [S084](../evidence/SOURCES.ko.md#s084) · [S085](../evidence/SOURCES.ko.md#s085).

## 4. 최소 동시 전환 묶음

R 카드가 하나라고 전체 내용을 하나의 transaction이나 하나의 Git commit으로 합치지 않는다. 아래 component 간 공개 유효 상태가 끊기지 않아야 한다. R14 관리 aggregate와 Composer는 구조적으로 분리되어도 좋고, 기본 권고 작업 단위만 합친 것이다.

### AT01 — R01

**같이 바꿀 경계:** typed metadata/compiler + latest RON/catalog + 직접 runtime 소비자.

**필요 이유:** 구형/신형 의미가 섞인 publication 또는 빈 metadata만의 성공을 막는다. 기존 AG-CAMEC-01을 그대로 재사용한다.

**코드/계약 위치:** `ability.rs; skill_data.rs; live_ron.rs; sim.rs; skill_runtime; game_resources/data/skills; P2G_40_PHASE_PLAN_002.json`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S010](../evidence/SOURCES.ko.md#s010) · [S013](../evidence/SOURCES.ko.md#s013) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S079](../evidence/SOURCES.ko.md#s079).

### AT02 — R02

**같이 바꿀 경계:** 외부 snapshot/resolve/compile/publication + new-run 데이터 선택 + fingerprint save/restore.

**필요 이유:** GameCore state-only commit으로 세대 선택을 잃거나 다른 내용의 동일 ID 저장을 복원하지 않게 한다.

**코드/계약 위치:** `application.rs; data/{live_ron,mod}; world.rs; world/{application,active_run,state,node_flow}`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S015](../evidence/SOURCES.ko.md#s015) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040).

### AT03 — R04

**같이 바꿀 경계:** Damage/Heal/Death/Cost receipt 생성 + 실제 mutation + 기존 optional trigger 소비.

**필요 이유:** HP 사실/처치 사실과 후속 반응의 의미를 동시에 바꿔야 한다. mandatory wave는 유지한다.

**코드/계약 위치:** `damage_runtime; commands; death_runtime; triggers; wave_runtime; recording/event_log`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S053](../evidence/SOURCES.ko.md#s053) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072).

### AT04 — R04

**같이 바꿀 경계:** persistent equipment copy → draft → RuntimeItem source binding.

**필요 이유:** base UUID/배치 인덱스로 영구 소유 사본을 추정할 수 없다.

**코드/계약 위치:** `resources/inventory; combat_setup/player_spawns; battle/core/build; core/types`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S034](../evidence/SOURCES.ko.md#s034) · [S064](../evidence/SOURCES.ko.md#s064) · [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027).

### AT05 — R05

**같이 바꿀 경계:** 새 Run 자원 writer + overlay/result/entry + checkpoint reader/validator.

**필요 이유:** 다음 저장 단계가 없어도 새 상태가 유효하게 왕복해야 한다.

**코드/계약 위치:** `world/{state,active_run,application,combat,node_flow}; 새 resource store`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044).

### AT06 — R06

**같이 바꿀 경계:** Attack/SpellPower·무기 성장 입력 + profile/stat 계산 + 해당 projection/save 검증.

**필요 이유:** 기존 무기 기여 이중 합산 또는 실제 계산과 표시 입력의 분리를 방지한다. 숫자 평가 가중 결정과는 별개다.

**코드/계약 위치:** `stats; stat_pipeline; inventory; player_spawns; range_preview; combat_power; snapshot`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S030](../evidence/SOURCES.ko.md#s030) · [S034](../evidence/SOURCES.ko.md#s034) · [S043](../evidence/SOURCES.ko.md#s043) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S091](../evidence/SOURCES.ko.md#s091).

### AT07 — R07

**같이 바꿀 경계:** 파편 copy aggregate + grant/equip/select/awake/dismantle/recovery + profile + snapshot/checkpoint.

**필요 이유:** 종류 장부와 새 사본 장부를 이중 truth로 남기거나 실제 장착/저장에서 사본성을 잃지 않게 한다.

**코드/계약 위치:** `skill_fragment; skill_fragment_data; maintenance; helpers; grant_state; reward; player_spawns; snapshot/state/active_run`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S064](../evidence/SOURCES.ko.md#s064) · [S091](../evidence/SOURCES.ko.md#s091).

### AT08 — R08

**같이 바꿀 경계:** BossTask raw/compile + 권한 있는 Action admission + result/cancel/facing 소비.

**필요 이유:** enum 존재·Accepted 응답만으로 실행/완료를 가장하지 않는다.

**코드/계약 위치:** `boss_runtime_data; boss_runtime; sim; skill_runtime/cast; build`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S022](../evidence/SOURCES.ko.md#s022).

### AT09 — R09

**같이 바꿀 경계:** 침식 시전 대체/kill/suppression + Withdrawn/attempt trauma + 강제 clock + 결과·Breakdown·복원.

**필요 이유:** 마지막 적 처치로 전투가 즉시 끝나도 대가를 잃지 않으며, optional trigger만 차단하고 진행은 유지한다.

**코드/계약 위치:** `sim; commands; death_runtime; build; world/combat; employee; employee_psychology; node_flow; run_policy_data; active_run/snapshot`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S022](../evidence/SOURCES.ko.md#s022) · [S044](../evidence/SOURCES.ko.md#s044) · [S059](../evidence/SOURCES.ko.md#s059) · [S060](../evidence/SOURCES.ko.md#s060) · [S040](../evidence/SOURCES.ko.md#s040) · [S061](../evidence/SOURCES.ko.md#s061) · [S039](../evidence/SOURCES.ko.md#s039) · [S091](../evidence/SOURCES.ko.md#s091) · [S046](../evidence/SOURCES.ko.md#s046) · [S077](../evidence/SOURCES.ko.md#s077).

### AT10 — R10

**같이 바꿀 경계:** 새 Flow AST/compile + continuation/result/각 carrier 소비 + actual fixture.

**필요 이유:** 새 결과 slot/완료 milestone을 정의만 하고 구형 step executor에 방치하지 않는다.

**코드/계약 위치:** `ability; skill_data; sim; skill_runtime/{cast,projectile,area}; core/types`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S027](../evidence/SOURCES.ko.md#s027).

### AT11 — R11

**같이 바꿀 경계:** source별 modifier/lease 설치 + 모든 종료/복귀 경로 + effective profile.

**필요 이유:** 반대 숫자 덧셈이나 오래된 원형 snapshot으로 다른 source 변경까지 덮어 복귀하지 않게 한다.

**코드/계약 위치:** `buffs; stat_pipeline; build; sim; scope별 state`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S029](../evidence/SOURCES.ko.md#s029) · [S030](../evidence/SOURCES.ko.md#s030) · [S022](../evidence/SOURCES.ko.md#s022) · [S018](../evidence/SOURCES.ko.md#s018) · [S082](../evidence/SOURCES.ko.md#s082).

### AT12 — R12

**같이 바꿀 경계:** pool debit + actual HP/survival commit + receipts + membership cleanup.

**필요 이유:** 한 번의 피해가 여러 수혜자의 복제 잔량에 적용되거나 pool만 차감된 실패를 막는다.

**코드/계약 위치:** `damage_runtime; damage; commands; resource/membership store`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S023](../evidence/SOURCES.ko.md#s023) · [S028](../evidence/SOURCES.ko.md#s028) · [S024](../evidence/SOURCES.ko.md#s024) · [S082](../evidence/SOURCES.ko.md#s082).

### AT13 — R13

**같이 바꿀 경계:** 해당 생성 request의 비용·배치·UUID·생성 + 진화/삭제 시 참조 무효화.

**필요 이유:** 생성 실패에 비용만 소모하거나 구형 node timer가 새 node에 적용되지 않게 한다.

**코드/계약 위치:** `spawn_kernel; summon_runtime; object store/typed adapters`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S087](../evidence/SOURCES.ko.md#s087).

### AT14 — R14

**같이 바꿀 경계:** numeric effective plan/compiler + cast/queued/impact source binding + 개인 preview/평가 입력.

**필요 이유:** 지연 적중이 전역 SkillId 원형을 다시 읽어 유물 변형을 잃지 않게 한다. 유물 관리-only는 독립 단위로 분리 가능하다.

**코드/계약 위치:** `sim; skill_runtime/{cast,projectile,area}; range_preview; player_spawns; snapshot; stat_pipeline`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S091](../evidence/SOURCES.ko.md#s091) · [S030](../evidence/SOURCES.ko.md#s030).

### AT15 — R16

**같이 바꿀 경계:** reaction admission/visited 검사 + 모든 지연 payload ancestry + root producer.

**필요 이유:** timer나 새 projectile이 ancestry를 세탁하지 못하게 한다.

**코드/계약 위치:** `triggers; sim; types; projectile; area; recording; separate ChainContext`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S026](../evidence/SOURCES.ko.md#s026) · [S018](../evidence/SOURCES.ko.md#s018) · [S027](../evidence/SOURCES.ko.md#s027) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072).

### AT16 — R19

**같이 바꿀 경계:** 복수 파편/선택 schema + 보상 plan/commit + 참조/중복 검증 + 정상 grant/save.

**필요 이유:** 현 단일 source.fragment_id 가정 중 실제 변경한 부분은 모든 reader/writer에 같은 의미여야 한다.

**코드/계약 위치:** `pve_data; encounter_progression; skill_fragment_data; reward; validation; world snapshot/save`.

최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.

근거: [S073](../evidence/SOURCES.ko.md#s073) · [S074](../evidence/SOURCES.ko.md#s074) · [S042](../evidence/SOURCES.ko.md#s042) · [S036](../evidence/SOURCES.ko.md#s036) · [S067](../evidence/SOURCES.ko.md#s067) · [S038](../evidence/SOURCES.ko.md#s038) · [S091](../evidence/SOURCES.ko.md#s091).

## 5. 순환과 병렬화

원래 최소 Plan의 compiler/catalog/runtime 상호 의존은 **기존 AG-CAMEC-01**로 유지한다. 새 사본/저장의 writer-reader 결합과 효과 계획/지연 소비자 결합도 component 동시 전환으로 해결한다. executor stub·임시 성공으로 순환을 없애지 않는다.

R23 최적화 뒤 R22의 영향받은 독립 제작 증거를 갱신하는 관계는 조건부 **재검증**이지 구현 선행 순환이 아니다. JSON에 `conditional_revalidation=true`를 별도 표시했다. 그 밖의 기능/필수 검증 관계가 기본 순서와 일치하는지는 로컬 정합성 검사 결과를 참조한다.

기본은 순차 진행이다. 예를 들어 R08의 기본 호출은 R06/R07과 기능상 독립일 수 있지만, `ability.rs`/`sim.rs`/`types.rs`/RON fixture와 compile surface를 함께 바꿀 수 있다. R06과 R07도 profile·snapshot·save 경계가 겹친다. 실제 병렬화는 파일·schema·state owner·시험 디렉터리·병합/통합 검증 owner가 분리됐을 때만 별도로 허용한다. 화살표 부재만으로 병렬 가능을 선언하지 않는다.
