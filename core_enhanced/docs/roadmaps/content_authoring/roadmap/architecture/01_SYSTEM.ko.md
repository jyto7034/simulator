# A01 — 권장 아키텍처와 계약 경계

계획 v2.1 · 2026-09-09 · **권장 설계(R), 구현 확인 아님**. 모듈·타입·인터페이스 이름은 제안이다. 이 문서가 모든 현재 코드를 조사해 확정한 저수준 명세라는 뜻은 아니다.

## 1. 선택한 구조

**불변 콘텐츠 세대 + 타입화된 행동/효과 계획 + 출처를 보존하는 도메인별 실행기**를 채택안으로 제시한다.

```text
FileSource / PackSelection
        ↓
PackResolver ── SourceMap / CompositionReceipt
        ↓
AuthoringCompiler ← CapabilityRegistry
        ↓
ContentGeneration (불변 정의·계약 버전·해시)
        ↓ 고정 참조
RunContext ── OwnedFragment / OwnedRelic / OwnedEquipment / RuleProfile
        ↓
LoadoutComposer → EffectivePlanTemplate + SourceBindingTable
        ↓
BattleSession (attempt state, clock, queue, transaction boundary)
  ├─ BossController   ─┐
  ├─ SkillFlowRunner  ├─ typed requests/results → domain services
  ├─ BasicAttack      │       ├─ Target / Spatial / Movement
  └─ ReactiveEffects ─┘       ├─ Damage / Heal / Modifier / Resource
                              ├─ Channel / Aura / Delivery
                              └─ Spawn / Object / Relation
                  ↓ committed facts
          Wave / Objective / Result / RunCommit
```

BossController는 다음 행동을 고르고 SkillFlowRunner는 유한한 행동의 순서·분기·반복을 진행한다. 공통 효과 서비스는 피해·회복·상태를 실제로 적용한다. 시나리오는 웨이브와 승리, 런은 소유·성장·저장을 소유한다. 다섯 책임을 하나의 문자열 명령 실행기에 합치지 않는다.

## 2. 논리 모듈과 의존 방향

| 논리 모듈 | 소유하는 것 | 소유하지 않는 것 | 현재 연결 후보 |
| --- | --- | --- | --- |
| `authoring_contract` | typed authoring AST, ID 종류, descriptor, 진단 코드 | mutable 전투 state, 파일 I/O | `ability.rs`, `stats.rs`, `data/*`의 작성 타입 |
| `content_pipeline` | source 묶음, pack 합성, 이름 해석, compile, 게시 | 실행 중 상태 변경 | `data/live_ron.rs`, `data/mod.rs`, `validation.rs` |
| `composition` | 효과 선택, 정적 활성, 생성·변형·수치 합성, explain | 실제 HP·자원 잔량, event 실행 | 새 논리 경계; 현행 profile 조립에서 연결 |
| `execution_contract` | source·handle·typed request/result·상태 범위·오류 | 특정 보스 선택 알고리즘 | `battle/damage.rs`, `core/types.rs` |
| `effect_services` | 피해·회복·자원·modifier·전달·매개체 | 룰북 전체·보상 승인 | 기존 battle kernel 재사용 |
| `boss_control` | phase/behavior/pattern/task·blackboard·selection | 스킬 피해 재계산, 직접 reward grant | `data/boss_runtime_data.rs`, `core/boss_runtime.rs` |
| `run_domain` | 사본·형태·장착·정책·결과·저장 | live battle의 개별 효과 처리 | `skill_fragment.rs`, `resources/*`, `world/*` |
| `authoring_tools` | describe/validate/explain/simulate 포트 | 별도 validator·테스트용 전용 엔진 | `CoreApplication`과 신규 얇은 도구 target |

경로의 짧은 이름은 `core_enhanced/src/game/` 기준이다. 최종 crate/디렉터리 경계는 구현 AI가 실제 의존 그래프를 확인한 뒤 결정한다. 위의 **책임과 의존 방향**을 파일 이름과 동일시하지 않는다. 새로운 기능 하나 때문에 반드시 새 crate를 만들 필요도 없다.

## 3. 왜 이 구조인가

