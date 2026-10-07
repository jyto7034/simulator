# R03 mandatory HP/Stats observation source review 029

Bounded read-only implementation review requested by root under AUTORUN §8. This is not a completion audit, integrated observer approval, or test execution. Root is the sole repository writer. Only this new report pair was written; source/docs/Git/build/tests were not modified or executed. Read window 2026-09-10T12:22:23.461030+00:00–2026-09-10T12:25:08.324581+00:00; close hash check 2026-09-10T12:25:32.643591+00:00. Thirteen source paths, seventeen captured reads; no close hash drift.

**Finding:** no concrete implementation defect was established in the new mandatory HP/Stats owner, direct-source guards, or sticky rejection paths within the inspected scope. This conclusion does not cover the still-unimplemented public setup enable, entry binding, terminal transfer, optional trace, or complete public session failure consumer. Specific remaining validation dependencies are below.

## Actual producer ordering and counters

authoring_observation159–189 reserves a next application ordinal and potential next nonzero/source counts with checked arithmetic. The reservation is private and contains only scalar counters plus one immutable source Arc handle. It does not mutate the stored counters. reserve charges before that handle copy; failed checked arithmetic marks the owner failed and latches control::reject. commit192–225 computes actual nonzero HP using i64 subtraction (45–50), installs counters and latest witness, and retains the latest nonzero *sourced* HP witness separately.

Checking next_nonzero even for a later zero-delta application is not an independently reachable overflow in valid state: nonzero_hp_applications <= applications holds from default initialization and each commit; if applications can increment, nonzero can increment. Likewise sourced_applications <= applications. No wrapping or saturating semantic ordinal appears. The current field mutability permits crate-local tests to corrupt counters, but there is no external mutable observer API; key binding is deliberately pending.

Three real mutation ports are connected:

| Port | Reservation | Actual mutation | Witness commit | Later work |
| --- | --- | --- | --- | --- |
| apply_damage_result_and_record | damage_runtime445–448 after live target check |453–454 |456–466 |log468, resonance487, death496 |
| apply_hp_delta_and_record |532–535 after zero/missing/inactive checks |540–547 |550–560 |log561, resonance580, death589 |
| ApplyModifier command |commands618–621 after active target check |626–629 |630–640 |StatChanged642, HP clamp/death653 |

No callback/other observer reservation occurs between the reservation and its commit in these inspected ports. Reborrowing the checked target is safe in this synchronous interval: reservation changes only observer/TLS accounting. commit's “reserved observation owner” expect is supported by that ownership interval; no visible path removes/reset the owner before commit.

HP requested zero returns before reservation; missing/inactive target likewise returns. A positive heal against full health still executes its existing clamped assignment and records an application with zero hp_delta. Damage may likewise apply zero actual HP loss. This is application count, not nonzero-effect count. The distinct nonzero counter/witness records actual change. Stats records one complete before/after mutation, including a max-health clamp; record_runtime_stat_hp_change_and_finalize599–649 emits only the companion log/death path and does not add a second observer ordinal. That avoids counting one modifier twice because it emits both StatChanged and HpChanged.

A pre-existing arithmetic caveat remains outside this new observer's proof: damage_runtime455/549 calculate the legacy i32 event delta before commit, while the new witness's hp_delta itself is i64-safe. Very large u32 HP values crossing the signed boundary can make that legacy debug subtraction overflow if such a state reaches these ports. This is unchanged arithmetic, not a newly established external-pack defect; this review did not establish loader/reachable-stat upper bounds. Do not interpret the new i64 witness method as a proof that every earlier legacy operation is overflow-free.

## Source isolation and application ordering

commands486–534 retains the old recording source logic, then installs a separate authoring_direct_source only if observation is enabled. The new source handle clone follows its explicit charge at510. It restores both prior sources at522–523 before the result poll524, including the explicit source-admission failure branch511–513. Missing authored source or cast origin produces None rather than inferred provenance.

Unsourced process_commands448–456 and process_command_batches459–483 take the direct source, execute with None, and restore it afterward. Their normal exits and loop-break failures all reach restoration. The direct lower command executor has exactly those two producer callers and process_skill_commands among production uses found by rg; no fourth bypass was found.

This repairs the concrete nested death path identified in design026: an HP/Stats witness is committed before finalize_unit_death; death_runtime155–164 executes death-trigger batches through the new unsourced guard, so those ordinary item effects do not inherit the outer skill's direct source. Later genuine skill invocation will install its own authored source. Outer direct-source restoration after nested batches preserves the enclosing skill context without changing recording/log behavior.

