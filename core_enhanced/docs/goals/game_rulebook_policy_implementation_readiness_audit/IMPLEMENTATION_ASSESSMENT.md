# Implementation Assessment

이 평가는 2026-08-12 baseline working tree에서 재확인한 15개 finding의 **individual marginal
difficulty**다. Shared foundation을 이미 사용할 수 있다고 가정한 finding별 비교값이며 서로 더하지
않는다. 실제 후속 Goal 판단에는 `WORK_PACKAGE_MAP.md`의 package-level 점수를 사용한다.

## Primary Matrix

| Finding | Current validity | Target disposition | A | B | C | D | E | F | Sum | Difficulty | Risk | Confidence | Policy decisions | Work package | Item readiness |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- | --- | --- | --- | --- |
| `GRB-M-001` | confirmed | approved_target | 2 | 2 | 2 | 2 | 3 | 3 | 14 | 4 | high | medium | none (closed `GRB-PD-006`) | `GRB-WP-003` | ready_with_prerequisite |
| `GRB-M-002` | confirmed | approved_target | 3 | 3 | 3 | 3 | 2 | 3 | 17 | 5 | critical | high | none (closed `GRB-PD-004`) | `GRB-WP-001` | ready_with_prerequisite |
| `GRB-M-003` | confirmed | approved_target | 1 | 1 | 0 | 0 | 0 | 2 | 4 | 2 | low | high | none | `GRB-WP-001` | ready_with_prerequisite |
| `GRB-M-004` | confirmed | approved_target | 3 | 3 | 3 | 3 | 0 | 3 | 15 | 5 | critical | medium | none | `GRB-WP-003` | ready_with_prerequisite |
| `GRB-M-005` | confirmed | approved_target | 3 | 2 | 3 | 1 | 1 | 3 | 13 | 4 | high | medium | none (closed `GRB-PD-008`) | `GRB-WP-001` | ready_with_prerequisite |
| `GRB-M-006` | confirmed | approved_target | 0 | 0 | 0 | 0 | 1 | 2 | 3 | 1 | low | high | none (closed `GRB-PD-009`) | `GRB-WP-002` | ready |
| `GRB-M-007` | confirmed | approved_target | 2 | 3 | 2 | 3 | 1 | 3 | 14 | 4 | high | medium | none (closed `GRB-PD-004/010`) | `GRB-WP-004` | ready_with_prerequisite |
| `GRB-M-008` | confirmed | approved_target | 3 | 3 | 3 | 3 | 2 | 3 | 17 | 5 | critical | medium | none (closed `GRB-PD-004/010`) | `GRB-WP-001` | ready_with_prerequisite |
| `GRB-M-009` | confirmed | approved_target | 1 | 1 | 0 | 0 | 1 | 2 | 5 | 2 | medium | high | none (closed `GRB-PD-004`) | `GRB-WP-001` | ready_with_prerequisite |
| `GRB-M-010` | confirmed | approved_target | 2 | 2 | 2 | 1 | 1 | 3 | 11 | 4 | high | medium | none (closed `GRB-PD-004`) | `GRB-WP-001` | ready_with_prerequisite |
| `GRB-M-011` | confirmed | approved_target | 2 | 2 | 2 | 2 | 2 | 3 | 13 | 4 | high | medium | none (closed `GRB-PD-005`) | `GRB-WP-001` | ready_with_prerequisite |
| `GRB-M-012` | confirmed | approved_target | 2 | 2 | 2 | 1 | 1 | 3 | 11 | 4 | critical | medium | none (closed `GRB-PD-004/005`) | `GRB-WP-001` | ready_with_prerequisite |
| `GRB-M-013` | confirmed | approved_target | 2 | 3 | 2 | 1 | 0 | 3 | 11 | 4 | high | high | none | `GRB-WP-001` | ready_with_prerequisite |
| `GRB-M-014` | confirmed | approved_target | 2 | 2 | 2 | 1 | 0 | 2 | 9 | 4 (override) | high | high | none | `GRB-WP-001` | ready_with_prerequisite |
| `GRB-M-015` | confirmed | future_or_dormant | 0–3 | 1–3 | 0–2 | 1–3 | 3 | 3 | 8–17 | future provisional 3–5 | high | low | none | `GRB-WP-005` | future_or_dormant |

