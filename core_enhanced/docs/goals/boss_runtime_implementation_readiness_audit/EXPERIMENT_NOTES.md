# Boss Runtime Implementation Readiness Audit — Notes

Goal: `boss_runtime_implementation_readiness_audit`
Initialized: 2026-08-08

## Structural decision

- 이 Goal은 Boss Runtime을 구현하는 Goal이 아니라 구현 준비도와 정책 완전성을 판정하는
  `investigation_only` Goal이다.
- Core, live RON, game_server와 Unity는 별도 owner를 가지지만 이번 결과는 하나의 authoring→publication→
  setup→runtime→event/checkpoint→replay 계약 사슬의 readiness 판정이다.
- Production mutation과 migration이 없으므로 단일 Goal로 시작한다.
- 조사에서 독립 구현 경계를 여러 Phase로 나눠야만 안전한 결과를 낼 수 있음이 확인되면 자동으로
  Master를 만들지 않고 `scope_decision_required`로 전환한다.

## Document-grade separation

- `docs/tmp/boss_behavior_state_machine_policy_notes_2026-08-07.ko.md`의 `BP-01`~`BP-34`는 승인된 목표
  정책을 소유하는 temporary decision record다.
- `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`, `skill_target_contract.md`는 current
  Core 계약의 durable canonical 후보로 실제 runtime과 대조한다.
- Unity 문서, code와 test는 current Core/game_server보다 뒤처진 stale consumer baseline이다. 이를 current
  producer canonical로 취급하지 않고 producer-consumer drift와 future synchronization 범위를 확인하는
  evidence로만 사용한다.
- Current runtime/live data가 current canonical과 다르거나 target policy가 아직 미구현인 상태를 서로
  섞지 않는다.
- Goal/audit 문서는 current gameplay canonical을 대체하지 않는다.

## Initial repository baseline

### Simulator repository

- Root: `/mnt/f/work/simulator`
- Core path: `/mnt/f/work/simulator/core_enhanced`
- game_server path: `/mnt/f/work/simulator/game_server`
- Core와 game_server는 같은 Git root를 사용한다.
- Start commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- 신규 Goal 경로 생성 전 `git status --short --untracked-files=all` entry count: `211`
- 신규 Goal 경로 생성 전 status SHA-256:
  `7358c0145d11bbc1799400d20429174996a1a20ad12461e19ce761a983565cfe`
- Status에는 Core source/test/canonical, live data, game_server source와 다수의 기존 Goal/audit 문서 변경이
  포함된다. 모두 사용자 소유로 간주한다.

Completion audit에서는 이 Goal 디렉터리를 pathspec에서 제외한 production-scoped status를 같은 방식으로
비교한다. 기존 211개 entry가 반드시 불변이라는 뜻은 아니다. 사용자가 병행 변경할 수 있으므로 차이가
있으면 파일별 책임과 시점을 대조하고 이 Goal이 만든 mutation인지 판정한다.

### Unity repository

- Root: `/mnt/f/unity projects/ark`
- Start commit: `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`
- Full `git status --short --untracked-files=all` entry count: `2059`
- Full status SHA-256:
  `55d940dc476ee297b267ed15afffd40570d448289cdccb0f172957bf254c1dcf`
- Initial Boss-audit supporting scope (`docs`, `Assets/Scripts/ArkClient`, `Assets/Tests`) entry count: `306`
- Supporting-scope status SHA-256:
  `50ab153889caa2437bed4d0518dd67ac50c5b190dde08b7460ff6fcbedaaff14`
- Unity worktree는 대규모 dirty/untracked asset과 사용자 작업을 포함한다. 이 Goal은 실제 battle contract
  consumer와 test를 읽기만 하며 asset import, fixture regeneration, Unity 실행으로 파일을 생성하지 않는다.

## Dirty-worktree precautions

- `git restore`, `git reset`, `git checkout`, `git clean`, stash, formatter와 cleanup을 수행하지 않는다.
- Existing modified/untracked file을 과거 상태로 되돌리거나 비교 결과만으로 stale라고 삭제하지 않는다.
- Current working tree의 실제 runtime을 우선 evidence로 읽되 start commit과 기존 audit는 supporting
  comparison으로만 사용한다.
