# Simulator 제작 기반 로드맵 — v2.1 패키지

2026-09-09 · 문서 개정만 수행. 프로젝트 코드·RON 수정, 빌드·테스트·벤치마크는 미실행. 권장 아키텍처와 구현 AI가 완성할 상세 설계를 구분한다.

## 무엇부터 읽는가

일상 작업은 [ROADMAP_STATUS.md](../ROADMAP_STATUS.md)에서 현재 진행과 다음 작업을 인수한다. 이 로드맵 묶음을 처음 이해할 때는 [ROADMAP](ROADMAP.ko.md)을 읽고, 선택 작업에는 [DESIGN_WORKFLOW](DESIGN_WORKFLOW.ko.md)와 해당 CA 카드를 사용한다. 기존 진행이 없다고 확인된 경우 첫 후보는 [CA00](goals/CA00.md)이며, 번호만으로 재착수하지 않는다. 기존 v1/v2의 실행 목적·검증·보스 제작·효과 합성·상태 계약은 유지한다.

| 문서 | 내용 |
| --- | --- |
| [ROADMAP](ROADMAP.ko.md) | 전체 범위·권장 구조·9단계·24개 작업 |
| [DESIGN_WORKFLOW](DESIGN_WORKFLOW.ko.md) | P/R/L·D0/D1/D2/DG/I/V/A와 변경 권한 |
| [POLICY_GATES](POLICY_GATES.ko.md) | U01–U13과 PG00–PG09 |
| [COVERAGE](COVERAGE.ko.md) | 15축·전체 원문 추적·대표 C01–C07·보스 coverage |
| [ACCEPTANCE](ACCEPTANCE.ko.md) | 기존 T00–T24와 v2 T25–T29·실제 검증 기록 |
| [CONTRACTS](CONTRACTS.ko.md) | 공유 K 계약의 owner·소비자 |
| [DEPENDENCIES](DEPENDENCIES.ko.md) / [GOALS.json](GOALS.json) | 계획상 선행 후보와 구조화된 작업 카드 |
| [AI_HANDOFF](AI_HANDOFF.ko.md) | 첫 조사와 후속 구현 AI에 전달할 요청 예시 |
| [SOURCES](SOURCES.ko.md) / [CHANGELOG](CHANGELOG.ko.md) | 이번 확인·과거 source 입구·개정 내용 |

## 권장 설계 문서

| 문서 | 범위 |
| --- | --- |
| [A01](architecture/01_SYSTEM.ko.md) | 논리 모듈·의존 방향·K01–K10 계약과 대안 |
| [A02](architecture/02_AUTHORING_AND_COMPILER.ko.md) | typed AST·중간 표현·Effect/ActionSite·검증/compile |
| [A03](architecture/03_RUNTIME.ko.md) | identity·상태 수명·ActionHandle·receipt·원자성·연쇄 |
| [A04](architecture/04_COMPOSITION.ko.md) | 효과 선택·grant closure·변형·충돌·활성/휴면 |
| [A05](architecture/05_BOSS_AUTHORING.ko.md) | 커뮤니티 보스 패턴·기믹 제작 규격과 런타임 포트 |
| [A06](architecture/06_PACKS_AND_TOOLS.ko.md) | 외부 팩 합성·세대 pin·저장·공통 도구 |
| [A07](architecture/07_CUTOVER_AND_REUSE.ko.md) | 기존 코드 재사용·교체·최신 전환 지도 |
| [A08](architecture/08_WORKED_EXAMPLES.ko.md) | RON 의사자료와 compile→runtime 예제 E01–E06 |

새 타입·API·RON 예제는 제안이고 아직 current parser에서 실행되지 않았다.

## 작업별 카드