`GRB-M-014`의 합계 기본 등급은 3이지만 serialized checkpoint state 변경이므로 minimum rule로 4다.
`GRB-M-015`의 3–5는 별도 future campaign이 활성화될 때의 정보성 범위다. 이번 correction은 live
conversion을 명시적으로 제외하므로 current package readiness와 overall gate에는 이 점수를 사용하지
않는다.

## Canonical Disposition Shared By All Findings

- **Current actual:** 이 문서와 `IMPLEMENTATION_BOUNDARY_AUDIT.md`가 조사 시점 runtime/live RON/
  persistence/server/Unity source에서 직접 추적한 경로다.
- **Current durable canonical:** 접근 가능한 Unity contract는 current producer/transport shape를 설명하지만
  target research removal, sweep, profile과 new encounter result를 아직 포함하지 않는다. 완료된 선행 Goal은
  evidence pointer이지 current gameplay canonical로 승격하지 않는다.
- **Approved target:** `docs/game_rulebook.md`의 2026-08-12 주석과 상세 절은 명시적으로 “정책 승인·구현
  대기”이며 current implementation이라고 주장하지 않는다. 각 section의 `approved_target`은 이 target
  의미를 가리킨다.
- **Audit resolution:** implementation shape와 scores/package/sequence는 production canonical이 아니지만,
  decision 값은 canonical evidence 또는 audit/user resolution으로 모두 닫혔다. 이는 구현 준비 완료를
  뜻할 뿐 production mutation 권한은 아니다. PD004/005/006 concrete 값은 final content decision이 아니라
  initial runnable/live-bootstrap fixture다.

따라서 아래 상세 section에서 current code/data와 target claim이 달라도 drift를 숨기지 않으며,
`POLICY_DECISION_LOG.md`가 기록한 closed resolution을 구현 입력으로 사용한다.

## GRB-M-001 — Fixed Standard Tutorial

- **Claim/disposition/acceptance:** `approved_target`. Minimum bootstrap `ST1-NT60`은 세 authored fixed Floor와 fixed
  tuning, F1 Normal, F2 Porcubbus Elite, F3 Nothing There NormalBoss를 사용한다. F3 boss actor에만 base
  HP/ATK/DEF/MR 60%에서 유도한 exact override `720/53/36/33`을 적용하며 나머지 적·노드·보상과 다른
  Nothing There 출현은 ST1 100%다. FinalBoss와
  adaptive power는 없고 F3 승리와 tutorial flag를 함께 commit한다. Script fragment 지급은 없다.
- **Content authority limit:** 이 layout/encounter/tuning은 final tutorial content가 아니다. 별도 content
  Goal이 fixed three-Floor/no-adaptive/terminal transaction invariant를 유지하면서 교체할 수 있다.
- **Current path/live data:** `StartNewGame` → starter commit → shared `generate_current_floor_map` → facility
  generator/assignment/difficulty path다. Live Standard schedule도 generated template pool이고 F3 terminal은
  FinalBoss class다. Existing tests가 이 current contract를 고정한다.
- **Contract/persistence/consumer:** Run map/checkpoint shape는 재사용 가능하나 Standard active-run
  replacement 정책과 tutorial profile publish가 필요하다. Core admission/snapshot, server profile publish,
  Unity menu disabled reason가 함께 바뀐다.
