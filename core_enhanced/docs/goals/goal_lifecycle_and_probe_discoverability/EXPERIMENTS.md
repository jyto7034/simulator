# Goal Lifecycle And Probe Discoverability Experiments

Goal: G25

Updated: 2026-07-14

## Entry Evidence

- DCA-M-23 remains open because the workflow describes goals and completion review separately but
  does not define one child→review→correction-rerun→phase/final closure order.
- DCA-M-25 remains open because `engineering_workflow.md` uses `<available-probe-path>` although the
  executable probes live under the external Unity project.
- `docs/README.md` already defines source-of-truth order and treats goal documents as non-durable,
  while `engineering_workflow.md` still says completed goal documents should be removed. The current
  master instead retains them as historical evidence, so the retention wording needs reconciliation.
- The master has completed R00..R06 and all active implementation children except user-deferred G05
  and G22. Several finding table statuses still contain pre-completion wording.

## Dirty Baseline

- Core/server repository HEAD: `c2c5dc5ec54bd26530e46f0d7ee7909de3ed7a1f`.
- Core status fingerprint: `f543cdbc57f1d2e3a3e92b1e23561ac621975494283de1532b5016e57c557ab3`.
- game_server status fingerprint: `ae099d8e78a4632a5d01ec74656013ae3f6b8a4ad9a30a9536d58bdc877e8a51`.
- Unity HEAD: `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`.
- Unity status fingerprint: `c54b95068819a8a27a66d98980169fadc811dd3403d4431fcbcdeef29134eb40`.

## Planned Experiments

| Date | Experiment | Expected Evidence | Status |
|---|---|---|---|
| 2026-07-14 | Compare README, workflow, review guide and master lifecycle | One non-conflicting ordered lifecycle | passed: workflow now defines child completion, independent phase review, correction child plus same-review rerun, next-phase gate and final review; README classifies goals as evidence/history |
| 2026-07-14 | Inspect every external probe and adjacent contract | Exact path/purpose/prerequisite registry | passed: three scripts and three adjacent contracts exist; all scripts parse and declared environment/defaults match the registry |
| 2026-07-14 | Reconcile finding/child/phase tables | One disposition per active-scope item; deferrals preserved | passed: all prior active-scope children and R00..R06 are complete; only G25 remains active before R07; G05/G22 remain `deferred_by_user` with triggers |
| 2026-07-14 | Canonical ownership audit | No implemented policy owned only by completed goals | passed: PD-01..PD-18 route to the durable ownership matrix below; PD-05/PD-14 remain explicitly deferred rather than being represented as implemented |
| 2026-07-14 | Link, stale phrase, diff and cargo safety validation | Clean durable documentation boundary | passed: all eight registered script/contract/canonical paths exist, three probes parse, stale placeholders/current-status phrases are absent, scoped `git diff --check` passed, and Core/server `cargo check --all-targets` passed |

## Durable Canonical Ownership Audit

| Policy group | Durable owner | Audit result |
|---|---|---|
| PD-01 revision and PD-02 capability/preflight authority | `docs/core_runtime_contract.md`; external `core_unity_battle_transport_contract.md` and `unity_core_contract.md` | Current runtime/wire meaning is durable; goal records are evidence only |
| PD-03/PD-04 illustrated employee-check Event | `docs/game_rulebook.md`, `docs/core_runtime_contract.md`; external `unity_core_contract.md` | One-shot pure d20, typed command/state and Event-versus-VN boundary are durable |
| PD-05 abnormality range content | Master deferred registry plus `docs/data_loading_contract.md` and `docs/skill_target_contract.md` for the temporary strict/current boundary | Correctly deferred; no invented authored values |
| PD-06/PD-07 targeting and PD-08 player-only preview | `docs/skill_target_contract.md`, `docs/data_loading_contract.md`, `docs/core_runtime_contract.md`; external battle/Unity contracts | Selection, footprint, explicit skill authority and projection ownership are durable |
| PD-09 live RON strictness | `docs/data_loading_contract.md` | Exact version, unknown/removed-field rejection and validation/publication gates are durable |
| PD-10/PD-11 boss/failure/retreat | `docs/game_rulebook.md`, `docs/core_runtime_contract.md`; external `unity_core_contract.md` | Dedicated Gate, RunFailed recovery, result commit and exact retreat restore are durable |
| PD-12 compendium/debug separation | `docs/game_rulebook.md`, `docs/core_runtime_contract.md`; external `unity_core_contract.md` | Global monotonic commit/projection and non-authoritative diagnostics are durable |
| PD-13 research delivery | `docs/game_rulebook.md`, `docs/core_runtime_contract.md`; external `unity_core_contract.md` | Atomic authoritative planning-return delivery is durable |
| PD-14 Equipment/Maintenance content | Master deferred registry; current implemented surface in `docs/game_rulebook.md` and external `unity_core_contract.md` | Correctly deferred; no generic recipes/block balance invented |
| PD-15 inventory admission | `docs/game_rulebook.md`, `docs/core_runtime_contract.md`; external `unity_core_contract.md` | Unbounded logical grant plus 50/10 display high-water and item identity are durable |
| PD-16 incapacitation | `docs/game_rulebook.md`, `docs/core_runtime_contract.md` | Per-death attempt transaction, retreat discard and employee aggregation are durable |
| PD-17 Artifact removal | Current code/data/DTO absence, strict rejection tests and deletion evidence; durable inventory/Shop contracts intentionally contain no replacement policy | Complete deletion is durable by absence; the master records the accepted removal decision without creating a new subsystem |
| PD-18 Shop economy | `docs/game_rulebook.md`, `docs/core_runtime_contract.md`, `docs/data_loading_contract.md`; external `unity_core_contract.md` | Deterministic sessions, checked quotes, optional refresh/sell and removed prototype vocabulary are durable |

The audit found no implemented active-scope policy whose sole owner is a completed goal directory.
The older `completed_goal_document_retirement` proposal was explicitly marked
`historical_unexecuted` to prevent its name from being confused with this master child.
