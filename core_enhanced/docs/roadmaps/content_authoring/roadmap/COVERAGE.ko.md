# 요구사항·15축·대표 콘텐츠 추적표

계획 v2.1 · 2026-09-09. 이 문서는 계획상 책임 배정이다. `담당 CA가 있음`은 `구현 지원됨`이 아니며, 현재 결과는 전부 미실행이다. CA00에서 전체 콘텐츠 원문의 합의된 기계적 요구를 개별 COV 행으로 확장한다. 대표 콘텐츠 목록만으로 전체 표현력 coverage를 대신하지 않는다.

## 개별 요구 행의 필수 열

`COV ID / 원문 파일·heading·줄 / 합의·제안·미정 / 콘텐츠·형태 / 단계·분기·병행 관계 / 개별 효과 / 해당 축과 하위 요소 / 현재 지원·공백 근거 / 담당 CA / PG / 작성 가능한 조합 / 긍정·부정 시험 / 실제 검증 receipt / 남은 위험`.

최종 분류는 `지원·실행 확인`, `공통 기능 누락`, `사용자 정책 미정`, `명시적 비범위`, `해당 없음`으로 한다. 원문이 합의한 기계적 구조의 공통 기능 누락을 임의로 비범위로 돌려 완료해서는 안 된다. 발견된 누락이 기존 CA의 경계 밖이면 새 작업 후보와 의존성·완료 기준을 기록하고 실행 범위를 명시적으로 조정한다.

## 15축별 작업과 최소 검증

| 축 | 반드시 포함할 하위 요소 | 담당 CA | 검증 |
| --- | --- | --- | --- |
| 1 사용·조작 | 상시/자동/수동, 재입력, 상태별 모드와 자격 | CA07, CA08, CA10 | T08–T09, T11. 준비 focus와 지속 channel, 재입력 비용 분리 |
| 2 대상·선정 | 자기/아군/적/object, 단일/다수·수 제한, 필터·우선순위·재선정 | CA07, CA12, CA16 | T08, T13, T17. target과 footprint 독립 |
| 3 공간 | 시전/탐색/효과/연결 범위, 원점·방향·고정/추종 | CA07, CA08, CA14, CA15 | T08–T09, T15–T16. 원점 변경과 자기 회복 무관 |
| 4 전달·매개 | Instant/Projectile/Area, 관통, 부착, trap/object/actor | CA07, CA08, CA14 | T08–T09, T15. 소비자별 지원과 비지원 검증 |
| 5 시간 | 준비·상대 지연·유지·cadence·정지 clock | CA04, CA07, CA08, CA18 | T05, T08–T09, T19 |
| 6 반복·발생 | 회차·동시/순차·대상 재타격·기회 소모·마지막 회차 | CA07, CA15 | T08, T16. 간격 변경은 횟수와 독립 |
| 7 종류 | 피해/회복/shield/modifier/기본 공격 교체/control/HP 보장 | CA05–CA09, CA14 | T06–T10, T15 |
| 8 양·참조 | Attack/SpellPower/HP/시간/자원/결과, snapshot/live | CA05, CA06, CA09 | T06–T07, T10 |
| 9 집계·분배·공유 | count/sum/max, tile grouping, 동일량/총량, shared pool | CA04, CA05, CA09, CA15 | T05–T06, T10, T16 |
| 10 자원·축적 | owner, cap/grant/decay/consume, segmented pool, run/attempt | CA04, CA10, CA15, CA19 | T05, T11, T16, T20 |
| 11 비용·위험 | HP/공명/자원, nonlethal preflight, 기회 포기·실패 | CA04, CA06, CA09, CA18 | T05, T07, T10, T19 |
| 12 조건·분기·연계 | 사건·확률·인정 단위·우선순위, branch/join, 전파·chain | CA06, CA07, CA12, CA13, CA15, CA16 | T07–T08, T13–T14, T16–T17 |
| 13 병행·제약 | attack/move/block/channel, lock 소유·취소·released work | CA07–CA09, CA16 | T08–T10, T17 |
| 14 종료·존속 | 만료/소진/재시전/이탈/사망, run·attempt·재배치 | CA04, CA08, CA10, CA14, CA19 | T05, T09, T11, T15, T20 |
| 15 출처·소유·귀속 | 원효과·modifier 제공자·executor·resource owner·beneficiary·credit | CA01, CA04, CA06, CA12–CA15, CA19 | T01, T05, T07, T13–T16, T20 |

