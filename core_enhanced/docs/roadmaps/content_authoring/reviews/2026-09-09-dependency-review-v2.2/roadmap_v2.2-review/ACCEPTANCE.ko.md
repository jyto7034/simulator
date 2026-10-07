# 검증 owner·입력·기대 결과

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

**이 파일은 시험 계획이다. 이번 게임 시험은 실행하지 않았다.** 실제 local 문서 검사는 별도 `evidence/DOCUMENT_VALIDATION` 결과만 따른다. 과거 P2G validator PASS나 현재 test 함수의 존재를 이 T00–T29의 이번 PASS로 세지 않는다.

## T00 — 기준선·출처

기준 구현/검증 owner: [R00](goals/R00.ko.md). 동반 회귀 owner: 없음.

입력: 원문/P2G/HEAD·dirty/old49edge/CA-M 표.

기대: 모든 요구와 미정의 출처/owner가 있고 기존 완료·NEXT-0009 보존.

현재: **이번 게임 시험 미실행**.

## T01 — 최소 작성/실행 지원

기준 구현/검증 owner: [R01](goals/R01.ko.md). 동반 회귀 owner: 없음.

입력: 실제 latest RON 최소 Damage/Heal/Instant+미지원 property.

기대: loader→compiler→executor 연결·descriptor 일치·기존 의미 유지.

현재: **이번 게임 시험 미실행**.

## T02 — 외부 다중 팩

기준 구현/검증 owner: [R02](goals/R02.ko.md). 동반 회귀 owner: 없음.

입력: A/B pack, add/replace/remove, conflict/dependency cycle, invalid refs.

기대: 재컴파일 없는 선택·명시 충돌/지원 불가 거부·성공 candidate만 게시.

현재: **이번 게임 시험 미실행**.

## T03 — 런 세대 고정

기준 구현/검증 owner: [R02](goals/R02.ko.md). 동반 회귀 owner: 없음.

입력: A런 유지 중 B게시·B실패·같은ID 다른내용 save.

기대: A불변·B새런만·fingerprint mismatch 거부.

현재: **이번 게임 시험 미실행**.

## T04 — 공통 작성 포트

기준 구현/검증 owner: [R03](goals/R03.ko.md). 동반 회귀 owner: 없음.

입력: public API/CLI 동일입력·release target·reload failure·readonly query.

기대: 동일 진단/실제 nonzero 실행·old test session 유지·state/RNG/revision 불변.

현재: **이번 게임 시험 미실행**.

## T05 — 자원/수명

기준 구현/검증 owner: [R05](goals/R05.ko.md). 동반 회귀 owner: 없음.

입력: 같은 정의 두copy×deployment/attempt/run resource·비용 실패.

기대: 잔량/사용기록 분리·owner별 cleanup·거부 원자성.

현재: **이번 게임 시험 미실행**.

## T06 — 계산식·집계·성장

기준 구현/검증 owner: [R06](goals/R06.ko.md). 동반 회귀 owner: 없음.

입력: 단위 오류/overflow/0나눗셈·empty Max/Sum·weapon+SpellPower.

기대: typed거부·snapshot 일관·중복 성장 없음.

현재: **이번 게임 시험 미실행**.

## T07 — 사건·결과

기준 구현/검증 owner: [R04](goals/R04.ko.md). 동반 회귀 owner: 없음.

입력: basic vs skill·strike/segment·negative Heal·lethal pre-death.

기대: 기존 basic trigger와 새 facts 분리·actual量 정확·필수 death/wave 유지.

현재: **이번 게임 시험 미실행**.

## T08 — 대상·전달·Flow

기준 구현/검증 owner: [R10](goals/R10.ko.md). 동반 회귀 owner: 없음.

입력: Sequence/Branch/Parallel/Repeat·result 생산전읽기·nohit·delayed settle.

기대: 유한 continuation·분기/완료/취소·descriptor/executor 일치.

현재: **이번 게임 시험 미실행**.

## T09 — 가역 상태·집중·재입력

기준 구현/검증 owner: [R11](goals/R11.ko.md). 동반 회귀 owner: 없음.

입력: Hello expiry/reinput·channel 입출범위·서로다른 modifier 해제.

기대: 자기 lease만 정리·natural expiry Goodbye 없음·원형스탯 덮어복원 금지.

현재: **이번 게임 시험 미실행**.

## T10 — 보호막·방어·생존

기준 구현/검증 owner: [R12](goals/R12.ko.md). 동반 회귀 owner: 없음.

입력: 공유600에감쇄후A400/B350·개인/공유/HP1/비용실패.

기대: 공유 pool 한번생성·정책순서일치·HP/pool/receipt 원자성.

현재: **이번 게임 시험 미실행**.

## T11 — 사본·개화·형태

