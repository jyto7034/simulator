# Run Persistence, Facility, And Monolith Policy Correction Master Experiments

Status: complete

Implementation complete: true

Created: 2026-07-28

이 문서는 Master 초기화, phase gate와 실제로 수행한 검증 결과만 시간순으로 기록한다. Child의 상세
실험은 각 child `EXPERIMENTS.md`가 소유한다.

## 2026-07-28 — Master Goal 초기화

### 검증하려던 가설

- 사용자가 승인한 네 수정 경계를 하나의 implementation patch가 아니라 기존 master 관례의 순차 phase로
  관리할 수 있다.
- 새 Master 이름과 경로가 기존 Goal과 충돌하지 않는다.
- 문서 초기화 전 관련 두 저장소의 dirty baseline을 확인할 수 있다.

### 수행한 확인

- 사용자 clarification에 따라 처음 제안했던 네 개별 Goal 대신 하나의 Master + phase 구조를 선택했다.
- 기존 `run_persistence_facility_monolith_policy_master`의 `PLAN.md`, `EXPERIMENTS.md`,
  `EXPERIMENT_NOTES.md`를 읽어 registry/dependency/child activation 관례를 확인했다.
- 새 경로 `run_persistence_facility_monolith_policy_correction_master`의 충돌 여부를 확인했다.
- Core/server/data workspace와 외부 Unity 저장소의 `git status --short`를 실행했다.

### 실행한 정확한 명령

```bash
sed -n '1,320p' core_enhanced/docs/goals/run_persistence_facility_monolith_policy_master/PLAN.md
sed -n '1,220p' core_enhanced/docs/goals/run_persistence_facility_monolith_policy_master/EXPERIMENTS.md
sed -n '1,220p' core_enhanced/docs/goals/run_persistence_facility_monolith_policy_master/EXPERIMENT_NOTES.md
if [ -e core_enhanced/docs/goals/run_persistence_facility_monolith_policy_correction_master ]; then echo EXISTS; else echo AVAILABLE; fi
git status --short
git -C '/mnt/f/unity projects/ark' status --short
```

### 실제 test 수와 결과

- Test runner를 실행하지 않았다.
- 실제 test 수: runner가 제공한 수치 없음.
- Pass/fail/skip: 해당 없음.
- 새 Goal 경로는 `AVAILABLE`이었다.
- 두 `git status --short`는 출력이 없었다.
- 기존 Master 문서 세 개는 읽을 수 있었고 phase registry, dependency graph, exit gate와 child 활성화
  관례를 확인했다.

### 변경한 내용