- **Boundary/work type:** authored tutorial map/script catalog+strict validation, F3 actor-only four-axis exact
  stat override, mode-specific generator, difficulty bypass, terminal result/profile transaction과 legacy Standard
  assertions 제거. Generic encounter-wide three-axis `BattleUnitStatScale`로 이 override를 대체하지 않는다. Strict coordinated
  release를 가정하며 old/new generator fallback은 두지 않는다.
- **Safety net:** exact map/content loading golden tests, fixed difficulty invariants, script fragment
  success/dismantle/failure, F3 result/profile atomicity, checkpoint/restart, server order와 Unity menu contract;
  broad map/combat/save regression.
- **Score:** A2 B2 C2 D2 E3 F3 = 14 → difficulty 4. Initial authored map/tuning fixture는 closed
  `GRB-PD-006`의 `ST1-NT60`, profile migration은 closed `GRB-PD-002`가 고정한다. Final content design은
  별도 Goal이며 남은 implementation 불확실성은 production authoring과 balance/runner 검증이다. Package `GRB-WP-003`, prerequisite `GRB-WP-001`의
  FinalBoss record seam. Readiness `ready_with_prerequisite`.

## GRB-M-002 — Remove Research And Pending Delivery

- **Claim/disposition/acceptance:** `approved_target`. Run/reward/HQ/result/save/DTO/Unity 어디에도 research
  point, completion, pending delivery가 없고 fragment owned/upgrade/awakening/dust는 보존한다.
- **Current path/live data:** RunState research, inventory research/pending queue, combat victory and planning
  delivery가 live run policy/abnormality/reward RON과 연결돼 있다. V3 active-run/checkpoint, root/result DTO,
  server mapping과 strict Unity delivery reader까지 reachable하다.
- **Boundary/work type:** research types/fields/branches/effects/RON/DTO/readers를 제거하고 encounter ledger의
  direct acquisition으로 원자 교체한다. `GRB-M-008` 없이 먼저 제거하거나 두 acquisition authority를 같이
  운영하지 않는다.
- **Migration/rollout:** Closed `GRB-PD-001`에 따라 V3는 strict reject한다. Public result cutover는
  Core/server/Unity 한 release에서 strict하게 한다.
- **Safety net:** old-field load rejection/approved migration fixtures, first/repeat acquisition, planning return
  no-delivery, server ordering/Unity strict parsing과 repository-wide old-symbol zero check; broad run/save/economy.
- **Score:** A3 B3 C3 D3 E2 F3 = 17 → difficulty 5. Risk critical, confidence high. Closed `GRB-PD-004`는
  repeat dust를 Elite 5/NormalBoss 10/FinalBoss 20으로 고정하고, closed `GRB-PD-001`은 strict V3 rejection을
  고정한다. Package `GRB-WP-001`. Readiness `ready_with_prerequisite`.

## GRB-M-003 — Remove Endless Auto-Completion

- **Claim/disposition/acceptance:** `approved_target`. Endless의 어떤 encounter/ledger completion도
  `RunComplete`를 만들지 않고 node/floor progression을 계속한다.
- **Current path/live data:** Combat victory가 `all_response_complete`를 계산해 result commit에서
  `RunComplete`로 전이한다. Public RunComplete type은 Standard에 계속 필요하다.
- **Boundary/work type:** `GRB-M-002` cutover 안에서 계산/branch/current assertion만 제거한다. Data,
  save 또는 consumer shape를 별도로 바꾸지 않는다.
- **Safety net:** 마지막 completion victory 후 NodeCompleted/FloorAdvanced, FinalBoss victory 후 exit gate,
  Standard RunComplete 유지와 abandonment/failure regression.
- **Score:** A1 B1 C0 D0 E0 F2 = 4 → difficulty 2. Risk low, confidence high. Decision 없음;
  `GRB-WP-001`에서 M002/M008 prerequisite와 coimplement. Readiness `ready_with_prerequisite`.

## GRB-M-004 — Global Tutorial Flag And Permanent Records

