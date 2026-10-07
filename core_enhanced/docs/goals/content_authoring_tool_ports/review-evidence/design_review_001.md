# R03 P2G-20 bounded design review 001

Candidate `/tmp/r03_code_review_candidate_001.ko.md`, SHA-256 `53c997b286792710741f31b2987ef961ba5efad7ee4140c252c6ed3ab8c812dd` (unchanged at report close).

Two narrow technical clarifications should close before complete P2G-20 publication. No concrete material policy ambiguity or acceptance change established. Design is implementable in a bounded atomic phase subject to those clarifications and explicit GEX obligations; no implementation success is certified.

This is a scoped read-only design review against the six decided policies, all 13 acceptance entries and fixed RQ01–RQ07 plan. It is not a fresh completion audit. Root is the sole repository writer. No code/formal records were edited; no tests/builds/runtime execution or subagents were used. Root communicated intended successor fixes; this report evaluates candidate001 only.

## Findings

### DR01 — Separate mandatory admission/receipt/fuel from optional trace

Classification: `technical_decision`; disposition: `required_design_clarification`. Candidate lines 104–106, 114–124, 159–162. Policies: E03-AC2, E05-AC2, E06-AC1.

P05 correctly prohibits gameplay abort on trace collection overflow, but P04 stages observation/receipt caps and P05 then rejects receipt hard-limit violations. Without an explicit partition, adding trace rows to the receipt can make trace-on exceed the hard cap and roll back a step while trace-off commits. Optional observer work charged against gameplay execution fuel can cause the same problem. The phrase same admitted execution does not by itself prove equal admission for an otherwise identical request.

Minimal fix: State that mandatory gameplay result/binding documents and their deterministic admission budget are independent of trace. Optional trace has a separate bounded allowance, fixed bounded coverage metadata, and overflow/observer limitations have an observational disposition. Optional trace cannot consume mandatory receipt allowance or gameplay fuel, alter binding input documents, or change step publication. Preserve P04 rollback for genuine mandatory request/receipt errors. Add an at-cap test where trace-on overflows while trace-off does not and compare commit disposition, continuation and RNG.

Source basis: world/application.rs152–161,182–215 establishes authoritative commit and infallible publication; battle/event_log.rs616–643 and751–756 shows existing observational eviction and skipped source memory; candidate P04/P05 cross-reading.

### DR02 — Install control before generated setup/startup, including load-time route

Classification: `technical_decision`; disposition: `required_design_clarification`. Candidate lines 25–27, 47–55, 120–124, 161–162. Policies: E02-AC2, E05-AC2, E06-AC1.

Current startup begins before world active-session installation: events/combat.rs129–205 builds/validates scenario, constructs BattleCore and calls start_battle_execution at197–198. sim.rs1783–1817 validates, clones/places obstacles, builds runtime field and enqueues scenario events. This path also executes inside content_pack::load generated setup validation before any CoreApplication exists. A control installed only on active battle or subsequent time-zero/clock draft misses actual reachable setup work. Candidate intent includes time-zero preparation but needs this earlier source seam and load path made explicit.

Minimal fix: Add the exact load→generated-validation→setup/start and trial-preparation source chain to Current Actual and P06. Require authoring opt-in control or a source-backed bounded argument for this direct generated validation/startup route, with pre-allocation/expansion checks and load-stage cancellation/error propagation. Control must start before construction/start and not share mutable counters with A. Include low-fuel/cancel during generated setup and during valid-token trial startup; keep their phases distinct. Do not claim complete R23 protection.

Source basis: content_pack/mod.rs60–63; data/mod.rs1048–1051; data/validation.rs1485–1596,especially1579–1585; events/combat.rs72–97,129–205; battle/core/sim.rs1783–1817; world/combat.rs636–688.

### DR03 — Outer candidate needs private opaque fork/staging ownership

Classification: `technical_decision`; disposition: `implementation_obligation_not_policy_blocker`. Candidate lines 19, 104–106, 122. Policies: E01-AC1, E05-AC2, E06-AC2.

