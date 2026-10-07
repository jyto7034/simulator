# Boss Runtime v1 — P05 Live Reference Boss Decision Packet

Status: `approved — option 1`  
Prepared: 2026-08-09  
Decision owner: user  
Approved: 2026-08-09  
Implementation authority: P05 child only

## Approval record

The user approved option 1 and explicitly authorized the agent to author the missing details as replaceable test
content because the content will be redesigned later. This approval does not promote the proposed pursuit behavior
to final Little Red lore/content canon. It authorizes one production-live reference fixture for exercising and
validating Boss Runtime v1 under the exact balance, presentation and non-scope boundary below.

## Why a decision is required

P04 proved the runtime with test-only content. P05 changes official live content and therefore needs an explicit
choice of identity, required gameplay subset, balance/assets and public presentation meaning. These are not technical
defaults. This packet records current evidence and a recommended bounded package; it does not activate P05 or mutate
production data.

## Recommended package — Little Red pursuit reference

### Identity

- Existing encounter: `suppress_little_red`
- Existing abnormality: `f-01-57_little_red` / Little Red Riding Hooded Mercenary
- Current classification: `NormalBoss`, WAW, `Boss/Boss` live combat, `threat_class: boss`

This is the recommended identity because its approved design direction is target marking and pursuit, which maps
directly to the already implemented activation-owned target lock, `AcquireTarget`, `MoveToTarget`, forward route
rejoin and `RouteCombat`. It exercises the v1 kernel without pretending that an unrelated mechanic is implemented.

### Required v1 gameplay subset

- One exact bound actor: `primary_abnormality`, spawn index 0.
- One finite, one-shot pursuit Pattern using only the current compiled vocabulary:
  `AcquireTarget → MoveToTarget → RejoinForwardRoute → RouteCombat`.
- Target query: active reachable hostile, deterministic minimum path cost, `LockUntilInvalid`.
- No implicit retarget inside the Pattern; no-target, invalidation, path failure and timeout use authored exhaustive
  outcomes and all return to ordinary route combat.
- Add `route_id: "defense_main"` only to the bound primary wave. All three current `Ambush/Medium` live templates
  author that route.
- After the finite pursuit, the existing basic attack and automatic `little_red_hunt_the_prey` skill remain the leaf
  combat authorities.

Not included in the recommended v1 reference:

- a new public bounty-mark icon or duration lifecycle;
- special bounty reward logic;
- Big Bad Wolf pair/inter-encounter rules;
- new Task/selector/condition/mechanic vocabulary;
- final-boss content.

### Balance and rewards

Preserve current live values and composition:

- stats: HP 800, attack 70, defense 35, magic resist 35, movement 1500;
- existing basic attack cadence/delivery and `little_red_hunt_the_prey` skill;
- current WAW difficulty, two supporting wave entries, spawn time 5000 ms and reward UUIDs;
- no new reward quantity, skill number or encounter multiplier.

P05 still runs all manifested-difficulty and full setup feasibility gates. A failing gate triggers a technical
correction or returns a material balance decision to the user; it is not silently normalized.

### Presentation and assets

Recommended v1 boundary: existing public transport only.

- Core/server use existing `UnitSpawned`, movement, basic/skill projectile, area, HP, buff, death and checkpoint
  projections. Internal Phase/Pattern/target-lock state remains private.
- Existing `unit_source.abnormality_id`, Boss HUD threat class and current skill presentation IDs remain the identity
  and leaf presentation inputs.
- Current Unity battlefield renders every opponent as the common red capsule. It consumes actor identity, Boss HUD,
  movement timing and damage/checkpoint state, but has no Little Red-specific battle prefab or integrated Spine/VFX
  mapping.
- Approving this package therefore approves the common opponent capsule as the first v1 reference's current asset
  boundary. Dedicated Little Red art/animation/VFX is deferred and is not claimed complete.

Under this boundary P05 is expected to prove P06 `not_applicable_to_v1`, but that disposition is made only after the
actual live path and external consumer evidence pass. It is not pre-approved by this packet.

## Alternatives considered

| Candidate | Advantage | Material cost or mismatch | Disposition |
| --- | --- | --- | --- |
| Little Red | Existing NormalBoss/Boss HUD/skill; pursuit maps directly to TrackAndRejoin; all Ambush/Medium templates have `defense_main` | Full bounty marker/reward and wolf relationship are deliberately outside the v1 subset | recommended |
| Nameless Fetus | P04 already used its route encounter in the raw source proof; no route edit needed | Current encounter is Elite/Defense and the identity does not represent an official NormalBoss reference | fallback technical fixture, not recommended live identity |
| Burrowing Heaven | Existing NormalBoss and every ChokePoint/Medium template has `defense_main` | Approved concept is observation/gaze; a pursuit Pattern would misrepresent its defining mechanic | reject for this minimal v1 reference |
| Freischütz | Existing NormalBoss, Ambush route, field-work object and skill content | Defining line penetration, friendly-fire/7th bullet telegraph and first-clear aftermath exceed current v1 vocabulary/presentation | defer to a richer content Goal |
| Funeral Butterfly / Nothing There | Existing NormalBoss content | Defining execution field or transformation mechanics are not current v1 tasks; BossArena also has no authored route | defer |
| WhiteNight / Silent Orchestra / Apocalypse Bird | Strong future stress cases | Explicitly separate final-boss systems and public presentation work | out of P05 scope |

## Presentation alternative requiring broader work

The user may instead approve Little Red with dedicated battle art/animation/VFX or an explicit bounty marker. That
changes the activation package:

- dedicated visual asset and mapping require an approved asset source/art direction and current Unity integration;
- an explicit mark/telegraph with authoritative lifetime may require a Core semantic lifecycle, exact game_server
  setup/update/resync parity and synchronized Unity state/reconnect handling;
- P05/P06 boundaries must be updated before production mutation, and P06 may become required.

## Exact approval needed

Choose one:

1. Approve the recommended Little Red pursuit reference, preserving current balance/rewards and using the existing
   generic opponent presentation with no new public lifecycle.
2. Approve Little Red but require dedicated visual/VFX and/or a public bounty marker; specify the required visible
   behavior and approved asset/art source.
3. Select another identity and state its minimum required gameplay, balance and presentation boundary.

Option 1 was approved on 2026-08-09. The Master may activate only the bounded P05 child. Any dedicated visual/VFX,
public bounty marker, new gameplay vocabulary or broader Little Red content remains unapproved and must stop P05
before implementation if it becomes necessary.

## Evidence anchors

- Simulator live encounter: `game_resources/data/pve/encounters.ron`
- Simulator actor/skill: `game_resources/data/abnormalities/base.ron`, `game_resources/data/skills/base.ron`
- Simulator route templates: `game_resources/data/map/battlefield_templates.ron`
- Runtime contract: `core_enhanced/docs/core_runtime_contract.md`
- External consumer canonical: `/mnt/f/unity projects/ark/docs/unity_core_contract.md`,
  `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- External current actor renderer:
  `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldFieldAssembler.cs`
