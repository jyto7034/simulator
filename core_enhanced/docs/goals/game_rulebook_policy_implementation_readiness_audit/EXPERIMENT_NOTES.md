# Experiment Notes

## Initial Repository State

- Repository root: /mnt/f/work/simulator
- Start commit: f318414fb0156a67e04c5479000c2a2683fc0c09
- Goal path collision: 없음
- Worktree는 Goal 시작 전부터 Core, live data, server, canonical과 다수 Goal 경로에 걸쳐 dirty하다.
- 기존 tracked/untracked 변경은 사용자 소유로 간주하고 수정하지 않는다.
- 외부 Unity repository commit, dirty 상태와 runner 접근성은 Goal 실행 시 확인한다.

## Initial Decisions

- Goal 종류: single
- 실행 모드: investigation_only
- production mutation: 승인되지 않음
- 기존 alignment audit의 finding을 구현하지 않고 구현 준비도만 평가한다.
- 15개 finding은 하나의 조사 deliverable로 평가하되 실제 구현 Goal 경계는 authority와 persistence
  lifetime을 읽은 뒤 제안한다.
- 난이도는 1~5 구조적 복잡도이며 인일이나 일정이 아니다.
- 개별 finding 난이도와 shared foundation을 포함한 work-package 난이도를 분리한다.
- 정책이 확정되지 않은 항목은 단일 점수 대신 provisional range와 stable decision ID를 사용한다.
- Difficulty, implementation risk와 assessment confidence를 서로 독립적으로 기록한다.
- 기존 감사 finding은 현재 worktree에서 재검증하며 과거 Goal의 결론만 proof로 재사용하지 않는다.
- Policy/finding 번호, 문서 순서, severity와 difficulty는 구현 순서로 사용하지 않는다.
- 실제 구현 순서는 work-package dependency graph, cycle/SCC disposition과 intermediate-state safety를
  별도 감사해 정한다.

## Pending Discovery

- 기존 감사 finding과 현재 worktree의 일치 여부
- 최신 정책 authority와 정책별 승인 상태
- finding별 observable acceptance criteria
- A~F 난이도 축의 실제 evidence와 override 필요 여부
- 공통 작업 묶음과 후속 Goal 경계
- package 간 typed dependency edge, 순환 의존과 안전한 topological sequence
- stage activation/completion gate, 병렬 가능 범위와 중간 code/data/save/consumer 안전성
- 접근 가능한 Unity 저장소와 runner 상태
- save/profile/checkpoint migration과 producer-consumer rollout 가정
- 추가로 필요한 gameplay, balance, content, persistence와 compatibility 결정

## Execution Baseline

- Simulator investigation commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Unity investigation commit: `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`.
- 두 저장소 모두 선행 audit와 같은 commit이나 working tree는 광범위하게 dirty하다. 현재 실제
  evidence는 HEAD가 아니라 2026-08-12 baseline 시점 working tree다.
- Simulator와 Unity initial status fingerprint 및 input/canonical hash는 `EXPERIMENTS.md`의
  `Execution Baseline And Evidence Freeze`에 기록했다.
- Core, live RON, game_server와 외부 Unity source/canonical은 readable하다.
- Unity runner는 shell에서 발견되지 않았다. Unity 관련 평가는 source와 기존 test assertion까지만
  가능하며 fresh runner evidence는 제공하지 않는다.
- 조사 중 status fingerprint가 Goal 문서 write 때문에 달라질 수 있으므로 final 무변경 감사에서는
  허용된 Goal 디렉터리를 제외한 path-level diff와 input hash를 함께 비교한다.

## Dirty Worktree Boundary

- 이 Goal에서 허용되는 write는
  core_enhanced/docs/goals/game_rulebook_policy_implementation_readiness_audit/ 아래 문서뿐이다.
- game_rulebook.md, 기존 alignment audit artifact, production Rust/RON/test, game_server와 외부
  Unity는 읽기 전용이다.
