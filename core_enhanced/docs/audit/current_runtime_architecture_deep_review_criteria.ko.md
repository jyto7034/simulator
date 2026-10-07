# Current Runtime Architecture Deep Review Criteria

> 참조 상태 정정 (2026-09-06): 아래 과거 입력 목록의 일반 보스 알 메모는 개별 연출을 AI가 임의로
> 공통 정책으로 일반화한 문서여서 사용자 요청으로 삭제했다. 현재 검토 입력이나 승인 정책으로 사용하지 않는다.

> 인수인계 문서 정리 (2026-09-06): 과거 입력 목록의 파편·장비 인수인계 문서도 역할 중복으로 삭제했다.
> 최신 결정과 완료 상태는 유물 정책 및 파편·장비 설계 문서로 이관했으며, 아래 경로는 당시 입력 기록이다.

## 1. 문서 목적

이 문서는 `core_enhanced` Rust runtime의 현재 설계를 원자성 결함이나 과거 감사 finding에서 출발하지 않고
독립적으로 심층 검토할 때 사용하는 판정 기준이다.

최종 질문은 다음과 같다.

> 현재 runtime은 최신 정책 source인 Game Rulebook의 기능을 구현하면서 보스 runtime, movement,
> skill/battle, 로그라이크 시스템을 계속 확장하기에 적합한가? 적합하지 않다면 국소 리팩터링, subsystem
> 재작성, application shell 재작성, 전체 재작성 중 어느 선택이 가장 안전하고 경제적인가?

검토는 설계 결함을 찾는 데서 끝나지 않는다. 보존 가치가 있는 자산, 변경을 강제하는 구조, migration seam,
대안별 의미 재발견 비용과 검증 가능성을 함께 증명한다.

## 2. 문서 지위와 사용자 승인 제약

- 문서 종류: architecture deep review 기준 및 rewrite decision 증거 계약
- 적용 대상: 검토 시작 시 고정한 `core_enhanced` 현재 작업 트리
- 코드 범위: `core_enhanced` Rust runtime
- 제품 상태: 출시 전
- 개발 조건: 일정 제약이 없는 1인 팀과 고성능 AI 기반 구현, 사용자는 정책 판단을 담당
- 호환 조건: server와 Unity 구현 비용은 rewrite 결정의 주요 제약이 아님
- 정책 조건: `docs/game_rulebook.md`가 최신 gameplay 정책의 source of truth이며 current runtime은 이에 비해
  stale할 수 있음
- 보존 조건: current runtime behavior는 먼저 inventory하고 rulebook·current contract·사용자 승인과 대조한다.
  Stale behavior를 target requirement로 자동 승격하지도, 분류 없이 조용히 폐기하지도 않음
- 성능 조건: 대표 workload에서 현재 기준선보다 악화하지 않는 것을 최소 조건으로 하며, 가능한 한 더 가벼운
  구조를 선호함

개별 Goal은 이 기준을 완화하거나 전면 재작성 결론을 선결하지 않는다. 기준 변경이 필요하면 이 문서를 먼저
수정하고 사용자 승인 및 이유를 기록한다.

## 3. Source of Truth와 문서 우선순위

### 3.1 Candidate source catalog

다음 문서는 architecture 검토에 사용할 수 있는 candidate source catalog다. 목록에 있다는 이유만으로 전체
본문을 읽거나 모든 문서를 동일한 권위로 취급하지 않는다.

1. `docs/game_rulebook.md`와 연결된 `docs/game_rules/` 상세 규칙
2. `docs/core_runtime_contract.md`
3. `docs/data_loading_contract.md`
4. `docs/skill_target_contract.md`
5. `docs/tmp/run_relic_system_design_notes_2026-08-21.ko.md`
6. `docs/tmp/handoff_skill_fragment_design_2026-08-20.ko.md`
7. `docs/tmp/normal_boss_egg_design_notes_2026-08-19.ko.md`
8. `docs/tmp/ego_corrosion_system_design_notes_2026-08-19.ko.md`
9. `docs/tmp/generic_dice_and_tool_item_rework_notes_2026-08-17.ko.md`
10. `docs/tmp/skill_activation_and_resonance_rework_notes_2026-08-17.ko.md`
11. `docs/skills/abnormality_design_notes.ko.md`
12. `docs/skills/abnormality_skill_fragments_design_notes.md`

문서의 초기 역할은 다음처럼 구분하되, 실제 relevant section과 current code를 확인한 뒤 source map에서
`normative`, `current technical contract`, `future pressure`, `historical`, `irrelevant` 중 하나로 재분류한다.

- `game_rulebook.md`와 `game_rules/`: 최신 gameplay 정책과 보존해야 하는 기능의 규범적 inventory. 과거 룰북 아카이브는 당시 비교 자료로만 사용
- `core_runtime_contract.md`, `data_loading_contract.md`, `skill_target_contract.md`: 명시된 runtime/data/targeting
  contract 후보. 문서 전체 또는 일부가 Rulebook이나 current code보다 stale할 수 있으므로 relevant section마다
  상태를 확인함
