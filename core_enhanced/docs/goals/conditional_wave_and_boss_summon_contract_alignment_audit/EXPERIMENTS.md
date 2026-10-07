# Experiments — Conditional Wave and Boss Summon Contract Alignment Audit

## 상태

- Goal mode: `audit_only`
- 실제로 수행한 실험 또는 검증: 없음
- Production 변경: 없음
- Test 실행: 없음

Goal 문서 초기화와 사전 `git status --short` 확인은 runtime 가설을 검증한 실험이 아니므로 experiment로
기록하지 않는다.

## 기록 규칙

실제로 수행한 read-only 조사, probe 또는 test만 시간순으로 추가한다. 각 기록은 가능한 범위에서 다음을
포함한다.

- 시각 또는 순서
- 연결된 Alignment Matrix Row ID
- 검증하려던 가설
- 조사·실험·검증 내용
- 변경한 내용 (`없음`이어야 하며, 필요해지면 이 Goal을 중단)
- 실행한 정확한 명령 또는 test runner job
- 실제 실행된 test 수
- pass/fail/skip 수
- 실제 결과
- 실패 또는 관찰 불가 원인
- 되돌리거나 수정한 내용
- 결과에 따른 verdict 또는 다음 판단

Runner가 test 수나 pass/fail/skip 수를 제공하지 않으면 추측하지 않고 `runner가 보고하지 않음`으로 기록한다.
계획했지만 실행하지 않은 명령은 결과처럼 기록하지 않는다.

## Planned Validation — 아직 실행하지 않음

- Freeze member 및 임시 정책 노트 section coverage 검사
- 각 row의 `game_rulebook.md` clause와 runtime entrypoint→mutation/result 추적
- applicable live RON→loader→runtime 경로 확인
- applicable producer/consumer와 existing test seam 확인
- 정적 근거가 불충분한 row에 한정한 최소 focused probe 또는 test
- 최종 verdict/evidence 및 dirty-worktree 보존 확인

## E-001 — 감사 source basis와 정책 member inventory 고정

- 순서: 1
- 연결된 Alignment Matrix Row ID: 전체 25개 row의 공통 basis
- 가설: `FPF-CWBS-P2G30-002`가 current 승인 정책이며 Freeze member와 임시 정책 노트 section을 완전하게
  inventory할 수 있다.
- 수행한 조사:
  - 감사 대상의 scoped `git status`와 tracked diff path를 확인했다.
  - Working Policy current-selection metadata와 Final Freeze payload를 확인했다.
  - Final Freeze SHA-256과 canonical 문서 SHA-256을 고정했다.
  - Freeze ordered member/acceptance 25개와 임시 정책 노트 policy heading을 대조했다.
- 변경한 내용: Goal 실행 기록만 갱신; production/canonical 변경 없음
- 실행한 명령:

```bash
git status --short -- core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy core_enhanced/src/game/battle/core core_enhanced/src/game/combat_setup core_enhanced/src/game/data game_resources/data/boss_runtime game_resources/data/summoned_units game_resources/data/pve/encounters.ron game_server/src game_server/docs
git diff --name-status -- core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/src/game/battle/core core_enhanced/src/game/combat_setup core_enhanced/src/game/data game_resources/data/pve/encounters.ron game_server/src game_server/docs
sha256sum core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/WORKING_POLICY.ko.md core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/P2G_30_FINAL_POLICY_FREEZE_002.ko.md core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md
rg -n 'Current|current|P2G_30_FINAL_POLICY_FREEZE_002|Freeze Ref|Content Digest|Status|Disposition|Member|Policy Element' core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/WORKING_POLICY.ko.md core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/P2G_30_FINAL_POLICY_FREEZE_002.ko.md
sed -n '78,223p' core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/P2G_30_FINAL_POLICY_FREEZE_002.ko.md
rg -n '^### ' core_enhanced/docs/tmp/conditional_wave_and_boss_summon_policy_notes_2026-08-13.ko.md
```

