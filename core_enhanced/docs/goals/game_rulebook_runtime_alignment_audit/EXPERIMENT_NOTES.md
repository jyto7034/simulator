# Game Rulebook Runtime Alignment Audit Notes

## Initial structural decision

- Goal name: `game_rulebook_runtime_alignment_audit`
- Goal type: single
- Execution mode: `investigation_only`
- Production mutation: not authorized
- Audit finding ID: 없음

룰북은 여러 gameplay domain을 포함하지만 이번 작업의 결과는 하나의 updated canonical에 대한
read-only alignment matrix와 mismatch catalog다. 수정·migration을 원자적으로 닫는 작업이 아니므로
초기에는 단일 감사 Goal로 유지한다. 조사 결과 독립 범위 때문에 하나의 completion verdict를 안전하게
낼 수 없으면 production 작업 없이 `scope_decision_required`로 전환한다.

## Baseline and dirty worktree

- Repository root: `/mnt/f/work/simulator`
- Start commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Goal path collision: 없음
- Initial working tree: broad dirty state
- `docs/game_rulebook.md`, adjacent canonical, 다수 Core source/test, `../game_resources/data`,
  `../game_server`가 이미 변경돼 있었다.
- 이 변경은 사용자 소유이며 수정, 정리, restore, stash 또는 formatter 대상으로 삼지 않는다.

감사의 대상은 clean `HEAD`가 아니라 사용자가 업데이트한 현재 working tree다. 실행 중 tree가 바뀌면
명령별 timestamp/commit/status evidence를 기록하고, 같은 claim의 양쪽 evidence가 서로 다른 generation에
속하지 않는지 재확인한다.

Execution baseline (2026-08-12 04:05:28 +09:00):

- Simulator commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Rulebook SHA-256: `c11a09402febbb6b16dcc98a11dd1364bac8dfe53645627f1670e6fbd7380f1e`
- Unity commit: `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`
- 외부 `unity_core_contract.md`와 `core_unity_battle_transport_contract.md`: readable

## Source-of-truth distinction

- 룰북은 gameplay canonical이지만 실제 runtime/live data보다 우선하는 구현 증거는 아니다.
- 룰북 안에는 현재 규칙, 승인됐으나 구현 대기인 목표 정책, future/dormant 후보와 다른 canonical로
  위임된 세부가 함께 존재한다.
- 따라서 단순한 text-to-symbol diff가 아니라 각 문장의 lifecycle tag와 production reachability를 먼저
  정해야 한다.
- 문서 선행이 명시된 정책은 예상된 상태라도 정합성 관점에서는 `document_ahead_expected` finding으로
  투명하게 남긴다. 이를 구현 defect 또는 correction 승인으로 확대하지 않는다.

## Planned evidence discipline

- line number는 보조 anchor로만 사용하고 heading + stable claim ID를 주 식별자로 사용한다.
- code symbol, enum variant 또는 RON field 존재만으로 live behavior를 입증하지 않는다.
- test fixture와 admin-only path는 production live path와 별도로 표시한다.
- live data는 loader가 실제 읽는 root와 schedule/pool/assignment까지 연결한다.
- serialized 계약은 producer, server mapping과 접근 가능한 consumer를 각각 확인한다.
- external Unity source를 읽지 못하면 Core/server 일치만으로 전체 consumer 일치를 주장하지 않는다.

## External dependencies

- 룰북은 `/mnt/f/unity projects/ark/docs/unity_core_contract.md`와
  `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`를 외부 canonical로 지목한다.
- 두 canonical과 current Unity consumer/test source는 읽을 수 있었고 audit generation의 Unity commit을
  기록했다.
- Unity runner는 shell에서 찾을 수 없어 fresh Unity test는 실행하지 않았다. 이 제한은 source access
  blocker가 아니며 catalog/completion audit에 명시했다.

## Applied alternatives

- Master/Phase 구조는 초기 선택에서 적용하지 않았다. 이번 요청은 구현 campaign이 아니라 단일
  canonical의 read-only mismatch catalog라는 하나의 결과를 가진다.
- 기존 리팩토링 audit 결과를 그대로 재사용하지 않는다. 룰북과 working tree가 업데이트됐으므로
  현재 generation의 evidence를 다시 연결해야 한다. 기존 audit은 탐색 색인으로만 사용할 수 있다.

## Validation omitted at initialization time

- repository-wide code search: 신규 Goal 문서 초기화 전 금지되어 그 시점에는 수행하지 않음; 이후 완료
- tests/probes: 초기에는 claim inventory와 focused boundary가 정해지지 않아 수행하지 않음; 이후 Rust
  1,126개 test 완료
- external consumer inspection: 초기에는 수행하지 않음; 이후 current Unity source/assertion inspection 완료