- Goal 문서 외 write가 필요해지는 순간 investigation scope를 벗어난 것이므로 중단하고 별도 구현 Goal을
  제안한다.
- Unity의 자동 `.meta`, `Library`, `TestResults` 또는 fixture 변경을 유발할 수 있는 명령은 completion에
  필수라는 근거와 사용자 승인이 없으면 실행하지 않는다.

## Existing refactor audit relationship

- 2026-08-07 refactor Master는 51개 final candidate를 만들었지만 production 구현을 승인하거나 수행하지
  않았다.
- Boss audit는 그 catalog를 다시 점수화하거나 전체 조사하지 않는다.
- 특히 `CRA-P06-C01/C02/C03`과 `CRA-P07-C02`는 Boss same-time transition과 atomic commit의 잠재
  prerequisite로 재확인한다.
- `CRA-P07-C03/C04` movement cleanup은 새 Boss intent seam을 제거할 위험이 있어 선행 여부를 가정하지
  않는다.
- `P01/P05` publication/setup, `P08/P09` effect/target/delivery와 `P10` DTO/server seam은 필요한 부분만
  actual code evidence로 교차한다.
- Existing candidate와 새 Boss foundation이 같은 primitive를 공유해도 transaction lifetime과 owner가
  다르면 하나의 거대 refactor로 병합하지 않는다.

## Consumer boundary

- Core and game_server source는 simulator repository에서 직접 확인한다.
- Stale Unity repository와 consumer 문서에 접근 가능하므로 Boss presentation/replay 완료 조건에서 Unity
  consumer drift 감사를 비범위로 두지 않는다.
- Unity docs만 읽고 consumer를 검증했다고 표현하지 않는다. 실제 parser/store/presentation readiness와
  cancellation code 및 관련 test/fixture를 읽는다.
- 별도 배포된 미확인 client나 public external Rust consumer가 발견되면 `unverified_external`로 기록하고
  completion 필수 여부를 판정한다.

## Policy-gap handling

- 초기화 시점에 사용자 결정이 필요한 추가 정책은 없다.
- 조사 중 conflict를 발견해도 관련 없는 audit stream을 중단하지 않는다.
- Material gameplay/RON/wire/save/compatibility 선택은 current runtime, current canonical, target policy,
  선택지와 권장안을 기록한 뒤 Status를 `policy_decision_required`로 둔다.
- 예상 가능한 기술 세부는 target policy와 current contract를 바꾸지 않는 한 감사자가 disposition할 수
  있지만 implementation recommendation과 승인된 policy를 구분한다.

## Test-evidence policy

- Test 이름, 통과 수나 mock 간 일치만으로 Boss readiness를 확정하지 않는다.
- 각 관련 test의 setup/action/assertion, production reachability, exact invariant와 private-shape 결합을 읽는다.
- Existing test는 `retain`, `strengthen`, `merge`, `remove`, `unverified` 중 하나로 disposition한다.
- 새 target behavior가 아직 구현되지 않았으므로 target-policy test가 green일 것을 completion 조건으로
  두지 않는다. 후속 implementation Goal의 test-first 실패/성공 조건을 구체적으로 작성한다.
- Static evidence gap 때문에 runner가 필요할 때만 좁은 command를 실행하고 actual count를 기록한다.

## Alternatives not adopted

### Full refactor implementation before audit

채택하지 않는다. 51개 후보 중 Boss와 무관한 경계가 많고 movement cleanup이 실제 extension seam을
선행 제거할 수 있다.

### Boss implementation mixed into this Goal

채택하지 않는다. 조사 결과와 production mutation evidence가 섞이고 필요한 정책 또는 prerequisite를
발견하기 전에 구조를 고정할 위험이 있다.

### Re-audit all 51 candidates

채택하지 않는다. 기존 evidence를 supporting input으로 사용하고 Boss target authority와 교차하는 후보만
current tree에서 확인한다.

### Unity contract documents only

채택하지 않는다. `BP-30`은 replay parser, readiness, cancellation과 checkpoint reconcile의 실제 consumer
behavior를 요구한다.

### Final-boss features as v1 requirements

