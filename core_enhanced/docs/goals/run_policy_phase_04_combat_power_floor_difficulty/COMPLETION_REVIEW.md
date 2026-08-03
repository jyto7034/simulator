# P04 Certified Combat Power And Floor Difficulty Completion Review

Review date: 2026-07-27

Verdict: **Complete / High**

Implementation complete: true

## Audit Table

| Item | Source of truth | Verdict | Fit |
| --- | --- | --- | --- |
| Versioned certified power | `run/policy.ron`, `combat_power.rs`, persistent stat pipeline | Complete | High |
| Run peak and Floor anchor | `RunPowerAuthorityState`, Floor creation transaction | Complete | High |
| Deterministic node difficulty | facility assignment profile + stored `MapNodeCombatDifficulty` | Complete | High |
| Runtime difficulty effects | combat preview/stat/wave and reward contracts | Complete | High |
| Non-rewind persistence | Core checkpoint boundary + active-run envelope v2/backup merge | Complete | High |
| Qualitative client projection | `MapNodeDto` + Unity reader/presenter/UGUI | Complete | High |
| Legacy/P03 convergence | strict missing-authority rejection and broad regressions | Complete | High |

## 1. Certified Power And Anchor

Core derives power deterministically from living roster members and persistent battle inputs: level, permanent
stats, equipped equipment/enhancement, equipped fragments, attack interval/range/block and active skill. Current
HP, Trauma/Breakdown and active consumables are excluded and projected as readiness. Floor 1 initializes after
starter/loadout commit; later committed Combat entries max-update the run peak. New Floor anchor follows the
approved three-way max and same-Floor entry never rewrites it.

The live model is required and versioned. All weights are positive, Standard coverage is exact, Endless ends in an
open-ended stage, and invalid schedules/profiles fail publication.

## 2. Manifested Difficulty And Effects

P03 facility generation now resolves the required difficulty profile after complete category/kind assignment.
Ordinary Combat uses seeded largest-remainder Easy/Standard/Hard quotas whose counts sum exactly; Elite and boss
roles are excluded. Each node stores model/profile/class/anchor/target once. BossOmen capstone inherits the same
Floor anchor and receives the BossOmen role.

Enemy stats and generated waves consume that stored tuning. Quantity rewards consume its deterministic multiplier;
Hard and every special role are validated stronger than Standard. Missing or stale manifested authority fails
closed. Floor number and current roster are not alternate difficulty sources.

## 3. Persistence And Wire Boundary

RunMap/node authority is checkpoint-owned and round-trips exactly. Model version, historical peak and current Floor
anchor are separate monotonic active-run metadata: checkpoint load and battle/transaction rollback do not rewind
them, and server current→backup recovery max-merges peak/anchor/loads for the same model version.

Core exposes only difficulty band, current relative-threat band and readiness warnings. Tests prove serialized node
DTOs omit exact target/anchor/model/profile fields and readiness changes leave stored target/anchor unchanged.
Unity has no power formula; it forwards qualitative fields into marker/detail and renders a threat glyph.

## 4. P03 Regression And Legacy Removal

Facility template selection, constrained assignment, manifestation, capstone, checkpoint/active-run round-trip,
Unity exact resource/version and marker layout all reran in the broad suites. The only first-pass failures were five
manual test maps missing the newly required authority; fixtures were corrected and all reran green.

Production no longer defines or consumes `floor_combat_scaling`, overflow clamp, Floor-number scale or legacy alias.
Negative tests intentionally retain the removed name to prove rejection.

## Validation

- Core final broad: `cargo test --lib` — **775 passed, 0 failed**.
- Server final broad: `cargo test` — **52 passed, 0 failed**; doc tests green.
- Unity compile: ready; Console errors **0**.
- Unity broad EditMode: job `07197ac99b074dadb7e9a530d1fe712d` — **254/254 passed**.
- Unity qualitative UGUI PlayMode: job `6b0fc75ee5284a31a51ac0c1f38b31e4` — **1/1 passed**.
- Focused Core evidence includes power 3/3, rewards 2/2, facility checkpoint 9/9, stored stat/wave authority,
  DTO secrecy/readiness, Combat-entry peak and rollback/backup max-merge.

## Final Verdict

- Primary verdict: **Complete**
- Long-term fit: **High**
- Improvement class: **none required for P04**
- Remaining risk: provisional coefficients still require later representative-content simulation/playtest, as
  explicitly out of scope. P05 must consume the authored MonolithRemanifestation role without adding a parallel
  formula or quota authority.
