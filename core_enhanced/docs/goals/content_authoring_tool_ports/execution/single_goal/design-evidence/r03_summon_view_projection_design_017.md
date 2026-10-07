# R03 summon view projection design assistance 017

Status: bounded AUTORUN §8 source/design assistance, not implementation or a completion audit. No repository/source/docs/Git mutation or build/test/runtime execution occurred. Only this new report pair is written. Root's INPUT058 work/testing is context, not my execution.

The selected design below incorporates root's explicit implementation choice: **all-reference seven-field projection; private visit_owned_shape/into_owned; one full Vec meter; active-only two-pass control and stable insertion sort.** It does not prescribe a broader projection framework.

## Current source facts and exact projection

In core_enhanced/src/game/battle/core/boss_runtime.rs:201–209, BossSummonTaskView owns six fixed fields and one ActiveBossSummonTask. Current summon_task_views (:1493–1525) copies selected tasks before knowing the complete retained Vec shape, then sorts by request_id. summon_request_is_active (:1528–1553) already performs borrowed identity checks and should remain independent.

Every projection field can borrow runtime-owned storage for the same lifetime 'a:
- actor_runtime_key: &'a usize; source &actor.actor_slot_index.
- summoner_unit_id: &'a UnitInstanceId; source actor.unit_id.as_ref()?.
- behavior_activation_id: &'a u64; source actor.behavior_activation_id.as_ref()?.
- pattern_activation_id: &'a u64; source &pattern.activation_id.
- state_activation_id: &'a u64; source &pattern.state.activation_id.
- task_activation_id: &'a u64; source &task.activation_id.
- task: &'a ActiveBossSummonTask; source summon from match &task.kind.

Use actor.unit_id.as_ref() and actor.behavior_activation_id.as_ref(), not references to locals obtained by copying Some values. Actor, pattern, task and summon are all borrowed below &self; no projection reference needs a temporary or an Arc clone. UnitInstanceId is Copy (ids.rs:7–9). The first six into_owned clones are scalar copies; task is the actual deep owner.

ActiveBossSummonTask (:174–186) owns request/definition/batch IDs, placement, occupancy, optional route/lifetime/removal signal and fixed data. Its existing native visitor (:273–285) visits all children; task contains no Arc definition graph. Placement can own a zone String, occupancy can own a Vec and removal signal owns an ID (data/boss_runtime_data.rs:230–281,804–813). Reuse that visitor without flattening its field list here.

## Small private macro and compile guards

Proposed shape sketch (not compiled/executed by this helper; imports/qualification belong to root):
```rust
macro_rules! summon_view_projection {
    ($($field:ident: $ty:ty),+ $(,)?) => {
        struct BossSummonTaskProjection<'a> {
            $($field: &'a $ty,)+
        }

        impl<'a> BossSummonTaskProjection<'a> {
            fn visit_owned_shape(
                &self, meter: &mut CloneMeter,
            ) -> Result<(), GameError> {
                let Self { $($field,)+ } = self;
                meter.scan_storage(Some(1))?;
                $(<$ty as CloneShape>::visit_clone_shape(*$field, meter)?;)+
                Ok(())
            }

            fn into_owned(self) -> BossSummonTaskView {
                let Self { $($field,)+ } = self;
                BossSummonTaskView {
                    $($field: <$ty as Clone>::clone($field),)+
                }
            }
        }
    };
}

summon_view_projection! {
    actor_runtime_key: usize,
    summoner_unit_id: UnitInstanceId,
    behavior_activation_id: u64,
    pattern_activation_id: u64,
    state_activation_id: u64,
    task_activation_id: u64,
    task: ActiveBossSummonTask,
}
```

This macro does **not** implement CloneShape for the projection; visiting references as a new owned record would conflate pointer storage with the future output. The explicit method visits owned field types and includes exactly one future BossSummonTaskView root slot.

The producer must construct BossSummonTaskProjection with all seven named fields and no default/update syntax. Generated destructuring also has no '..'; generated BossSummonTaskView literal is complete. Therefore projection fields missing from visitor/materialization, added output fields, renamed fields and incompatible type drift fail compilation. Concrete UFCS selects the owned field's Clone, avoiding accidentally cloning an extra reference. Keep existing owned BossSummonTaskView::CloneShape (:287–295) independent as the oracle.

Compile guards cannot detect a same-type semantic miswire such as supplying pattern_activation_id as state_activation_id. The exact source map above and distinct-ID output checks remain necessary. This is the material limit of the field-list guarantee.

## One traversal definition, two active passes

Prefer one allocation-free, fallible nested traversal callback:

```rust
fn for_each_summon_projection<'a>(
    &'a self,
    mut scan: impl FnMut() -> Result<(), GameError>,
    mut selected: impl FnMut(BossSummonTaskProjection<'a>)
        -> Result<(), GameError>,
) -> Result<(), GameError>
```

Call scan before each actor inspection and before each task inspection. Preserve the current eligibility/order exactly: actor unit + behavior activation + control must exist; control must contain a pattern; iterate its state.tasks in native order; select Summon kind with task.result absent. Do not filter pending-map duplicates here: the current consumer applies that filter after obtaining the whole sorted snapshot. No Vec/Box/collect or source clone is needed in traversal. The callback form avoids duplicating eligibility or introducing a complicated fallible filter-map iterator.

**Active first pass:**