채택하지 않는다. 세 최종보스는 kernel 표현력 stress test이며 첫 vertical slice에 placeholder schema,
Task나 mechanic을 추가할 근거가 아니다.

## Initially unverified

- Current live source bundle에 새 Boss Runtime catalog를 결합할 정확한 ownership seam
- Opt-in actor binding을 preview/setup/scenario 중 어느 resolved type이 소유해야 하는지
- Existing movement planner와 block/basic target에서 Boss target/movement intent를 주입할 최소 seam
- Battle event queue에서 Boss microstep과 activation epoch를 표현할 reusable primitive
- Core-only active battle checkpoint의 실제 capture/restore surface와 target runtime state 수용 방식
- 새 Boss presentation lifecycle event를 current event enum/DTO에 넣을 때 server/Unity parser 증폭 범위
- Unity duration readiness/cancellation의 실제 current code/test coverage
- Final-boss stress requirements 중 v1 kernel이 extension point만으로 수용 가능한 범위

이 목록은 결함 주장이나 구현 요구가 아니다. `BOUNDARY_AUDIT.md`와 `POLICY_COVERAGE_MATRIX.md`에서
evidence로 disposition한다.

## Range-out follow-up policy

독립적인 bug, cleanup, content issue나 tool/editor requirement를 발견해도 자동 구현하거나 새 Goal을 만들지
않는다. 현재 Boss readiness에 영향을 주면 dependency/gate로 기록하고, 무관하면 이 문서의 후속 후보로
짧게 남긴다.

## Durable incremental evidence checkpoint

- 사용자 우려는 장시간 감사 뒤 대화가 compact되면서 source observation, reasoning과 disposition이 유실되는
  것이다. 따라서 대화와 model context는 감사 evidence의 저장소로 사용하지 않는다.
- 한 source/consumer path, policy 행, refactor candidate 또는 test family를 읽은 bounded investigation unit이
  끝날 때마다 다음 unit 전에 finding, 정확한 source path/symbol, current/target 구분, disposition과 unresolved
  gap을 관련 Goal artifact에 누적한다.
- 미확정 관찰은 이 문서에 즉시 `unresolved`로 남긴 뒤 확정 artifact에 연결한다. 명령/probe는 같은 시점에
  `EXPERIMENTS.md`에 기록한다.
- Compact 또는 세션 중단 뒤에는 Goal artifact의 마지막 durable checkpoint부터 재개한다. 감사 마지막에
  대화 기억을 바탕으로 finding을 한꺼번에 작성하는 방식은 허용하지 않는다.
- 이 규칙과 Unity stale baseline은 `PLAN.md`의 `BRA-I13`, `BRA-I14`, durable protocol 및 `BRA-C14` 완료
  gate로 고정됐다.

## Full rulebook durable checkpoint

- Current `game_rulebook.md` 1~961줄을 세 연속 구간으로 EOF까지 다시 읽었다.
- Boss audit과 직접 교차하는 current gameplay authority는 DefenseRoute 공통 전장/transport, ordinary authored
  route와 sticky block, Core-owned target/action validity, Hard CC의 unreleased cancel/released delivery 보존,
  Pause의 battle-time 정지, Core event/checkpoint authority다.
- Target Boss control graph는 rulebook의 current behavior가 아니다. 후속 구현 시 승인된 target policy와 함께
  durable canonical을 갱신해야 한다.
- Rulebook의 외부 Unity `canonical` 링크는 stale Unity baseline의 현재 신선도를 보장하지 않는다. 이를
  external consumer documentation drift로 기록하며 producer-side 계약 누락과 분리한다.

## Adjacent canonical durable checkpoint

- `core_runtime_contract.md` 775줄, `data_loading_contract.md` 305줄,
  `skill_target_contract.md` 373줄의 current content를 읽었다.
- Reusable current contract: battle-time event pipeline, append-only seq, checkpoint reconcile, Hard CC/stale
  action epoch, detached delivery, strict embedded source parsing/cross-validation, deterministic basic/skill
  targeting과 captured target lifecycle.
- Target Boss gap: activation-owned target lock, typed multi-key/path-cost query, monotonic route rejoin,
  Phase/Pattern/State/Task activation checkpoint, transition atomic plan과 Boss catalog/binding.
