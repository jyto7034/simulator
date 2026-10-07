# Current Runtime Architecture Deep Review

> 참조 상태 정정 (2026-09-06): 아래 과거 입력 목록의 일반 보스 알 메모는 개별 연출을 AI가 임의로
> 공통 정책으로 일반화한 문서여서 사용자 요청으로 삭제했다. 현재 검토 입력이나 승인 정책으로 사용하지 않는다.

> 인수인계 문서 정리 (2026-09-06): 과거 입력 목록의 파편·장비 인수인계 문서도 역할 중복으로 삭제했다.
> 최신 결정과 완료 상태는 유물 정책 및 파편·장비 설계 문서로 이관했으며, 아래 경로는 당시 입력 기록이다.

## Status

`complete`

실행 시작: `2026-08-22T06:44:27+09:00`

A00~A11 독립 판정을 E-022에서 잠갔고 E-023 past structural audit cross-check, E-024 A12 rewrite decision과 E-025
final identity/completion audit을 완료했다.

## Goal Name

`current_runtime_architecture_deep_review`

## 목표

현재 `core_enhanced` Rust runtime의 설계를 과거 원자성 finding이나 rewrite 선호에서 출발하지 않고 독립적으로
심층 검토한다.

검토의 최종 질문은 다음과 같다.

> 최신 정책인 Game Rulebook의 기능을 보존하고 stale할 수 있는 current runtime behavior를 분류하면서 boss
> runtime, movement, skill/battle, roguelike system을 장기적으로 확장할 때, current architecture를
> 유지·리팩터링·부분 재작성·전면 재작성 중 어떤 방식으로 진화시키는 것이 correctness, 성능, 검증 비용과
> 1인+AI 개발의 총위험을 최소화하는가?

## 해결할 사용자-visible 문제

기존 구조 감사는 현재 runtime의 결함과 위험을 증명했지만, 그것만으로 다음을 결정할 수 없다.

- 현재 domain boundary와 state model이 실제 기능 구조에 적합한가?
- 새 boss, movement, skill, roguelike 정책이 어디에 들어가며 무엇을 함께 변경하는가?
- 큰 코드 규모가 풍부한 domain 때문인지 잘못된 책임 결합 때문인지?
- 기존 test/data/persistence/battle logic 중 무엇을 보존해야 하는가?
- 출시 전·일정 제약 없음이라는 조건에서 refactor보다 rewrite가 더 경제적인 범위가 있는가?
- AI가 구현하고 사용자가 정책만 검토하는 방식에서 architecture가 검증 가능한 context를 제공하는가?

이 Goal은 이를 대표 change scenario와 current production code evidence로 분류하고 subsystem별 disposition과 최종
rewrite decision을 제공한다.

## 판정 Source of Truth

판정 기준과 rewrite decision 증거 계약의 source of truth:

- `docs/audit/current_runtime_architecture_deep_review_criteria.ko.md`

이 Goal은 위 문서의 A00~A12, 판정 상태, severity/confidence, finding contract, rewrite protocol과 완료 조건을
완화하지 않는다.

`game_rulebook.md`는 최신 gameplay 정책의 source of truth다. 현재 production Rust는 current implementation
structure와 behavior의 evidence지만 최신 정책 source는 아니다. Core/data/skill contract는 relevant section마다
current technical contract인지 stale한 의도인지 판정한다. Gameplay 의미가 충돌하면 Rulebook이 우선하고,
technical contract는 Rulebook과 양립하는 detail만 보완한다. Current code는 policy를 덮어쓰지 않는다. Rulebook으로
확정할 수 없는 충돌은 `policy_undecided`로 격리한다.

## 사용자 승인 정책과 근거

다음 사항은 2026-08-22 사용자 입력으로 승인되었다.

- 검토 code scope는 `core_enhanced`만이다.
- game_server는 소규모이며 rewrite할 수 있다.
- Unity도 rewrite할 수 있고 stale contract compatibility는 주요 제약이 아니다.
- Rulebook 기능은 임의로 버리지 않는다. Current behavior는 제거 전에 inventory하고 정책 일치·유효 technical
  contract·stale implementation drift·미결정으로 분류하지만 존재만으로 보존 requirement가 되지는 않는다.
