# Experiments

## 기록 규칙

실제로 수행한 runtime 조사 명령, probe와 test만 시간순으로 기록한다.

각 실행은 결과를 확인한 직후, 다음 코드 표면을 조사하기 전에 기록한다. 여러 실행을 마지막에 기억으로
복원하지 않는다.

각 항목은 가능한 범위에서 다음 형식을 사용한다.

```markdown
## E-<순번>. <제목>

- 시각:
- 기준선:
- 관련 criterion:
- 가설:
- 정확한 명령 또는 runner job:
- 실행 위치:
- 변경한 내용:
- 실제 실행 test 수:
- pass/fail/skip:
- 실제 결과:
- 실패 원인:
- 되돌리거나 수정한 내용:
- 판단:
- 연결된 finding 또는 gap:
```

Runner가 test 수나 pass/fail/skip 합계를 제공하지 않으면 추측하지 않고 그대로 기록한다.

감사 중 production 파일은 수정하지 않는다. 임시 probe가 필요하면 저장소 밖의 임시 위치를 우선하며,
사용한 입력과 현재 기준선을 재현 가능하게 기록한다.

## Baseline Test 반복 금지

최초 `cargo test --all-targets --all-features`가 all-pass이고 production code, live data, test, dependency와
실행 환경이 바뀌지 않았다면 같은 test suite를 다시 실행하지 않는다.

추가 test나 probe가 필요하면 실행 전에 `EXPERIMENT_NOTES.md`에 다음을 기록한다.

- 확인할 구체적인 감사 가설
- 기존 baseline으로 충분하지 않은 이유
- 중복되지 않는 새 증거

## 실행 기록

## E-001. Dirty runtime 기준선 식별

- 시각: `2026-08-21T23:03:28+09:00 KST`
- 기준선: `B-001`
- 관련 criterion: C00
- 가설: commit ID와 dirty-tree content identifier를 함께 기록하면 이후 증거의 기준선을 재현 가능하게
  식별하고 외부 변경을 탐지할 수 있다.