- `unresolved`: runtime canonical의 실패 no-mutation 선언이 실제 world advance의 projection/trigger failure
  경로에서도 성립하는지 source와 test assertion으로 재확인해야 한다.
- `unresolved`: BP-28의 versioned atomic bundle과 failed update의 last-known-good 보존을 current embedded
  startup-only loader에 어떻게 적용할지는 hot reload를 이번 v1에 도입할지 여부와 분리해 readiness report에
  정확히 disposition해야 한다.

## Live publication and opt-in durable checkpoint

- Actual source inventory는 manifest 외 22개 gameplay source다. `data_loading_contract.md`의 “manifest 외
  21개” 문구는 current source보다 하나 뒤처진 documentation drift다.
- Strict parse/cross-validation/candidate DB/preview validation 뒤 `Arc<GameDataBase>`를 반환하는 startup
  fail-closed seam은 Boss catalog를 같은 atomic candidate에 결합할 기반으로 충분하다.
- Current path는 embedded startup-only이므로 failed dynamic update 뒤 last-known-good generation 유지가
  필요한 hot reload owner는 없다. BP-28의 partial-generation 금지는 지켜야 하지만 hot reload 자체를 Boss
  v1 선행 요구로 확대하지 않는다.
- `PveEncounter`에는 `boss_runtime_id`/actor binding이 전혀 없고 Boss class도 runtime admission이 아니다.
  BP-01에 따라 후속 schema는 optional valid configuration reference로 opt-in해야 한다.
- Preview gate는 모든 encounter의 baseline floor 0/five seeds warning만 확인한다. Live Boss publication 전에는
  실제 manifested difficulty와 typed setup/scenario binding feasibility를 별도 validation으로 닫아야 한다.

## Preview-to-runtime binding durable checkpoint

- Actual chain: `CombatPreview` → `BattleStartPlan` + player start →
  `CombatExecutor::build_battle_scenario_from_preview` → `ScenarioSpawnGroup`/timed `ScenarioEvent` →
  `BattleCore::start_battle_execution_with_setup` → `spawn_scenario_group` → `RuntimeUnit`.
- Authored route authority is `SpawnWave.route_id` resolved to preview route cells, stored once per opponent
  `ScenarioSpawnGroup`, then cloned into each spawned `RuntimeUnit.enemy_movement_plan`.
- Missing seam: `PveEncounter` has no Boss reference, `BattleScenario` no encounter runtime binding,
  `ScenarioUnitSpawn` no actor behavior binding/immutable parameter frame, `RuntimeUnit` no Boss actor runtime.
- Existing setup tests execute the real builder and assert exact route-cell transfer. Retain them and add explicit
  non-opt-in parity plus invalid/missing Boss binding failure tests.
- Recommendation: resolve definitions before battle start into immutable typed setup. Do not make BattleCore leaf
  execution repeatedly query mutable catalog IDs during ticks.

## Movement/target authority durable checkpoint

- Current opponent planner has no Boss intent seam. Missing per-unit plan falls back to scenario authored route;
  this exact fallback is the non-opt-in preservation branch.
- `route_progress_along_centers` derives nearest progress from the current position and only prefers forward progress
  on equal distance. There is no monotonic cursor, so it cannot prove BP-02 forward-only rejoin after off-route action.
- Continuous movement stop causes (`TargetReached`, lock, no goal, static obstacle) are internal engine outputs. The
  apply path discards cause except for stopping presentation segment, so BP-25 closed Task results need an adapter and
  activation ownership.
- Existing `RuntimeUnit.current_target` belongs to basic/block behavior and can be reselected by range/usefulness.
  It must not double as BP-18 Pattern lock.
- Reusable primitives: deterministic unit iteration, stable IDs/spawn order, tile range/usefulness, sticky block,
  continuous movement/collision and current event projection.
- New foundation: typed target filters/order keys, reachability/path cost, activation lock, monotonic route cursor,
  Boss movement intent and exhaustive result delivery.

## Same-time scheduler and terminal durable checkpoint

