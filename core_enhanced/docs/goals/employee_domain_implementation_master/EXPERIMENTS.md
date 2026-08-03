# Employee Domain Implementation Master Experiments

실제로 수행한 Master 수준 조사와 검증만 시간순으로 기록한다. Production 구현과 Phase별 테스트 결과는 각 Child Goal의 `EXPERIMENTS.md`에 기록한다.

> 경로 정리: 2026-07-18 당시 `docs/tmp/employee_domain_policy_discussion_2026-07-17.md`였던
> 입력 문서는 이후 `docs/decisions/employee_domain_policy_2026-07-17.md`로
> 승격됐다. 아래 기록의 파일 경로는 현재 탐색 가능한 위치로 정규화했으며 당시 실행 의도와
> 결과는 바꾸지 않았다.

## 2026-07-18 — Master Goal 초기화

### 검증하려던 가설

- `employee_domain_implementation_master` 경로가 기존 Goal과 충돌하지 않는다.
- 기존 난이도 감사가 제안한 네 Goal을 독립적인 순차 Phase로 승격할 수 있다.
- Master 문서 생성은 production 파일을 변경하지 않고 수행할 수 있다.

### 수행한 조사 및 검증

- 기존 Goal 파일 목록과 Master Goal 관례를 확인했다.
- 생성 예정 경로의 충돌 여부를 확인했다.
- 신규 Goal 생성 전 Core 저장소의 dirty 상태를 확인했다.
- 완료된 직원 도메인 난이도 감사의 Goal 경계와 순서를 다시 읽었다.

### 변경한 내용

- `docs/goals/employee_domain_implementation_master/PLAN.md` 생성
- `docs/goals/employee_domain_implementation_master/EXPERIMENTS.md` 생성
- `docs/goals/employee_domain_implementation_master/EXPERIMENT_NOTES.md` 생성

Production runtime, live RON, DTO, server, Unity, fixture, test 또는 canonical 문서는 변경하지 않았다.

### 실행한 정확한 명령

```bash
pwd && rg --files -g 'AGENTS.md' -g 'docs/goals/**' -g 'docs/decisions/employee_domain_policy_2026-07-17.md' | sort
```

```bash
sed -n '1,260p' docs/goals/core_unimplemented_policy_implementation_master/PLAN.md && sed -n '1,220p' docs/goals/core_unimplemented_policy_implementation_master/EXPERIMENT_NOTES.md
```

```bash
sed -n '1,280p' docs/goals/cross_layer_contract_hardening_master/PLAN.md && sed -n '1,220p' docs/goals/cross_layer_contract_hardening_master/STATUS.md
```

```bash
sed -n '450,720p' docs/decisions/employee_domain_policy_2026-07-17.md && sed -n '1,280p' docs/goals/employee_domain_implementation_difficulty_audit/PLAN.md
```

```bash
if [ -e docs/goals/employee_domain_implementation_master ]; then printf 'collision\n'; else printf 'available\n'; fi
git status --short
```

```bash
mkdir -p docs/goals/employee_domain_implementation_master
```

### 실제 실행된 test 수

해당 없음. 문서 전용 Master 초기화이며 production 동작을 변경하지 않았다.

### pass/fail/skip 수

- pass: test runner를 실행하지 않아 수치 없음
- fail: test runner를 실행하지 않아 수치 없음
- skip: test runner를 실행하지 않아 수치 없음

### 실제 결과

- 생성 예정 경로는 `available`이었다.
- 저장소는 생성 전부터 광범위한 dirty 상태였다.
- 기존 Master 관례는 orchestration 문서와 Child Goal 실행 기록을 분리하고, Child 디렉터리를 실제 착수 시 생성한다.
- 난이도 감사의 A → B → C → D 순서와 source-of-truth 경계가 Phase 1 → 2 → 3 → 4에 그대로 반영됐다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

Master는 네 Phase의 범위와 게이트만 소유하고 production 구현은 각 Child Goal이 독립적으로 수행하도록 했다. 시작하지 않은 Child Goal 디렉터리는 생성하지 않았다.

## 2026-07-18 — Master 문서 구조 검증

### 검증하려던 가설

- 필수 문서 3개가 모두 존재한다.
- 네 Child Goal이 Phase 1~4에 정확히 한 번씩 배정되고 고정 순서와 선행 조건이 기록됐다.
- 시작하지 않은 Child Goal 디렉터리는 생성되지 않았다.
- 생성한 Markdown에 trailing whitespace가 없다.

### 수행한 실험 또는 검증

