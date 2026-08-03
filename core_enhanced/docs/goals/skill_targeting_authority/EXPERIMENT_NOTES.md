# Skill Targeting Authority Notes

Goal: G10

Status: complete

## Guardrails

- Do not change per-skill intended target choices while making authority explicit.
- Do not implement G09 Global reach/footprint migration in this child.
- Keep G11 opponent preview removal and Player preview surfaces fixed.
- Prefer one reusable typed selector/preset domain over copied weapon-profile fallback logic.

## Final Findings

- The skill schema and runtime were already structurally skill-owned; the remaining implicit seam
  was `EnemySingle.rule` defaulting to `Nearest` when omitted.
- `EnemySingle.rule` is now required and unknown competing authority fields fail deserialization.
  The only cast-targeting enum variant remains `Explicit`; no legacy weapon-inheritance variant is
  accepted.
- All 25 live skills already authored their intended selection. A complete inventory test now pins
  each id to its exact self/nearest/current-target/lowest-health choice.
- Runtime proof changes the caster weapon profile across conflicting target candidates while the
  skill continues to obey `LowestHealthEnemy`; the existing basic-attack test preserves weapon
  profile authority.
- Unity has no local selection fallback or target-policy consumer. Its canonical catalog wording
  now treats any projected `target_policy` as skill-owned metadata.
- Final verdict: Complete / High / no immediate correction.
