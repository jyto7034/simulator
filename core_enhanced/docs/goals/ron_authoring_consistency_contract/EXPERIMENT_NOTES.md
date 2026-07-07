# Experiment Notes: RON Authoring Consistency Contract

## Policy Decisions

- Use `deny_unknown_fields` for new/core live RON schemas where practical.
- Reject stale gameplay fields instead of preserving aliases.
- Gameplay-meaningful fields should be explicit unless there is a documented default.
- Harmless display/optional fields may keep explicit defaults.
- Enum casing should be consistent within each data domain and documented.

## Initial Risk Notes

- Applying `deny_unknown_fields` everywhere at once can create a large migration. Prefer domain-by-domain strictness with tests.
- Some data structs may deserialize test fixtures or domain-owned builtins rather than live RON. Classify before editing.
- Enum casing may already be constrained by serde attributes or existing live authoring. Do not blindly normalize all data in one patch.
- `docs/data_loading_contract.md` is not present in the current tree. Use `docs/core_runtime_contract.md` as the canonical implementation-contract location for this goal rather than creating a new document.
- PvE and event enum variants use Rust variant casing in current live RON unless a type explicitly declares `rename_all`. Corroded employee profile role is snake_case because the enum declares `#[serde(rename_all = "snake_case")]`.

## Follow-Up Candidates

- A dedicated authoring guide for content writers may be useful after this goal identifies the final casing/default conventions.
- If many schemas need strictness, split follow-up goals by domain: skills, PVE encounters, map data, rewards/shops, run policy.
- Abnormality, equipment, artifact, consumable, reward metadata, and employee candidate schemas still contain live authoring structs that may be worth tightening in follow-up domain goals. They were not all changed here to avoid a broad migration without domain-specific tests and policy review.
- `BasicAttackDef` still has gameplay defaults such as `range_units`, `range_policy`, `damage_type`, and timing fields. Some defaults are intentional runtime fallbacks today; tightening them should be handled in a separate combat-authoring goal so weapon/abnormality/corroded employee policy can be reviewed together.
