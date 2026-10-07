# Game Rulebook Runtime Mismatch Catalog

This catalog contains only confirmed rulebook/runtime mismatches for the audit generation recorded in
`RULEBOOK_CLAIM_MATRIX.md`. It does not choose whether documentation, code or live data should be
changed. No correction was performed.

## Summary

| ID | Taxonomy | Direction | Short result |
| --- | --- | --- | --- |
| `GRB-M-001` | `document_ahead_expected` + `runtime_contradiction` | document-ahead and live behavior opposite | Standard still uses generated/shared difficulty and a FinalBoss endpoint. |
| `GRB-M-002` | `legacy_live_path` + `producer_consumer_mismatch` | legacy retained end-to-end | Research progress and pending fragment delivery remain authoritative through Unity. |
| `GRB-M-003` | `runtime_contradiction` + `test_contract_mismatch` | runtime opposite | Completing all responses still completes Endless. |
| `GRB-M-004` | `document_ahead_expected` | document-ahead | Tutorial unlock and three permanent records have no profile authority. |
| `GRB-M-005` | `runtime_contradiction` + `test_contract_mismatch` | runtime opposite | Elite failure/victory exclusion and weighting replace the new two-pool cycles. |
| `GRB-M-006` | `live_data_gap` | data value mismatch | Monolith Stage 2/3 stat multipliers are 100%/125%, not 150%/300%. |
| `GRB-M-007` | `document_ahead_expected` | document-ahead | Mastered-Elite sweep has no command/result/consumer path. |
| `GRB-M-008` | `document_ahead_expected` + `legacy_live_path` | target schema absent, old authority retained | Encounter-grade fragment/repeat-reward ledger is not modeled. |
| `GRB-M-009` | `runtime_contradiction` + `live_data_gap` | live reward source opposite | A general Reward-node pool can grant skill fragments directly. |
| `GRB-M-010` | `document_ahead_expected` + `runtime_contradiction` | target pool absent, old selector active | NormalBoss uses recent/research weighting rather than a consume/return cycle. |
| `GRB-M-011` | `document_ahead_expected` + `runtime_contradiction` | schema missing and global gate opposite | BossOmen uses “any awakened fragment” globally, not per-chain optional conditions. |
| `GRB-M-012` | `document_ahead_expected` | document-ahead | FinalBoss chain completion/cycle state does not exist. |
| `GRB-M-013` | `document_ahead_expected` + `test_contract_mismatch` | target failure policy absent | Event combat retries and generic node completion would consume failed omen sources. |
| `GRB-M-014` | `runtime_contradiction` | runtime opposite | Exhausted Monolith remanifestation failure does not become permanent. |
| `GRB-M-015` | `document_ahead_expected` | documented placeholder remains | Formal fragment active skills remain 29 direct basic-attack modifiers. |

## Findings

### `GRB-M-001` — Standard is not the fixed tutorial generation/difficulty contract

- **Rulebook claims:** `GRB-L0063`-`GRB-L0072`, `GRB-L0151`, `GRB-L0206`, and
  `GRB-L0617`-`GRB-L0619` require three fixed authored Standard maps, authored fixed combat tuning,
  Floor 1 `Normal`, and a Floor 3 `NormalBoss` that completes the tutorial.
- **Actual authority path:** `GameCore::generate_current_floor_map` initializes and manifests the
  shared certified-power floor anchor for both modes, then calls
  `FacilityMapGenerator::generate_with_requirements` and `assign_map_encounters`. The generator uses
  seeded assignment and shared `assign_combat_difficulties`. `encounter_class_for_map_node` explicitly
  returns `FinalBoss` for a Boss on the final Standard floor.
- **Live data:** `map/node_assignment_profiles.ron` assigns Standard Floor 1 `terminal_guard` to
  `combat_elite`, assigns Floor 2 the same way, and leaves Floor 3 categories to weighted constraints.
  The schedule fixes a template pool/profile identity per floor, but not the complete node/content
  assignment or tutorial encounter identity described by the rulebook.
- **Consumer impact:** Core projects the resulting generated map/difficulty and FinalBoss encounter to
  server/Unity; no consumer-side reinterpretation changes the class.
- **Test evidence:** the passing tests
  `generated_standard_maps_use_gate_before_final_floor_and_boss_on_final_floor` and
  `final_standard_floor_boss_node_selects_final_boss_encounter` protect the current shape.
