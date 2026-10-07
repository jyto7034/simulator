# Core Enhanced Refactoring Audit Phase 04 — Experiments

Status: complete

## 2026-08-07 — Phase initialization and pre-investigation gate

### 검증하려던 가설

Phase 3 `Complete / High`, Parent CRA-13 명명 위임과 Phase 0의 승인된 순차 경계가 Phase 4 child를
활성화할 수 있고, 경로 충돌이나 다른 active child 없이 investigation-only 문서만 안전하게 초기화할
수 있는지 확인했다.

### 수행한 실험 또는 검증

- 사용자 제공 재개 지침 전체 확인
- Master 필수 문서 3개 전체 확인과 Status/dependency 확인
- Phase 0 boundary proposal 및 Phase 4 inventory seam 확인
- Phase 3 PLAN, completion audit와 handoff decision 확인
- Goal 작성·실행 가이드, 완료 감사 가이드와 frozen refactor standard 확인
- Phase 4 경로 충돌, 다른 active child와 current dirty worktree 확인

Repository-wide production code search, runtime/data 조사, test runner, probe와 formatter는 문서 초기화 전
수행하지 않았다.

### 변경한 내용

- `core_enhanced_refactoring_audit_phase_04_interactive_node_and_visual_novel` 필수 Goal 문서 3개 생성
- Parent Master Phase 4 registry를 `active`로 전환

Production code, live data, test, canonical, server와 Unity는 수정하지 않았다.

### 실행한 정확한 명령

```bash
sed -n '1,260p' <attachment>
sed -n '<chunks>' core_enhanced/docs/goals/core_enhanced_refactoring_audit_master/{PLAN.md,EXPERIMENTS.md,EXPERIMENT_NOTES.md}
sed -n '<chunks>' core_enhanced/docs/goal_authoring_and_execution_guide.md
sed -n '<chunks>' core_enhanced/docs/goal_completion_review_guide.md
sed -n '1,250p' core_enhanced/docs/engineering_workflow.md
sed -n '1,190p' core_enhanced/docs/README.md
sed -n '1,180p' core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_00_surface_inventory/BOUNDARY_PROPOSAL.md
sed -n '1,260p' core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy/PLAN.md
sed -n '1,220p' core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy/COMPLETION_AUDIT.md
sed -n '1,180p' core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy/EXPERIMENT_NOTES.md
git rev-parse HEAD
git status --short
git status --short -- <Master/Phase0~4 Goal paths>
rg -n -C 3 '<Phase 4 inventory terms>' core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_00_surface_inventory/SURFACE_INVENTORY.md
mkdir -p core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_04_interactive_node_and_visual_novel
```

### 실제 실행된 test 수

Test runner를 실행하지 않았다. 해당 없음.

### Pass / Fail / Skip

Test 기준 해당 없음. Required-document, dependency, active-child, path-collision과 dirty gate 확인은
성공했다.

### 실제 결과

- Master Status는 `active`, production mutation authority는 `no`다.
- Phase 0~3은 모두 `Complete / High`이고 Phase 4 direct dependency가 충족됐다.
- 다른 active child는 없고 Phase 4 경로는 생성 전 존재하지 않았다.
- Start commit은 `f318414fb0156a67e04c5479000c2a2683fc0c09`다.
- Current worktree에는 Core code/test/canonical, live data와 server의 사용자 소유 dirty 변경 및 기존
  audit Goal 산출물이 있다.
- Phase 4는 Event/Facility/node session과 VN trigger/session lifecycle만 감사한다.

### 실패 원인

없음. 사용자 요청의 축약 경로는 실제 `core_enhanced/docs/goals/...` 위치로 해소했다.

### 되돌리거나 수정한 내용

없음. 기존 dirty/untracked 파일을 복원, 이동, 삭제, format 또는 정리하지 않았다.

### 결과에 따른 판단

Phase 4 read-only production 조사와 audit artifact 작성이 허용된다. Phase 5는 Phase 4 자체
`Complete / High` 전에는 생성하거나 활성화하지 않는다.

## 아직 수행하지 않은 실험과 검증

- Phase 4 scoped production-content/status hash baseline
- exact source/data/test/canonical/consumer inventory
- Event/Facility/VN authority and lifecycle trace
- test-family meaning, production reachability, assertion and duplication audit
- candidate/signal catalog and completion audit
- test runner or probe

위 항목은 문서 초기화 evidence가 아니며 이후 실제 실행 결과만 기록한다.