```bash
test -f docs/goals/employee_domain_implementation_master/PLAN.md
test -f docs/goals/employee_domain_implementation_master/EXPERIMENTS.md
test -f docs/goals/employee_domain_implementation_master/EXPERIMENT_NOTES.md
rg -n '^Status: active$|^Implementation complete: false$|^### Phase [1-4]|employee_definition_profile_contract|employee_recruitment_queue_ownership|employee_condition_lifecycle_migration|employee_psychology_state_contract|Phase 1: EmployeeDefinition/Profile|Master final integration review' docs/goals/employee_domain_implementation_master/PLAN.md
for d in employee_definition_profile_contract employee_recruitment_queue_ownership employee_condition_lifecycle_migration employee_psychology_state_contract; do test ! -e "docs/goals/$d" || { printf 'unexpected child dir: %s\n' "$d"; exit 1; }; done
if rg -n '[[:blank:]]+$' docs/goals/employee_domain_implementation_master; then exit 1; fi
wc -l docs/goals/employee_domain_implementation_master/*.md
git status --short -- docs/goals/employee_domain_implementation_master docs/goals/employee_domain_implementation_difficulty_audit docs/decisions/employee_domain_policy_2026-07-17.md
```

### 변경한 내용

검증 자체로 변경한 production 내용은 없다. 이 검증 기록만 추가했다.

### 실제 실행된 test 수

해당 없음. 파일·문서 구조 검증이며 test runner를 실행하지 않았다.

### pass/fail/skip 수

- pass: test runner가 수치를 보고하지 않음
- fail: test runner가 수치를 보고하지 않음
- skip: test runner가 수치를 보고하지 않음

### 실제 결과

- 필수 문서 3개가 모두 존재했다.
- Phase 1~4 heading, 네 Child 이름, 고정 순서와 Master final review가 확인됐다.
- 네 Child Goal 경로는 모두 아직 존재하지 않았다.
- trailing whitespace 검색 결과가 없었다.
- Master 문서 3개는 총 615줄이었다.
- scoped status에는 기존 untracked 난이도 감사/정책 문서와 새 Master 디렉터리만 표시됐다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

Master Goal 문서화 완료 조건을 충족했다. Master 자체는 네 Phase가 아직 시작되지 않았으므로 `active`를 유지한다.

## 2026-07-18 — 단일 Goal 지침 준수 감사 및 문서 보강

### 검증하려던 가설

- Master 문서가 사용자가 다시 제시한 단일 Goal 지침의 필수 문서 내용과 절차를 충족한다.
- 수행 이력에 지침과 어긋난 순서가 있다면 실제 기록으로 식별할 수 있다.

### 수행한 실험 또는 검증

- 기존 Goal 재개 규칙에 따라 Master 문서 3개를 처음부터 끝까지 다시 읽었다.
- PLAN 필수 항목과 EXPERIMENT_NOTES 필수 항목을 제목 및 내용 기준으로 대조했다.
- 문서 생성 전에 실행된 명령 순서를 EXPERIMENTS 기록과 지침 2.1에 대조했다.

### 변경한 내용

- PLAN에 `구현 범위`, `필수 Focused 검증`, `필수 Broad 검증`, `생략할 검증과 근거`를 독립 절로 보강했다.
- EXPERIMENT_NOTES에 `지침 준수 감사`를 추가했다.
- exact Master goal name을 생성 전에 사용자에게 확인하지 않은 점과, 필수 문서 초기화 전에 허용 범위를 넘은 read-only 조사를 수행한 절차 위반을 기록했다.

### 실행한 정확한 명령

```bash
sed -n '1,520p' docs/goals/employee_domain_implementation_master/PLAN.md
sed -n '1,260p' docs/goals/employee_domain_implementation_master/EXPERIMENTS.md
sed -n '1,260p' docs/goals/employee_domain_implementation_master/EXPERIMENT_NOTES.md
```

```bash
rg -n '^Status:|^## (목표|사용자-visible 문제|감사 Finding ID|대상 저장소와 경로|현재 Runtime 및 Live Data 근거|Source of Truth와 소유 계층|공통 불변식|승인된 정책|미결정 정책|Master 범위|구현 범위|명시적 비범위|DTO Migration|Data 또는 Live RON Migration|Save 및 Checkpoint Migration|필수 Focused 검증|필수 Broad 검증|생략할 검증과 근거|중단 조건|Compatibility|Master 사용자-visible 완료 조건)$' docs/goals/employee_domain_implementation_master/PLAN.md
rg -n '^## (구조적 판단|지침 준수 감사|Source-of-truth 충돌|정책 질문|외부 시스템과 직접 의존성|Dirty Worktree 주의사항|접근하거나 검증하지 못한 소비자|적용하지 않은 대안과 이유|생략한 검증과 근거|남은 위험|범위 밖 후속 후보)$' docs/goals/employee_domain_implementation_master/EXPERIMENT_NOTES.md
if rg -n '[[:blank:]]+$' docs/goals/employee_domain_implementation_master; then exit 1; fi
git diff --check -- docs/goals/employee_domain_implementation_master
git status --short -- docs/goals/employee_domain_implementation_master
```