- 정확한 명령: `date '+%Y-%m-%dT%H:%M:%S%:z %Z'`, `git rev-parse HEAD`,
  `git status --short`, content manifest 계산 명령, `rustc -vV`,
  `cargo --version --verbose`, `uname -a`, Cargo/Rust 관련 환경 변수 조회
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: production/test/live data 변경 없음. 결과 기록용 Goal 문서만 추가·갱신.
- 실제 실행 test 수: 해당 없음
- pass/fail/skip: 해당 없음
- 실제 결과: HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`; 대상 210개 파일의 집계
  SHA-256은 `e30995cec4249a92c1d3981d4e7b5e64c153dac4b6e5df90d4f7d9fbb510fb61`;
  rustc/cargo 1.91.1; WSL2 Linux x86_64; Cargo/Rust 관련 환경 변수 출력 없음.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 판단: B-001은 commit과 dirty content를 함께 식별한다. 전체 status와 계산 계약은
  `BASELINE.md`에 보존했다.
- 연결된 finding 또는 gap: 없음

## E-002. 최초 전체 test baseline

- 시각: 2026-08-21 23:04~23:06 KST
- 기준선: `B-001`
- 관련 criterion: C00, C11
- 가설: 현재 dirty tree가 모든 target과 feature 조합에서 compile되고 기존 suite가 runner 수준에서
  all-pass하는지 확인한다.
- 정확한 명령: `cargo test --all-targets --all-features`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: production/test/live data 변경 없음. Cargo가 workspace `target` 아래 build artifact와 test가
  의도한 debug export를 생성할 수 있으나 감사 대상 source 기준선은 변경하지 않음.
- 실제 실행 binary와 test 수:

  | runner binary | tests | passed | failed | ignored | measured | filtered out |
  | --- | ---: | ---: | ---: | ---: | ---: | ---: |
  | `src/lib.rs` | 1080 | 1080 | 0 | 0 | 0 | 0 |
  | `src/bin/generate_encounter_compendium_contract_fixture.rs` | 0 | 0 | 0 | 0 | 0 | 0 |
  | `src/bin/generate_event_contract_fixture.rs` | 0 | 0 | 0 | 0 | 0 | 0 |
  | `src/bin/generate_inventory_contract_fixture.rs` | 0 | 0 | 0 | 0 | 0 | 0 |
  | `src/bin/generate_shop_contract_fixture.rs` | 0 | 0 | 0 | 0 | 0 | 0 |
  | `tests/live_item_skill_activation.rs` | 4 | 4 | 0 | 0 | 0 | 0 |
  | `tests/live_skill_catalog_audit.rs` | 4 | 4 | 0 | 0 | 0 | 0 |
  | `tests/ron_loading.rs` | 25 | 25 | 0 | 0 | 0 | 0 |
  | `tests/skill_runtime_contract.rs` | 14 | 14 | 0 | 0 | 0 | 0 |
  | `tests/skill_test_suite.rs` | 4 | 4 | 0 | 0 | 0 | 0 |
  | 안전하게 합산한 전체 | 1131 | 1131 | 0 | 0 | 0 | 0 |

- 실제 실행 test 수: runner가 binary별로 보고한 수를 합산한 1131개
- pass/fail/skip: 1131 pass / 0 fail / 0 ignored(skip)
- 실제 결과: exit code 0. `game_core` compile은 45.50초에 완료되었고 모든 runner summary가 `ok`였다.
  Cargo는 전체 binary 합계를 별도로 출력하지 않았으므로 위 전체는 각 runner의 명시적 summary를 더한 값이다.
- compile 또는 runner 오류: 없음
- 실패 원인: 해당 없음
- 되돌리거나 수정한 내용: 없음
- 판단: 최초 baseline은 all-pass다. 이는 C00의 기본 건전성 증거이며 C01~C13의 구조적 합격 증거가
  아니다. B-001 포함 표면과 실행 환경이 유지되는 동안 full suite와 단순 focused test를 반복하지 않는다.
- 연결된 finding 또는 gap: 현재 없음

## E-003. Crate root, feature와 direct dependency 사용 표면

- 시각: 2026-08-21 23:08 KST
- 기준선: `B-001`
- 관련 criterion: C00, C05, C09
- 가설: crate root와 Cargo manifest에서 production module/feature/dependency 진입 표면을 열거할 수 있다.
- 정확한 명령: `sed -n '1,260p' Cargo.toml`, `sed -n '1,220p' src/lib.rs`,
  `sed -n '1,260p' src/game/mod.rs`, 각 direct dependency namespace에 대한
  `rg -l '\b<dependency>::' src/lib.rs src/game | sort`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 실행 test 수: 해당 없음
- pass/fail/skip: 해당 없음
- 실제 결과: library root는 `pub mod game`; `game`은 26개 public top-level module을 노출한다. Feature는
  opt-in `event_log_checks` 하나이며 E-002의 `--all-features`로 compile/test되었다. Cargo의 8개 direct
  dependency(`rand`, `rapier2d`, `ron`, `serde`, `serde_json`, `serde_path_to_error`, `tracing`, `uuid`)는
  production namespace 사용이 확인되었다. `rapier2d`는 movement backend, `serde_path_to_error`는 live RON
  loader 한 곳에 국한되고 나머지는 여러 domain/data/projection 표면에 분포한다.
- 실패 원인: 없음
- 판단: crate/feature/dependency inventory의 첫 표면을 고정했다. dependency 책임의 적절성과 public
  module 변경 증폭 판정은 vertical path 조사 뒤 확정한다.
- 연결된 finding 또는 gap: 없음

### E-003 계수 정정

- 시각: 2026-08-21 23:09 KST
- 정정 명령: `rg -n '^pub mod ' src/game/mod.rs`
- 최초 요약의 `26개 public top-level module`은 수동 계수 오류다. 명령은 29개 선언을 출력했으며
  `SURFACE_INVENTORY.md`의 실제 열거가 29개와 일치한다.
- runtime 관찰이나 기준선은 바뀌지 않았다.

## E-004. Rustfmt 검사

- 시각: 2026-08-21 23:10 KST
- 기준선: `B-001`
- 관련 criterion: C00
- 정확한 명령: `cargo fmt --all -- --check`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음(`--check`)
- 실제 결과: exit code 0, stdout/stderr 출력 없음
- 판단: 현재 Cargo target의 Rust source는 rustfmt check를 통과한다. 구조적 합격 증거로 확대하지 않는다.
- 연결된 finding 또는 gap: 없음

## E-005. Clippy all-target/all-feature warning gate

- 시각: 2026-08-21 23:11 KST
- 기준선: `B-001`
- 관련 criterion: C00, C13(signal only)
- 정확한 명령: `cargo clippy --all-targets --all-features -- -D warnings`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: production/test/live data 없음; Cargo check artifact만 생성
- 실제 결과: exit code 101. Compiler는 library target에 `82 previous errors`, library test target에
  `133 previous errors`를 보고하고 compile을 중단했다. 따라서 뒤 target의 lint 완료 여부는 증명되지 않았다.
- 관측된 진단 범주: `unusual_byte_groupings`, `unnecessary_map_or`, `too_many_arguments`,
  `clone_on_copy`, `field_reassign_with_default`, `wrong_self_convention`, `useless_conversion`,
  `type_complexity`, `items_after_test_module`, `unnecessary_get_then_check`, `eq_op`,
  `bool_assert_comparison`, `useless_vec` 등. Production과 test 코드 모두 포함된다.
- 실패 원인: 현재 source가 clippy warning을 포함하며 `-D warnings`가 이를 error로 승격했다.
- 판단: lint gate는 통과하지 않는다. 이 수치나 lint 종류만으로 구조 finding을 만들지 않는다. 실제
  authority path/불변조건/변경 증폭과 연결되는 항목만 후속 C01~C13 조사에서 증거로 사용한다.
- 연결된 finding 또는 gap: 없음

## E-006. GameCore root entry와 state authority 표면

- 시각: 2026-08-21 23:13 KST
- 기준선: `B-001`
- 관련 criterion: C01, C02, C03, C05, C06, C09
- 가설: 외부 command는 `GameCore::execute*`에서 단일 private state owner를 거치며, public 우회 mutation이
  존재하는지 확인할 수 있다.
- 정확한 명령: world/state/snapshot/active-run의 line count와 type/method `rg`, `world.rs`의 root execute
  및 dispatch 범위 `sed`, `GameCoreState`/`ActiveBattleSession`/`RunState` 범위 `sed`,
  `set_enkephalin` definition/caller 및 enkephalin read/write `rg`, 관련 resource/shop/snapshot `sed`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과: `GameCore`는 private `state: GameCoreState`, `game_data: Arc<GameDataBase>`, `run_seed`를
  소유한다. 외부 behavior root는 battle authority validation → action gate → exhaustive domain dispatch → 일부
  transport projection 순서다. State module은 world 내부 private지만 그 안의 field는 sibling domain module이
  직접 mutate할 수 있도록 public이다. `GameCore::set_enkephalin`은 주석상 test helper이나 `#[cfg(test)]` 없이
  public이며, repository caller는 world test 5곳뿐이다. 이 method는 execute/capability/event/result를 거치지
  않고 authoritative currency를 덮어쓰며 snapshot과 shop eligibility가 즉시 그 값을 읽는다.
