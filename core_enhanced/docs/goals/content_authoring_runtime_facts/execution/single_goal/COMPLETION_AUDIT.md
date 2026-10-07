# R04 GEX-20 completion audit — fresh initial 001

**Verdict: Not Complete.** Two completion-blocking findings require the existing R04 GEX-10 owner: direct strike identity is allocated after formula evaluation, contrary to E02[0], and the required t0/t100/t300 delayed-hit ActionHandle fixture is absent. The captured implementation and its valid passing executions remain useful evidence, but neither closes those requirements. This is an in-scope technical correction return, not a policy or access blocker.

## Exact target, authority and separation

The audited execution target is `(GS-CARF-P2G50-002, GOAL-CARF-01)`, `PHASE-CARF-01`, Handoff `HOF-CARF-P2G60-002`, Working Revision `WR-CARF-P2G20-002`, Freeze `FPF-CARF-P2G30-003`, Plan `PP-CARF-P2G40-004`, assigned final nodes `AG-CARF-01` and `WP-CARF-03`. The actual [Goal Set](../../P2G_50_GOAL_SET_002.ko.md), JSON member, Handoff002, Freeze003, Plan004, accepted Working basis, [CODE_REVIEW_003](../../CODE_REVIEW_003.ko.md), [DESIGN_001](DESIGN_001.md), PLAN, EXPERIMENTS, EXPERIMENT_NOTES, GEX_10_RESULT_001/002 and CORRECTION_001 were read as authority, design, execution history and evidence respectively. Implementation maps do not replace the exact twenty acceptance members, ten IVA, C01–08, I01–06 or ten required dimensions.

This is a separate fresh same-model reviewer (`/root/r04_runtime_facts_fresh_audit`) with no implementation conversation fork or implementation participation. The user-adopted AUTORUN §§6/8 and parent's exact review assignment authorize this pass. The actual pipeline revision `2026-09-09-gex-r1`, Reader Contract, §5 currentness, §6 evidence/quality/separation and GEX-20 govern it. No applicable additional ancestor/scoped AGENTS instruction was found. Because the Goal is `single_phase`, this audit includes the full Goal Set and broad/final baseline responsibility; no later GEX-30 substitutes for it. There is no separately approved shared initial/final repetition directive.

Writes were confined to this report and `audit-evidence/fresh_001/`. No source, tests, data, canonical contracts, owner records, old audit targets, fixtures or shared binaries were changed. No Cargo command, gameplay run, build or owner harness was executed by this reviewer. Findings were reported without repair; the implementation owner kept the reviewed target frozen through closure.

The sole implementation target is [AUDIT_TARGET_002.json](AUDIT_TARGET_002.json), `ATR-CARF-R04-002`, SHA256 `12dae8985bcb07f80f003e654d4076d4528683c3fe98631300300f39fd693d91`, 1,776,538 bytes. Target002 is the recorded capture-only correction that binds formerly omitted complete native budget/allocator reports and binaries, original P01 baseline and fixture identities. It does not replace the implementation with a new source revision. Target001 remains historical.

HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`, branch `cleanup/simulator-dirty-worktree-20260803`; toolchain is `rustc 1.91.1 (ed61e7d7e 2025-11-07)` and `cargo 1.91.1 (ea2d97820 2025-10-10)`. [Entry currentness](audit-evidence/fresh_001/currentness_entry.json) independently checks the target identity, all 4,190 manifest members, two required absences, exact membership and bytes of 39 external inventories containing 2,515 files, and three named default-release binaries. It also checks the actual supplemental targets and audit registries below. Final currentness disposition is recorded in the closure section.

## Actual predecessor supplements

These are actual current audit inputs, explicitly excluded future audit outputs of the frozen R04 implementation target. Their reports were read and their applicable producer semantics independently followed: R01 strict typed effect/source execution and errors; R02 exact pack generation/pinning and V6 compatibility; R03 current public trials, source-byte receipts, privacy, bounds and external API/CLI consumers. Each actual report says `Complete / High` on its own target. These verdicts do not certify R04 or override this audit's findings.

All paths in this table are under `core_enhanced/docs/goals/<topic>/execution/single_goal/`. Full paths, bytes and all member checks are in entry/closure JSON.

| Topic | Actual target / SHA256 | Actual report / SHA256 | Registry / SHA256 / outputs |
| --- | --- | --- | --- |
| `content_authoring_minimal_effect_contract` (R01) | `AUDIT_TARGET_005.json` / `631c3aa01a8ef7b3303bc9b979e572c81fdb0e11e2d41915112baff98e548af2` | `COMPLETION_AUDIT_005.md` / `ffed3caa83498b3064fe53fb9da42d96ebf25cb353bd5d74b90b49184b2491a3` | `audit-evidence/r04_successor_005/AUDIT_OUTPUTS.json` / `7a264d31561bff3c1522d726289352b19656bc59b068ca6ea25453001579c339` / 13 |
| `content_authoring_pack_generation` (R02) | `AUDIT_TARGET_005.json` / `a090b1856086f1c98c5d1630bb4464deb28b244eaed950514ce1132300fda57b` | `COMPLETION_AUDIT_004.md` / `b504c1101f24f64467db7aa29bea91131cebc4ed1e3c1b24d306aca61b69151e` | `audit-evidence/r04_successor_004/AUDIT_OUTPUTS.json` / `19e00cee37f3d3db341cae8c39ab97d3f045dcda9884a97e65b03b2250c24006` / 11 |
| `content_authoring_tool_ports` (R03) | `AUDIT_TARGET_003.json` / `a9e68225527c259f5c1eca054b12a92982139ed041928001f436889b6c7fb16c` | `COMPLETION_AUDIT_003.md` / `86a4176ca5ca930f7e2fa1f9cbba66e22ba9fcf754df03b26d28fff1de5b37df` | `audit-evidence/r04_successor_003/AUDIT_OUTPUTS.json` / `5785fb50f8634a7e146efc35d610270c82312de28621ce436a01f100fa295ca0` / 17 |

R01 verification covers 3,785 manifest members, 31 inventories/2,235 external files and three binaries; R02 covers 4,194 members, 39 inventories/2,515 files and three binaries; R03 covers 4,197 members, 39 inventories/2,515 files and three binaries. Each has two required absences. All 41 registered supplemental outputs are hashed at entry and closure. The prior superseded predecessor verdicts are not reused as current completion.

## Findings and exact correction owner

### F01 — blocking implementation gap: direct strike is identified after calculation

**Requirement:** `PER-CARF-E02-REVIEWED-001[0]`, IVA-CARF-04, C02/I02, Plan SC01 require distinguishing pre-calculation direct strike identification from post-calculation actual HP loss. This is explicit in Freeze003's acceptance, not an extra reviewer design preference. CRA003 P02 at line 64 specifies the strike key before the formula; DESIGN001 §3 likewise says a target receives its checked strike ordinal before its first formula evaluation.

**Actual source:** `core_enhanced/src/game/battle/core/damage_runtime.rs:352` calls `calculate_damage(&request, &ctx)` before `prepare_damage_fact` at line 353. The latter calls `fact_strike_segments` in `core/runtime_facts.rs:1353`. The sole direct strike allocation is `self.runtime_facts.strike()` inside that helper at line 711. `begin_strike_scope` at line 683 creates an empty target map; it assigns no strike. The optional `RuntimeFactSource.strike` field (`game/battle/runtime_facts.rs:139`) is not populated into a pre-formula context. A source-wide callsite/assignment review found no alternate pre-formula allocation or consumption path. Exact source hashes and line excerpts are retained in [finding_source_evidence.json](audit-evidence/fresh_001/finding_source_evidence.json).

**Consequence and evidence limit:** the implementation can group the completed damage mutations correctly by target and iteration, but no direct strike identity exists at the required pre-calculation boundary. The passing `runtime_facts_group_effect_major_damage_by_actual_iteration_and_target` test checks post-calculation grouping and the old HP-loss gate count; it does not observe an identity before the first formula. This finding does not assert that current numeric damage, RNG or old gate outputs are wrong.

**Attribution and return:** this is the R04 AG-CARF-01 addition in damage/fact integration, not a defect to repair in R01/R02/R03 or a need for new policy. Correction GEX-10 must admit/assign the direct strike before the first formula and carry that same identity into the actual segment/result context. Preserve basic-only pre-formula trigger semantics, formulas, existing gate/RNG/order, finite admission and operation rollback. WP-CARF-03 must add a meaningful oracle at that boundary, retain the current grouping assertions, rerun affected validation/cost/consumers, and produce a new stable target for a separate fresh audit. Source-impact predecessor restoration is required where applicable.

### F02 — blocking validation omission: t300 expiry is presented as the required delayed hit

**Requirement:** `PER-CARF-E03-REVIEWED-001[1]`, IVA-CARF-05, C03/I03 and C08/I06 explicitly require the actual current-carrier fixture with t0 Accepted, t100 Released and t300 hit, with FlowFinished and EffectsSettled distinguished by real progress.

**Actual source and mapped claim:** `ACCEPTANCE_EVIDENCE_001.md` E03[1] cites `native_action_area_observes_accept_release_control_finish_and_actual_settlement` and labels it t300 actual hit/settlement. The test at `core_enhanced/src/game/battle/core/mod.rs:9579` sets focus 100 ms, area duration 200 ms and interval 100 ms. The area implementation at `core/skill_runtime/area.rs:650` schedules the initial area tick at release (t100). Its tick rescheduling condition at lines 903–955 requires the next tick to be strictly before expiry, so the next hit is t200 and t300 is expiry/removal. The test observes action status through t300 but never proves zero HP/fact mutation in the release-to-delayed-impact interval. It therefore cannot certify the required first delayed hit at t300. Exact test/carrier excerpts are included in the finding evidence.

All current `request_skill_action`/`query_action` test callsites were inspected for another matching fixture. None supplies the required delayed-hit milestone oracle. The existing piercing empty-final-expiry test deliberately separates carrier removal from an unresolved old damage gate and is not a substitute. The external delayed-source evidence proves source propagation, not this specific ActionHandle timing contract.

**Consequence and return:** this is a narrow missing durable WP-CARF-03 validation seam owned by the same R04 GEX-10, not evidence that delayed projectiles fail in production. Preserve the useful area test and add a real carrier with acceptance t0, release t100, first hit t300. Query the actual handle before release, after release but before impact, and at impact/settlement; observe actual HP/facts so release cannot count as a hit and expiry cannot stand in for one. Record the applicable control terminal separately. Update the owner evidence map truthfully and rerun affected validation before freezing a successor target. No accepted requirement needs reinterpretation.

### O01 — nonblocking evidence text discrepancy

E01[0] in the frozen evidence map says `skill_runtime_contract25`; full164 actually runs **15** tests in that integration binary. The complete total of 1,361 is correct: 1,309 library plus 4/4/25/15/4 integration tests. This typo does not hide a missing current run or change an assertion and is not an additional blocker. The existing WP03 owner should correct it in the next legitimate evidence revision. The frozen file was not edited by the reviewer.

## Evidence validity, baseline and true execution counts

[verify_evidence.py](audit-evidence/fresh_001/verify_evidence.py) and [evidence_review.json](audit-evidence/fresh_001/evidence_review.json) independently reconstruct 353 dirty-baseline archive members, 329 final source archive members, all 230 INPUT007 entries and 63 attribution rows. The exact reconstructed scoped patch is 534,629 bytes with SHA256 `a76c731c2013b7d72fdcbee3c415136ae3f10aad2505edb82818bc000e78aaaf`. It includes eight added files and distinguishes the prior Working Policy/P2G correction from the runtime/tool/contract work. Unrelated preexisting dirty state was preserved and not credited as R04 work. The source/context inputs carried by the handoff were checked directly. INPUT007 is explicitly partial; complete target manifests and source archives bind the omitted integration/harness/fixture inputs rather than treating the partial list as a full snapshot.

The evidence verifier passes **1,130 read-only checks**. An initial audit-script reconstruction incorrectly used a `before/` header for newly added files instead of the original patch's `/dev/null`; that one auditor assumption was corrected after inspecting the actual format. The initial 1,129/1,130 result remains in `evidence_review_initial.json`, with the explanation in [audit_observations.md](audit-evidence/fresh_001/audit_observations.md). No implementation assertion or owner evidence was changed.

The reviewer reuses **26 actual owner command records**, including their original command, exit, raw log hash and captured result/input identity, enumerated in `evidence_review.json`. These are full164, build165, budget167, allocator168, guards170, alltargets171, defaultrelease172, examples173 and 174–191. A build is a build, not a gameplay test. Original timestamps and scope remain in those records.

`cargo test --locked --offline -p game_core` in full164 passed **1,361 individual tests**: 1,309 library, then 4, 4, 25, 15 and 4 integration tests, with zero failed/ignored/filtered. The actual test rows are retained in [full164_test_rows.txt](audit-evidence/fresh_001/full164_test_rows.txt). Zero doctests are N/A and are not added to the gameplay count. Existing assertions were reviewed in the scoped diff; necessary packed-command/equipment-binding consumer adaptations did not weaken their expected gameplay behavior. This full run and the final all-target/default-release/current-consumer executions satisfy the broad baseline execution responsibility to their actual oracles; they do not prove the absent F01/F02 oracles.

[verify_outputs.py](audit-evidence/fresh_001/verify_outputs.py) and [output_review.json](audit-evidence/fresh_001/output_review.json) pass **738 read-only checks** of captured stream hashes, whole result/receipt parity, source observations and cost bookkeeping. These are independent comparisons of existing executions, not 738 new runtime tests. There were **zero new gameplay runs, zero new builds and zero owner harness reruns** in this audit. Hash/currentness verification, source review and Python analysis of captured evidence are the new audit work.

### Original unchanged guards and finite cost

The original `runtime_architecture_final_validation/src/final_performance.rs` guard logic, original fixture and P01 baseline were read and their exact captured identities checked. Its per-sample allocation limits, deterministic parity, repeatability, plain p95 and permutation comparison were recomputed from the captured raw results. No baseline/golden or threshold was loosened. Native budget167's full 6.9 MB result contains 67 search cases, a default execution and six exact/one-less endpoint executions, all checked for actual typed faults and complete fact/log parity at success.

| Existing measured boundary | Actual current evidence and limit |
| --- | --- |
| Native five-second fixture | Minimum clone 843,279; work 2,600,736; retained 6,090,008. Exact minima match the complete baseline facts and log hash; one less produces the selected typed fault. Defaults remain 1,048,576 / 10,000,000 / 8 MiB. |
| Kernel plain event/concurrent | p95 13.180868 ms under 25 ms; 8.182800 ms under 10 ms. Captured original comparisons do not show a statistically supported regression under their own unchanged guard. |
| Kernel allocator event | 41,660 operations; maximum peak requested bytes 255,622; original cap assertions pass. |
| Kernel allocator concurrent | 367,088 operations; maximum peak requested bytes 9,018,492 under the original 9,030,188 cap, a margin of 11,696 bytes. |
| Original ordinary public paths | `ORDINARY_GUARDS_003` retains original public plain/allocator checks and current deterministic outputs; not replaced with the new fact-only workload. |
| New-state cost002 | 685 plain samples over five repeats and 411 instrumented samples over three repeats, one warmup each. Every 137-phase sequence has matching complete receipt/work data. 126 native fact queries allocate zero times. |
| Lifetime cost002 | Twelve measured load/drop pairs reclaim each of four coexisting generations' 1,658,777 retained bytes; sampled maximum 25 accepted/24 settled, one active action, one delivery, four actor and two provider rows, 39,892 fact retained-byte bound. |

These measurements establish the recorded workloads and retained ownership, not arbitrary gameplay performance or RSS. Requested allocator bytes are not process resident memory; phase/step sampling is not instruction-level high water. The small concurrent peak margin makes an F01 implementation correction a material reason to rerun the affected original guards. Source admission and rollback review below is additional evidence; timing PASS fields alone are insufficient.

### Actual direct consumers

Captured command stream hashes and raw result payloads were checked instead of relying only on summary PASS fields. `PREDECESSOR_PACK_002` (174) has ten recorded packaging/consumer commands and actual 500→733→500 currency, preserving unrelated custom files. CLI175 has 21 commands, 13 CLI cases and six forbidden compile cases. API176 has 11 original public checks including expected privacy/fixture-feature failures. Diagnostic177 preserves exact source file/field errors for six negative calls and restores input bytes. Source178 has 36 failed replacements, three successful replacements and three caller-edit proofs with byte-source identity and off/on/gap parity.

RUNTIME_FACT_EXTERNAL_002 (179) contains four API replays, two CLI simulations and two queries. Off/on/gap/repeat agree on complete mandatory output: 91 damage mutations/strikes, 277 HP lost, two deaths and one actual tactical payment of ten. SOURCE002 (180) uses trace-off native source as output authority; captured trace is only an independent oracle for delayed projectile/periodic area provenance. Ordinary CoreApplication181 has 13 actual steps and the same terminal facts as Trial; consuming the result leaves no current world fact owner.

Current predecessor combat185 runs 12 commands and preserves A18/B36 damage and the 7,950 ms application. Resource186 and buff187 each have ten commands with whole-output mode parity. Invalid188 has 13 cases, 26 API attempts and 26 CLI attempts in 29 top-level commands, with typed deterministic rejection. Delivery189 has nine commands and preserves delayed repeat/projectile/area behavior and terminal 19,950 ms. Preview190 has four commands, four API replays, one discovery and nine positive previews. Replacement191 preserves failed/successful A/B swaps, continued A state and bounded rejection. These literal isolated current RON inputs pass production loading/validation/compilation and default-release public execution; the audit does not turn them into proof of F02's absent handle fixture.

## Source and structural quality assessment

Direct inspection covered the scoped implementation diff and current producer/consumer paths, including the physical queue and native clone changes. The following are source-level findings within the captured implementation, separate from runtime evidence reuse.

| Boundary / Plan structure | Actual implementation and assessment |
| --- | --- |
| SC01 actual mutation/facts | Damage formula and basic trigger selection retain the old request/context calculations; combined HP transition is captured without inventing a per-component overkill split. Heal records signed requested versus actual change, including zero/overheal/negative, and does not invent Damage/Cost classifications. Resource modifier change and real resonance/tactical Cost are separate. Post-calculation strike grouping works, but required pre-formula identity fails F01. |
| SC01 death/predeath | Predeath source and active buff snapshot are admitted before lethal mutation/cleanup. Lethal damage, signed Heal/stat HP and direct Kill enter the boundary; Despawn stays distinct. Mandatory lifecycle/dead/wave processing precedes optional death/kill dispatch and remains once-only; victim buff source is distinct from killer source. Existing cleanup/graveyard and sorted ally reaction order remain authoritative. |
| SC02 source/current carriers | Ordered provider-bearing batches preserve effect-before-activation order; nested source/strike scopes restore their caller. Common native source survives ApplyBuff, buff ticks, basic attack children, skill steps and projectile/area snapshots independently of optional narration. The remaining legacy `source.zip(origin)` belongs to observation formatting rather than fact authority. Scope is flat known provenance, with explicit absence for unknown root/copy. |
| SC03 action state | Public qualification and result-slot admission precede acceptance publication. Opaque immutable token pairs validate relative to the queried core; cloning does not share revocation state, and an aborted candidate's reused ordinal cannot validate its old token. Reserved results cannot be silently evicted; take/query are typed and bounded. Control terminal is once-only, finite delivery tickets and pending applications settle separately, and long internal actions retire to bounded summaries. F02 leaves the exact delayed-hit milestone validation incomplete. |
| SC03 cancellation and gates | Withdrawal removes pending/active control and Follow areas while released projectile/GroundFixed continue. HardCC, Silence and pending-only InterruptCast preserve existing differences. Pending death can reach the old CastEnd resonance reset without a false Released state. Death does not acquire withdrawal-style cleanup. Piercing actual removal closes its ticket after pending effects while leaving an unresolved damage gate/deferred control unresolved; no invented FlowFinished callback. |
| SC04 real equipment copies | Both player-spawn draft producers and both runtime build/deploy paths pass actual EquippedRef inventory instance IDs with definition IDs. Runtime item IDs retain their old derivation and cooldown/RNG role. AtBattleStart and Wave tests use real same-definition owned copies; scenario/fragment source absence remains None. Inventory serialization remains the authority. |
| SC05 transaction/retention | Native operation work/first-fault state lives outside cloned candidate gameplay state and remains sticky until final poll. Runtime transactions stage before mutation; timestamp stepping commits successful prefixes but aborts a failed timestamp. Outer world candidate rollback retains its existing wider operation scope; in-candidate ports are restricted to actual staged owners. Action/source/receipt/queue/resource/RNG/ID/log state participates. Fresh attempts invalidate old tokens; returned immutable historical receipts have no new authority. |
| SC05 finite storage | Active/result reservations, ticket sets, predeath buffers, strike segments, sources and counters are admitted with checked finite limits before growth. Maintained ledgers avoid a lifetime-history scan. COW B-tree bounds account for physical slots/edges and empty roots; actor vectors use sorted exact-capacity rows. New live retained payload and positive growth across one operation are accounted independently of shared clone handles. |
| Physical queue and clone changes | Packed events compare the complete original event ordering; metadata/Arc addresses do not affect priority. AttackStart/movement remain inline, admitted larger payloads are shared, and unpack admission precedes materialization. Heap and same-time bucket growth preserve old ordering with physical admission. Native unit-buffer reuse requires matching unsaturated insertion lineage, length and key order and is disabled for callbacks/authoring; exhaustive fields are copied. It does not reseed legacy maps or claim a total clone bound for arbitrary external mutation of the public HashMap. Existing authoring closed-construction CloneShape traversal remains present. |
| Scenario/movement sharing | Immutable scenario registry/movement data and lazy empty joint state use existing runtime content and consumer paths. They preserve targeting/timing and do not create a new content-name execution branch or synthetic joint insertion. |
| SC06 public/document boundaries | RuntimeFactView, ordinary world/CoreApplication, authoring MandatoryObservation/Trial and CLI consume native facts. Handle serialization cannot manufacture authority; no Deserialize handle or ordinal-as-authority shortcut was added. Canonical core/skill/data contracts document bounded source/action/copy and current unsupported scope. DESIGN001's pre-formula statement is contradicted by F01, and the milestone evidence text by F02, so full documentation/evidence consistency is not closed. |

The scoped review found no production test-name/fixture-ID branch, silent invalid-data normalization, broad catch-all that converts the reviewed faults into success, new content-specific executor, consumer-side duplicate fact authority, guessed persistent-copy fallback, unneeded future Flow/root registry, arbitrary timing/retry workaround, ignored failing test or weakened original allocation threshold. Existing runtime item/log/trace values have their prior legitimate roles and do not duplicate the new fact authority. Those quality guard checks do not erase the two explicit gaps. The unused pre-formula strike slot cannot be counted as an implemented boundary merely because the type exists.

Active attempt durable storage was examined in the actual V6/checkpoint loader/writer: `RunCheckpointPayload` does not serialize an active BattleCore/TrialSession, so restoring ActionHandles from a durable save is currently N/A. Live clone/reconnect is applicable and covered; existing owned inventory IDs still roundtrip. No new save-loss rule or migration was invented. External arbitrary retained clones of returned immutable values are outside engine-owned retention, and arbitrary external allocation history of the legacy public units HashMap is not claimed to have a new total physical bound.

## Twenty acceptance members

`E01[0]` abbreviates the exact zero-based member `PER-CARF-E01-REVIEWED-001[0]`, similarly E02–E05. “Supported” below is the narrow member assessment of this frozen target, not a positive overall verdict.

| Exact member | Assessment | Direct evidence and relevant limit |
| --- | --- | --- |
| E01[0] | Supported | Old formula/basic-only trigger, resonance and actual-HP-loss gate code; full164 original skill/basic/gate regressions, combat185 and unchanged deterministic kernel/public guards. O01 is clerical. |
| E01[1] | Supported | Lethal combined HP requested 2,000/actual 1,000 test; signed-Heal/predeath test distinguishes over/zero/negative actual change, no HP Cost invention; unchanged old actual damage summary assertions. |
| E01[2] | Supported | Actual CastEnd/manual and world tactical resource owners produce Cost; pending-death reset test, rejection-before-mutation/result admission, resource modifier requested −100/actual −30 with zero Cost; external179/application181. |
| E01[3] | Supported | Predeath then mandatory death/wave once, optional suppression tests, eight-case stat-HP death table, direct summon Kill versus Despawn; source preserves reaction order. |
| E02[0] | **Not closed — F01** | Existing two-effect/two-iteration grouping test proves completed-mutation deduplication, but strike allocation occurs after calculate_damage. |
| E02[1] | Supported | Signed-Heal/predeath test retains victim buff source and distinct killer equipment before cleanup; expiry/duplicate death and sourced/unsourced stat-death table preserve prior HP/source. |
| E02[2] | Supported | Native source fields in queued and active carriers; clearing event log retains full summary; external179 off/on/gap/repeat and source180 delayed provenance. |
| E02[3] | Supported within published scope | Actual RuntimeFactView→CoreApplication/Trial/CLI and describe/source outputs; explicit absent true join/root/fragment/zero-HP eligibility. Does not excuse E02[0]. |
| E03[0] | Supported | Native transaction qualification and reserved-result admission; no-target/invalid/discarded request tests, failure-before-focus/queue and public finite-limit errors. |
| E03[1] | **Not closed — F02** | Cited area hits at t100/t200, expires at t300; no actual t0/t100/t300 delayed-hit ActionHandle oracle. |
| E03[2] | Supported | t50/t150 withdrawal test with distinct undispatched control; old released projectile/GroundFixed, Follow, HardCC/Silence/InterruptCast and death carrier tests. |
| E03[3] | Supported | Terminal/delivery once; clone-relative token and aborted ordinal reuse tests; reserved-result and exact-cap typed failures; piercing empty-final-expiry preserves unresolved gate/control. |
| E04[0] | Supported | Native world same-definition two-copy test gets real owned inventory IDs and checks provider snapshots, distinct transient runtime IDs and unchanged derivation. |
| E04[1] | Supported | Both player drafts and constructors inspected; AtBattleStart/Wave coverage, durable inventory roundtrip, all four live-item integration tests and existing equipment/save consumers. |
| E04[2] | Supported | Scenario sources retain absent persistent-copy values; actual external source witnesses agree; no base/runtime UUID guessing or R07 claim. |
| E05[0] | Supported | Failed-timestamp prior-prefix/retry test, discarded request/result take/predeath rejection, bare staged operations and outer application candidate source; old late-failure/authoring atomicity suite. |
| E05[1] | Supported | Clone-independent cancellation/completion and aborted-token tests; old entry/retreat/restart/current-authority tests; application181 releases world owner and cost002 measures reconnect/replacement/drop. |
| E05[2] | Supported | New state in live snapshots and rollback; actual V6 writer/loader proves active-attempt durable N/A, with existing checkpoint/profile/copy roundtrips retained. |
| E05[3] | Supported for actual scope | Finite source/state/result/clone/fanout admission and overflow source; native exact/one-less budget167, unchanged original guards168/170/ordinary003, cost002 and mode/repeat/pack/container comparisons. |
| E05[4] | Current executions supported; overall closure withheld | Applicable T07/T23/T26/T28 full/default release/alltargets, direct consumers and actual restored predecessor audits are current. F01/F02 prevent complete R04 acceptance/canonical-evidence closure; later R work is not counted. |

## Integration, completion conditions and invariants

| Integration obligation | Assessment of actual durable seam |
| --- | --- |
| IVA-CARF-01 | Supported: old formula/basic snapshot/self-heal/repeat gates, overkill/signed Heal; pending-death CastEnd and post-death queued/Follow behavior retained; piercing removal versus old gate checked where applicable. |
| IVA-CARF-02 | Supported: actual manual/auto resonance reset and tactical deploy/relocation/swap payment/rejection paths; negative resource modifiers remain changes, including GF001/002 source regressions. |
| IVA-CARF-03 | Supported: lethal/stat/direct Kill/Despawn and predeath victim/killer split, mandatory/optional death boundary and original order. |
| IVA-CARF-04 | **Not closed: F01.** Current post-mutation grouping, source-death carrier provenance and log-independent public output are supported; pre-formula strike identity is missing. |
| IVA-CARF-05 | **Not closed: F02.** Qualification/native limits and other milestones have tests; the explicitly owned delayed-first-hit handle fixture does not. |
| IVA-CARF-06 | Supported: actual cause-specific cancellation, terminal/settlement distinction, stale/duplicate/aborted handles and GF001/002 behavior. |
| IVA-CARF-07 | Supported: real same-definition copies through both actual draft/build/deploy paths, source equality with inventory IDs, preserved runtime identity and explicit absence. |
| IVA-CARF-08 | Supported: failed request/timestamp/outer operation rollback, retained successful prefix, live clones/entry/retry/result/drop and durable applicability, including required GF regression assertions. |
| IVA-CARF-09 | Supported within recorded safety/cost scope: inclusive boundaries and repeated typed failure/recovery, native/authoring source-state limits, determinism, original performance guards and measured retained/reclaimed state. |
| IVA-CARF-10 | **Overall incomplete.** Current literal public API/CLI, 1,361 tests, alltargets/default-release, predecessor restoration and contracts reviewed; source/evidence consistency and complete fresh acceptance closure fail F01/F02. |

| Completion / invariant | Assessment |
| --- | --- |
| C01 / I01 | Supported: actual damage/heal/cost/death and mandatory bookkeeping preserve existing formula/basic trigger/gate/resonance semantics. |
| C02 / I02 | **Not closed — F01**; known source/predeath/log independence is otherwise supported. |
| C03 / I03 milestone part | **Not closed — F02**; actual required delayed-hit validation is absent. |
| C04 / I03 terminal/cancel part | Supported: cause-specific terminal once, finite released settlement and stale/duplicate validity, without promoting an unresolved gate. |
| C05 / I04 | Supported: actual persistent copy identity through all current player constructors, existing runtime identity and inventory storage. |
| C06 / I05 operation/lifetime part | Supported: candidate/rollback/prefix/live/durable boundary and observed reclaim. |
| C07 / I05 bounded-cost part | Supported for captured inputs and documented native/new-state scope, with the stated allocator/sampling limits. |
| C08 / I06 | **Not closed — F01/F02**; complete accepted producer-consumer and durable evidence consistency cannot be declared. |

## All ten required quality dimensions

No dimension is waived or globally N/A. Durable active-attempt save is only a source-proven subcase N/A within the required persistence dimension.

| Required dimension | Assessment and actual evidence |
| --- | --- |
| success | **Incomplete F01/F02** despite direct numeric/source/copy successes, native179/application181 and current API/CLI consumers. Pre-formula identity and required delayed-hit observation are part of success. |
| invalid_input | Supported: no-target/invalid-state/native limit qualification, invalid188 production loader and API176/CLI175 privacy negatives; typed rejection without Accepted or success receipt. |
| failure_rollback | Supported: E05[0] bare/outer candidate tests and source, native exact167, source178/replacement191 preserve queue/resources/RNG/ID/log/new tokens and prior successful timestamp prefix. |
| retry_idempotency | Supported: duplicate/late completion, result take, aborted ordinal reuse, repeated fault then success, old attempt tokens and clone-relative independent authority. |
| ordering_concurrency | **Incomplete at F01's required pre/post calculation boundary.** Existing gameplay same-time/death/trigger/gate/carrier order, packed-event comparisons and original parity/repeatability are supported; logical concurrent sessions have independent owners, with no multithreaded gameplay claim. |
| boundary_empty_max | Supported: overkill/overheal/signed zero, empty target and final piercing expiry, action/result/source/predeath/physical-tree/ordinal bounds, original exact/one-less native167. |
| disconnect_restart_persistence | Supported: live query/clone/reconnect, failed candidates, entry/retreat/restart/result/drop and real inventory roundtrip; actual V6 source establishes active-attempt durable N/A. |
| live_data | Actual current literal RON production execution is supported by external179 and185–190 and malformed188; **F02's required timed carrier validation remains missing** and is not replaced by those successes. |
| producer_consumer | **Incomplete F01/F02.** Native mutations/carriers/inventory→native query/summary→ordinary app/Trial/CLI are present and tested, but complete accepted boundary behavior/evidence is not. |
| legacy_fallback_absence | Source review and negative public consumers support absence of log authority, guessed copy/root, future dummy executor, duplicate fact authority and unauthorized fallback. Original narration/transient runtime IDs remain legitimate existing roles. |

## Remaining scope and disposition

No required current core consumer was inaccessible during this pass. The two identified omissions are not labeled out of scope. Server/Unity deployment, external host adapters/personal store populations, future R05–R24 full ResourceStore/Flow/boss/relic/fragment/chain behavior, true joins/periodic roots and new HP-cost/shield/HP1/parry/special-kill gameplay remain excluded by the accepted Goal. This audit claims no external rollout or future completion.

Return F01 to **AG-CARF-01 with WP-CARF-03 validation**, and F02 to **WP-CARF-03**, both in correction GEX-10 for the same exact Goal. Preserve all immutable history, correct the implementation/fixture and evidence in new owner records, determine affected final baseline and predecessor restoration from the actual changes, freeze a successor target, then obtain a separate fresh GEX-20. This Not Complete result does not authorize normal dependent completion or substitute for that audit.

## Closure and immutable output boundary

**Currentness: current / unchanged at entry and closure.** The [closure verification](audit-evidence/fresh_001/currentness_closure.json), started at `2026-09-11T01:46:40.891387+00:00`, completed successfully. It rehashed all applicable members, required absences, exact external inventories, named binaries and all actual supplemental audit outputs. Its result equals the entry verification, started at `2026-09-11T01:36:08.016349+00:00`, in every field except the verification start timestamp. All four targets and the three supplemental report/registry identities remained unchanged, with no errors. [closure_identity_comparison.json](audit-evidence/fresh_001/closure_identity_comparison.json) records this direct comparison and when it was checked.

The final [AUDIT_OUTPUTS.json](audit-evidence/fresh_001/AUDIT_OUTPUTS.json) binds this report and every audit-local output by exact SHA256 and byte length, excluding the registry itself, and records the exact implementation/supplemental inputs, verdict and true execution counts. Output hashes and exact registry membership were read back after writing. The sealed report contains **two blocking findings, one nonblocking clerical observation, zero new runtime/build executions, 26 reused owner command records, 1,361 reused passing tests, 1,130 final evidence checks and 738 output-comparison checks**. Passing currentness preserves the attribution of this **Not Complete** verdict; it does not turn it into positive completion.