- **Claim/disposition/acceptance:** `approved_target`. Tutorial flag는 Standard F3 승리에만 false→true,
  Endless record는 즉시 monotonic durable commit하며 checkpoint/Run failure가 되돌리지 않는다. False이면
  Core가 Endless를 거부한다.
- **Current path/live data:** global durable authority는 compendium V1의 four sets뿐이다. Target fields,
  start gate, result mutation, host publish 또는 Unity disabled presentation이 없다.
- **Boundary/work type:** unified/adjacent global gameplay profile, monotonic reducer, Core→host publish/
  acknowledgement, store version+backup/reset, snapshot/capability와 Unity reader를 만든다. All-FinalBoss input은
  M012 ledger의 typed event만 사용한다.
- **Migration/rollout:** closed `GRB-PD-002`에 따라 existing compendium V1의 four sets를 successor profile에
  보존하고 새 tutorial flag/records는 기본값에서 시작한다. Active-run과 profile의
  서로 다른 lifetime을 result 하나에서 atomic하게 다루며 in-memory-only success를 금지한다.
- **Safety net:** V1/current/backup/corruption/reset, publish failure/retry/reconnect, checkpoint rollback
  independence, monotonic max/achievement, Standard exclusion, Core/server/Unity fixtures와 process restart.
- **Score:** A3 B3 C3 D3 E0 F3 = 15 → difficulty 5. Risk critical, confidence medium. Decision
  no open decision; closed `GRB-PD-002` fixes V1 preservation. Package `GRB-WP-003`, prerequisite M012.
  Readiness `ready_with_prerequisite`.

## GRB-M-005 — Elite Unmastered/Mastered Pools

- **Claim/disposition/acceptance:** `approved_target`. Third eligible victory까지 unmastered이며 Floor의 Elite
  slots는 distinct unmastered를 우선 채운다. 부족한 slot만 mastered deterministic repeat pool로 채우고,
  실패/retreat/unvisited assignment의 정확한 반환 규칙을 지킨다.
- **Current path/live data:** Monolith natural outcome은 첫 승패 뒤 hard exclusion한다. Map assignment는
  resolved Elite를 제거하고 legacy research/recent weighting을 사용하며 empty pool은 Normal로 바꾼다.
- **Boundary/work type:** replacement encounter ledger에 victory/mastery/assigned/cycle cursor를 저장하고
  selector와 fallback을 교체한다. Third victory reward와 same-Floor uniqueness를 M008 transaction에서
  coimplement한다.
- **Migration/rollout:** V4 active-run policy와 stable authored identity가 gate다. Public wire는 최소
  projection만 필요하고 save/checkpoint strict validation은 필수다.
- **Safety net:** victory 1/2/3, defeat/retreat, reload, cycle refill, unvisited Gate return, scarce candidates,
  same-floor uniqueness와 deterministic seed corpus.
- **Score:** A3 B2 C3 D1 E1 F3 = 13 → difficulty 4. Risk high, confidence medium. Decisions
  no open decision; strict V3 rejection, explicit authored source key and mixed-fill policy are closed by
  `GRB-PD-001/003/008`.
  Package `GRB-WP-001`. Readiness `ready_with_prerequisite`.

## GRB-M-006 — Monolith Stat Multipliers

- **Claim/disposition/acceptance:** `approved_target`. Stage 2/3 stat multiplier의 초기값은 각각 150%/200%이며
  개발 중 RON tuning으로 조절할 수 있다. Reward multiplier는 이 finding에서 변경하지 않는다.
- **Current path/live data:** live values 100/125가 existing role tuning path에 적용되고 loader는 positive와
  increasing만 검사한다. Wire/save/consumer와 새 state는 관련 없다.
- **Boundary/work type:** 두 live value를 고치고 positive/ordered/safe-range validation과 baseline tests를
  보강한다. Existing tuning primitive와
  rounding을 재사용하고 별도 compatibility/fallback을 두지 않는다.