- 실패 원인: 없음
- 판단: root owner는 단일 `GameCore`로 캡슐화되지만 domain sibling에 광범위한 field mutation을 허용한다.
  Public test helper leakage는 RUNTIME-AUDIT-001 후보로 즉시 등록한다.
- 연결된 finding 또는 gap: RUNTIME-AUDIT-001

## E-007. Shop sale overflow candidate 반증과 shop transaction 순서

- 시각: 2026-08-21 23:17 KST
- 기준선: `B-001`
- 관련 criterion: C02, C03, C10
- 가설: sale commit의 unchecked addition에 도달 가능한 overflow가 있거나, 앞선 validation이 동일 상태와
  동일 quote의 overflow를 배제한다.
- 정확한 명령: `sale_evaluation`, `sell_item_quantity`, `execute_shop_action`, `total_quote`, `checked_add`
  definition/caller `rg`와 해당 범위 `sed`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과: sole production sale dispatch는 `sell_item_quantity`; 이 함수는 먼저 `sale_evaluation`을 호출한다.
  Evaluation은 nonzero/ownership/quantity/policy/tag/quote를 확인하고 `unit_quote.checked_mul(quantity)` 및 현재
  `enkephalin.amount.checked_add(total_quote)`까지 성공해야 같은 `total_quote`를 반환한다. 반환과 unchecked
  addition 사이에는 cloned inventory만 변경되고 authoritative currency mutation은 없다.
- 실패 원인: 없음
- 판단: 관측된 addition은 local unchecked 표현이지만 현재 call path에서 overflow 조건은 바로 앞의 동일
  value check로 배제된다. RUNTIME-AUDIT-002는 Rejected. 다만 validation과 commit 산술 중복은 compiler가
  결합을 보장하지 않는 C13 유지보수 signal로 inventory에 남긴다.
- 연결된 finding 또는 gap: RUNTIME-AUDIT-002 (Rejected)

## E-008. BattleCore authority와 event total-order 표면

- 시각: 2026-08-21 23:22 KST
- 기준선: `B-001`
- 관련 criterion: C01, C02, C04, C06, C09, C12
- 가설: BattleCore의 event heap은 same-time event를 완전한 stable key로 정렬하고 authority mutation을 typed
  API로 제한하거나, public/float 경계가 그 전제를 우회한다.
- 정확한 명령: battle core/type/sim/event/determinism line count 및 time/collection/identity search; `BattleCore`
  fields/constructors/public methods; `BattleEvent` 전체 enum/Eq/Ord; `WorldVec2`/runtime unit/scenario types;
  non-core field callers; finite-value validation; battle exports/UUID helpers를 `rg`와 `sed`로 조사
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과:

  - `BattleCore` owns a `BinaryHeap<BattleEvent>`, scenario/wave/summon/boss runtimes, unit/item/graveyard/buff/
    cast/projectile/area/movement maps, Rapier world, log/counters/seed/cause stacks.
  - event loop drains all events at the smallest timestamp, sorts each bucket by the custom total order, processes newly
    queued same-time buckets until exhaustion, then evaluates terminal outcome.
  - `BattleEvent::Ord` orders by time, semantic priority, variant discriminator, variant-specific identity, then allocates
    a full Debug string fallback. Manual `Eq` relies on a comment that float world positions are finite.
  - `WorldVec2` fields/constructor are public and do not validate finiteness. `battle::enums` and `battle::core` are public;
    public `enqueue_event` accepts arbitrary `BattleEvent` in production builds.
  - `BattleCore` and `RuntimeUnit` expose many authority fields as public mutable. Current non-test world callers read
    battle fields; active world battle has no external mutable getter, but external code can instantiate standalone core.
  - runtime-owned UUIDs use namespaced deterministic functions and `UuidManager` uses per-namespace `BTreeMap` counters.
- 실패 원인: 없음
- 판단: stable ordering design exists for valid finite events, but public authority/event injection and unencoded finite
  invariant require RUNTIME-AUDIT-003/004 follow-up. Debug-string comparator fallback is a C12 allocation signal.
- 연결된 finding 또는 gap: RUNTIME-AUDIT-003, RUNTIME-AUDIT-004

## E-009. Battle cursor와 playback/targeting/activation transaction ordering

- 시각: 2026-08-21 23:27 KST
- 기준선: `B-001`
- 관련 criterion: C03, C06, C10, C11
- 가설: 모든 mutating battle transport command는 future cursor를 mutation 전에 거부하거나 staged clone을
  commit하므로 `Err` 뒤 authority가 보존된다.
- 정확한 명령: transaction/rollback/revision/commit/handler/apply-live-command search; state revision/update DTO,
  battle advance staging, playback handlers, tactical staging, skill focus/activation 범위 `sed`; future-cursor test search
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과:

  - advance path clones active battle + VN/triggers/UUID/game state/content and rolls all listed roots back on `Err`;
    cursor validation occurs before step mutation.
  - deploy/withdraw/stance/relocate/swap clone `ActiveBattleSession`, build revision/update DTO on the staged value, replace
    live session only after success, and restore if VN attachment fails.
  - pause/resume/set-speed/begin-targeting/cancel-targeting/activate-skill mutate the live active session, commit revision,
    then call `battle_update_dto_after(after_seq, ...)`; that call can return `InvalidBattleResyncSeq`.
  - root `execute_with_transport_context` validates optional battle authority but not event cursor. Thus arbitrary public
    `battle_event_after_seq` reaches these handlers.
  - existing future-cursor atomicity tests cover advance and the five staged tactical commands. Search found no equivalent
    future-cursor test for playback/focus/activation commands.
