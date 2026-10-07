# A01 — 시스템 구성·단계별 책임

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

원래 논리 설계와 승인 목표는 유지하고 새 R owner/현재 코드 전환 경계를 반영했다. 새 문법·타입·포트 이름은 실제 구현 존재의 증거가 아니다.

## 1. 제작 입력과 실행 도메인

외부 source selection → snapshot/pack resolve → typed validation/compile → ValidatedGeneration → 새 런 pin → immutable definition/개인 effective plan → 각 도메인의 공통 executor 흐름을 유지한다. Flow는 일반 effects를 연결하고, Boss graph는 상태/task/progress를, World/Economy는 등장/획득/장착/결과를 소유한다. 하나의 무제한 VM이 모든 도메인 authority를 대신하지 않는다.

## 2. 현재 기반과 목표 변화

현재 `load_live_sources`의 검증 후 공개, GameCore/BattleCore의 데이터 참조, world clone/commit, skill carrier, boss compiled graph와 SpawnKernel은 재사용한다. R01은 현 경로의 metadata, R02는 외부 게시/new-run/save, R04는 사실/handle/source, R05 이후는 새로운 상태/표현력이다. R14가 개인 plan을 도입할 때 모든 지연 consumer를 같은 source binding으로 전환한다.

## 3. Identity와 권한

definition address, compiled index, persistent copy, employee, runtime unit, cast/action, strike, damage segment, delivery/activation/chain을 구별한다. immutable template은 개별 직원/사본 UUID를 품지 않고 binding을 받는다. World result authority, wave/victory, boss task, object lifetime, acquisition은 별도다.

## 4. 재사용의 조건

typed request를 기존 damage/heal/spatial kernel에 낮추는 adapter는 유효하다. 구형 parser/신형 parser 또는 서로 다른 의미의 이중 executor를 영구 병행시키는 것을 기본 해법으로 하지 않는다. 현재 최소 Goal의 old save reader 보존은 새 전투 의미 이중 실행과 다르다.

## 5. 관찰·검증

R03 공개 포트와 각 R의 positive/negative actual RON, source-aware trace를 연결한다. R21 대표 실행과 전체 COV 표현력, R22 독립 제작, R23 측정/회귀, R24 독립 감사를 구별한다. 미래 공통 기능을 확장할 수 있지만 현재 미지원 op는 publication error여야 한다.

원문 설계 근거: [S080](../../evidence/SOURCES.ko.md#s080). 현재 코드 근거와 각 단계의 상세 동시 전환은 [DEPENDENCIES](../DEPENDENCIES.ko.md) 및 [조사 결과](../../report/REVIEW.ko.md)를 따른다.