Additional direct HP callers inspected were field-work completion428 and rest tick413. These are event-driven sources with no invented authored attribution. death_runtime167's reconciliation calls field_work163; reconciliation schedules later FieldWorkCompletion events (field_work277–318), not an immediate nested HP application. BuffTick explicitly routes its ordinary damage through process_commands (sim3375), which clears any direct source. This is not a proof of authored buff provenance: that source propagation is outside the HP/Stats implementation being reviewed.

The ordinal is allocated independently of log seq/cast seq/UUID and committed before death/nested effects, so directly nested applications cannot be ordered ahead of their cause just because the outer helper returns later. Latest source is immutable sharing; no mutable global sink or RNG feed was introduced. Logged skill provenance remains existing behavior and is not silently rewritten by observation-only guards.

## Infallible failures and their actual limit

control::reject367–386 first asks interruption for any existing failure/cancellation, preserving first failure, then latches a typed execution failure with current phase and no fabricated limit/requested fields. Outside an active scope it returns a domain error. A checked counter error additionally leaves MandatoryObservation.failed=true; the private accessor146–156 rejects that state even outside a scope.

In an active scope, reservation work failure or checked rejection returns before the HP/Stats mutation. The void HP functions return, and modifier breaks its command loop. The result remains rejectable despite those infallible signatures: process_skill_commands restores source then polls524, its skill consumer polls at sim2415, process_event polls inner completion2659, bucket polls1955, and run_with_provenance performs its final acceptance poll247. A counter-invalid observer cannot become an accepted success merely because its immediate helper returns void.

This is **candidate rejection**, not leaf-wide transactionality or an immediate stack abort. execute_committed_damage calculated a summary and may grant basic-attack resonance before it calls the HP helper (damage_runtime278–297); after HP reservation rejection it can still form the precomputed CommandExecutionSummary310–312. Any command batches then meet the sticky charge guard. Rest/field-work event code can perform subsequent bounded local mutations/logging before process_event's poll. Therefore outer code must continue treating the scope error as authoritative and discard the whole candidate; a void helper return or that summary is not a standalone successful execution result. No accepted-state escape was found in the inspected scoped callers.

An observer manually enabled outside control can retain a failed private owner and return from a void leaf; it is detectable through the private getter. This mode is useful for local fixtures but is not a promised standalone transactional observer API. Root states the future private trial will enable it under run scope. Public owner failure validation remains a next-step obligation, not a current completed seam.

## Copy, Arc, reset, and ordinary path

ApplicationWitness's direct owned payload is scalar/Copy change data and an Option<Arc<SkillEffectExecutionSource>>. Source's two Arc children retain EffectSource strings and SkillCastExecutionSource data, not GameDataBase. The borrowed serializer73–109 exposes actual authored fields and cast/proc fields without cloning the source graph or repairing unknown source from names. CooldownSource is a Copy unit/item enum; source structs' remaining owned strings live behind immutable Arcs.

The private owner has two witness slots, not a growable list. witness.clone at221 copies fixed data and at most one Arc handle; reservation's fixed charge3 covers the operation plus up to two source handle copies. The new owner does not accumulate a full history and does not claim trace retention.

BattleCore includes both new fields in its exhaustive destructure180–181 and visit230–231. ApplicationWitness and MandatoryObservation list every field in their struct visitors, including the serde-skipped failed flag. AppliedChange and ObservationKey use Copy-constrained visitors, preventing hidden Vec/Arc ownership from silently entering that path. Arc's visitor prices the handle rather than traversing its immutable pointee. No shared mutable observer state appears in the complete native clone.

New BattleCore initialization sets both fields None (mod506–507). reset_runtime_state_for_battle1631–1633 preserves enabled/disabled status but reinitializes counters/key/fault/source for a new battle through enable_authoring_observation. It is a fresh runtime reset, not a terminal-source retention operation. The future terminal owner must be moved/preserved before such a reset or active destruction.

For ordinary cores the observer stays None: reservation returns immediately without control work, and commit(None) is a no-op. Source take/restore operates on None. Existing gameplay calculations/log emissions/recording-source paths are preserved by the inspected changes; no ordinary numeric/output schema change was established here. This is source reasoning, not an ordinary-regression test result.

## Validation evidence and next obligations

Root reported focused102 on INPUTS073: 2 tests passed, 51.86s build/0.20s tests; full battle regression103 was running at the time of the message. I did not run these commands or read their raw receipts, so those outcomes are parent-reported historical execution, not this report's execution evidence. The earlier focused101 import compile failure is also root-reported and is not rewritten as a successful test.