- 주요 미래 pressure는 boss runtime, movement system, skill/battle system, roguelike system이다.
- 성능은 현재보다 나빠지지 않아야 하며 가능하면 더 가벼운 구조를 선호한다.
- 합리적인 provisional performance budget은 검토자가 current evidence를 측정한 뒤 제안할 수 있다.
- 제품은 출시 전이다.
- 개발 시간과 출시 약속의 제약이 없다.
- 팀은 사용자 1인이 정책을 검토하고 고성능 AI가 구현을 지원하는 방식이다.

이 전제는 full rewrite를 자동으로 정당화하지 않는다. Rewrite 자유도는 높지만 gameplay 의미 재발견, test oracle,
determinism, data와 persistence 자산의 유실 위험은 별도로 평가한다.

## Candidate source catalog와 routing

목록에 있다는 이유만으로 전체 본문을 읽거나 동일한 권위를 부여하지 않는다. 먼저 경로, hash, heading 구조,
알려진 freshness와 예상 책임만 catalog하고, 각 criterion과 change scenario에 필요한 section만 읽는다. 읽은
section/range와 선택·생략 이유는 `DOCUMENT_SOURCE_MAP.md`에 남긴다.

### Latest policy와 technical contract 후보

- `docs/game_rulebook.md`: 최신 gameplay 정책과 보존할 기능 inventory
- `docs/core_runtime_contract.md`: authority/lifecycle technical contract 후보; section별 freshness 확인
- `docs/data_loading_contract.md`: authoring/loading/publication technical contract 후보; section별 freshness 확인
- `docs/skill_target_contract.md`: targeting/delivery/interrupt technical contract 후보; section별 freshness 확인

### Future change pressure

- `docs/tmp/run_relic_system_design_notes_2026-08-21.ko.md`
- `docs/tmp/handoff_skill_fragment_design_2026-08-20.ko.md`
- `docs/tmp/normal_boss_egg_design_notes_2026-08-19.ko.md`
- `docs/tmp/ego_corrosion_system_design_notes_2026-08-19.ko.md`
- `docs/tmp/generic_dice_and_tool_item_rework_notes_2026-08-17.ko.md`
- `docs/tmp/skill_activation_and_resonance_rework_notes_2026-08-17.ko.md`

이 문서는 목적과 정책 pressure를 제공하지만 current implementation 완료를 증명하지 않는다. 해당 scenario를
평가하지 않거나 Rulebook과 code만으로 충분하면 생략 이유를 기록하고 읽지 않는다.

### Boss/skill/equipment stress case

- `docs/skills/abnormality_design_notes.ko.md`
- `docs/skills/abnormality_skill_fragments_design_notes.md`

전체 콘텐츠 품질을 평가하지 않고 선택한 representative mechanic에 필요한 section에서만 runtime 표현력,
variation과 change pressure를 추출한다.

### Historical reference

- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`

Stale historical consumer intent로만 사용한다. Current compatibility나 target wire schema의 source가 아니며,
adapter/history 질문에 실제 필요할 때 관련 section만 읽는다.

### Progressive source routing

1. 현재 criterion, domain workflow 또는 change scenario의 질문을 먼저 적는다.
2. 질문에 답할 후보 section과 직접 필요한 정의·전제·cross-reference를 선택한다.
3. Heading이나 snippet만으로 판정하지 않고 선택 범위를 문맥이 충분할 만큼 읽는다.
4. Source 충돌·용어 불명·미해결 의존이 있을 때만 범위를 확장한다.
5. 읽은 범위, 목적, source status와 읽지 않은 범위의 이유를 즉시 기록한다.

근거 있는 `irrelevant` 판정과 생략은 gap이나 blocker가 아니다. Completion은 모든 candidate 문서의 소비량이
아니라 criterion과 representative scenario의 evidence coverage로 판정한다.

## 기존 감사와 독립성

독립 domain/dependency/change-scenario mapping과 A00~A11 1차 판정 전에는 다음을 읽거나 search scope에 포함하지
않는다.

- `docs/goals/current_runtime_structural_integrity_audit/**`
- `docs/goals/current_runtime_structural_integrity_audit/DESIGN_REVIEW_DRAFT.md`
- 기타 과거 `docs/goals/**`

독립 판정을 잠근 뒤 cross-check한다. Cross-check 목적은 다음뿐이다.

- 독립 분석의 누락 확인
- 같은 current fact의 재증명 또는 반증
- 과거 defect 중심 감사와 architecture scenario 중심 검토의 결론 차이 설명
- previous draft의 유지·수정·폐기 항목 분류

과거 finding ID나 verdict를 현재 finding으로 가져오지 않는다.

## 핵심 평가축

1. Baseline과 requirement preservation
2. Domain 구조와 ubiquitous language
3. Aggregate/state/invariant 표현
4. Dependency direction과 compiler enforcement
5. Extension scenario fitness
6. Change amplification과 cognitive load
7. Testability와 behavior oracle
8. Operation/failure/I/O boundary
9. Data/protocol/persistence evolution
10. Performance/resource budget
11. Determinism/identity/continuation
12. AI-assisted solo development sustainability
13. Migration/rewrite economics

각 축은 독립적으로 조사한다. 한 finding이 여러 criterion에 영향을 줄 수 있지만 criterion FAIL 수를 rewrite
점수로 사용하지 않는다.

## 유지할 불변식

### Goal 수행 불변식

- production code, tests, live data와 canonical contract를 수정하지 않는다.
- 기존 dirty 변경을 reset, restore, checkout, clean, stash, format, move 또는 delete하지 않는다.
- Goal 문서 변경과 사용자 production 변경을 구분한다.
- 서로 다른 baseline의 evidence를 섞지 않는다.
- source는 criterion에 필요한 문맥까지 읽되 관련 없는 section을 일괄 정독하지 않는다.
- past audit에 anchoring되지 않도록 독립 1차 판정 전 접근 금지를 지킨다.
- line count, public field, clone, panic, lint와 AI 생성 자체를 단독 finding으로 사용하지 않는다.
- 최신 Rulebook 기능과 사용자 승인/current technical contract로 확인된 semantics를 rewrite 편의를 위해 삭제하지
  않는다. Stale current behavior는 분류 없이 보존하거나 폐기하지 않는다.
- current Rust module/type/wire/save 구조를 영구 compatibility requirement로 오해하지 않는다.
- architecture critique에는 realistic alternative와 migration seam/blocker를 연결한다.
- production finding remediation이나 prototype을 이 Goal에서 구현하지 않는다.
- 새 Goal을 자동 생성하지 않는다.

### Evidence 기록 불변식

- bounded investigation 결과를 다음 조사 전에 기록한다.
- 실제 실행하지 않은 명령을 result처럼 기록하지 않는다.
- source role과 precedence 변경을 `DOCUMENT_SOURCE_MAP.md`에 즉시 기록한다.
- change scenario마다 수정 표현, owner, validation, data, contract, persistence, tests와 compiler guard를 기록한다.
- finding 후보는 즉시 `Hypothesis`로 등록한다.
- preservation asset과 migration seam도 finding만큼 즉시 기록한다.
- rejected/superseded finding과 correction history를 삭제하지 않는다.
- test/benchmark 수치는 runner가 실제 보고한 값과 measurement environment만 사용한다.
- full rewrite 결론은 A00~A11 1차 판정과 cross-check 전에 내리지 않는다.

## 범위

### 포함 범위

- `Cargo.toml`
- `src/lib.rs`
- `src/game/**`
- contract와 behavior evidence에 필요한 `tests/**`와 inline tests
- command/query/event/result/snapshot/error/DTO serialization
- data parse/validate/compile/publication
- save/checkpoint/restore/reconnect core boundary
- RNG, UUID, time, Rapier/physics, filesystem diagnostic boundary
- representative hotspot 판단에 필요한 local Git history
- 사용자 지정 문서와 필요 시 `../game_resources/data`의 bounded read-only evidence

### 비범위

- game_server code audit
- Unity code audit
- current server/Unity compatibility certification
- production remediation, refactor와 prototype implementation
- gameplay balance, content completeness와 fun
- 문서에 없는 새 정책 결정
- 확정되지 않은 architecture를 canonical contract에 반영
- rewrite migration 실제 실행

## 이 Goal의 단일 책임

단일 결과는 다음이다.

> 보존 요구와 대표 미래 변경을 current core architecture에 대입한 evidence-based architecture catalog,
> 동일 기준으로 비교한 대안, subsystem disposition과 최종 rewrite decision.

결함 수정, target architecture 구현, data/save/wire migration과 Unity/server 재작성은 별도 승인된 후속 작업이다.

## 미결정 정책

현재 알려진 미결정 사항:

- long-battle event/time/byte hard cap의 제품 숫자
- target server tick deadline과 동시 session 수
- save/wire backward compatibility를 실제로 얼마나 유지할지
- future boss/relic/corrosion 세부 규칙

이 Goal은 current workload를 측정해 provisional performance budget과 architecture recommendation을 제안할 수 있다.
Gameplay 숫자와 live production limit은 recommendation으로 표시하고 canonical policy로 확정하지 않는다.

정책 없이도 구조를 평가할 수 있으면 UNKNOWN 또는 explicit assumption으로 격리하고 Goal 전체를 중단하지 않는다.
A12 결론이 정책 선택에 따라 반대로 바뀌며 격리할 수 없으면 `policy_decision_required`로 전환한다.

## 구현 및 Migration 범위

- Production 구현: 없음
- DTO migration: 설계 대안만 허용, 구현 없음
- Data/live RON migration: 설계 대안만 허용, 구현 없음
- Save migration: 전략과 feasibility 평가만 허용, 구현 없음
- Crate/module split: target option만 허용, 구현 없음
- Performance optimization: 후보와 budget만 허용, 구현 없음

허용되는 write는 이 Goal 디렉터리의 산출물과 이 Goal을 위한 criteria 문서뿐이다. Probe source/binary는 `/tmp`를
사용한다.

## Baseline 계획

### Baseline identification

Goal 실행 시작 후 candidate source catalog를 만들고 production code 조사 전에 다음을 기록한다.

- 시작 시각과 timezone
- `git rev-parse HEAD`
- `git status --short`
- `Cargo.toml`, `src/lib.rs`, `src/game/**`, `tests/**`, candidate source 문서의 file manifest와 aggregate hash
- 각 candidate source 문서의 byte size와 SHA-256
- 필요 시 bounded `../game_resources/data` snapshot
- rustc/cargo version
- OS/architecture와 feature

현재 repository는 대규모 dirty 상태다. Commit ID만으로 baseline을 식별하지 않는다.

### 최초 broad baseline

다음을 각각 정확히 한 번 실행한다.

```bash
cargo test --all-targets --all-features
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

All-pass suite는 architecture 전체 합격 증거가 아니다. Baseline이 변하지 않으면 동일 suite와 baseline에 포함된
focused test를 안심 목적으로 반복하지 않는다.

### 추가 probe

정적 trace와 existing tests만으로 다음을 증명할 수 없을 때만 사전 기록 후 허용한다.

- representative current performance 분포
- behavior/determinism/continuation equivalence
- compiler/API boundary 또는 change scenario의 구체적 fan-out
- retained state/payload growth

Production test 파일은 수정하지 않고 `/tmp` harness를 우선한다.

## Representative Change Scenario

최소 scenario set:

1. Boss runtime에 새로운 phase/summon/omen/특수 승패 조건 추가
2. Movement에 새로운 collision/occupancy/route 또는 unit movement mode 추가
3. Skill에 새로운 activation/resonance/target/delivery/interrupt 조합 추가
4. Run relic와 monolith progression 추가·확장
5. Skill fragment 성장·각성·침식/개화 형태 확장
6. EGO corrosion의 lifecycle와 combat/run consequence 추가
7. Generic dice/tool item과 equipment combat trigger 확장
8. 새로운 roguelike node/session/reward/shop/facility 추가
9. 장기 battle과 다수 unit/projectile/effect를 처리하며 reconnect delta 제공
10. Save/continue 뒤 위 기능들이 uninterrupted run과 같은 결과 유지

각 scenario는 다음 질문에 답한다.

- 어느 domain/aggregate가 소유해야 하는가?
- 현재는 어떤 type/module이 소유하는가?
- 몇 종류의 representation을 수정해야 하는가?
- data validation, runtime execution, protocol, persistence와 test가 어떻게 연결되는가?
- 누락을 compiler가 잡는가?
- unrelated regression 범위는 어디까지인가?
- current architecture에서 구현, refactor 후 구현, subsystem rewrite 중 어느 것이 좁은가?

## Performance 계획

Current 대비 비퇴행이 primary contract다. Goal은 current measurement 전에 임의 absolute budget을 확정하지 않는다.

최소 workload 후보:

- common non-battle command
- battle pause/resume/target/skill/tactical command
- representative server-tick battle advance
- boss runtime event-heavy step
- 1K/10K/100K retained presentation history에서 constant-tail query
- representative run snapshot/checkpoint export
- increasing unit/projectile/effect count scaling

실제 code/contract를 읽고 representative fixture를 확정한다. `PERFORMANCE_BUDGET.md`에는 다음을 기록한다.

- measurement environment와 build profile
- input fixture와 반복 수
- current median/p95 또는 안전하게 계산 가능한 분포
- time/space complexity model
- current 대비 non-regression threshold
- desired headroom
- product policy가 필요한 provisional cap

Benchmark 숫자만으로 rewrite를 결정하지 않고 정확성·결정론·구조 비용과 함께 판단한다.

## 검토 단계

### Phase 0. Goal/source/baseline

1. Goal Status와 current phase에 필요한 criteria/PLAN/기록 section 확인
2. Candidate source identity·heading·freshness catalog와 criterion별 progressive section routing
3. 읽은 범위·source status·생략 이유를 포함한 `DOCUMENT_SOURCE_MAP.md` 작성
4. dirty baseline과 broad test/fmt/clippy

### Phase 1. Requirement와 current surface

5. Rulebook capability inventory
6. Production entry/data/result/test 연결
7. `REQUIREMENT_PRESERVATION_MAP.md` 작성

### Phase 2. Domain과 aggregate

8. Domain responsibility와 ubiquitous language
9. Aggregate, lifecycle, phase, authority, derived state
10. Cross-domain application workflow
11. `DOMAIN_MODEL_MAP.md`, `AGGREGATE_STATE_MAP.md` 작성

### Phase 3. Dependency와 change pressure

12. Import/call/data-flow/public API map
13. Desired one-way dependency option
14. Representative change scenario 1~10
15. `DEPENDENCY_DIRECTION_MAP.md`, `CHANGE_SCENARIO_MATRIX.md` 작성

### Phase 4. Verification과 operation

16. Test seams, fixture, oracle와 failure injection
17. Host/core, persistence, retry, reconnect, I/O와 observation
18. Data/wire/save evolution
19. `TESTABILITY_MAP.md`, `OPERATIONS_BOUNDARY_MAP.md`, `DATA_CONTRACT_EVOLUTION_MAP.md` 작성

### Phase 5. Performance와 continuity

20. Representative workload와 current measurement
21. Resource growth와 provisional budget
22. Determinism/identity/save continuation evidence
23. `PERFORMANCE_BUDGET.md`, `DETERMINISM_CONTINUATION_MAP.md` 작성

### Phase 6. Independent verdict lock

24. A00~A11 1차 판정
25. Root pressure, preservation asset, migration seam/blocker 정리
26. `PRESERVATION_ASSET_REGISTER.md` 작성

### Phase 7. Past-audit cross-check

27. 최초로 past structural audit와 design draft 접근
28. 독립 결과와 일치·차이·누락·반증 기록
29. 필요한 current evidence를 bounded investigation으로 다시 확인

### Phase 8. Alternatives와 rewrite decision

30. 다섯 architecture option을 같은 기준으로 비교
31. Subsystem별 disposition 결정
32. Recommended target architecture와 migration stages
33. A12 판정, `ARCHITECTURE_OPTIONS.md`, `REWRITE_DECISION.md` 작성

### Phase 9. Completion

34. A00~A12 final evidence matrix
35. Gap/UNKNOWN/finding history/root cause 정리
36. Final production/document contamination check
37. `COMPLETION_AUDIT.md`와 Status 완료

각 단계의 bounded investigation은 다음 단계 전에 즉시 기록한다.

## 필수 산출물

Criteria가 요구하는 다음 문서를 완료한다.

- `PLAN.md`
- `EXPERIMENTS.md`
- `EXPERIMENT_NOTES.md`
- `FINDINGS.md`
- `BASELINE.md`
- `DOCUMENT_SOURCE_MAP.md`
- `REQUIREMENT_PRESERVATION_MAP.md`
- `DOMAIN_MODEL_MAP.md`
- `AGGREGATE_STATE_MAP.md`
- `DEPENDENCY_DIRECTION_MAP.md`
- `CHANGE_SCENARIO_MATRIX.md`
- `TESTABILITY_MAP.md`
- `OPERATIONS_BOUNDARY_MAP.md`
- `DATA_CONTRACT_EVOLUTION_MAP.md`
- `DETERMINISM_CONTINUATION_MAP.md`
- `PERFORMANCE_BUDGET.md`
- `PRESERVATION_ASSET_REGISTER.md`
- `ARCHITECTURE_OPTIONS.md`
- `REWRITE_DECISION.md`
- `EVIDENCE_MATRIX.md`
- `GAP_REGISTER.md`
- `COMPLETION_AUDIT.md`

첫 evidence가 생길 때 생성하고 즉시 갱신한다. 초기화 시점에는 운영 재개에 필요한 core catalog 문서만 생성한다.

## Finding ID와 기록

- ID: `ARCH-REVIEW-###`
- 상태: `Hypothesis`, `Active`, `Rejected`, `Superseded`
- confidence: `Confirmed`, `Strong inference`, `Hypothesis`
- severity: `S0 Blocker`, `S1 High`, `S2 Medium`, `S3 Low`

Finding은 bug count가 아니라 architecture pressure catalog다. 같은 root cause의 여러 증상을 부풀리지 않고 scenario와
affected path로 연결한다. 보존 자산과 migration seam은 별도 positive catalog에 기록한다.

## Rewrite 대안

최소한 다음을 같은 evidence로 비교한다.

1. 유지 + defect remediation
2. module/visibility 중심 refactor
3. application/protocol shell 단계적 rewrite
4. subsystem rewrite 조합
5. entire core big-bang rewrite

필요하면 sixth hybrid option을 추가할 수 있지만 불리한 strawman을 만들지 않는다.

Subsystem disposition:

- `RETAIN`
- `REFACTOR`
- `REWRITE_BOUNDARY`
- `REWRITE_SUBSYSTEM`
- `FULL_REWRITE_CANDIDATE`

최종 `REWRITE_DECISION.md`는 각 subsystem의 현재 책임, preservation asset, pressure, option, migration order,
verification gate와 reversibility를 연결한다.

## 사용자-visible 완료 조건

다음을 모두 충족해야 한다.

- 기준선과 candidate source identity가 재현 가능함
- criterion별 실제 source section, 선택 이유, 판정한 freshness와 생략 이유의 trace가 있음
- broad test/fmt/clippy의 정확한 결과가 있음
- latest rulebook/승인된 behavior preservation map과 stale runtime drift 분류가 production surface와 연결됨
- domain/aggregate/state/dependency가 call/data flow 수준으로 매핑됨
- 대표 change scenario 10개가 실제 code/data/test에 대입됨
- testability, operation, data/protocol/save evolution이 독립 판정됨
- current performance와 provisional budget이 representative input과 연결됨
- determinism/identity/continuation evidence와 gap이 있음
- A00~A11 1차 판정이 past audit 접근 전에 잠겼음
- cross-check의 일치·차이·누락이 기록됨
- A00~A12 모두 verdict와 evidence ID가 있음
- finding, rejected candidate, preservation asset, migration seam/blocker가 보존됨
- 다섯 architecture option이 같은 기준으로 비교됨
- subsystem disposition과 target architecture가 있음
- full rewrite 권고/비권고가 criteria의 evidence requirement를 만족함
- production/test/live data/canonical contract를 수정하지 않음
- dirty baseline final hash가 일치함
- 최종 identity check는 production/test, Rulebook과 실제 evidence로 사용한 source를 대상으로 하며 irrelevant candidate
  변경만으로 baseline을 무효화하지 않음
- Goal Status와 architecture/rewrite assessment가 분리됨

## 중단 조건

### `scope_decision_required`

- core-only evidence로 architecture/rewrite 결론을 안전하게 취합할 수 없음
- server 또는 Unity 전체 code audit이 필수가 됨
- 독립된 제품을 한 Goal에 합치지 않으면 결론이 달라짐

### `policy_decision_required`

- 기능 보존 여부나 workload policy가 A12를 반대로 만들며 UNKNOWN으로 격리 불가

### `baseline_changed`

- production/test/live data/사용 evidence source/dependency/environment가 Goal 실행 중 변경됨

### `externally_blocked`

- 현재 criterion에 필요한 source, permission, dependency, test runner 또는 재현 가능한 baseline 없음

중단 시 안전하게 완료한 조사, 남은 범위와 필요한 사용자 입력을 즉시 기록한다.

## Compatibility와 Migration

Current server/Unity compatibility는 주요 제약이 아니지만 current wire/save/data 의미를 이해하는 evidence는
필요하다. Compatibility를 유지할지 버릴지는 subsystem별 architecture option에서 비용과 preservation risk로
다룬다.

Migration 구현은 비범위다. 그러나 single authority, old/new behavior equivalence, rollback point와 단계별 release
가능성을 설명하지 못하는 target architecture는 실행 가능한 대안으로 인정하지 않는다.

## Canonical 문서

- 판정 기준: `docs/audit/current_runtime_architecture_deep_review_criteria.ko.md`
- 보존 기능: 최신 정책인 `docs/game_rulebook.md`와 사용자 승인/current technical contract로 확인된 behavior
- Current intended contract 후보: 지정 core/data/skill contract의 relevant section; Rulebook·code 대조 후 상태 판정
- Future pressure: 실제 scenario에 선택한 tmp/boss/skill-fragment 설계 section
- Historical consumer intent: stale Unity contract
- Current implementation: 고정된 dirty production Rust baseline
- Execution history: 이 Goal 디렉터리의 문서

Goal은 canonical gameplay/runtime 문서를 수정하지 않는다. Target architecture가 승인되고 구현 evidence가 생긴
후 별도 작업에서 canonical 문서를 갱신한다.

## 완료 판정

`COMPLETION_AUDIT.md`가 모든 사용자-visible 완료 조건을 evidence ID와 산출물에 연결한 뒤에만 Status를
`complete`로 바꾼다.

다음은 완료 evidence가 아니다.

- full test all-pass만으로 architecture 합격
- 파일 크기/import/lint 수만으로 rewrite 권고
- past audit finding 재사용
- 문서 의도를 current code behavior처럼 표현
- server/Unity code를 보지 않고 그 구현을 검증했다고 표현
- current 구조의 defect 수를 rewrite 점수로 합산
- target architecture diagram만 작성하고 migration/oracle을 생략
- 후보와 UNKNOWN을 삭제해 gap이 없는 것처럼 표현

Status가 `complete`여도 Current Architecture Assessment는 FAIL일 수 있고 Rewrite Decision은 부분 또는 전체
rewrite일 수 있다. 반대로 current architecture가 불완전해도 Goal evidence catalog는 complete일 수 있다.
