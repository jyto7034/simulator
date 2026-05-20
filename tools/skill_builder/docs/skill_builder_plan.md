# Skill Builder Plan

## Purpose

The skill builder is a local desktop tool for creating and editing game skills without writing RON by hand.

The target user is a non-programmer designer who understands combat intent, timing, targets, effects, and presentation, but should not need to know Rust enum syntax, RON formatting, or internal validation rules.

The tool should make a skill readable as:

- when each step happens
- who the step targets
- how the step is delivered
- what effects are applied
- what VFX or presentation hints are attached
- whether the skill is valid before export

## Product Direction

The main editor should be a skill timeline editor, not a free-form node graph.

Current skill data is hierarchical:

```text
Skill
  Step[]
    Condition
    Repeat
    Target
    Delivery
    Effect[]
    Presentation
```

Because of that shape, a timeline plus readable cards is easier for non-programmers than a node graph. The editor can still be visual:

- a horizontal timeline for step timing
- cards for each step
- compact chips for target, delivery, effects, and presentation
- a board preview for area/range shapes
- a validation panel with plain-language errors
- a read-only RON preview for technical review

Graph visualization can be added later as a separate relationship view for:

- abnormality -> skill
- item/artifact trigger -> skill
- skill -> buff/effect references

## Recommended Stack

- Tauri for the desktop shell and local file access
- React + Vite + TypeScript for the UI
- Tailwind CSS for fast, consistent tool UI styling
- Zustand for editor state once the draft model grows
- React Hook Form + Zod for user-facing form validation
- @dnd-kit for step/effect reorder interactions
- Monaco Editor or CodeMirror for read-only RON preview
- Rust commands for loading, validating, and exporting skills

MVP can avoid the heavier libraries at first. Start with React state and Tailwind, then add Zustand/Zod/DnD once the data flow is stable.

## MVP Scope

1. Load skill data from `game_resources/data/skills/base.ron`.
2. Display a searchable skill list.
3. Show the selected skill as a timeline of steps.
4. Display target, delivery, effects, conditions, repeat rules, and presentation as readable cards.
5. Provide a basic property panel for selected skill/step data.
6. Show a read-only RON preview.
7. Add a validation panel.
8. Export generated RON to a chosen path.

The first implementation can use representative sample data in the frontend while the Tauri backend commands are wired.

## Data Model Strategy

Do not expose raw RON as the editing model.

Use a frontend-friendly `SkillDraft` model:

```text
SkillDraft
  id
  name
  kind
  focusTimeMs
  focusPermissions
  steps

SkillStepDraft
  id
  delayMs
  rangeTiles
  condition
  repeat
  target
  delivery
  effects
  presentation
```

The app should convert:

```text
SkillDraft -> SkillDef -> RON
```

This keeps the UI stable even if RON formatting or Rust serde details change.

## Validation Strategy

Validation should run in two layers.

UI validation:

- required IDs and names
- positive durations and ranges
- at least one step
- at least one effect per step when appropriate
- obvious incompatible combinations

Core validation:

- deserialize/serialize through `game_core` types
- construct `SkillDatabase`
- validate known buff references
- validate projectile and area runtime contracts
- validate duplicate skill/step IDs

Core validation errors should be translated into designer-facing messages.

## Screen Layout

```text
+------------------+-----------------------------+----------------------+
| Skill List       | Skill Timeline              | Inspector            |
| search/filter    | step cards by delay         | selected skill/step  |
| create/duplicate | timing/effect summaries     | editable properties  |
+------------------+-----------------------------+----------------------+
| Validation       | RON Preview                                        |
+------------------+----------------------------------------------------+
```

## Implementation Phases

### Phase 1: Static App Skeleton

- Create `tools/skill_builder` with the official Tauri scaffold.
- Add Tailwind CSS.
- Build the main layout.
- Add sample `SkillDraft` data.
- Render skill list, timeline cards, inspector, validation panel, and preview.

### Phase 2: File Loading

- Add Tauri command to read a RON file as text.
- Add a Rust loader command that returns parsed skill summaries.
- Surface file/path errors in the UI.

### Phase 3: Editing

- Add create, duplicate, delete skill.
- Add create, duplicate, delete step.
- Add basic property editing for skill and step fields.
- Keep UI state as `SkillDraft`.

### Phase 4: Conversion and Export

- Convert `SkillDraft` to `SkillDef`.
- Serialize to RON.
- Write export output to a selected file.
- Create backups before overwrite.

### Phase 5: Core Validation

- Reuse `game_core` skill validation from Rust.
- Return structured validation messages.
- Highlight invalid fields and affected steps.

### Phase 6: Previews

- Add range/area board preview.
- Add timeline event preview for skill steps.
- Later, run a small battle sandbox and display generated timeline output.

## Open Decisions

- Whether to add the Tauri crate to the root Rust workspace immediately or keep it standalone under `tools/skill_builder/src-tauri`.
- Whether generated RON should overwrite `base.ron` or export to a separate generated file at first.
- How buff/VFX catalogs should be sourced before the client meta database is complete.