- initial status가 매우 넓으므로 final 검증은 repository 전체 clean 여부가 아니라 Goal-scoped
  path 목록, input hash와 관련 production path의 변경 여부를 비교한다.

## Bounded Unit A Decisions And Unknowns

- `GRB-M-001`, `GRB-M-003`, `GRB-M-004`는 룰북 상단 승인 주석과 상세 acceptance가 있어
  target 방향 자체는 `approved_target`이다.
- Standard authored exact fragment ID와 지급 시점은 룰북이 후속 RON authoring으로 위임했다. 이는
  fixed tutorial framework를 구현하는 package와 content publication gate를 구분할 근거다.
- Target global profile은 compendium과 같은 local-install monotonic lifetime이지만 current durable
  envelope는 compendium-only V1이다. 기존 발견 기록을 보존할 migration/strict cutover 여부는
  사용자 progression에 영향을 주므로 compatibility/rollout decision으로 분리한다.
- Standard completion과 tutorial flag durable publish가 원자적이어야 한다. Core in-memory flag만 먼저
  적용하거나 server publish 뒤 result 실패 시 rollback할 방법 없이 두 authority를 두는 방식은
  허용할 수 없다.
- All-FinalBoss permanent record는 `GRB-M-012`의 run-local chain completion identity가 먼저 정의돼야
  정확히 계산할 수 있다. Profile schema를 미래 placeholder로 먼저 넣지 않고 해당 package dependency로
  기록한다.

## Bounded Unit B Decisions And Unknowns

- Research 제거와 encounter-grade fragment ledger 도입은 competing acquisition authority를 남기지
  않기 위해 같은 save/result cutover package로 묶어야 한다. 새 ledger 없이 research를 먼저 제거하거나
  research와 새 source를 동시에 유지하는 중간 상태는 허용하지 않는다.
- 룰북은 exact fragment ID/repeat quantity를 개별 RON authoring에 위임했으므로 숫자 선택을 readiness
  평가자가 대신하지 않는다. 후속 Goal completion에는 target live entry publication이 필요하다.
- “reward source”의 stable dedupe identity가 PVE encounter ID인지 explicit authored shared source ID인지
  current canonical만으로 완전히 닫히지 않는다. 여러 encounter variant가 한 파편 source를 공유할 때
  사용자-visible first/repeat 의미가 달라질 수 있어 policy decision 후보로 유지한다.
- Active-run payload V3는 research state를 strict 저장한다. V4 strict rejection만 할지, 기존 V3의 owned
  fragment/upgrade state를 보존하면서 research-only fields를 변환할지에 따라 player progression과
  구현 난이도가 달라지므로 rollout decision이 필요하다.
- Ordinary Reward exclusion은 data-only removal로 끝내지 않는다. Production loader가 source category와
  reward pool을 교차 검증하지 않으면 같은 잘못된 authoring이 재진입할 수 있다.

## Applied Alternatives

- 기존 MISMATCH_CATALOG.md에 난이도 열을 직접 추가하지 않는다. 확인된 불일치 사실과 미래 구현
  추정을 분리하기 위해 새 Goal artifact가 평가를 소유한다.
- 15개 finding을 즉시 15개 구현 Goal로 변환하지 않는다. shared authority와 migration lifetime을
  확인한 뒤 package를 제안한다.
- difficulty만으로 readiness를 판정하지 않는다. 미결 정책, 외부 evidence, migration과 test
  safety net을 독립 gate로 둔다.
- finding을 번호 순서대로 구현하거나 단순 prerequisite 목록만 작성하지 않는다. Package graph,
  coimplementation cycle, topological layer와 각 stage의 안전성을 별도 산출물로 검증한다.

## Bounded Unit C Decisions And Unknowns

- Elite pool/mastery, Monolith failure와 encounter reward ledger를 서로 다른 durable authority로 추가하지
  않는다. `GRB-M-005`, `008`, `014`는 한 replacement run-state/checkpoint schema에서 coimplement한다.
