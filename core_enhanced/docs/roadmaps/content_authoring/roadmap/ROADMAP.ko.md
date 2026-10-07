# Simulator 커뮤니티 콘텐츠 제작 기반 — 개선된 개발 로드맵

**v2.1 · 2026-09-09 · 권장 아키텍처 + 코드 기반 상세 설계·구현 절차**

이 문서는 v2.0의 구조·요구·24개 작업을 보존하면서 순서의 증거 수준과 실행 인수 기준을 명료화한 v2.1이다. 프로젝트 코드·데이터 수정, 빌드·테스트·성능 검증을 수행한 결과가 아니다. 구체 타입·포트·RON 예제는 권장 설계이며 현재 코드에 이미 존재하거나 parser로 검증되었다는 뜻이 아니다.

**CA 번호·M 단계·선행 표는 계획상 후보다. 모든 의존 관계와 단계별 중간 상태를 실제 코드로 검증해 확정한 실행 순서가 아니다.** 선택한 범위의 P2G-20 코드 근거와 P2G-40 Phase Plan으로 실제 순서를 정하고, 구현 직전에는 실제 선행 구현·감사·현재성을 확인한다. 문서·JSON의 목록 일치는 이 검증을 대신하지 않는다.

## 1. 목표와 역할 분담

최종 목표는 제3자가 공통 명령·행동·조건·효과·규칙을 RON으로 조합해 보스·장비·무기·파편·유물을 만들고 기존 콘텐츠를 교체할 수 있는 기반이다. 지원하는 조합 안의 콘텐츠마다 전용 실행 코드를 추가하지 않는다. 부족한 표현 수단은 엔진 공통 기능으로 확장할 수 있다.

**개선본은 “목적만 제시하고 구조는 전부 구현자에게 맡기는 계획”과 “현재 코드를 모두 읽은 것처럼 세부를 고정한 설계서” 사이를 명확히 나눈다.**

| 구분 | 담당과 기준 |
| --- | --- |
| P — 확정 요구·정책 | 사용자 결정과 승인된 최신 원문. 구현 편의로 바꾸지 않음 |
| R — 권장 구조 | 이 문서의 모듈·타입·알고리즘·인터페이스 제안. 코드 근거와 영향 검토로 변경 가능 |
| L — 작업 상세 설계 | 구현 AI가 실제 관련 코드를 읽고 exact 타입·함수·자료구조·실행/전환 방법을 완성 |
| 구현·검증 | 구현 AI가 승인 범위 안에서 실제 수정·시험·완료 검토를 수행 |

함수 이름·컨테이너·동일 의미 최적화는 구현자가 정할 수 있다. 사본 소유·사건 순서·완료/취소 계약을 작업마다 독립적으로 다시 정하지는 않는다.

## 2. 이미 확정한 범위

| 항목 | 적용 기준 |
| --- | --- |
| 제작 프로그램 | RON 조합기 UI는 제외. 공통 작성 정보·검증·설명·headless 시험 포트는 포함 |
| 콘텐츠 | 대표 공식 구조 C01–C07을 실제 RON으로 이식. 전체 합의 구조는 coverage로 대조 |
| 호환성 | 최신 RON·저장 계약만 지원. legacy parser·alias·dual runtime·자동 저장 migration 없음 |
| 외부 데이터 | 로컬 live_data의 다중 팩·명시 의존/교체/충돌. 신규/공식 ID 차별 없음 |
| 적용 시점 | 새 런이 검증한 데이터 세대를 고정. 시험 reload는 성공 후 초기화. 진행 전투 hot patch 제외 |
| 권한 | 신규·변형·교체·등장/획득 연결과 지원된 custom rule 수정 허용. 외부 코드 실행 제외 |
| 파편 | 개화·형태·사본 자원은 개별 인스턴스와 연결 |
| 공식 유물 | 공용 3슬롯·독립 사본·휴면·경로별 순환 제한·상태 수명·침식 제외 유지 |
| 비범위 | game_server/Unity 전환, 실제 렌더링·asset 제작, 배포 서비스/마켓, 전체 밸런스·공급 정책 |