기준 구현/검증 owner: [R07](goals/R07.ko.md). 동반 회귀 owner: 없음.

입력: A/B동일definition 중 A개화·이전·form선택·save.

기대: B오염 없음·동일copy 이중장착 거부·정상지급/저장.

현재: **이번 게임 시험 미실행**.

## T12 — 유물 관리

기준 구현/검증 owner: [R14](goals/R14.ko.md). 동반 회귀 owner: 없음.

입력: 직원3slot·비호환 순서바꿈·다른직원·분쇄확인/취소·무기해제.

기대: 독립copy·휴면slot유지·무료장착·판매금지·정상회수/기존장비가루.

현재: **이번 게임 시험 미실행**.

## T13 — 합성·활성

기준 구현/검증 owner: [R14](goals/R14.ko.md), [R15](goals/R15.ko.md). 동반 회귀 owner: 없음.

입력: 동일 loadout permutation·numeric/structure recipe·새grant강화.

기대: 개인 immutable plan의 즉시/지연실행 일치·effect선택/전용ID 보존.

현재: **이번 게임 시험 미실행**.

## T14 — 경로별 연쇄

기준 구현/검증 owner: [R16](goals/R16.ko.md). 동반 회귀 owner: 없음.

입력: A1.e→B1.e→A1.e;A2;다른effect;독립root;sibling;delay;logeviction.

기대: 경로별만 차단·동일ancestry보존·timer/root세탁없음.

현재: **이번 게임 시험 미실행**.

## T15 — 소환·설치물

기준 구현/검증 owner: [R13](goals/R13.ko.md). 동반 회귀 owner: 없음.

입력: actor/trap/wall·small→big·비용/점유 실패·withdraw/TTL.

기대: 서로다른capability·비용/ID 원자성·stale event 무효.

현재: **이번 게임 시험 미실행**.

## T16 — 관계·감염·공간 집계

기준 구현/검증 owner: [R17](goals/R17.ko.md). 동반 회귀 owner: 없음.

입력: 붉은실/포자sourceA/B/기폭/타일grouping/진화삭제.

기대: 원천집합 snapshot/소비·재감염/재발동 방지·유한 전파.

현재: **이번 게임 시험 미실행**.

## T17 — 보스 제작

기준 구현/검증 owner: [R08](goals/R08.ko.md), [R18](goals/R18.ko.md). 동반 회귀 owner: 없음.

입력: boss skill request/failure/cancel/facing→HP형태/fieldsignal/multiactor.

기대: 실제task 실행·보스명특례없음·phase/wave/reward권한분리.

현재: **이번 게임 시험 미실행**.

## T18 — 등록·획득·평가

기준 구현/검증 owner: [R06](goals/R06.ko.md), [R07](goals/R07.ko.md), [R14](goals/R14.ko.md), [R19](goals/R19.ko.md). 동반 회귀 owner: 없음.

입력: 정상획득→선택→장착→전투·같은IDoverride·평가입력.

기대: 각도입단계정상route·복수보상transaction·미정weight미승격.

현재: **이번 게임 시험 미실행**.

## T19 — 침식

기준 구현/검증 owner: [R09](goals/R09.ko.md). 동반 회귀 owner: [R12](goals/R12.ko.md), [R16](goals/R16.ko.md), [R18](goals/R18.ko.md).

입력: Pause/실제시전·현재적/미래wave·lastkill·optionalproc·hostclock.

기대: 특수kill·정상진행·대가once·강제정지 mutation거부·Breakdown복원.

현재: **이번 게임 시험 미실행**.

## T20 — 저장·결과·복원

기준 구현/검증 owner: [R02](goals/R02.ko.md), [R05](goals/R05.ko.md), [R07](goals/R07.ko.md), [R09](goals/R09.ko.md), [R14](goals/R14.ko.md), [R20](goals/R20.ko.md). 동반 회귀 owner: [R19](goals/R19.ko.md).

입력: checkpoint/entry/attempt/retreat/retry/reconnect 복합.

기대: 새state가도입부터유효·contentmismatch거부·resultonce.

현재: **이번 게임 시험 미실행**.

## T21 — 대표 공식 RON/표현력

기준 구현/검증 owner: [R21](goals/R21.ko.md). 동반 회귀 owner: 없음.

입력: C01–C07 실제 latest 외부 공식RON+전체COV별도대조.

기대: builder/prototype만으로대체금지·미정/미지원명시.

현재: **이번 게임 시험 미실행**.

## T22 — 독립 data-only 인수

기준 구현/검증 owner: [R22](goals/R22.ko.md). 동반 회귀 owner: 없음.

입력: 다른작성자·새ID·4도메인·crosspack·boss2skill+mechanism·data-onlydiff.

기대: 엔진동결후실제재조합·추가enginecode필요시다시일반기능확장.

현재: **이번 게임 시험 미실행**.