### 실제 실행된 test 수

해당 없음. 지침 및 Markdown 문서 구조 감사이며 test runner를 실행하지 않았다.

### pass/fail/skip 수

- pass: test runner가 수치를 보고하지 않음
- fail: test runner가 수치를 보고하지 않음
- skip: test runner가 수치를 보고하지 않음

### 실제 결과

- PLAN의 지침상 필수 내용이 모두 독립 절로 확인됐다.
- EXPERIMENT_NOTES의 필수 내용과 지침 준수 감사가 모두 확인됐다.
- trailing whitespace 검색 결과가 없었다.
- Master 디렉터리는 여전히 하나의 untracked 경로로 표시됐다.
- Master가 여러 source-of-truth를 직접 구현하지 않고 네 Child를 조정하는 구조는 사용자 요청에 따른 명시적 orchestration이므로 범위 규칙과 양립한다.
- 문서 생성 전 read-only 조사 순서는 지침 2.1과 양립하지 않았다. 이 사실은 문서에 기록했으며 소급해 없애지 않았다.

### 실패 원인

- 초기 절차 준수는 실패했다. 필수 문서 3개 생성 전에 기존 Master 예시와 난이도 감사 문서를 읽었기 때문이다.
- exact Master goal name도 생성 전에 사용자에게 별도로 확인하지 않았다.

### 되돌리거나 수정한 내용

- 이미 수행한 read-only 명령은 되돌릴 대상이 아니며 실행 순서를 소급 수정하지 않았다.
- 형식상 분산돼 있던 PLAN 필수 절은 독립 제목으로 보강했다.

### 결과에 따른 판단

현재 문서 내용은 지침 체크리스트에 맞게 보강됐지만, 최초 생성 절차까지 완전 준수했다고 표현하지 않는다. 이후 Phase는 각 Child의 세 문서를 먼저 초기화한 뒤 조사하는 순서를 강제한다.

## 2026-07-18 — 지침 감사 보강 후 최종 재검증

### 검증하려던 가설

- 보강한 필수 절과 절차 위반 기록이 실제 문서에 존재한다.
- Child Goal 디렉터리를 미리 만들지 않았고 문서 whitespace 오류가 없다.

### 수행한 실험 또는 검증

```bash
test -f docs/goals/employee_domain_implementation_master/PLAN.md
test -f docs/goals/employee_domain_implementation_master/EXPERIMENTS.md
test -f docs/goals/employee_domain_implementation_master/EXPERIMENT_NOTES.md
rg -n '^## (구현 범위|필수 Focused 검증|필수 Broad 검증|생략할 검증과 근거)$|^## 지침 준수 감사$|초기 절차 준수는 실패했다|이후 Phase는 각 Child의 세 문서를 먼저 초기화' docs/goals/employee_domain_implementation_master/*.md
if rg -n '[[:blank:]]+$' docs/goals/employee_domain_implementation_master; then exit 1; fi
for d in employee_definition_profile_contract employee_recruitment_queue_ownership employee_condition_lifecycle_migration employee_psychology_state_contract; do test ! -e "docs/goals/$d" || { printf 'unexpected child dir: %s\n' "$d"; exit 1; }; done
wc -l docs/goals/employee_domain_implementation_master/*.md
git status --short -- docs/goals/employee_domain_implementation_master
```

### 변경한 내용

검증 자체로 production 변경은 없다. 이 실행 기록만 추가했다.

### 실제 실행된 test 수

해당 없음. 문서 구조 검증이며 test runner를 실행하지 않았다.

### pass/fail/skip 수

- pass: test runner가 수치를 보고하지 않음
- fail: test runner가 수치를 보고하지 않음
- skip: test runner가 수치를 보고하지 않음

### 실제 결과

- 필수 문서 3개와 보강한 네 PLAN 절, 지침 준수 감사 및 절차 위반 기록을 확인했다.
- trailing whitespace가 없었다.
- 네 Child Goal 디렉터리는 모두 아직 생성되지 않았다.
- 검증 시점의 Master 문서는 총 769줄이었고 디렉터리는 untracked 상태였다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

현재 Master 문서에는 지침 준수 여부와 최초 절차 위반이 모두 추적 가능하게 남아 있다.

## 2026-07-18 — Phase 1 completion review와 재개 기록 복구

- Phase 1은 이전 실행에서 이미 `Status: complete`였고 production implementation과 최종 증거가 Child 문서에 기록돼 있었다.
- 재개 초기에 경로 확인 실패를 성공으로 오판해 기존 Child 문서를 덮어썼으나, 이전 Codex session의 9개 patch를 순서대로 재생해 정확한 최종 문서 3개를 복구했다.
- current 상태에서 Core 759, game_server 30, Unity EditMode 240, Unity PlayMode 57 pass/4 explicit skip과 Rust fixture exact match를 재검증했다.
- `employee_definition_profile_contract/COMPLETION_REVIEW.md`를 작성하고 Master Phase 1을 `complete`로 갱신했다.
- production 구현은 이 Master 재개 단계에서 변경하지 않았다.

