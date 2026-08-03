# Server RunStart Fixture Contract Experiment Notes

Status: complete

Implementation complete: true

Created: 2026-07-12

## Structural Judgments

### Fixture validity is the owned boundary

G04 does not own RunStart gameplay semantics. It owns the validity and reuse boundary of server test data that claims to represent a startable Standard game.

The long-term direction is one reusable construction seam that supplies the minimal valid Visual Novel definition and trigger under the same Core validation used by production. Adding the same literal separately to six tests or bypassing validation would preserve the source-of-truth split and is not acceptable.

### Production validation stays authoritative

The observed error is useful: it proves the fixture no longer satisfies a production invariant. The fix must bring the fixture forward. It must not make missing RunStart triggers optional, insert a runtime fallback, or special-case tests.

### Dirty worktree overlap

The current worktree contains many existing user changes and untracked Visual Novel files. `../game_server/src/game/player_game_actor/handlers.rs`, the reported fixture owner, is already modified. Before patching it, inspect the exact current diff and surrounding code, preserve unrelated changes, and use the smallest semantic patch at the fixture seam.

## Source-Of-Truth Conflicts

- The historical audits describe a fixture that contains a Visual Novel definition but no RunStart trigger. This must be revalidated against current code before relying on it.
- Audit severity differs (`DCA-H-08` High in the snapshot table but normalized to Medium in the Master; `CPR-M-01` is the same underlying issue). The Master registry's normalized ownership and priority govern orchestration; the actual failing regression gate governs implementation urgency.
- Goal documents are not proof that the current worktree still fails. Current test output is required.

Current-source audit on 2026-07-12:

- `GameDataBuilder::empty()` intentionally owns an empty `VisualNovelTriggerDatabase`.
- The server starter fixture adds matching Standard/Endless Visual Novel definitions but no triggers.
- Core's current `validate_no_duplicate_run_start_triggers` proves only “at most one per mode”; the runtime start path still proves presence and reports the focused error.
- Live RON supplies both supported modes with the accepted trigger policies.
- Core world tests contain a correct private paired test fixture, but exposing a `#[cfg(test)]` Core helper across crates would invert test ownership. The public builder method is the appropriate seam.

The G04 correction will therefore keep one paired definition/trigger helper inside the server test module. Broad supported-mode cardinality and policy validation remains a G12 follow-up. R00 must not claim that CPR-M-01's late-invariant aspect is fully closed merely because the six fixtures are green.

## Policy Questions

None currently. The accepted policy is to retain the mandatory Standard RunStart trigger and repair the fixture.

Escalate only if current code shows that fixture repair requires a new trigger cardinality, live schema/content change, DTO change, or production loader policy.

## Cross-Goal Dependencies

- G01, G06, and G07 depend on a trustworthy server baseline.
- R00 must verify G04 evidence before Phase 1 starts.
- G12 later owns broad RunStart cardinality and generated-bundle strictness; G04 must not pre-implement that Phase 1 validation campaign.
- Current Visual Novel code is user-owned dirty worktree context, not G04 cleanup scope.

## Follow-Up Candidates Outside G04

- A cross-repository Rust-generated JSON fixture format shared with Unity belongs to the later owning DTO/Unity goals unless G04 can identify an existing path read-only.
- Broad live-bundle RunStart cardinality validation belongs to G12 if current production startup does not already enforce it at bundle validation time.
- Battle reconnect/capability semantics exposed after the fixture is repaired belong to G07, even if newly reachable tests reveal failures.

## Alternatives Not Chosen

- Relaxing or deleting the mandatory RunStart lookup: rejects the accepted production contract.
- Adding a test-only bypass in `GameCore::start`: makes production and fixture behavior diverge.
- Inserting trigger literals independently into each failing test: duplicates fixture authority and invites another drift.
- Loading the complete production bundle for every focused server unit test by default: may hide fixture ownership and make tests slow/coupled; consider only if current builders prove a narrower canonical seam impossible.

## Implemented Boundary

The server test module now owns one paired fixture rather than separate unrelated literals. `starter_policy_game_data_builder()` cannot add the pre-run Visual Novel definitions without also adding the matching Standard and Endless RunStart triggers.

This deliberately remains test-fixture code. It does not add a production fallback or expose Core's private world-test helpers across crate boundaries. Defense-derived fixtures may replace the paired Visual Novel database with the live database while retaining the same accepted IDs and trigger semantics; broad validation must prove that composition remains valid.

## Validation And Runner Classification

- The exact Unity Editor matching `ProjectVersion.txt` exists outside PATH at `C:\Program Files\Unity\Hub\Editor\6000.3.4f1\Editor\Unity.exe`.
- Unity test entry assemblies are `ArkClient.EditModeTests` and `ArkClient.PlayModeTests`; the project uses Unity Test Framework 1.6.0.
- G04 has no Unity code, DTO, or asset change, so Unity execution is not mandatory evidence for the fixture-only correction. The runner remains available for later cross-repository phases.
- The first focused command executed zero tests because `--exact` lacked the module path. It is classified as invalid evidence and was replaced by nonzero filters.
- The server test `player_state_snapshot_preserves_core_snapshot_root_fields` is the proven real Rust serialization path for the Phase 0 baseline. It serializes typed Core and server snapshot DTOs rather than comparing two mocks.
