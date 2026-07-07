# Goal: RON Authoring Consistency Contract

## Objective

Make live RON authoring stricter and more consistent across core data domains without introducing broad compatibility layers.

This goal focuses on authoring clarity:

- Unknown fields should fail for new/core live schemas.
- Enum/string casing should be documented and consistent within a domain.
- Gameplay-meaningful fields should be explicitly authored instead of silently defaulted.
- Simple presentation/optional fields may keep explicit defaults when that is genuinely harmless.

## Source Of Truth Order

Verify facts in this order:

1. Actual runtime code and raw RON structs.
2. Live RON/data.
3. Data validation and live loading tests.
4. Unity-facing snapshot/command contract if a RON field reaches DTO output.
5. Latest canonical docs.
6. Historical goal/audit docs only as context.

Relevant starting points:

- `src/game/data/**`
- `src/game/ability.rs`
- `src/game/pve.rs`
- `src/game/map/**`
- `src/game/combat_preview/**`
- `../game_resources/data/**`
- `tests/ron_loading.rs`
- `docs/data_loading_contract.md`
- `docs/core_runtime_contract.md`
- `docs/game_rulebook.md`

## Fixed Policies

- New and core live RON schemas should use `#[serde(deny_unknown_fields)]`.
- Unknown gameplay fields must not be silently ignored.
- Fields that change gameplay meaning must be explicit unless there is a documented, intentional default.
- Defaults are allowed for simple display or optional authoring convenience fields when missing values do not change gameplay semantics.
- Enum casing must be consistent within each data domain and documented where authors need to know it.
- Do not convert old field names into aliases unless the user approves a short transition with a removal condition.
- Live RON loading must catch authoring mistakes before gameplay starts.

## Plan

1. Inventory raw RON schemas.
   - Find structs that deserialize live RON and whether they use `deny_unknown_fields`.
   - Identify enum casing conventions currently used in live data.
   - Identify gameplay-meaningful defaults.
   - Identify stale aliases or previously removed fields still accepted by raw schema.

2. Classify domains.
   - Core gameplay data: encounters, waves, enemies, skills, buffs, items, run policy, map nodes/templates.
   - Presentation/content metadata: names, descriptions, hints, visual ids.
   - Test-only or builder-only fixtures.
   - Domain-owned builtins that are not part of `GameDataBase::load_live_embedded()`.

3. Apply strictness incrementally.
   - Add `deny_unknown_fields` to core live raw schemas where live data and tests can support it.
   - Remove stale accepted fields that are no longer policy.
   - Keep harmless defaults only with explicit code names and tests.
   - Add focused tests that stale fields are rejected for each touched domain.

4. Document authoring rules.
   - Update the closest canonical doc, likely `docs/data_loading_contract.md` or a section linked from it.
   - If a rule is gameplay-facing, link from `docs/game_rulebook.md` or `docs/core_runtime_contract.md` instead of burying it in a goal doc.

5. Validate live data.
   - Run full live RON loading tests.
   - Run focused tests for domains touched.
   - Run `cargo check -p game_core` and `cargo fmt --check`.

## Completion Conditions

- Core live RON schemas touched by this goal reject unknown fields.
- Stale authoring aliases found in this goal are removed or explicitly reported as blocked by policy.
- Gameplay-meaningful defaults are removed or documented with tests.
- Enum casing expectations are documented for touched domains.
- Live RON loading succeeds with current data.
- Tests prove at least one stale/unknown field rejection for each domain modified.
- `EXPERIMENTS.md` records failed attempts, fixes, and reruns.
- `EXPERIMENT_NOTES.md` records domains intentionally left for follow-up.

## Stop Conditions

Stop and report questions if:

- A missing/defaulted field changes gameplay meaning and no policy says what explicit value should be authored.
- Live RON contains many unknown/stale fields whose removal implies content deletion or rebalance.
- Enum casing standardization would require mass data migration across many domains.
- A raw schema is shared with external save files or Unity-facing JSON and strictness would be a breaking migration.
- A domain already has a different intentional authoring convention.

## Suggested Verification Commands

```bash
rg -n "derive\\(.*Deserialize|Deserialize\\)|deny_unknown_fields|default\\)|serde\\(default|rename_all|alias|flatten" src/game ../game_resources/data tests
rg -n "ALEPH|WAW|TETH|ZAYIN|boss|Boss|Common|common|snake_case|PascalCase" ../game_resources/data src/game docs

cargo test -p game_core --test ron_loading -- --test-threads=1
cargo test -p game_core live_ron --lib -- --test-threads=1
cargo test -p game_core validation --lib -- --test-threads=1
cargo check -p game_core
cargo fmt --check
```
