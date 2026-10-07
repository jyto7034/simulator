# 콘텐츠 제작 기반 — 개정 로드맵

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

## 1. 결론과 범위

**기존 kernel/게시/상태/보스 graph를 재사용하는 부분 재설계**를 권고한다. 전체 재작성의 필수 근거는 발견하지 않았다. 목표는 제3자가 지원하는 명령·행동·조건·효과·규칙을 RON으로 조합해 보스·장비/무기·파편·유물을 제작하고 등록·교체·등장·획득까지 연결하는 것이다. 새로운 공통 엔진 기능의 추가는 가능하며 모든 미래 기믹을 무한히 표현해야 한다는 요구가 아니다.

RON 조합기 UI, 외부 스크립트/플러그인 실행, game_server·Unity/client 구현은 제외한다. 작성 정보·검증·진단·설명·headless 시험 포트는 포함한다. 코어의 실제 소비/표시 경계는 기록하되 외부 API 불일치를 전체 코어 재작성의 근거로 사용하지 않는다.

## 2. 기존 성과 인수

CA00 요구/COV 조사와 CA01 최소 계약의 P2G-60 Handoff를 그대로 인수한다. 현재 다음 카드는 NEXT-0009이나 이번 요청은 실행 요청이 아니다. **현재 GOAL-CAMEC-01은 첫 구현 단위 R01로 유지하는 A 권고**다. 정식 Plan·Goal·Handoff 원본 및 current 선택을 바꾸지 않는다.

## 3. 하나의 권장 기본 순서

| 순서 | 작업 단위 | 기능 선행 | 검증 선행 |
| --- | --- | --- | --- |
| R00 | [기존 M0 인수·전체 대응 기준선](goals/R00.ko.md) | — | — |
| R01 | [효과 주소·태그·typed property 최소 전환 — 현 GOAL 유지](goals/R01.ko.md) | — | R00 |
| R02 | [외부 다중 팩·검증 세대·새 런/저장 결속](goals/R02.ko.md) | R01 | — |
| R03 | [공통 작성 API·CLI·설명·headless 최소 포트](goals/R03.ko.md) | R01, R02 | — |
| R04 | [사건·실제 결과·출처·현재 전달체의 ActionHandle](goals/R04.ko.md) | R01 | — |
| R05 | [타입화 자원·owner×lifetime·원자적 소비](goals/R05.ko.md) | R04 | — |
| R06 | [ValueExpr·집계·Attack/SpellPower와 무기 성장](goals/R06.ko.md) | R01, R04, R05 | — |
| R07 | [파편 사본·개화 선택·정상 획득/장착/저장 동시 전환](goals/R07.ko.md) | R01, R05 | — |
| R08 | [보스 최소 스킬 요청·완료·실패·취소·방향 adapter](goals/R08.ko.md) | R01, R04 | — |
| R09 | [파편 침식·강제 시계·직원 대가/Breakdown 동시 전환](goals/R09.ko.md) | R04, R05, R07 | — |
| R10 | [대상·전달·구조화 Flow·유한 반복](goals/R10.ko.md) | R01, R04, R05, R06 | — |
| R11 | [가역 modifier·membership·channel·기본 공격 교체](goals/R11.ko.md) | R05, R06, R10 | — |
| R12 | [개인/공유 보호막·생존·제어·HP 비용 중재](goals/R12.ko.md) | R04, R05, R06, R11 | — |
| R13 | [소환 actor·정지 trap·field object·진화](goals/R13.ko.md) | R04, R05, R07, R10, R12 | — |
| R14 | [유물 관리 + 최소 수치 합성의 실제 실행 연결](goals/R14.ko.md) | R01, R05, R07, R10 | — |
| R15 | [유한 활성 closure·생성 효과·구조 recipe](goals/R15.ko.md) | R11, R12, R14 | — |
| R16 | [반응형 연쇄·branch-local 재방문·비동기 인과](goals/R16.ko.md) | R04, R05, R14, R15 | — |
| R17 | [연결망·원천별 감염·공간 집계/일제 소비](goals/R17.ko.md) | R06, R13, R16 | — |
| R18 | [보스 형태·조건·선택·전장 신호/다중 actor 통합](goals/R18.ko.md) | R08, R11, R13 | — |
| R19 | [등록·획득 선택·정책 profile·전투력 최종 입력](goals/R19.ko.md) | R02, R07, R14, R15, R18 | — |
| R20 | [런 결과·후퇴·재시도·저장·재접속 교차 검증](goals/R20.ko.md) | — | R09, R17, R18, R19 |
| R21 | [대표 공식 최신 RON C01–C07·전체 표현력 대조](goals/R21.ko.md) | R03 | R15, R16, R17, R18, R20 |
| R22 | [엔진 동결 후 제3자 data-only 재조합 인수](goals/R22.ko.md) | — | R21 |
| R23 | [결정론·예산·회귀·측정 기반 성능 닫힘](goals/R23.ko.md) | — | R22 |
| R24 | [독립 완료 감사·제작 기반 인계](goals/R24.ko.md) | — | R23 |