- `BattleEvent::Ord`는 timestamp/priority가 같은 뒤 일부 variant 조합을 `_ => Equal`로 접는다. Priority 8의
  `AutoCastEnd`/`ManualCastEnd` 교차 kind와 별도 manual-end event 등이 distinct해도 equal일 수 있어 BP-16의
  total order foundation으로는 불충분하다.
- Current drain loop는 처리 중 생성된 same-time event를 다시 drain하므로 microstep 실행 형태는 이미 있다.
  하지만 timestamp별 step counter/limit과 zero-time livelock failure가 없다.
- `step_battle_execution`은 next event가 max time을 넘으면 Draw/Finished로 finalize하지만
  `step_battle_execution_until`은 max time까지 전진한 상태를 Running으로 반환할 수 있다. Boss terminal/phase
  arbitration 전에 한 helper와 한 의미로 통일해야 한다.
- `finalized` guard와 same-time attack-resolve-before-end regression은 재사용/유지 대상이다. Test coverage는
  모든 event variant total-order, insertion permutation, microstep limit과 두 API max-time parity로 강화한다.
- Disposition: total order, bounded microsteps, terminal parity는 모두 `prerequisite_before_boss`; Boss graph
  owner나 새 public event를 이 수정에 끼워 넣지 않는다.

## World live-advance atomicity durable checkpoint

- `advance_active_battle_by_internal`/next-event path는 BattleCore step/finalize로 active execution을 먼저 바꾼
  뒤 VN trigger, tactical resource/live signal, employee/deployment reconcile, projection revision/DTO, VN capability,
  finished state transition을 여러 fallible 단계로 수행한다. 전체 staged transaction이나 rollback은 없다.
- 가장 직접적인 counterexample은 caller `after_seq` validation이다. Future cursor 검사가 step과 revision commit
  뒤 `battle_update_dto_after`에서 일어나므로 `Err(InvalidBattleResyncSeq)`가 이미 변경된 execution/event/resource/
  trigger/revision과 함께 반환될 수 있다. `game_server` tick이 이 after-seq API를 실제 호출한다.
- Existing future-seq test는 `RequestBattleState` read path만, overflow no-mutation test는 playback command의
  preflight만 보호한다. Advance error atomicity를 증명하지 않는다.
- Earlier `unresolved`였던 runtime canonical failure no-mutation code 준수 여부는 `not_satisfied`로 확정한다.
  Boss의 stage→validate→commit을 world integration에 얹기 전 `prerequisite_before_boss`로 해결해야 한다.
- Required acceptance: 실패를 주입한 뒤 execution time/queue/unit, event log seq, live signal/resource,
  employee/deployment/trigger, revision과 top-level game state가 모두 byte-for-byte-equivalent 또는 domain-equivalent
  unchanged인지 검사한다.

## Active runtime/checkpoint durable checkpoint

- Authoritative active battle is process-local `ActiveBattleSession { battle: BattleCore, execution, ... }`.
  `RunCheckpointPayload`와 pre-battle `BattleEntrySnapshot`은 active battle을 저장하지 않으며 active-session type은
  Clone/Serialize transaction surface가 아니다.
- Public `LiveBattleStateCheckpointDto`는 same in-memory session에서 on-demand 생성되는 reconnect/reconcile
  projection이다. Core execution을 이 DTO로 복원하지 않는다.
- Current public state: seq/time, playback/focus, field-work, active unit identity/HUD/control/tactical/range/mobility/
  position/stats, tactical resource/deployment. Missing for the target: authoritative facing and any continuing public
  Boss phase/objective/duration-presentation lifecycle.
- Internal Phase/Pattern/State/Task, target lock, task result queue, epoch와 route cursor는 Core-only state로 두고
  public checkpoint에 노출하지 않는다. Reconnect로 gameplay graph를 재계산하지 않는다.
- Current scope에서는 active battle process-restart resume가 없으므로 save migration은 `not_applicable`다.
  Public lifecycle checkpoint additions and server/Unity sync are still required at the first Wire-changing slice.

## game_server transport durable checkpoint

- Server는 Core gameplay를 재계산하지 않고 setup/update/resync ordering, active socket routing과 last-sent seq cursor를
  소유한다. Start setup→update, tick update, final update→result snapshot의 existing ordering tests는 retain한다.