## 2026-08-07 — Scoped production-content baseline

### 검증하려던 가설

Phase 4가 읽을 primary runtime, shared carrier/consumer, published data, focused test와 canonical의 현재
dirty-tree 내용을 category별 aggregate hash와 scoped status로 고정해 종료 시 production no-mutation을
입증할 수 있는지 확인했다.

### 수행한 실험 또는 검증

- Phase 0 inventory와 초기 symbol/path inventory에서 Phase 4 primary/shared/data/test/canonical 경로 확정
- 각 category의 ordered `sha256sum` aggregate 계산
- 같은 경로 집합의 `git status --short` aggregate와 실제 status 행 보존

### 변경한 내용

이 audit 기록만 갱신했다. Production 파일은 변경하지 않았다.

### 실행한 정확한 명령

```bash
sha256sum <8 primary interactive runtime files> | sha256sum
sha256sum <7 shared carrier/consumer files> | sha256sum
sha256sum <3 typed loader/data files + 6 live RON files> | sha256sum
sha256sum <5 focused test files> | sha256sum
sha256sum core_enhanced/docs/{game_rulebook.md,core_runtime_contract.md,data_loading_contract.md} | sha256sum
git status --short -- <same 35 production paths> | sha256sum
git status --short -- <same 35 production paths>
```

The exact ordered path lists are the files named in `PLAN.md` plus the following baseline expansion:

- primary: `map/{facility,facility_generator}.rs`
- shared: `world.rs`, `behavior.rs`, `world/{state,node_flow,map_content,snapshot,active_run}.rs`
- data/live: `data/{visual_novel_data,visual_novel_trigger_data,live_ron}.rs`,
  `events/{rewards,shops,story}/base.ron`, `map/facility_templates.ron`,
  `visual_novels/base.ron`, `visual_novel_triggers/base.ron`
- tests: `world/tests/{node_sessions,facility,map_flow,snapshots_and_start}.rs`, `tests/ron_loading.rs`

### 실제 실행된 test 수

Test runner를 실행하지 않았다. 해당 없음.

### Pass / Fail / Skip

Test 기준 해당 없음. 모든 baseline path는 읽을 수 있었고 six aggregate hashes를 계산했다.

### 실제 결과

| Category | Initial aggregate SHA-256 |
| --- | --- |
| primary interactive runtime | `b7290f9a501927273c09c8096c2f8dc5eb348975ba83c09e9cfb5d19a4ce2c98` |
| shared carrier/consumer | `79b8defc40f472db6e68e997d2a4d01d330cd3e2377663eec41f28200b47a49c` |
| typed data and live RON | `13229ab33c62a93fb3bd55ab35d92b4d7aa42f7316b77d5f732418022a9215f2` |
| focused test source | `cab5a479f5c75a5d46e63364e4fecd6594c5f6850938ecc3119698ef034d8ccd` |
| nearest canonical | `da3f35594f77174b9251da9b6ee7a2fb2e9e4fb9bfeb2da33c366034325c6fac` |
| scoped status text | `b14f3d7b528d5e3dc2c88fd4b9d9239ab4e619961683edcf78206c72fe6194f0` |

Fifteen scoped paths were already modified in the user-owned baseline: the three canonical files,
`behavior.rs`, `data/live_ron.rs`, `world.rs`, `world/{active_run,node_flow,snapshot,state,visual_novel_flow}.rs`,
`world/tests/{facility,snapshots_and_start}.rs`, `tests/ron_loading.rs`, and
`game_resources/data/events/rewards/base.ron`.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

이 six hashes와 exact scoped status set을 Phase exit에 같은 ordered path list로 다시 계산한다. 이후 새
직접 evidence path가 발견되면 별도 baseline category를 조사 전에 추가하고 그 이유를 기록한다.

## 2026-08-07 — Facility-family lifecycle baseline expansion

### 검증하려던 가설

`ActiveNodeContent` inventory 결과, Phase 0의 facility/session 경계가 하나의 generic `FacilitySession`이
아니라 SavePoint/Maintenance/Shop/Reward/HeadquartersContact lifecycle seam으로 구현되어 있으므로 경제
내부를 다시 열지 않고 session admission/completion을 감사할 supplemental paths가 필요한지 확인했다.

### 수행한 실험 또는 검증

- `FacilitySession`, `ActiveNodeContent` variant와 `commit_interactive_node_completion` symbol inventory
- 아직 상세 내용을 읽지 않은 selection/helper/HQ/reward/shop lifecycle file의 aggregate hash/status 고정