- **Visible effect:** Standard begins with an Elite terminal encounter, applies adaptive authority, and
  ends with the FinalBoss class rather than the approved tutorial sequence.
- **Classification:** `document_ahead_expected` plus `runtime_contradiction`; direction is
  document-ahead with live behavior explicitly opposite. Confidence: high.
- **Unverified boundary:** exact Unity presentation for every generated seed was not executed; it does
  not affect the Core class/authority conflict.
- **Correction:** none performed.

### `GRB-M-002` — Removed research and pending fragment delivery remain live end-to-end

- **Rulebook claims:** `GRB-P0734-0736`, `GRB-L0757`, and `GRB-L1030` remove abnormality research,
  `research_progress`, `skill_fragment_progress`, `pending_research_deliveries`, and planning/HQ receipt.
- **Actual authority path:** `RunState` owns `abnormality_research`; combat victory calls
  `apply_suppression_victory_with_bonus`; planning-return paths call
  `deliver_pending_research_in_staged_inventory`; result/snapshot DTOs include research diffs and
  deliveries.
- **Live data:** `run/policy.ron.abnormality_research`, every
  `abnormalities/base.ron.response_complete_skill_fragment_id`, and
  `RewardEffect::GrantSkillFragmentResearch` entries in `events/rewards/base.ron` are loaded by the
  production bundle.
- **Producer/consumer impact:** the server test
  `planning_return_preserves_research_deliveries_before_trailing_snapshot` confirms mapping. Unity
  `ResearchDeliveryNotificationModel.FromResultPayload` strictly parses `research_deliveries`, and its
  current EditMode contract tests exercise the field.
- **Test evidence:** Core's passing tests include
  `start_new_game_initializes_abnormality_research_snapshot_from_live_catalog`, planning-return delivery
  tests, and `reward_session_reports_skill_fragment_and_research_diffs`.
- **Visible effect:** victories/reward nodes still advance research and fragments may arrive only after
  returning to planning, exactly the flow the rulebook removes.
- **Classification:** `legacy_live_path`; it is also a producer/consumer mismatch relative to the new
  gameplay contract, not an accidental Core/Unity shape disagreement. Direction: legacy retained
  end-to-end. Confidence: high.
- **Unverified boundary:** Unity tests were inspected but not freshly run because no Unity runner was
  present.
- **Correction:** none performed.

### `GRB-M-003` — Research completion still ends Endless

- **Rulebook claims:** `GRB-L0095`, `GRB-P0114-0116`, `GRB-P0734-0736`, and `GRB-L0798` state that
  Endless has no automatic `RunComplete`.
- **Actual authority path:** `plan_player_victory_combat_result_completion` computes
  `research.all_response_complete()`. `commit_planned_player_victory_combat_result_completion` then
  transitions to `GameState::RunComplete` and returns `BehaviorResult::RunComplete`.
- **Live data:** the production abnormality catalog and research policy provide the completion set and
  thresholds used by this path.
- **Consumer impact:** server result mapping preserves `RunComplete`; there is no Endless-specific
  suppression downstream.
- **Test evidence:** passing test
  `endless_final_response_completion_returns_run_complete_without_planning_delivery` explicitly pins
  the behavior.
- **Visible effect:** an Endless run can terminate automatically when the old response ledger fills.
- **Classification:** `runtime_contradiction` + `test_contract_mismatch`; runtime and test are opposite
  the rulebook. Confidence: high.
- **Unverified boundary:** none material.
- **Correction:** none performed.

### `GRB-M-004` — Tutorial unlock and permanent personal records have no authority

- **Rulebook claims:** `GRB-P0055-0059`, `GRB-L0070`, and `GRB-L0775`-`GRB-L0786` require
  `standard_tutorial_completed`, Endless admission gating, and highest-floor/deathless/all-FinalBoss
  permanent records.
- **Actual authority path:** the only game-global gameplay profile is
  `EncounterCompendiumProfileState`, containing four discovered/defeated ID sets. Core, server
  persistence, and snapshots have no tutorial-completion or personal-record fields/mutations.
- **Live data:** no separate profile schema or persistence document exists for these values.
- **Consumer impact:** Unity `GameCommandBus` accepts either typed mode, and current main-menu
  presentation/tests expose both Standard and Endless commands without a tutorial-profile gate.