- **Safety net:** live load, Stage별 target/HP/attack/defense derivation, one-rounding/reward unchanged와 broad
  combat-difficulty regression.
- **Score:** A0 B0 C0 D0 E1 F2 = 3 → difficulty 1. Risk low, confidence high. Closed `GRB-PD-009`;
  `GRB-WP-002`, readiness `ready`.

## GRB-M-007 — Mastered Elite Sweep

- **Claim/disposition/acceptance:** `approved_target`. Mastered Elite에만 synthetic unconditional victory를
  만들고 same base reward, fragment count가 0이면 fragment 재획득 또는 1 이상이면 repeat reward 중 하나,
  alive-roster XP와 item duration만 적용한다. Condition, bonus, discovery, battle record와 simulation은
  바꾸지 않는다. 분쇄→재획득 cycle은 횟수 제한 없이 허용한다.
- **Current path/live data:** command/action/result/capability/server/Unity sweep path가 없다. Existing combat
  staged economy/XP primitives는 실제 battle/event log/result stats에 결합돼 있다.
- **Boundary/work type:** typed admission/preview, item validation, battle-independent staged planner,
  reward-ledger/mastered-cycle/node completion, result DTO/server/Unity presentation vertical slice다.
- **Migration/rollout:** M005/M008 V4 ledger 이후만 활성화한다. 기존 public contract에 새 action/result가
  추가되므로 strict coordinated release, old consumer fallback 없음.
- **Safety net:** eligibility matrix, no-battle/no-condition/no-record assertions, reward/XP/duration/cursor atomic
  failure, checkpoint rollback, server mapping/Unity command-result and broad combat/economy regression.
- **Score:** A2 B3 C2 D3 E1 F3 = 14 → difficulty 4. Risk high, confidence medium. Closed `GRB-PD-004`가
  live source mapping과 repeat reward를, `GRB-PD-003`이 explicit authored source identity를,
  `GRB-PD-010`이 reacquisition branch/cycle을 고정한다.
  `GRB-WP-004`, prerequisite
  `GRB-WP-001`. Readiness
  `ready_with_prerequisite`.

## GRB-M-008 — Encounter Fragment Acquisition Ledger

- **Claim/disposition/acceptance:** `approved_target`. Eligible victory마다 authored source 기준 first grant 또는
  repeat reward 정확히 하나를 atomic commit하고 defeat/retreat는 무변경이다. First-grant ledger는 분쇄 뒤에도
  reset하지 않는다. 단, mastered Elite 소탕은 현재 fragment count가 0이면 repeat reward 대신 한 copy를
  재획득하며 이 cycle을 제한 없이 반복할 수 있다. Checkpoint는 ledger/inventory를 함께 rewind한다.
- **Current path/live data:** strict PVE schema에 fragment/repeat/source가 없고 old research와 Monolith Stage 3
  special fragment만 있다. First/repeat/mastery/source ledger와 dedicated result consumer가 없다.
- **Boundary/work type:** typed source/acquisition data, cross-catalog validation, unified run ledger,
  Elite/NormalBoss/FinalBoss/BossOmen victory planner, economy diff/result projection과 strict live authoring을
  research authority와 원자 교체한다.
- **Migration/rollout:** closed strict V3 rejection, explicit authored source identity와 minimum 29-row
  bootstrap/repeat dust 5/10/20을 적용한다. Save/public wire와
  Core/server/Unity를 한 strict cutover에서 바꾼다.
- **Safety net:** class×first/repeat×success/failure matrix, dismantle/reacquire, late-failure atomicity,
  checkpoint/reconnect/migration, invalid/missing RON, server ordering/Unity read and broad reward/save regression.
