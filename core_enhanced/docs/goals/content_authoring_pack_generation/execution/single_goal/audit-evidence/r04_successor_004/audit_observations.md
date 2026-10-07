# Audit observations and execution boundary

Reviewer `/root/r02_r04_fresh_audit` is a separate same-model agent, without conversation fork or implementation participation. Only new R02 audit004/report/evidence outputs were written. Source, tests, owner records, target005 and earlier artifacts, fixtures and shared binaries stayed frozen. No gameplay, Cargo test/build, owner probe, personal-store or network operation was executed.

New read-only verification commands (working directory `/mnt/f/work/simulator`):

```
python3 core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal/audit-evidence/r04_successor_004/verify_currentness.py currentness_entry
python3 core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal/audit-evidence/r04_successor_004/verify_evidence.py
python3 core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal/audit-evidence/r04_successor_004/verify_consumers.py
python3 core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal/audit-evidence/r04_successor_004/verify_currentness.py currentness_closure
```

Their exact structured outputs are currentness_entry.json, evidence_review.json, consumer_review.json and currentness_closure.json. The scripts retain the checks and consumed identities, commands and raw test/consumer locators. Final output sealing verifies report and evidence hashes separately from the target. Read-only shell source searches and excerpts were used for human-equivalent source review, not counted as gameplay runs.

The first consumer verifier attempted `c['stdout']` for a receipt row that records only `name` and `stdout_sha256`, producing `KeyError: 'stdout'` before an output verdict. The actual diagnostic capture names streams `<name>.stdout`/`<name>.stderr`. The audit script now resolves that explicit convention and verifies all twenty diagnostic stream hashes. No owner receipt or test oracle changed. Initial source-location guesses for directory modules (`application/mod.rs`, `world/mod.rs`, `state/checkpoint.rs`, `authoring/runtime.rs`) were absent; actual files are application.rs, world.rs, world/active_run.rs and battle/core/runtime_facts.rs. Those failed searches were not evidence of absent implementation.

INPUT007 is a bounded 230-entry production checkpoint, not an exhaustive test manifest. Independent Rust/RON enumeration identifies 18 additional integration/harness inputs in the formal target. Seventeen match the original 353-member dirty baseline archive byte-for-byte. The remaining `live_item_skill_activation.rs` transformation is exactly the original ordered equipment UUID vector mapped to `BattleEquipmentBinding { base_uuid, persistent_copy_uuid: None }`; all other fixture/assertion bytes are unchanged and full164 executes its four tests. This adaptation is included in R04's reconstructed patch. No missing fixture is treated as verified merely because INPUT007 exists.

Actual full164 groups are 1309 library plus 4/4/25/15/4 integration, totaling 1361. The R04 evidence-map skill-runtime row saying 25 is clerical; the actual runner has 15. This audit uses the raw count. No test was skipped or replaced to obtain a positive result.

Old R02 target004 changes in 58 members. Fifty-four are in R04's scoped attribution. The other four are R03's earlier trial.rs correction and three owner execution records; current trial bytes match the R04 intake archive. R04's working-policy change is prior P2G work, as its attribution specifies. These are not fresh R02 implementation or unrelated work credited to R04.

R04 target002 is an evidence-capture-only successor preserving target001 and implementation bytes. The current R02 target005 binds the newly durable full reports, captured binaries and original fixtures. Reused ordinary guard arithmetic uses target-bound allocator168 raw JSON and guard170, not the summary boolean or a new performance threshold. No claim is made that requested allocator bytes are RSS or that all native-budget work is certified by this R02 review.