### 변경한 내용

Phase scope wording과 이 audit 기록만 갱신했다. Production 파일은 변경하지 않았다.

### 실행한 정확한 명령

```bash
rg -n '<facility/session/lifecycle symbols>' core_enhanced/src/game --glob '*.rs'
sha256sum core_enhanced/src/game/resources/selection.rs core_enhanced/src/game/world/{helpers,headquarters,reward,shop}.rs | sha256sum
git status --short -- <same five files> | sha256sum
git status --short -- <same five files>
```

### 실제 실행된 test 수와 결과

Test runner를 실행하지 않았다. 해당 없음.

### 실제 결과

- Supplemental lifecycle content hash:
  `160d2de4213c56243e85e74c9b0486de3e161a5c4eb187a626b6e4b15f2a755e`.
- Supplemental scoped-status hash:
  `35533aaf3d279690e79be86835ea3d9549f52a36fd6467799b7911f0e58c0857`.
- 이 다섯 파일 중 `world/helpers.rs`만 user-owned baseline에서 modified다.

### 결과에 따른 판단

Phase 4는 Facility-family session lifecycle과 common node completion을 포함한다. Shop/reward/HQ 비용·grant
계산 자체와 maintenance asset operations는 Phase 3 authority로 남기며 lifecycle/atomic handoff만 읽는다.

## 2026-08-07 — Static authority, replay and test-gap trace

### 검증하려던 가설

Event/Facility/VN이 같은 `ActiveNodeContent` 또는 top-level state에 보인다는 사실만으로 authority 중복을
뜻하는지, 아니면 checkpoint/re-entry와 현재 interaction projection을 위한 의도된 역할 분리인지 확인했다.
또한 runner를 실행하기 전에 validation·rollback·test assertion의 구체적인 정적 공백이 있는지 찾았다.

### 수행한 실험 또는 검증

- Event enter/advance/resolve/grant/combat/complete와 `RunState.event_sessions`/active mirror 전체 추적
- Shop/Reward/SavePoint/Maintenance/HQ enter/action/complete와 common node transaction 추적
- VN trigger sort/match/repeat/history/session construction, exact command identity, queue promotion, HQ/Monolith
  completion hook과 checkpoint carriage 추적
- published Event/VN RON, typed validation, official live bundle gate와 nearest canonical 대조
- Phase 4 focused test source의 함수 단위 invariant/overlap inventory

### 실행한 정확한 명령

```bash
sed -n '<chunks>' core_enhanced/src/game/world/{event_node,facility,map_content,node_flow,headquarters,reward,shop,visual_novel_flow,visual_novel_triggers}.rs
sed -n '<chunks>' core_enhanced/src/game/{visual_novel,resources/selection,map/session}.rs
sed -n '<chunks>' core_enhanced/src/game/data/{event_data,visual_novel_data,visual_novel_trigger_data,validation,live_ron}.rs
sed -n '<chunks>' core_enhanced/src/game/world/tests/{snapshots_and_start,facility,node_sessions,map_flow,combat}.rs
sed -n '<chunks>' core_enhanced/docs/{core_runtime_contract,data_loading_contract,game_rulebook}.md
rg -n '<session/trigger/replay/commit/checkpoint/test symbols>' core_enhanced/src core_enhanced/docs game_resources/data
```

### 실제 실행된 test 수

아직 runner를 실행하지 않았다. 아래 공백을 먼저 기록했으므로 다음 focused runner는 현재 safety net의
baseline을 확인하는 용도로 허용한다.

### 실제 결과

- `RunState.event_sessions`/`shop_sessions`는 checkpoint와 node re-entry를 위한 지속 authority이고,
  `active_node_content`는 현재 command/projection mirror다. Event/Shop update helper가 둘을 함께 쓰므로 역할
  자체는 의도된 분리다.
- Event resolve는 exact node/event/source-scene/choice/employee duplicate만 저장 결과로 반환하며 reroll/regrant
  하지 않는다. 다른 stale identity는 mutation 전에 거부하고 grant/combat는 clone-stage한다.
- common node completion과 node enter는 full non-battle transaction을 사용한다. HQ만 10개 field를 직접
  열거하는 별도 rollback helper와 active-content `take`/restore planner 우회를 가진다.