- 실패 원인: 없음
- 판단: RUNTIME-AUDIT-005를 S1/Strong inference Active로 확정한다. Dynamic probe는 complete call/mutation/error
  order가 정적으로 결정되고 existing baseline에 해당 regression이 없으므로 반복 test 대신 생략한다.
- 연결된 finding 또는 gap: RUNTIME-AUDIT-005, GAP-001

## E-010. GameData construction과 StartNewGame failure ordering

- 시각: 2026-08-21 23:34 KST
- 기준선: `B-001`
- 관련 criterion: C03, C07, C09, C10, C11, C13
- 가설: GameCore only receives a validated published data type, so StartNewGame static prerequisites cannot fail after
  mutation; alternatively public construction can reach that error and/or panic.
- 정확한 명령: `GameDataBase`/`GameDataBuilder`/live loader/trigger validation definitions and ranges; StartNewGame,
  trigger evaluation, transition/map query; missing-trigger test/callers and duplicate-index validation search via `rg`/`sed`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과:

  - official `load_live_embedded` is fallible: parse sources → `ValidatedGameplayRon` → private
    `from_validated_live_catalogs` → generated-preview validation → `Result<Arc<GameDataBase>, LiveRonError>`.
  - `GameDataBuilder` and `GameDataBase::new(GameDataBaseParts)` are independently public production-build APIs.
    `new` calls index validators that panic on duplicates and `.expect(...)` for several reference/policy contracts.
  - Builder documentation calls `live_defaults` fixture-oriented, but the type and all `empty/with_*/build*` methods are
    not `cfg(test)` and `GameCore::new` accepts the same bare `Arc<GameDataBase>` from either route.
  - Builder can publish a bundle without a RunStart trigger; E-002 executed an existing test that confirms
    StartNewGame returns `InvalidStaticData`. The handler first initializes player, clears run/roster/session/battle/VN/
    checkpoint roots, and assigns `pending_game_mode`; no transaction restores them on that error. Test asserts only error.
- 실패 원인: 없음
- 판단: RUNTIME-AUDIT-006 and 007 become S2/Strong inference Active. Official live loader reduces current live-data
  reachability but compiler/type/visibility do not enforce exclusive use.
- 연결된 finding 또는 gap: RUNTIME-AUDIT-006, RUNTIME-AUDIT-007, GAP-002

## E-011. Core checkpoint/export/continue boundary

- 시각: 2026-08-21 23:37 KST
- 기준선: `B-001`
- 관련 criterion: C02, C03, C04, C08, C11
- 가설: durable payload is versioned/strict, validated completely before mutation, restores owned state and deterministic
  streams, and existing tests cover round-trip plus post-restore behavior.
- 정확한 명령: `active_run.rs`, `RunCheckpointPayload/State`, save/load/validator/restore paths and checkpoint test
  inventory/ranges via `rg`/`sed`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과:

  - core export envelope is strict JSON V4 (`deny_unknown_fields`) containing the last completed `RunCheckpointPayload`;
    loads-used and run-peak are explicit sideband fields in `ActiveRunCheckpointExport`.
  - continue rejects active-run/load-count/header/version/deserialize/graph errors before assigning checkpoint/run/player.
    Graph validation covers seed/UUID consistency, progression/map/template, power authority, recruitment/economy identities,
    encounter/boss/monolith references.
  - restore rebuilds `RunState`, clears temporary node/session/pending/battle recovery state, restores checkpoint-owned UUID
    streams while preserving VN-session namespace monotonicity, and explicit load is wrapped by whole non-battle transaction.
  - existing baseline tests cover strict field/version rejection, graph mutation rejection before continue/load mutation,
    serialize→continue→recapture semantic identity, load count/peak semantics, queue/shop/monolith state, and a checkpoint
    gate replay that compares next anchor/map after rewind.
  - no core filesystem write occurs; atomic durable file/database replacement belongs to the adjacent server consumer.
- 실패 원인: 없음
- 판단: core serialization/validation/restore is strongly evidenced, but full C08 requires the actual server durable write
  boundary and broader continuation behavior equivalence. Adjacent read scope is justified before access.
- 연결된 finding 또는 gap: GAP-003

## E-012. Adjacent active-run durable store와 direct transaction consumer

- 시각: 2026-08-21 23:39 KST
- 기준선: core `B-001`; adjacent snapshot `ADJ-PERSIST-001`
- 관련 criterion: C03, C08, C10, C11
- 가설: direct consumer atomically persists core bytes/sideband, detects corruption, preserves a previous good generation,
  serializes writers, and rolls core back on publication failure.