수정 가능한 공식 policy profile과 타입·참조·사본 독점·원자성·실행 예산 같은 엔진 무결성을 구분한다. 공식 슬롯 3개를 custom profile로 바꿀 수 있는 것과, 한 사본을 두 직원이 동시에 소유할 수 있게 하는 것은 다르다. 옛 RON 폐기 허용은 실제 파일·사용자 저장을 지금 삭제하라는 지시가 아니다.

## 3. 핵심 구조 선택

### 3.1 불변 데이터 세대와 도메인 실행

```text
외부 공식/제3자 팩 + 명시 선택·교체
  → source 묶음 확보
  → 팩 합성·타입·최종 참조 검증
  → 스킬/효과/보스 정의 compile
  → immutable ContentGeneration
  → 새 런 pin
  → 장착·형태·유물의 EffectivePlan + SourceBinding
  → 보스 제어 / 스킬 flow / 기본 공격 / 반응 효과
  → 기존 피해·회복·상태·이동·전달·생성 kernel
  → 웨이브·결과·런 반영
```

스킬 내부는 구조화된 `Sequence/Branch/Parallel/Repeat/Wait/Invoke`를, 보스의 장기 행동은 기존 계층형 phase/behavior/pattern/state/task를 사용한다. 공통 타입·수식·대상·실행 결과를 공유하되 한 거대 VM으로 통합하지 않는다.

### 3.2 작성 정의·컴파일 정의·상태를 분리

작성 AST는 읽기 쉬운 ID·타입·원문 위치를 가진다. 컴파일 결과는 검증된 참조 index, typed operation, result slot, SourceMap을 갖는 불변 계획이다. 사본·직원·effect instance는 별도 binding과 runtime store가 소유한다.

단계뿐 아니라 개별 `EffectSite`를 식별한다. 투사체 등 공통 carrier는 `ActionSite`, 피해·회복·상태 적용은 `EffectSite`로 구분한다. 피해 footprint만 늘렸는데 같은 action의 자기 회복이나 독 대상이 자동으로 넓어지지 않게 한다. carrier 속성 변경은 그 action/group을 명시적으로 선택해야 한다.

### 3.3 상태 owner와 lifetime을 별도 계약으로 둔다

사본·직원·공유 원천은 owner다. Run·attempt·deployment·activation·delivery는 lifetime이다. 모든 상태를 RuntimeUnit에 넣지 않는다.

유물 전투당 사용 기록은 `(attempt, relic copy, relic effect)`를 기준으로 유지하고, 배치 임시 스택에는 deployment epoch를 붙인다. 같은 정의의 두 파편이 template을 공유해도 개화·자원을 공유하지 않는다.

### 3.4 사건·명령·완료를 구분

요청과 확정 사실은 다르다. HP 비용은 피해 사건이 아니고 실제 회복량은 요청 회복량과 다르다. Action/Strike/Segment와 source·credit을 보존한 receipt를 사용한다.

ActionHandle은 접수, release, flow 완료, 효과 해결의 진행을 표현한다. 어떤 지점을 기다리는지 consumer가 명시한다. 취소 token은 controller의 미완료 작업과 이미 release된 독립 전달을 구분한다. 이전 activation의 늦은 완료가 새 상태를 전이시키지 않게 한다.

### 3.5 효과 합성은 전역 정의를 수정하지 않는다

장착 조합에서 base effects→유한한 부여 closure→typed 구조 변환→property별 수치 변경→최종 graph 검증을 거쳐 effective plan을 생성한다. 동적 HP·자원은 실행 식으로 남긴다.

