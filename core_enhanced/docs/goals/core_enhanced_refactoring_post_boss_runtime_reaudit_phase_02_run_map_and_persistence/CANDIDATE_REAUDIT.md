# Phase 02 Candidate Reaudit

## Current Dispositions

| Candidate | Current evidence | Boss/current impact | Disposition | Work type/readiness |
| --- | --- | --- | --- | --- |
| CRA-P02-C01 | `ActiveRunStore::publish` still encodes/installs incoming current before any max-merge; only previous backup receives maxima | Boss did not alter host monotonic lifetime | `still_valid` | correctness; test first |
| CRA-P02-C03 | checkpoint capture and restore still manually copy the run/root field set in opposite directions | private Boss session is correctly excluded, so mapping target does not expand | `still_valid` | behavior refactor; golden parity first |
| CRA-P02-C04 | fields remain public and BossOmen/admin directly assign progression/node state | later tactical/Boss work does not centralize map mutation | `still_valid` | behavior refactor; transition characterization |
| CRA-F-C01 P02 contribution | validator checks referenced nodes but not run/UUID seed equality or graph-derived progression consistency | private Boss runtime adds no persisted invariant | `still_valid contribution` | correctness; P03/Final required |

## C01 Detail

Current publish validates and writes the incoming envelope to temp, raises monotonic values only on the previous
current generation before rotating it to backup, then installs the unchanged incoming envelope as current. A lower
valid incoming `loads_used` or peak therefore still becomes the preferred next load. Existing nine store tests cover
backup recovery, checksum/version, invalidation and failure, but no higher-current→lower-incoming current load.

- Original axes remain `2/3/2/1/3/3/unknown`, subtotal 14/18.
- Preserve incoming Core payload/anchor, current/backup generation, checksum/fsync/tombstone and error shapes.
- Target remains max-merging readable current host metadata into the incoming current envelope before install.
- No lease/conflict policy is implied.

## C03 Detail

`save_run_checkpoint` constructs every `RunCheckpointPayload` field manually; `restore_run_checkpoint_payload`
constructs `RunState` and root fields manually in reverse. `RunState::for_next_floor_from` retains its deliberately
different carry-forward lifecycle. Current `RunCheckpointPayload` has no active battle/Boss graph field and restore
clears `active_battle`, which matches the current private session lifetime rather than closing mapping duplication.

- Original axes/readiness remain `2/1/2/3/2/3/unknown`, 13/18, golden V3 field parity first.
- Exact V3 shape and exclusions, VN/UUID behavior, Floor carry-forward and `combat_recovery_restriction: None` remain
  the preservation boundary.

## C04 Detail

`MapProgression` has typed normal transition methods, but `boss_omen.rs` still writes
`available_node_ids` directly and admin fixtures assign/clear `current_node_id`/`available_node_ids` alongside map
node state. The fields remain public/serialized. No invariant validator composes stored IDs with graph-derived
selectability before restore.

- Original axes/readiness remain `3/2/2/2/2/2/unknown`, 13/18, transition characterization first.
- Preserve V3 shape, sorting, directed/concealed behavior, BossOmen exit and admin results.
- Removing a serialized field remains migration/consumer gated; private mutation encapsulation is policy-free.

## F-C01 P02 Contribution

`validate_run_checkpoint_payload` currently checks power/difficulty authority, facility template, that stored
progression IDs reference existing nodes, recruitment ownership and Monolith state. It does not:

- compare `run_progression.run_seed` with serialized `uuid_manager` seed;
- compare explicit in-run load with current `GameCore::run_seed`;
- validate current/completed/available IDs and node states against graph-derived progression/selectability.

Continue adopts the progression seed and resets the live UUID manager before restore, but the payload's serialized
UUID state is later restored without the missing release-mode precondition. P02 therefore confirms the original
seed/progression half remains. P03 must audit asset/economy edges before Final disposes F-C01.

## Gates And New Candidates

- Cross-actor lease/conflict, repair/rebind/V2 migration, admin persistence and serialized field removal remain
  policy/external gates, not hidden inside these candidates.
- Boss private/no-save state agrees with current runtime canonical and is not a persistence defect or new candidate.
- No distinct `CRR-P02-*` root cause was found.

