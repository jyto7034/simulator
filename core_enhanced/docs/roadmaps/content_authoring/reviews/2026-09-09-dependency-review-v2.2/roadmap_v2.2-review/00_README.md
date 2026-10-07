# 개정 로드맵 파일 색인

**2.2-review.1 · 연구 권고 / 정식 Goal Set 아님 / 저장소 미적용**

[핵심 보고서](../report/REVIEW.ko.md)에서 결론과 한계를 먼저 읽는다. 원래 CA/M는 요구 추적을 위해 남기고 R 번호는 권장 구현 단위를 식별한다.

| 문서 | 역할 |
| --- | --- |
| [ROADMAP](ROADMAP.ko.md) | 하나의 기본 순서와 중간 상태 |
| [DEPENDENCIES](DEPENDENCIES.ko.md) | 49개 기존 관계 disposition·76개 새 관계·16개 동시 전환 |
| [GOALS.json](GOALS.json) | 카드·표의 연구용 구조 데이터 |
| [CONTRACTS](CONTRACTS.ko.md) | K01–K10와 처음/확장 owner |
| [POLICY_GATES](POLICY_GATES.ko.md) | PG00–PG09·승인 의미·미정 조건 |
| [ACCEPTANCE](ACCEPTANCE.ko.md) | T00–T29·주/보조 owner·시험 입력·기대 결과 |
| [COVERAGE](COVERAGE.ko.md) | 대표 C01–C07와 전체 표현력 대조의 분리 |
| [HANDOFF](HANDOFF.ko.md) | 기존 M0를 보존한 후속 인수 안내 |
| [적용 안내](../APPLY.ko.md) | 원본을 덮어쓰지 않는 별도 배치 |

## 작업 카드

| ID | 실제 개정 카드 |
| --- | --- |
| R00 | [기존 M0 인수·전체 대응 기준선](goals/R00.ko.md) |
| R01 | [효과 주소·태그·typed property 최소 전환 — 현 GOAL 유지](goals/R01.ko.md) |
| R02 | [외부 다중 팩·검증 세대·새 런/저장 결속](goals/R02.ko.md) |
| R03 | [공통 작성 API·CLI·설명·headless 최소 포트](goals/R03.ko.md) |
| R04 | [사건·실제 결과·출처·현재 전달체의 ActionHandle](goals/R04.ko.md) |
| R05 | [타입화 자원·owner×lifetime·원자적 소비](goals/R05.ko.md) |
| R06 | [ValueExpr·집계·Attack/SpellPower와 무기 성장](goals/R06.ko.md) |
| R07 | [파편 사본·개화 선택·정상 획득/장착/저장 동시 전환](goals/R07.ko.md) |
| R08 | [보스 최소 스킬 요청·완료·실패·취소·방향 adapter](goals/R08.ko.md) |
| R09 | [파편 침식·강제 시계·직원 대가/Breakdown 동시 전환](goals/R09.ko.md) |
| R10 | [대상·전달·구조화 Flow·유한 반복](goals/R10.ko.md) |
| R11 | [가역 modifier·membership·channel·기본 공격 교체](goals/R11.ko.md) |
| R12 | [개인/공유 보호막·생존·제어·HP 비용 중재](goals/R12.ko.md) |
| R13 | [소환 actor·정지 trap·field object·진화](goals/R13.ko.md) |
| R14 | [유물 관리 + 최소 수치 합성의 실제 실행 연결](goals/R14.ko.md) |
| R15 | [유한 활성 closure·생성 효과·구조 recipe](goals/R15.ko.md) |
| R16 | [반응형 연쇄·branch-local 재방문·비동기 인과](goals/R16.ko.md) |
| R17 | [연결망·원천별 감염·공간 집계/일제 소비](goals/R17.ko.md) |
| R18 | [보스 형태·조건·선택·전장 신호/다중 actor 통합](goals/R18.ko.md) |
| R19 | [등록·획득 선택·정책 profile·전투력 최종 입력](goals/R19.ko.md) |
| R20 | [런 결과·후퇴·재시도·저장·재접속 교차 검증](goals/R20.ko.md) |
| R21 | [대표 공식 최신 RON C01–C07·전체 표현력 대조](goals/R21.ko.md) |
| R22 | [엔진 동결 후 제3자 data-only 재조합 인수](goals/R22.ko.md) |
| R23 | [결정론·예산·회귀·측정 기반 성능 닫힘](goals/R23.ko.md) |
| R24 | [독립 완료 감사·제작 기반 인계](goals/R24.ko.md) |

## 아키텍처 문서

- [01_SYSTEM.ko.md](architecture/01_SYSTEM.ko.md)
- [02_AUTHORING_AND_COMPILER.ko.md](architecture/02_AUTHORING_AND_COMPILER.ko.md)
- [03_RUNTIME.ko.md](architecture/03_RUNTIME.ko.md)
- [04_COMPOSITION.ko.md](architecture/04_COMPOSITION.ko.md)
- [05_BOSS_AUTHORING.ko.md](architecture/05_BOSS_AUTHORING.ko.md)
- [06_PACKS_AND_TOOLS.ko.md](architecture/06_PACKS_AND_TOOLS.ko.md)
- [07_CUTOVER_AND_REUSE.ko.md](architecture/07_CUTOVER_AND_REUSE.ko.md)
- [08_WORKED_EXAMPLES.ko.md](architecture/08_WORKED_EXAMPLES.ko.md)

## 검사 경계

[문서 검사 결과](../evidence/DOCUMENT_VALIDATION.ko.md)는 이 배포본 내부의 참조·ID·순서·소유 관계만 확인한다. 실제 parser/compiler/runtime의 구현·시험·독립 감사는 수행하지 않았다. [현재 source](../evidence/SOURCES.ko.md)와 [한계](../report/LIMITATIONS.ko.md)를 함께 인수한다.