- **Score:** A3 B3 C3 D3 E2 F3 = 17 → difficulty 5. Initial runnable source/repeat fixture is closed by
  `GRB-PD-004`; sweep reacquisition is closed by `GRB-PD-010`; final roster/pairing/balance remains a separate
  content authority. Strict V3/source-key resolutions are closed by `GRB-PD-001/003`. `GRB-WP-001`.
  Readiness `ready_with_prerequisite`.

## GRB-M-009 — Exclude Fragments From Ordinary Reward Nodes

- **Claim/disposition/acceptance:** `approved_target`. Ordinary Reward pool에 direct fragment/research effect가
  있으면 load-time fail하고 live membership은 제거한다.
- **Current path/live data:** production `default_treasures`에 두 direct fragment와 세 research family가
  있으며 generic reward resolver가 source category 없이 grant한다.
- **Boundary/work type:** live membership correction, source-aware pool validation과 research effect variant
  removal. Encounter/Standard가 쓰는 generic GrantSkillFragment primitive는 유지한다. Save/wire/Unity는
  별도 영향 없다.
- **Safety net:** invalid pool load rejection, live pool zero source count, encounter/Standard grant still works,
  reward session broad regression.
- **Score:** A1 B1 C0 D0 E1 F2 = 5 → difficulty 2. Risk medium, confidence high. Closed `GRB-PD-004`의 minimum
  bootstrap replacement와 `GRB-WP-001` coimplementation prerequisite. Readiness `ready_with_prerequisite`.

## GRB-M-010 — NormalBoss Cycle

- **Claim/disposition/acceptance:** `approved_target`. 각 authored identity를 cycle당 한 번 해결하며 victory와
  exhausted defeat는 소진, unvisited는 반환, cycle exhaustion 뒤 refill한다.
- **Current path/live data:** 12 live NormalBoss를 generic node seed selector가 고르고 run/checkpoint cycle
  state와 outcome-aware assignment resolution이 없다. Current content roster source는 Moonlit Wail을 제외하고
  Queen/Mountain을 NormalBoss로 재분류해 target membership candidate를 13개로 둔다.
- **Boundary/work type:** authored membership/source key, remaining+assigned cycle ledger, deterministic selector,
  node/Gate resolution과 first/repeat reward transaction을 unified V4 ledger에 추가한다.
- **Migration/rollout:** closed strict V3/source-key policy와 initial 13-member bootstrap target/reward fixture를
  적용한다. Final membership/content는 별도 Goal이 정하며 internal projection 외 consumer shape는 acquisition
  result 변경을 M008에서 공유한다.
- **Safety net:** full-cycle/refill, victory/exhausted/unvisited/retreat, reload determinism, first/repeat reward,
  invalid duplicate membership and broad map/save tests.
- **Score:** A2 B2 C2 D1 E1 F3 = 11 → difficulty 4. Risk high, confidence medium. Closed `GRB-PD-004` fixes the
  initial runnable manifest/repeat fixture, not final content; closed `GRB-PD-001/003` fix cutover/source identity. `GRB-WP-001`.
  Readiness `ready_with_prerequisite`.

## GRB-M-011 — BossOmen Per-Chain Eligibility

- **Claim/disposition/acceptance:** `approved_target`. Floor generation 때 chain별 optional required fragment와
  awakening을 평가하고 locked chain은 cycle에 남긴다. Unknown/cyclic/unreachable authoring은 fail-closed다.
- **Current path/live data:** chain schema에 condition/dependency가 없고 any-awakened fragment 하나가 global
  gate다. Live chain은 `white_night` 하나며 Core node DTO 외 typed Unity condition consumer가 없다.
- **Boundary/work type:** strict condition/dependency schema, catalog/DAG validation, eligible subset selector,
  cycle integration과 necessary projection/presentation. Generic any-awakened gate와 fallback을 제거한다.