- `BattleResync`는 typed `LiveBattleUpdateDto`를 중첩해 그대로 보존한다. Normal `BattleUpdate`와
  `BattleSetupSnapshot`은 Core top-level fields를 server enum에 수동 열거한다. Nested event delta/checkpoint/
  catalog refs는 whole typed values로 전달된다.
- Consequence: 새 nested event/checkpoint field는 compile/serde 경로에 자연스럽게 들어가지만 새 top-level
  update/setup field는 server mapper가 조용히 누락할 수 있다. Existing tests는 selected-field assertions이지
  Core DTO와 envelope의 exact normalized JSON parity가 아니다.
- Disposition: 최초 public Boss Wire 변경과 함께 server mapper/normal-update/resync/setup parity를 고친다.
  Gameplay graph/condition/order를 server로 옮기지 않는다.
- Actor cursor는 socket mailbox enqueue 시 `to_seq`로 진행하며 다음 live tick의 `after_seq`가 된다. Core
  advance의 future-cursor atomicity flaw와 실제로 연결되는 live path다.

## Stale Unity consumer durable checkpoint

- Stale baseline은 raw setup/update/resync를 받아 revision/seq/checkpoint relation을 검증하고 buffered battle-time
  presentation seq 뒤 checkpoint를 reconcile한다. Representative EditMode tests가 revision, gap/resync, movement/
  projectile timing과 checkpoint delay를 보호한다.
- Actual battle renderer support is `UnitSpawned` identity + movement segment/stop interpolation + `HpChanged` damage
  feedback. Attack/skill/projectile/area/death Spine playback은 event stream과 연결되어 있지 않다. Projectile/area
  names in readiness logic are delay markers, not visual consumers.
- Unknown/new event는 parse 실패하지 않지만 즉시 presented로 취급된다. 그러므로 Boss duration event를 Core에
  추가하는 것만으로는 timing/cancel/interrupt/idempotence가 구현되지 않는다. Stable presentation instance와
  start/end/cancel owner 및 checkpoint active-lifecycle projection이 필요하다.
- Same battle UUID의 standalone setup은 `BattleRuntimeState.Reset()`을 호출하지 않는다. 기존 local queue,
  readiness markers/checkpoint를 버리고 rebuild한다는 stale contract와 code가 불일치한다. Same-revision ordinary
  update도 무시되므로 explicit setup-loss reset test/implementation이 필요하다.
- Reconciled checkpoint에서 빠진 actor/HUD는 즉시 destroy된다. Death event readiness/animation owner가 없어
  death cue 완료 전에 제거되지 않는다는 보장이 없다.
- Unity는 checkpoint `facing`을 optional로 읽고 누락 시 right로 default한다. Current Core checkpoint에는 facing이
  없어 direction reconcile은 실제로 authoritative하지 않다.
- Disposition: current event만 쓰는 internal tracking slice에는 downstream blocker가 아니다. 첫 public lifecycle
  또는 Boss-specific presentation slice에서는 Core/server와 함께 consumer sync가 필수다.

## Existing refactor correlation durable checkpoint

- Named 19 candidates were read from their Phase catalogs rather than inferred from the final summary table.
- `prerequisite_before_boss`: P06-C01/C02/C03 for any runtime integration; P01-C02/P05-C02 specifically before official
  live Boss publication, not before isolated kernel/schema work.
- `coimplement_with_boss_foundation`: P01-C01 source inventory and P05-C04 typed setup/binding.
- `defer_until_after_vertical_slice`: P07-C03/C04 cleanup and P09-C05 projectile kernel; P10-C03 waits until the first
  actual public Boss Wire change if the minimal slice uses existing DTOs.
- `independent_existing_wave`: P07-C01/C02, P08-C01/C05, P09-C03/C06, P10-C02/C04. Coordinate shared primitives but do
  not merge owners or make these unrelated corrections Boss gates.
- No candidate creates policy authority. Especially P09-C03 multi-step usefulness and P09-C06 tile reach do not define
  Boss target lock/path cost.

## Final-boss stress durable checkpoint