- **Test evidence:** Core/server compendium persistence tests pass and demonstrate the narrower current
  profile; Unity `ArkClientContractTests` expects both mode buttons.
- **Visible effect:** a fresh install can request Endless immediately, and the three commemorative
  records are not stored.
- **Classification:** `document_ahead_expected`; direction is document-ahead. Confidence: high.
- **Unverified boundary:** Unity runner unavailable; source and existing assertions are explicit.
- **Correction:** none performed.

### `GRB-M-005` — Elite natural and mastered pools still use the old exclusion model

- **Rulebook claims:** `GRB-L0806`-`GRB-L0821` keep exhausted natural failures in the unmastered pool,
  open mastered repeat assignment only after Stage 3, and consume every mastered identity once per
  deterministic cycle without recent-floor exclusion.
- **Actual authority path:** `MonolithProgress::is_naturally_excluded` is true for both `Victory` and
  `Failed`. `assign_map_encounters` removes every such Elite identity and converts the slot to ordinary
  combat when none remain. Unresolved candidates use `encounter_repeat_weighted_candidates`, which
  applies recent-floor exclusion and old research-completion weights. No mastered-repeat pool ledger
  exists.
- **Live data:** `run/policy.ron.encounter_repeat_weighting` sets recent window `1` and response-complete
  weight `25%` versus incomplete `100%`.
- **Consumer impact:** consumers receive only the already-assigned encounter; no UI-side pool exists.
- **Test evidence:** passing tests
  `natural_resolution_hard_excludes_and_only_victory_unlocks_purchase`,
  `endless_resolved_elite_pool_falls_back_to_authored_ordinary_combat`,
  `endless_repeat_weighting_excludes_recent_abnormality_candidates`, and
  `response_complete_candidates_receive_lower_repeat_weight` protect the old model.
- **Visible effect:** an Elite lost after all attempts never naturally returns, while mastered Elites do
  not enter the new sweep/repeat cycle.
- **Classification:** `runtime_contradiction` + `test_contract_mismatch`; direction is runtime opposite.
  Confidence: high.
- **Unverified boundary:** none material.
- **Correction:** none performed.

### `GRB-M-006` — Monolith Stage 2/3 stat tuning is 100%/125%, not 150%/300%

- **Rulebook claims:** `GRB-L0852`-`GRB-L0855` fix the Stage 2 and 3 stat weights at 150% and 300%.
- **Actual authority path:** remanifestation conversion reads typed stage tuning from RunPolicy and
  manifests that difficulty; there is no later correction to the rulebook values.
- **Live data:** `run/policy.ron` sets `stage_2_stat_multiplier_percent: 100` and
  `stage_3_stat_multiplier_percent: 125` (and reward multipliers 100/125).
- **Consumer impact:** Unity receives only the enhanced/final-enhanced tier, so it cannot display or
  repair the numeric mismatch.
- **Test evidence:** live RunPolicy loading/validation passes with the current numbers.
- **Visible effect:** enhanced Elite stats are substantially below the rulebook multipliers.
- **Classification:** `live_data_gap`; direction is data-only numeric mismatch. Confidence: high.
- **Unverified boundary:** the rulebook intentionally leaves per-stat composition/rounding to RON; this
  finding is limited to the two explicit headline multipliers.
- **Correction:** none performed.

### `GRB-M-007` — Mastered-Elite sweep has no production path

- **Rulebook claims:** `GRB-L0886`-`GRB-L0899` require a mastered-Elite direct/sweep choice, a synthetic
  unconditional victory CombatResult, ordinary/repeat rewards and XP, condition preservation, no
  bonus evaluation, and no battle record.
- **Actual authority path:** exhaustive current-source searches found no sweep behavior/action kind,
  capability, mastery-based node option, synthetic result marker or result-commit branch.
- **Live data:** no sweep policy, encounter flag or authored option exists.
- **Consumer impact:** server and Unity have no sweep command payload or result reader.
- **Test evidence:** no current sweep test exists; the Elite assignment tests instead show no mastered
  repeat node can be assigned (`GRB-M-005`).
- **Visible effect:** the option cannot be presented or executed.
- **Classification:** `document_ahead_expected`; direction is document-ahead. Confidence: high.
- **Unverified boundary:** none; absence was checked across Core, server, live data and accessible Unity
  source.
- **Correction:** none performed.

### `GRB-M-008` — Encounter-grade fragment acquisition and repeat rewards are not modeled