## 2026-07-18 — Phase 2 RecruitmentQueue ownership 완료

- Child `employee_recruitment_queue_ownership`이 run-seeded finite queue, active session과 roster exactly-one ownership을 구현했다.
- 시작 미선택 starter와 non-starter definitions가 unified queue에 포함된다.
- HQ 채용/응급지원/상점 transition, invalid command rollback, checkpoint load validation과 Floor carry-forward를 focused tests로 증명했다.
- node-seeded recruitment reroll 함수와 HQ active content의 candidate ownership 사본을 제거했다.
- public candidate/result/command wire shape는 Phase 1 `employee_definition_id` 계약을 유지했고 server/Unity production 변경은 필요하지 않았다.
- Core 766, game_server 30, Unity EditMode 240, Unity PlayMode 57 pass/4 explicit skip이 통과했다.
- `cargo clippy --all-targets -- -D warnings`는 Phase 2 밖 기존 lint debt 117개 때문에 실패했으며 Child 기록에 남겼다.
- `employee_recruitment_queue_ownership/COMPLETION_REVIEW.md`를 작성하고 Master Phase 2를 `complete`, Phase 3을 `active`로 갱신했다.

## 2026-07-18 — Phase 3 Employee Condition lifecycle 완료

- Run HP, generic injury, mutable life/availability authority를 제거하고 physical damage 0~6 + trauma 0~100 condition으로 complete cutover했다.
- attempt-local incapacitation/redeploy, staged result commit, retreat rollback, death-prevent/asset recovery, Treatment/trauma recovery와 checkpoint를 한 lifecycle로 구현했다.
- Core DTO/server/Unity를 condition과 purpose eligibility 계약으로 전환하고 구형 RON/admin/Unity payload를 strict reject했다.
- Core 775, game_server 30, Unity EditMode 240, ordinary PlayMode 57과 Rust fixture exact 검증이 통과했다.
- 전체 PlayMode의 외부-server live-only 4개는 server 미기동 connection failure로 분리 기록했다.
- Phase 3 completion review를 작성하고 Master Phase 3을 `complete`, Phase 4를 `active`로 갱신했다.

## 2026-07-18 — Phase 4 Employee Psychology state 완료

- authored Profile personality와 run-local Psychology를 분리하고 기존 `employee_trust`/policy surface를 제거했다.
- trust score/band, semantic memory merge, typed subject/source dedupe, expiry와 recent event receipt를 하나의 aggregate로 구현했다.
- 현재 실제 Rest support event만 production mutation으로 유지하고 dormant high-risk/refusal/cue/combat/trauma modifier를 public/runtime 계약에서 제거했다.
- checkpoint, retreat와 사망 roster record persistence, exact Core DTO/server/Unity strict reader 및 legacy rejection을 검증했다.
- Core 776, game_server 30, Unity EditMode 241, ordinary PlayMode 57과 Rust fixture exact 검증이 통과했다.
- Phase 4 completion review를 작성하고 Master Phase 4를 `complete`, final integration review를 `active`로 갱신했다.

## 2026-07-18 — Master 최종 통합 검증과 완료

Phase 4 완료 뒤 Child 검증을 대체 증거로 재사용하지 않고 최종 합성 상태에서 broad validation을 별도로 다시 실행했다.

- Core `cargo test --all-targets`: lib 732 + integration 44 = 776 pass.
- game_server `cargo test`: 30 pass.
- Unity full EditMode job `88af486eb436415c89b05468bf70a224`: 241/241 pass.
- Unity ordinary PlayMode job `7ec21a13886546128dadef667cffcc0a`: 57/57 pass.
- typed Rust employee profile fixture exact comparison: pass.
- Core/server/Unity 관련 diff check와 Goal trailing whitespace check: pass.
- production Core/server/live RON/Unity fixture·reader/canonical 검색에서 employee trust policy/reaction/cue/modifier, Run HP/availability와 구형 `trust` projection이 남지 않았다. 남은 transient `current_hp`는 battle runtime HP이고 server legacy 문자열은 strict rejection test다.
- `cargo clippy --all-targets -- -D warnings`는 기존 범위 밖 lint debt로 실패했다. Phase 4에서 새로 보고된 derivable Default는 수정했으며 최종 broad tests는 모두 통과했다.

네 Child와 Master completion review를 연결해 source-of-truth, rollback/checkpoint/death/reconnect, exact producer-consumer projection과 no-compatibility 완료 조건을 충족한다고 판단했다.