- 접근 근거: `EXPERIMENT_NOTES.md` 23:37 KST narrow-scope record
- 정확한 명령: direct `ActiveRunStore` caller `rg`; exact six-file status/SHA-256; full store and focused actor/profile/main/
  module ranges via `sed`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced` (read paths under `../game_server`)
- adjacent exact status: active store/module/actor handler/actor module/main are modified; `player_profile_store.rs` is untracked.
- adjacent file hashes:

  - `active_run_store.rs`: `f43c3fc5caf1eaeaf64207f37c240289548149b31292a83385c89b3eaa74cd3d`
  - `persistence/mod.rs`: `9b599603d0af7f0139d622bbab397293e56505211ab86652e6f5e19992236e31`
  - `player_profile_store.rs`: `d093656e63e4c9442a5729dcfc6638bdf40d145ddd8e13e42eb97e0dc9f464de`
  - `player_game_actor/handlers.rs`: `c8ac53202e412cf34d341a350c977827bcf3ab3731b994b5fed7823c33bae750`
  - `player_game_actor/mod.rs`: `b24dd94e20e4ac6cdcee168c2c4a891beb7d799a942a8d4784ff53b1a124b43f`
  - `main.rs`: `bc9cf2174e9cc189c2f8ef59698ec3d31838aba3057863f1c73627bf2021f661`

- 변경한 내용: 없음(read-only)
- 실제 결과:

  - store envelope is strict V4 with checksum over core bytes + loads/peak metadata; current and backup generations;
    temp write + file `sync_all` + platform replacement + directory sync.
  - exclusive file lock enforces one store writer; invalidation writes/syncs a tombstone before deleting generations;
    load rejects checksum/version/both-invalid and repairs current from a valid backup.
  - publish merges monotonic sideband maxima into current and backup. Tests in the same file cover real-file recovery,
    corruption/version/checksum, lock, failure-preserves-current, invalidation, and monotonic backup behavior, but were not
    executed by the core E-002 baseline.
  - actor captures core rollback and checkpoint-before, runs core, then publishes/invalidate when durable state changed;
    store failure restores non-battle core state. Standard completion uses a profile journal then invalidates active run and
    has startup recovery. Core execution `Err` returns before this persistence phase, relevant to RUNTIME-AUDIT-006.
- 실패 원인: 없음
- 판단: adjacent store design provides atomic/recoverable evidence. This is not a server-wide pass; unexecuted adjacent
  tests and GAP-003 keep C08 PARTIAL.
- 연결된 finding 또는 gap: GAP-003; RUNTIME-AUDIT-006 boundary confirmation

## E-013. Official live RON parse/validate/compile/publication code path

- 시각: 2026-08-21 23:42 KST
- 기준선: `B-001`
- 관련 criterion: C04, C07, C10, C11, C13
- 가설: official source set is closed/versioned/non-empty, parser reports source/path, every catalog and cross-reference is
  validated before immutable publication, and compiled data is built only inside that boundary.
- 정확한 명령: focused `live_ron.rs` source macro/types/validation/parser/publication/test ranges; validation/default/unknown
  field scans via `sed`/`rg`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과:

  - compile-time source set names 24 logical RON files plus strict schema manifest version 1; inventory rejects duplicate
    logical paths and empty source text.
  - parser uses `ron::Deserializer`, `serde_path_to_error`, and `.end()`; `LiveRonError` preserves source/domain/object/field/
    cause. Many raw domain shapes declare `deny_unknown_fields`; defaults are present for optional/legacy-tolerant semantics
    and are followed by domain validation.
  - `ParsedGameplayRon::into_validated` runs run-policy, catalog key/UUID/pool, domain contract, cross-reference, reward,
    encounter, RunStart, extra-wave, map/template/difficulty/payload/fixed-standard validation and compiles summoned/boss
    runtime before producing `ValidatedGameplayRon`.
  - official publication consumes the validated wrapper privately and then validates generated combat previews before
    returning `Arc<GameDataBase>`; no partial public database is returned on error.
  - E-002 baseline ran live publication parity, source/path/schema/unknown/legacy/malformed validation and live RON
    integration tests. Validation order is fixed by code but first error across collection entries follows authored Vec order.
- 실패 원인: 없음
- 판단: official live path is strongly bounded. C07 remains FAIL because public builder/new is a second unsafe publication
  path (RUNTIME-AUDIT-007), not because official path is incomplete. Actual embedded file inventory is narrowly justified next.
- 연결된 finding 또는 gap: RUNTIME-AUDIT-007

## E-014. Embedded live source file inventory

- 시각: 2026-08-21 23:43 KST
- 기준선: `B-001` (these files are within its aggregate)
- 관련 criterion: C00, C07
- 접근 근거: `EXPERIMENT_NOTES.md` 23:42 KST narrow resource record
- 정확한 명령: exact manifest + 24 source paths passed to `wc -c` and `sha256sum`; manifest first 40 lines only
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음(read-only)
- 실제 결과: all 25 files exist and are non-empty; total 408,400 bytes. Manifest is exactly schema version 1 in a
  27-byte RON document. Exact per-path size/hash is `LIVE_DATA_INVENTORY.md`.
- 실패 원인: 없음
- 판단: the bytes compiled/tested by E-002 and audited by E-013 are reproducibly identified without evaluating balance
  or content policy.
- 연결된 finding 또는 gap: 없음

## E-015. External behavior/result/error/DTO serialization seam

- 시각: 2026-08-21 23:47 KST
- 기준선: `B-001`
- 관련 criterion: C05, C06, C09, C10, C13
- 가설: external command와 result는 stable transport DTO로 분리되고, internal authority/domain changes do not
  automatically rewrite the wire contract.
- 정확한 명령: `behavior.rs` line counts; public behavior/result/error/DTO declaration scan; focused
  `PlayerBehavior`, `BehaviorResult`, `GameError` ranges via `wc`/`rg`/`sed`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과:

  - `PlayerBehavior` is a strict internally tagged, snake-case, `deny_unknown_fields` command enum. It directly embeds
    numerous domain identifiers and enums (`GameMode`, `MapNodeId`, `Position`, `FacingDirection`, `SkillCastTarget`).
  - `BehaviorResult` is directly `Serialize`/`Deserialize` but has no stable tag/rename/unknown-field policy on the enum.
    It mixes purpose-built DTOs with authoritative/internal domain shapes including `MapNodePayload`, `NodeSession`,
    `EmployeeDefinition`, `ActiveConsumableModifier`, `EmployeeCondition`, `ShopEventOption`, `RewardOption`, and recursive
    boxed `BehaviorResult` values.
  - `GameError` is also directly serialized as a Rust enum and separately exposes a stable `code()` mapping. Several
    variants contain `&'static str`, while others embed action/domain DTOs.
  - `behavior.rs` is 6,459 lines and contains command, transport projections, protocol error, result-contract
    introspection, and related tests in one module. Purpose-built battle DTOs are present and extensively derived, so the
    concern is an inconsistent boundary rather than absence of DTOs.