## 대표 공식 콘텐츠 실제 이식 목록

원문의 정확한 수치·세부 정책이 미정이면 test-only prototype profile과 결정 기록을 함께 둔다. 빈약한 기존 live placeholder가 아니라 원문 구조를 새 RON으로 작성한다. 이식 범위는 아래 효과/콘텐츠 단위이며, 해당 환상체의 모든 장비·서사·연출·보상 완성을 뜻하지 않는다.

| C ID | 실제 이식 대상 | 핵심 원문 구조 | 첫 시험 단계 → 최종 검증 | 남길 미정 |
| --- | --- | --- | --- | --- |
| C01 | 나무꾼 파편 + 깊은 쐐기 | 순서 있는 다중 직접 타격·강한 마지막 타격; 시전별·대상별 이번 타격 포함 n에 따른 증폭 | CA07 → CA12/13 → CA20 | 타격별 수치·적중 인정·중복 정책. 10%p는 예시 |
| C02 | 풍운승려 파편 및 범위 Physical 무기·조건부 강화/관통의 대표 장비 효과 | 범위 피해 후 1회 자기 회복, Attack·HP 식, basic attack과 direct skill source 구분 | CA05/07/08 → CA20 | 최종 선택·수치·보호막 적중 인정. Staff가 Magic을 뜻하지 않음 |
| C03 | 아무 것도 없는 보스 + Hello/Goodbye 기본 파편 | 보스 연속 HP·변신 중 피해·phase skip; 파편 관통 기본 공격 override→timeout/reinput→복귀 | CA08/16 → CA20 | 개화형 효과는 미정이므로 창작하지 않음. 시간·위력·상태 변화 동시 판정 |
| C04 | 포큐버스 파편 + 초기량 증폭 액세서리 효과 + 단일 전환 범용 유물 적용 | 여러 수혜자·하나의 원천 잔량, 생성 시 modifier 1회, 단일 원천 2S | CA09/12 → CA20 | 기본 보호량 식·범위·다른 shield 우선순위. 침식 4S는 비교안 |
| C05 | 태아 파편 + 유지시간/범위 변경과 단일 전환의 범용 적용 | 지속 집중, 동적 aura membership, basic 피해/속도 한정, 종료 해제 | CA08/12 → CA20 | 정확한 집중 시간·block 규칙·단일 대상 선택. 상태별 수치 교체와 일반 배율 구분 |
| C06 | 래티씨아 기본/개화 파편 + 붉은 실타래 | 타일 배치·작은→큰 진화·사본별 형태·전용 대상 제한·연결망 순차 발동 | CA14/15 → CA20 | 연결 거리·동시 node 수·소비 범위·공격 순서 세부 |
| C07 | 포큐버스 보스 | 잠복/노출·거리 기반 피해 조건·Route 진행·강화·작업 완료→억제 | CA09/16 → CA20 | map layout·거리·노출 시간·wave 구성. 플레이어 shield와 보스 거리 면역을 혼동하지 않음 |

두 보스인 C03/C07은 서로 다른 구조를 검증한다. 현재 Little Red/CWBS RON은 기존 kernel 회귀용 참고이며 최종 설계 이식 성공의 대체물이 아니다.

## 공식 전량 이식과 별개로 필요한 표현력 시험

고기벽의 segmented HP 분리와 HP fallback, 원천별 포자 감염의 성장·사망 전파·일제 소비, 나비 tile 집계, 루돌타의 유한 기회 반복, 흑조의 감쇄 축적/패링 방출, 직원 귀속 상태와 파편 귀속 Run 상태 등은 별도 최소 RON fixture로 검증한다. 이 목록은 전체 원문 목록의 대체가 아니며 CA00의 COV 행이 최종 누락 검사 기준이다.

