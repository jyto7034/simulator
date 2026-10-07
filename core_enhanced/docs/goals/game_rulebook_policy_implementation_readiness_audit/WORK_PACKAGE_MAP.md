# Work Package Map

Finding 번호가 아니라 authority, persistence lifetime과 원자 완료 조건으로 15개 finding을 5개 package에
묶는다. Package difficulty는 individual score 합계가 아니라 실제 end-to-end cutover를 다시 평가한 값이다.

## Exact Finding Coverage

| Package | Findings | Scope status | Relationship summary |
| --- | --- | --- | --- |
| `GRB-WP-001` | `GRB-M-002`, `GRB-M-003`, `GRB-M-005`, `GRB-M-008`, `GRB-M-009`, `GRB-M-010`, `GRB-M-011`, `GRB-M-012`, `GRB-M-013`, `GRB-M-014` | current | core prerequisite; internal coimplementation |
| `GRB-WP-002` | `GRB-M-006` | current | semantically independent; serialize shared RON edit |
| `GRB-WP-003` | `GRB-M-001`, `GRB-M-004` | current | prerequisite `GRB-WP-001` |
| `GRB-WP-004` | `GRB-M-007` | current | prerequisite `GRB-WP-001` |
| `GRB-WP-005` | `GRB-M-015` | future_or_dormant | separate future Master only; if activated, prerequisite `GRB-WP-001` |

Coverage audit: M001~M015 each appear exactly once; duplicates 0, omissions 0. `WP-005`는 exact finding
coverage를 위해 registry에 남기되 current implementation sequence 밖의 `future_or_dormant` node다.

## GRB-WP-001 — Run-Local Encounter Progression And Fragment Economy V4

- **Findings/decisions:** M002, M003, M005, M008, M009, M010, M011, M012, M013, M014. All decision inputs are
  closed: `GRB-PD-004` supplies the minimum runnable 29-row bootstrap and repeat dust 5/10/20;
  `GRB-PD-001/003/005/008/010` supply strict-cutover/source-identity/live-chain-bootstrap/Elite-fill/
  sweep-reacquisition inputs.
- **Completion result:** Research/pending delivery가 전 계층에서 제거되고 Elite/NormalBoss/FinalBoss/
  BossOmen/Monolith가 하나의 run-local source/cycle ledger를 사용한다. First/repeat reward, mastered-sweep
  reacquisition, failure, mastery, unmastered-first mixed fill, pool return/refill와 no Endless auto-completion이
  V4 save/result에서 observable하게 일치한다.
- **Current→target authority:** RunState research + Monolith natural flags + partial BossOmen state + generic
  assignment/ordinary reward를 `EncounterProgressionLedger` 계열의 단일 typed authority와 source-aware
  victory/failure transaction으로 교체한다. Exact type/schema 이름은 구현 Goal이 정하되 authority는 하나다.
- **Shared foundation/legacy removal:** source identity/membership, assigned/resolved cycle, first/repeat/mastery,
  BossOmen provisional/active/forced state와 Monolith terminal failure를 한 strict checkpoint invariant로
  만든다. Old research policies/types/results/Unity reader, response weighting, outcome-blind source consume,
  hard exclusion과 old Monolith unique-fragment special path를 같은 cutover에서 제거한다.
- **Included boundaries:** Core RunState/map selection/node/Event/combat/economy/snapshot/behavior; PVE,
  BossOmen, run/reward RON and loaders; V4 checkpoint/active-run store; game_server result/order; Unity strict
  reward/research/omen consumers and fixtures.
- **Relations:** Internal findings are `coimplement`; M008 is acquisition foundation, M005/M010/M012 are
  assignment consumers, M013/M014 are failure transitions, M002/M003/M009 are legacy removal. External
  prerequisite 없음. PD004 bootstrap is an initial implementation fixture, not an open gate or final content
  authority. Produces stable ledger/result/profile-event
  seam for WP003 and mastered eligibility/result seam for WP004.
