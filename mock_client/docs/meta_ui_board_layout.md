# Meta UI Board Layout

## Goal

Build the meta-game UI as a single continuous board scene.

- Battle board size is `7 x 8`
- Top `7 x 4` belongs to the opponent side
- Bottom `7 x 4` belongs to the player side
- Outside battle, the opponent side transforms into a `meta stage`
- Avoid full-screen modal overlays like TFT augment selection
- Keep the player field visible so state changes feel continuous

This should feel closer to `The Bazaar` than to a separate menu screen.

## Core Principles

1. The board remains visible in all major states.
2. The player side is the stable anchor of the screen.
3. The opponent side is both:
   - the enemy field during battle
   - the meta stage during shop / event / reward / suppression prep
4. Meta states should feel like a transformation of the opponent half, not a popup layered over gameplay.
5. The center frontline remains a strong visual divider between player territory and opponent territory.
6. Opponent side and player side can use different framing, material language, and lighting, similar to Runeterra's top vs bottom separation.

## Board Model

```text
Top half    = Opponent side / Meta stage   = 7 x 4
Bottom half = Player field                 = 7 x 4
Total board = 7 x 8
```

## High-Level Screen Structure

```text
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ TOP BAR   Phase / Ordeal / Qliphoth / Enkephalin / Status                           │
├──────────────────────────────────────────────────────────────────────────────────────┤
│                                OPPONENT SIDE / STAGE                                │
│                                7 x 4 REGION                                          │
│───────────────────────────────── CENTER FRONTLINE ───────────────────────────────────│
│                                PLAYER SIDE / FIELD                                   │
│                                7 x 4 REGION                                          │
├──────────────────────────────────────────────────────────────────────────────────────┤
│ BOTTOM HUD  Bag / Artifacts / Bench / Inventory / Selection / Actions               │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

## Shared Battlefield Layout

Use this as the base composition for every state.

```text
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ TOP BAR   Phase I   Ordeal Dawn   Qliphoth 2   Enk 12                               │
├──────────────────────────────────────────────────────────────────────────────────────┤
│                                OPPONENT SIDE / STAGE                                │
│  ┌────┬────┬────┬────┬────┬────┬────┐                                                │
│  │    │    │    │    │    │    │    │  row 8                                         │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │    │    │    │    │    │  row 7                                         │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │    │    │    │    │    │  row 6                                         │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │    │    │    │    │    │  row 5                                         │
│  └────┴────┴────┴────┴────┴────┴────┘                                                │
│───────────────────────────────── CENTER FRONTLINE ───────────────────────────────────│
│                                PLAYER SIDE / FIELD                                   │
│  ┌────┬────┬────┬────┬────┬────┬────┐                                                │
│  │    │    │    │    │    │    │    │  row 4                                         │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │    │    │    │    │    │  row 3                                         │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │    │    │    │    │    │  row 2                                         │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │    │    │    │    │    │  row 1                                         │
│  └────┴────┴────┴────┴────┴────┴────┘                                                │
├──────────────────────────────────────────────────────────────────────────────────────┤
│ [Bag] [Artifacts] [Bench] [Inventory] [Selected] [Action 1] [Action 2] [Action 3]   │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

## State Layouts

### 1. Event Selection

Design intent:

- Opponent half becomes an event stage
- Player field remains visible for build context
- No darkened full-screen modal
- The three choices should feel physically placed in the opponent territory