- **Rulebook claims:** `GRB-L0741`-`GRB-L0759`, `GRB-L0794`-`GRB-L0799`, `GRB-L0828`,
  `GRB-L0850`, and `GRB-L1179` require exact encounter `fragment_id`, per-source completion ledgers,
  direct commit on Elite third victory/NormalBoss/FinalBoss victory, and authored `repeat_reward`.
- **Actual authority path:** `PveEncounter` has no `fragment_id` or `repeat_reward`; it has ordinary
  reward UUIDs and the legacy `suppression_research`. Non-Monolith victory feeds abnormality research.
  Stage 3 Monolith is a partial special case: it reads the abnormality's single old
  `response_complete_skill_fragment_id` and grants it once via `unique_fragment_granted`.
- **Live data:** PVE encounters contain no per-grade fragment/repeat contract. Abnormalities each carry
  the old single response-complete fragment field instead.
- **Consumer impact:** CombatResult still projects research diffs/deliveries, not a unified encounter
  acquisition/repeat ledger. BossOmen data also cannot author the required fragment/repeat result.
- **Test evidence:** passing research-completion and Monolith final-stage tests pin the two current
  authorities; no test covers the new grade-trigger/repeat contract.
- **Visible effect:** NormalBoss/FinalBoss victories do not directly grant the rulebook fragment, and
  repeated completed sources have no authored replacement reward transaction.
- **Classification:** `document_ahead_expected` plus `legacy_live_path`; direction is target schema
  absent with old research retained. Confidence: high.
- **Unverified boundary:** exact future fragment IDs/reward quantities are intentionally un-authored;
  their values are not the finding—the missing authority and trigger are.
- **Correction:** none performed.

### `GRB-M-009` — General Reward nodes can grant skill fragments directly

- **Rulebook claims:** `GRB-P0712-0716` restricts fragments to encounter-grade conditions or explicit
  Standard tutorial script and says ordinary Reward nodes/pools do not grant them.
- **Actual authority path:** Reward-node content resolves its declared pool and executes any selected
  `RewardEffect::GrantSkillFragment` through the normal grant transaction.
- **Live data:** `map/node_definitions.ron` points Reward nodes at `default_treasures`.
  `events/rewards/base.ron.default_treasures` includes
  `freischutz_black_round_fragment_reward` and `funeral_butterfly_eulogy_fragment_reward`, both direct
  `GrantSkillFragment` effects. It also includes several legacy research rewards.
- **Consumer impact:** server/Unity receive ordinary Reward-session skill-fragment diffs; there is no
  source-type rejection downstream.
- **Test evidence:** passing integration test `live_rewards_can_grant_skill_fragments_from_ron` and Core
  test `reward_session_reports_skill_fragment_and_research_diffs` explicitly allow this path.
- **Visible effect:** a generic treasure room can bypass the encounter-grade acquisition conditions.
- **Classification:** `runtime_contradiction` + `live_data_gap`; direction is live source opposite.
  Confidence: high.
- **Unverified boundary:** deterministic selection frequency was not sampled; pool membership plus the
  production resolver proves reachability.
- **Correction:** none performed.

### `GRB-M-010` — NormalBoss has no consume/return cycle pool

- **Rulebook claims:** `GRB-L0827`-`GRB-L0832` require a run-local cycle where victory/exhausted failure
  consumes an identity for that cycle, unvisited assignment returns it, and the pool refills only after
  all identities resolve.
- **Actual authority path:** `assign_map_encounters` selects all compatible `NormalBoss` encounters
  independently for each node/floor. `select_encounter_for_map_node` applies recent-floor exclusion and
  research-completion weight. There is no NormalBoss cycle membership, consumed set, unvisited return or
  refill state in `RunState`/checkpoint.
- **Live data:** the NormalBoss catalog exists, while RunPolicy supplies the old generic repeat-weighting
  values used by the selector.
- **Consumer impact:** consumers see the selected encounter only; no cycle state is transported.
- **Test evidence:** encounter repeat-weighting tests pass; there is no NormalBoss cycle test.
- **Visible effect:** a boss can repeat according to weighting rather than after every authored identity
  has appeared once.
- **Classification:** `document_ahead_expected` + `runtime_contradiction`; direction is target pool
  absent and old selector active. Confidence: high.
- **Unverified boundary:** no seed enumeration was needed because the selector and missing persisted
  state establish the semantic conflict.