- 실패 원인: first broad derive scan was truncated; narrower declaration/range commands completed and are the evidence
  used here.
- 판단: C06 cannot pass. The protocol types are typed and the input command schema is strict, but result/error wire shape
  is coupled to Rust variant/field/domain representation. Whether the adjacent server serializes these types directly must
  be checked before classifying runtime reachability; static coupling is already established.
- 연결된 finding 또는 gap: candidate `RUNTIME-AUDIT-008`

## E-016. Adjacent direct behavior transport adapter와 central projection

- 시각: 2026-08-21 23:50 KST
- 기준선: core `B-001`; adjacent snapshot `ADJ-TRANSPORT-001`
- 관련 criterion: C05, C06, C09, C10, C13
- 접근 근거: `EXPERIMENT_NOTES.md` 23:48 KST narrow-scope record
- 정확한 명령: adjacent reference scan; exact three-file status/SHA-256; direct state/messages/session ranges; core
  `command_result_projection` implementation and exhaustive characterization-test ranges
- 실행 위치: `/mnt/f/work/simulator/core_enhanced` (read paths under `../game_server`)
- adjacent exact status/hashes:

  - `state.rs`: modified, `c42c486de1ad194ed9360892167311d47bc07aedb95a39eae31f0ff5464d2e07`
  - `messages.rs`: modified, `28229b6b6d32bd8f95b7cd456ec06407ea99e322903a9e996c7beccca550d7f7`
  - `session.rs`: clean, `bde6f11f1fd299bff57da5916b255a3d10fb48f453a08216e1c1fa4f1fd5cee1`

- 변경한 내용: 없음(read-only)
- 실제 결과:

  - server does not serialize the whole `BehaviorResult`/`GameError` enum directly. It calls core's central exhaustive
    `command_result_projection`, wraps explicit `result_type` + `Value`, routes battle setup/update separately, and maps
    errors through stable `GameError::code()` into an explicit error envelope.
  - `PlayerGameServerMessage` is tagged snake-case but contains no protocol version. Its payload and snapshot remain
    untyped `serde_json::Value`; live battle DTOs are inserted directly.
  - core's central projection has an exhaustive match and E-002 executed a 47-case characterization test that requires
    every `BehaviorResult` to retain the old direct-serde result type/payload. This strongly reduces accidental outer-shape
    drift and proves one conversion point.
  - the projection payload still serializes the same internal/domain structures noted in E-015. The characterization test
    explicitly derives its expected payload from direct `BehaviorResult` serde, so it freezes coupling rather than proving
    domain/wire separation.
- 실패 원인: 없음
- 판단: direct adapter evidence narrows and mitigates RUNTIME-AUDIT-008: outer result/error envelopes are centralized and
  characterized. The remaining confirmed structural issue is unversioned payloads directly reusing domain serde shapes,
  not raw serialization of the entire result/error enums.
- 연결된 finding 또는 gap: RUNTIME-AUDIT-008

## E-017. Required determinism probe, RNG, time, collection ordering

- 시각: 2026-08-21 23:52 KST
- 기준선: `B-001`; execution result is the single E-002 full-suite run
- 관련 criterion: C04, C08, C11
- 가설: identical initial state/seed/commands and insertion-order-equivalent battle input produce identical ordered
  results without post-hoc sorting away observable event order.
- 정확한 명령: focused source reads of RNG callers, facility A/B test, event heap 720-permutation test, movement reversed
  insertion test, observation-log identity test, playback delta math, staged battle advance
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과:

  - the mandatory `run(initial_state, seed, commands) -> A/B` probe is fulfilled by
    `same_seed_and_state_reproduce_template_assignment_and_ids`: two independent generations with the same live catalogs,
    policy, seed, mode, floor and state compare the complete generated values equal. E-002 executed it successfully.
  - the stronger insertion-order probe builds two BattleCores with reversed unit insertion order, executes two identical
    movement ticks, and compares ordered outputs, every position, and exact event-log JSON. E-002 passed it. The heap test
    also compares exact drain order across all 720 relevant insertion permutations.
  - production RNG uses seeded `StdRng`; recruitment sorts IDs before shuffle, shop sorts identity keys before weighted
    selection, and map generation sorts constrained slots/candidates. Other selections consume authored Vec order, which is
    part of the validated data input. No production wall clock, thread RNG, or `Uuid::new_v4` was found (`new_v4` hit is
    under `#[cfg(test)]`). The adjacent server converts a fixed tick interval, not wall elapsed duration, to the core tick.
  - playback uses integer ratios plus retained remainder; pause returns no delta, speed changes reset remainder, and battle
    advance stages authority and prevalidates cursor/revision. Checkpoint replay has representative deterministic gate
    evidence (E-011/GAP-003).
  - remaining negative evidence is the public non-finite/event injection boundary (RUNTIME-AUDIT-003/004), direct public
    mutable BattleCore internals, and lack of a separate-process/cross-platform Rapier probe.
