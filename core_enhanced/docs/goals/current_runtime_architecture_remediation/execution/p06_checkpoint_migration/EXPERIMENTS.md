# P06 Checkpoint Migration Experiments

## 2026-08-23 — GEX-00 Intake

- Run mode: full Goal Set `goal-run`.
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P06-CHECKPOINT-MIGRATION)`.
- Execution node: standalone `WP-CRAR-C2-07`; no parallel Phase or external migration work is authorized.
- P01, P02, P03 and P05 completion/currentness evidence was read before selection. All prerequisites are
  `Complete / High`.
- Existing exact-identity P06 execution document: none; canonical directory `execution/p06_checkpoint_migration`
  selected.
- Production/source mutation during intake: none.

### Actual Boundary Trace

- Reviewed the Core active-run writer/reader, application draft route, checkpoint payload and the actual
  validated-data SavePoint fixture/callers.
- Confirmed current Core emits and accepts only strict version 4, rejects unknown document fields and validates the
  restored graph before draft assignment, but has no checksum, named legacy importer or removal certification gate.
- Confirmed the existing test fixture covers exact checkpoint semantic fields, excluded transient/history state,
  continuation recapture, graph-invalid inputs and foreign seed authority.
- Confirmed the current canonical test document comes from the production writer; migration evidence therefore needs
  an independently constructed V4 input instead of renaming the same writer output.
- Confirmed checksum input cannot use ordinary typed `HashMap` reserialization bytes because deserialization may change
  iteration order. Canonical recursive object-key ordering is required for stable verification.

### Baseline Execution

Command from the simulator workspace root:

```text
env CARGO_TARGET_DIR=/tmp/crar-p06-gex00-target \
  cargo test -p game_core --lib checkpoint_validation -- --nocapture
```

Result: exit `0`; eight matching library tests passed, zero failed/ignored, 1,084 filtered out. This includes the seven
current checkpoint-validation tests plus the encounter-progression checkpoint graph test. The baseline verifies the
current V4 semantic oracle before the writer cutover; it is not a P06 completion result.

### Feasibility and N/A Recheck

- Exact version dispatch plus separate document types can retain the V4 reader while making V5 the sole serializer.
- The existing staged application route is sufficient for pre-publication rejection; no second transaction owner or
  external durable envelope is required.
- `sha2 0.10.9` is already resolved in the workspace lock, but P06 will add a direct Core dependency so the algorithm is
  explicit and stable. Cargo manifest/lock currentness will be audited as an expected successor delta.
- Released/deployed save population remains evidence-backed N/A from the frozen handoff. This pass did not inspect
  excluded external stores, server/client source or sibling resources to manufacture compatibility evidence.
- P05 history separation remains applicable and testable. External reconnect/archive/receipt/store remains out of
  scope rather than a hidden completion dependency.
- No policy/Goal/Phase change, unsafe dirty overlap or stop condition was found.

### GEX-00 Result

- Result: `passed`.
- Approved next pass: GEX-10 V5-only writer, canonical checksum, explicit V4/V5 importers, independent V4 corpus,
  continuation/corruption/removal-gate tests, runtime-contract update and predecessor successor-currentness records.

## 2026-08-23 — GEX-10 Implementation and Correction Loop

### Codec and Corpus Experiments

- Added a deserialize-only strict V4 document and a strict serialize/deserialize V5 document. Production export has
  one serializer and always writes version 5 with `checkpoint_sha256`; exact dispatch rejects every other version.
- Canonical checksum initially used the intended recursive key sort. The first focused run intentionally failed only
  on the placeholder canonical-corpus digest and reported
  `64f78ce035c38e523c95b14456296750aa2a04a1b43f3f0f00b5c101d608bcc2`; that exact identity is now frozen.
- A reordered V5 envelope with separately deserialized typed maps verifies and continues identically, proving the
  checksum is independent of incidental `HashMap`/document member order.
- The canonical V4 corpus builder reads the completed SavePoint payload directly and creates exact legacy input
  without calling the V5 writer. Exact root/19 fields and field-by-field source parity remain executable.

### Strict Rejection, Transaction and Boundary Experiments

- Malformed JSON; missing version/checkpoint/checksum; unknown version; extra envelope/checkpoint field; stale checksum;
  non-canonical uppercase checksum; and checksum-valid graph corruption all return `InvalidStaticData`. First/retry
  errors are equal and the complete player/lifecycle/run/UUID/inventory/profile/checkpoint/power/seed snapshot remains
  equal.
- V4 one-edge graph corpus and V5 checksum-valid graph mutation both reject before continue publication. Competing
  continue against an active run and `loads_used = max + 1` return `InvalidAction` with the same authority snapshot.
- `loads_used == 3`, empty checkpoint export and an inventory containing exactly all 50 persisted reservation slots
  round-trip. A 51st slot with an unchanged reservation and a valid checksum rejects with the exact inventory graph
  error before mutation.
- V4 and V5 from the same completed SavePoint continue to the same map/state/next result, retain monotonic host metadata
  and recapture as the same V5 document. Diagnostic battle history/result state is absent from both exact schemas.
- The explicit reader-removal negative gate returns the frozen certification-required error while the V4 importer
  remains live.

### Correction and Predecessor Currentness

- The first P01 verifier run found three missing stable locators because the exact canonical constant
  `CURRENT_V4_CHECKPOINT_KEYS` had been generalized. Correction restored that V4-corpus alias without changing the P01
  manifest. The rerun passes with 42 contents, composite
  `3ec400cf300247eaab0555e842663ebc5c7908a7edbe8f189df007560c1690d0`, errors 0.
- Direct `sha2` use mechanically updated the workspace and standalone predecessor probe locks. Every P02-P05 valid
  release probe still exits 0 and every invalid probe still exits the expected 101 with its original API/privacy error
  family. These exact dependency-only deltas are recorded rather than hidden as unchanged.
- Successor-currentness records: P01 `_006`, P02 `_005`, P03 `_004`, P04 `_003`, P05 `_002`; all
  `Complete / High`.

### Broad Result

- Focused checkpoint selector: 15/15, zero failure/ignored.
- Three actual caller selectors—facility active-run round trip, typed monolith checkpoint continuation and combat cache
  recomputation—pass.
- `cargo check -p game_core --release --lib`: exit 0, warnings 0.
- Full Core: 1,099 library + 51 integrations = 1,150/1,150, failed/ignored 0; `ron_loading` 25/25.
- No external host/store/server/client source, sibling resources or released/deployed save population was inspected or
  used. P06 makes no compatibility or performance PASS claim.
- GEX-10 exit criteria are met; next operation is immutable target publication followed by separate fresh GEX-20.