- **Cutover:** Strict atomic release. No V3/V4 dual read/write, one-shot semantic transform, old/new research
  acquisition coexistence, placeholder chain schema or missing-content fallback. Closed PD001 fixes V3 as explicit
  incompatible-version rejection; closed PD003 fixes authored stable source keys; closed PD005 keeps the current
  one-step WhiteNight as the sole live development bootstrap while retaining fixture-tested generic machinery and
  deferring final chain content.
- **Test-first/completion gate:** Approved V4 state-transition table와 invalid-data/migration fixtures를 먼저
  고정하되 old production과 동시에 merge되는 failing placeholder test는 만들지 않는다. Completion requires
  class/source outcome matrix, checkpoint/restart/reconnect, result late-failure atomicity, server/Unity contract,
  old symbol/field/live entry zero search and broad Core/server/Unity regressions.
- **Package score:** A3 B3 C3 D3 E3 F3 = 18 → difficulty 5. Risk critical, confidence medium. Main risks are
  save loss, double/omitted reward, partial commit and cycle deadlock; runtime implementation input은 닫혔지만
  final content selection은 의도적으로 별도 Goal에 남는다.
- **Goal boundary:** Master 권장. Bootstrap fixture와 migration policy는 이 audit에서 닫혔다. Final content
  manifest는 별도 content Goal이 소유한다. Production Goal의 Phase 0은 fixture/preflight 확인, Phase 1은 Core/data/save/server/Unity를 한 completion boundary에서 닫는 atomic
  vertical cutover다. Phase 1을 여러 partially-live schema Goal로 분할하지 않는다.

## GRB-WP-002 — Monolith Initial Stat Tuning

- **Findings/decisions:** M006; open decision 없음.
- **Completion result:** Live Stage 2/3 stat multiplier initial value가 150%/200%이고 loader/tests가
  positive/ordered/safe-range invariant와 derived combat tuning을 보호한다. Values는 개발 중 RON tuning으로
  조절할 수 있으며 reward multiplier는 현 정책을 유지한다.
- **Current→target authority:** Existing `RunPolicyData.monolith` data values/validation만 correction한다.
  새 state, producer/consumer와 migration은 없다.
- **Shared foundation/legacy removal:** Existing role tuning/rounding을 그대로 사용하며 permissive
  positive/increasing validation에 safe-range protection을 더한다. Initial 150/200을 loader constant로
  영구 고정하지 않는다.
- **Relations:** Semantically `independent` of WP001/003/004/005. 단 WP001도 same live
  `game_resources/data/run/policy.ron`과 loader tests를 수정하므로 동일 working tree에서 병렬 실행하지 않는다.
- **Included boundaries:** live run policy, loader validation, focused difficulty tests. Server/Unity/save는
  non-applicable because no shape or projected exact multiplier changes.
- **Cutover:** Strict data replacement; compatibility/fallback 불필요.
- **Test-first/completion gate:** initial live data load, invariant rejection, Stage 2/3 all derived stat axes,
  rounding and reward-unchanged assertions, broad combat difficulty tests.
- **Package score:** A0 B0 C0 D0 E1 F2 = 3 → difficulty 1. Risk low, confidence high.
- **Goal boundary:** Single Goal. WP001보다 먼저 완료해 shared-file merge conflict를 제거하는 것을 권장한다.

## GRB-WP-003 — Unified Global Profile And Fixed Standard Tutorial

- **Findings/decisions:** M001, M004. Closed `GRB-PD-006` supplies minimum bootstrap `ST1-NT60`; closed `GRB-PD-002`
  fixes existing compendium-set preservation.