- **Migration/rollout:** V3 state cutover가 gate다. Closed `GRB-PD-005`에 따라 new RON field/load와 generic
  runtime을 atomic하게 내고 current one-step WhiteNight를 유일한 live development bootstrap으로 유지한다.
  이 fixture는 generic eligibility/DAG/cycle/failure와 all-current-FinalBoss record를 검증하지만 최종
  WhiteNight content authority는 아니다.
- **Safety net:** no/owned/awakened prerequisite matrix, locked-cycle starvation, dependency cycle/unreachable
  rejection, next-Floor-only activation, checkpoint and Core/Unity presentation tests.
- **Score:** A2 B2 C2 D2 E2 F3 = 13 → difficulty 4. Risk high, confidence medium. PD005 bootstrap과 strict V3
  rejection은 closed이며 final chain content는 별도 future Goal이다. `GRB-WP-001`. Readiness
  `ready_with_prerequisite`.

## GRB-M-012 — FinalBoss Chain Cycle

- **Claim/disposition/acceptance:** `approved_target`. 모든 일반 chain 승리 뒤에만 refill하고, victory가
  reward/chain completion/exit gate/profile candidate를 commit하되 RunComplete/Floor advance를 만들지 않는다.
- **Current path/live data:** BossOmen state는 completed cycle을 저장하지 않고 victory 뒤 active state만
  clear한다. 후보가 비면 skipped를 무시해 즉시 재사용한다. Live one-chain behavior는 general cycle proof가
  아니다.
- **Boundary/work type:** unified ledger의 chain membership/completion/cursor, eligible selector/refill,
  FinalBoss reward/exit transaction과 global profile event seam. Hidden bosses는 membership에서 제외한다.
- **Migration/rollout:** Closed V3/source identity와 PD004 bootstrap reward fixture를 사용한다. PD005의
  one-step WhiteNight live bootstrap으로 production one-chain E2E를 검증하고 generic multi-chain fixtures로
  multi-chain state machine을 보강한다. Global
  durable publish 자체는 M004 package가 소유하며 V4 ledger는 checkpoint와 함께 rollback한다.
- **Safety net:** multiple-chain eligibility/order/refill, victory/defeat/retreat/load, no auto completion,
  reward late failure, exit gate and monotonic all-boss profile candidate tests.
- **Score:** shared ledger foundation 가정 A2 B2 C2 D1 E1 F3 = 11 → difficulty 4. Risk critical,
  confidence medium. Initial reward fixture is closed by `GRB-PD-004`, while final content is deferred;
  cutover/source identity/live-chain defer are
  closed by `GRB-PD-001/003/005`; `GRB-WP-001`. Readiness `ready_with_prerequisite`.

## GRB-M-013 — BossOmen Source Failure

- **Claim/disposition/acceptance:** `approved_target`. Failed provisional source는 candidate 뒤로, failed active
  source는 same step을 유지하며 node 내부 재시도/reward/step progress/RunComplete가 없다.
- **Current path/live data:** Event failure final advance와 exhausted Combat defeat가 ordinary node completion을
  만들고 outcome-blind `consume_completed_source`가 step을 성공 처리한다. Forced FinalBoss defeat만 target과
  일치하는 run failure 특례다.
- **Boundary/work type:** Event-started battle provenance와 typed source resolution(success/failed/unvisited),
  queue/active transition, node/Gate integration을 M012 state machine에 coimplement한다. 새 live content나
  public result shape는 필수 아님이다.
- **Migration/rollout:** V4 ledger/state cutover를 공유한다. Temporary old consume path나 success fallback은
  금지한다.
- **Safety net:** Event/Combat × provisional/active × victory/exhausted/retreat/unvisited matrix,
  checkpoint/retry, no partial reward/step and existing forced-boss failure regression.
- **Score:** A2 B3 C2 D1 E0 F3 = 11 → difficulty 4. Risk high, confidence high. Strict V3 rollout is fixed by
  closed `GRB-PD-001`; `GRB-WP-001`. Readiness `ready_with_prerequisite`.

