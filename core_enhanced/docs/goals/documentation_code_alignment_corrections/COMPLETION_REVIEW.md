# Documentation–Code Alignment Corrections Completion Review

Status: complete

Review date: 2026-07-18

Verdict: Complete / High

## Review Scope

This is an independent post-child review of all 12 included `DCA2-*` findings and accepted policies
P01–P06. Runtime code and live data were treated as primary evidence; child completion claims were
used only as pointers. The two explicitly excluded findings were checked for scope preservation.

## Item Audit

| Primary child | Included finding(s) | Direct evidence sampled | Verdict | Long-term fit |
| --- | --- | --- | --- | --- |
| G01 nested RON strictness | `DCA2-DATA-002` | All demonstrated nested owners use `deny_unknown_fields`; production-shaped outer-source regressions cover consumable, abnormality, skill, equipment modifier, map payload and battlefield paths. | Complete | High |
| G02 setup catalog refs | `DCA2-WIRE-003` | Core filters non-empty IDs by actual abnormality catalog membership and `EnemyKind::Abnormality`, then sorts/deduplicates; server moves the typed `catalog_refs` unchanged. | Complete | High |
| G03 Endless completion | `DCA2-TEST-001` | The named direct test constructs the final incomplete response, completes combat, asserts nested `RunComplete`, terminal state and an undelivered planning-return fragment. | Complete | High |
| G04 weapon block authority | `DCA2-GAME-001` | `WeaponCombatProfile.block_capacity` is required; all 29 current weapon profiles author it; unarmed is zero; `apply_weapon_profile` copies the exact authored value; six-archetype/armor/replacement/snapshot behavior is tested. | Complete | High |
| G05 placeholder/policy boundary | `DCA2-GAME-002`, `DCA2-BATTLE-001`, `DCA2-CONTENT-001`, `DCA2-GAME-003` | Canonical separates formal fragments/ranges from current placeholders, WhiteNight remains future/undecided, and Maintenance preview/execution requires per-item recipe lookup. Live recipes remain only `standard_armor`. | Complete | High |
| G06 grenade schema removal | `DCA2-BATTLE-002` | Current `WeaponArchetype` has six variants and no grenade variant; removed value has a fail-closed parse test; remaining mentions are explicit negative/future notes. | Complete | High |
| G07 final drift cleanup | `DCA2-DATA-001`, `DCA2-DOC-001`, `DCA2-WIRE-002` | Loader constant/test and durable docs say 21; current runtime canonical contains no `G15`/`G24`; external inventories match ten map categories and four targeting profiles. | Complete | High |

Coverage result: 12 included findings, 12 terminal `Complete` dispositions, exactly one primary child
per finding.

## Cross-Component Findings

No material cross-component defect remains.

- Required nested strictness and the new required weapon block field compose without a compatibility
  default: missing/stale authoring fails before publication and all current weapon RON is explicit.
- Weapon ownership has one base authority. Employee/unarmed starts at zero, armor does not contribute,
  and the equipped weapon profile supplies the authoritative combat/snapshot value.
- Removed/current/future vocabulary is consistent. `GrenadeLauncher` remains only in the negative
  parser regression and explicitly non-current future notes; `SplashClusterFirst` is absent from the
  current Core and Unity contracts.
- Placeholder data remains runnable without being presented as completed formal design. No fragment,
  abnormality range or WhiteNight mechanics were silently promoted or migrated.
- Maintenance's current data and runtime agree: missing recipe produces a disabled preview and
  execution fails closed; there is no generic recipe calculation.
- Core setup catalog refs are passed through by game_server. The child live WebSocket probe observed
  an empty abnormality list for a corroded-only encounter, and the final Unity EditMode suite remained
  green.

## Legacy And Fallback Audit

- No current code/live-data authority for `GrenadeLauncher` or `SplashClusterFirst` was found.
- `GrenadeLauncher` search hits are limited to fail-closed evidence and explicitly future/non-current
  documentation.
- No current `G15`/`G24` domain vocabulary or stale 23-source statement remains in the affected
  canonicals.
- No ignored test was found in the reviewed Core/server correction surfaces.
- No weapon-archetype block fallback or Maintenance generic-recipe fallback was found.

## Validation

- `cargo fmt --all -- --check`: passed.
- `cargo test -p game_core --all-targets --quiet -- --test-threads=1`: 786 passed total
  (741 library + 3 + 4 + 21 + 13 + 4 integration tests), 0 failed/ignored.
- `cargo check -p game_server`: passed.
- `cargo test -p game_server --quiet -- --test-threads=1`: 30 passed, 0 failed/ignored.
- Unity MCP full EditMode job `36c79d3dae8f4eeb91ec19ad6d056c2d`: 241 passed, 0 failed/skipped.
- Unity console error query after the run returned no compiler/runtime error; the only entry was the
  Test Runner result-save message.
- Core full and scoped `git diff --check`: passed.
- Scoped external contract/visual-doc `git diff --check`: passed. Whole external-repository checking
  still reports pre-existing CRLF whitespace in unrelated `.gitignore`/IDE files and is not attributed
  to this goal.
- Exact enum/count/legacy searches and the G07 embedded-source focused test passed.

## Exclusions And Remaining Risk

- `DCA2-WIRE-001` remains excluded exactly as approved. This goal did not weaken its current battle
  presentation contract or implement the P1 Unity presentation work.
- `DCA2-DOC-002` remains excluded. `/mnt/f/unity projects/ark/docs - 복사본` is still an untracked
  external tree and no operation in this goal targeted it.
- Formal fragment content, authored special abnormality ranges, WhiteNight mechanics, additional
  Maintenance recipes and future grenade content remain deliberate future content/policy work, not
  incomplete corrections in this goal.

## Correction And Follow-Up Candidates

- Immediate corrections: none.
- Follow-up refactors: none required by the accepted correction scope.
- Policy questions: none.

The Master may close.