- **Completion result:** Standard는 authored fixed three-Floor tutorial이고 F3의 boss-only 60% Nothing There
  NormalBoss victory가 tutorial flag와 run completion을 원자 commit한다. F3 target은 HP/ATK/DEF/MR
  exact actor override `720/53/36/33`이며 다른 ST1 actor/node/reward와 다른 Nothing There 출현은 100%다.
  Generic encounter-wide three-axis stat scale로 이를 구현하지 않는다. Endless gate와 permanent highest/deathless/all-FinalBoss records가
  separate monotonic profile에서 즉시 durable하며 Unity가 eligibility/records를 표시한다.
- **Content authority limit:** `ST1-NT60`의 topology/encounter/tuning/reward는 initial runnable fixture다. Final
  Standard content는 별도 Goal이 fixed three-Floor/no-adaptive/F3 completion+profile invariant를 유지하면서
  교체·완성한다.
- **Current→target authority:** Shared generated Standard와 compendium-only V1을 mode-specific authored
  Standard authority + unified/adjacent V2 gameplay profile reducer/store로 교체한다. Profile record input은
  Core result에서 typed event로 나오고 host publish/acknowledgement가 completion을 소유한다.
- **Shared foundation/legacy removal:** WP001 FinalBoss cycle completion event를 all-boss input으로 사용한다.
  Ungated Endless, F3 FinalBoss/adaptive Standard assertions와 compendium-only persistence bootstrap을 제거/
  migrate한다. No profile placeholder before all record semantics exist.
- **Included boundaries:** Standard map/script RON/loaders, Core start capability/generator/result/profile DTO,
  active-run interaction, profile V2 current/backup/reset/retry, game_server durable ordering/reconnect, Unity menu/
  record/disabled reason and fixtures.
- **Relations:** `prerequisite WP-001` for M012 all-boss identity/event. WP004와 semantic independent지만
  behavior/action/snapshot/server/Unity contract files를 공유해 parallel-safe가 아니다.
- **Cutover:** Strict coordinated producer-consumer release with bounded explicit V1→successor preservation.
  Dual store/dual publish, forced profile reset와 generated-Standard fallback 없음.
- **Test-first/completion gate:** strict-valid runnable bootstrap tutorial data, profile state/publish failure table,
  V1/current/backup/corruption/restart, flag/result atomicity, checkpoint independence, all records, Core gate,
  server ordering, Unity menu/read and broad map/save/profile regressions.
- **Package score:** A3 B3 C3 D3 E3 F3 = 18 → difficulty 5. Risk critical, confidence medium.
- **Goal boundary:** Master 권장. Bootstrap/persistence preflight를 Phase 0으로 닫고, authored Standard +
  Core/profile/server/Unity production cutover를 하나의 atomic Phase로 구현한다.

## GRB-WP-004 — Mastered Elite Sweep Vertical Slice

- **Findings/decisions:** M007; consumes closed `GRB-PD-003` source identity, closed `GRB-PD-004` live mappings and
  closed `GRB-PD-010` sweep reacquisition policy.
- **Completion result:** Mastered Elite node에서 직접 전투/소탕 선택이 보이고, 소탕 result가 no battle/
  condition/record/bonus mutation을 보장하면서 base reward + fragment reacquisition 또는 repeat reward 중
  하나, alive XP, duration, cycle/node completion을 atomic commit한다. Dismantle→reacquire cycle은 허용한다.
- **Current→target authority:** No current command에서 WP001의 mastered/source ledger를 소비하는 typed
  sweep planner/action/result authority를 추가한다. Existing economy diff/XP primitives는 재사용하지만
  CombatBattleState/event-log authority는 호출하지 않는다.
- **Shared foundation/legacy removal:** WP001 source reward planner, mastered eligibility와 cycle cursor가
  prerequisite다. Fake battle/fabricated record나 unconditional generic CompleteNode fallback은 만들지 않는다.
- **Included boundaries:** Core action/capability/preview/item duration/planner/result/node completion;
  applicable item/live data validation; game_server behavior/result; Unity choice/disabled reason/result UI;
  save/checkpoint ledger transaction.
- **Relations:** `prerequisite WP-001`. WP003과 semantic independent지만 public behavior/snapshot/server/Unity
  boundary를 공유해 같은 branch의 parallel implementation은 unsafe다.
