# Core Enhanced Refactoring Audit Phase 02 — Experiments

Status: complete

## 2026-08-07 — Phase initialization

### 검증하려던 가설

Phase 1이 `Complete / High`이고 Phase 2 예약 경로 충돌/다른 active child 없이 승인된 sequential
run-map-persistence investigation 문서를 초기화할 수 있다.

### 수행한 실험 또는 검증

- Parent Master와 Phase 1 completion audit 확인
- Phase 0 boundary proposal의 Phase 2 범위 확인
- Phase 2 경로 존재 여부와 active audit child 확인
- Phase 1 production-content/status hash 보존 결과 확인

Production run/map/persistence source search, test runner와 formatter는 Phase 2 필수 문서 생성 전에
수행하지 않았다.

### 변경한 내용

- 이 Phase의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md` 생성
- Parent Master registry에서 Phase 1을 complete, Phase 2를 active로 전환

Production code, live data, test, canonical과 server code는 수정하지 않았다.

### 실행한 정확한 명령

```bash
if test -e core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_02_run_map_and_persistence; then echo EXISTS; else echo MISSING; fi
find core_enhanced/docs/goals -maxdepth 2 -name PLAN.md -path '*core_enhanced_refactoring_audit_phase_*' -print | sort
rg -n '^Status: active' core_enhanced/docs/goals/core_enhanced_refactoring_audit_*/*
```

### 실제 실행된 test 수

Phase initialization test runner는 실행하지 않았다. 해당 없음.

### Pass / Fail / Skip

Test 기준 해당 없음. Dependency/path/active-child gate는 통과했다.

### 실제 결과

- Phase 1 completion matrix 10/10 pass, verdict `Complete / High`.
- Phase 2 path는 생성 전에 존재하지 않았다.
- Phase 1 외 active child는 없었다.
- Phase 1 initial/final production hashes와 scoped status hash가 동일했다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음. 기존 dirty/untracked production 파일을 수정, 이동, 삭제, format, restore하지 않았다.

### 결과에 따른 판단

Phase 2를 `investigation_only`, production mutation `no`로 활성화할 수 있다. 실제 조사 전 필수 문서와
direct dependency를 다시 읽는다.

## 아직 수행하지 않은 실험과 검증

- exact run/map/checkpoint/persistence surface inventory
- authoritative mutation/transaction/rollback trace
- save/version/server store/bootstrap trace
- focused test inventory/runner
- candidate/disposition and completion audit

위 항목은 `discovery pending`이다.

## 2026-08-07 — Run/map/checkpoint/server persistence investigation

### 검증하려던 가설

현재 Core가 run/map/checkpoint gameplay authority를 소유하고 server는 host envelope/lifetime만 소유하는지,
그리고 이 경계의 중복·부분 mutation·fallback·version·단조 메타데이터 신호를 실제 코드/테스트로 모두
분류할 수 있는지 확인했다.

### 수행한 실험 또는 검증

- 초기 dirty 목록과 다음 content hash를 고정했다.
  - Core world/map/resources: `4a734882937f92eb7e564b25943aac5f00e442d542cd1aedaa77a684697a0da4`
  - shared `world.rs`/`behavior.rs`/`run_policy_data.rs`:
    `415b3c0c0853421f8283970852cbb1d78f8f0e0dab96c40b62c09af715818bea`
  - Core integration tests: `08ed65a65446a8564e59af80a69e2d16150f0a182e25b1638c5ca6a1cee8e892`
  - relevant canonical: `11f848dcd4f1a8d911fe70a5af3506cfc00de6fa06a0c97a5dff702262870ade`
  - server root/persistence baseline: `2c57784db98c82909032849c90e08bd7302528bbb7b9c44a70752c41f9624c42`
  - direct active-run actor/store seam reconciliation baseline:
    `3a7c535fabb9e963a0ab42d1ca486ad1f757c2ace47f175e020199647bace97d`
- `StartNewGame`→briefing→starter selection→`RunProgression`/facility generation과 run seed/UUID namespace를
  추적했다.
- `SelectMapNode` preview, confirmed entry, completion transaction, directed transit, Gate/Floor advance와 run
  completion을 추적했다.
- SavePoint capture, explicit load, max 3회, V3 export/import, host monotonic metadata, UUID/VN counter와 rollback
  exclusion을 추적했다.
- server title projection/Continue, current/backup/checksum/temp/fsync/tombstone, publish failure rollback과
  compendium 독립 transaction을 추적했다.
- legacy/alias/migration/fallback/ignored-test와 direct external consumer를 검색했다.
- canonical의 단조 load/peak, strict V3, fixed map, current+backup 의미를 runtime과 대조했다.

### 변경한 내용

- Phase Goal 문서에만 `BOUNDARY_AUDIT.md`, `CANDIDATE_CATALOG.md`와 이 조사 기록을 추가했다.
- production source/data/test/canonical/server 파일은 변경하지 않았다.

### 실행한 대표 명령

```bash
rg -n "RunCheckpoint|run_checkpoint|RunProgression|MapProgression|ContinueRun|StartNewGame" core_enhanced/src game_server/src
sed -n '1,240p' core_enhanced/src/game/world/active_run.rs
sed -n '1200,1425p' core_enhanced/src/game/world/node_flow.rs
sed -n '1,495p' game_server/src/persistence/active_run_store.rs
sed -n '120,390p' game_server/src/game/player_game_actor/handlers.rs
git diff -- <Phase 2 relevant existing dirty production paths>
rg -n "#\[ignore" <Phase 2 focused source/test paths>
```

### 실행한 test와 정확한 결과

| Command/filter | Passed | Failed | Ignored | Filtered |
| --- | ---: | ---: | ---: | ---: |
| `cargo test -p game_core --lib game::map:: -- --nocapture` | 25 | 0 | 0 | 837 |
| `cargo test -p game_core --lib game::world::tests::facility -- --nocapture` | 11 | 0 | 0 | 851 |
| `cargo test -p game_core --lib game::world::tests::map_flow -- --nocapture` | 40 | 0 | 0 | 822 |
| `cargo test -p game_core --lib game::world::tests::node_sessions -- --nocapture` | 18 | 0 | 0 | 844 |
| `cargo test -p game_core --lib game::world::tests::snapshots_and_start -- --nocapture` | 23 | 0 | 0 | 839 |
| `cargo test -p game_core --lib game::managers::uuid_manager::tests -- --nocapture` | 2 | 0 | 0 | 860 |
| `cargo test -p game_server active_run -- --nocapture` | 13 | 0 | 0 | 43 library; main 0 |
| `cargo test -p game_core --lib transaction -- --nocapture` | 4 | 0 | 0 | 858 |

총 runner execution은 136 pass, 0 fail, 0 ignored다. `typed_monolith_stage_round_trips_checkpoint_continue_and_transaction_rollback`
한 건이 node-session과 transaction filter에 모두 포함되어 서로 다른 test name은 135개다. Filtered는
filter와 이름이 맞지 않은 test이며 skip/pass로 세지 않았다.

### 실제 결과

- 정상 생성/진행/checkpoint/backup/version/rollback 경로는 focused safety net을 통과했다.
- current publish는 lower incoming monotonic metadata를 방어하지 않아 C01로 분류했다.
- import validator는 RNG seed와 semantic progression cross-field invariant를 완전히 검증하지 않아 C02로
  분류했다.
- 13개 run field의 수동 checkpoint 양방향 mapping은 C03, map availability/progression의 분산 mutation은
  C04로 분류했다.
- local-install store를 서로 다른 player actor가 동시에 소유할 때의 lease/conflict 정책은 canonical이
  결정하지 않으므로 S05 deferred로 남겼다.
- Core/host V3 분리, backup 복구, strict no-migration과 validated-generation failure는 의도된 계약으로
  rejected/deferred 처리했다.

### 실패 원인

Test 실패는 없었다. 병렬 Core filter 실행은 Cargo package/build lock을 순차 획득했지만 모든 runner가
정상 종료했다. 한 번의 source `sed`는 `core_enhanced` workdir에서 repository-root 상대 server 경로를 사용해
파일을 찾지 못했으나, 같은 파일은 앞뒤 repository-root 명령에서 완전히 읽혔고 test 결과에는 영향이 없다.

### 되돌리거나 수정한 내용

없음. 기존 dirty production 변경을 restore/format/move/delete하지 않았다.

### 결과에 따른 판단

현재 evidence로 Phase 2의 candidate/disposition과 completion audit을 작성할 수 있다. C01/C02는 test-first
correctness correction, C03/C04는 serialization/transition characterization 뒤 behavior-preserving refactor다.

## 아직 수행하지 않은 실험과 검증 — investigation 후

- C01 lower-current characterization test
- C02 corrupted-but-current-V3 seed/progression negative tests
- C03 golden V3 field-parity test
- C04 transition invariant characterization
- cross-player active-run lease/conflict policy 및 Unity restore 검증

위 항목은 이 read-only Phase의 누락이 아니라 후보 implementation/deferred policy work다.

## 2026-08-07 — Final production-preservation verification

초기와 동일한 파일 순서/집합으로 다시 계산한 7개 hash는 모두 동일했다.

- Core world/map/resources: `4a734882937f92eb7e564b25943aac5f00e442d542cd1aedaa77a684697a0da4`
- shared root/run policy: `415b3c0c0853421f8283970852cbb1d78f8f0e0dab96c40b62c09af715818bea`
- Core integration tests: `08ed65a65446a8564e59af80a69e2d16150f0a182e25b1638c5ca6a1cee8e892`
- relevant canonical: `11f848dcd4f1a8d911fe70a5af3506cfc00de6fa06a0c97a5dff702262870ade`
- server root/persistence: `2c57784db98c82909032849c90e08bd7302528bbb7b9c44a70752c41f9624c42`
- direct actor/store seam: `3a7c535fabb9e963a0ab42d1ca486ad1f757c2ace47f175e020199647bace97d`
- direct Phase production status: `ddec869d42b79d8ea5badc65b08b68bcdc9c998022cd48ec3390395c6b38c28a`

Completion matrix 10/10 pass, verdict `Complete / High`다.
