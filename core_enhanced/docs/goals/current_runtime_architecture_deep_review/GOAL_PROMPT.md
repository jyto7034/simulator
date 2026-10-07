# Goal Execution Prompt

> 참조 상태 정정 (2026-09-06): 아래 과거 입력 목록의 일반 보스 알 메모는 개별 연출을 AI가 임의로
> 공통 정책으로 일반화한 문서여서 사용자 요청으로 삭제했다. 현재 검토 입력이나 승인 정책으로 사용하지 않는다.

> 인수인계 문서 정리 (2026-09-06): 과거 입력 목록의 파편·장비 인수인계 문서도 역할 중복으로 삭제했다.
> 최신 결정과 완료 상태는 유물 정책 및 파편·장비 설계 문서로 이관했으며, 아래 경로는 당시 입력 기록이다.

아래 내용을 현재 workspace에서 하나의 Goal로 끝까지 수행하라.

## Goal

- `goal_name`: `current_runtime_architecture_deep_review`
- Goal 디렉터리: `docs/goals/current_runtime_architecture_deep_review`
- 대상 runtime: `/mnt/f/work/simulator/core_enhanced`
- repository root: `/mnt/f/work/simulator`
- Status 시작값: `PLAN.md`의 최신 `Status`를 따른다.

현재 `core_enhanced` Rust runtime을 원자성 결함이나 과거 finding에서 출발하지 않고 독립적인 architecture
평가축으로 read-only 심층 검토하라. Domain 구조, aggregate/state 표현, 의존 방향, 확장성, 변경 용이성,
테스트 가능성, 운영·I/O 경계, data/protocol/persistence evolution, 성능, 결정론, AI-assisted solo development와
migration 경제성을 증명하고 전체 rewrite 여부를 마지막에 판정하는 것이 목표다.

Production code를 고치는 Goal이 아니다. 최신 정책인 Game Rulebook의 기능을 보존하고 stale할 수 있는 current
behavior를 정책 일치·유효 technical contract·구현 drift·미결정으로 분류하면서 어떤 자산을 유지, 리팩터링,
경계 재작성, subsystem 재작성 또는 전체 재작성해야 하는지 evidence로 판단한다.

## 판정 Source of Truth

다음 문서를 판정 기준, finding evidence contract, rewrite decision protocol과 완료 조건의 source of truth로 사용하라.

- `docs/audit/current_runtime_architecture_deep_review_criteria.ko.md`

이 문서의 A00~A12, 판정 상태, 심각도, 확신도, source precedence, 필수 산출물과 완료 조건을 완화하거나
대체하지 마라.

## 사용자 승인 전제

다음은 이 Goal의 고정 전제다.

- 코드 감사 범위는 `core_enhanced`다.
- game_server와 Unity는 소규모이며 필요하면 재작성할 수 있으므로 current compatibility는 주요 rewrite 제약이
  아니다.
- `game_rulebook.md`는 최신 gameplay 정책이다. Current runtime은 stale할 수 있으므로 current behavior를 먼저
  inventory·분류하되 그 존재만으로 보존 requirement로 승격하지 않는다.
- 주요 미래 pressure는 boss runtime, movement, skill/battle, roguelike system과 지정 policy note다.
- 성능은 대표 workload에서 current baseline보다 악화하지 않는 것이 최소 조건이며 가능한 한 가벼운 구조를
  선호한다.
- 제품은 출시 전이고 일정 제약이 없다.
- 개발은 사용자 정책 검토와 고성능 AI 구현을 결합한 1인 팀 환경이다.

이 전제를 바꾸어야 결론을 낼 수 있으면 임의로 가정하지 말고 `policy_decision_required`로 전환한다.

## 시작 절차

Repository-wide search, production code 조사, Git history 조사, test, formatter 또는 probe보다 먼저 현재 phase를
안전하게 시작하는 데 필요한 운영 section을 읽어라.