CoreApplication currently owns private GameCore and has no Clone or public mutable inner port. The proposed outer candidate is implementable inside the crate but is not an already callable public capability. The inner prepared commit owns only state/run_seed/game_data/output; added control/observation ownership cannot be assumed to follow it automatically.

Minimal fix: Keep exact private fork/stage/control/result plumbing an explicit GEX obligation. Preserve external privacy; compare complete authority including UUID/RNG, token, profile, receipt and quota across rejection. No public raw GameCore/BattleCore escape or active-run checkpoint substitution. Candidate already states the ownership invariant; this is a handoff reminder, not a reason to return to P2G-10.

Source basis: application.rs781–790,822–825; world/application.rs182–215; world/state.rs381–421.

## Acceptance coverage

| Acceptance | Proposed implementation / validation | Assessment |
| --- | --- | --- |
| E01-AC1 | P01/P04/P07; V01/V03 | Public external loader and command route exists; generic facade/CLI and nonzero scenario remain proposed. |
| E01-AC2 | P03/P04; V01/V02 | EFFECT_SUPPORT literal editability is separated from wider executor vocabulary; explicit finite observed bindings avoid content dispatch. |
| E02-AC1 | P02; V01/V02 | Same shared envelope is proposed, with producer-owned codes/phases; not current API/CLI parity proof. |
| E02-AC2 | P02/P04/P06; V02/V06 | Real available source preserved with unknown precision explicit; generated/startup phase control needs DR02 clarification. |
| E03-AC1 | P04/P05; V03 | Actual public battle preparation and nonzero external effect delta is mandatory; existing battle probe alone does not satisfy. |
| E03-AC2 | P04/P05/P06; V04/V07 | Explicit initial authority/seed/content/templates/resolved clocks/occurrences; DR01 closes optional-trace admission coupling. |
| E04-AC1 | P03/P05; V04/V05 | Readonly projection plus identical deterministic continuation verifies no state/RNG/revision mutation. |
| E04-AC2 | P03/P05; V01/V05 | Exact winner/effect source and compiled identities proposed; absent spans/derived chains are not fabricated. |
| E05-AC1 | P04; V06 | B requires actual battle entry/time-zero or verified immediately terminal battle identity before one aggregate swap. |
| E05-AC2 | P04/P06; V06/V07 | Invalid load and valid-token failed prep preserve complete A repeatedly; low-fuel/setup boundary added by DR02. |
| E06-AC1 | P02/P04/P06; V02/V06/V07 | Finite strict request/expansion/work/queue/output caps and typed rejection are required; numbers remain measured implementation decisions. |
| E06-AC2 | P03/P04/P06/P07; V06/V08 | Owned DTOs and last-reference release, ephemeral tool state, V6/profile/public contracts retained; no personal files or new midbattle save. |
| E06-AC3 | P07; V03/V08/V09 and166 | Actual hashes/toolchain/workload/nonzero execution and failure receipts mandatory; no runtime work executed in this design review. |

## Other assessed design boundaries

Finite Command/Snapshot/Clock/Assert steps with prior observed JSON-pointer bindings are a viable generic minimum for deliberately authored finite scenarios. No evidence requires a content-specific runner, private fixture, arithmetic interpreter or automatic gameplay decision. An author may choose explicit array positions in the recipe. This is not a claim that every arbitrary external pack can be automatically navigated to battle without its own finite inputs.

The outer tool candidate separates tool acceptance from ordinary public command commitment coherently: the inner core may commit inside an unpublished private clone, while the aggregate is installed only after mandatory result preparation. Previous successful tool steps remain committed. External transport failure after acceptance must remain a separate disposition. DR01 prevents optional observations from secretly changing this boundary.