- **Correction:** none performed.

### `GRB-M-011` — BossOmen eligibility uses a global awakened-fragment gate

- **Rulebook claims:** `GRB-L1137`-`GRB-L1147` require each chain to author optional exact fragment and
  whether awakening is required, with no global “any awakened fragment” prerequisite.
- **Actual authority path:** `apply_boss_omen_to_map` calls `has_awakened_skill_fragment` before selecting
  any provisional chain. The function returns true for any awakened inventory entry. After that,
  `select_provisional_chain` filters only by `min_floor` and skipped proposal IDs.
- **Live data/schema:** `BossOmenChain` and `boss_omen/chains.ron` have no required-fragment/awakening
  fields. The sole live `white_night` chain therefore inherits the global any-fragment gate.
- **Consumer impact:** no eligibility explanation/identity can be projected per chain; Unity sees only an
  already-placed omen.
- **Test evidence:** passing test `live_awakened_fragment_can_trigger_boss_omen_overlay` protects the
  global gate.
- **Visible effect:** an unrelated awakened fragment unlocks every eligible chain, while a future chain
  that should need no fragment is nevertheless blocked until some fragment awakens.
- **Classification:** `document_ahead_expected` + `runtime_contradiction`; direction is schema missing
  and global condition opposite. Confidence: high.
- **Unverified boundary:** exact chain fragment IDs are explicitly future authoring and not part of the
  finding.
- **Correction:** none performed.

### `GRB-M-012` — FinalBoss chain completion/cycle ledger is absent

- **Rulebook claims:** `GRB-L1137`, `GRB-L1145`-`GRB-L1150`, `GRB-L1179`, and `GRB-L1185` require every
  authored ordinary FinalBoss chain once per cycle, locked unfinished members, completion reset, and
  checkpointed active/completed/cycle state.
- **Actual authority path:** `BossOmenRunState` stores provisional, active, placed source, forced boss and
  skipped proposals only. `clear_after_forced_boss_victory` clears all four active fields without adding
  a completed chain. `select_provisional_chain` excludes only skipped proposals and resets that exclusion
  when candidates empty.
- **Live data:** one chain currently exists, but the state/schema would not retain completion if more
  were authored.
- **Consumer impact:** no completed/cycle ledger is serialized or shown.
- **Test evidence:** forced-boss victory test expects `active` and `forced_boss` to become `None`; there is
  no completion-ledger assertion.
- **Visible effect:** after victory, the same chain can be selected again on the next eligible Floor
  instead of waiting for every cycle member.
- **Classification:** `document_ahead_expected`; direction is document-ahead. Confidence: high.
- **Unverified boundary:** with one live chain, immediate reselection is also what a one-member *new*
  cycle would permit; the mismatch is the absent durable completion/cycle transaction and becomes
  externally distinguishable as soon as another chain is authored or records inspect completion.
- **Correction:** none performed.

### `GRB-M-013` — Event/BossOmen source defeat uses the generic retry/completion path

- **Rulebook claims:** `GRB-L0271`-`GRB-L0275` make ordinary Event-started combat defeat end the Event
  immediately with no retry. `GRB-L1160`-`GRB-L1168` require failed first omen sources to return to the
  proposal queue and failed active sources to retain the same step for a later Floor.
- **Actual authority path:** Event combat calls the common `start_abnormality_attempt` path and common
  combat result handler. A nonfatal defeat with attempts remaining returns to `NodeConfirm`; exhausted
  defeat calls generic `plan_complete_current_node(false)`. Generic node completion calls
  `BossOmenRunState::consume_completed_source` without a success flag, which activates a provisional
  chain or advances an active step.
- **Live data:** current live Events contain no `StartCombat` choice and the sole BossOmen step is an
  advance-only Event, so the conflicting branches are production code/test contract but not exercised
  by today's live Event identities.
- **Consumer impact:** server/Unity have no typed “omen source failed/requeued” result; they receive
  ordinary retry/NodeCompleted behavior.
- **Test evidence:** passing test `event_started_combat_defeat_reenters_while_attempts_remain` explicitly
  protects the retry. `boss_omen_event_step_consumption_forces_authored_capstone` demonstrates generic
  completion consumes a placed source.
- **Visible effect:** once StartCombat/Combat omen content is live, the first defeat does not close an
  ordinary Event, and exhausted omen defeat can incorrectly progress the chain.
