# 목표·대표 사례·전체 표현력 대응

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

전체 M/CA owner 대응은 [전체 대응표](../report/M_CA_MAPPING.ko.md)를 따른다. 원본 CA00 COV는 보존하며 이 문서의 대표 사례로 대체하지 않는다. **공식 최신 RON 이식**과 **전체 합의 구조의 표현력 대조**는 R21의 서로 다른 제출물이다.

## C01 — 따뜻한 마음의 나무꾼·깊은 쐐기

기능 owner: [R04](goals/R04.ko.md), [R06](goals/R06.ko.md), [R07](goals/R07.ko.md), [R10](goals/R10.ko.md), [R14](goals/R14.ko.md), [R16](goals/R16.ko.md). 공식 RON/통합 owner: [R21](goals/R21.ko.md).

보존할 구조: 3타 순서/마지막 강화, 이번 n번째 직접 타격 보너스. segment 수·대상 수와 strike 수를 혼동하지 않는다.

조건: n타 판정/지속기간/실제 수치 중 미정은 PG03/06

현재: 이번 실제 RON 이식/게임 실행 미수행.

## C02 — 풍운승려 파편·Physical 염주 Staff·장비

기능 owner: [R06](goals/R06.ko.md), [R07](goals/R07.ko.md), [R10](goals/R10.ko.md), [R14](goals/R14.ko.md), [R15](goals/R15.ko.md). 공식 RON/통합 owner: [R21](goals/R21.ko.md).

보존할 구조: 범위 bite 뒤 조건부 자기회복 1회. damage footprint만 변형하고 heal/source/횟수 유지. Staff≠Magic/SpellPower 자동분류.

조건: 유효 적중의 exact 의미 PG03; example HP 손계산은 test-only

현재: 이번 실제 RON 이식/게임 실행 미수행.

## C03 — 아무 것도 없는 보스·Hello/Goodbye

기능 owner: [R08](goals/R08.ko.md), [R10](goals/R10.ko.md), [R11](goals/R11.ko.md), [R18](goals/R18.ko.md). 공식 RON/통합 owner: [R21](goals/R21.ko.md).

보존할 구조: 보스 HP/actor identity 연속·변신 중 피해·임계치 skip. 플레이어 Hello는 기본공격 overlay, 재입력 Goodbye와 자연 종료 분리.

조건: 개화 공란/동시임계치 세부를 임의완성하지 않는다.

현재: 이번 실제 RON 이식/게임 실행 미수행.

## C04 — 포큐버스 파편·공유 보호막·단일 전환

기능 owner: [R05](goals/R05.ko.md), [R07](goals/R07.ko.md), [R12](goals/R12.ko.md), [R14](goals/R14.ko.md), [R15](goals/R15.ko.md). 공식 RON/통합 owner: [R21](goals/R21.ko.md).

보존할 구조: 한 원천 pool·membership·초기량 한번. 단일 recipient 일반2S recipe는 owner/수명 보존.

조건: S공식/범위·개인공유 순서·침식 비교안 미정. 600/400/350은 감쇄후 test입력.

현재: 이번 실제 RON 이식/게임 실행 미수행.

## C05 — 이름이 없는 태아 집중·단일 전환

기능 owner: [R06](goals/R06.ko.md), [R11](goals/R11.ko.md), [R14](goals/R14.ko.md), [R15](goals/R15.ko.md). 공식 RON/통합 owner: [R21](goals/R21.ko.md).

보존할 구조: regular basic damage/speed만 수혜자에 적용; 채널/lease 유지. 상태별 두property의 교체값을 무조건×2로 바꾸지 않는다.

조건: 최대시간·저지·침식 대가 미정은 gate

현재: 이번 실제 RON 이식/게임 실행 미수행.

## C06 — 래티씨아 생성·진화·붉은 실타래

기능 owner: [R07](goals/R07.ko.md), [R13](goals/R13.ko.md), [R15](goals/R15.ko.md), [R16](goals/R16.ko.md), [R17](goals/R17.ko.md). 공식 RON/통합 owner: [R21](goals/R21.ko.md).

보존할 구조: 내 source small→big, node진화/삭제/consume와 관계 invalidation, 접근/선통과 반응.

조건: nodeID 유지/교체는 기술선택; 전체/유효node소비·linecrossing상세는 별도조건

현재: 이번 실제 RON 이식/게임 실행 미수행.

## C07 — 포큐버스 보스·전장 작업 억제

기능 owner: [R08](goals/R08.ko.md), [R11](goals/R11.ko.md), [R13](goals/R13.ko.md), [R18](goals/R18.ko.md). 공식 RON/통합 owner: [R21](goals/R21.ko.md).

보존할 구조: 잠복/출현·거리조건·Route·wave지원·작업완료→typed signal→억제. player sharedshield 사례로 대신하지 않는다.

조건: 거리/수치·작업/전이 정책의 미정만 gate

현재: 이번 실제 RON 이식/게임 실행 미수행.

## 대표 사례 외 구조

고기벽의 segment HP/HP비용, 포자의 원천별 감염/사망 전파/기폭, 흑조의 감쇄 후 저장과 패링, 나비의 타일별 집계, 루돌타의 유한 반복/확률, 직원 Run 성장과 파편 사본 Run 자원을 별도로 추적한다. 원문 COV의 owner_ca에서 새 R로 이어지는 경로를 없애지 않는다. 보스별 actor/blackboard/parameter/signal 요구와 도구형 9종의 scope 후보는 대표 플레이어 파편에 흡수하지 않는다.

## 분해 틀

15개 분해 축은 원문 검토 틀로 유지한다. 단계·분기·병행을 먼저 읽고 개별 효과를 분리한 뒤 대상/전달/범위/계산/자원/사건/수명 등 해당 하위 축을 적용한다. 이 축들을 15개 schema 필드나 서로 독립된 15개 엔진 기능으로 기계적으로 옮기지 않는다. 소환물·변신·공유 자원의 별도 책임도 추적한다. exact 원문 분류·COV 행은 S006/S076/S078을 참조한다.

## 독립 제작 판정

R22에서는 엔진 revision을 동결한 뒤 다른 작성자가 새 ID/새 조합의 팩을 작성한다. 보스·장비/무기·파편·유물 도메인, cross-pack 참조/명시 교체, 둘 이상 스킬과 소환 또는 작업 신호·실패/취소 route를 포함한다. 이름·HP만 바꾼 복사는 부족하다. 공통 기능이 모자라 engine patch가 필요하면 그 기능을 보완한 뒤 재동결/해당 인수를 다시 수행한다. 이번에는 독립 작성자나 실행 receipt가 없다.

근거: [S006](../evidence/SOURCES.ko.md#s006) · [S054](../evidence/SOURCES.ko.md#s054) · [S057](../evidence/SOURCES.ko.md#s057) · [S058](../evidence/SOURCES.ko.md#s058) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S078](../evidence/SOURCES.ko.md#s078) · [S084](../evidence/SOURCES.ko.md#s084) · [S087](../evidence/SOURCES.ko.md#s087) · [S096](../evidence/SOURCES.ko.md#s096).