- 실제 실행된 test 수: N/A — test runner를 실행하지 않음
- pass/fail/skip: N/A
- 실제 결과:
  - Working Policy는 `FPF-CWBS-P2G30-002`를 current planning Freeze로 선택한다.
  - Freeze SHA-256은 `4c94407a83e3252635d2e033fe792e98dbafa6ce202b0cfc8f52d910780838e8`이다.
  - Freeze는 정확히 25개의 ordered `code_reviewed` member와 25개의 Observable Policy Acceptance를 가진다.
  - 임시 정책 노트의 `SP-01`부터 `RF-01`까지 정책 section도 정확히 같은 25개 ID다.
  - 감사 대상 canonical/runtime/data/server 경계에는 기존 modified/untracked 변경이 다수 존재한다.
- 실패 또는 관찰 불가 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 판단: exact current dirty source를 대상으로 25개 정책 요소의 fresh audit을 진행한다. 임시 노트의
  `알려진 현재 구현 차이`는 과거 기준선이므로 current actual evidence로 사용하지 않는다.

## E-002 — 재개 source basis 재고정

- 순서: 2
- 연결된 Alignment Matrix Row ID: 전체 25개 row의 공통 basis
- 가설: 감사 재개 시점에도 `FPF-CWBS-P2G30-002`가 current 승인 정책이며, 2026-08-17 관찰 이후 바뀐
  canonical/runtime source가 있는지를 분리할 수 있다.
- 수행한 조사:
  - 감사 경계와 Goal 경로의 scoped dirty 상태를 다시 확인했다.
  - Working Policy, Final Freeze, `game_rulebook.md`, runtime/data canonical 문서의 SHA-256을 다시 계산했다.
  - 이전 `ASB-001`과 비교해 Freeze, Working Policy, Core/runtime·data canonical 문서는 같은 digest이고,
    `game_rulebook.md`만 새 digest임을 확인했다.
- 변경한 내용: Goal 실행 기록만 갱신; production/canonical 변경 없음
- 실행한 명령:

```bash
git status --short -- core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy core_enhanced/src/game/battle/core core_enhanced/src/game/combat_setup core_enhanced/src/game/data game_resources/data/boss_runtime game_resources/data/summoned_units game_resources/data/pve/encounters.ron game_server/src game_server/docs
git diff --name-status -- core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy core_enhanced/src/game/battle/core core_enhanced/src/game/combat_setup core_enhanced/src/game/data game_resources/data/boss_runtime game_resources/data/summoned_units game_resources/data/pve/encounters.ron game_server/src game_server/docs
sha256sum core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/WORKING_POLICY.ko.md core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/P2G_30_FINAL_POLICY_FREEZE_002.ko.md core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md
```

- 실제 실행된 test 수: N/A — test runner를 실행하지 않음
- pass/fail/skip: N/A
- 실제 결과:
  - `FPF-CWBS-P2G30-002`와 Working Policy digest는 `ASB-001`과 같다.
  - `core_runtime_contract.md`, `data_loading_contract.md` digest도 `ASB-001`과 같다.
  - `game_rulebook.md` digest는 `ASB-001`의
    `3aeec0e8e3eab5c57002083e0cead4f29003c31b39c00f5f877cbb40f08ac086`에서
    `8aa954cc215dccc63b736f884327ca03375f82895b5e86eb2252e597e8fcbf6f`로 바뀌었다.
  - core/runtime/data/server와 canonical 문서 경계에는 기존 modified/deleted/untracked 사용자 변경이
    계속 존재한다. 감사 Goal 문서 경로도 untracked이며 이 실행 기록만 갱신했다.
- 실패 또는 관찰 불가 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 판단: 이전 rulebook observation은 재사용하지 않고, 현재 `game_rulebook.md`와 current dirty runtime을
  기준으로 25개 행을 다시 대조한다.

## E-003 — Wave·Spawn Kernel current runtime focused 검증

- 순서: 3
- 연결된 Alignment Matrix Row ID: `SP-01`, `SP-02`, `WV-01`~`WV-07`
- 가설: current Wave Runtime과 Spawn Kernel이 Frozen trigger, ordering, required-victory gate, exact-once 및
  all-or-none spawn 계약을 실제 test seam에서 유지한다.
