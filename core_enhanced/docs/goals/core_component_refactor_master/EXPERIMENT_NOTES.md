# Core Component Refactor Master Notes

## User Intent

The user wants to read every component directly and document refactor findings for each one.

Therefore this master goal should not imply that all implementation sub goals already exist or will be automatically generated without inspection. The primary output is one `<component_name>_refactor.md` audit document per component.

## Refactor Criteria To Preserve

- Source-of-truth duplication is a primary refactor target.
- If a feature can be expressed by composing existing mechanics, special-case code should be audited.
- Canonical refactor criteria for every component audit are fixed to `docs/refactor_preparation_plan.md`.

Example:

- A projectile rule such as "despawn on void tile" may not need bespoke logic if the same gameplay intent can be represented by existing long range plus valid-tile clipping. The audit must check whether this kind of simplification is semantically valid before proposing a refactor.

## Worktree Safety

The worktree is already dirty. Treat unrelated modifications as user changes.

Do not revert, restore, delete, or normalize unrelated files while performing component audits.

## Policy Discussion Items

Do not decide unilaterally when a candidate requires:

- Unity-facing DTO shape changes.
- live RON schema changes.
- saved data migration.
- UX meaning changes.
- balance criteria.
- existing content deletion or replacement.
- failure, reward, or consumption timing changes.

Do not stop the component audit just to ask immediately. Record the item in the relevant `<component_name>_refactor.md` document with the exact phrase `사용자와 정책 논의 필요`, then continue to the next audit item or component.

## Follow-up Candidate Policy

If a useful improvement is outside the current component scope, record it here as a follow-up candidate.

Only apply out-of-scope improvements when they are necessary to complete the current goal or materially reduce blast radius.