태그는 selector 조건이며 변경 가능 property의 증명이 아니다. 생성 효과는 안정적인 output ID·출처를 갖고 다른 유물의 조건에 참여한다. 외부 근거 없이 서로만 활성화하는 순환은 휴면, 음의 순환·모호한 다중 Replace는 진단하는 것이 기본 제안이다. 정확한 gameplay 충돌은 PG01/PG02로 결정한다.

### 3.6 연쇄는 경로별 실행 상태다

branch-local `(copy,effect)` 방문 frame을 지연·투사체·영역의 자식에 전달한다. sibling의 이력을 전투 전체 visited로 합치지 않는다. Timer마다 새 root를 만들어 제한을 우회할 수 없고, 진짜 여러 부모의 join은 명시적으로 인과 제약을 합성한다.

## 4. 보스 제작을 위한 구체 구조

보스는 다음 단위로 작성한다.

```text
Encounter
 ├─ actor slot ↔ scenario spawn binding
 ├─ phase graph / shared typed facts
 └─ actor behavior
      └─ PatternCall + selector
           └─ reusable parameterized Pattern
                └─ State
                     ├─ 동시 Task와 각 result
                     ├─ 상태 소유 modifier/lease
                     └─ 조건·우선순위·전이
```

권장 보스 Task 포트는 `StartAction(skill/action ref, actor, target/aim, owner scope, wait milestone, cancel policy)`가 handle을 반환하고, 진행·완료·실패·취소를 typed result로 받는 구조다.

**“사격 후 이동”은 Released까지 기다리는지 EffectsSettled까지 기다리는지에 따라 다르다.** 전자는 발사 직후 이동, 후자는 피해 해결 뒤 전이다. 접수 성공을 완료로 간주하지 않는다. 지속 aura·자립 소환처럼 종료 보장이 없는 action에는 무조건 모든 효과 완료를 기다리게 하지 않는다.

상태 전환은 leave 작업 취소·자기 lease 제거→새 activation/task 계획→원자적 확정으로 처리한다. 이미 방출된 투사체는 자신의 수명을 따른다. phase 교체를 unit 사망·재생성으로 표현하지 않는다.

FieldWork는 작업 진행과 완료 fact, boss는 그 신호에 따른 억제·강화 중단, WaveRuntime은 spawn·필수 웨이브, Scenario는 승리·보상 권한을 소유한다. 조립 가능한 연결은 명시적 signal·actor binding으로 제공한다.

보스 descriptor에는 각 task의 입력·결과·finite completion·capability 충돌·취소가 공개되어야 한다. trace에는 “왜 이 패턴인가 / 왜 이 상태로 전이했는가 / 어떤 작업이 취소되었는가”를 source 위치와 함께 남긴다.

CA16은 **작성/검증 → 스킬 handle/취소 → 형태/조건/선택 → 작업·소환·wave·다중 actor·trace**의 네 묶음으로 구체화한다. 상세는 [보스 제작 설계](architecture/05_BOSS_AUTHORING.ko.md)다.

## 5. 코드 조사와 설계 점검을 작업에 포함한다

모든 CA 작업은 다음 과정을 거친다.

```text
D0 범위·실제 선행 계약 확인
  → D1 관련 코드 종단 조사
  → D2 현재 코드에 맞는 상세 설계
  → DG 설계 준비 점검
  → I 구현
  → V 실제 검증·필요 시 설계 환류
  → A 완료 검토·인계
```

D1은 매번 전체 저장소를 읽는다는 뜻이 아니다. 선택 경계의 생산자→검증→실행→상태/결과·저장→테스트를 빠짐없이 연결해 읽는 단계다. D2에는 exact 타입·입출력·상태 key·순서·취소·오류·변경 심볼·전환·테스트 기대값이 있어야 한다.

