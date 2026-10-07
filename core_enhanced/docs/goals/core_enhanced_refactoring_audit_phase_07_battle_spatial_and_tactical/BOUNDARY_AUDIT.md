# Core Enhanced Refactoring Audit Phase 07 — Boundary Audit

Audit Date: 2026-08-07
Status: complete; phase-exit audit `Complete / High`

## Exact Boundary

Phase 7 owns the current validated battlefield topology to runtime body/occupancy to continuous movement and live
deploy/withdraw/stance/relocation/swap/field-work mutation seam. Phase 5 owns preview/setup construction, Phase 6
owns the battle clock and advance pipeline, Phases 8~9 own effect/target semantics, and Phase 10 owns public DTO and
server transport refactor discovery.

### Primary runtime

- `src/game/battle/battlefield/{field,mod}.rs` and `src/game/battle/tile_range.rs`
- all nine files under `src/game/battle/core/movement/**`
- `src/game/battle/core/{field_work,spatial,tactical_relocation,tactical_stance,tactical_swap}.rs`
- spatial/tactical slices of `battle/core/{build,commands,mod,sim,types}.rs`
- `src/game/world/{combat,state}.rs` at live deployment/resource/revision transaction seams

### Direct carriers and consumers

- tactical plan and unit carriers in `battle/{scenario,types}.rs` and `resources/mod.rs`
- `range_preview.rs` only where authoritative body/tile state is projected
- `data/run_policy_data.rs`, `behavior.rs` and published `game_resources/data/run/policy.ron`
- focused inline and `world/tests/combat.rs` tests
- `docs/{core_runtime_contract,game_rulebook}.md`

Damage formulas, buff/control/resonance/death interpretation remain Phase 8 carriers. Attack/skill selection,
projectile/area delivery and range correctness remain Phase 9 carriers. DTO shape and server cursor ownership remain
Phase 10 even where their current calls establish Phase 7 failure reachability.

## Authority Matrix

| Domain | Current authority | Derived/consumer state | Finding |
| --- | --- | --- | --- |
| Static topology | `BattlefieldLayout` dimensions, valid tiles and static obstacles | placement, path and movement obstacle input | one runtime topology; current setup supplies it |
| Runtime position | `RuntimeUnit.body.position` | projected tile, occupancy, range, block, checkpoint and movement log | one authoritative body; active segment is interpolation only |
| Player occupancy | active Player `Combatant | DefenseObject` bodies | deploy/relocation/swap/field-work admission | derived deterministically; enemy overlap intentionally excluded |
| Movement intent | per-tick planner goal map plus unit locks/block state | movement tick input | current stored body goal is overwritten and is not authority |
| Movement physics | default `RapierMovementWorld` | corrected body endpoint and segment | production has one backend, but test-only Direct and wall paths remain |
| Blocking | `BlockRuntimeState` rebuilt from bodies/capacity and retained pairs | target preferences and movement locks | deterministic route/spawn/ID ordering; retained |
| Tactical state | stance/tile/profile/cooldowns/rest epoch/activity | block, Focus, Rest and DTO snapshot | actual effect mutation is split and drifts for FieldWorking cross-tile moves |
| Field work | scenario definition plus BTree runtime assignment/progress/generation | activity and checkpoint | deterministic scheduling/reconcile; cross-tile effect integration is defective |
| Tactical resource | one active-session pool and battle-time recovery cursor | command eligibility/cost/checkpoint | saturating and Pause-safe; retained |
| Command transaction | Core mutation plus world mapping/resource/revision/DTO | one command response | future cursor and rare reconciliation error can reject after commit |

## Battlefield, Position And Occupancy Trace

`BattlefieldLayout` stores immutable dimensions/valid-tile shape and battle-local static obstacles. Start clears and
rebuilds only obstacles; valid topology remains the scenario-derived field. Walkability first validates the actual
valid-tile set, then static obstacles. Non-rectangular void cells are also projected into continuous movement as
static obstacles, so placement and physics agree on inaccessible cells.

`RuntimeUnit.body.position` is the live spatial authority. Logical positions are projections of that world body;
checkpoint, block distance, route progress and relocation all consume the body. `ActiveMovementSegment` stores the
fixed-tick start/end and samples intermediate event times. The endpoint in the body is intentionally the reconcile
target while the immutable segment is presentation and mid-window sampling state; current canonical explicitly
separates event timing from checkpoint position, so this is not a competing authority.