- Stage 2/3 stat 배율의 initial target은 150%/200%다. 개발 중 RON tuning으로 조절할 수 있으므로 loader는
  exact constant보다 positive/ordered/safe-range invariant를 보호한다. Reward 배율은 이 finding의 target이
  아니므로 암묵적으로 stat 배율에 맞추지 않는다.
- Sweep은 fake `CombatBattleState`나 fabricated event log를 만들지 않는다. Battle-independent planner가
  기존 economy diff/XP primitives를 재사용하고 condition/discovery/battle-record path를 건너뛰어야 한다.
- Sweep eligibility가 의존하는 “mastered Elite”와 repeat reward source identity는 `GRB-M-005`/`008`이 먼저
  확정·구현되어야 한다. 따라서 sweep은 별도 후속 vertical slice이며 ledger package와 병렬 안전하지 않다.
- Remanifestation exhausted failure는 approved terminal policy다. Active-run migration 방식은 Unit B의
  공통 rollout decision에 포함하고 별도 gameplay policy 질문으로 중복하지 않는다.

## Bounded Unit D Decisions And Unknowns

- NormalBoss와 FinalBoss cycle, BossOmen provisional queue/active step을 한 run-local encounter progression
  authority에 모은다. 서로 다른 ad-hoc set/cursor를 추가하면 assignment, reward와 checkpoint가 갈라진다.
- Event/Combat source 실패 처리와 FinalBoss defeat는 룰북에서 이미 확정됐다. 추가 gameplay decision으로
  올리지 않고 outcome/provenance를 typed state로 운반하는 구현 문제로 분류한다.
- Per-chain minimum Floor, optional fragment/awakening requirement와 step content는 공통 fallback을 만들지
  않고 live chain RON authoring gate로 남긴다. 현재 `white_night` 값을 readiness 평가자가 추론하지 않는다.
- FinalBoss cycle completion은 global all-boss permanent record의 유일한 Core authority다. Profile package가
  PVE class 전체나 compendium을 보고 독자적으로 재계산하지 않도록 `GRB-M-012 -> GRB-M-004` dependency를
  유지한다.
- Boss identity/reward-source identity는 stable authored key 결정과 함께 정해야 한다. Encounter variant가
  cycle 구성원인지 하나의 shared boss identity인지에 따라 NormalBoss/FinalBoss first/repeat 의미가 바뀌므로
  Unit B의 source identity rollout decision이 이 unit도 gate한다.

## Bounded Unit E Decisions And Unknowns

- `GRB-M-015`를 framework-only 구현 finding으로 축소하지 않는다. Existing ActiveSkill foundation은 이미
  live 2개가 사용하므로 29개 placeholder를 그대로 둔 framework patch는 observable target을 충족하지 않는다.
- 29개 개별 design 없이 evaluator가 mechanics/balance/compatibility를 발명하지 않는다. Initial unit은
  campaign scope 결정을 요구한다고 분류했지만, 뒤의 reconciliation은 룰북의 explicit no-conversion
  sentence에 따라 별도 future Master로 넘기는 것으로 이 판단을 supersede했다.
- Campaign을 승인하면 batch마다 RON+Core battle semantics+Unity presentation/assets+tests가 닫힌 vertical
  slice여야 한다. Schema만 먼저 만들거나 모든 ID를 공용 no-op/generic buff에 연결하는 중간 상태는 금지한다.
- 최종 zero-placeholder gate는 exact live count와 active skill references/asset coverage를 다시 계산한다.

## Final Readiness Disposition

- Individual fixed difficulty는 M006=1, M003/M009=2, M001/M005/M007/M010~M014=4,
  M002/M004/M008=5다. M015는 design에 따라 provisional 3–5다. 이 값은 합산 공수가 아니다.