- **Cutover:** Strict additive action/result coordinated release. Old consumer ignores-new-field 전략이나
  server fallback 없이 Core/server/Unity를 함께 낸다.
- **Test-first/completion gate:** eligibility and atomic-failure matrices, explicit absence of battle/session/log/
  condition/discovery mutation, reward/XP/duration/cursor, checkpoint/reconnect, server/Unity fixtures and broad
  combat/economy/action capability regression.
- **Package score:** A2 B3 C2 D3 E1 F3 = 14 → difficulty 4. Risk high, confidence medium.
- **Goal boundary:** Single vertical Goal after WP001.

## GRB-WP-005 — Dedicated Fragment Skill Content Campaign

- **Findings/decisions:** M015; closed `GRB-PD-007` places it outside the current correction. A future Master still
  requires per-batch approved design records.
- **Completion result:** Activated scope의 각 fragment가 dedicated active skill semantics/RON/Core tests/Unity
  presentation/assets를 갖고, final campaign에서 production BasicAttackModifier placeholder count가 zero다.
- **Current→target authority:** Existing two-skill ActiveSkill framework를 default로 재사용하고 29 placeholder
  effect/content를 approved batches로 교체한다. Design이 요구할 때만 framework/DTO를 확장한다.
- **Shared foundation/legacy removal:** Generic/no-op shared skill을 foundation으로 삼지 않는다. 각 batch가
  production-complete해야 하고 final phase에서 old placeholder definitions/descriptions를 제거한다.
- **Included boundaries:** skill/fragment live catalogs and validation, any needed Core battle primitives,
  deterministic per-skill tests, snapshot/catalog, Unity descriptions/ranges/targeting/VFX/audio/assets and QA.
- **Relations:** 현재 `future_or_dormant` node. 사용자가 별도 future Master를 명시적으로 활성화하면 WP001
  뒤에 두어 acquisition/BossOmen fragment identity와 catalog/Unity edits를 안정화한다. WP002/003/004와
  직접 semantic prerequisite는 없지만 broad skill/battle/Unity files conflict may require serialization.
- **Cutover:** Batch별 strict content publication. Schema-only placeholder, missing asset fallback와 incomplete
  design publication 없음. Existing equipped IDs를 보존하는 것을 기본 rollout assumption으로 두되 design이
  ID/state migration을 요구하면 별도 decision을 만든다.
- **Test-first/completion gate:** approved design→capability matrix, reference/asset coverage, cast/target/effect/
  upgrade/awakening/compatibility, equipped-save replay, Unity runner, broad battle and final zero-placeholder audit.
- **Package score:** provisional A0–3 B1–3 C0–2 D1–3 E3 F3 = 8–17 → difficulty 3–5. Risk high,
  confidence low. Each high-difficulty batch requires a read-only capability audit before implementation.
- **Goal boundary:** Current wave에는 Goal을 만들지 않는다. Future activation 요청이 생기면 Master + approved
  vertical content batches를 새로 author하고, formal fragment policy를 바꾸려면 별도 rulebook correction
  Goal이 먼저 필요하다.

## Relationship And Atomicity Conclusions

- `WP001` 내부 dependency cycle은 competing acquisition/cycle/failure/save authorities를 피하려고 하나의
  coimplementation package로 합쳤다.
- `WP002`는 semantic dependency가 없지만 shared RON edit conflict 때문에 sequence에서 먼저 직렬화한다.
- `WP001 -> WP003`은 FinalBoss completion/profile record authority edge다.
- `WP001 -> WP004`는 mastered/source ledger producer→sweep consumer edge다.
- `WP005`는 canonical-deferred future node이며 별도 활성화 시 stable fragment identity/catalog를 위해
  `WP001 -> WP005`다.
- 어떤 package도 다음 package를 위해 unused schema/DTO/state를 먼저 publish하지 않는다.