- Master `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 생성했다.
- Production code, live data, schema, server, Unity, fixture, test와 canonical은 수정하지 않았다.
- 사용자 clarification 전에 만든 네 개의 빈 후보 디렉터리는 파일 생성 없이 `rmdir`로 제거했다.

### 실패 원인과 되돌림

- 명령 실패 없음.
- Production 변경이나 되돌림 없음.

### 판단

- Master 초기화 입력과 경로가 확보됐다.
- P00~P04 dependency와 exit gate를 문서화했다.
- Goal-specific production baseline, 코드 조사, 구현과 테스트는 아직 시작하지 않았다.
- 다음 production action은 P00 child Goal 활성화와 그 child 지침에 따른 baseline 확보다.

## 2026-07-28 — Master 문서 완전성 검사

### 검증하려던 가설

- 필수 문서 세 개가 모두 존재하고 Status/Implementation complete 및 PLAN 필수 구조를 읽을 수 있다.
- 문서에 trailing whitespace나 diff whitespace 오류가 없다.
- 사용자 clarification 전에 만든 네 빈 후보 Goal 경로가 남아 있지 않다.

### 실행한 정확한 명령

```bash
test -f core_enhanced/docs/goals/run_persistence_facility_monolith_policy_correction_master/PLAN.md
test -f core_enhanced/docs/goals/run_persistence_facility_monolith_policy_correction_master/EXPERIMENTS.md
test -f core_enhanced/docs/goals/run_persistence_facility_monolith_policy_correction_master/EXPERIMENT_NOTES.md
rg -n "^Status:|^Implementation complete:|^Goal name:|^## " core_enhanced/docs/goals/run_persistence_facility_monolith_policy_correction_master/*.md
rg -n "[ \\t]+$" core_enhanced/docs/goals/run_persistence_facility_monolith_policy_correction_master
git diff --check -- core_enhanced/docs/goals/run_persistence_facility_monolith_policy_correction_master
git status --short -- core_enhanced/docs/goals/run_persistence_facility_monolith_policy_correction_master
```

네 과도기 후보 경로에는 각각 `test ! -e` 존재 검사를 수행했다.

### 실제 test 수와 결과

- Test runner를 실행하지 않았다.
- 실제 test 수: runner가 제공한 수치 없음.
- Pass/fail/skip: 해당 없음.
- 필수 문서 세 개의 `test -f`가 모두 성공했다.
- Header/section 검색으로 PLAN 23개 numbered section, Notes 11개 numbered section과 Experiments 기록을
  확인했다.
- trailing whitespace와 `git diff --check` 오류가 없었다.
- scoped status는 새 Master 디렉터리만 `??`로 보고했다.
- 네 과도기 후보 경로는 모두 존재하지 않았다.

### 변경/되돌림

- 검증 중 production 변경이나 되돌림 없음.

### 판단

- Master 문서 초기화와 scope/dependency 기록이 완료됐다.
- Production 구현과 child Goal 생성은 아직 시작하지 않았다.

## Phase Gate Log

| Phase | Child status | Completion review | Broad validation | Next action |
| --- | --- | --- | --- | --- |
| P00 | complete | Complete / High | Core/server/workspace 통과 | P01 child 활성화 가능 |
| P01 | complete | Complete / High | Core/server/workspace 통과 | P02 child 활성화 가능 |
| P02 | complete | Complete / High | Core/server/workspace 통과 | P03 child 활성화 가능 |
| P03 | complete | Complete / High | Rust workspace, Unity Edit/Play 통과 | P04 독립 재감사 가능 |
| P04 | complete | Complete / High | current Rust/Unity broad + finding audit | Master complete |

## 2026-07-28 — P00 Child 활성화

### Dependency와 문서 gate

- Correction Master 필수 문서 3개, 공통 governance 문서, 기존 implementation Master와 P00~P05/P07
  기록, 재감사 메모를 끝까지 읽었다.
- P00의 direct predecessor는 없고 승인되지 않은 policy question도 없다.
- 신규 child 경로 충돌이 없음을 확인한 뒤
  `run_policy_correction_phase_00_compendium_transaction_rollback_integrity`의 `PLAN.md`,
  `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 생성하고 끝까지 읽었다.

### Dirty baseline

- `/mnt/f/work/simulator`와 `/mnt/f/unity projects/ark` 모두 광범위한 기존 tracked/untracked 변경이 있다.
- 기존 변경을 사용자 변경으로 보존하고 P00 대상 파일의 scoped diff를 production 수정 전에 읽는다.

### Production 변경과 검증

- Child/Master Goal 문서 이외의 production code/data/server/Unity/canonical 변경 없음.
- Test runner 미실행; 실제 pass/fail/skip 수 없음.

## 2026-07-28 — P00 완료와 P01 활성화

- P00 completion review: `Complete / High`.
- P00 final Rust evidence: Core 788, integration 47, server 53, failure 0.
- P01 신규 경로 충돌과 관련 scoped dirty 상태를 확인한 뒤 child 필수 문서 3개를 생성·완독했다.
- P01 direct predecessor와 Master registry가 일치하며 production baseline 조사 가능 상태다.

## 2026-07-28 — P01 완료와 P02 활성화

- P01 completion review: `Complete / High`.
- Certified roster membership은 shared combat eligibility로 수렴했고 live power model은 version 2다.
- P01 final Rust evidence: Core 789, integration 47, server 53, failure 0.
- Old model active-run rejection과 zero-current-power qualitative threat를 focused regression으로 고정했다.
- Unity DTO shape는 바뀌지 않아 P01 Unity 변경/실행은 적용 대상이 아니었다.
- Master serial gate에 따라 P02를 active로 전환한다.

## 2026-07-28 — P02 완료와 P03 활성화

- P02 completion review: `Complete / High`.
- Checkpoint가 model version/current Floor anchor를 map/node와 함께 소유하고 host high-water는 loads/run
  peak만 남는다.
- Core payload V2와 active-run envelope V3가 구 schema/field를 migration 없이 거부한다.
- P02 final Rust evidence: Core 790, integration 47, server 54, failure 0.
- Unity source 영향 검색에서 내부 save field consumer가 없어 wire/Unity 변경은 적용 대상이 아니었다.
- Master serial gate에 따라 P03를 active로 전환한다.

## 2026-07-30 — P03 완료와 P04 활성화

- P03 completion review: `Complete / High`.
- Typed stage/ledger authority, Stage2·3 exact stat, unchanged wave, one-pass reward, checkpoint 왕복과
  qualitative Unity projection을 current source와 회귀로 확인했다.
- P03 final focused evidence: Core monolith 17/17, live RON 23/23, Unity EditMode 3/3.
- P03 broad evidence: workspace all-targets 전부 통과(Core lib 798/798), Unity EditMode 260/260,
  PlayMode 62 passed/4 explicit live-server skips.
- Master serial gate에 따라 P04 child를 생성하고 독립 finding/legacy/record 재감사를 활성화한다.

## 2026-07-30 — P04 독립 재감사와 Master 완료

- F-01/F-02/D-01/P-01/P-02/P-03을 current runtime/data/save/server/Unity/canonical에 독립 매핑했다.
- D-01 P00 experiments/notes header를 current 완료 사실에 맞추고 historical body는 보존했다.
- Current Rust workspace all-targets는 nonzero suites 합계 899 passed, 0 failed다.
- Unity는 focused 3/3, EditMode 260/260, broad PlayMode 62 passed/4 explicit skips다.
- Explicit live 4건을 실제 server에서 추가 실행해 correction-relevant run-start/combat/admin-event 3건은
  통과했다. 독립 Maintenance fixture 1건 실패는 P04 review의 out-of-scope risk로 기록했다.
- Legacy/status/diff/canonical audit에 material correction gap이 없다.
- P04와 correction Master final review: `Complete / High`.
