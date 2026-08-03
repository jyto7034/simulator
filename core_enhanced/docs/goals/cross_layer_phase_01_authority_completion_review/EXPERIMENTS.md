# Cross-Layer Phase 01 Authority Completion Review Experiments

## 2026-07-13 — R01 initialized

Hypothesis:

The four Complete / High child reviews provide sufficient inputs for an independent combined Phase 1
review, but their labels are not accepted as runtime evidence until R01 retraces the seams.

Action:

- Created `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md` before R01 code search or test
  execution.
- Bounded R01 to review/evidence work. Phase 2 implementation remains blocked.

Changed files:

- R01 goal documents only.

Commands and actual test counts:

- Documentation initialization only; 0 tests executed and no success claimed.

Observed result:

- R01 is active and has explicit review items, evidence order, validation expectations, and failure
  conditions.

Decision and next step:

Read all three R01 documents completely, then collect actual runtime/data/server/Unity evidence before
running combined validation.

## 2026-07-13 — Combined authority source trace

Hypothesis:

The current implementation composes G01/G06/G07/G12 without a runtime authority split, while the
Master finding registry may still contain historical status text.

Action:

- Traced Core command authority validation, common capability preflight, battle projection commits,
  deployment/skill payload eligibility, VN attachment, reconnect/read-only resync, and strict bundle
  startup.
- Traced game_server actor cursor, command response/side-message order, stale error plus resync,
  reconnect attach, and the single startup `Arc<GameDataBase>` handoff.
- Traced Unity strict capability/revision/eligibility parsing, command authority attachment,
  ordinary/resync application, defensive stale-error resync, presenter/command gates, and actual Rust
  fixture provenance.
- Searched canonical documents and runtime sources for removed authority/fallback terminology.

Commands and actual test counts:

- Read-only source/document searches only; 0 tests executed and no validation success claimed.

Observed result:

- Core validates exact battle authority before action/domain preflight; read-only query/recovery is
  scope-separated.
- Every reviewed battle mutation checks revision capacity, revalidates payload, commits exactly one
  revision, then projects checkpoint. VN/capabilities are attached read-only to that revision.
- Server actor owns the connection event cursor and emits command response before ordered side
  messages through one socket path. Stale mutation returns typed correlated error and current resync
  without mutation.
- Unity selects battle capability from the revisioned battle update, validates exact payload
  eligibility shapes, attaches current battle UUID/revision to mutations, and requires Core preview
  before live deployment confirmation. Local tile filtering is presentation-only over Core-authored
  cells and cannot authorize commit.
- Production startup has one strict bundle path; fixture builders do not enter server startup.
- Canonical runtime/data/server/Unity documents contain no removed `allowed_actions`, cursor,
  revisionless, destructive reconnect, or separate map-loader contract.
- Historical goal/audit documents intentionally retain old terms as evidence.
- Master registry drift found: DCA-H-08, DCA-M-13, DCA-M-18, and DCA-M-19's shared portion still say
  not started despite their Complete/High child reviews.

Decision and next step:

Correct only the Master finding statuses, record the correction, then run focused fixture/probe and
broad validation. Keep concrete inventory preview/eligibility under G24.

## 2026-07-13 — First focused validation batch

Hypothesis:

The actual Rust fixtures and focused revision/capability/stale/live-data filters will provide nonzero
combined contract evidence.

Commands and actual result:

- Both `generate_battle_revision_fixture` and `generate_action_capability_fixture` exact-value
  comparison commands passed. They execute 0 tests and count only as typed provenance evidence.
- `cargo test -q -p game_core battle_state_revision --lib -- --test-threads=1`: 0 executed,
  669 filtered out. This is not success; the substring did not match current test names.
- `cargo test -q -p game_core action_capabilit --lib -- --test-threads=1`: 1/1 passed,
  668 filtered out.
- `cargo test -q -p game_core stale_battle --lib -- --test-threads=1`: 1/1 passed,
  668 filtered out.
- Live-RON focused: 41/41 passed, 628 library tests filtered out. Other integration binaries matched
  zero under this module filter and are not counted.

Failure cause:

The revision concept is present in assertions/fields but not in a test function name containing the
literal `battle_state_revision`.

Correction and next step:

List current library tests, select actual revision/commit/reconnect test-name filters, and rerun them
with nonzero counts before broad validation.