1. Criteria의 문서 지위·source routing·범위·현재 criterion·기록 규율·중단/완료 조건
2. `PLAN.md`의 Status, source precedence, 현재 phase, baseline과 즉시 기록 규율
3. `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, `FINDINGS.md`의 마지막 상태와 현재 phase 관련 항목

현재 phase에 필요한 Goal 문서나 section이 누락되었으면 추측해 복원하지 말고 blocker를 보고한다. Phase나
criterion이 바뀔 때 관련 criteria/PLAN section을 다시 확인하며, 무관한 section까지 선행 정독하지 않는다.

Status가 `ready`면 실행 시작 시각과 함께 `active`로 변경한다. `active` 또는 다른 중단 Status면 마지막 즉시 기록
이후부터 안전하게 재개한다. 이미 충분한 evidence가 있는 명령과 bounded investigation을 의미 없이 반복하지
마라.

### Candidate source catalog와 progressive routing

다음 목록은 architecture evidence의 candidate source catalog다. 목록에 있다는 이유만으로 전체 본문을 읽거나
동일한 권위로 취급하지 않는다. Production code를 읽기 전에 경로, byte size, content hash, heading 구조, 알려진
freshness와 예상 책임만 `DOCUMENT_SOURCE_MAP.md`에 catalog한다.

1. `docs/game_rulebook.md`
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
13. `/mnt/f/unity projects/ark/docs/unity_core_contract.md`

Source precedence:

- `game_rulebook.md`: 최신 gameplay 정책의 normative source
- current production Rust: 현재 구현 구조·동작 evidence이며 최신 정책 source는 아님
- core/data/skill contract: relevant section별로 current technical contract, stale 또는 미결정인지 판정
- tmp/boss/skill 문서: 해당 change scenario에 실제로 필요한 경우에만 future pressure/stress case로 사용
- Unity contract: stale historical reference이며 current requirement나 compatibility source가 아님

Gameplay 의미 충돌에서는 Rulebook이 우선한다. Technical contract는 Rulebook과 양립하는 세부사항만 보완하며,
current code나 stale 문서가 최신 정책을 덮어쓰지 않는다. Rulebook만으로 확정할 수 없는 충돌은
`policy_undecided`로 격리한다.

각 criterion·domain workflow·change scenario를 시작할 때 필요한 section을 먼저 지정한다. 해당 section과 결론을
이해하는 데 직접 필요한 정의·전제·cross-reference만 읽고, 충돌이나 미해결 의존이 있을 때만 범위를 넓힌다.
Heading이나 검색 snippet만으로 결론을 내리지 않지만 관련성 없는 본문도 읽지 않는다. 읽은 section/range,
선택 목적, 판정한 source status와 생략한 section/문서의 이유를 `DOCUMENT_SOURCE_MAP.md`에 즉시 기록한다.
근거 있는 `irrelevant` 판정과 생략은 gap이나 blocker가 아니다. 문서를 얼마나 읽었는지가 아니라 현재
criterion·scenario 질문에 필요한 evidence coverage가 충분한지로 완료를 판정한다.

## Anchoring 방지

독립 분석과 A00~A11의 1차 판정을 잠그기 전에는 다음을 읽거나 검색 결과에 포함하거나 근거로 사용하지 마라.

- `docs/goals/current_runtime_structural_integrity_audit/**`
- `docs/goals/current_runtime_structural_integrity_audit/DESIGN_REVIEW_DRAFT.md`
- 그 밖의 과거 `docs/goals/**`

Repository search를 실행할 때 위 경로를 명시적으로 제외하라. A00~A11 1차 판정, preservation asset register와
change-scenario matrix가 작성된 뒤에만 별도 cross-check experiment를 사전 기록하고 과거 감사를 읽을 수 있다.

Cross-check는 과거 finding을 import하는 단계가 아니다. 현재 기준선에서 독립적으로 같은 사실을 재증명하거나
반증하고, 누락·차이·anchoring 가능성을 기록한다. A12 rewrite decision은 cross-check 뒤에만 내린다.

## 기본 범위

포함 범위:

- `Cargo.toml`
- `src/lib.rs`
- `src/game/**`
- runtime 구조와 contract evidence에 필요한 `tests/**`와 inline test
- command/query/event/result/snapshot/error/DTO serialization
- data deserialize/validate/compile/publication
- save/checkpoint/restore/reconnect core boundary
- RNG, UUID, time, filesystem diagnostic과 physics boundary
- representative current change hotspot을 이해하는 데 필요한 local Git history
- 위 candidate source 중 criterion별로 선택한 section

`../game_resources/data`는 live-data authoring과 validation/extension pressure를 이해하는 데 반드시 필요할 때만
read-only로 확인한다. 접근 전 `EXPERIMENT_NOTES.md`에 필요성, 정확한 경로와 확인 범위를 기록하고 첫 read의
status/hash를 남긴다.

## 명시적 비범위

- production Rust code, tests, live data와 canonical contract 수정
- architecture remediation 또는 prototype의 production tree 구현
- game_server/Unity code 감사
- current server/Unity compatibility 인증
- 밸런스, 콘텐츠 품질과 재미 평가
- 문서에 없는 gameplay 정책 결정
- 새 Goal 또는 Master Goal 자동 생성
- 전체 rewrite 결론을 evidence 수집 전에 선결
- past Goal 완료 선언을 current evidence로 재사용

## Dirty Worktree와 기준선

기존 변경과 untracked 파일은 사용자 소유다. Reset, restore, checkout, clean, stash, 포맷, 이동, 삭제 또는 관련
없는 파일 수정을 하지 마라.

Candidate source catalog를 만든 직후 production architecture 조사를 시작하기 전에 현재 dirty 기준선을 재현
가능하게 식별하고 `BASELINE.md`와 `EXPERIMENTS.md`에 즉시 기록하라.

최소 기록:

- Goal 실행 시작 시각과 timezone
- `git rev-parse HEAD`
- `git status --short`
- `Cargo.toml`, `src/lib.rs`, `src/game/**`, `tests/**`, candidate source catalog의 file manifest와 aggregate content hash
- 접근한 `../game_resources/data`의 별도 manifest
- `rustc --version`, `cargo --version`
- OS/architecture, Cargo feature와 실행 환경

Commit ID만으로 dirty tree를 식별했다고 표현하지 마라.

Goal 실행 도중 audited production file, test, live data, 판단 근거로 사용한 source 문서, dependency 또는 실행
환경이 바뀌면 즉시 기록하고 `baseline_changed`로 전환하라. 기존 기준선에서 마칠지 새 기준선으로 재시작할지
사용자 결정 없이 서로 다른 evidence를 섞지 마라.

## 최초 Broad Baseline

기준선 식별 직후 runtime architecture 조사를 시작하기 전에 다음을 정확히 한 번 실행하라.

```bash
cargo test --all-targets --all-features
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

실행 위치:

```text
/mnt/f/work/simulator/core_enhanced
```

각 명령의 정확한 command, start/end 시각, exit code, test binary와 runner가 실제 보고한 pass/fail/skip,
compiler/lint 결과를 다음 조사 전에 `EXPERIMENTS.md`에 기록하라.

Test all-pass이고 production/test/data/dependency/environment가 바뀌지 않았다면 같은 full suite와 baseline에 이미
포함된 focused test를 반복하지 마라. Fmt/clippy가 실패해도 architecture 검토를 계속하고 결과를 basic signal로만
사용한다.

추가 test 또는 runtime/performance probe는 정적 evidence와 broad baseline으로 특정 가설, behavior equivalence,
complexity 또는 budget을 증명할 수 없을 때만 허용한다. 실행 전에 다음을 `EXPERIMENT_NOTES.md`에 기록한다.

- 확인할 구체적인 가설
- broad baseline과 static trace가 충분하지 않은 이유
- representative input/workload
- 새 실행이 제공할 중복되지 않는 evidence
- production tree를 변경하지 않는 방법

Probe는 우선 `/tmp`를 사용한다. 실행 직후 정확한 command와 실제 결과를 `EXPERIMENTS.md`에 기록한다.

## 즉시 기록 규율

여러 subsystem을 읽은 뒤 기억으로 한꺼번에 취합하지 마라. 하나의 bounded domain workflow, dependency edge,
change scenario 또는 operation boundary마다 다음 순서를 지킨다.

1. criterion, 대상, 질문과 가설을 정한다.
2. 하나의 bounded code/data/test path만 조사한다.
3. 다음 표면으로 이동하기 전에 raw observation과 실제 결과를 기록한다.
4. domain/aggregate/dependency/source map이 바뀌면 해당 inventory를 즉시 갱신한다.
5. structural pressure 가능성이 보이면 `FINDINGS.md`에 `ARCH-REVIEW-###`와 `Hypothesis`로 즉시 등록한다.
6. 보존 자산이나 migration seam이 보이면 `PRESERVATION_ASSET_REGISTER.md` 또는 관련 map에 즉시 등록한다.
7. 그 후에만 다음 조사로 이동한다.

기록 위치:

- command, test, probe와 실제 결과: `EXPERIMENTS.md`
- 가설, 접근 근거, 문서 precedence, 구조 판단, 대안과 생략 이유: `EXPERIMENT_NOTES.md`
- hypothesis/active/rejected/superseded finding: `FINDINGS.md`
- criterion evidence와 1차/최종 판정: `EVIDENCE_MATRIX.md`
- 미확인 source/scenario와 후속 방법: `GAP_REGISTER.md`
- 나머지 관계: criteria의 필수 산출물 중 해당 map

Finding을 반증하거나 통합해도 기존 ID와 history를 삭제하지 마라. 새 evidence에 맞추어 과거 실험 결과를
다시 쓰지 말고 correction record를 추가한다.

## 조사 수행 방식

파일 순서대로 읽거나 line count부터 결론을 내리지 마라. 다음 세 접근을 병행한다.

1. Rulebook capability에서 production entry, authority, data, result와 test까지 이어지는 vertical workflow
2. Representative change scenario가 type, module, validator, DTO, persistence와 test에 미치는 change trace
3. Runtime operation에서 failure, I/O, resource growth와 recovery까지 이어지는 boundary trace

반드시 다음 design pressure를 실제 code/data/test 경로에 대입한다.

- boss runtime 확장
- movement/collision/occupancy/route 확장
- skill activation, resonance, targeting, delivery와 interrupt 확장
- run relic, monolith, skill fragment와 ego corrosion
- generic dice/tool item과 equipment trigger
- 새로운 roguelike node/session/reward/shop/facility 흐름
- 장기 battle, 다수 unit/projectile/effect와 reconnect presentation

기능을 구현하지 말고 current architecture가 이를 어디에 어떻게 수용하며 무엇을 함께 수정하게 하는지
증명한다.

## 판정 순서

다음 결과 범위를 모두 다룬다.

1. A00 기준선, source precedence와 requirement preservation
2. A01 domain 구조
3. A02 aggregate/state/invariant
4. A03 dependency direction
5. A04 extension scenario fitness
6. A05 change amplification/cognitive load
7. A06 testability
8. A07 operation/failure/I/O boundary
9. A08 data/protocol/persistence evolution
10. A09 performance/resource budget
11. A10 determinism/identity/continuation
12. A11 AI-assisted solo development sustainability
13. A00~A11 1차 판정 잠금
14. 과거 감사/draft cross-check
15. A12 migration/rewrite decision

각 criterion을 `PASS`, `PARTIAL`, `FAIL`, `UNKNOWN`, `N/A` 중 하나로 판정한다. 문제를 발견하지 못했다는 이유로
PASS를 주지 마라. 조사 범위와 positive evidence가 부족하면 PARTIAL 또는 UNKNOWN이다.

Subsystem별로 `RETAIN`, `REFACTOR`, `REWRITE_BOUNDARY`, `REWRITE_SUBSYSTEM`, `FULL_REWRITE_CANDIDATE` 중 하나를
별도 판정한다. Criterion verdict 수를 평균하거나 단순 합산해 rewrite 결론을 만들지 마라.

## Rewrite Decision 금지·허용 규칙

A00~A11 1차 판정, `PRESERVATION_ASSET_REGISTER.md`, `CHANGE_SCENARIO_MATRIX.md`, migration seam/blocker와 과거 감사
cross-check가 완료되기 전에는 full rewrite 권고 또는 비권고를 최종화하지 마라.

다음은 full rewrite의 충분한 근거가 아니다.

- 큰 파일과 긴 함수가 많음
- public field가 많음
- lint가 실패함
- AI가 많은 코드를 생성함
- active finding 또는 FAIL criterion 수가 많음
- 출시 전이고 시간이 충분함

Criteria의 A12 증거 요건에 따라 최소 다섯 대안을 같은 기준으로 비교한 뒤 `REWRITE_DECISION.md`를 작성한다.

## 성능 Budget 규율

사용자가 current 대비 비퇴행을 최소 조건으로 승인했다. 임의의 절대 budget을 먼저 정하지 말고 다음 순서를
지킨다.

1. representative current workload와 measurement environment 정의
2. current median/p95 또는 반복 분포 기록
3. time complexity, allocation/clone, retained bytes와 payload size의 growth model 기록
4. 실제 server tick 또는 contract가 제공하는 deadline 확인
5. current 비퇴행과 headroom을 반영한 provisional budget 제안

전투 tick, command staging, event delta, long-battle retained state, snapshot/save payload를 최소 포함한다. 제품 정책이
필요한 hard limit은 독단적으로 production policy로 확정하지 않고 provisional recommendation으로 구분한다.

## Finding Evidence

모든 Active architecture finding은 criteria의 finding evidence contract를 만족해야 한다. 특히 다음을 빠뜨리지
마라.

- representative change scenario 또는 operation requirement
- current owner/type/module과 production dependency/data flow
- 실제 변경 증폭·제약과 compiler/test protection
- 보존 자산과 반대 evidence
- incremental alternative와 migration seam/blocker
- subsystem/full rewrite decision에 미치는 영향

스타일이나 line count만으로 finding을 만들지 마라. `Hypothesis`는 확정 finding 수에 포함하지 않는다.

## 범위 밖 문제

현재 Goal 밖의 독립 문제를 발견해도 구현하거나 새 Goal을 자동 생성하지 마라. `EXPERIMENT_NOTES.md`의 범위 밖
후속 후보에 다음을 즉시 기록한다.

- 관찰 사실
- 현재 architecture decision과 독립적인 이유
- 예상 영향
- 권장 후속 조사 또는 implementation 순서

현재 evidence 수집에 필요하지 않으면 더 깊게 조사하지 않는다.

## Status와 중단 조건

사용할 Status:

- `ready`
- `active`
- `scope_decision_required`
- `policy_decision_required`
- `baseline_changed`
- `externally_blocked`
- `complete`

다음 경우 해당 Status로 변경하고 선택에 따라 달라지는 조사를 중단한다.

- `scope_decision_required`: core 밖 전체 감사가 완료에 필수이거나 승인 범위로 안전하게 취합 불가
- `policy_decision_required`: preservation requirement나 대표 workload 정책 없이는 A12가 달라지며 UNKNOWN으로
  격리할 수 없음
- `baseline_changed`: audited production/test/data/사용 evidence source/environment가 실행 중 변경됨
- `externally_blocked`: 현재 criterion에 필요한 source, 권한, dependency, runner 또는 재현 가능한 baseline이 없음

Status를 바꾸면 근거, 현재까지 완료한 안전한 조사, 남은 범위와 필요한 사용자 입력을 Goal 문서에 즉시
기록한다. 중단 상태를 complete라고 표현하지 않는다.

## 필수 산출물과 완료

Criteria의 필수 산출물 22개가 현재 evidence와 일치해야 한다. 초기화된 문서는 다음과 같다.

- `PLAN.md`
- `EXPERIMENTS.md`
- `EXPERIMENT_NOTES.md`
- `FINDINGS.md`
- `EVIDENCE_MATRIX.md`
- `GAP_REGISTER.md`

나머지 산출물은 첫 관련 evidence가 생길 때 생성하고 즉시 갱신한다. 마지막 취합 단계에서 이전 조사에 없던
raw observation이나 finding을 기억으로 새로 만들지 마라. 새 사실을 발견하면 해당 bounded investigation으로
돌아가 즉시 기록 절차를 적용한다.

완료 전에 `COMPLETION_AUDIT.md`에서 criteria 완료 조건 각각을 실제 evidence ID와 산출물에 연결한다.

다음을 분리한다.

```text
Goal Status: 승인된 검토와 evidence catalog의 완료 여부
Current Architecture Assessment: current architecture의 적합성
Rewrite Decision: retain/refactor/partial rewrite/full rewrite 권고
```

Finding이 있어도 Goal evidence가 완결되면 Status는 complete일 수 있다. Finding이 없어도 source, scenario,
criterion 또는 rewrite comparison이 미완료면 complete가 아니다.

## 최종 Contamination Check

완료 직전 baseline과 동일한 production/test, 최신 정책인 Rulebook, 실제 evidence로 사용한 source의 manifest,
HEAD와 adjacent resource snapshot을 다시 계산한다. 읽지 않았고 `irrelevant`로 분류된 candidate source의 변경만으로
baseline을 무효화하지 않는다. Goal 문서 이외의 audited file이 바뀌면 baseline 변경 규칙을 적용한다.

## 최종 보고

사용자에게 최소한 다음을 보고한다.

- 최종 Goal Status
- Current Architecture Assessment
- Rewrite Decision과 subsystem별 disposition
- 기준선, candidate source identity와 실제 section routing coverage
- 최초 broad test/fmt/clippy 실제 결과
- A00~A12 판정 요약
- severity/confidence별 finding과 root cause
- preservation asset과 migration seam/blocker
- 대표 change scenario별 핵심 증폭
- testability, operation, performance budget과 determinism gap
- UNKNOWN, 접근하지 못한 source, 읽지 않은 문서/section의 이유와 생략한 검증
- 과거 감사 cross-check에서 확인된 일치·차이
- production/test/live data/canonical contract를 수정하지 않았다는 확인
- 기존 dirty worktree 보존 결과
- 생성·갱신한 산출물
- 사용자 결정이 필요한 미완료 항목

계획만 제시하고 멈추지 마라. 유효한 중단 조건이 발생하지 않는 한 Goal의 read-only 검토와 문서화를 실제로
끝까지 수행하라.