- Actual package difficulty는 WP002=1, WP004=4, WP001/WP003=5, WP005 provisional 3–5다.
- Raw finding cycles는 WP001로 coimplement했고 package graph에는 multi-node SCC가 없다.
- Recommended current serialization은 WP002 → WP001 → WP003 → WP004다. WP001과 WP002는 graph상 같은
  layer이나 shared RON/loader ownership 때문에 병렬 안전하지 않다. WP005는 룰북이 이번 correction에서
  제외한 `future_or_dormant` node다.
- Candidate reduction 당시 open이던 PD004/006 runtime bootstrap은 이후 사용자 resolution으로 닫혔다. Save V3, profile V1
  preservation, authored source-key 방식, live BossOmen dormancy와 fragment campaign scope도
  PD001/002/003/005/007로 닫혀 전체 open runtime/bootstrap input은 0이다. Final content input은 해결된 것이
  아니라 별도 content Goal로 명시적으로 defer됐다.
- Investigation artifacts와 completion audit은 재평가를 마쳤고 Goal은 `complete`, overall verdict는
  `ready_for_implementation`이다.
- `complete`는 read-only readiness investigation의 완료이며 implementation authorization 또는 production
  implementation 완료를 뜻하지 않는다.

## Superseding Decision Reconciliation Notes

- Strict active-run rejection은 사용자 선택을 새로 요구하지 않는다. `data_loading_contract.md`의 exact V3,
  `core_runtime_contract.md`의 immediate V2 reject/no dual reader와 룰북의 no repair/migration이 Option A를
  함께 고정한다.
- Profile migration은 arbitrary compatibility가 아니라 same-semantic monotonic set 보존이다. 기존 local
  compendium 정책과 최신 룰북 모두 confirmed full reset 외 삭제를 허용하지 않으므로 V1 set reset은
  기본 rollout이 될 수 없다. 새 flag/record는 소급 근거가 없어 false/0으로 시작한다.
- Stable source key의 field 이름은 implementation plan이 정할 수 있다. Live 30개 eligible encounter와
  primary abnormality는 각각 30/30 unique이고 King of Greed reference도 correct다. 그럼에도 future
  variants/multiple fragments에서 abnormality key는 독립 source를 표현하지 못하고 raw encounter ID는
  content refactor에 lifetime을 결합한다. Explicit authored key + intentional shared-key validation은 approved
  first/repeat semantics를 이 두 layout identity에서 분리한다.
- M015는 initial audit가 scope question으로 과대 분류했다. Formal future policy와 current correction
  completion condition은 다르며, 룰북의 explicit no-conversion sentence가 current scope를 닫는다.
- Initial reconciliation은 PD004/005/006을 production-visible authored 값으로 보아 open으로 유지했다.
  뒤의 content candidate reduction은 PD005가 새 값을 요구하는 문제가 아니라 canonical future boundary를
  적용하는 문제임을 확인했고, 이후 user resolution이 PD004 repeat/PD006 tutorial content도 닫아 이 판단을
  supersede한다.
- Reconciliation 도중 content-roster source에 사용자 동시 변경으로 Moonlit Wail 제외가 확정됐다.
  Rulebook의 Queen/Mountain NormalBoss 재분류와 합치면 target encounter membership candidate는 Elite 13,
  NormalBoss 13, FinalBoss 3이다. PD004는 이 membership을 다시 묻지 않고 29개 exact fragment/repeat
  authoring만 남긴다.

## Content Candidate Reduction Notes

- PD004의 fragment identity는 `CONTENT_INPUT_CANDIDATES.md`의 exact 29-row table로 유도 가능하다. 이 표는
  current response-complete fragment identity를 target roster에 투영한 것이며 repeat quantity 승인은
  포함하지 않는다.
- Repeat reward의 진짜 남은 선택은 current live policy/primitive를 보존하는 dust 5/10/20과 explicit
  `None`이다. 둘 다 룰북이 허용하지만 economy 결과가 다르므로 audit가 대신 고르지 않는다.
