# Experiments

| Date | Attempt | Result | Follow-up |
|---|---|---|---|
| 2026-06-23 | Goal setup | Created the subgoal document for runtime unit lifecycle. | Run after `core_battlefield_layout_extraction`. |
| 2026-06-23 | Runtime lifecycle implementation inventory | Reading withdrawal/death cleanup showed that withdrawal-specific buff/cast cleanup cannot be represented precisely by current event-log contracts. | Complete with `POLICY_DECISION_REPORT.md` before implementation. |
| 2026-06-23 | Runtime lifecycle implementation resume | Added `RuntimeUnitLifecycle`, kept withdrawn/dead units in `BattleCore.units`, converted active gameplay candidate filters to `is_active()`, and made death set lifecycle/HP/action invariants. | Continue completion review before moving to redeploy lifecycle. |
| 2026-06-23 | Withdrawal cleanup events | Implemented `BuffExpireReason::{TargetWithdrawn,CasterWithdrawn}` and `BattleLogEvent::SkillCastCancelled { reason: Withdrawn }` for pending and active skill runtime cleanup. | Keep `SkillCastInterrupted` external-interruption-only. |
| 2026-06-23 | Test compile failure: stale layout occupant query | `cargo test` initially failed because `tests/skill_test/common/run.rs` still called removed `BattlefieldLayout::units_at`. | Replaced the fixture lookup with `BattleCore.units` + `RuntimeUnit.body.projected_tile()` active-unit scan; `cargo test --no-run` then passed. |
| 2026-06-23 | Broad test failure: HP-only dead fixtures | `cargo test` initially failed in tests that set only `stats.current_health = 0` to model death. | Updated fixtures to set `RuntimeUnitLifecycle::Dead`; `cargo test --lib` then passed. |
| 2026-06-23 | Focused and broad validation | `cargo check`, `cargo test --no-run`, `cargo test withdraw_`, `cargo test validator_accepts_cancelled_active_skill_cast_referencing_ability_cast`, `cargo test --lib`, and `cargo test` passed. | Run subgoal completion review using `docs/goal_completion_review_guide.md`. |
| 2026-06-23 | Completion review | Created `REVIEW.md`; verdict complete / high long-term fit for runtime lifecycle scope. | Update master goal and proceed to the next subgoal after user-visible progress report. |
