# R03 runtime control 읽기 전용 보조 검토 002

Two concrete normal-result/ID consumption gaps in the captured failed-draft paths; outer sticky rejection still holds in inspected control. No additional concrete replacement arithmetic, aggregate HashMap-order, or newly inserted panic defect identified. Current changed successor source is not certified.

Root는 검토 중 다른 runtime leaf와 clone/area-history 코드를 수정했다. 따라서 이 보고서는 캡처한 source의 좁은 GEX-10 보조 검토이며 완료 감사나 현 source 승인서가 아니다. INPUTS_014/ focused0196pass는 위치 단서로만 받았고 실행하지 않았다. Source 수정·시험·build·runtime 실행은 0회다.

## Concrete findings

### RCR02-01 (P2) — A failed retarget/context lookup still creates a normal AbilityStepTriggered event ID

Primary: `core_enhanced/src/game/battle/core/sim.rs` (2264–2277, 2310–2343); SHA-256 `70c777297c74d7a95aebf8ee7e5cf6024b12c95b6829b3dcfddd9d239c13bd0b`.

Trigger: A RetargetOnStep skill reaches execute_skill_step with enough fuel for its entry charge but insufficient fuel for skill_target_candidates or the guarded tile-pattern query. Use Always and Once so the absent target does not short-circuit gameplay condition handling.

Actual: resolve_skill_step_context returns None with a sticky execution failure; execute_skill_step treats it as a normal missing target and appends AbilityStepTriggered before the next charge/poll. A normal source/event sequence is consumed inside the failed draft.

Limit of finding: Outer control remains sticky and acceptance polling rejects, so this is not evidence of live publication or successful trial completion. It is a concrete failure-result/ID consumption gap at the newly guarded lookup boundary.

Minimal fix: Poll immediately after the initial resolve_skill_step_context result and before condition/repeat evaluation or normal event creation. Preserve ordinary no-target behavior when no execution fault exists.

Focused check, not run: Force failure at skill_target_candidates; assert exact failure code/site and unchanged event_log_seq/AbilityStepTriggered count after the lookup failure, in addition to outer rollback.

Other inspected source: skill_runtime/cast.rs377–397: choose_skill_target_by_rule converts failed charge/geometry admission to None; sim.rs982–1048: RetargetOnStep forwards that Option; sim.rs1147–1187: Always/Once do not reject missing target; recording.rs13–35: normal event append advances event_log_seq.

Root acknowledged this exact path and stated it would add the poll. Source changed during review; successor correction was not re-reviewed or certified.

### RCR02-02 (P2) — Persistent area registration returns true after its context/queue control failure

Primary: `core_enhanced/src/game/battle/core/skill_runtime/area.rs` (626–662); SHA-256 `475fbd736d8b6435acc64df354a0196833d73d84a7a68373e6cbbb18c31e0c84`.

Trigger: Admission allows the area object/reference creation but fuel/entity limit fails while storing impact context, or queue admission fails on SkillAreaTick/SkillAreaExpire enqueue.

Actual: register_persistent_tile_area continues past the failed context update / enqueue and returns true. execute_skill_step increments spawned_areas from this success value before its final scope poll.

Limit of finding: The final/next-loop scope check still rejects the outer attempt; no successful external publication was demonstrated. The helper result nevertheless advertises successful registration after a known execution fault.

Minimal fix: Poll immediately after impact-context storage and before returning true after both enqueues; ensure caller does not increment its success count for a sticky failure. Retain the existing bool semantics for legitimate non-authoring no-spawn cases.

Focused check, not run: Force impact_context_storage failure and each queue enqueue limit separately; verify the registration result is not successful and no success count is consumed, while the same first failure survives to the outer owner.

Other inspected source: cast.rs339–375: update_skill_cast_impact_context may return early on work/replacement failure; area.rs643–661: subsequent queue operations can also set or observe sticky failure; sim.rs2584–2599: caller counts true as a spawned area before checking scope.

Sent to root during review. Area source changed during concurrent tick/clone work; report records the observed pre-correction path rather than approving current bytes.

## 그 밖의 좁은 판정

- **Runtime replacement arithmetic**: No concrete arithmetic defect found in inspected implementation. Current shape includes context map entries, the last-context record, and both hit vectors. Replacing prior p hits and last l hits with n hits each uses C-p-l+2n+new-map-entry+new-last-entry, matching actual insert/overwrite storage. checked_sub underflow forces over-limit sentinel; final admit uses checked addition. Sources: core/mod.rs220–271; cast.rs339–374.

- **HashMap enumeration and usage/failure**: No concrete ordering defect found in the aggregate shape scan. Each structural count charges exactly one unit under the same site, while the size sum is admitted only after the whole scan. Different HashMap traversal orders therefore do not expose partial size totals in the entity-limit failure or peak. Number of scan calls depends on shape, not order. Cast deferred/context maps are BTreeMap, and deferred release preserves their ordered keys. This does not certify excluded target-usefulness/backend code. Sources: core/mod.rs220–271; cast.rs263–316; types.rs326–346.

- **Detached projectile accounting**: advance_skill_projectile removes its runtime before motion. The motion check adds 1+existing hit count back to the scanned retained shape; each accepted hit checks 2+old hit count (runtime record + old hits + new hit). This avoids ignoring the detached record and does not double-count it in self. The private motion/impact Result failures return before normal motion/hit result consumption. Sources: projectile.rs141–178; 195–205; 425–524.