Player tactical occupancy is derived on demand from active Player combatants and defense objects. Exclusion lists
support relocation and two-participant swap, candidates are sorted by unit ID, and enemy overlap is deliberately
ignored. Deploy validates world deployment-zone kind/affinity, then Core rechecks walkability and current occupancy
immediately before mutation. Relocation and swap similarly preflight final zones, profiles, work-cell kinds and all
participants before changing bodies.

## Continuous Movement And Blocking Trace

The planner refreshes block state, reads scenario or unit route policy, and creates a goal map for the current tick.
Core sorts unit IDs and the engine canonicalizes its input again. Ground movement projects valid/static topology;
airborne movement ignores static obstacles but still clamps to the board. Rapier creates kinematic unit bodies,
ignores unit colliders for correction by approved overlap policy, corrects against static obstacles, and returns a
body endpoint. Core applies the result, updates opponent facing and records one immutable movement segment.

Existing engagements are retained before new block assignment. New candidates use route progress descending, spawn
order and unit ID, while blocker ties use distance and unit ID. Death, withdrawal, Hard CC, stance, relocation and
field work all rebuild or release the same block state. No second occupancy reservation or path-position store was
found.

Production `BattleCore` constructs `ContinuousMovementBackend::Rapier` and no production call switches it. The
Direct backend, its swept-AABB/depenetration implementation and the Core switch are exercised only by tests; the
authored-route integration test explicitly selects Direct. Rapier also carries a `#[cfg(test)]` board-wall collider
subsystem even though production board bounds are enforced by the shared clamp. These paths make tests maintain
non-production collision authorities.

The public movement model also contains state-shaped values that do not drive runtime: tick `now_ms`, input owner
and current target are unread by the engine; two output variants are never produced; body mode/physics handle are
never read; and stored body goal is overwritten from the planner map. Previous position and velocity are written but
have no production reader. Public Rust consumers are unknown, so this is a cleanup candidate with a compatibility
gate rather than evidence that body position itself is duplicated.

## Deployment, Resource And Command Trace

World deploy resolves current employee eligibility, authoritative preview zone kind, effective profile, cost and
redeploy lock. Core revalidates destination/work-cell state, creates the runtime unit and tactical state, logs deploy
and reconciles field work. World then advances the instance salt, pays once, updates deployment mapping, applies
live resource signals, reconciles employee attempt state, commits one revision and builds the delta/checkpoint.

Withdraw mutates the same runtime instance to Withdrawn, clears battle-local tactical/action state and logs it;
world records current-HP redeploy policy and the battle-time lock. Stance, relocation and swap preflight current
Core disabled reasons; relocation/swap additionally require and pay one global resource cost. Pause freezes the
battle clock and natural resource recovery, but these command-time mutations remain legal at the frozen time.

`TacticalResourceState` clamps initial/current/maximum, recovery products, payments and signed deltas. Published
policy validation rejects zero movement/Rest/cooldown values and out-of-bounds resource costs. Required field-work
time is also publication-validated as nonzero. No current infinite same-time Rest or field-work schedule was found.

### Late failure boundary

All five tactical world handlers call `battle_update_dto_after(after_seq)` only after Core mutation, resource or
mapping mutation and projection revision commit. A future cursor therefore returns `InvalidBattleResyncSeq` after
the authoritative state changed, contradicting the active-battle no-mutation-on-failure contract. Deploy/withdraw
also call fallible employee reconciliation after Core mutation. Server-owned cursor behavior reduces ordinary live
frequency, but public Core transport accepts the cursor and current tests cover only read-only future-seq rejection.
Phase 6 C03 owns the same root in battle advance; Final should combine the shared cursor/transaction mechanism while
keeping each domain's staged work explicit.

## Tactical State, Relocation, Swap And Field Work

`UnitTacticalState` stores only stance, tile, weapon profile, base capacity, cooldown deadlines, Rest epoch/deadline
and exclusive activity. Snapshot derives `None | DefenseFocus | Rest` from the full activity/stance/tile tuple. The
actual unit modifier/block/Rest state, however, is updated independently by stance change, position update and field-
work transition.

