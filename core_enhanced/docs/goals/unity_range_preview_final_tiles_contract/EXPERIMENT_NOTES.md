# Unity Range Preview Final Tiles Contract Notes

## Policy Decisions

- Unity must draw range overlays from core-resolved final cells, not from intermediate fields.
- `defense_tile_range` remains the internal/RON authoring representation.
- Basic attack and active skill previews are separate DTO concepts.
- Default basic attack fallback is own tile plus one tile forward.
- Manual tile targeting should remain possible as a future explicit skill mode, but current live skills do not require Unity to implement it now.
- Core should include availability/reason metadata so empty previews are not ambiguous.
- Debug source metadata is allowed, but Unity must not use it for range calculation.

## Implementation Judgement

The long-term direction is not to delete internal range patterns. The better split is:

- data authors write compact range patterns
- core resolves final cells with current battle state
- Unity displays the resolved cells

This avoids duplicating range projection, facing rotation, battlefield validity filtering, and fallback rules in Unity.

The current core has no existing pending-deployment preview command or pending-placement DTO. The plan only authorizes using a pending preview surface "if such a DTO already exists", so this goal adds a core helper `deployment_range_preview_dto(employee_uuid, position, facing)` but does not introduce a new `PlayerBehavior` command or WebSocket message.

The local `core_enhanced/docs` directory does not contain `core_unity_battle_transport_contract.md` or `unity_core_contract.md`; README points to the external Unity project as canonical. Within this repository, `skill_target_contract.md`, `game_rulebook.md`, and `docs/README.md` were the relevant docs to update. The external Unity canonical docs were also updated because they still contained stale `defense_tile_range`-as-preview-source wording.

## Questions To Stop For

Stop and ask the user if implementation discovers any of these:

- A live skill already requires player-selected tile targeting.
- Current Unity-facing DTOs cannot carry range previews without a broader command/snapshot redesign.
- A range rule depends on UX meaning rather than runtime facts, such as whether to preview target-dependent splash before target selection.
- Supporting final cells requires changing live RON schema or deleting existing content.
- A compatibility layer or dual schema appears necessary.

## Follow-up Candidates Outside This Goal

- Full manual tile-targeting UX and validation.
- Ground-fixed persistent area skills.
- Skill-specific target-dependent effect preview after Unity can select/hover concrete targets.
- Better authoring tools for visualizing `defense_tile_range` presets.
- Removing obsolete Unity-side range calculation once the client has migrated.
