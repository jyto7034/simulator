# Unity Recovery And Roster Command Flows

Goal: G08

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Complete ArkClient's player-facing command paths for the three existing Core behaviors that are
currently unreachable from normal Unity UI: `RecoverBattleSetupLoss`, `LoadRunCheckpoint`, and
`MoveRosterUnit`. Unity must consume the current authoritative action capability and typed roster/map
projections without adding local eligibility rules, a parallel roster model, or a new wire schema.

## Required Resume Read Set

- `../cross_layer_contract_hardening_master/STATUS.md`
- `../cross_layer_contract_hardening_master/phases/phase_02_combat_transaction/PLAN.md`
- `../cross_layer_contract_hardening_master/phases/phase_02_combat_transaction/EXPERIMENT_NOTES.md`
- this file
- `EXPERIMENTS.md`
- `EXPERIMENT_NOTES.md`
- `../boss_omen_gate_and_rollback_policy/COMPLETION_REVIEW.md`

Read the historical audit source only for concrete finding provenance or an ownership conflict.

## Owned Finding

- DCA-H-13 / CPR-H-07: Unity lacks normal typed command and UI flows for battle setup recovery,
  checkpoint load, and roster movement even though Core and the canonical contract expose them.

## Accepted Authority Boundary

- Core root or battle `action_capabilities` determines whether each command is relevant and enabled.
- Unity may display a disabled reason from the projected stable reason code but may not infer
  eligibility from `GameState`, checkpoint counters, BossOmen fields, roster count, or kind strings.
- `RecoverBattleSetupLoss` remains a read-only recovery-scoped battle command. It reprojects the
  immutable setup and current revisioned state without consuming a revision or carrying mutation
  authority.
- `LoadRunCheckpoint` remains the existing recovery command. It covers ordinary ViewingMap
  checkpoint use and the qualified G14 forced-BossOmen RunFailed exception only when Core projects
  it; Unity must not create a generic GameOver escape.
- `MoveRosterUnit` sends the existing employee UUID, destination slot, and optional swap UUID. The
  displayed slots and post-command state come from authoritative snapshot/result DTOs.

## Planned Implementation

1. Add explicit typed `GameCommandBus` methods with the exact existing Rust field names and the
   existing mutation-versus-read-only authority behavior.
2. Add presentation gateway/command payloads only where needed to keep composers independent from
   transport details.
3. Surface checkpoint and setup recovery only from projected capabilities on the applicable screen
   or battle recovery presentation.
4. Add roster move controls to the existing maintenance roster surface using projected slot facts;
   avoid local legality assumptions beyond selecting a concrete source/destination payload.
5. Apply authoritative command results/snapshots through the existing store pipeline and clear
   local selections after successful transitions.
6. Add focused EditMode contract/composer/interaction tests and an applicable live PlayMode path,
   then run the full Unity suite and unchanged Core/server regressions required by Phase 2.

## Invariants

- No command is sent when its current capability is omitted or disabled.
- Battle setup recovery never consumes a revision or substitutes resync/reconnect behavior; its
  returned setup/current projection remains revisioned authority.
- Checkpoint load is never inferred from checkpoint existence or failure reason.
- Roster ordering never uses card order as an independent source of truth.
- BossOmen victory still uses the projected ordinary Gate select/confirm flow; no local Floor
  transition is introduced.
- No compatibility aliases, generic untyped command UI, ignored regression tests, or static action
  whitelist are added.

## Explicit Non-Scope

- Core, server, save, DTO, RON, balance, or BossOmen policy changes.
- Event flow, item identity, Shop/reward, targeting, or deferred G05/G22 work.
- Drag-and-drop polish if deterministic accessible move controls can prove the complete command
  contract with less surface area.

## Validation

1. Focused command serialization/gating tests for all three behaviors.
2. Composer and interaction tests for enabled, disabled, omitted, successful, and failed command
   states, including RunFailed checkpoint recovery and roster swap/reorder.
3. Battle authority regression for setup recovery and no local capability fallback.
4. Unity MCP EditMode full suite and applicable PlayMode/live-server flow.
5. Core all-targets and game_server all-targets to prove no cross-layer regression.
6. Scoped diff, console, ignored-test, generic-escape-hatch, and static-recovery legacy searches.
7. Guide-based completion review before R02 unlock.

## Stop Conditions

Stop if the implementation requires a new public payload/result shape, local recovery policy, a
parallel roster source, a protected policy decision, or reverting unrelated dirty-worktree changes.