## GRB-M-014 — Permanent Monolith Failure

- **Claim/disposition/acceptance:** `approved_target`. 두 attempt exhausted 뒤 해당 stage가 terminal failed,
  재변환/fragment/mastery/residual이 없고 checkpoint는 transaction 전체를 rewind한다.
- **Current path/live data:** exhausted failure는 active node만 clear해 uses가 남으면 same stage를 다시
  manifest할 수 있다. Serialized progress에는 failed stage가 없다.
- **Boundary/work type:** unified encounter/Monolith state에 terminal failed transition과 conversion gate,
  validation/projection을 추가하고 old reusable-failure path를 제거한다.
- **Migration/rollout:** V3 policy를 공유한다. New save field/state 때문에 strict cutover and checkpoint
  invariants가 필요하다.
- **Safety net:** attempt 1/2, retreat, exhausted, conversion rejection, reload before/after, floor clear와
  reward/mastery non-mutation.
- **Score:** A2 B2 C2 D1 E0 F2 = 9, 기본 difficulty 3이나 save migration minimum으로 **4 override**.
  Risk high, confidence high. Strict V3 rejection is fixed by closed `GRB-PD-001`; `GRB-WP-001`. Readiness
  `ready_with_prerequisite`.

## GRB-M-015 — Replace 29 Placeholder Fragments

- **Claim/disposition/acceptance:** `future_or_dormant`. Dedicated player-selected active skill은 formal 장기
  정책이지만 룰북은 이번 correction에서 29개 live placeholder conversion을 명시적으로 제외한다.
- **Current path/live data:** production은 BasicAttackModifier 29, ActiveSkill 2다. ActiveSkill schema,
  validation, equip, upgrade/awakening selection, battle targeting/activation과 Unity seam은 이미 live다.
- **Boundary/work type:** 29개 개별 design, skill/fragment RON, compatibility/balance, battle tests와 Unity
  presentation/assets의 vertical content batches다. 승인 design이 existing primitive를 넘을 때만 runtime/
  DTO/save extension을 추가한다. Generic/no-op 변환은 acceptance가 아니다.
- **Migration/rollout:** current wave는 placeholder behavior와 IDs를 유지한다. 별도 future Master가
  승인되면 fragment ID/state를 보존할 수 있는 data cutover를 기본으로 하되 effect behavior/assets를
  coordinated publication하고 마지막 zero-placeholder audit를 수행한다.
- **Safety net:** fragment별 cast/target/effect/upgrade/awakening/compatibility deterministic tests,
  catalog reference/assets, Unity presentation, save-equipped fragment replay와 broad battle regression.
- **Score:** A0–3 B1–3 C0–2 D1–3 E3 F3 = 8–17 → future provisional difficulty 3–5. Risk high,
  confidence low; future activation 시 승인 design batch별 read-only capability audit/spike가 필요하다.
  `GRB-PD-007` is closed by canonical defer; `GRB-WP-005` is `future_or_dormant`. Current readiness
  `future_or_dormant`.

## Cross-Finding Assessment Conclusions

- `GRB-M-002/003/005/008/009/010/011/012/013/014`는 하나의 run-state/save/result authority cutover에서
  competing state 없이 함께 닫혀야 한다. Individual score를 합산하지 않는다.
- `GRB-M-006`은 독립적으로 가장 작은 correction이지만 같은 live `run/policy.ron`을 큰 cutover도
  편집하므로 병렬 작업은 안전하지 않다.
- `GRB-M-001/004`는 FinalBoss completion seam 뒤 global profile과 Standard를 한 durable package로
  완성해야 profile schema placeholder 또는 두 번 migration을 피한다.
- `GRB-M-007`은 ledger 이후 별도 public vertical slice다.
- Unity runner는 unavailable이다. 모든 Unity verification 요구는 future completion gate이며 fresh pass로
  오인하지 않는다.