P02/P03 correctly preserve source at known producers. LiveRonError retains field/action/effect but currently lowers parser details to strings; SourceTable maps final winning definitions to original file/collection positions and currently discards the table after load. WriterReceipt is already part of semantic identity, so retaining a copy must preserve its existing semantic input/order rather than reclassify it as irrelevant to identity. The proposal states that preservation. Original/Added origin, descriptor editability, actual compiled IDs and unavailable finer spans are distinguished honestly.

Battle event provenance is currently skipped in public serialization and terminal release state drops the active battle; observer extraction before terminal removal is therefore required. Actual effect application observations must retain deterministic occurrence identity without substituting log correlation or changing existing event/RNG allocations. Existing retention caps are not heap or total work bounds.

Process-local candidate/session/control/receipt ownership is a technical realization of the approved trial policy. Existing V6 checkpoint is content-bound and omits live battle; the tool must replay explicit inputs instead of claiming a lossless live checkpoint. Default in-memory profile provenance is explicit, and no existing personal-store files need to be touched. The absence of a public CoreApplication clone is an implementation seam, not material policy ambiguity.

A single minimum atomic phase is defensible because loader diagnostics, shared facade/CLI, source metadata, actual control/observation, candidate publication and validation share acceptance dependencies. It does not certify implementation cost or completion; exact code types, numeric limits and control propagation belong to GEX. Existing ordinary core semantics, V6/profile/public privacy and prior currentness must remain proven after implementation.

RQ01–RQ05 have scoped source support here; RQ06 predecessor currentness and full evidence inventory remain root-owned. RQ07 exclusions agree with adopted AUTORUN scope. This helper does not certify the root’s complete Review Batch artifact linkage or classification vocabulary.

## Exact inspected source identity

Hashes below come from raw bytes at report close; targeted spans were inspected by read-only nl/sed/rg or JSON payload reads. The candidate was hashed at both ends. Whole-source stability across every read is not independently proven. Helper reports are corroborating material, not substitutes for the independently inspected critical source.