```text
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ TOP BAR   Event Selection                                              Enk 12        │
├──────────────────────────────────────────────────────────────────────────────────────┤
│                              OPPONENT EVENT STAGE                                    │
│  ┌──────────────┬──────────────┬──────────────┐                                      │
│  │   EVENT A    │   EVENT B    │   EVENT C    │                                      │
│  │    SHOP      │    BONUS     │ SUPPRESSION  │                                      │
│  │ short desc   │ short desc   │ short desc   │                                      │
│  │ reward/info  │ reward/info  │ risk/reward  │                                      │
│  └──────────────┴──────────────┴──────────────┘                                      │
│                                                                                      │
│                    Selected: EVENT B                                                 │
│                    detail / flavor / expected outcome                                │
│───────────────────────────────── CENTER FRONTLINE ───────────────────────────────────│
│                                PLAYER FIELD                                          │
│  ┌────┬────┬────┬────┬────┬────┬────┐                                                │
│  │    │ U  │    │ U  │    │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │ T  │    │ U  │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │ B1 │ B2 │ B3 │ B4 │ B5 │ B6 │ B7 │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │ reserve / owned formation / current loadout / placement context       │          │
│  └────┴────┴────┴────┴────┴────┴────┘                                                │
├──────────────────────────────────────────────────────────────────────────────────────┤
│ [Bag] [Artifacts] [Bench] [Profile] [Event Detail]             [Inspect] [Select]    │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

### 2. Shop

Design intent:

- Opponent half becomes a merchant stage or display shelf
- Shop items should appear staged inside the opponent zone, not in a detached panel
- Player side remains visible to support comparison and planning

```text
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ TOP BAR   Shop Stage                                                   Enk 12        │
├──────────────────────────────────────────────────────────────────────────────────────┤
│                               OPPONENT SHOP STAGE                                    │
│  ┌────────┬────────┬────────┬────────┬────────┐                                      │
│  │ ITEM 1 │ ITEM 2 │ ITEM 3 │ ITEM 4 │ ITEM 5 │                                      │
│  │ cost   │ cost   │ cost   │ cost   │ cost   │                                      │
│  │ tag    │ tag    │ tag    │ tag    │ tag    │                                      │
│  └────────┴────────┴────────┴────────┴────────┘                                      │
│                                                                                      │
│              Selected item detail / compare / equip target / lore                    │
│───────────────────────────────── CENTER FRONTLINE ───────────────────────────────────│
│                                PLAYER FIELD                                          │
│  ┌────┬────┬────┬────┬────┬────┬────┐                                                │
│  │    │ U  │    │ U  │    │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │ T  │    │ U  │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │ B1 │ B2 │ B3 │ B4 │ B5 │ B6 │ B7 │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │ inventory / equipment relation / current build context               │           │
│  └────┴────┴────┴────┴────┴────┴────┘                                                │
├──────────────────────────────────────────────────────────────────────────────────────┤
│ [Bag] [Artifacts] [Inventory] [Selected Item] [Equip Target] [Reroll] [Buy] [Exit]   │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

### 3. Reward Selection

Design intent:

- Reward selection should look like a staged reveal in the opponent territory
- This is still part of the board flow, not a separate screen

```text
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ TOP BAR   Reward Stage                                                                 │
├──────────────────────────────────────────────────────────────────────────────────────┤
│                              OPPONENT REWARD STAGE                                   │
│                                                                                      │
│           [ REWARD A ]           [ REWARD B ]           [ REWARD C ]                 │
│                                                                                      │
│           rarity/effect           rarity/effect           rarity/effect              │
│                                                                                      │
│                    Selected reward detail / synergy / impact                         │
│───────────────────────────────── CENTER FRONTLINE ───────────────────────────────────│
│                                PLAYER FIELD                                          │
│  ┌────┬────┬────┬────┬────┬────┬────┐                                                │
│  │    │ U  │    │ U  │    │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │ T  │    │ U  │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │ B1 │ B2 │ B3 │ B4 │ B5 │ B6 │ B7 │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │ build context remains visible                                       │           │
│  └────┴────┴────┴────┴────┴────┴────┘                                                │
├──────────────────────────────────────────────────────────────────────────────────────┤
│ [Artifacts] [Bench] [Reward Detail] [Skip/Close]                    [Take Reward]     │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

### 4. Suppression Preparation

Design intent:

- Opponent half becomes a pre-battle briefing stage
- It should feel like enemy territory revealing encounter information
- Keep formation and unit management visible on the player side

```text
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ TOP BAR   Suppression Prep                                            Enk 12         │
├──────────────────────────────────────────────────────────────────────────────────────┤
│                           OPPONENT SUPPRESSION STAGE                                 │
│  ┌──────────────────────────────────────────────────────────────────────────────┐    │
│  │                          BOSS / ENCOUNTER PREVIEW                           │    │
│  │                                                                              │    │
│  │  Risk: WAW            Tags: bleed / summon / fear            Rewards: 3     │    │
│  │  enemy preview slots / encounter summary / abnormality lore                 │    │
│  └──────────────────────────────────────────────────────────────────────────────┘    │
│───────────────────────────────── CENTER FRONTLINE ───────────────────────────────────│
│                                PLAYER FIELD                                          │
│  ┌────┬────┬────┬────┬────┬────┬────┐                                                │
│  │    │ U  │    │ U  │    │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │ T  │    │ U  │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │ B1 │ B2 │ B3 │ B4 │ B5 │ B6 │ B7 │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │ formation tuning / unit inspect / transfer context                 │           │
│  └────┴────┴────┴────┴────┴────┴────┘                                                │
├──────────────────────────────────────────────────────────────────────────────────────┤
│ [Bag] [Artifacts] [Selected Unit] [Enemy Info] [Move] [Transfer] [Start Suppression] │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

