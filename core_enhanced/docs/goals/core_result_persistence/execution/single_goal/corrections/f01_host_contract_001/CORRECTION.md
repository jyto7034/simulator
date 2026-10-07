# U04 F01 canonical host contract correction

Exact owner: GS-CRRP-001 / GOAL-CRRP-01 / PHASE-CRRP-01, correction GEX-10.
The [closed independent audit](../../../../post_audit/u04_20261004_001/COMPLETION_AUDIT.md)
returned Not Complete / High with F01 only at 2026-10-04T09:29:04Z. Its original report,
condition/predecessor judgments, raw checks and target/u04_001 remain unchanged.

## Scope and authority

The existing continuous request includes finding correction and fresh independent audit without
another approval. P2G_50_GOAL_SET.ko.md:82 already assigns this exact canonical host document.
F01 is a missing implementation deliverable, not a new policy choice: the frozen15 acceptances,
Goal, Phase Plan and Handoff retain their exact meaning and references. No P2G policy revision or
new implementation scope is needed. The owner is this Goal's HOST/VERIFY work; U05 is still ineligible.

Only `game_server/docs/unity_bridge_protocol.md` changes product-facing documentation. Product
source/data/config/test bytes are unchanged. Execution summaries, this correction's verification
and freeze tooling are supporting records. The document now describes the actual prepared paired
store path, V5/V10, conditional revision/tombstone, accepted-intent recovery, latest-generation
backup, immediate result/receipt/choice, retreat, Continue and independent SavePoint. The retained
profile-only dirty/retry mode is explicitly separate. Related stale request/overlay names and
research-delivery/result-ack descriptions in the same required consumer document were reconciled
against current `ApplicationCommand`, `PlayerGameClientMessage` and snapshot DTOs.

## Direct source and consumer mapping

Paths below are repository-relative. The precise source hashes are in [VALIDATION.json](VALIDATION.json).
The source and original current execution semantics were independently reviewed in audit001;
the following mapping connects the corrected documentation to those exact current owners.

| Required contract | Current owner inspected | Existing runtime evidence reused |
| --- | --- | --- |
| Request/ack/query and typed host overlay | `game_server/src/game/player_game_actor/messages.rs::deserialize_command`; `handlers.rs::handle`; `state.rs::PlayerStateSnapshotDto`; `core_enhanced/src/game/behavior.rs` | V12; host DTO/route tests, public API/CLI and stock probes |
| Immediate settlement and receipt-only acknowledgement | `core_enhanced/src/game/world/combat.rs::transition_finished_live_battle_to_result`, `handle_complete_combat_result` | V01/V07; manual and real terminal host tick, restart before ack |
| Fixed ChooseOne right and once-only claim | `core_enhanced/src/game/world/reward.rs`; `active_run/continuation.rs` | V02/V10; fixed2/0, selected/claimed restart and stale actor refusal |
| Two retreats and unfinished pre-entry Continue | `core_enhanced/src/game/world/combat.rs`; `active_run/continuation.rs` | V03/V04; two retreats/third challenge, current unfinished node cases |
| Latest confirmed Continue versus SavePoint rewind | `core_enhanced/src/game/world/active_run/continuation.rs` and checkpoint restore | V05/V06; first SavePoint-independent Continue, load3, repeated battle identity |
| Exclusive preparation and accepted paired persistence | `game_server/src/game/player_game_actor/handlers.rs::publish_prepared_application`, automatic tick; `persistence/player_profile_store/application_intent.rs` | V09/V12/V15; preaccept rejection, accepted failure/retry, every process-crash stage and observer commit |
| CAS generation, tombstone and latest backup | `game_server/src/persistence/active_run_store.rs`; `player_profile_store/application_intent.rs` | V09/V10; stale choice, terminal/missing ABA and current/backup load monotonicity |
| Strict codec/unsupported-original protection | `core_enhanced/src/game/world/active_run/continuation.rs`; both host stores | V11; malformed V10, V4/V9/unsupported profile/backup/journal protection |
| Exact recovery names/status and profile-only mode | `handlers.rs` paired/direct branches, `handle_player_profile_management`; `state.rs::PlayerProfilePersistenceSnapshotDto`; `PlayerBehavior` variants | V09/V12; current RetryPlayerProfile, pending/protected/fatal and profile-only behavior |
| Same actor reconnect versus native fault/Quit/fresh owner | `game_server/src/game/player_game_actor/handlers.rs::build_state_snapshot` and fault gates | V08/V15; actual five modes, stock fault/reconnect/file protection/Quit/new-owner Continue |

The document is an interface contract; it does not claim an implemented Unity screen or make the
host a second gameplay authority. Existing R05 concurrent-peak-only warning269,128 bytes, raw101,
OBS001, no Windows/general power-loss/remote deployment claim and20-minute exclusion remain exact.

## Validation, preservation and next pass

The first audit's additional claim that this file was absent from its target is incorrect.
[MEMBERSHIP_ERRATUM.json](MEMBERSHIP_ERRATUM.json) verifies the unchanged original manifest,
membership and actual before/after archive members: the stale file was already present in all of
them and in the original source baseline. The valid F01 is failure to update the assigned content,
not failure to archive its bytes. This factual correction preserves the closed audit and asks the
fresh reviewer to adjudicate the error explicitly; its original omission statement is not reused.

`baseline/HOST_PROTOCOL_BEFORE.tar.gz` and its member manifest were captured before editing and
match the auditor's `HOST_PROTOCOL_AT_AUDIT.md` exactly (SHA256
`ebfb9534fc901afe8a40885c23c6504e6524f0b07420f83aa0f45a60898429c4`).
This supplemental archive is a redundant exact copy of the already captured original, not an added
source member. The successor union retains the required file and all original source/document
baselines. `target/u04_003` owns actual before/after archives and fuzz0 forward/reverse restoration;
old U04 and U03 archives are preserved and each external member is verified without recursive copies.
Unaudited `target/u04_002` is also preserved; it was superseded to correct supporting records that
repeated the first audit's false omission claim. Product and corrected host-document bytes did not
change between these two correction targets.

`../../../../evidence/correction_f01_host_contract001.json` is the actual verification receipt.
[verify.py](verify.py) checks original closed audit/target artifacts, before bytes, current product
identity,40 execution receipts/logs/source/input/runner bindings, exact command/DTO vocabulary,
relative links and whitespace. All checks pass; these mechanical checks do not self-award semantic
audit closure. [VALIDATION.json](VALIDATION.json) binds the corrected document and source owners.

The product basis remains
`cb033469ed897c2bb38cdd5cd2bc4561b8b820acd0b680d45c5381abf4b3c629`.
Core1734/1734/1743, host89/97+separate heap1, static/release,16 public consumers, actual stock
continuation/rewind/fault and PC576 remain current unchanged executions. No new product execution
is claimed: documentation-only correction does not invalidate them.

PE15 and V12/V14 documentation/membership closure are submitted for fresh independent GEX-20.
All15/15/10/112+31 conditions retain their original coverage; the re-auditor may reuse validated
unchanged judgments from audit001 and must directly judge this correction and successor target.
Only a current independent Complete / High closes F01 and unlocks U05. This is a self-verification
record, not that verdict.