- `/tmp/r03_code_review_candidate_001.ko.md` — SHA-256 `53c997b286792710741f31b2987ef961ba5efad7ee4140c252c6ed3ab8c812dd`; inspected: 1–181 (entire candidate).
- `core_enhanced/docs/goals/content_authoring_tool_ports/P2G_10_POLICY_001.json` — SHA-256 `eea71a35d9f66ddcf3c5fb60d395ca65dd9ee6117813869e57b7d32c12c83fa6`; inspected: PER-CATP-E01..E06-DECIDED-001 policy statements and all 13 acceptance entries; corresponding DR payloads.
- `core_enhanced/docs/goals/content_authoring_tool_ports/P2G_20_SOURCE_PLAN_001.json` — SHA-256 `0aef3066029105968090bf1b9b6b6959486fb89954d2fd5bf374248c00a448ee`; inspected: RQ01–RQ07 payloads.
- `core_enhanced/docs/roadmaps/content_authoring/AUTORUN_REQUEST.ko.md` — SHA-256 `3e05b131d445cd10ecf478ef959b80f55ce718c41c8a1820e5a5b7d969f9a252`; inspected: 1–140, especially 39–57,96–130.
- `/tmp/r03_runtime_trace_review_001.md` — SHA-256 `5f0a3ec32f9014ca73c8d5566024c85bd3485ad75d8a16c10c67f45fc9e89cdc`; inspected: runtime/private authority, observation, output commit, startup/fuel, lifetime and validation recommendations; helper evidence only.
- `core_enhanced/src/application.rs` — SHA-256 `636b3c9cc15ae2adfa39a15392e0dee22ee7c6683885087f9cc77cefda66081a`; inspected: 120–244; 780–847; 929–1028.
- `core_enhanced/src/game/world/application.rs` — SHA-256 `9eec3c8c6904f3473acbcfd66747f3717fa56b86772966327f75d689c6ec0554`; inspected: 130–218.
- `core_enhanced/src/game/events/combat.rs` — SHA-256 `65677baa2deee83e1dcae974500d0da7ed61c633c8688fd8804ef60600261614`; inspected: 49–210.
- `core_enhanced/src/game/world/combat.rs` — SHA-256 `23d440b58516258e9e174d586c6d06a675f9ce8c9b21534cde102574b2feb35f`; inspected: 590–696; 1347–1408.
- `core_enhanced/src/game/battle/core/sim.rs` — SHA-256 `ee52fbcd1f92c5730be3fa4e0d2d4dabca782c41721d66553cb0b60fa2b0c314`; inspected: 1780–1950.
- `core_enhanced/src/game/battle/core/commands.rs` — SHA-256 `1ccd3ca9b7c87339d32f0c7c047f1532c52a5be2eb7007079e0a90ca654440f6`; inspected: 445–579.
- `core_enhanced/src/game/data/content_pack/mod.rs` — SHA-256 `84936b0a6844316678a029d648abefd30055e345ce13b4924dcdc4e56c9dd32e`; inspected: 1–100.
- `core_enhanced/src/game/data/content_pack/raw.rs` — SHA-256 `5493fbfdfbf2cb07a1ec68f249e37445c87e080e97d23096398170b8dac44133`; inspected: 335–448; 555–655; 870–898.
- `core_enhanced/src/game/data/content_pack/resolver.rs` — SHA-256 `697b219ec8f60c63052e451708dfd8bb31805be1367bd1aee5b41344399d0561`; inspected: 30–65; 378–445.
- `core_enhanced/src/game/data/live_ron.rs` — SHA-256 `3e108ed0e0c6e0df0dfc69d5154db17e9f2f92f0e9f41c5145c1df171beff3c5`; inspected: 45–93; 1430–1460.
- `core_enhanced/src/game/ability/authoring.rs` — SHA-256 `c4ca67ceac4502c2c31c8c91a584e09dff3c87fd8976269d436dbddbc2e08e1a`; inspected: 10–55; 110–181.
- `core_enhanced/src/game/data/mod.rs` — SHA-256 `b82bc924568b98acf5bc61620e3efc3f5b774303c91e2ee2f5127b5894f42e33`; inspected: 220–286; 1038–1056.
- `core_enhanced/src/game/data/validation.rs` — SHA-256 `a8471db1aa6f08ff76f166afcb2ee1e2f944140d1af8f18f4884d7401ec5a8b4`; inspected: 507–640; 1485–1618.
- `core_enhanced/src/game/world/active_run.rs` — SHA-256 `c344f0fff0aa28e3f8afb9b5a48c2c706855dbca13c5c53342aa14d2b7d52b20`; inspected: 25–48; 103–156.
- `core_enhanced/src/game/world/state.rs` — SHA-256 `129a4063f7e448b5795fd370a22a997f66eb09cd3029fb87d41fee892cf34bb0`; inspected: 144–190; 375–441.
- `core_enhanced/src/game/battle/event_log.rs` — SHA-256 `e20c7ff9ab45a4797c615b744dbfc15848267af40b040f04a192ba6362ad2c5a`; inspected: 35–60; 610–656; 745–773.
- `core_enhanced/src/game/battle/skill_source.rs` — SHA-256 `4fff389728a0dcb34fa50b56108e8790a750205fa0d391c21e65fd4553ca50d1`; inspected: 1–45.
- `core_enhanced/src/game/battle/recording.rs` — SHA-256 `e3c1afedaf9b5ec61c76d415e2123dbea8a9c6d9213da5393dddfaf0e6beaf2d`; inspected: 1–93.
- `core_enhanced/tools/battle_kernel_api_audit/valid/src/main.rs` — SHA-256 `029179ae0b7d715ae01001e377800ec128276616be1406204750be94915cd745`; inspected: 1–120.
- `core_enhanced/Cargo.toml` — SHA-256 `f489f2392299ff61ecda75e5f2e2e83acb3e72419b4128cfd9cec064eb0d419d`; inspected: entire manifest.

Only new `/tmp/r03_design_review_001.md` and `.json` are written. No candidate successor is reviewed or approved by this report.
