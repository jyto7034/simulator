# Employee Condition Lifecycle Migration Experiments

실제로 수행한 조사, 구현과 검증을 시간순으로 기록한다.

## 2026-07-18 — Child Goal 초기화

### 수행한 확인

- Phase 3 경로 확인 결과는 `phase3_missing`이었다.
- Master에서 Phase 1과 Phase 2가 complete임을 확인했다.
- Phase 2 completion review와 다음 Phase 안정 계약을 확인했다.

### 변경한 내용

- Phase 3 필수 문서 3개를 production 조사 전에 생성했다.
- production 파일은 변경하지 않았다.

### 실제 실행된 test 수

해당 없음. Child 문서 초기화다.

### 다음 판단

승인 condition 정책과 current runtime/DTO/live RON/Unity consumer를 read-only로 조사한다.

## 2026-07-18 — Complete cutover 구현

### 수행한 변경

- `Employee`의 Run HP/health, injury list, mutable life state와 availability를 제거하고 `EmployeeCondition { physical_damage, trauma }`와 중앙 purpose evaluator로 교체했다.
- physical 0~6, trauma 0~100, Critical/Dead/Breakdown 구간과 reason priority를 고정했다.
- 전투불능 instance ledger는 working roster에 physical +1과 policy trauma를 한 번 적용하고, 새 배치는 갱신 상태의 final max HP에 exact trauma 공식을 적용한다.
- staged combat result에 일반 사망 장비 reverse reference/shared fragment/active modifier 회수를 포함했다.
- `ReducePhysicalDamage { amount }` Treatment를 추가하고 Rest/SavePoint/Gate는 trauma-only로 유지했다.
- admin command를 `admin_set_employee_condition` 하나로 전환하고 구형 두 command와 Run HP consumable variant를 strict reject했다.
- roster DTO를 `condition`, purpose별 `eligibility`, `combat_eligible_employee_ids`로 전환했다.
- Unity `RosterModel`과 roster/loadout/item/support/maintenance/battle presentation이 새 계약을 필수로 읽으며 gameplay affordance는 Core eligibility를 직접 사용한다.
- live run policy에서 `incapacitation_run_hp_loss_percent`를 제거하고 canonical rulebook/runtime contract를 갱신했다.

### 검증 결과

- Core `cargo test --all-targets -q`: unit 731 + integration 44 = 775 pass.
- game_server `cargo test --all-targets -q`: 30 pass.
- Unity EditMode job `332889c81c374b8cae2d0a7c950fc039`: 240/240 pass.
- Unity ordinary PlayMode job `a398e95e9a8f4213986860005d933fab`: 57/57 pass.
- 전체 PlayMode job `21810ab8c62445109ec08cc7344b824a`: 61개 중 ordinary 57개 pass, 외부 서버 미기동 live-only 4개 connection failure.
- employee profile/action capability Rust fixture exact validation: pass.
- `git diff --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: 기존 범위 밖 lint debt 119개로 fail. 이번 변경에서 발견한 condition 관련 collapsible-if, redundant local, literal grouping은 수정했다.

### 결과 판단

Phase 3 완료 조건을 충족한다. 실제 Treatment 수치는 production authoring하지 않았고 schema/runtime/test만 제공하므로 승인되지 않은 밸런스 값을 만들지 않았다.