`apply_tactical_position_update` derives old/new Focus from tile kind without checking activity. Canonical permits a
FieldWorking unit to relocate or swap and says effects are absent while working, then derived once from final
occupancy:

1. Ground FieldWorking already had Focus removed on assignment.
2. Ground to Platform subtracts that Focus again; Platform to Ground adds it while still working.
3. If the move leaves work, exit derives the final Ready+Defense effect again, causing double subtraction/addition
   relative to unrelated aggregate modifiers.
4. If it lands on another work cell, activity remains FieldWorking but Platform relocation exposes a Rest deadline;
   the scheduled event later no-ops only because the handler rechecks activity.
5. Swap calls the same helper twice and has the same cross-kind defect.

Existing field-work relocation coverage uses only Ground cells and checks the derived enum, not aggregate modifier
identity. Current live equipment has only GroundOnly or PlatformOnly profiles and the first live work cell is Ground,
so the exact cross-kind sequence is not reachable with today's published bundle. The typed equipment contract,
validator, Core tests and canonical nevertheless explicitly support Any profiles and Ground↔Platform relocation as
an existing capability. This is a correctness correction to an approved contract, not a new balance feature.

Field-work progress itself is coherent. It settles elapsed battle time before assignment changes, uses deterministic
BTree maps/sets, preserves progress, suspends only contribution under Hard CC, increments generations when worker
count changes and consumes completion exactly once. Relocation/swap reconcile only after all participant positions
are committed, as required. A single full-state tactical-effect transition should replace the three partial effect
mutators without changing these scheduling rules.

## Preview And Core Hardening Signals

Preview validation unions deployment cells but does not reject a cell appearing in two zones; world zone lookup
selects the first match. Official ASCII template construction assigns one character per cell and current live zones
are disjoint, so Phase 7 did not establish a current runtime failure. The validation omission belongs with Phase 5
C02 publication/setup feasibility rather than a duplicate spatial candidate.

Core live deploy pushes a scenario group before every downstream operation is complete. A manually constructed
test-fixture profile or reused salt can make the public `apply_live_command` error after partial Core mutation, but
world deploy uses publication-validated employee profiles and a monotonic salt. External direct Rust consumers are
unverified and no current live route was proved. This remains a hardening signal to include in C02's failure tests,
not a separate production correction.

## Determinism, Pause And Canonical Alignment

- Valid positions, static obstacles, unit input, occupancy candidates, field-work objects/cells and block priorities
  have explicit deterministic order.
- Movement tick cadence and all tactical cooldown/Rest/field-work deadlines use battle time. Pause creates no
  catch-up and instantaneous approved commands use the frozen time.
- Same-tile facing-only relocation preserves Rest cadence; cross-tile relocation intentionally starts a full new
  interval when the unit is Ready+Defense.
- Resource payments, deploy/redeploy locks, swap one-cost semantics and final-occupancy field-work reconciliation
  agree with live policy and canonical outside the two confirmed corrections.
- Effect/damage value interpretation and target/delivery geometry were not treated as verified; their carriers are
  handed to Phases 8 and 9.

## Consumer Classification

- Confirmed authoritative consumers: BattleCore execution/planner, world live tactical commands, active-session
  resource/deployment state and current Core DTO construction.
- Confirmed derived consumers: range preview, tactical/field-work checkpoint and immutable movement presentation
  events.
- Current live RON was read for policy/affinity reachability only; no data mutation was performed.
- Server cursor semantics are supporting evidence only and Phase 10 remains its refactor owner. Unity was not read.
- Unknown external Rust users of public movement engine/body/backend and `BattleCore::apply_live_command` are
  unverified; public removals require inventory or an approved compatibility decision.

## Boundary Verdict

Two correctness candidates are confirmed: derive FieldWorking cross-tile tactical effects exactly once from the
full final state, and make tactical world commands fail-atomic before cursor/reconciliation errors. Two movement
cleanup candidates remove non-production physics paths and ghost state/API after external-consumer review. Static
topology, body/segment ownership, deterministic block planning, tactical resource, ordinary relocation/swap and
field-work scheduling are retained. Phase 7 has sufficient boundary evidence for a completion audit without
changing production files.