## Remaining risks

- broad dirty worktree가 실행 중 추가 변경되면 audit generation이 섞일 수 있다.
- 룰북의 폭이 넓어 claim granularity를 너무 크게 잡으면 부분 불일치를 놓칠 수 있다.
- 명시적 implementation-pending 정책과 실제 stale documentation을 섞어 분류할 위험이 있다.
- dead code, fixture-only code와 production live path를 혼동할 위험이 있다.
- 외부 Unity consumer가 접근 불가능하면 일부 end-to-end 계약은 완전히 닫을 수 없다.

## Out-of-scope follow-up candidates

아직 없음. 실제 감사에서 발견된 correction 후보는 자동으로 구현하거나 새 Goal로 만들지 않고
별도 후보로만 기록한다.

## Confirmed preliminary source-of-truth conflicts

아래 항목은 symbol 존재만이 아니라 현재 production 경로와 live data 연결을 확인한 preliminary
finding이다. 최종 ID, claim coverage와 영향은 `MISMATCH_CATALOG.md`에서 닫는다.

### `GRB-M-001` — Standard fixed tutorial policy is not the live generator contract

- Rulebook: Standard는 세 Floor의 고정 map/progression, 고정 tutorial RON 난이도와
  `Floor 1 Normal → Floor 2 Elite → Floor 3 NormalBoss`를 사용한다.
- Runtime/data: `node_assignment_profiles.ron`은 Standard Floor 1 terminal guard를
  `combat_elite`로 고정하고 Floor 3 category를 weighted assignment한다.
  `FacilityMapGenerator::generate_with_requirements`는 Standard에도 seeded assignment와 공통
  `assign_combat_difficulties`를 적용한다.
- Runtime: `generate_current_floor_map`은 Standard에도 certified power/run peak 기반 floor anchor를
  manifest한다. `encounter_class_for_map_node`는 final Standard Boss를 `FinalBoss`로 고른다.
- Direction: `document_ahead_expected` + `runtime_contradiction`.

### `GRB-M-002` — Removed research and pending fragment delivery remain authoritative

- Rulebook: 기본/환상체 연구도, `research_progress`, `skill_fragment_progress`,
  `pending_research_deliveries`와 본사/계획 복귀 수령 절차를 모두 폐기한다.
- Runtime: `RunState.abnormality_research`, `RunAbnormalityResearchState`, planning-return research
  delivery와 `SkillFragmentResearchDiffDto`가 active result/checkpoint 경로에 남아 있다.
- Live data: `run/policy.ron.abnormality_research`, abnormality의 단일
  `response_complete_skill_fragment_id`, reward의 `GrantSkillFragmentResearch`가 production bundle에 있다.
- Consumer: Unity `ResearchDeliveryNotificationModel`과 current EditMode tests가
  `research_deliveries`를 읽는다.
- Direction: `legacy_live_path` + `producer_consumer_mismatch` relative to updated rulebook.

### `GRB-M-003` — Endless still completes when all abnormality responses are researched

- Rulebook: Endless에는 자동 `RunComplete`가 없고 모든 FinalBoss/숨겨진 보스를 격파해도 계속된다.
- Runtime: `plan_player_victory_combat_result_completion` computes
  `research.all_response_complete()` and `commit_planned_player_victory_combat_result_completion`
  transitions to `GameState::RunComplete` when true.
- Test: `world/tests/combat.rs` explicitly expects final Endless response completion to immediately
  complete the run.
- Direction: `runtime_contradiction` and `test_contract_mismatch`.

### `GRB-M-004` — Global tutorial unlock and personal records have no production profile authority

- Rulebook: `standard_tutorial_completed` gates Endless and the global profile owns three Endless
  personal records.
- Runtime/server search: the only game-global gameplay profile is
  `EncounterCompendiumProfileState`; no tutorial-completion flag or personal-record fields/mutations exist.
- Unity: mode selection always exposes both typed game modes and has no profile-gated start path.
- Direction: `document_ahead_expected`.

### `GRB-M-005` — Elite pool and natural failure semantics still use the old exclusion/weighting model

- Rulebook: an exhausted natural Elite failure stays in the unmastered pool; repeat pool activates
  only after every identity has a natural victory and then cycles each identity once.
- Runtime: `record_natural_failure` stores `NaturalEliteOutcome::Failed`,
  `is_naturally_excluded()` treats both victory and failure as resolved, and `assign_map_encounters`
  removes these identities. The current repeat selector uses recent-floor exclusion and
  research-completion weights rather than the authored unmastered/mastered cycle.
- Test: `endless_resolved_elite_pool_falls_back_to_authored_ordinary_combat` explicitly protects
  natural-failure exclusion.
