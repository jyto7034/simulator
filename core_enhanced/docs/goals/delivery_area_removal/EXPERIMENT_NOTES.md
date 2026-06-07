# Delivery Area Removal Notes

- Do not preserve geometric area tests with `#[ignore]`.
- If compile failures reveal a non-skill official gameplay dependency on `SkillAreaShapeDef`, stop and ask.
- `spatial.rs` still needs projectile/collision helpers, but no longer needs a configurable spatial query backend after geometric skill area query removal.
