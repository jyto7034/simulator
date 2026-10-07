# Phase 44 CRA-P10-C06 Experiment Notes

Status: complete
Implementation Complete: true

## Structural Decision

- This is package/target/dependency cleanup, not gameplay refactoring or a build-system redesign.
- Historical dependency/target names are discovery keys; current Cargo metadata, source/cfg graph and consumers must
  be reconstructed after sole activation.
- Retained library/server/tool behavior and fixture output are stronger authorities than manifest minimalism.

## Direct Dependencies

- Phase 43 `Complete / High` is the exact linear predecessor; its audit hash is `a9b674b4...`.
- P10-C01~C05, P06-C07, P09-C07 and P10-C07 are complete and fix the retained command/wire/log/range/test surfaces.
- Master final audit remains separate after Phase 44 completion.

## Policy And Compatibility

- CRA-POL-006 normally preserves public/package/tool compatibility until consumer evidence exists.
- The user approved every remaining registered cleanup through Phase 44, including verified package/tool/public
  breaking removal after candidate inventory and retained controls, without a shim.
- Approval does not authorize fixture output, command-name, feature, gameplay/domain library API, server behavior or
  runtime changes; it authorizes only the separately inventoried unconsumed logger/init public package surface here.

## Dirty Worktree

- Pre-Child expanded status is 603 rows at `1510b29c...`; existing rows are user/completed-campaign work.
- Phase 44 path did not exist. The three initialized documents are the only expected pre-activation additions.
- Candidate manifests/config/source/tools may already be dirty; current paths/hashes/diffs remain unopened until
  sole activation and prospective fingerprinting.
- Protected marker `game_server/logs/app.log.2026-08-09` must remain 57 bytes at `614d2fbd...`.

## Current Evidence Gate

After activation, filename-only discovery fixes prospective manifest/config/root/bin/example/test/script/doc/
external/canonical fingerprints before content inspection. Current discovery then accounts for every Cargo target,
dependency edge, feature/config effect, logger/hook, generator command/output and accessible consumer.

## Current Disposition

Unresolved until activation and current inventory. Retain every surface by default. Remove or normalize only proven
package noise under the user's advance approval, with exact before/after metadata, feature, fixture and both-profile
controls.

The three initialized files are the only status-set additions: 606 rows at `1d7fae2a...`. Phase 44 is now the sole
active Child. Active-document reread and prospective filename/hash capture remain mandatory before discovery.

Prospective fingerprints are fixed. Every workspace/Core/server manifest, lock/config, Core main/lib and five Core
fixture-bin roots is clean; the member-local workflow is clean. Package canonicals are already dirty, and only the
server battle-revision example is dirty among the three server generator examples. Current content discovery can
proceed with exact clean-path attribution.

Current metadata and compiled-source inventory prove the bounded cleanup: 38 normal + four inert build dependencies
reduce to eight positively consumed normal edges; the live feature remains. The empty default bin and five zero-test
fixture harnesses are package noise, but the five fixture implementations, names and stdout are retained. Their exact
pre-change output hashes/byte counts are fixed in E04. No workspace or accessible sibling consumer imports the Core
logger or invokes the default binary; the server logger is a separate type. The user-approved no-shim removal may now
proceed on the exact clean candidate paths.

The bounded cleanup is implemented and validated. Core now has one library, five explicit non-test fixture bins, no
default bin/build edges/member rustflag/logger ctor, and eight positive normal dependencies. Cargo reduced the lock
from 524 to 311 packages. All five stdout hashes and both package test-name hashes are exact; Core/server debug and
release suites, workspace debug/release/locked checks, the retained feature check, normal Clippy, fmt and diff checks
pass. Expanded status is the expected 611 rows (activation 606 + five clean-path changes); protected marker and all
retained tool/workflow hashes are exact. Independent Child audit is the only remaining Phase gate.

The independent 210-line audit at `e27e8b23...` returns `Complete / High`, fit `High`, improvement class `none`,
with all twelve rows passing. Its fresh locked metadata, `/tmp` fixture parity, removed-surface and broad debug
controls are green. Phase 44 is complete at 612 expanded rows (`c74c0615...`); Master final audit remains separate.

## Out-Of-Scope Handling

Do not change runtime/data/wire behavior, regenerate checked-in or Unity fixtures, invent CI, clean inherited lints,
remove transitive packages by force or mark the Master complete. Record independent findings for the Master final
audit or a later explicitly authorized Goal.
