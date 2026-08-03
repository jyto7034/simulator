# Runtime Identity Determinism Experiments

Goal: G17

Updated: 2026-07-14

## 2026-07-14 — Goal initialized

- DCA-M-02 is a latent checkpoint/VN UUID collision claim; the audit did not reproduce failure.
- DCA-M-22 is a latent observation-sequence/delivery-order claim; the audit did not reproduce an
  outcome change.
- No implementation is authorized until current code and focused reproductions establish an actual
  defect.

## 2026-07-14 — Both latent identity defects reproduced

Checkpoint/VN reproduction:

- Added `checkpoint_load_does_not_reuse_preserved_visual_novel_session_identity` with a
  nonblocking OncePerNode/RewindWithRunCheckpoint trigger.
- Flow: save checkpoint -> fire trigger -> preserve active top-level VN -> load checkpoint -> fire
  rewound trigger again.
- The test executed 1 case and failed because active and queued session ids were exactly equal:
  `cb475136-0750-48ea-8640-f086f480ff91`.
- Cause: checkpoint payload rewinds the whole `UuidManager` while intentionally preserving
  top-level `VisualNovelRuntimeState`.

Observation/delivery reproduction:

- Added `observation_log_insertion_does_not_change_basic_projectile_identity` using identical
  gameplay launch inputs and projectile counters, with one extra observation-only log before the
  parent record in the second core.
- The test executed 1 case and failed: control delivery id
  `644a6f59-1f9b-48be-9e50-77153ce8362d` versus observed
  `1b835d61-ade9-4265-a223-cb18023c4609`.
- Cause: basic projectile identity seeds from `recording_cause.parent_seq`. Skill projectile uses
  the same pattern, while skill cast/area identity still uses the AbilityCast event-log sequence.

Decision:

Both DCA-M-02 and DCA-M-22 are actual deterministic identity defects. G17 must implement a complete
boundary: preserve the VN session UUID namespace across checkpoint load, and allocate delivery/cast
identities from gameplay counters/source identities rather than observation-log sequence.

## 2026-07-14 — Dirty baseline and overlap classification

- Core: branch `main`, HEAD `c2c5dc5ec54bd26530e46f0d7ee7909de3ed7a1f`; resumed red-test
  scoped fingerprint `bfdb308be2ff65291058603ad1d2b460041c39bdc3c990c432490c8b31a15564`.
- game_server: branch `main`, same HEAD; resumed full diff fingerprint
  `4b8017a34a803b9143d4c3edd3dfa4adea7f7e1a1a22b19639a10d79fd418921`.
- Unity ArkClient: branch `master`, HEAD `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`; inherited
  Phase 3 fingerprint `733c198630027dce8e006da1c3806cd182837583cdd9800a5726c33c158a6f5a`.
- All repositories were already heavily dirty. Core target files overlap completed predecessor
  work; the two failing reproduction tests are G17-owned. G17 has no game_server or Unity runtime
  target and preserves all unrelated hunks.

## 2026-07-14 — Deterministic identity boundaries implemented

Checkpoint/VN:

- `UuidManager::restore_checkpoint_preserving_counters` replaces checkpoint-owned streams but
  preserves the maximum counter for explicitly global namespaces.
- Checkpoint load preserves only `NS_VISUAL_NOVEL_SESSION`; equipment, consumable, employee, and
  battle-session streams retain checkpoint rewind behavior.
- The VN regression now also completes the preserved active session, verifies exact queued-session
  promotion, rejects the stale old session id, and verifies no queue/active mutation on rejection.

Battle:

- Added battle-local `skill_cast_seq`, initialized and reset independently from `event_log_seq`.
- AbilityCast event seq remains the event-log parent/cancellation reference. `ActiveSkillCast`
  retains it as `ability_event_seq`, while skill steps, runtime maps, projectile/area identity, and
  same-time gameplay tie-breaks use the gameplay cast seq.
- Basic projectile identity seeds from the battle seed plus the already stable damage-source
  occurrence UUID. Skill projectile identity seeds from battle seed plus gameplay cast seq. Area
  identities use battle seed, gameplay cast seq, caster, time, and area counter.
- No wire DTO, RON schema, save migration, Unity reader, or server transport shape changed.

## 2026-07-14 — Validation

- Initial shared-target rerun after formatting failed before tests because Cargo could not update
  `/mnt/f/work/simulator/target/debug/.cargo-lock` in the restricted filesystem. Re-running with
  `CARGO_TARGET_DIR=/tmp/core_enhanced_g17_target` fixed the environment-only failure.
- Checkpoint/VN exact test: 1 passed; it covers unique identity, queue promotion, stale targeting,
  and unchanged state on rejection.
- Observation identity exact tests: 3 passed; basic projectile, skill cast/projectile/queue order,
  and area identity/damage outcome.
- UUID checkpoint stream test: 1 passed; local stream rewinds and global VN stream remains monotonic.
- Core library: 694 passed, 0 failed, 0 ignored.
- Skill runtime integration: 13 passed, 0 failed, 0 ignored.
- Core all-targets: 735 passed total (694 + 3 + 3 + 18 + 13 + 4), 0 failed, 0 ignored;
  zero-test binary targets are not counted as evidence.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- Legacy search found no production delivery UUID seed derived from `recording_cause.parent_seq`, no
  production AbilityCast event seq reused as gameplay cast identity, and no relevant serde alias or
  ignored test. The one `cast_seq = record_event_log` match is a manual test fixture.
- game_server/Unity suites were not rerun because G17 changes no external DTO, serialization, RON,
  transport, or client surface.
