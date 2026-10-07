# R03 mandatory receipt — bounded read-only review 003

This is AUTORUN §8 source/design assistance, not a completion audit. Root is the sole implementation/artifact writer. This helper ran no source, tests, builds or Git operations and changed no production/tests/docs files. Only this new report and its JSON companion are written; previous reports remain preserved.

No concrete encoder/control source defect was established in the requested scope. One wording/test precision point was reported: result_bytes_peak is an attempt-wide maximum, not necessarily the current receipt length. Root clarified PLAN009 and added a source test during the review. The missing session caller remains a follow-up owner, not a defect in this private encoding unit.

## Exact successful data/accounting flow

In output.rs 70–90, encode_receipt requires an existing controlled attempt, enters Receipt phase, and rejects zero/>64MiB total caps or a committed count above the cap. These inputs are scalar caller-owned basis; the function receives no mutable old receipt/aggregate reference.

usage_prefix(default) constructs the actual reserved JSON prefix. Writer receives remaining = max_total_bytes − committed_bytes and writes the prefix before the payload and final closing brace (91–100). Writer::write checks remaining cumulative bytes, then admits ResultBytes at the prospective current buffer length, then charges the bytes, and only then appends (31–49). Therefore the initial usage placeholder is already part of actual mandatory output admission; there is no invisible header added after the byte check.

After payload and closing JSON encoding, the function charges 2 × reserved prefix length for the replacement preparation/copy. Only then does it call usage_snapshot (101–108), build the real prefix and require its length to match (109–115). Replacing exactly that prefix changes no admitted byte count, and String::from_utf8 takes ownership of the completed Vec. The returned total_bytes is committed_bytes plus the complete document length (116–123).

For a normal serializable DTO that does not make its own control calls, let W be work before this function's writes, H the reserved prefix length and L the final document length. The resulting work is W + L + 2H. Every payload, punctuation, padding and closing byte is in L. Both prefix constructions are bounded by the current fixed scalar usage schema; the replacement is priced before the snapshot. There is no iterative self-referential counter serialization.

## Fixed-width JSON and snapshot semantics

ExecutionUsage is the current fixed nine-field object of u64/usize counters (control.rs 87–98). usage_prefix verifies an object and unsigned numeric fields, encodes keys with serde_json, and writes each numeric value into a minimum width of 20 characters (output.rs 126–150). On the current 32/64-bit Rust targets, all these counters fit within u64's maximum 20 decimal digits. Padding follows the decimal digits, so it is legal JSON whitespace and preserves JSON numeric types. Field names/schema remain identical between the default and actual counter values. The explicit length equality check prevents a future incompatible header shape from being copied into the reservation.

usage_snapshot (control.rs 280–284) returns a cloned owned scalar ExecutionUsage. It does not expose mutable Attempt state and does not charge or mutate counters. The preceding charge polls cancellation/sticky work failure before the snapshot. From the snapshot to successful return there is no further charged/admitted operation in encode_receipt. PhaseGuard restoration changes phase only. control::run's successful acceptance poll also leaves counters unchanged (172–224). Thus receipt usage equals the final successful scope usage **provided the caller does no later charged/admitted work**.

A cancellation arriving after the last charge can still allow the private helper to return a PreparedReceipt candidate; outer control::run acceptance must reject it before session publication. This behavior is explicitly tested in source and is the requested caller responsibility. PreparedReceipt is not an accepted session or a commit receipt by itself.

## RR03-N01 — peak meaning, clarified during review

control::admit sets peak = max(old_peak, requested), shared across the attempt (366–425). Writer passes its current buffer length plus the new chunk. Consequently the correct successful formula is:

    usage.result_bytes_peak = max(previous_attempt_result_peak, receipt.document.len())

It equals this receipt's length when no previous output in the same attempt was larger. It is neither cumulative receipt bytes nor allocator/Vec capacity. A failed write may have advanced an admitted peak before a work failure, but that failure is sticky and no successful outer receipt can certify that attempt.

The original PLAN009 wording said the peak was the completed receipt length. Root corrected it to the attempt-wide maximum (PLAN 270–271) and added output.rs 241–257: encode a larger earlier result, then a small receipt in the same scope, expect the earlier length as the peak while receipt usage still equals final scope usage. This was a design/oracle precision point, not a code defect. This helper read the new test but did not execute it.

## Boundaries, overflow and failures

