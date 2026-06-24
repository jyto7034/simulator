# Abnormality Skill Design Notes Plan

## Objective

Create `docs/skills/abnormality_skill_design_notes.md`, a source-linked design note for the 41 S/A/B priority abnormalities currently represented by live placeholder equipment and skill fragments.

Policy correction: WhiteNight and Plague Doctor are the same abnormality line. Plague Doctor is treated as WhiteNight's prelude/form, not as a separate target.

## Scope

In scope:

- Read current core docs and live data to confirm the 41 target abnormalities.
- Research one abnormality page at a time from Lobotomy Corporation Wiki/Fandom or an equivalent public reference.
- Summarize each abnormality's original identity, original mechanics, E.G.O motifs, game translation, skill fragment direction, encounter/equipment direction, implementation tier, and required core extensions.
- Add the new document to `docs/README.md`.

Out of scope:

- Rust code changes.
- Live RON changes.
- Unity DTO/contract changes.
- Numerical balancing.
- Copying long wiki prose or using source text as game text.
- Bulk crawling or opening multiple wiki pages at once.

## Target Abnormalities

S grade:

- WhiteNight
- Nothing There
- Apocalypse Bird
- Big Bird
- Judgement Bird
- Punishing Bird
- Queen of Hatred
- King of Greed
- Knight of Despair
- Mountain of Smiling Bodies
- Blue Star
- CENSORED
- The Silent Orchestra
- Melting Love

A grade:

- Der Freischutz
- The Funeral of the Dead Butterflies
- Little Red Riding Hooded Mercenary
- Big and Will be Bad Wolf
- Laetitia
- Child of the Galaxy
- The Red Shoes
- One Sin and Hundreds of Good Deeds
- Army in Black
- The Burrowing Heaven
- Alriune
- Snow White's Apple

B grade:

- Scorched Girl
- Spider Bud
- Fragment of the Universe
- Warm-hearted Woodsman
- Forsaken Murderer
- Fairy Festival
- The Little Prince
- Queen Bee
- Dream of a Black Swan
- The Dreaming Current
- The Firebird
- Yin
- Yang
- Singing Machine
- Schadenfreude

## Work Plan

1. Confirm target list from `docs/skills/skill_fragment_wiki.md`, `docs/skills/ego_equipment_wiki.md`, and live RON.
2. Create `docs/skills/abnormality_skill_design_notes.md` with method, reading guide, summary table, and per-grade sections.
3. Research S grade abnormalities one at a time.
4. Research A grade abnormalities one at a time.
5. Research B grade abnormalities one at a time.
6. Normalize implementation tier labels and core extension notes.
7. Add the document to `docs/README.md`.
8. Run verification commands.

## Completion Conditions

- `docs/skills/abnormality_skill_design_notes.md` exists.
- All 41 S/A/B priority abnormalities are included.
- Each abnormality has source URL, original identity, original mechanics, game translation, skill fragment direction, encounter/equipment direction, implementation tier, and core extension notes.
- The document uses summaries and links, not copied source prose.
- `docs/README.md` links to the new document.
- Research was performed one abnormality page at a time.
- Any policy questions are recorded rather than resolved silently.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- Wiki/Fandom access is blocked or unstable enough that source verification is impossible.
- External sources conflict on a material fact and a choice of source requires user approval.
- Translating a mechanic requires deciding gameplay failure/reward/run rules beyond existing policy.
- Target inclusion/exclusion becomes ambiguous.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Verification Commands

```sh
rg -n "abnormality_skill_design_notes" docs/README.md docs/skills/abnormality_skill_design_notes.md
rg -n "TODO|TBD|Open Questions" docs/skills/abnormality_skill_design_notes.md
rg -n "^### " docs/skills/abnormality_skill_design_notes.md
```

## Completion Audit

- `docs/skills/abnormality_skill_design_notes.md` exists.
- Summary table includes 41 S/A/B rows after the Plague Doctor consolidation. Verified with `rg -c "^\\| [SAB] \\|" docs/skills/abnormality_skill_design_notes.md`.
- Detailed sections include 41 abnormality sections after the Plague Doctor consolidation. Verified with `rg -c "^### " docs/skills/abnormality_skill_design_notes.md`.
- PLAN target names and design-note section names match exactly. Verified with `comm -3` on sorted target/section lists.
- Each section uses the shared template: source, original identity, original mechanics, game translation, skill fragment direction, equipment/encounter direction, implementation tier, core extension notes, and open questions. Verified with a 369 heading count, equal to 41 entries x 9 headings.
- Each entry has a Lobotomy Corporation Wiki/Fandom source URL. Verified with a 41 source-line count.
- `docs/README.md` links to the new design note and this goal.
- Wiki access was performed one abnormality page at a time.
- Open questions remain intentionally recorded for later implementation goals; none block this documentation goal because this goal does not implement those policies. Verified with 41 `Open Questions` headings.
- No `TODO`/`TBD` remains in the design note.
- No blockquote-style copied source prose was found in the design note.