### 5. Battle / Replay

Design intent:

- Only in battle does the opponent half return to a true enemy field
- The overall board proportions remain unchanged

```text
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ TOP BAR   Battle / Replay                                            Speed 1.0x      │
├──────────────────────────────────────────────────────────────────────────────────────┤
│                                  ENEMY FIELD                                         │
│  ┌────┬────┬────┬────┬────┬────┬────┐                                                │
│  │    │ E  │    │ E  │    │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │ E  │    │ B  │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │    │    │    │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │    │    │    │    │    │                                                │
│  └────┴────┴────┴────┴────┴────┴────┘                                                │
│───────────────────────────────── CENTER FRONTLINE ───────────────────────────────────│
│                                  PLAYER FIELD                                        │
│  ┌────┬────┬────┬────┬────┬────┬────┐                                                │
│  │    │ U  │    │ U  │    │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │ T  │    │ U  │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │    │    │    │    │    │                                                │
│  ├────┼────┼────┼────┼────┼────┼────┤                                                │
│  │    │    │    │    │    │    │    │                                                │
│  └────┴────┴────┴────┴────┴────┴────┘                                                │
├──────────────────────────────────────────────────────────────────────────────────────┤
│ [Selected Unit] [Timeline] [0.5x] [1x] [2x] [Pause] [Restart] [Continue]             │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

## Visual Direction

### Opponent Side

- More theatrical and presentational
- Can change theme by state:
  - event stage
  - merchant stall
  - reward altar
  - suppression briefing zone
  - enemy battle field
- Stronger decorative framing is acceptable here

### Player Side

- Stable and functional
- Clear formation readability
- Strong ownership cues
- Should not be visually rebuilt every state change

### Center Frontline

- Maintain a strong horizontal divider
- This is the key line that preserves the board identity
- The frontline should stay visible even in non-battle states

## Interaction Rules

1. Avoid darkening the whole screen for event selection.
2. Avoid floating a full-screen modal over the board.
3. Keep the player field visible in event, shop, reward, and suppression prep.
4. Put primary meta choices in the opponent half.
5. Use the bottom HUD for:
   - selected entity detail
   - inventory and economy
   - actions and confirmation
6. Let the opponent half change role while the player half stays familiar.

## Unity Layout Mapping

Suggested UGUI hierarchy:

```text
MetaGameCanvas
├── TopBar
├── BoardRoot
│   ├── OpponentStageRoot
│   │   ├── OpponentFieldFrame
│   │   ├── EventStageRoot
│   │   ├── ShopStageRoot
│   │   ├── RewardStageRoot
│   │   ├── SuppressionStageRoot
│   │   └── EnemyBattleRoot
│   ├── FrontlineDivider
│   └── PlayerFieldRoot
├── BottomHudRoot
│   ├── BagPanel
│   ├── ArtifactPanel
│   ├── BenchPanel
│   ├── InventoryPanel
│   ├── SelectionPanel
│   └── ActionTray
└── OverlayFeedback
    ├── ToastRoot
    ├── ConnectionBadge
    └── LightweightTransientEffects
```

## State Mapping

- `NotStarted` / `WaitingPhaseRequest` / `SelectingEvent`
  - show player field
  - show event-style opponent stage when phase events are available
- `InShop`
  - show shop stage in opponent half
- `InBonus` / `InBonusClaimed`
  - show reward stage in opponent half
- `InSuppression`
  - show suppression prep stage in opponent half
- `InSuppressionReplay`
  - show battle / replay layout

## Final Rule

The player side is always `my board`.

The opponent side is either:

- `their board`
- or `the world-facing stage where the next decision appears`

That rule should govern every meta UI state.