- 수행한 조사: Wave registration/binding/settlement와 Spawn Kernel identity/preflight/transaction source를 읽고,
  해당 모듈 test를 실행했다.
- 변경한 내용: 없음
- 실행한 명령:

```bash
cargo test -p game_core --lib 'battle::core::wave_runtime::tests'
cargo test -p game_core --lib 'battle::core::spawn_kernel::tests'
```

- 실제 실행된 test 수: 20
- pass/fail/skip: 20 passed / 0 failed / 0 ignored (각 runner는 추가로 1,070 filtered를 보고함)
- 실제 결과:
  - Wave test 10개가 same-bucket priority/ID ordering, primary-over-deadline, required unspawned Wave Victory
    hold, HP/point bind-time evaluation, prospective death, cancellation과 atomic rejection을 통과했다.
  - Spawn Kernel test 10개가 value-bearing occupancy, full preflight, atomic rollback/cached settlement,
    removal-first cancel, identity conflict와 ordered compendium candidate ledger를 통과했다.
- 실패 또는 관찰 불가 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 판단: 해당 row의 runtime evidence는 source trace와 current focused test로 충분하다.

## E-004 — Summon lifecycle current runtime focused 검증

- 순서: 4
- 연결된 Alignment Matrix Row ID: `BR-01`~`BR-05`, `SM-01`~`SM-04`, `SP-01`, `SP-02`, `WV-04`
- 가설: current Summon Runtime이 typed result, provenance, limit/counter, placement, removal 및 batch-clear
  계약을 atomic하게 유지한다.
- 수행한 조사: pending request settlement, batch commit, removal reconciliation과 candidate placement source를
  읽고 해당 모듈 test를 실행했다.
- 변경한 내용: 없음
- 실행한 명령:

```bash
cargo test -p game_core --lib 'battle::core::summon_runtime::tests'
```

- 실제 실행된 test 수: 10
- pass/fail/skip: 10 passed / 0 failed / 0 ignored (runner는 추가로 1,070 filtered를 보고함)
- 실제 결과: independent definition/provenance, atomic limit/space failure, FIFO waiter, owner-state cancel,
  placement order/occupancy, per-summoner counter, Persist/Despawn, lifetime/signal binding, exact batch-clear와
  non-victory gating을 모두 통과했다.
- 실패 또는 관찰 불가 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 판단: 해당 row의 runtime evidence는 source trace와 current focused test로 충분하다.

## E-005 — live RON publication focused 검증

- 순서: 5
- 연결된 Alignment Matrix Row ID: live RON이 applicable인 전체 row, 특히 `RF-01`, `SP-02`, `BR-01`~`BR-05`,
  `SM-01`~`SM-04`
- 가설: current live RON loader가 Boss Runtime과 summoned-unit catalog를 한 compiled publication으로
  받아들인다.
- 수행한 조사: loader manifest, domain validation, live `reference_cwbs_final_boss`, Boss Runtime 및 summoned
  unit RON을 추적하고 loader test를 실행했다.
- 변경한 내용: 없음
- 실행한 명령:

```bash
cargo test -p game_core --lib 'game::data::live_ron::tests::successful_live_publication_compiles_boss_runtime_exactly_once'
```

- 실제 실행된 test 수: 1
- pass/fail/skip: 1 passed / 0 failed / 0 ignored (runner는 추가로 1,079 filtered를 보고함)
- 실제 결과: live publication이 compiled Boss Runtime을 정확히 한 번 구성하는 seam이 통과했다.
- 실패 또는 관찰 불가 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 판단: live RON file presence만이 아니라 loader→compiled database 경로의 current evidence를 확보했다.

## E-006 — same-process reconnect server seam 확인

- 순서: 6
- 연결된 Alignment Matrix Row ID: `PS-01`, `BL-04`, `BL-05`
- 가설: server actor가 active battle disconnect를 Pause하고 reconnect에 동일 projection을 보내며, 이 경계가
  current source에서 test로도 실행 가능한지 확인한다.