- 실패 원인: one broad HashMap/RNG scan was truncated; exact RNG/time hits and focused readers completed.
- 판단: required same-process probes pass with observable order preserved. C04 remains FAIL/PARTIAL pending final wording
  because confirmed public NaN/authority injection violates the total-order precondition and physics portability is only
  represented by same-process tests.
- 연결된 finding 또는 gap: RUNTIME-AUDIT-003/004; GAP-003

## E-018. Battle event-log growth and per-tick delta cost

- 시각: 2026-08-21 23:54 KST
- 기준선: core `B-001`; adjacent files already identified by `ADJ-PERSIST-001`/`ADJ-TRANSPORT-001`
- 관련 criterion: C10, C11, C12, C13
- 가설: retained battle history and live delta extraction have explicit bounds or costs proportional to the requested
  delta rather than total battle history.
- 정확한 명령: `event_log.rs`, `recording.rs`, BattleCore execution loop, world update builder, bound/limit searches; one
  narrow adjacent reference search for direct tick/delta consumer
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과:

  - `BattleEventLog.entries` is append-only and `record_event_log` pushes without a battle event/count/byte cap. The
    event-log sequence uses unchecked `+= 1`; practical overflow is remote but no explicit failure contract exists.
  - `entries_after_seq` performs a full linear scan from entry zero and clones every matching entry. Every active live tick
    asks for a delta after the last delivered seq, so even a small tail pays O(total retained history); repeated ticks make
    cumulative scan cost quadratic in event growth. Result attachment serializes and gzip-compresses the complete log.
  - `run_battle_with_hooks` loops until terminal with no global event/time/work budget; incremental advance until a very
    distant target likewise has no general work budget. Boss microsteps have a local limit, which is a positive
    counterexample, but no generic same-time/event-bucket or whole-battle bound was found.
  - server uses a fixed configured tick interval and last delivered sequence, limiting ordinary request amplification but
    not the core full-history scan. No existing stress/complexity assertion was found in E-002.
- 실패 원인: the adjacent tick/delta `rg` was performed as a narrow extension before its scope-note wording was appended;
  no further server files were opened. This sequencing deviation is preserved in `EXPERIMENT_NOTES.md` rather than hidden.
- 판단: confirmed S2 growth/change-cost finding and test gap; normal live data may end battles quickly, but the runtime
  contract has no enforceable bound and the O(total history) per-tick scan is on the direct production path.
- 연결된 finding 또는 gap: RUNTIME-AUDIT-009; GAP-004

## E-019. Production error/panic/log/I/O boundary classification

- 시각: 2026-08-21 23:56 KST
- 기준선: `B-001`
- 관련 criterion: C00, C03, C07, C09, C10, C12
- 가설: recoverable external/data/runtime conditions return typed errors before authority mutation; panic/expect is
  restricted to compiler-proven or validated internal invariants, and non-authoritative I/O cannot fail gameplay.
- 정확한 명령: production-oriented `NotImplemented`/TODO and panic/expect/unwrap/unreachable/assert scans; focused reads
  of root routing, stat classification, wave resolution, map generation/progression, result projection, official loader,
  debug trace, checkpoint validation
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과:

  - no production construction of `GameError::NotImplemented` was found; hits are error formatting and tests. Two TODOs
    are tuning/logging notes, not declared unreachable feature branches.
  - the broad panic-family scan produced 2,848 line hits because many production source files contain inline test modules;
    that number is retained only as a noisy search result, not a production defect count. Focused runtime hits divide into:
    exhaustive-route/invariant `unreachable!`, validated-data `expect`/`panic`, and genuine overflow/invalid-public-boundary
    risks.
  - ActionRoute/root matches are exhaustive and E-002 tests characterize every action. Stat/facility weighted-selection
    unreachables follow exhaustive classification or checked nonzero arithmetic. Official server startup maps live loader
    failure to `InvalidData` rather than panicking; debug combat-trace filesystem failure is warning-only and happens before
    result transition without affecting authority.
  - validated-data assumptions still panic on the public alternate builder/new path (RUNTIME-AUDIT-007), and six live
    commands return a typed projection error after mutation (RUNTIME-AUDIT-005).
  - Endless `advance_floor` uses `checked_add(...).expect` while checkpoint validation accepts every Endless `u32` floor.
    This is a real theoretical terminal boundary at 2^32 floors, but practical reachability is too remote to create a
    separate severity finding; it remains a C10/C12 residual risk.
  - nested `BehaviorResult` projection panics if a public caller constructs a recursive completion containing a battle
    update. Current exhaustive GameCore result writers do not create that shape; it is another public-type-state signal,
    not a confirmed active command path.
- 실패 원인: broad scan could not syntactically exclude inline test modules. Focused call-path classification, rather than
  the raw count, is the audit evidence.
- 판단: C10 fails on RUNTIME-AUDIT-005/007. Current official data startup and debug I/O are positive counterexamples; no
  new production feature hole was found.
- 연결된 finding 또는 gap: RUNTIME-AUDIT-005/007; RUNTIME-AUDIT-006 related transaction risk

## E-020. Feature reachability and dependency/change-amplification audit

- 시각: 2026-08-21 23:58 KST
- 기준선: `B-001`
- 관련 criterion: C05, C06, C09, C11, C13
- 가설: every declared player/admin action has one reachable owner and projection; leaf battle/data logic does not depend
  backwards on transport/world orchestration, and enum changes are compiler-enforced at synchronization points.