DG는 모든 기술 선택에 대한 사용자 승인 요청이 아니다. 내부 동등 변경은 구현자 판단, 공유 K 계약 변경은 영향 검토·revision 갱신, gameplay·범위 변경은 별도 사용자 결정으로 나눈다. 기존 R 설계가 실제 코드와 맞지 않으면 근거와 대안을 기록하고 바꿀 수 있다.

## 6. 9개 단계와 24개 작업

기존 CA00–CA23 ID와 검증 목적을 보존한다. [A07 전환 지도](architecture/07_CUTOVER_AND_REUSE.ko.md)는 현재 심볼별 재사용·교체 방향과 최신 데이터·테스트 전환을 연결한다. 각 카드에는 목표 외에 **권장 구조·이유·실제 조사 질문·D2/DG 산출물·구현 묶음·추가 인수 기준**이 들어 있다.

| 단계 | 작업 | 구현 구조와 종료 증거 |
| --- | --- | --- |
| M0 | CA00 요구 추적 / CA01 작성·컴파일 계약 | 전체 COV·P/R/미정 구분, EffectSite/ActionSite·typed flow·result/source map의 최소 실행 slice |
| M1 | CA02 외부 팩 / CA03 공통 도구 | raw/effective 검증 분리, generation pin, 동일 API·CLI, 같은 바이너리 A/B 로딩 |
| M2 | CA04 상태 / CA05 수식 / CA06 사건 / CA07 flow | owner×lifetime store, typed expressions, receipts, explicit wait/branch/parallel/repeat |
| M3 | CA08 지속·공격 / CA09 방어 / CA10 파편 | source ledger·leases·aura·override, 단일 shield pool, independent OwnedFragment |
| M4 | CA11 유물 관리 / CA12 합성 / CA13 연쇄 | copy/slot aggregate, closure·typed transforms, branch-local ancestry |
| M5 | CA14 매개체 / CA15 관계 / CA16 보스 | actor/trap/object 분리, relation/status membership, task→action handle·signal |
| M6 | CA17 등록·평가 / CA18 침식 / CA19 통합 | 일반 grant/registry, 특별 cast/kill suppression, attempt delta·저장 fingerprint |
| M7 | CA20 대표 RON / CA21 독립 제작 | C01–C07과 별도 data-only 보스·장비·파편·유물 조합 |
| M8 | CA22 안전·성능 / CA23 감사 | 실제 runtime 예산·결정론·profile, 전체 K/COV/evidence 대조 |

세부 선택:

**CA04–CA07:** identity·state·event·completion 계약을 먼저 연결한다. 이후 기능이 임의의 전역 변수·로그 parent로 각자 비슷한 기능을 만들지 않게 한다.

**CA08–CA09:** modifier는 handle 삭제로 해제하고, aura는 membership에서 파생한다. shield는 ResourcePool이며 HP 회복·비용과 구분한다. Hello/태아/포큐버스를 별도 원문의 구조대로 시험한다.

**CA10–CA13:** 사본 관리와 효과 계산을 분리하고 형태 변경이 유효 plan을 무효화하게 한다. 새 grant 정의의 활성 계산과 실제 runtime chain은 다른 알고리즘이다.

**CA14–CA16:** 생성·설치·진화의 기본 실행은 유물 연쇄 전체에 종속시키지 않는다. relation/유물 통합은 CA15, 보스 task와 scenario 연결은 CA16에서 검증한다.

**CA17–CA19:** 상태를 만드는 각 작업이 그때 저장·후퇴 영향을 반영한다. CA19는 종단 검증이지 저장을 마지막에 몰아넣는 단계가 아니다. 침식은 대가·kill bookkeeping·금지된 부가 effect suppression을 함께 확정한다.

**CA20–CA23:** prototype을 마지막까지 미루지 않고 선행 단계에서 시험한다. 독립 제작은 frozen engine에서 data-only로 수행한다. 이후 최적화가 계약·실행 의미를 바꾸면 그 증거도 재획득한다.

## 7. 계획상 순서·코드 검토·실행 준비도를 구분한다