I read the actual new tests commands2546–2722. They test repeated damage17, source Arc identity, fixture entries.clear independence, manually installed ambient source around an unsourced heal, a non-HP modifier, native clone serialization equality, work2 preventing an HP mutation, and u64::MAX counter rejection/getter failure. entries.clear is not production retention eviction; manually setting the ambient source is not an actual lethal callback. Current tests do not establish full observer/default-release/terminal acceptance.

Meaningful focused follow-ups:

1. Actual lethal sourced HP or max-health modifier -> existing death-trigger ordinary heal, followed by genuine proc skill: verify ordinal order, unsourced None, proc source, restoration, and no double count for the modifier clamp.
2. Zero-request vs clamped-zero vs missing/inactive target; negative Heal, overkill, and a sourced stats mutation that clamps HP. Verify the actual three counters and preservation of latest_nonzero_sourced_hp.
3. Cancel/work exhaustion at the new source-handle charge and at each mutation reservation, including nested batch exits; verify both temporary fields restore and full candidate rollback preserves accepted observation/continuation.
4. Clone a populated observation, mutate only the clone, and verify original counters/witnesses remain unchanged while immutable source pointers may match. The current native clone equality is useful but does not mutate the clone.
5. On the next production boundary: enable before actual trial startup, bind checked entry identity, require private fault getter success, preserve default-release terminal capsule before active removal, and only then serialize mandatory observation in the final receipt. Optional trace remains a separate allowance; it must not enter these mandatory counters or dynamic clone/work admissions.

### Exact captured inputs

Full timestamps/individual read spans/byte lengths and close hashes are in the JSON companion. All hashes below were computed from the same complete bytes used to display the numbered source; close hashes matched.

1. `core_enhanced/src/game/battle/core/authoring_observation.rs`, SHA256 `b79d42c1f6f7f1a337934c56de0dc6a591f7ab2d14e23b8e703d6dbd72863b96`; spans 1–226.
2. `core_enhanced/src/game/battle/core/commands.rs`, SHA256 `a755cd6fb08053d13b75124a9b680d66384b97dcee966ae092e765ed0f976d9a`; spans 448–550, 606–758, 2538–2730, 550–606.
3. `core_enhanced/src/game/battle/core/damage_runtime.rs`, SHA256 `423a368b26279ea6322fac1a64f6c078c2a4efa0ad7895c644e155f2ae623a6d`; spans 265–314, 427–649.
4. `core_enhanced/src/game/battle/core/mod.rs`, SHA256 `05cb798e42b9c2b38e00ce10dfc6f67a50b177f098bbe0fb50dd712d136f15d3`; spans 118–242, 492–514.
5. `core_enhanced/src/game/battle/core/sim.rs`, SHA256 `1061fdc3774f67504c9ba727602b03c0a16bc75ef8cc4f8b336693f07caefc3a`; spans 1628–1645, 2647–2674, 3325–3390, 1630–1668, 2400–2423, 1868–1888, 1940–1962.
6. `core_enhanced/src/authoring/control.rs`, SHA256 `5361b2eac90d518d17bedbc856b1b974489af9db839d7cb5bb59c677d7b263e9`; spans 310–401, 215–254.
7. `core_enhanced/src/game/battle/core/field_work.rs`, SHA256 `594a5aac3aebe0697f0c46583098852711a23436d17b1c0c9d7f3818788433a7`; spans 146–175, 265–323, 397–446.
8. `core_enhanced/src/game/battle/core/tactical_stance.rs`, SHA256 `61af123232aba8af42cdc864738cab3e174b787da8f403c824ff7fcca456f570`; spans 375–434.
9. `core_enhanced/src/game/battle/core/death_runtime.rs`, SHA256 `abdcb6de0b58487011f28df0652a7531fccabec2fa2e6ac833662d30eaf6ce71`; spans 88–171.
10. `core_enhanced/src/authoring/clone_shape.rs`, SHA256 `6e56d3990b14357c98663ace941fc44b6cde7c41667829b4dddec78e71adbc6b`; spans 35–66, 94–125, 150–158.
11. `core_enhanced/src/game/battle/skill_source.rs`, SHA256 `798d5a96e4550bab80cda46a724f13990be5cffe73d6a3535c179f2d3a8d7d30`; spans 1–35.
12. `core_enhanced/src/game/ability/authoring.rs`, SHA256 `c4ca67ceac4502c2c31c8c91a584e09dff3c87fd8976269d436dbddbc2e08e1a`; spans 10–55.
13. `core_enhanced/src/game/battle/cooldown.rs`, SHA256 `fa401d4db5a2c6572a8ef404979303fe4c5b957e1aa3160f62befef12a3d55f4`; spans 7–11.