The cumulative check maintains writer.bytes.len() <= receipt_remaining by induction: it starts at zero, each append is admitted against remaining − current length, and no failed write appends. The subtraction cannot underflow along this private Writer path. Validated committed_bytes <= cap <=64MiB plus admitted L <= cap−committed_bytes makes the final committed_bytes + L safe on supported targets, including when the cap equals the already committed total (the first nonempty prefix write then fails). Equality is admitted; cap+1 is rejected.

Per-result admission and cumulative receipt accounting are independent. ResultBytes is the largest individual mandatory buffer admitted in the attempt; total_bytes sums the prior committed receipt byte scalar and the new receipt. Vector capacity, other owned state, possible later outer framing and retained payload inputs are not certified by either number.

| Failure | Direct result / acceptance behavior |
| --- | --- |
| No active attempt | authoring_scope_required; no writer or PreparedReceipt |
| Invalid cumulative cap/basis | authoring_receipt_limits_invalid |
| Prefix/payload/closing exceeds remaining bytes | Writer latches receipt_limit_failed and returns authoring_receipt_limit; buffer is local |
| ResultBytes or work/cancellation interruption | Writer/encode return an encoding-layer error locally; control keeps the real sticky ExecutionFailure, which outer run must propagate first |
| Payload serializer returns Err | authoring_result_encoding; local prefix/payload buffer drops, no PreparedReceipt |
| Usage shape/type or reserved-length mismatch | authoring_receipt_header_invalid before prepared return |
| Cancellation after prepared return but before scope acceptance | outer control::run rejects; caller must discard candidate |

Writer::error distinguishes its cumulative-cap failure from ordinary encoding errors (output.rs 19–29). The helper does not mutate committed byte count on any path. usage_prefix's String formatting expect is the infallible std::fmt::Write-to-String operation, not a fake ID allocator or gameplay fallback.

The writer does not impose a separate CPU deadline on arbitrary Serialize implementations; this private function must receive the session's bounded owned mandatory DTO. The current fixed header is small and independent of arbitrary content. Byte ceilings and rewrite charge are logical engineering bounds, not measured memory/latency claims.

## Session obligations, kept separate

1. Call encode_receipt only after all mandatory candidate work/admission is finished. Propagate both its inner Result and the outer control::run Result; a sticky control failure must not be masked by a local encoding error. After successful final poll, only infallible aggregate commit may publish the candidate.
2. Pass the actual committed receipt-byte basis and final mandatory provenance payload. On any mandatory failure, retain prior state/token/RNG/input/results/quota/receipts. This function only returns candidate bytes and a candidate scalar; full payload schema and application acceptance are still unconnected.
3. Keep optional trace out of this Writer, its mandatory payload/accounting and binding sources. Optional overflow is gap-only and must not affect work, mandatory admission, commit or RNG.
4. Run required focused/integration checks and record actual cost at the owning validation stage. These reviewed source test definitions are not this helper's test executions.

## Read evidence and source drift

All production output.rs code 1–177 and source tests through the final 345 lines were read. Existing test definitions cover JSON parse/payload preservation, final usage equality, max-width u64/usize counters, inclusive result/work/cumulative caps, failed append/retry basis, invalid limits, cancellation after prepared output, real serializer error and missing scope. The newly added prior-larger-output test is also read. No test receipt is claimed here.

Preliminary live source reads were followed by one-read bytes+SHA256 snapshots kept in functions memory. Later output.rs changed only by the added prior-larger-output test; that exact delta was read and the source snapshot updated. PLAN009 changed to clarify the same peak meaning. control.rs remained unchanged. At 2026-09-10T07:58:52.895872+00:00, the reported final file hashes were:

| Source | Bytes | SHA256 | Reviewed spans |
| --- | ---: | --- | --- |
| core_enhanced/src/authoring/output.rs | 12499 | e1797e0fabf632e1300149b52b45bc22a9dbd040345e5212c4765e9cf82c34bf | 1–345 |
| core_enhanced/src/authoring/control.rs | 16562 | fe69b34d3303d4d465d94c999dc750fa8eaf33a6f34982957f3c5a93ff976bc5 | 1–110, 159–286, 338–425 |
| core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/PLAN.md | 28876 | e74128f0fe05f2d4770538dad27e7c28e75e4bee520c41ac6302dfaaad29c62e | 265–275 |

The initial output.rs snapshot was 5c45ebf80c728033142c154e847b4ee8d8292201a07e616c2d7f5e122f9a0f01; final is the table's e1797e0f…. Initial PLAN hash was f9ba43cc441a53aac338c7463eb19601b989ce4866bef7ac081f38640125f017; final is e74128f0…. Individual file reads are not an atomic repository snapshot. Later root changes need a fresh comparison. No full-session, API/CLI or R03 completion finding is made.