- `abnormality_skill_design_notes.ko.md` explicitly marks the three final bosses as future concepts with open content/
  runtime/wire choices. They are stress inputs only.
- WhiteNight requires future adapters for encounter objectives, terminal interception/VN, partial reset/resurrection,
  multi-zone actor mechanics and run-local history; only generic encounter/actor ownership belongs in v1 foundation.
- Silent Orchestra fits timed phases, typed immunity/target conditions, waves, phase-end events and resource mechanics;
  no boss-specific graph node kind is needed.
- Apocalypse Bird fits persistent objectives, one-way mechanic disable and hazard cancellation but fog, large-map nav,
  tactical deployables and Unity camera/minimap remain separate future systems.
- Kernel verdict: encounter runtime + per-actor runtime + finite typed graph + explicit mechanic adapters is extensible
  enough. Universal scripting, concept-specific enums and unresolved content policy are excluded.

## Policy matrix durable checkpoint

- `POLICY_COVERAGE_MATRIX.md` now contains exactly one row for every `BP-01`~`BP-34`; row count is 34, duplicate output
  is empty, and expected-versus-actual ID difference is empty.
- Each row fixes the approved target decision, current source/canonical relation, missing authority, one primary
  disposition, implementation owner/sequence and future test-first acceptance. The matrix does not depend on
  conversation context for any policy conclusion.
- Primary totals are 28 new-foundation, 4 existing-refactor coordination and 2 existing-primitive correction rows.
  This does not mean 28 independent projects: they are facets of one Boss definition/runtime foundation.
- No material policy conflict or inaccessible required consumer remains. Future objective repath and dynamic hot
  reload/LKG remain strict-reject/future-owner subfeatures, not duplicate BP rows or hidden v1 requirements.

## Readiness report durable checkpoint

- Primary verdict is `conditionally_ready`, with three intentionally different meanings: G0 correctness is ready for
  separate authorization, G1 foundation/G2 fixture slice is ready after G0, and official live/public Wire publication
  is not yet ready.
- Minimal dependency is P06-C01/C02/C03 → Boss foundation with P01-C01/P05-C04 → existing-event fixture tracking
  slice. P01-C02/P05-C02 gate official live opt-in; P10-C03 + server/Unity sync gates the first public Boss lifecycle.
- The reference fixture explicitly covers opt-in/non-opt-in parity, one encounter/actor/phase, a finite
  `TrackAndRejoin` graph, typed acquisition/lock/movement results and monotonic forward route recovery. It adds no
  official Boss content and no public graph/target/epoch field.
- Test-first matrices now fix prerequisite atomic/order/terminal characterization, strict catalog/setup validation,
  lifecycle/arbitration/target/movement/time/RNG atomicity, interrupt/reconnect and unchanged-Wire acceptance.
- Current active battle reconnect retains internal Boss state in Core session memory. Public Unity checkpoint must not
  be treated as a Core restore format; any future durable active-battle restoration owns a separate Core-only capture.
- No additional policy decision is required. Dynamic nav/repath, hot reload, public lifecycle and final-boss mechanics
  remain strict future boundaries rather than implicit prerequisites.

## Unresolved ledger closure

이 section은 앞선 chronological checkpoint의 `Initially unverified`/`unresolved` 문구를 삭제하지 않고 최종
disposition에 연결한다. 아래 항목 중 material unresolved는 없다.