- Direction: `runtime_contradiction` + `test_contract_mismatch`.

### `GRB-M-006` — Monolith stage tuning differs from the approved rulebook values

- Rulebook: Stage 2/3 stat weights are 150%/300%; exact composition remains authored.
- Live data: `run/policy.ron` currently sets stage stat multipliers to 100%/125% and reward
  multipliers to 100%/125%.
- Direction: `live_data_gap`.

### `GRB-M-007` — Mastered Elite sweep has no live command/result path

- Rulebook: mastered Elite nodes offer direct battle or sweep, and sweep produces an unconditional
  victory CombatResult without battle setup/statistics.
- Runtime/data/Unity search: no sweep command, capability, result marker, mastery-based node option
  or consumer path exists.
- Direction: `document_ahead_expected`.

## Final finding boundaries

Preliminary seven findings were retained and expanded into the final 15-item catalog. The additional
boundaries are:

- `GRB-M-008`: target encounter-grade `fragment_id`/`repeat_reward` authority is absent; this is
  separate from the old research mechanism still being live (`GRB-M-002`).
- `GRB-M-009`: `default_treasures` makes direct fragments reachable from a general Reward node, not just
  present as unused reward metadata.
- `GRB-M-010`: NormalBoss has no consume/return/refill cycle state and uses the generic old weighted
  selector.
- `GRB-M-011`: BossOmen chain data cannot author per-chain fragment eligibility and runtime instead
  imposes a global any-awakened-fragment gate.
- `GRB-M-012`: forced-boss victory clears active state without a completed-chain/cycle ledger. With the
  single current live chain, immediate later selection is also compatible with a new one-member cycle;
  the confirmed gap is the missing durable cycle transaction, not a claim that today's one-chain order
  visibly differs.
- `GRB-M-013`: generic Event combat retry and generic node completion are contrary to approved failure
  semantics. Current live Event/BossOmen RON does not activate the source-combat branch, so this is
  `document_ahead_expected`, not a claim of a currently triggerable player defect.
- `GRB-M-014`: exhausted Monolith remanifestation failure clears only `active_node_id`; remaining uses
  make the same stage usable again instead of permanently failing the identity.
- `GRB-M-015`: the rulebook itself discloses the 29 direct `BasicAttackModifier` placeholders. They are
  retained as an expected target/live difference rather than treated as an undocumented regression.

## Classification decisions

- Explicit exact follow-up values (`fragment_id`, reward quantity, BossOmen minimum Floor) were not
  counted as missing current behavior. The absent authority/trigger is a finding; an intentionally
  un-authored content value is not.
- Hidden-boss implementation was classified `future_or_dormant` because the Monolith section explicitly
  says its consumer/gameplay implementation is follow-up. The later target policy does not override that
  lifecycle statement for this audit.
- The current minimal `white_night` BossOmen RON was treated as live evidence for generic BossOmen
  machinery, not as proof that the rulebook's future detailed WhiteNight design has been approved.
- Unity-local classes and semantic paths named `HeadquartersSupport*` select the current typed HQ actions
  (`RecruitEmployee`, `RequestEmergencySupplies`, `OpenHeadquartersShop`). They are not the removed Core
  `choose_support` wire/game mode, which the server explicitly rejects.
- Tactical tile `Rest` is also a different domain from the removed noncombat `Rest` node/mode and was
  not marked legacy.

## Executable evidence interpretation

- Core lib: 1,012 passed.
- Core integration targets: 51 passed.
- Server lib: 63 passed.
- Total fresh Rust tests: 1,126 passed, 0 failed, 0 ignored.

The green result confirms the inspected working tree is executable and current tests pin several old
contracts. It does not turn a rulebook contradiction into alignment. In particular, current tests
explicitly expect Standard FinalBoss selection, response-complete Endless termination, Elite hard
exclusion/weighting, global awakened-fragment BossOmen activation and Event combat retry.

The external Unity tree was readable and current consumer/test source was inspected. No Unity runner was
available in the shell, so no fresh Unity test count is claimed.

## Final external and legacy boundary

- No required source boundary is inaccessible: Core, live data, server and current Unity source are all
  readable.
- Unity execution is a validation limitation, not a source access blocker. Findings relying on Unity are
  also proven by explicit current reader/command code and Core/server authority.
- `timeline_delta`/`battle_delta` are absent from the current live transport source and server tests
  reject legacy battle transport.
- Removed Core `ChooseSupport` does not deserialize.
- No production or canonical correction was made; all mutations remain confined to this Goal directory.

## Out-of-scope follow-up candidates

The 15 mismatch findings are correction candidates only. This audit does not select whether each should
change documentation, Core, live data, server/Unity, tests or multiple owners, and it does not create
implementation Goals automatically.
