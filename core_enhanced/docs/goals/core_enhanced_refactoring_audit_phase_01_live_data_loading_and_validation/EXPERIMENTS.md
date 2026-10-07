# Core Enhanced Refactoring Audit Phase 01 — Experiments

Status: complete

## 2026-08-07 — Phase initialization

### 검증하려던 가설

Phase 0이 `Complete / High`이고 사용자의 CRA-13 naming/activation 위임이 기록됐으며 다른 활성 Phase나
경로 충돌 없이 investigation-only Phase 1 문서를 초기화할 수 있다.

### 수행한 실험 또는 검증

- Parent Master 필수 문서 3개 전체 재확인
- Phase 0 boundary proposal과 completion audit 확인
- Phase 1 예정 경로 존재 여부 확인
- audit-owned Goal path의 scoped `git status --short` 확인
- current authoring/phase-start와 refactor-audit 기준 확인

Production code/data/test search, test runner와 formatter는 필수 문서 생성 전에 수행하지 않았다.

### 변경한 내용

- 이 Phase의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 생성했다.
- Parent Master registry에서 Phase 1을 `active`로 전환했다.
- Production code, live data, test와 canonical은 수정하지 않았다.

### 실행한 정확한 명령

```bash
git status --short -- core_enhanced/docs/goals/core_enhanced_refactoring_audit_master core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_00_surface_inventory core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation
if [ -e core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation ]; then echo EXISTS; else echo MISSING; fi
sed -n '1,140p' core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_00_surface_inventory/BOUNDARY_PROPOSAL.md
sed -n '1,220p' core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_00_surface_inventory/COMPLETION_AUDIT.md
```

### 실제 실행된 test 수

Test runner를 실행하지 않았다. 해당 없음.

### Pass / Fail / Skip

Test 기준 해당 없음. Dependency, activation authority와 path collision gate는 통과했다.

### 실제 결과

- Phase 0 verdict는 `Complete / High`다.
- 사용자는 current proposal 실행과 future Goal naming을 작업자에게 위임했다.
- Phase 1 path는 생성 직전 존재하지 않았다.
- Master와 Phase 0은 기존 audit-owned untracked directory이며 다른 활성 Phase는 없다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음. 기존 dirty/untracked 항목을 수정, 이동, 삭제하거나 정리하지 않았다.

### 결과에 따른 판단

Phase 1은 `investigation_only`, production mutation `no`로 안전하게 시작할 수 있다. 실제 loader/data/test
조사는 필수 문서와 dependency를 다시 읽은 뒤 시작한다.

## 아직 수행하지 않은 실험과 검증

- exact Phase 1 source/test/canonical/live-data inventory
- startup→loader→publication call trace
- embedded/live source sorted-set comparison
- parse/validation/default/fallback/error path audit
- focused test inventory 또는 runner
- candidate/disposition과 completion audit

위 항목은 현재 시점에 `discovery pending`이다.

## 2026-08-07 — Live data loading/validation boundary investigation

### 검증하려던 가설

Production startup, embedded source inventory, typed parse/validation과 publication을 실제 current-tree
evidence로 연결하고, count drift/default/fixture/preview validation 신호를 후보 또는 명시적
rejected/deferred/unverified disposition으로 분류할 수 있다.

### 수행한 실험 또는 검증

- `src/game/data/**`, domain-owned buff/map/battlefield types, startup consumer, tests와 canonical inventory
- `GameplayRonSourceSet::embedded()` reference와 `game_resources/data/**` exact sorted-set comparison
- startup → parse → domain/cross validation → private candidate → preview validation → `Arc<GameDataBase>` trace
- top-level strict schema, `serde(default)`, domain validator와 test surface scan
- direct live-file loader, fixture builder, fallback/alias/ignored test와 config consumer search
- current canonical의 source count와 generated-preview stage coverage 비교
- focused integration/unit runner 실행

### 변경한 내용

- `BOUNDARY_AUDIT.md` 생성
- `CANDIDATE_CATALOG.md` 생성
- Phase Goal execution record만 갱신