| Earlier question | Final disposition | Durable evidence |
| --- | --- | --- |
| Boss catalog의 live bundle seam | Same atomic candidate DB에 새 logical source를 coimplement; startup fail-closed를 재사용하고 dynamic reload는 v1 비범위. | `BOUNDARY_AUDIT.md` Current end-to-end trace/Adjacent canonical, matrix BP-28, readiness G1 |
| Opt-in binding owner | `PveEncounter` optional reference를 preview/setup에서 immutable typed scenario encounter/actor binding으로 resolve. | Boundary Setup trace, matrix BP-01/BP-05/BP-29, readiness G1 |
| Movement/target injection seam | Planner authored-route fallback 앞의 explicit opt-in intent; separate activation target lock, static-grid query and monotonic route cursor. | Boundary Movement/target, matrix BP-02/BP-03/BP-18~21, readiness G1/G2 |
| Same-time microstep/epoch primitive | Existing recursive timestamp drain and leaf epochs are reusable but total order and microstep budget/lifecycle owner are missing. P06-C02 prerequisite + Boss foundation. | Boundary Scheduler detail, matrix BP-11/BP-16/BP-34 |
| Core checkpoint/restore surface | Active runtime is in-memory Core session; public checkpoint is Unity reconcile projection, not Core restore. Future durable active-battle restore owns a separate Core-only capture. | Boundary Event/checkpoint, readiness G1 and interrupt/reconnect cases |
| Boss presentation amplification | Existing-event slice has no Wire change. First public lifecycle requires Core semantic DTO, P10-C03 server parity and stale Unity synchronized lifecycle/reset consumer. | Boundary Event/checkpoint, matrix BP-30, readiness G3b |
| Unity readiness/cancellation | Stale consumer supports seq/revision/checkpoint/movement/damage only; unknown events, same-UUID reset, duration cancel/end, death/facing remain future synchronization debt. | Boundary stale Unity findings, matrix BP-30, readiness G3b |
| Final-boss expression fit | Generic encounter/actor ownership + finite typed graph + explicit mechanic adapters is sufficient; concept-specific systems remain separate future Goals. | Boundary stress table, readiness final-boss stress table |
| Runtime canonical failure no-mutation | Current live advance does not satisfy it; CRA-P06-C03 is a prerequisite with full pre/post equality acceptance. | Boundary Adjacent canonical/Scheduler, matrix BP-31, readiness G0 |
| BP-28 last-known-good question | Atomic startup candidate is sufficient for v1. LKG is required only if a future dynamic publisher exists and belongs to that publisher. | Matrix BP-28, readiness Authoring/publication |

The initial `discovery pending` text in `PLAN.md` remains historical preliminary evidence and is superseded by this
ledger plus the final boundary/matrix/report. It is not an active Stop Condition or open policy decision.

## Concurrent Unity Spine change durable checkpoint

- Final status comparison changed from initial Unity full `2059`/supporting-scope `306` entries to `2108`/`350`.
  Simulator production-scoped status stayed exactly `211` entries with the initial SHA-256.
- Files newer than the Goal start timestamp show 22 concurrent Unity writes concentrated in
  `Assets/Scripts/ArkClient/SpinePresentation/**`, its EditMode/PlayMode tests and
  `docs/goals/spine_character_presentation_core/**` evidence/docs. This audit issued no Unity write command and every
  recorded write is an `apply_patch` under the Simulator Goal directory.
- Current-tree re-audit found `ArkClient.SpinePresentation` and its sandbox reference only Spine assemblies; tests
  explicitly assert no `ArkClient.Runtime`, JSON or WebSocket dependency. No source outside the SpinePresentation
  directory references `SpineUnitVisual`/`IUnitVisual`/the assembly.
- Updated disposition: Unity now has a reusable caller-driven semantic visual/action/interrupt/death leaf primitive,
  but it remains sandbox/local-input and has no battle event, seq, presentation-instance, checkpoint lifecycle or
  same-UUID rebuild integration. The earlier “no Spine battle integration” finding remains true with this narrower
  evidence; G3b can reuse the primitive but is still required.

## Completion-review durable checkpoint

- All seven required artifacts are non-empty. The completion audit contains 14/14 completion rows and 14/14 invariant
  rows, with no `fail` row and an explicit `Complete / High` verdict.
- BP coverage remains exactly 34 and refactor intersection exactly 19. All historical unresolved items point to the
  closure ledger and no open policy question remains.
- Simulator production-scoped status/hash remains byte-identical to the Goal baseline. Concurrent Unity Spine status
  remained stable through the final check and was re-audited rather than reverted or ignored.
- Runtime/broad tests remain intentionally unexecuted because this is an investigation-only/no-production-diff Goal;
  no test count is fabricated. Future failing acceptance is fixed in the readiness report.
- Final status action: `active` → `complete`; `Implementation Complete` stays `not_applicable`; implementation
  readiness stays `conditionally_ready`, and no follow-up implementation is authorized by this transition.