R00은 새 게임 구현이 아니라 기존 인수다. R01–R24는 위 순서대로 진행하는 기본 권고이며 모든 인접 관계가 기능상 필수는 아니다. R08 보스 최소 호출과 R09 침식을 큰 후반 기능 묶음에서 먼저 분리했다. R14는 최소 수치 유물을 실제 지연 전달까지 연결하고, R15/R16/R17에서 구조 합성·연쇄·관계를 차례로 확장한다.

## 4. 단계마다 독립적으로 유효해야 하는 것

새 기능을 추가하는 owner는 schema/descriptor/parser/validator/compiler/실제 consumer/정상 및 거부 fixture를 함께 소유한다. 새 state는 필요한 snapshot/checkpoint/entry/결과/취소/실패 복원도 같은 시점에 도입한다. R20은 저장의 최초 구현이 아니고, R23은 예산의 최초 도입이 아니다. 기존 A03/A07/CA19/CA22가 이미 밝힌 의도를 현재 코드 경계에 구체화했다.

동일 최신 경로 하나로 전환하되, 기존 kernel로 낮추는 typed adapter를 레거시 shim으로 오해하지 않는다. 최소 Goal에서는 기존 V4 reader/V5 writer를 보존한다. 외부 팩이나 새 사본 저장으로 실제 저장 경계를 전환할 때 해당 owner가 latest-format 전환과 승인된 손실/호환 조건을 소유한다. 이번 연구는 원본 파일 삭제/이동/변환 권한을 행사하지 않는다.

## 5. 정책 대기 때의 제한된 분기

**조건:** PG03/PG09의 침식 판정시점·threshold·authored duration/Breakdown 계약 미해결.

유예: R09. 진행 가능: R10, R11, R12, R13, R14, R15, R16, R17, R18, R19. 전체 완료 차단: R20, R21, R22, R23, R24.

R09이 구현되지 않았으면 다른 단위의 침식 회귀는 미적용 사유로 기록하되 나중 R09이 신규 모든 subscriber까지 인수해야 함.

**조건:** PG01/02/04/06 등 특정 recipe/합성 정책 미해결.

유예: 해당 family/recipe. 진행 가능: 이미 확정된 primitive와 독립 family. 전체 완료 차단: 해당 단위의 전체 완료, 그 기능이 필수인 대표/독립인수.

unsupported/정책대기를 성공이나 0/무제한/default로 바꾸지 않는다.

**조건:** 관리만 우선 필요한 독립 구현 요청.

R14를 관리-only와 실제 numeric plan 실행으로 분리 가능.

앞 단위 완료 조건은 관리·정상 grant/save만이어야 하며 효과 합성을 이미 지원한다고 표시하지 않는다. 기본 권고는 R14 통합 유지.

## 6. 검증과 인계

T00–T29, C01–C07, K01–K10, PG00–PG09를 유지하며 [검증 owner와 입력](ACCEPTANCE.ko.md) 및 [전체 대응](COVERAGE.ko.md)를 함께 적용한다. 각 단계의 actual execution receipt와 R22 독립 작성/R24 독립 감사를 요구한다. 이번 정적 조사·문서 검사로 이를 통과했다고 하지 않는다.

## 7. 적용 지위

이 로드맵은 후보 권고이며 정식 P2G Plan과 동시에 확정 실행 순서가 되지 않는다. 지금 formal target은 GOAL-CAMEC-01 하나다. 후속 범위는 해당 P2G owner가 이 근거를 소비해 필요할 때 새 formal 산출물로 만든다. 적용 절차는 [별도 적용 안내](../APPLY.ko.md)를 따른다.

근거: [S001](../evidence/SOURCES.ko.md#s001) · [S005](../evidence/SOURCES.ko.md#s005) · [S006](../evidence/SOURCES.ko.md#s006) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S080](../evidence/SOURCES.ko.md#s080) · [S082](../evidence/SOURCES.ko.md#s082) · [S085](../evidence/SOURCES.ko.md#s085) · [S086](../evidence/SOURCES.ko.md#s086) · [S088](../evidence/SOURCES.ko.md#s088) · [S089](../evidence/SOURCES.ko.md#s089).
