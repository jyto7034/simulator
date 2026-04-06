# Meta UI Layout Plan

## Goal

Build a React mockup for the Unity meta-game UI using a field-centric layout inspired by:

- `The Bazaar` for meta-game presentation, event/reward staging, and economy surfaces
- `TFT` for the battle board feeling, formation readability, and two-side field composition

This is not a side-panel app layout.
The screen should feel like a strategy game board first, with UI layered around that board.

## Core Layout Principles

1. The field is the visual center of the screen.
2. The top logical stage belongs to the opponent side.
3. Event choices and reward choices appear in the opponent stage, not in a separate menu screen.
4. The player field remains visible across states whenever possible.
5. Bottom HUD contains economy, inventory, artifacts, profile, and primary action controls.
6. State changes should mainly swap the content of the opponent stage, while preserving the player field and bottom HUD structure.

## Logical Screen Structure

The layout is divided into three logical bands, but they should still compose into one board-centered screen.

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                         OPPONENT / ENCOUNTER STAGE                          │
│ enemy formation OR event choices OR reward choices OR suppression preview   │
├──────────────────────────────────────────────────────────────────────────────┤
│                              PLAYER FIELD                                   │
│ player formation board, bench relation, placement readability               │
├──────────────────────────────────────────────────────────────────────────────┤
│                              BOTTOM HUD                                     │
│ bag, artifacts, profile, currency, inventory, bench, action controls        │
└──────────────────────────────────────────────────────────────────────────────┘
```

## State-Specific Layouts

### Event Selection

- Opponent stage shows three event cards
- Player field stays visible below
- Bottom HUD shows current resources, artifacts, bench, and selection actions

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                    [ Event A ]  [ Event B ]  [ Event C ]                    │
│                  shop         bonus        suppression                       │
│                    selected event detail / flavor text                      │
├──────────────────────────────────────────────────────────────────────────────┤
│                              player grid                                    │
│                          current formation visible                          │
├──────────────────────────────────────────────────────────────────────────────┤
│ bag | artifacts | bench | enkephalin | profile | [select] [inspect]        │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Shop

- Opponent stage becomes the shop offering stage
- Player field remains stable for comparison and planning
- Bottom HUD contains inventory, currency, equip context, and shop actions

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│             [ Item ] [ Item ] [ Item ] [ Item ] [ Item ]                    │
│                     selected item detail / compare panel                    │
├──────────────────────────────────────────────────────────────────────────────┤
│                              player grid                                    │
│                          current formation visible                          │
├──────────────────────────────────────────────────────────────────────────────┤
│ bag | artifacts | inventory | bench | gold/enkephalin                      │
│ [refresh] [buy] [sell] [equip] [unequip] [exit shop]                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Reward Selection

- Opponent stage shows reward cards
- Player field remains as build context
- Bottom HUD handles confirmation and build impact

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                 [ Reward ] [ Reward ] [ Reward ]                            │
│                    selected reward detail / rarity / effect                 │
├──────────────────────────────────────────────────────────────────────────────┤
│                              player grid                                    │
│                          current formation visible                          │
├──────────────────────────────────────────────────────────────────────────────┤
│ bag | artifacts | bench | profile | reward mode                            │
│ [take reward] [skip] [continue]                                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Suppression Preparation

- Opponent stage shows boss or enemy composition preview
- Player field is the main preparation surface
- Bottom HUD shows formation info, resources, and start action

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                    boss / enemy formation preview                           │
│                  risk / tags / rewards / encounter summary                  │
├──────────────────────────────────────────────────────────────────────────────┤
│                              player grid                                    │
│                        active formation and placement                        │
├──────────────────────────────────────────────────────────────────────────────┤
│ bag | artifacts | bench | selected unit | enkephalin                       │
│ [move] [transfer] [inspect enemy] [start suppression]                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Replay / Battle View

- Opponent stage returns to enemy field
- Player field remains the lower half of the board
- Bottom HUD becomes thinner and more replay-focused

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                               enemy field                                   │
├──────────────────────────────────────────────────────────────────────────────┤
│                               player field                                  │
├──────────────────────────────────────────────────────────────────────────────┤
│ replay info | speed | selected unit detail | [play] [pause] [continue]     │
└──────────────────────────────────────────────────────────────────────────────┘
```

## React Mockup Architecture

### Top-Level Structure

```text
App
└── MetaGameMockShell
    ├── AtmosphereFrame
    ├── TopStatusBar
    ├── BoardViewport
    │   ├── OpponentStage
    │   │   ├── EventStage
    │   │   ├── ShopStage
    │   │   ├── RewardStage
    │   │   ├── SuppressionStage
    │   │   └── EnemyBoardStage
    │   └── PlayerBoard
    ├── BottomHud
    │   ├── BagPanel
    │   ├── ArtifactPanel
    │   ├── BenchPanel
    │   ├── ResourcePanel
    │   ├── ProfilePanel
    │   └── ActionTray
    └── OverlayFeedback
        ├── CommandToast
        ├── ConnectionBadge
        └── SelectionDetails
```

### Mock State Shape

The first mockup should be driven by local state, not live server wiring.

Suggested mock state groups:

- `screen`: start, phase, event_selection, shop, bonus, suppression_prep, replay
- `resources`: enkephalin, qliphoth, ordeal, phase
- `playerBoard`: placed units, reserve units, highlighted unit
- `opponentStage`: enemy preview, event cards, reward cards, shop cards
- `economy`: bag, artifacts, inventory, bench
- `ui`: selected card, selected unit, notifications, command status

## Implementation Plan

### Phase 1: Field-Centric Shell

Goal: lock the overall composition before filling in detail

- Replace the starter Vite page
- Build the screen frame and atmospheric background
- Add `TopStatusBar`, `OpponentStage`, `PlayerBoard`, and `BottomHud`
- Establish responsive spacing and board-centered proportions

### Phase 2: Event and Reward Stages

Goal: validate the Bazaar-style stage behavior

- Implement `EventStage`
- Implement `RewardStage`
- Add card selection highlighting
- Add bottom HUD selection summary and CTA buttons

### Phase 3: Shop and Suppression Stages

Goal: validate economy and preparation flow

- Implement `ShopStage`
- Implement `SuppressionStage`
- Add item detail panel and compare area
- Add formation summary and suppression CTA

### Phase 4: Replay and Enemy Board Stage

Goal: validate the TFT-like board feeling

- Implement `EnemyBoardStage`
- Implement lighter replay HUD
- Add basic board tokens, health bars, and targeting emphasis

### Phase 5: Shared Polish

Goal: make the mockup useful as a Unity implementation reference

- Normalize spacing, card sizing, and typography
- Add hover, selected, disabled, and active states
- Add visual hierarchy for economy and artifact areas
- Ensure desktop-first layout still degrades well on smaller widths

## Immediate Execution Order

The recommended build order inside `mock_client`:

1. `MetaGameMockShell`
2. `TopStatusBar`
3. `BoardViewport`
4. `PlayerBoard`
5. `BottomHud`
6. `EventStage`
7. `RewardStage`
8. `ShopStage`
9. `SuppressionStage`
10. `EnemyBoardStage`

## Success Criteria

The mockup is on track when:

- the board is visually dominant
- event and reward choices feel like they happen on the opponent side
- the player field remains readable across states
- the bottom HUD consistently communicates economy and ownership
- the screen reads as a game table, not a dashboard or terminal app
