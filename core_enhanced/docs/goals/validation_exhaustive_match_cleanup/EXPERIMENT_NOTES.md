# Validation Exhaustive Match Cleanup Notes

## Initial Judgment

This is not a gameplay redesign. It is validation hardening.

Broad wildcard branches are risky in validation code because they make new enum variants compile without a validation decision. The long-term direction is explicit exhaustive matching.

## Guardrails

- Do not change skill/effect gameplay behavior unless validation reveals a real policy gap.
- Do not "fix" live RON by weakening validation.
- Do not replace broad wildcard with another hidden fallback.

## Scope Note

Runtime event-log validators still use wildcard branches to ignore unrelated `BattleLogEvent` variants. Those are not live RON skill/effect schema validators, so this goal leaves them untouched.

## Follow-Up Candidates

- Apply the same exhaustive-match standard to non-skill data validation if this goal finds the pattern elsewhere.