Production code, live RON, test, canonical, config와 server code는 수정하지 않았다.

### 실행한 정확한 명령 또는 동등한 재현 명령

```bash
comm -3 <(sed -n '133,191p' core_enhanced/src/game/data/live_ron.rs | rg -o '\.\./\.\./\.\./\.\./game_resources/data/[^" ]+' | sed 's#../../../../##' | sort) <(rg --files game_resources/data | sort)
sed -n '133,191p' core_enhanced/src/game/data/live_ron.rs | rg -o '\.\./\.\./\.\./\.\./game_resources/data/[^" ]+' | sort -u | wc -l
rg --files game_resources/data | wc -l
rg -n --hidden -g '!target/**' -g '!.git/**' -g '!core_enhanced/docs/goals/**' -g '!core_enhanced/docs/tmp/**' 'game_balance\.toml|game_balance' .
cargo test -p game_core --test ron_loading -- --nocapture
cargo test -p game_core --test live_skill_catalog_audit -- --nocapture
cargo test -p game_core --lib game::data:: -- --nocapture
cargo test -p game_core --lib game::combat_preview:: -- --nocapture
```

Additional source reads/searches are summarized in `BOUNDARY_AUDIT.md`; secret-bearing values were not read or
recorded.

### 실제 실행된 test 수

- `ron_loading`: 24
- `live_skill_catalog_audit`: 4
- selected `game::data::*` unit tests: 162
- selected `game::combat_preview::*` unit tests: 27
- total executed: 217

### Pass / Fail / Skip

- 217 passed
- 0 failed
- 0 ignored/skipped
- runner-reported filtered out: 700 + 835 on the two selected lib filters

### 실제 결과

- Embedded reference set과 live file set은 각각 23이고 symmetric difference가 없다.
- Production loader는 1 manifest + 22 source를 parse/validate하며 server bind 전에 실패한다.
- Strict parse/local/cross validation과 immutable publication path가 확인됐다.
- Count constant/test/canonical은 병렬 derived authority이고 canonical 21이 실제 22와 drift한다.
- Generated preview publication gate는 canonical 주장과 달리 baseline difficulty만 검사한다.
- Fixture loaders/builders는 production path가 아닌 독립 test lifecycle이며 current schema validation을
  우회하지 않는다.
- `config/game_balance.toml` workspace consumer는 확인하지 못했고 external consumer가 unknown이다.

### 실패 원인

Test failure는 없다. 세 cargo runner를 처음 병렬로 시작해 package/build lock 대기가 발생했지만 각
runner는 lock 획득 뒤 정상 종료했다. Lock 대기는 test skip이나 환경 blocker가 아니다.

### 되돌리거나 수정한 내용

없음. 기존 dirty/untracked production 파일을 수정, restore, format, 이동, 삭제하지 않았다.

### 결과에 따른 판단

- `CRA-P01-C01`: source inventory authority consolidation, candidate
- `CRA-P01-C02`: manifested difficulty 전체 publication validation, priority correctness candidate
- `CRA-P01-C03`: private parsed-to-published mapping consolidation, candidate
- Fixture/production loader 병합, fail-fast error aggregation 변경, default 일괄 제거는 rejected
- `config/game_balance.toml` 삭제는 unverified

별도 Phase completion audit가 coverage, candidate contract와 production hash 보존을 재검토해야 한다.

## 2026-08-07 — Phase completion audit

`COMPLETION_AUDIT.md`가 P1-01~P1-10을 별도 pass로 재검토했다. Exact-set comparison은 계속
23/23/차이 0이고, Core source/test, non-Goal docs, live data, server와 scoped status의 initial/final hash가
모두 동일했다. Candidate 3개는 필수 evidence field를 모두 가지며 S01~S08은 disposition이 있다.

Primary verdict는 `Complete`, long-term fit은 `High`다. Phase 1은 `complete`로 전환하며 Parent Master가
registry를 갱신한 뒤 Phase 2 activation이 허용된다.