- **Geometry/target guard calculations**: Tile rows are ordered; each row is admitted before cells increment, and projection/sort multiplications saturate into finite fuel rejection. Target vector collection is precharged using unit/tile counts, then sorted by UUID. No new unchecked indexing or panic path was identified in these inserted guards. Object-count admission is distinct from heap/geometry memory accounting; this review does not enlarge it into a whole-process bound. Sources: area.rs36–67; 301–348; cast.rs388–397.

- **Ordinary execution, IDs and order**: Control helpers return success outside ACTIVE. The inspected guards do not change ordinary branch choices, UUID formulas or comparison/heap ordering. Projectile spawn admission occurs before next_skill_delivery_id and successful dispatch count is polled before incrementing; instant area target polling occurs before its delivery ID allocation. The two findings identify remaining places where a failed scope still consumes a normal event/success result. No runtime equivalence test was executed. Sources: control.rs218–265; projectile.rs618–678; 880–887; sim.rs2478–2498.

## Actual source identity and drift

Line spans refer to the inspected snapshot, not shifted current lines. Initial hash then nl/sed reads were used for most files; control was later captured/read from one byte buffer at its explicitly recorded reviewed hash. Root changes were reported while inspection proceeded. Final hashes detect drift but do not certify or automatically close corrections.

- `core_enhanced/src/authoring/control.rs`; inspected 163–195, 218–265, 323–404; initial `65e2025350d2b8f8ffc6c7e5c646e92c894cae3fc6d2375506f8ff10b8169709`; reviewed `ad20a4e3a75a9bb16b402f50631559ae5028b760411a11b612d8945944461f8d`; close `ad20a4e3a75a9bb16b402f50631559ae5028b760411a11b612d8945944461f8d`; drift from reviewed=False.
- `core_enhanced/src/game/battle/core/mod.rs`; inspected 195–282; initial `d8e3914ceec1f91597349ad073150ba2ce2b5fb3628d7075625a3a668dec067c`; reviewed `d8e3914ceec1f91597349ad073150ba2ce2b5fb3628d7075625a3a668dec067c`; close `026b85b1b7d47cfc3f08d3e88467dfe7de9558926aa1783e76b3cb56bf5bf67e`; drift from reviewed=True.
- `core_enhanced/src/game/battle/core/skill_runtime/cast.rs`; inspected 1–415, 409–586; initial `f00e9f6264d8c6ef48ab480c469d1b662b3928a4664e7dcf9d56cc07a13b0a67`; reviewed `f00e9f6264d8c6ef48ab480c469d1b662b3928a4664e7dcf9d56cc07a13b0a67`; close `f00e9f6264d8c6ef48ab480c469d1b662b3928a4664e7dcf9d56cc07a13b0a67`; drift from reviewed=False.
- `core_enhanced/src/game/battle/core/skill_runtime/area.rs`; inspected 1–146, 155–287, 280–383, 485–670; initial `475fbd736d8b6435acc64df354a0196833d73d84a7a68373e6cbbb18c31e0c84`; reviewed `475fbd736d8b6435acc64df354a0196833d73d84a7a68373e6cbbb18c31e0c84`; close `b7cad108fd507a19ee7893bffc763bd9be06f6ab2187306a19f21b48042ea89d`; drift from reviewed=True.
- `core_enhanced/src/game/battle/core/skill_runtime/projectile.rs`; inspected 110–717, 733–900; initial `480106a76bd670514ab6a16d0057b880b18510c0d325dcdf3b1e4834777d3312`; reviewed `480106a76bd670514ab6a16d0057b880b18510c0d325dcdf3b1e4834777d3312`; close `8780ed276ff988e6ba8cb1e15e3cdc8d645f73e9207a0b71854286b2130f580c`; drift from reviewed=True.
- `core_enhanced/src/game/battle/core/sim.rs`; inspected 982–1065, 1147–1200, 2255–2620; initial `70c777297c74d7a95aebf8ee7e5cf6024b12c95b6829b3dcfddd9d239c13bd0b`; reviewed `70c777297c74d7a95aebf8ee7e5cf6024b12c95b6829b3dcfddd9d239c13bd0b`; close `e624854ac13e8172b104c54e4626a761c4d017d67b8ffdb4adf3f15e50a31207`; drift from reviewed=True.
- `core_enhanced/src/game/battle/core/types.rs`; inspected 75–176, 326–346; initial `385402e5e49ead95a505f2345211875542ed370aa5558e73472c33de03e42a2d`; reviewed `385402e5e49ead95a505f2345211875542ed370aa5558e73472c33de03e42a2d`; close `385402e5e49ead95a505f2345211875542ed370aa5558e73472c33de03e42a2d`; drift from reviewed=False.
- `core_enhanced/src/game/battle/recording.rs`; inspected 6–45; initial `e3c1afedaf9b5ec61c76d415e2123dbea8a9c6d9213da5393dddfaf0e6beaf2d`; reviewed `e3c1afedaf9b5ec61c76d415e2123dbea8a9c6d9213da5393dddfaf0e6beaf2d`; close `e3c1afedaf9b5ec61c76d415e2123dbea8a9c6d9213da5393dddfaf0e6beaf2d`; drift from reviewed=False.

Known unfinished clone/area-tick/history/usefulness/backend/public-session/CLI breadth is intentionally not repeated as new findings. Root remains sole production writer; only this new /tmp report pair was created.