| 대안 | 채택하지 않는 이유 |
| --- | --- |
| 효과 enum만 계속 추가 | 개별 효과 선택·출처·종료·변형 관계를 별도로 해결하지 못함 |
| 전부 하나의 VM/전투 스크립트 | 기존 domain authority와 검증을 버리며 작성자에게 구현 세부를 노출할 위험 |
| 콘텐츠 이름별 callback | 공식 예제는 작동해도 제3자 조합이 전용 코드를 요구함 |
| 임의 JSON/RON 경로 patch | 타입·변경 가능 속성·scope·합성을 검증하기 어렵고 내부 구조가 제작 API가 됨 |
| 모든 팩을 전투 중 가변 조회 | 같은 런에서 다른 정의 세대가 섞이고 재현·저장이 불안정해짐 |
| 변경 때마다 entire world 영구 복제 | 의미상 필요한 원자성과 구체적인 복사 전략을 혼동; 비용은 측정 후 결정 |

이 선택은 전투 코어 전체 재작성을 금지한다는 뜻이 아니다. 실제 조사에서 특정 경계의 교체가 더 작고 정확하면 그 경계를 교체한다. 전체 재작성으로 넓히려면 기존 자산으로 목표를 충족하지 못하는 구체적인 근거와 영향 분석을 남긴다.

## 4. 공통 계약 K01–K10

| ID | 구현 전에 공유해야 할 계약 | 소비자 |
| --- | --- | --- |
| K01 | definition/effect/copy/action/strike/segment/activation/delivery identity와 SourceBinding | 모든 도메인 |
| K02 | typed authoring·descriptor·SourceMap·compiled reference | loader, compiler, tools, composition |
| K03 | ContentGeneration pin·fingerprint·저장 일치 | run, battle, reload, persistence |
| K04 | ActionHandle·진행 milestone·terminal result·cancel | boss, skill, basic, delivery |
| K05 | StateOwner·Lifetime·ResourceTxn·lease | aura, shield, relic, summon, run |
| K06 | EventEnvelope·Damage/Heal/DeathReceipt·transaction | reactive, boss, result, wave |
| K07 | EffectSelector·PatchPlan·활성 판정·write conflict | relic, equipment, tool explain |
| K08 | ChainContext·fork/join·root 생산자·budget | reactive, delayed work, relation |
| K09 | ScenarioSignal·actor binding·생성/진행 권한 | boss, field work, wave, objective |
| K10 | Diagnostic·TraceReceipt·execution evidence | validator, tools, QA, implementer |

각 계약에는 owner 작업과 revision을 기록한다. 소비자가 의존하는 것은 모호한 “CAxx가 대충 끝남”이 아니라 계약 revision과 실제 실행 증거다. 논리 계약 확정은 모든 future opcode를 먼저 구현하는 작업이 아니다.

## 5. 캐시·재사용의 기본 선택

ContentGeneration과 compiled template은 공유한다. 효과의 정적 구조를 합성한 template도 같은 입력이면 공유할 수 있다. 사본·직원·활성 상태는 별도 binding table과 runtime store에 둔다. template을 공유했다는 이유로 서로 다른 사본의 자원이나 연쇄 기록이 합쳐지면 안 된다.

EffectivePlan 캐시 키는 최소 generation, 정의/형태, 장비 성장 입력, rule profile, 구조에 관여하는 modifier 집합을 포함한다. 실제 copy ID가 들어간 plan을 캐시하면 copy 집합도 키에 포함한다. 동적 HP·남은 시간은 runtime expression으로 남겨 매 프레임 전체 plan을 재컴파일하지 않는다. 무효화 dependency는 explicit하게 기록한다.

## 6. 현재 사실과 조사할 가정

검증 후 데이터 게시 경계는 이번에 `data/mod.rs:803–821`에서 재확인했다. 그 밖의 세부 구현 위치는 v1 source 색인과 앞선 검토를 잇는 **조사 입구**이며 현 상태 재검증을 대신하지 않는다.

구현 AI는 chosen task에서 생산자→검증→소비자→상태/저장→테스트까지 읽고, 권장 경계에 이미 대응하는 구조가 있으면 재사용한다. 기존 내부 구조가 더 적합하면 비교안을 쓰되 K 계약과 승인 정책을 보존한다. 모든 파일을 일괄 재조사하는 작업을 로드맵의 선행 요구로 두지 않는다.