- Shop/Event의 paired persistent+active write는 domain helper가 있음에도 map/HQ entry에서 다시 수동
  구현된다. 현재 테스트는 결과를 보호하지만 모든 paired-write entry가 같은 helper를 통과하는지는
  구조적으로 보장하지 않는다.
- VN trigger session과 Monolith exchange session은 UUID/definition entry/presentation/session assembly를 각각
  수동 구현한다. Trigger는 queue tail, Monolith는 기존 queue 앞을 의도적으로 사용한다.
- VN command는 exact `session_id + current_scene_id`를 검증한다. Trigger는 priority descending/id ascending,
  successful activate/queue 시 repeat history를 commit하고 active를 interrupt하지 않는다.
- official live loader는 supported mode별 exact RunStart와 RunStart policy/reference를 검증한다. 그러나
  `MapNodeEntered + OncePerBattle`은 typed/live validation을 통과한 뒤 runtime `consumed_key_for_trigger()`에서만
  거부된다. Node-enter transaction은 rollback하지만 invalid authored bundle이 gameplay까지 publish된다.
- `VisualNovelSession.committed_choice_id`는 모든 constructor에서 `None`, scene change에서 다시 `None`으로만
  쓰이고 `Some` writer가 없다. Snapshot choice-selectability가 이 field를 읽지만 현재 항상 true다. Public
  serialized Rust type의 외부 consumer는 확인되지 않아 즉시 삭제 증거는 부족하다.
- 현재 focused tests에는 HQ completion의 late failure가 roster/currency/recruitment까지 rollback되는지와
  Monolith exchange VN의 queue-front/accept/decline world lifecycle을 직접 고정하는 integration test가 없다.
  VN trigger data unit tests에도 runtime-rejected repeat/source 조합의 publication rejection test가 없다.

### 결과에 따른 판단

다음 runner는 기존 Event/HQ/facility/VN baseline을 확인한다. Green 결과를 공백 해소로 간주하지 않고,
정적 공백은 candidate safety-net 요구로 유지한다. Generic Event/Facility/VN session merge와 persistent
session cache 제거는 현재 증거상 rejected signal이다.

## 2026-08-07 — Focused current-tree runner baseline

### 검증하려던 가설

정적 source audit에서 의미를 확인한 Event/Facility/HQ/Shop/Reward/VN test families가 현재 dirty tree에서
실제로 실행 가능하고 green인지, 그리고 기록한 missing negative/late-failure/Monolith lifecycle test가
기존 runner 이름 뒤에 숨지 않는지 확인했다.

### 실행한 정확한 명령과 결과

```bash
cargo test -p game_core --lib visual_novel -- --nocapture
# 23 passed, 0 failed, 0 ignored, 839 filtered out

cargo test -p game_core --lib game::world::tests::facility:: -- --nocapture
# 11 passed, 0 failed, 0 ignored, 851 filtered out

cargo test -p game_core --lib game::world::tests::node_sessions:: -- --nocapture
# 18 passed, 0 failed, 0 ignored, 844 filtered out

cargo test -p game_core --lib game::world::tests::map_flow:: -- --nocapture
# 40 passed, 0 failed, 0 ignored, 822 filtered out

cargo test -p game_core --lib game::world::event_node::tests:: -- --nocapture
# 5 passed, 0 failed, 0 ignored, 857 filtered out

cargo test -p game_core --lib battle_started_trigger -- --nocapture
# 1 passed, 0 failed, 0 ignored, 861 filtered out

cargo test -p game_core --lib battle_time_trigger -- --nocapture
# 1 passed, 0 failed, 0 ignored, 861 filtered out

cargo test -p game_core --lib game::world::tests::snapshots_and_start:: -- --nocapture
# 23 passed, 0 failed, 0 ignored, 839 filtered out

cargo test -p game_core --lib run_start -- --nocapture
# 4 passed, 0 failed, 0 ignored, 858 filtered out
```

### 실제 실행된 test 수

Runner invocation 합계 126개 test 실행: 126 passed, 0 failed, 0 ignored/skipped. Filter가 겹치므로 이것은
126개의 unique test라는 뜻이 아니다. `visual_novel`, full facility/snapshot, `run_start` 사이에 의도된
재실행이 있다.

### 실제 결과

- 모든 focused runner가 성공했고 baseline failure는 없다.
- Event exact duplicate/no-regrant, stale identity no-mutation, one-roll checkpoint reuse, retreat resume와 live
  branch tests가 실행됐다.