`최소 fixture`도 production과 같은 최신 parser/compiler/검증/실행을 통과해야 한다. 테스트 목적으로 의미를 임의로 바꿔야만 통과하는 경우에는 해당 정책을 보류하고 누락 사실을 남긴다.

## 재조합 holdout

CA20 후 엔진 revision을 고정한 다음 CA21에서 별도 작성자가 다른 ID/팩으로 새 조합을 만든다. 네 도메인 신규 제작, 타 팩 효과 참조, generic modifier의 생성 효과 적용, 전용 modifier의 제한 유지, 공식 정의 override와 등록 경로가 포함된다.

ID를 바꾸는 시험은 hardcoded 이름 분기를 찾기 위한 것이다. ID를 RNG identity에 사용하는 계약에서는 이름 변경 전후 난수 비트까지 같다고 요구하지 않는다. 이 시험은 100% 발동 등 비확률 입력으로 의미를 비교하고, 결정론 재현은 동일 ID/버전/팩/seed/입력/환경 조건에서 별도로 검증한다.


## v2: 권장 구조와 요구의 연결

| 설계 | 관련 CA | 추가로 확인할 요구 |
| --- | --- | --- |
| A01 모듈·계약 | CA00–CA01, CA23 | P/R/L 구분, producer-consumer·K01–K10 owner |
| A02 작성·compile | CA01–CA03, CA05, CA07, CA12 | EffectSite/ActionSite 독립, result slot, finite completion, 변형 후 재검증 |
| A03 runtime | CA04–CA09, CA13–CA15, CA18–CA19 | owner×lifetime, handle/epoch, receipts, transaction fault·복원 |
| A04 composition | CA11–CA13 | finite grant closure·negative cycle, typed patch, preserve set, binding/cache |
| A05 boss | CA07, CA14, CA16–CA18, CA20–CA21 | task milestone, cancel, blackboard, signal/multi-actor·wave 권한·trace |
| A06 data/tools | CA02–CA03, CA17, CA19, CA21–CA22 | effective catalog 검증, pin, loader 일관성, diagnostic·semantic fingerprint |
| A07 cutover | CA01–CA03, CA10, CA19–CA23 | 기존 심볼 재사용·교체, 최신 단일 경로·데이터/시험/저장 동반 전환 |
| A08 worked examples | CA07–CA09, CA12–CA16, CA20 | E01–E06를 실제 최신 RON과 trace/state 기대값으로 전환 |

### 보스 전용 제작 coverage

CA00에서 `BOSS-COV-*` 하위 행을 추가하여 phase/selector/task/parameter/query/movement/skill/telegraph/state effect/blackboard/signal/summon/wave/multi-actor를 원문별로 연결한다. row를 만들었다고 구현된 것은 아니다. 실제 code와 검증 단계까지 연결해야 한다.

CA21의 독립 보스는 새 state/pattern과 두 개 이상의 skill task, 소환 또는 작업 사건 기믹, 정상·실패·취소 route를 data-only로 조합한다. 단순 다른 stats/이름으로 이 요구를 대신하지 않는다.

v1 COV 식별자가 이미 실제 실행 기록에 생겼다면 재번호로 증거 연결을 잃지 않는다. 실제 수행 기록이 없는 이번 패키지는 추적 양식과 선정 사례를 제공하며 전체 원문 전수 검토가 끝난 것으로 표시하지 않는다.

## v2.1 — 순서 근거 연결

각 선택 요구의 선행 관계와 전환 근거는 실제 Code Review·Phase Plan을 참조한다. COV 또는 CA 번호의 존재는 순서 검토 완료가 아니다. 현황에는 `계획상 후보 / 코드 검토 근거·범위 / 실제 선행 충족 여부`를 짧게 분리해 표시하며, 별도 중복 원본을 만들지 않는다.