공통 K 계약의 최소안이 나온 뒤 보스·유물 상세 설계 초안은 병행할 수 있다. 실제 compile/runtime 검증은 구현 의존성이 충족된 후 수행한다. 설계 초안이 있다는 이유로 미구현 API를 실제로 사용할 수 있다고 표시하지 않는다.

v2에서 **CA14의 CA13 필수 의존을 제거하고 CA15에 CA13 의존을 명시**했다. 생성·진화 자체에 모든 유물 연쇄 완료를 요구하지 않기 때문이다. CA16의 보스 설계·실행은 이 매개체와 공통 action 계약을 사용하며 전체 유물 제작 완료를 불필요한 선행 조건으로 만들지 않는다.

[DEPENDENCIES.ko.md](DEPENDENCIES.ko.md)와 [GOALS.json](GOALS.json)은 같은 **계획상 선행 후보**를 표시한다. 이는 실행 가능한 스케줄러나 실제 Phase Plan이 아니다. 각 관계를 기능 필수·동시 전환·검증 선행·진행상 권장으로 구분하고, 필요한 생산자·소비자·수명·실패·저장·테스트 근거를 확인한다. 실제 Phase Plan이 수립되면 그 범위의 순서는 해당 owner 문서가 소유한다. 같은 M 번호라는 이유만으로 병렬화하지 않는다.

처음에는 전체의 주요 공유 경계에서 큰 역전·누락·과잉 선행을 살피고, 곧 수행할 범위와 직접 의존 부분을 깊게 검토한다. 후반 모든 함수·24개 작업 전체의 상세 설계를 완료해야 첫 작업을 시작할 수 있다는 요구는 아니다. 승인된 의미·scope를 유지하는 순서 조정은 근거 있는 기술 판단으로 처리하며, 이미 동결된 Phase/Goal을 바꾸어야 하면 P2G의 owner 변경 절차를 따른다.

## 8. 대표 이식과 독립 보스 제작

| 사례 | 핵심 검증 |
| --- | --- |
| C01 나무꾼 + 깊은 쐐기 | 타격/segment·이번 타격 인덱스·대상별 증가 |
| C02 풍운승려 파편·대표 장비 | AoE 뒤 자기 회복 1회, 식·scope·source 독립 |
| C03 아무 것도 없는 보스 + Hello/Goodbye | 변신 중 피해·HP skip·재입력·기본 공격 복귀 |
| C04 포큐버스 파편·초기량 장비·단일 전환 | 공유 원천 하나, 초기 modifier 1회, 일반 2S |
| C05 태아 파편과 범용 변형 | channel·aura membership·basic 전용 보정 |
| C06 래티씨아 기본/개화 + 붉은 실타래 | 사본별 형태·배치·진화·연결·소비 |
| C07 포큐버스 보스 | 잠복/노출·거리 조건·작업 사건→억제 |

전체 원문은 CA00의 COV 행으로 대조하며 대표 성공으로 미지원 기능을 숨기지 않는다. 고기벽·포자 감염·나비 타일 집계·루돌타 반복·흑조 축적 등은 별도 최소 최신 RON으로 표현력을 검증한다.

CA21의 보스 holdout에는 신규 상태/패턴·둘 이상의 스킬 작업, 소환 또는 작업 사건 기믹, 실패·취소 route를 포함한다. 공식 이름만 바꾼 복제가 아니라 기능군 재조합을 요구한다. 이때 새로운 공식 서사·밸런스를 설계할 필요는 없다.

[설계 예제 A08](architecture/08_WORKED_EXAMPLES.ko.md)은 RON 의사자료→compiler 처리→runtime owner/order→기대 결과를 보여준다. 그것을 실제 실행 검증으로 취급하지 않는다.

## 9. 정책 gate와 내부 설계의 경계