- HQ/session queue, Shop persistence, Reward failure atomicity, SavePoint/Maintenance, VN priority/queue/checkpoint,
  RunStart loader policy, battle-start/time trigger transport가 실행됐다.
- Runner 목록에도 `MapNodeEntered + OncePerBattle` validation rejection, late HQ completion rollback,
  Monolith exchange VN world accept/decline/queue-front test는 없다. Green 결과는 이 세 safety-net 공백을
  해소하지 않는다.

### 실패 원인

없음. Cargo package/build lock, compilation 또는 runtime fixture failure도 관찰되지 않았다.

### 되돌리거나 수정한 내용

없음. Cargo는 workspace `target` build artifacts만 사용했고 production source/data/test/canonical은 수정하지
않았다.

## 2026-08-07 — Phase exit and no-mutation audit

### 검증하려던 가설

P4-01~P4-11과 CRA-01~CRA-14가 모두 evidence-backed disposition을 갖고, 후보 4개와 signal 9개 및
assigned test family 13개가 필수 필드를 충족하며, 조사 전후 production content/status가 동일한지
독립적으로 재확인했다.

### 수행한 실험 또는 검증

- `BOUNDARY_AUDIT.md`, `CANDIDATE_CATALOG.md`, `TEST_AUDIT.md` 전체 재독
- candidate heading과 10개 필수 field, signal disposition, test-family row 수 기계적 확인
- Phase Goal/Master audit-doc whitespace and status 확인
- 초기와 동일한 ordered path list로 six primary hashes와 supplemental lifecycle hashes 재계산
- focused runner 결과와 omitted validation 재대조

### 실행한 정확한 명령

```bash
rg -c '^### CRA-P04-C[0-9]+' CANDIDATE_CATALOG.md
rg -c '<10 candidate required fields>' CANDIDATE_CATALOG.md
rg -c '^| CRA-P04-S[0-9]+' CANDIDATE_CATALOG.md
rg -c '<test family rows>' TEST_AUDIT.md
git diff --check -- <Master and Phase 4 Goal paths>
sha256sum <same ordered primary/shared/data/test/canonical path lists> | sha256sum
sha256sum <same supplemental lifecycle path list> | sha256sum
git status --short -- <same primary scoped paths> | sha256sum
git status --short -- <same supplemental paths> | sha256sum
```

The primary ordered list starts `visual_novel.rs`, `employee_psychology.rs`, then the four world runtime files and
the two map facility files; using another order changes the aggregate despite identical content.

### 실제 실행된 test 수와 결과

Exit audit에서 runner를 추가 실행하지 않았다. Phase focused baseline은 126 executions, 126 passed,
0 failed, 0 ignored/skipped이며 overlapping filters임을 유지한다.

### 실제 결과

| Category | Initial | Final |
| --- | --- | --- |
| primary interactive runtime | `b7290f9a501927273c09c8096c2f8dc5eb348975ba83c09e9cfb5d19a4ce2c98` | same |
| shared carrier/consumer | `79b8defc40f472db6e68e997d2a4d01d330cd3e2377663eec41f28200b47a49c` | same |
| typed data and live RON | `13229ab33c62a93fb3bd55ab35d92b4d7aa42f7316b77d5f732418022a9215f2` | same |
| focused test source | `cab5a479f5c75a5d46e63364e4fecd6594c5f6850938ecc3119698ef034d8ccd` | same |
| nearest canonical | `da3f35594f77174b9251da9b6ee7a2fb2e9e4fb9bfeb2da33c366034325c6fac` | same |
| primary scoped status text | `b14f3d7b528d5e3dc2c88fd4b9d9239ab4e619961683edcf78206c72fe6194f0` | same |
| supplemental lifecycle content | `160d2de4213c56243e85e74c9b0486de3e161a5c4eb187a626b6e4b15f2a755e` | same |
| supplemental scoped status | `35533aaf3d279690e79be86835ea3d9549f52a36fd6467799b7911f0e58c0857` | same |

- Confirmed candidates: 4, each with all 10 required fields.
- Disposed signals: 9.
- Test-family disposition rows: 13, with one exact mergeable redundant unit example and four explicit safety-net
  strengthening needs.
- `git diff --check` reported no Phase/Master audit-doc whitespace error.

### 결과에 따른 판단

Phase 4 verdict는 `Complete / High`다. Phase 5 activation dependency가 충족됐다. C01 correction과 C02~C04
refactor는 구현하지 않고 Final catalog로 handoff한다.