- `docs/tmp/**`: 최근 확정되거나 검토 중인 정책이 runtime에 가할 미래 change pressure; 세부 구현 완료 증거가 아님
- boss/skill-fragment 설계 노트: boss, skill, equipment 조합이 요구하는 표현력과 확장성 stress case

현재 production Rust 코드는 현재 구현 구조, 호출 경로와 동작의 source of truth지만 최신 gameplay 정책의
source가 아니다. Rulebook과 runtime이 다르면 policy drift, intentional technical contract, stale implementation,
문서 stale 중 무엇인지 분류한다. 차이 자체를 architecture finding으로 자동 승격하지 않는다.

Gameplay 의미가 충돌하면 Rulebook이 우선한다. Core/data/skill contract는 Rulebook과 양립하는 technical detail만
current contract가 될 수 있고, current code는 구현 사실을 증명할 뿐 policy를 덮어쓰지 않는다. Rulebook만으로
의도를 확정할 수 없는 충돌은 임의로 해소하지 않고 `policy_undecided`로 둔다.

### 3.2 Progressive source routing

문서는 다음 순서로 필요한 만큼만 읽는다.

1. 경로, 크기/hash, 제목 구조, 문서 책임, 알려진 최신성만 먼저 catalog한다.
2. 현재 criterion, domain workflow 또는 change scenario에 필요한 section을 지정한다.
3. 해당 section과 그 결론을 이해하는 데 직접 필요한 정의·전제·cross-reference만 읽는다.
4. 용어 충돌, source precedence 또는 미해결 의존이 있을 때만 읽기 범위를 확장한다.
5. 읽은 section/range, 사용 목적, 확인한 source status와 생략한 section의 이유를 기록한다.

Heading이나 검색 snippet만으로 결론을 내리지 않지만, 관련성 없는 콘텐츠까지 무작정 정독하지도 않는다.
긴 boss/skill 설계 문서는 representative stress case에 필요한 mechanic만 선택한다. Stale Unity contract는 실제
adapter/history 질문이 생길 때 relevant section만 확인한다. `irrelevant` 판정과 근거 있는 생략은 정상적인 routing
결과이며 gap이나 blocker가 아니다. Source completeness는 문서 소비량이 아니라 criterion·scenario 질문의 evidence
coverage로 판정한다.

`DOCUMENT_SOURCE_MAP.md`의 각 routing record는 최소한 다음을 포함한다.

- source path, content identity와 candidate role
- 현재 criterion/change scenario와 답하려는 질문
- `read`, `skip`, `defer` 결정과 이유
- 실제 읽은 heading/range와 직접 따라간 cross-reference
- section별 판정: `normative`, `current technical contract`, `future pressure`, `historical`, `irrelevant`
- Rulebook/current code와의 conflict 또는 drift 및 연결된 evidence ID

### 3.3 Historical reference

다음 문서는 오래되어 현재 contract의 source of truth가 아니며 historical consumer intent 탐색에만 사용한다.

- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`

이 문서와 현재 runtime이 다르다는 이유로 finding을 만들거나 compatibility 비용을 과대평가하지 않는다.

### 3.4 과거 감사와 draft의 격리

다음은 독립 domain/dependency/change-scenario 분석과 1차 criterion 판정이 끝나기 전에는 읽거나 근거로
사용하지 않는다.

- `docs/goals/current_runtime_structural_integrity_audit/**`
- `docs/goals/current_runtime_structural_integrity_audit/DESIGN_REVIEW_DRAFT.md`
- 그 밖의 과거 `docs/goals/**` finding 또는 완료 선언

독립 분석이 끝난 뒤 별도 cross-check 단계에서만 읽을 수 있다. 이때도 과거 finding을 가져오지 않고 현재
기준선에서 독립적으로 재현·증명하거나 반증한다. 과거 문서와 다른 결론은 차이와 이유를 기록한다.

## 4. 검토 범위

### 4.1 기본 포함 범위

- `Cargo.toml`
- `src/lib.rs`
- `src/game/**`
- runtime 구조와 contract를 증명하는 `tests/**` 및 `src/**` 내부 테스트
- command, query, event, snapshot, DTO, error, persistence serialization 경계
- RON parse, validation, compile, publication 코드와 runtime data types
- seeded RNG, UUID, time, Rapier/physics, filesystem diagnostic boundary
- local Git history 중 change hotspot 판단에 필요한 read-only evidence
- 사용자가 지정한 candidate 정책·contract·설계 문서 중 criterion별 relevant section

`../game_resources/data`는 data-loading과 content-driven extension pressure를 이해하는 데 필요한 정확한 파일만
read-only로 확인할 수 있다. 접근 전 필요성과 범위를 기록한다.

### 4.2 명시적 제외 범위

- production Rust code, test, live data, canonical contract 수정
- finding remediation 또는 prototype을 production tree에 구현
- game_server 또는 Unity code 감사
- 오래된 Unity 구현과 현재 wire compatibility 인증
- 밸런스 수치, 콘텐츠 품질과 재미 평가
- 현재 문서에 없는 새로운 gameplay 정책 결정
- server/Unity migration 구현 비용을 rewrite blocker로 간주
- 새 Goal 또는 remediation Goal 자동 생성

### 4.3 검토와 설계를 분리

목표 architecture와 migration 대안은 문서로 제안할 수 있다. 그러나 현재 Goal 안에서 production 구현을
시작하거나 제안안을 이미 승인된 architecture처럼 표현하지 않는다.

## 5. 검토 원칙

### 5.1 독립 평가축

원자성, 특정 finding, 큰 파일 또는 AI 생성 부채를 전체 판단의 출발점으로 삼지 않는다. 먼저 전체 system
context, domain, aggregate, dependency와 change scenario를 매핑한 뒤 다음 축을 독립적으로 판정한다.

- domain 구조
- 상태·aggregate 모델
- 의존 방향
- 확장성
- 변경 용이성
- 테스트 가능성
- 운영 경계
- contract와 data evolution
- 성능·resource budget
- 결정론·save continuity
- AI-assisted solo development 지속 가능성
- migration과 rewrite 경제성

### 5.2 신호와 finding 구분

다음은 조사 신호이지 단독 finding이 아니다.

- 파일·함수 크기
- public field와 module 수
- `clone`, `unwrap`, `expect`, `panic!`
- enum variant와 match 수
- 추상화·wrapper 수
- 테스트 수 또는 낮은 lint 품질
- AI가 작성했다는 사실

Finding으로 확정하려면 실제 domain responsibility 혼합, change scenario의 수정 증폭, invalid state 표현,
compiler/test 보호 공백, runtime/운영 영향 또는 migration 비용을 제시한다.

### 5.3 기능 보존과 architecture 자유도

출시 전이고 server/Unity 호환 비용이 작다는 이유로 최신 정책인 Game Rulebook의 gameplay 의미를 삭제하지
않는다. 반대로 stale할 수 있는 현재 runtime behavior, Rust type, module layout, wire shape와 save schema를 영구
보존해야 한다고 가정하지 않는다. Current behavior는 제거 전에 목록화하고 정책 적합성을 분류하되, 그 존재만으로
보존 요구가 되지는 않는다.

다음을 구분한다.

- 보존 대상: 최신 rulebook 기능, 사용자 승인 정책, current technical contract로 확인된 gameplay semantics와
  deterministic identity/ordering
- 비교·분류 대상: 최신 정책과 일치하는 current behavior, stale implementation drift, 정책 미결정 behavior
- 변경 가능 대상: module/crate layout, internal type, transaction protocol, DTO version, save representation,
  fixture API, server/Unity adapter

### 5.4 대안 없는 비판 금지

중요한 structural finding은 최소 하나의 현실적인 개선 대안과 migration seam을 함께 기록한다. 대안이 full
rewrite뿐이라고 주장하려면 incremental path가 실패하는 구체적 이유를 증명한다.

### 5.5 점수 평균으로 rewrite 결정 금지

여러 PASS를 평균내어 치명적인 구조 문제를 가리거나, 여러 작은 FAIL 수로 전체 rewrite를 정당화하지 않는다.
Rewrite 결정은 의미 재발견 비용, 보존 자산, migration seam, test oracle, change pressure와 목표 구조의 차이를
함께 비교한다.

## 6. 기준선과 실행 증거

검토 시작 시 다음을 기록한다.

- 시작 시각과 timezone
- `git rev-parse HEAD`
- `git status --short`
- 포함 production/test와 source catalog 문서의 content manifest 또는 동등한 dirty-tree 식별자
- `rustc --version`, `cargo --version`
- OS/architecture와 Cargo feature
- source catalog 문서별 content hash와 실제 사용한 section/range

Dirty tree는 commit ID만으로 식별하지 않는다. 검토 도중 audited production code, test, data, 판단 근거로 사용한
source 문서 또는 실행 환경이 바뀌면 조사를 중단하고 기존 기준선 완료 또는 새 기준선 재시작을 사용자에게
확인한다.

### 6.1 최초 broad baseline

Runtime code 조사 전 다음을 정확히 한 번 실행한다.

```bash
cargo test --all-targets --all-features
```

Test all-pass이고 기준선이 바뀌지 않으면 동일 suite나 baseline에 포함된 focused test를 안심 목적으로 반복하지
않는다. Formatter와 strict clippy는 각각 한 번 실행해 basic engineering signal로 기록하되, lint 수를 architecture
finding 수로 사용하지 않는다.

추가 probe는 정적 증거와 기존 suite로 특정 architecture 가설, complexity 또는 behavior equivalence를 증명할
수 없을 때만 사전 기록 후 `/tmp`에서 수행한다.

## 7. 증거 우선순위

1. 현재 기준선의 production entry와 representative change scenario를 연결한 완전한 vertical trace
2. 현재 public/near-root behavior를 사용하는 test, deterministic probe와 behavior equivalence evidence
3. compiler가 강제하는 visibility, type, exhaustive match와 crate/module dependency
4. current code의 state/data/control flow에 대한 완전한 정적 증명
5. 대표 change를 적용한다고 가정한 expression/change-amplification map
6. local Git history의 반복 hotspot 또는 동시 변경 evidence
7. 선택한 latest policy·current technical contract·future pressure 문서 section의 요구와 설계 의도
8. 주석, 이름, 과거 Goal과 stale consumer 문서

Git history는 현재 dirty tree의 구조를 대체하지 않는다. 문서는 요구사항과 change pressure 증거가 될 수 있지만
현재 구현 완료 또는 설계 적합성의 단독 증거가 될 수 없다.

## 8. 판정 기준 요약

| ID | 기준 | 핵심 질문 |
| --- | --- | --- |
| A00 | 기준선과 정책·source 지도 | 무엇이 최신 정책이고, 무엇이 stale 구현·과거 의도이며, 어떤 근거를 실제 사용했는가? |
| A01 | domain 구조와 ubiquitous language | 기능이 안정적인 domain 책임과 경계로 나뉘는가? |
| A02 | aggregate·상태·불변조건 | 유효 상태와 권위 소유가 구조로 표현되는가? |
| A03 | 의존 방향과 계층 | domain이 transport·world·data authoring·I/O에 역의존하지 않는가? |
| A04 | 확장성 | 보스, movement, skill/battle, roguelike 변화가 기존 구조에 자연스럽게 들어가는가? |
| A05 | 변경 용이성과 인지 부하 | 대표 변경의 표현·동기화·회귀 범위가 통제되는가? |
| A06 | 테스트 가능성과 증명력 | subsystem과 contract를 격리해 빠르고 신뢰성 있게 검증할 수 있는가? |
| A07 | 운영·failure·I/O 경계 | core와 host, persistence, observation, retry 책임이 분명한가? |
| A08 | data·protocol·persistence evolution | authoring, wire, save schema가 domain과 독립적으로 진화 가능한가? |
| A09 | 성능·resource budget | 현재 대비 비퇴행과 명시적 workload/resource 상한을 보장할 수 있는가? |
| A10 | 결정론·identity·continuation | 같은 입력과 restore가 같은 관측 결과를 만드는가? |
| A11 | AI-assisted solo development 지속성 | AI가 추가한 기능을 사람이 정책 수준에서 검증·통제할 수 있는가? |
| A12 | migration과 rewrite 결정 | 어떤 범위를 보존·리팩터링·재작성해야 총위험이 최소인가? |

## 9. 세부 판정 기준

### A00. 기준선과 정책·source 지도

확인 항목:

- dirty baseline과 source catalog 문서 version이 재현 가능한가?
- latest rulebook feature를 domain capability와 production entry에 연결할 수 있는가?
- current behavior를 `latest_policy_match`, `current_technical_contract`, `stale_runtime_drift`,
  `policy_undecided`로 분류할 수 있는가?
- latest policy, future pressure, current implementation evidence와 stale historical intent가 구분되는가?
- criterion별로 실제 읽은 section/range, 선택 이유와 생략 이유가 추적 가능한가?
- unknown 또는 미결정 정책이 architecture 결론을 오염시키지 않도록 격리되는가?
- rewrite 대안마다 기능·save·determinism·data 의미 유실 위험을 비교할 수 있는가?

필수 산출물은 `DOCUMENT_SOURCE_MAP.md`, `REQUIREMENT_PRESERVATION_MAP.md`와 baseline 기록이다.
Preservation map은 current behavior를 곧바로 requirement로 복사하지 않고 Rulebook requirement, 승인된 technical
semantics, 구현 일치 여부, stale drift, 미결정 정책과 rewrite 시 disposition을 별도 열로 유지한다.

### A01. Domain 구조와 ubiquitous language

확인 항목:

- run, floor, node, event, shop, reward, profile, battle, skill, movement, boss runtime의 책임이 구분되는가?
- 같은 용어가 module마다 다른 의미로 사용되거나 다른 용어가 같은 의미를 가리키는가?
- domain object가 workflow/session/DTO/data-authoring 책임을 함께 소유하는가?
- subsystem 경계가 gameplay invariants와 일치하는가, 파일 분리만 존재하는가?
- cross-domain workflow가 명시적 application orchestration을 거치는가?
- feature family가 기존 domain을 확장하지 않고 새 parallel model을 계속 추가하는가?

### A02. Aggregate·상태·불변조건

확인 항목:

- 하나의 gameplay fact에 하나의 authoritative owner가 있는가?
- aggregate boundary와 transaction boundary가 일치하는가?
- phase-specific state가 enum/typed state로 표현되는가, 병렬 `Option` 조합에 의존하는가?
- invalid state와 partial state를 public constructor/field로 만들 수 있는가?
- derived projection/cache와 authority가 구분되는가?
- lifecycle transition이 한 경계에서 검증되고 commit되는가?
- profile/run/battle/temporary/monotonic state가 의미에 따라 분리되는가?

원자성은 이 기준의 한 하위 결과일 수 있지만 전체 검토의 출발점이나 유일한 판정 근거가 아니다.

### A03. 의존 방향과 계층

확인 항목:

- domain leaf가 `behavior`, world orchestration, serde wire, persistence 또는 filesystem을 참조하는가?
- data authoring type이 runtime execution type과 필요 이상으로 결합되는가?
- protocol enum 변경이 battle/data/map leaf를 수정하게 하는가?
- shared type module이 실질적인 순환 의존을 숨기는가?
- visibility가 의존 규칙을 compiler로 강제하는가?
- top-level `pub mod`와 public field가 우회 dependency를 허용하는가?
- 논리적 계층을 module로 정리한 뒤 crate로 분리할 현실적인 seam이 있는가?

필수 산출물은 current import/call/data-flow map과 desired one-way dependency map이다.

### A04. 확장성

다음 대표 change scenario를 실제 타입·handler·data·test 경로에 적용해 본다.

- 새로운 NormalBoss/FinalBoss runtime phase, summon, omen 또는 특수 승패 조건
- 새로운 movement/collision/occupancy/route 규칙
- 새로운 skill activation, resonance, targeting, delivery와 interrupt 형태
- 새 relic/monolith/skill-fragment/ego-corrosion 성장 체계
- generic dice/tool item과 새로운 equipment trigger
- 새로운 node/session/reward/shop/facility 흐름
- 장기 battle, 다수 unit/projectile/effect와 reconnect presentation

각 scenario마다 수정 표현, authority 계층, data schema, validation, protocol, persistence, test와 compiler coverage를
기록한다. 현재 기능을 구체적으로 구현하지 않고 architecture 수용성을 평가한다.

### A05. 변경 용이성과 인지 부하

확인 항목:

- 대표 변경 하나가 몇 개의 서로 다른 표현과 책임을 동시에 수정하는가?
- 변경 누락을 compiler가 잡는가, runtime test와 사람 기억에 의존하는가?
- 대형 module이 실제로 독립된 변경 이유를 여러 개 가지는가?
- helper/wrapper가 순서를 강제하는가 숨기는가?
- 유사 validator, mapper, result, event 또는 policy가 수동 동기화되는가?
- AI가 context 일부만 읽고도 안전한 변경을 만들 수 있는 bounded surface가 있는가?
- 한 기능을 이해하려면 얼마나 많은 unrelated module과 contract를 동시에 읽어야 하는가?

파일 수나 line count는 보조 지표다. `CHANGE_SCENARIO_MATRIX.md`에 표현 종류, 동기화 지점, compiler guard,
회귀 범위와 예상 migration 비용을 기록한다.

### A06. 테스트 가능성과 증명력

확인 항목:

- domain rules를 server/JSON/filesystem 없이 검증할 수 있는가?
- public/near-root test가 command의 관측 가능한 결과와 최종 authority를 비교하는가?
- fixture가 production constructor/validator를 우회하며 별도 의미를 만드는가?
- failure injection, time/RNG/physics control과 deterministic replay seam이 있는가?
- snapshot/characterization test가 의도된 contract를 동결하는가, 내부 구조만 동결하는가?
- subsystem test와 end-to-end behavior equivalence 사이의 공백이 명확한가?
- 대규모 test file과 helper가 새 정책 검증을 어렵게 하는가?
- rewrite 시 old/new implementation을 같은 oracle로 비교할 수 있는가?

### A07. 운영·failure·I/O 경계

확인 항목:

- core, server/host, persistence store, network adapter의 책임이 명시적인가?
- command identity, idempotency, revision, retry, reconnect와 delivery 의미가 분리되는가?
- gameplay commit과 persistence/transport/diagnostic I/O의 실패 의미가 명확한가?
- crash 후 durable state와 delivered result가 불일치할 수 있는가?
- log/trace/metrics가 authority와 분리되는가?
- long-running session의 cleanup, cancellation, backpressure와 resource ownership이 표현되는가?
- single-thread assumption이 명시되어 있고 future concurrency를 불필요하게 domain에 침투시키지 않는가?

Server code는 감사하지 않는다. Core가 host에 요구하거나 제공하는 port/contract가 설계 가능한지를 평가한다.

### A08. Data·protocol·persistence evolution

확인 항목:

- raw, parsed, validated, compiled, published data type-state가 구분되는가?
- data schema 변경이 runtime domain layout과 분리되는가?
- command/result/error/snapshot/event contract가 versionable한가?
- purpose-built DTO와 domain serde reuse가 일관적인가?
- save/checkpoint schema가 live state와 독립적으로 version/migrate/validate 가능한가?
- 신규 boss/skill/relic 데이터가 global validator와 runtime type을 얼마나 확장하는가?
- pre-release 자유도를 활용해 불필요한 legacy path를 제거할 수 있는가?

### A09. 성능·resource budget

기본 합격 의미:

> 동일한 대표 workload에서 새 architecture는 current baseline보다 의미 있게 느려지거나 더 많은 memory/payload를
> 사용하지 않으며, 장기 session의 state와 work growth에 설명 가능한 상한을 둔다.

확인 항목:

- tick/command마다 clone, full scan, full DTO/JSON projection이 일어나는가?
- unit, projectile, effect, boss phase, log 길이에 따른 시간복잡도가 무엇인가?
- queue, archive, cache, dedupe, graveyard, checkpoint와 payload가 무제한 성장하는가?
- transaction safety를 위해 전체 aggregate clone이 필요한가, 더 작은 staged state가 가능한가?
- Rapier world와 derived index를 반복 재구성하는가?
- data-driven input이 recursion, fan-out, event cascade를 무제한 늘리는가?

Goal은 대표 workload를 정의하고 current measurement를 먼저 기록한 뒤 `PERFORMANCE_BUDGET.md`에 provisional budget을
제안한다. Arbitrary absolute 숫자는 사용하지 않는다. 권장 기본 원칙은 다음과 같다.

- common command/tick latency: current median/p95 비퇴행
- long battle: tick budget의 명시적 headroom 확보
- delta/query cost: 반환량 또는 indexed lookup에 비례
- retained state: event/time/byte 중 최소 하나의 deterministic bound
- snapshot/save payload: representative run 기준 current 대비 비퇴행

### A10. 결정론·identity·continuation

확인 항목:

- same state/seed/commands가 ordered result/event/final state를 재현하는가?
- HashMap/heap/physics/float/time ordering 규칙이 explicit한가?
- UUID/RNG stream이 observation 또는 unrelated feature 추가에 과도하게 흔들리는가?
- save/continue 뒤 command sequence가 uninterrupted execution과 같은가?
- archive/debug projection이 gameplay identity를 바꾸지 않는가?
- subsystem rewrite 후 behavior equivalence를 측정할 canonical representation이 있는가?

### A11. AI-assisted solo development 지속성

확인 항목:

- source of truth가 명확해 AI가 동일 개념의 parallel implementation을 만들지 않는가?
- 한 변경에 필요한 context가 bounded하고 문서화 가능한가?
- compiler와 generated/exhaustive tables가 누락을 잡는가?
- test oracle이 AI가 만든 구현의 의미를 사용자 정책 수준에서 검토할 수 있게 하는가?
- speculative abstraction과 wrapper가 실제 variation보다 빠르게 늘어나는가?
- dead/legacy path를 안전하게 발견·제거할 evidence가 있는가?
- 대형 변경을 vertical slice로 쪼개고 매 단계 equivalence를 증명할 수 있는가?

AI 사용 자체는 부채가 아니다. 사람이 정책 결정을 내리고 automation이 구현을 확장할 때 구조적 검증 비용이
통제되는지를 평가한다.

### A12. Migration과 rewrite 결정

다음 대안을 최소 비교한다.

1. current architecture 유지 + defect remediation
2. current architecture 내부의 module/visibility refactor
3. application/protocol 계층의 단계적 재작성
4. subsystem 단위 재작성
5. 전체 core big-bang rewrite

각 대안에 대해 다음을 기록한다.

- latest rulebook/승인된 behavior 보존 위험과 stale runtime drift 처리 위험
- 의미 재발견 비용
- 재사용 가능한 domain/data/test/persistence 자산
- old/new equivalence 검증 가능성
- migration seam과 single-authority 유지 가능성
- 중간 상태에서 feature 개발 가능성
- current performance 비퇴행 검증 가능성
- 1인+AI 환경의 context·검토 부담
- 실패 시 되돌릴 수 있는 범위

전체 rewrite를 권고하려면 최소한 다음이 모두 증명되어야 한다.

- 핵심 state/domain boundary가 subsystem refactor로 분리될 현실적인 seam이 없음
- 현재 tests/contract를 new implementation oracle로 재사용하기 어려움
- 보존할 algorithm/data/persistence 자산보다 의미 재발견 비용이 낮음
- incremental migration이 dual authority 또는 더 큰 장기 위험을 필연적으로 만듦
- full rewrite의 target architecture와 behavior-equivalence 계획이 구체적임

반대로 일부 안전한 path가 존재한다는 사실만으로 current architecture 전체를 유지하지 않는다. Subsystem별로
`RETAIN`, `REFACTOR`, `REWRITE_BOUNDARY`, `REWRITE_SUBSYSTEM`, `FULL_REWRITE_CANDIDATE` 중 하나를 판정한다.

## 10. Criterion 판정 상태

| 상태 | 의미 |
| --- | --- |
| `PASS` | 조사 범위와 대표 scenario가 명시되고 현재 구조의 적합성을 뒷받침하는 증거가 있음 |
| `PARTIAL` | 일부 subsystem/scenario는 적합하지만 중요한 예외나 미래 pressure가 미확인 |
| `FAIL` | 현재 구조가 대표 change, 책임 경계, 검증 또는 운영 요구를 반복적으로 방해함이 증명됨 |
| `UNKNOWN` | source, workload, policy 또는 evidence가 부족해 판정 불가 |
| `N/A` | 현재 runtime과 승인 범위에 해당 축이 실제로 존재하지 않음 |

Criterion PASS/FAIL과 rewrite 선택은 동일하지 않다. FAIL이 많아도 보존 자산과 migration seam이 강하면 부분
재작성이 옳을 수 있고, 소수의 root FAIL이 모든 변화를 막으면 넓은 재작성이 필요할 수 있다.

## 11. Architecture finding 심각도와 확신도

### 11.1 심각도

| 등급 | 의미 |
| --- | --- |
| `S0 Blocker` | rulebook 핵심 기능 보존, deterministic correctness 또는 실행 가능한 migration을 막음 |
| `S1 High` | 여러 주요 subsystem의 변경을 반복적으로 손상시키거나 넓은 rewrite/migration 위험을 만듦 |
| `S2 Medium` | 한 subsystem의 확장·테스트·운영을 지속적으로 어렵게 하거나 중요한 수동 동기화를 요구함 |
| `S3 Low` | 국소적인 응집도, visibility, naming, speculative abstraction 또는 인지 비용 문제 |

Runtime bug severity와 architecture severity를 혼동하지 않는다. 실제 상태 손상은 영향 증거에 포함하지만, 이
Goal의 심각도는 장기 변경·검증·migration 영향도 함께 본다.

### 11.2 확신도

| 확신도 | 의미 |
| --- | --- |
| `Confirmed` | representative scenario, compiler dependency, test/probe 또는 반복 current path로 직접 확인됨 |
| `Strong inference` | current type/call/data flow가 결과를 사실상 증명하지만 동적 변경 실험은 없음 |
| `Hypothesis` | 위험 신호 또는 문서 pressure는 있으나 실제 증폭·영향·대안 비교가 미완료 |

## 12. Finding 증거 계약

최종 finding은 최소한 다음 필드를 가진다.

```markdown
## ARCH-REVIEW-###. <제목>

- 최초 기록 시각:
- 마지막 갱신 시각:
- 기준선:
- criterion ID:
- 상태: Hypothesis | Active | Rejected | Superseded
- 심각도:
- 확신도:
- 관련 domain/subsystem:
- 평가한 change scenario 또는 운영 요구:
- current owner/type/module:
- production entry와 dependency/data flow:
- raw observation:
- 실제 변경 증폭 또는 제한:
- compiler/test가 보호하는 범위:
- 사용자·개발·운영 영향:
- 보존해야 할 자산:
- 반대 증거와 불확실성:
- 근본 원인:
- 연관 finding:
- 가능한 개선 대안:
- migration seam 또는 blocker:
- rewrite decision에 미치는 영향:
- 필요한 검증:
- 연결된 experiment/evidence:
```

단순 line count나 import count finding은 허용하지 않는다. 대표 변경에서 서로 다른 표현·권위·validation·contract를
얼마나 수동으로 동기화해야 하는지, compiler/test가 누락을 잡는지, 더 좁은 대안이 가능한지를 설명한다.

반증된 후보는 삭제하지 않고 `Rejected`와 반증 근거를 보존한다.

## 13. Root-cause와 보존 자산 통합

최종 취합 시 finding만 root cause로 묶지 않는다. 다음 세 catalog를 병렬로 유지한다.

- structural pressure/root cause
- preservation asset
- migration seam/blocker

한 root cause가 여러 scenario에 나타나면 증상을 부풀리지 않고 공통 mechanism과 각 영향 경로를 연결한다.
반대로 owner, migration과 수정 전략이 다르면 비슷한 증상도 분리한다.

## 14. Rewrite decision protocol

Rewrite 결론은 모든 A00~A11 1차 판정, preservation asset register, scenario matrix와 migration seam map이 작성된
뒤에만 A12에서 내린다.

최종 보고는 다음을 분리한다.

```text
Goal Status
Current Architecture Assessment
Requirement Preservation Confidence
Incremental Refactor Viability
Partial Rewrite Viability
Full Rewrite Justification
Recommended Target Architecture
Migration Feasibility
```

권고는 최소 subsystem별 disposition과 단계 순서를 포함한다. 비용을 근거 없는 인월 숫자로 꾸며내지 않고,
relative size, semantic risk, dependency blast radius, oracle strength와 reversibility로 비교한다.

## 15. 권장 실행 순서

1. Goal 운영 문서와 criteria를 읽고 Status를 확인한다.
2. 후보 source의 identity, heading, 책임과 알려진 freshness를 catalog한 뒤 criterion·scenario별 관련 section을
   선택해 읽고 source map에 읽은 범위와 생략 이유를 기록한다.
3. dirty baseline과 최초 broad test/fmt/clippy 결과를 기록한다.
4. rulebook feature와 production surface를 연결한 requirement preservation map을 작성한다.
5. domain, aggregate, state lifecycle과 cross-domain workflow를 독립적으로 매핑한다.
6. import/call/data-flow dependency와 public API surface를 매핑한다.
7. 대표 change scenario를 실제 code/data/test 경로에 대입한다.
8. testability, operation/failure/I/O와 data/protocol/save evolution을 조사한다.
9. representative performance workload와 current budget을 측정·제안한다.
10. determinism, identity와 continuation equivalence evidence를 연결한다.
11. AI-assisted solo development 관점의 context와 verification cost를 평가한다.
12. A00~A11 1차 판정을 잠근다.
13. 이 시점에만 과거 구조 감사와 draft를 읽고 독립 결과를 cross-check한다.
14. architecture 대안과 subsystem disposition을 작성한다.
15. A12 rewrite decision과 completion audit을 끝낸다.

파일 순서대로 피상적으로 읽지 않는다. Domain workflow와 representative change scenario를 따라 vertical하게
읽고, bounded investigation 하나가 끝날 때마다 즉시 기록한다.

## 16. 필수 산출물

Goal 완료 시 최소한 다음 문서가 현재 evidence와 일치해야 한다.

1. `PLAN.md`
2. `EXPERIMENTS.md`
3. `EXPERIMENT_NOTES.md`
4. `FINDINGS.md`
5. `BASELINE.md`
6. `DOCUMENT_SOURCE_MAP.md`
7. `REQUIREMENT_PRESERVATION_MAP.md`
8. `DOMAIN_MODEL_MAP.md`
9. `AGGREGATE_STATE_MAP.md`
10. `DEPENDENCY_DIRECTION_MAP.md`
11. `CHANGE_SCENARIO_MATRIX.md`
12. `TESTABILITY_MAP.md`
13. `OPERATIONS_BOUNDARY_MAP.md`
14. `DATA_CONTRACT_EVOLUTION_MAP.md`
15. `DETERMINISM_CONTINUATION_MAP.md`
16. `PERFORMANCE_BUDGET.md`
17. `PRESERVATION_ASSET_REGISTER.md`
18. `ARCHITECTURE_OPTIONS.md`
19. `REWRITE_DECISION.md`
20. `EVIDENCE_MATRIX.md`
21. `GAP_REGISTER.md`
22. `COMPLETION_AUDIT.md`

추가 문서는 첫 관련 evidence가 생길 때 생성한다. 마지막에 기억으로 처음부터 복원하지 않는다. 복잡한 관계를
명확히 할 때만 diagram을 사용하고 작은 표·호출 경로로 충분하면 불필요한 대형 도식을 만들지 않는다.

## 17. 완료 조건

다음을 모두 만족해야 Goal을 `complete`로 변경할 수 있다.

- current dirty baseline과 source catalog identity가 재현 가능하게 기록됨
- criterion별 source routing, 실제 read range, section freshness와 근거 있는 skip/defer가 기록됨
- 최초 broad suite와 fmt/clippy 실제 결과가 기록됨
- latest rulebook/승인된 behavior의 preservation map과 stale runtime drift 분류가 주요 production entry와 연결됨
- 주요 domain, aggregate, phase, authority와 workflow가 목록화됨
- dependency direction이 import뿐 아니라 call/data flow로 조사됨
- 지정 change scenario 전부가 current code/data/test 경로에 대입됨
- testability, operation/failure/I/O, data/protocol/save evolution이 독립적으로 평가됨
- representative performance workload와 provisional non-regression budget이 기록됨
- determinism, identity, save/continue evidence와 gap이 연결됨
- A00~A12가 모두 판정되고 PASS/FAIL 근거가 evidence ID에 연결됨
- 모든 Active finding이 증거 계약을 만족함
- 모든 Hypothesis, Rejected, Superseded와 UNKNOWN이 보존됨
- preservation asset, root cause, migration seam/blocker가 병렬 catalog로 정리됨
- 과거 감사 cross-check가 독립 판정 이후에만 수행되었음
- 최소 다섯 architecture 대안이 동일 기준으로 비교됨
- subsystem별 retain/refactor/rewrite disposition이 있음
- full rewrite 권고 또는 비권고가 명시적 증거 요건을 만족함
- production code/test/live data/canonical contract가 수정되지 않음
- 최종 content identity check가 기준선 보존을 확인함
- Goal 완료와 architecture 합격/rewrite 결론이 분리됨

Goal이 완료되었다는 것은 현재 architecture가 합격이라는 뜻이 아니다. 승인된 범위의 설계와 rewrite 선택지가
재현 가능한 증거로 분류되었다는 뜻이다.

## 18. Goal 참조 문구

이 기준을 사용하는 Goal은 다음 의미를 포함한다.

```markdown
이 Goal은 `docs/audit/current_runtime_architecture_deep_review_criteria.ko.md`를 판정 기준과
rewrite decision 증거 계약의 source of truth로 사용한다. `game_rulebook.md`를 최신 gameplay 정책으로,
current runtime을 stale할 수 있는 구현 evidence로 취급한다. 후보 문서는 criterion과 scenario에 필요한
section만 progressive하게 읽고 선택·생략 근거를 기록한다. 현재 behavior는 정책 일치, 유효 technical contract,
stale drift와 미결정 behavior로 분류하며 current module/type/wire/save 구조를 영구 제약으로 간주하지 않는다.
과거 구조 감사와 design draft는 독립 domain/dependency/change-scenario 분석이 끝난 뒤에만 cross-check하며,
모든 판정은 고정된 현재 dirty baseline에서 다시 증명한다.
```