1. Poll entry; create one CloneMeter and count=0. meter.scan_storage(Some(1)) accounts for the future Vec owner.
2. scan charges 2 units per visited actor/task, reserving both measurement and later materialization scans.
3. For each selected projection, meter.scan_storage(Some(1)) adds its Vec element slot, then projection.visit_owned_shape(&mut meter) visits the full retained owner. Increment count only after successful admission.
4. Call meter.finish exactly once after traversal. No into_owned, allocation, Vec reserve or native field clone occurs earlier.

The accumulated storage is **1 + N + sum(each owned-view shape)**, matching Vec<BossSummonTaskView>::CloneShape (authoring/clone_shape.rs:72–80). Incremental element-slot admission is valid without a separate count-only scan because pass one allocates no output. Adding one full aggregate Vec shape on top of these incremental slots would double-charge storage. Using capture_shape per result would finish separate meters and lose the aggregate retained-Vec bound (clone_shape.rs:231–260).

For count arithmetic, each increment follows a successful cumulative slot admission under an active scope whose validated max_clone_slots is at most 2^20 (control.rs:103–107); N cannot reach usize overflow. Preserve that ordering/proof, or explicitly route checked arithmetic failure through existing control admission. Avoid a saturating counter that might silently shrink allocation. An unchecked count before admission loses this proof.

**Active second pass:** after finish succeeds, poll before Vec::with_capacity(count). Invoke the same traversal with scan performing poll only; selected pushes projection.into_owned(). The runtime stays immutably borrowed across both passes and eligibility uses ordinary native fields. Cancellation may end the second pass early; the partial local Vec drops and shared usage is retained. Do not clear or remove source tasks. Optionally verify emitted count equals preflight count before return as an invariant check, not an authority fallback.

**No scope:** keep one traversal, Vec::new/push and existing native stable sort. scan is a no-op. Do not run projection measurement or the extra pass. The method becomes Result<Vec<BossSummonTaskView>, GameError> internally, with unchanged successful ordinary values/order.

## Sorting is separate work

Root selected active-scope stable insertion sort. Before every neighboring request-ID comparison, charge min(left.len(), right.len()) + 1; this covers the compared byte prefix plus length/tie termination in the chosen work model. Return Err immediately on charge failure. Swap only when left > right; equal keys stay in their original relative order. Neighbor swaps move existing owners and create no native clone or scratch Vec.

This has the same lexical/stable result as the existing stable sort, but is a distinct algorithm with potentially quadratic comparison count. Fuel limits its executed comparisons; no guessed native-sort multiplier or unmeasured performance promise is needed. Comparison work remains separate from CloneMeter's owner-copy accounting. Poll once after sorting before returning the owned Vec. Cancellation/fuel failure drops that Vec.

## Caller and oracle

summon_runtime.rs:152–174 capture_boss_summon_requests already returns Result. Change its optional call to transpose()? and default only for absent boss runtime, then retain its existing insertion order and ownership moves. The pending snapshot is retained independently of later boss state changes (same file:41–53). The existing borrowed-active test at :1286–1325 must adapt to Result; it intentionally confirms that descriptive summoner/count fields are not part of authority matching. Do not expand that predicate.

Independent owned-output oracle:

- Build an ordinary reference Vec outside control, then measure it through the existing Vec and BossSummonTaskView native CloneShape implementations with one finish. Separately run projected preflight. clone_slots_peak must match exactly.
- For shape-only work, exclude traversal and sorting work. Successful preflight work equals the owned oracle's work plus 2×(visited actors + visited tasks). Materialization polls add no work; active sorting adds the actual sum of its per-comparison charges.
- Use multiple selected views with long request/zone/route/signal strings and nonempty occupancy Vecs. Choose a budget sufficient for each view individually but insufficient for their aggregate; failure must occur before any into_owned invocation. Exact aggregate storage should admit; one less should reject.
- Include zero selected tasks with many ineligible actors/tasks to prove scans are charged. Validate all seven output fields using distinct activation IDs and equal request IDs to exercise stable ordering. Compare owned-view child shapes without flattening them into a second handwritten meter.
- Cancellation before allocation, during second-pass scans, and before a sort comparison must leave source runtime/pending state unchanged; partial local output must not escape. These are proposals, not executed results.

## Source currentness and scope limits

Reads were rg discovery plus Path.read_bytes; numbered spans and full SHA256 were derived from identical bytes. Six paths/eight reads are recorded in JSON. At close rehash 2026-09-10T10:51:07.754452+00:00, all hashes still matched: **no drift observed before root's forthcoming projection edit**. That is not a review of the future implementation.

This design covers retained view copying and the selected local scan/sort work. It does not cover later pending-map key/trigger generation, all boss allocations, fresh Session final acceptance or generalized safety. Optional trace remains gap-only and cannot influence these mandatory gameplay/control decisions.

- core_enhanced/src/game/battle/core/boss_runtime.rs: 10c7899540c9efe2bc133994404de179fd9fad4508b558bea5d049188cfaed57
- core_enhanced/src/authoring/clone_shape.rs: 6e56d3990b14357c98663ace941fc44b6cde7c41667829b4dddec78e71adbc6b
- core_enhanced/src/authoring/control.rs: 8230d912809e601520f9c2acd7c4dadf0f67a186634309617bd8fa34b4173841
- core_enhanced/src/game/battle/core/summon_runtime.rs: 40bc34b9d1f0616c966a7b19bcb958356e91eec46ad91ed0600804d0c4585c2f
- core_enhanced/src/game/battle/ids.rs: 063075a0f4ac754132949232bd9db555277b4828c43aaecc9a57fd7561069ac9
- core_enhanced/src/game/data/boss_runtime_data.rs: 3df26fd5e0950e4bb1fd76ae0ba766871717fe8adeb5c37466301a2aac88ef8d