- 수행한 조사: `PlayerGameActor` attach/detach source와
  `detached_live_battle_has_no_ttl_and_reconnects_paused` test body를 읽었다. test runner도 실행했다.
- 변경한 내용: 없음
- 실행한 명령:

```bash
cargo test -p game_server detached_live_battle_has_no_ttl_and_reconnects_paused
```

- 실제 실행된 test 수: runner가 보고하지 않음 — test binary 실행 전 compiler가 중단됨
- pass/fail/skip: runner가 보고하지 않음
- 실제 결과: Rust 1.91.1 incremental compilation이 `rustc_metadata::...def_path_hash_map.rs`에서 panic했고,
  Cargo는 `game_server (lib test)` compile exit status 101을 보고했다. 테스트 assertion failure가 아니다.
- 실패 또는 관찰 불가 원인: current environment의 Rust compiler ICE. 별도 target directory/incremental off
  재시도는 30초 runner capture 안에 dependency compilation만 출력하고 test result를 만들지 못했다.
- 되돌리거나 수정한 내용: 없음
- 판단: server source와 existing test seam은 `PS-01` actual path의 read-only evidence로 사용하되, 이 Goal은
  server test pass를 주장하지 않는다. Core focused test 31개와 static server trace로 audit verdict를 제한한다.

## E-007 — final coverage·source currentness·dirty preservation 확인

- 순서: 7
- 연결된 Alignment Matrix Row ID: 전체 25개 row와 final completion
- 가설: 25개 Freeze member coverage가 complete이고 ASB-002 source basis가 조사 중 stale해지지 않았으며,
  Goal 외 파일을 수정하지 않았다.
- 수행한 조사:
  - Matrix의 exact 25 row ID를 count했다.
  - Goal 문서의 trailing whitespace를 검색했다.
  - Goal 경로, canonical/runtime/data/server 관련 scoped `git status`를 다시 확인했다.
  - Freeze와 canonical 문서 digest를 다시 계산해 ASB-002와 대조했다.
- 변경한 내용: Goal 실행 기록과 final Status만 갱신; production/canonical 변경 없음
- 실행한 명령:

```bash
rg -n '^\| `(SP-01|SP-02|WV-01|WV-02|WV-03|WV-04|WV-05|WV-06|WV-07|SM-01|SM-02|SM-03|SM-04|BR-01|BR-02|BR-03|BR-04|BR-05|BL-01|BL-02|BL-03|BL-04|BL-05|PS-01|RF-01)`' core_enhanced/docs/goals/conditional_wave_and_boss_summon_contract_alignment_audit/EXPERIMENT_NOTES.md | wc -l
rg -n '[[:blank:]]+$' core_enhanced/docs/goals/conditional_wave_and_boss_summon_contract_alignment_audit/{PLAN.md,EXPERIMENTS.md,EXPERIMENT_NOTES.md}
git status --short -- core_enhanced/docs/goals/conditional_wave_and_boss_summon_contract_alignment_audit core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/src/game/battle/core core_enhanced/src/game/combat_setup core_enhanced/src/game/data game_resources/data/boss_runtime game_resources/data/summoned_units game_resources/data/pve/encounters.ron game_server/src game_server/docs
sha256sum core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/P2G_30_FINAL_POLICY_FREEZE_002.ko.md core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md
```

- 실제 실행된 test 수: N/A — coverage/source inspection only
- pass/fail/skip: N/A
- 실제 결과:
  - Matrix row count는 정확히 25다.
  - Goal 문서에 trailing whitespace는 없다.
  - Freeze digest `4c94407a…`와 rulebook/core-runtime/data-loading digest가 ASB-002와 동일하다.
  - Goal 문서 경로만 untracked audit artifact이며, canonical/runtime/live RON/server에는 이미 존재하던 dirty
    변경이 남아 있다. 이 Goal이 production/canonical 파일을 수정한 흔적은 없다.
- 실패 또는 관찰 불가 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 판단: 모든 audit 완료 조건에 current evidence가 연결됐으므로 Status를 `complete`로 변경한다.