- 정확한 명령: `ActionKind`/descriptor/`PlayerBehavior::kind`/root-domain dispatch and admin command matches; behavior
  characterization tests; cross-layer import lists; top source line counts; lint suppression inventory
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과:

  - the macro-defined player action set has one exhaustive descriptor, one exhaustive behavior-to-kind mapping and one
    exhaustive root dispatch. Four recovery actions are deliberately host-owned; all remaining actions reach a concrete
    core handler. AdminCommand's 20 variants also have an exhaustive direct handler match. No production `NotImplemented`
    return path was found.
  - E-002 executed the action route/scope/authority characterization and the exhaustive 47-result transport
    characterization. Adding an enum variant forces compiler errors in the central exhaustive matches; leaf domain
    matches use wildcard `unreachable!`, but root routing remains the compiler-enforced owner.
  - battle leaf code does not import `world`, but 15 battle files import transport-owned `behavior::GameError` and several
    also import `ActionKind`/`ActionDisabledReasonCode`. Data compilation/runtime types directly reuse battle types in
    production. This is not a module cycle, but error/capability ownership points from leaf domain back toward the large
    behavior/protocol module.
  - `behavior.rs` is 6,459 lines and simultaneously owns actions, capability policy, many snapshots/DTOs, results, errors,
    projection contracts and tests. Other large change centers include `battle/core/mod.rs` 8,608, `data/live_ron.rs`
    4,803, `boss_runtime_data.rs` 4,240, `battle/core/boss_runtime.rs` 3,561, `world/combat.rs` 3,325 and
    `battle/core/sim.rs` 3,309 lines. Size is a signal; concrete amplification is RUNTIME-AUDIT-008 plus server error/result
    mapping and battle imports.
  - warning suppressions are local (four `too_many_arguments`, several `dead_code`), not crate-wide. Clippy still fails
    with the E-005 error set, so there is no broad suppression hiding that gate.
- 실패 원인: none; file sizes include inline tests where present and are not used as defect counts.
- 판단: C09 PASS for declared current entrypoint reachability. C05/C13 fail or partial because protocol/domain error
  ownership and giant mixed-responsibility modules amplify changes, despite strong compiler-enforced enum routing.
- 연결된 finding 또는 gap: RUNTIME-AUDIT-008/009; root-cause consolidation pending

## E-021. Focused BattleEventLog recent-tail scaling probe

- 시각: 2026-08-22 00:01 KST
- 기준선: `B-001`; exact rlib `libgame_core-6b908d11a144a5a7.rlib`, fingerprint features
  `["event_log_checks"]`, no rustflags
- 관련 criterion: C11, C12
- 사전 기록: `EXPERIMENT_NOTES.md` 23:59 KST
- 정확한 명령:
  `rustc --edition=2021 -O /tmp/runtime_audit_event_log_probe.rs -L dependency=../target/debug/deps --extern
  game_core=../target/debug/deps/libgame_core-6b908d11a144a5a7.rlib -o /tmp/runtime_audit_event_log_probe &&
  /tmp/runtime_audit_event_log_probe`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`; source/binary under `/tmp`
- fixture: public `BattleEventLog`, monotonically sequenced `BattleStart` entries, constant two-entry requested tail;
  20M total entries scanned at each size (1K×20K calls, 10K×2K, 100K×200). One full extraction per size.
- 변경한 내용: `/tmp/runtime_audit_event_log_probe.rs` and binary only; no production/test/live/adjacent file change
- 실제 결과:

| retained entries | tail repetitions | returned/call | tail ns/call | one full clone ns | estimated inline cloned bytes |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 20,000 | 2 | 4,987.6 | 191,123 | 240,000 |
| 10,000 | 2,000 | 2 | 47,869.6 | 1,517,461 | 2,400,000 |
| 100,000 | 200 | 2 | 489,661.7 | 13,761,982 | 24,000,000 |

  `BattleEventLogEntry` inline size in this build is 240 bytes. A 10× history increase produced approximately 9.6× and
  10.2× recent-tail call time while returned delta stayed exactly two entries. Full-clone inline bytes scale exactly with
  N and exclude heap allocations owned by richer event variants.
- 실패 원인: 없음
- 판단: bounded measurement corroborates the static O(total retained history) per-call finding. It is not a universal
  latency benchmark (single WSL2 process, optimized probe linked to the E-002 debug rlib, synthetic small event payload),
  and no threshold claim is made.
- 연결된 finding 또는 gap: RUNTIME-AUDIT-009; GAP-004

## E-022. Final baseline and adjacent-snapshot contamination check

- 시각: 2026-08-22 00:04 KST
- 기준선: `B-001`, `ADJ-PERSIST-001`, `ADJ-TRANSPORT-001`
- 관련 criterion: C00 and audit completion integrity
- 정확한 명령: `git rev-parse HEAD`; exact B-001 find/sort/sha256 aggregate and file count; focused status and SHA-256
  for all nine adjacent files opened during E-012/E-016/E-018
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 변경한 내용: 없음
- 실제 결과:

  - HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
  - B-001 manifest remains exactly 210 files and aggregate
    `e30995cec4249a92c1d3981d4e7b5e64c153dac4b6e5df90d4f7d9fbb510fb61`.
  - all six ADJ-PERSIST-001 hashes and all three ADJ-TRANSPORT-001 hashes exactly match their first-read records.
  - focused status still shows the same inspected adjacent files modified/untracked and the Goal directory untracked; no
    audit production/resource/test/server edit was introduced.
- 실패 원인: 없음
- 판단: no external baseline change or inspected-adjacent drift occurred during the audit. Completion may proceed without
  restarting B-001 or asking the user to choose a new baseline.
- 연결된 finding 또는 gap: 없음
