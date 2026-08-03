# Battle Setup Catalog Ref Correction Experiments

Status: complete

## 2026-07-18 — Child Initialization

- Read Master G02 scope and current external ownership recorded by the completed reaudit.
- Child path was available.
- Initial combined `git status` command incorrectly included an external path and failed after the
  path check; no state changed. Repository/external dirty checks are separated after initialization.
- Mandatory child documents created before producer search or mutation.

## 2026-07-18 — Producer Correction And Regression

### Baseline and inspection

- Confirmed the external transport contract defines `catalog_refs.abnormality_ids` as actual
  abnormality catalog IDs only and excludes corroded/profile-based identity.
- Confirmed `battle_setup_snapshot_dto` filtered spawn-wave entries by kind/non-empty but copied
  every enemy-briefing ID without a kind or empty check.
- The first two test commands incorrectly combined a short test-name filter with `--exact` and ran
  zero tests. Re-ran with the correct filter.
- A temporary cleanliness assertion against `suppress_clouded_monk` failed on the old producer; the
  case was then correctly classified as mixed rather than corroded-only and given an exact actual-ID
  expectation. The reaudit live probe remained the direct corroded-only baseline evidence.

### Implementation

- Added one producer-local predicate requiring non-empty, loaded-catalog-resolved abnormality IDs.
- Applied it to primary identity, enemy briefing and spawn waves.
- Required `EnemyKind::Abnormality` for briefing and wave candidates.
- Preserved sorting/deduplication and all public DTO/message shapes.

### Tests

- `cargo test -p game_core battle_setup_snapshot_corroded_only_catalog_refs_exclude_empty_abnormality_id --lib -- --nocapture`: 1 passed.
- `cargo test -p game_core battle_setup_snapshot_exposes_survival_timer_without_encirclement_variant --lib -- --nocapture`: 1 passed.
- `cargo test -p game_core battle_setup_snapshot_live_defense_state_request_returns_battle_update_without_advancing_cursor --lib -- --nocapture`: 1 passed.
- `cargo test -p game_core game::world::tests::combat --lib --quiet -- --test-threads=1`: 60 passed.
- `cargo test -p game_server --quiet -- --test-threads=1`: 30 passed.
- Started the real server on `127.0.0.1:18083` and ran the registered external
  `unity_ws_smoke_probe.py`: exit 0. The selected corroded-only encounter reported wave
  `abnormality_ids: []` and setup `catalog_refs.abnormality_ids: []`.
- The first probe attempt used a nonexistent repository-local path; the registry was re-read and the
  authoritative external probe path was used. Sandbox socket permission was then explicitly approved.

### Disposition

- `DCA2-WIRE-003` complete.
- No P1 presentation or external Unity artifact was modified.