## T23 — 안전·결정론·성능

기준 구현/검증 owner: [R02](goals/R02.ko.md), [R04](goals/R04.ko.md), [R10](goals/R10.ko.md), [R13](goals/R13.ko.md), [R16](goals/R16.ko.md), [R23](goals/R23.ko.md). 동반 회귀 owner: [R05](goals/R05.ko.md), [R17](goals/R17.ko.md).

입력: seed/input고정·컨테이너/pack순서·diag/logcaps·adversarialfanout.

기대: 기능별초기예산+현실부하측정·typedfault·gameplaycap창작없음.

현재: **이번 게임 시험 미실행**.

## T24 — 최종 독립 감사

기준 구현/검증 owner: [R24](goals/R24.ko.md). 동반 회귀 owner: 없음.

입력: 독립reviewer·현재실행receipt·전체coverage/미정/허용외diff.

기대: 남은필수누락없을때만완료;이보고서는대체아님.

현재: **이번 게임 시험 미실행**.

## T25 — 상세 설계·근거 정합성

기준 구현/검증 owner: [R00](goals/R00.ko.md), [R01](goals/R01.ko.md), [R03](goals/R03.ko.md), [R21](goals/R21.ko.md), [R24](goals/R24.ko.md). 동반 회귀 owner: 없음.

입력: 관찰/설계/실행구별·K별owner·소스/테스트/본문/JSON.

기대: 권장순서를필수로과장하지않고모든mapping동일.

현재: **이번 게임 시험 미실행**.

## T26 — 보스 action milestone

기준 구현/검증 owner: [R04](goals/R04.ko.md), [R08](goals/R08.ko.md), [R18](goals/R18.ko.md). 동반 회귀 owner: [R10](goals/R10.ko.md).

입력: t0admit/t100release/t300hit·stateexit50/150·stalehandle.

기대: Accepted≠Completed·waitmilestone·미방출cancel/방출후수명.

현재: **이번 게임 시험 미실행**.

## T27 — effect/carrier/source 경계

기준 구현/검증 owner: [R01](goals/R01.ko.md), [R06](goals/R06.ko.md), [R07](goals/R07.ko.md), [R14](goals/R14.ko.md), [R15](goals/R15.ko.md). 동반 회귀 owner: [R10](goals/R10.ko.md), [R11](goals/R11.ko.md), [R12](goals/R12.ko.md), [R13](goals/R13.ko.md).

입력: AoE피해+1selfheal·같은SkillId개인plan·carrier속성변형.

기대: effect와ActionSite분리·sharedtarget/delivery보존·sourcecopy캐시격리.

현재: **이번 게임 시험 미실행**.

## T28 — closure/연쇄 유한성

기준 구현/검증 owner: [R04](goals/R04.ko.md), [R15](goals/R15.ko.md), [R16](goals/R16.ko.md), [R17](goals/R17.ko.md). 동반 회귀 owner: 없음.

입력: positiveclosure/selfbootstrap/NOTcycle·fork/join/delayedancestry.

기대: compile유한성과runtime재방문분리·출처없는새root금지.

현재: **이번 게임 시험 미실행**.

## T29 — 게시·저장·fault 경계

기준 구현/검증 owner: [R02](goals/R02.ko.md), [R03](goals/R03.ko.md), [R05](goals/R05.ko.md), [R20](goals/R20.ko.md), [R23](goals/R23.ko.md). 동반 회귀 owner: [R09](goals/R09.ko.md), [R19](goals/R19.ko.md).

입력: candidate실패·testreload실패·mismatchsave·publicboundarybudgetfault.

기대: 기존런/세션보존·미게시state/queue/RNG/ID/log복원·이전commit전체rollback주장없음.

현재: **이번 게임 시험 미실행**.

## 기존 시험 seam의 구체 재사용

`core_enhanced/tests/skill_runtime_contract.rs::hit_gated_self_heal_and_buff_stack_repeat_attack_work_together`는 실제로 4step, opening hit 뒤 자기회복 1회, poison 3stack에 따른 barrage 3회를 assert한다. 이 시험의 원래 의미를 새 schema/compiler/consumer 경로에서 유지하는 것이 R01/R04/R10의 회귀 기반이다. 기존 private builder의 성공만으로 R03/R21 공개 외부 팩 인수를 대체하지 않는다. [S070](../evidence/SOURCES.ko.md#s070).

정상/거부/취소/종료·중복/stale/저장 왕복 시험은 각 owner가 실제 selector와 nonzero 건수·exit code·revision·fixture fingerprint를 기록해야 한다. 아직 존재하지 않는 새 test 이름이나 명령을 실행된 것으로 표시하지 않는다. 안전·성능은 대상 hardware/build/workload가 없는 숫자를 결과로 쓰지 않는다.