- **Classification:** `document_ahead_expected` + `test_contract_mismatch`. Direction: approved target
  absent; confidence high for code semantics, with live activation pending.
- **Unverified boundary:** no live source currently activates these branches; the finding does not claim
  a present player can trigger it with today's RON.
- **Correction:** none performed.

### `GRB-M-014` — Exhausted Monolith failure is reusable, not permanent

- **Rulebook claims:** `GRB-L0847`-`GRB-L0851` say the second defeat permanently fails that identity for
  the run and blocks later stage, fragment, mastery and sweep.
- **Actual authority path:** after attempts exhaust, `resolve_failed_monolith_managed_combat` calls
  `MonolithRunState::resolve_remanifestation_failure`. That method removes the active node and clears
  `active_node_id` only. It does not record failure, zero uses, or block `next_stage`. Since
  `is_usable()` depends on unchanged victories plus remaining uses, the same stage can be converted onto
  another Normal node.
- **Live data:** `initial_uses: 3` and `remanifestation_max_attempts: 2` mean a failed first conversion
  normally still has two uses and is eligible again.
- **Consumer impact:** a later NodeConfirm can present the same Monolith card again; there is no
  permanent-failure DTO.
- **Test evidence:** no exhausted-failure regression test exists. The passing state test
  `floor_transition_discards_active_node_without_refunding_committed_use` confirms that clearing an
  active node with uses remaining leaves `progress.is_usable()` true, matching the failure method's
  resulting state.
- **Visible effect:** permanent-risk semantics are replaced by additional same-stage attempts on later
  nodes until uses run out.
- **Classification:** `runtime_contradiction`; direction is runtime opposite. Confidence: high from the
  direct state transition.
- **Unverified boundary:** a fresh end-to-end synthetic probe was unnecessary; the mutation and
  eligibility predicates are closed in one module.
- **Correction:** none performed.

### `GRB-M-015` — Formal fragment active skills remain direct-attack placeholders

- **Rulebook claims:** `GRB-L0954`-`GRB-L0959` define fragments as representative active skills and
  explicitly call the 28 abnormality-derived plus one starter direct `BasicAttackModifier` entries
  temporary placeholders rather than completed policy.
- **Actual authority path:** the fragment effect runtime still supports and applies
  `BasicAttackModifier` directly; no dedicated active skill replaces these entries.
- **Live data:** `skill_fragments/base.ron` contains 29 `effect: BasicAttackModifier(...)` entries: one
  concept starter and 28 abnormality placeholders. The file descriptions also label the derived entries
  placeholders until dedicated skills are designed.
- **Consumer impact:** Unity receives direct fragment effect/progression rather than a selectable active
  skill for those identities.
- **Test evidence:** live skill-catalog/integration tests pass with the current fragment catalog. The
  Freischutz authored active path is a separate implemented example and does not replace the 29 listed
  placeholders.
- **Visible effect:** equipping most fragments modifies basic attacks instead of providing the formal
  fragment active-skill interaction.
- **Classification:** `document_ahead_expected`; the discrepancy is already disclosed by the rulebook,
  but remains a real target/live difference. Confidence: high.
- **Unverified boundary:** no claim is made about the final design of each future skill.
- **Correction:** none performed.

## Evidence gaps and exclusions

- External Unity source, fixtures and test assertions were readable at commit
  `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`; no `Unity`, `unity-editor` or `Unity.exe` runner was
  available in the execution shell. No finding depends solely on an unexecuted Unity assertion.
- Hidden-boss gameplay is not listed as a current mismatch: the Monolith section explicitly marks its
  consumer/implementation as follow-up, so the detailed hidden-boss rules are `future_or_dormant` for
  this audit.
- Missing exact future fragment IDs, reward quantities and BossOmen minimum Floors are not mismatches by
  themselves because the rulebook explicitly delegates those values to follow-up RON authoring.
- Unity-local Headquarters “Support” presentation identifiers were not classified as resurrection of
  the removed Core `ChooseSupport` mode. They select current typed HQ actions; the old wire behavior is
  explicitly rejected by the server test.

## Catalog verdict

The current working tree does **not** implement the updated rulebook completely. Fifteen confirmed
differences remain. Most belong to the explicitly approved/pending policy group; the catalog separates
expected missing authorities from reachable legacy/direct contradictions and from the explicitly
documented fragment placeholder set. This investigation authorizes no correction direction.