- PD005는 final content-input 질문으로 남기지 않는다. Current one-step WhiteNight는 generic BossOmen
  runtime과 all-current-FinalBoss record의 유일 live development bootstrap으로 유지한다. Detailed
  WhiteNight/Silent Orchestra/Apocalypse Bird content는 future Goal로 남기며 bootstrap을 final content로
  승격하지 않는다.
- Existing Standard topology 세 개는 fixed template 후보로 충분하다. Weighted category/content와 adaptive
  difficulty만 target authority가 아니며 F3 terminal은 FinalBoss에서 Standard-completing NormalBoss로
  바뀌어야 한다.
- Current scripted fragment 후보 둘은 starter loadout과 호환되지 않는다. Freischutz는 Ranged+Gun,
  Funeral Butterfly는 Ranged+Bow/Gun인데 모든 starter는 Melee+Sword다. Compatible equipment까지 함께
  authoring하지 않는 한 `scripted_fragment: None`이 유일한 bounded no-new-content candidate다.
- `ST1`은 exact approval candidate일 뿐 balance evidence가 아니다. Neutral 100% tuning과 repeated sealed
  archive Event 사용은 사용자가 승인하거나 수정해야 한다.
- Candidate-reduction 시점 open runtime/bootstrap input은 PD004/PD006 두 개였으나 이후 둘 다
  `closed_by_user`다. Final bootstrap-input state는 zero이며 final content authority는 out of scope다.

## User Decision Closure Notes

- PD004는 minimum bootstrap `R1-continuity`로 확정했다. Bootstrap manifest의 repeat reward는 encounter class별 explicit
  `GrantFragmentDust`: Elite 5, NormalBoss 10, FinalBoss 20이며 implicit fallback은 없다.
- PD010은 mastered Elite sweep에서 source fragment count 0이면 repeat reward 대신 one-copy reacquisition,
  count 1 이상이면 repeat reward로 확정했다. Dismantle→reacquire cycle은 제한 없이 허용하고 identity별
  upgrade/awakening progress는 보존한다.
- PD006은 minimum bootstrap `ST1-NT60`으로 확정했다. 기존 제안의 topology/slot/content/F2 Porcubbus/neutral tuning을
  유지하되 F3 NormalBoss를 Nothing There로 바꾼다.
- F3 Nothing There boss actor만 base HP/ATK/DEF/MR의 60%에서 유도한 exact override를 사용한다. Base
  `1200/88/60/55`에서 existing integer percentage convention을 적용한 target은 `720/53/36/33`이다. Current
  generic `BattleUnitStatScale`은 magic resistance가 없고 encounter-wide이므로 구현 수단이 아니다. 같은
  encounter의 다른 actor, 다른 Floor actor/node/reward와 다른 Nothing There 출현에는 이를 전파하지 않는다.
- Scripted fragment reward는 `None`, adaptive scaling도 없다. F3 승리의 Standard completion + permanent
  tutorial flag transaction은 기존 제안대로 유지한다.
- 모든 PD001~010이 닫혔고 package/sequence gate를 재평가했다. Final status는 investigation `complete`,
  readiness `ready_for_implementation`; production change는 separate approved implementation Goal이 필요하다.

## Bootstrap-Only Content Authority Notes

- 위 PD004/005/006 concrete 값과 PD010 branch는 게임 구동과 contract validation에 필요한 initial
  fixture/policy이며 실제/최종
  콘텐츠 결정이 아니다.
- 별도 content Goal은 source roster/pairing/repeat balance, Standard topology/encounter/tuning/reward/script/
  narrative/assets, dedicated skills와 final BossOmen chains를 소유한다.
- Final content가 explicit source row, first-or-repeat single transaction, ordinary Reward exclusion, fixed
  three-Floor/no-adaptive Standard와 F3 completion/profile atomicity를 지키면 bootstrap 값을 자유롭게 교체할
  수 있으며 이 audit/PD004/PD005/PD006/PD010을 reopen하지 않는다.
- 따라서 최종 상태 표현은 `Open Bootstrap Inputs: none`, `Final Content Authority:
  deferred_to_separate_content_goals`이다.