| 작업 | 단계 | 계획상 선행 후보 |
| --- | --- | --- |
| [CA00 — 기준 상태·전체 요구사항·정책 추적표](goals/CA00.md) | M0 | 없음 |
| [CA01 — 작성 단위·타입·실행 지원 계약](goals/CA01.md) | M0 | CA00 |
| [CA02 — 최신 schema 전환·외부 다중 팩 게시](goals/CA02.md) | M1 | CA01 |
| [CA03 — 공통 검증·설명·headless 시험 실행 포트](goals/CA03.md) | M1 | CA02 |
| [CA04 — 자원·상태 소유와 수명 kernel](goals/CA04.md) | M2 | CA03 |
| [CA05 — 계산식·집계·스탯 성장](goals/CA05.md) | M2 | CA04 |
| [CA06 — 의미 있는 전투 사건·피해 해결 결과](goals/CA06.md) | M2 | CA04, CA05 |
| [CA07 — 대상·전달·단계·분기·반복 실행](goals/CA07.md) | M2 | CA05, CA06 |
| [CA08 — 가역 modifier·오라·집중·기본 공격 교체](goals/CA08.md) | M3 | CA07 |
| [CA09 — 공유 보호막·생존·방어 중재·제어](goals/CA09.md) | M3 | CA06, CA08 |
| [CA10 — 파편 사본·개화·형태와 무기 성장 연결](goals/CA10.md) | M3 | CA05, CA08 |
| [CA11 — 유물 소유·장착·회수·분쇄](goals/CA11.md) | M4 | CA04, CA10 |
| [CA12 — 효과 선택·변형·유효 효과 구성](goals/CA12.md) | M4 | CA09, CA10, CA11 |
| [CA13 — 연쇄 실행·경로별 재방문 차단](goals/CA13.md) | M4 | CA06, CA12 |
| [CA14 — 설치물·장애물·소환의 구분된 실행](goals/CA14.md) | M5 | CA04, CA07, CA09, CA10 |
| [CA15 — 연결망·상태 전파·공간 집계](goals/CA15.md) | M5 | CA13, CA14 |
| [CA16 — 보스 공통 행동·패턴·전장 사건 연결](goals/CA16.md) | M5 | CA07, CA08, CA09, CA14 |
| [CA17 — 등록·획득·공식 규칙 교체·전투력 연결](goals/CA17.md) | M6 | CA10, CA11, CA12, CA16 |
| [CA18 — 파편 침식과 직원 대가의 특수 경계](goals/CA18.md) | M6 | CA06, CA09, CA10, CA13, CA16 |
| [CA19 — 결과·후퇴·저장·재접속의 통합 정합성](goals/CA19.md) | M6 | CA15, CA17, CA18 |
| [CA20 — 대표 공식 콘텐츠의 실제 RON 이식](goals/CA20.md) | M7 | CA15, CA16, CA19 |
| [CA21 — 제3자 관점의 재조합 인수](goals/CA21.md) | M7 | CA20 |
| [CA22 — 실행량·결정론·회귀·성능 검증](goals/CA22.md) | M8 | CA21 |
| [CA23 — 독립 완료 감사와 제작 기반 인계](goals/CA23.md) | M8 | CA22 |

## 템플릿과 기계 검증

[상세 설계](templates/DESIGN.template.ko.md), [설계 준비 점검](templates/DESIGN_REVIEW.template.ko.md), [설계 변경](templates/DESIGN_DELTA.template.ko.md)을 현재 저장소 workflow의 파일 또는 절에 맞게 사용한다. 빈 템플릿은 실제 실행/승인 기록이 아니다.

`DOCUMENT_CHECKS.json`은 문서 링크·ID·DAG·패키지 검사 결과다. Rust/RON/전투 검증을 의미하지 않는다. `PACKAGE_MANIFEST.json`은 파일 크기와 SHA-256을 제공하며 자기 자신은 해시 목록에서 제외한다.

단일 통합 Markdown은 같은 본문에서 생성한다. JSON과 checksum 파일은 ZIP의 기계용 부속이며 두 형태가 다른 개발 계획을 갖지 않도록 한다. 프로젝트 저장소에는 이 작성 작업으로 어떤 문서도 생성하지 않았다.

## v2.1 — 순서는 실행 지시가 아니라 검토할 후보

실제 첫 진입점은 함께 제공한 [ROADMAP_STATUS.md](../ROADMAP_STATUS.md)다. 이 파일의 목록이나 `GOALS.json`을 번호순 자동 실행 스케줄로 사용하지 않는다. [의존성 문서](DEPENDENCIES.ko.md)는 후보와 코드 근거·Phase Plan·실행 준비도의 차이를 정리한다. v2.1에서 후보 edge는 변경하지 않았고 실제 코드 의존성 전수 검토도 수행하지 않았다.

`goals/CAxx.md`는 정식 P2G Goal member가 아닌 작업 후보 카드다. 사용 중인 P2G r2와 GEX가 실제 단계·권한·인수를 소유한다. 문서 정합성 검사 PASS는 게임 테스트나 의미상 순서 검증 PASS가 아니다.

`GOALS.json.execution_state`의 `not_executed`는 로드맵 작성에서 해당 작업을 실행하지 않았다는 뜻이다. `execution_state_scope`로 이 범위를 표시하며, 현재 저장소가 미착수라는 판정이 아니다. 실제 현황은 ROADMAP_STATUS에서 근거를 인수하기 전까지 미확인이다.