기존 PG00–PG09를 유지한다. 동일 피해 증폭 가산, 사본별 개화, 외부 로딩과 적용 시점 등 이미 확정된 것은 다시 질문하지 않는다.

남은 gameplay 결정은 합성·교체, 활성·음의 조건·join/root, 적중·동시 사건, shield/cost/survival 중재, 자원·형태 수명, 콘텐츠 수치·획득, 침식 variant와 평가 가중 등이다. 기술적인 manifest 구조·cache·port 이름은 구현 AI가 제안·검토할 수 있다.

권장 DG는 R의 가정을 실제 코드로 확인하고 미정 정책을 분리하는 지점이다. 원문상 미정인 의미를 0·무제한·무조건 초기화 같은 default로 숨기지 않는다. fixture 숫자는 test-only로 분리하며 공식 정책을 승인한 것처럼 쓰지 않는다.

## 10. 완료 조건과 검증

기존 T00–T24를 유지하고 v2는 T25–T29로 설계 계약 정합성, 보스 handle/milestone, effect/carrier scope, compile closure·causal join, loader/save/fault 경계를 강화한다.

최종 증거는 source revision·관련 dirty, schema/contract·팩 fingerprint, seed/initial state/input/clock, 실제 명령·test 수·결과, semantic trace, 측정 환경과 남은 한계를 포함한다.

구형 테스트를 지워 녹색으로 만들지 않는다. 승인 정책 변경으로 기대값이 달라지면 근거와 대체 coverage를 기록한다. parser·graph·event·instance 예산은 초기부터 두고 CA22에서 실제 혼합 부하로 측정한다. 성능 개선이 의미를 바꾸면 DG와 이전 acceptance를 다시 확인한다.

**최종 인수는 네 도메인 신규 제작·교체·등록과 효과 변경·연쇄·정확한 상태 수명·침식·다중 팩·독립 재조합이 실제 최신 RON으로 성립할 때만 가능하다.** 필수 공통 기능·정책 gate가 남으면 부분 완료로 보고한다. 조합기/서버/Unity 출시나 전체 공식 콘텐츠의 밸런스 완료까지 주장하지 않는다.

## 11. 다음 구현 AI의 첫 작업

기존 진행이 없다고 확인된 경우의 첫 후보는 CA00이다. [현황 진입점](../ROADMAP_STATUS.md)에서 기존 Topic·Goal·구현·감사부터 인수하고, CA00의 요구 기준선과 주요 공유 경계 의존성 1차 조사를 연결한다. CA01은 첫 후속 계약 후보이지 번호만으로 확정된 다음 실행이 아니다. 선택 범위의 코드 검토와 유효한 중간 상태를 근거로 다음 Topic·Phase를 정한다. 전체 프로젝트를 한 번에 수정하거나 24개 작업의 R 설계를 각자 별도 구현하지 않는다.

인수 흐름은 **README → ROADMAP → 관련 architecture → 선택 CA → DESIGN_WORKFLOW → 관련 source·PG·tests**다. templates는 최소 상세 설계·설계 점검·변경 이력을 제공한다. 실제 저장소 Goal 절차가 우선이며 이 패키지는 아직 실행 기록이 아니다.

---

**v2의 변화:** 구현할 목적뿐 아니라 권장 구조와 그 이유, 코드에서 확인할 가정, 구현자가 완성할 상세 설계, 설계 변경 권한, 실행 증거의 경계를 함께 제공한다. 핵심 방향은 일관되게 유지하되 실제 코드를 읽은 구현 AI가 더 나은 구조를 근거로 선택할 수 있게 한다.

**v2.1의 수정 범위:** 권장 아키텍처·콘텐츠 정책과 기존 선행 후보 목록은 변경하지 않았다. 순서를 이미 코드로 검증했다는 오해를 없애고, 기존 P2G-20/40·GEX 인수에서 확인할 근거를 명시했다. 신규 승인 단계나 별도 의존성 상태 기계는 추가하지 않는다.