Second focused attempt also executed 0 tests for each of five commands. The selected function names
were correct, but combining their short names with test-harness `--exact` required the fully qualified
module path. Remove `--exact` and rerun the verified unique substrings. No success is claimed for
this attempt.

Final focused result:

- Core playback revision commit: 1/1 passed.
- Core reconnect/setup-loss revision preservation: 1/1 passed.
- Core stale authority rejects before mutation: 1/1 passed.
- Core common map/checkpoint typed capability parity: 1/1 passed.
- Core state-transition capability revalidation: 1/1 passed.
- game_server active-battle attach/recovery batch: 1/1 passed.
- game_server stale error then current resync without mutation: 1/1 passed.
- game_server future resync cursor rejection: 1/1 passed.
- game_server battle side-message mapping: 1/1 passed.
- game_server nested battle-resync serialization: 1/1 passed.

All corrected filters executed nonzero tests. Proceed to broad Core/server and Unity MCP validation.

## 2026-07-13 — Broad validation and hygiene

Results so far:

- Core all-targets: 710/710 passed, 0 failed, 0 ignored.
- game_server all-targets: 27/27 passed, 0 failed, 0 ignored.
- Unity MCP EditMode job `739a2ee1101a42bbafea30be3fe3a7d1`: 188/188 passed, 0 failed,
  0 skipped.
- Unity MCP PlayMode job `b9767db63f4448498614258cd7fedf73`: status succeeded, 53/53
  completed with no failures. The MCP result summary was null; the generated TestResults.xml proves
  51 passed, 0 failed, 2 explicit live-server skips.
- rustfmt check, Core/server all-target checks, and Python smoke-probe syntax compilation passed.

Hygiene command failure:

The combined `git diff --check` invocation failed after the successful checks because it passed the
Unity project path to the simulator repository's Git command. Git rejected that path as outside its
repository. This is a command-scoping mistake, not a diff finding. Rerun simulator and Unity scoped
diff checks with their respective repository roots.

Correction:

- Simulator-scoped and Unity-repository-scoped `git diff --check` both passed.
- Production searches found no old action/cursor/static-list authority, Serde alias, ignored Rust or
  Unity test, or stale canonical contract wording.
- The only game_server `load_live_embedded()` hits are main startup and a test-module helper.
- Final Unity MCP state is idle, not compiling/playing, and ready; the post-PlayMode error-console
  query returned 0 entries.

Review outcome:

- R01-H-01 and R01-H-02 are material implementation gaps despite all broad suites being green.
- `COMPLETION_REVIEW.md` records Partially complete / Medium / immediate correction recommended.
- R01 is externally blocked pending authorization for a dedicated correction goal; Phase 2 remains
  blocked.

## 2026-07-14 — Correction rerun passes Phase 1

Hypothesis:

G30 closes both initial R01 findings without invalidating G01/G06/G12 or introducing a new authority
or compatibility path.

Action:

- Re-read the active R01 documents and the G30 completion review.
- Re-inspected current Core gate/evaluator, server recovery/emission, Unity error/resync consumption,
  canonical documents, scoped diffs, legacy searches, and ignored tests.
- Reused the G30 broad results because no runtime code changed afterward; independently reran common
  capability exact fixture/focused tests and strict live-RON focused validation.
- Appended correction evidence to the original failed review rather than replacing it.

Commands and actual test counts:

- Common capability exact Rust fixture: passed, 0 tests by design.
- Common capability focused: 2/2 passed.
- strict live-RON focused: 42/42 passed.
- G30 current evidence: Core 711/711, server 28/28, Unity EditMode 189/189, Unity PlayMode 51 passed
  with 2 explicit skips, exact battle fixture and static checks passed.

Observed result:

- R01-H-01 and R01-H-02 are Complete / High.
- The combined revision/capability/recovery/strict-data boundary has no competing authority or
  immediate correction.
- Phase 1 exit gate passes.

Decision and next step:

Mark R01 and Phase 1 complete. Create the isolated Phase 2 documents, then start G26 under the
accepted employee-attempt transaction policy.

### YYYY-MM-DD — Short experiment title

Hypothesis:

Action:

Changed files:

Commands and actual test counts:

Observed result:

Failure cause:

Revert or correction:

Decision and next step:
