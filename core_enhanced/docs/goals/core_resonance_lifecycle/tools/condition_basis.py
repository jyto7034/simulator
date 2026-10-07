"""Recover exact inherited meanings; do not infer acceptance from test names."""
import json, re
from initialize import ROOT
from run import sha

R06 = ROOT / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment'
SOURCES = [
    R06 / 'CODE_REVIEW_003.ko.md',
    R06 / 'execution/single_goal/continuations/r06_completion_001/COVERAGE.json',
    R06 / 'execution/single_goal/handoffs/R06_R07_CONTINUOUS_IMPLEMENTATION_REQUEST_20260929.ko.md',
    R06 / 'successors/r06_scope_20260927_001/P2G_50_GOAL_SET.ko.md',
]

def inherited_meanings():
    coverage = json.loads(SOURCES[1].read_text())
    integrations = {row['id']: row for row in coverage['integration']}
    result = {}
    for line in SOURCES[0].read_text().splitlines():
        match = re.match(r'^\| V(\d\d)\s+[^|]+\|\s*([^|]+)\|', line)
        if not match:
            continue
        key = 'IVA-CAVER-' + match[1]
        result[key] = {
            'requirement': match[2].strip(),
            'completed_scope_expectation': integrations[key]['expected'],
            'original_acceptances': [
                {'id': row['acceptance'], 'expected': row['expected']}
                for row in coverage['acceptances'] if key in row['iva']],
            'source': str(SOURCES[0].relative_to(ROOT)),
            'locator': 'D09 V' + match[1],
            'source_sha256': sha(SOURCES[0]),
            'completed_scope_source': str(SOURCES[1].relative_to(ROOT)),
            'completed_scope_sha256': sha(SOURCES[1]),
        }
    assert len(result) == 21
    for line in SOURCES[2].read_text().splitlines():
        match = re.match(r'^\| (F\d\d) \| (.+) \|$', line)
        if match:
            result[match[1]] = {'requirement': match[2],
                'source': str(SOURCES[2].relative_to(ROOT)), 'locator': 'section5/' + match[1],
                'source_sha256': sha(SOURCES[2])}
    assert len(result) == 33 and all(row['requirement'] for row in result.values())
    return result

# These supplement the unchanged owner requirements, not replace them.
DELTAS = {
    'A09-01': 'Admission still requires full current charge and a valid exact copy/form; subsequent start checks the accepted credential rather than requiring the already-spent charge again.',
    'A09-02': 'Intentional U05 difference: accepted paused input consumes immediately, while corrosion effects remain at the existing actual start gate. Later Silence or zero charge does not revoke acceptance; actual cancellation still has no refund.',
    'A09-03': 'The admission Cost is the sole employee charge payment. Corrosion substitutes its payload at start without a second payment or ordinary skill damage.',
    'A10-03': 'Finite opportunity/interval semantics stay fixed; unexecuted opportunities keep the employee use until the last release, while independent area ticks own themselves.',
    'A10-04': 'Future Branch/Sequence/Parallel/Repeat control retains use. A final EffectsSettled waiter alone does not. Existing authored result ordering and stale-wake refusals remain.',
    'A10-05': 'Intentional manual target change: legal empty PlayerTile is admissible; accepted area anchor remains fixed after movement/withdrawal. Mandatory direct target loss cancels future control, and manual employee RetargetOnStep is rejected.',
    'A10-06': 'Cancellation separates unreleased employee control from already released carriers; the original release/settled milestone and corrosion start timing remain distinct from use completion.',
    'A11-01': 'Actual beneficial finite modifier records now additionally retain employee use. Removing or replacing a record ends only that record\'s use obligation; base and other sources remain.',
    'A11-02': 'Actual channel/aura termination ends its use obligation; released attack snapshots and exact member/provenance rules retain their prior owners.',
    'A11-03': 'Active channel retains the use gate as well as existing capability locks. Explicit stop remains legal and does not remove unrelated lock sources.',
    'A11-04': 'Silence does not cancel accepted use. Hard CC/interrupt end channel and future control; ordinary already-applied timed buffs remain until their own actual end.',
    'A11-05': 'Hello now retains the employee use through its actual finite window. Existing explicit reinput/Goodbye consumes no second charge; natural expiry remains distinct.',
    'A11-08': 'One-use/expiry overlay consumption also closes that selected use lease at its actual end. Projectile settlement alone never prolongs the original use.',
    'A12-03': 'The original expiry/consumption/source order remains. Employee recharge/recast waits for actual shield completion; tests no longer inject charge through an active use gate.',
    'A12-04': 'Rest still restores HP and never changes shield balance. Its obsolete resonance bonus is removed; explicit recharge remains the shield owner.',
    'A12-09': 'Intentional U05 difference: employee attack release, dealt damage and received damage grant no base charge, including shielded hits. Actual HP-loss gates, absorption and lifesteal remain unchanged.',
    'IVA-CAVER-05': 'Release Snapshot and consumption Live retain their exact moments even after employee use ends at last release.',
    'IVA-CAVER-06': 'Use cancellation cannot revoke already released Snapshot delivery authority. Live target/source availability and owner lease lifetime keep their distinct refusals.',
    'IVA-CAVER-09': 'Stat arithmetic and exact-copy composition are unchanged; only the separate employee resonance capacity/start composition moves to the selected fragment form.',
    'IVA-CAVER-11': 'Attack-speed cadence and other weapon timers are unchanged; the new whole-second charge pulse is a separate queue event, not an attack timer rewrite.',
    'IVA-CAVER-12': 'Resource reads remain side-effect free. Positive resonance mutations use the new current-use gate; resource definition/owner/scope and observation rules remain distinct.',
    'IVA-CAVER-13': 'Completed-prefix/settled-result requirements do not become use-lifetime requirements. A completed employee use never fabricates an EffectsSettled result.',
    'IVA-CAVER-15': 'Intentional current schema boundary: V11 Continue round-trips new inputs; historical V7 bytes/checksum stay untouched. Explicit legacy helpers produce conversion-only V12, not a resumable Continue. Unsupported custom old employee resonance is refused.',
    'IVA-CAVER-16': 'Catalog schema5 and new authored fields deliberately change semantic identity. Canonical ordering, exact-copy fingerprint and unchanged Certified weight rules remain.',
    'IVA-CAVER-18': 'Public successor packs explicitly author use_lifetime and form capacity. Original amount/trace and error oracles execute through the current release reader; absent fields are never defaulted.',
    'IVA-CAVER-20': 'New runtime state and scan work require fresh cost measurements. Report actual cost deltas separately from fixed gameplay/error oracles; original numerical caps and the sole approved concurrent-peak warning remain controlling.',
    'F01': 'Grant UUID/receipt atomicity remains unchanged; newly composed battle charge derives from the granted copy\'s selected form only after equip/deployment.',
    'F02': 'Selecting/awakening/transferring A changes only A\'s actual capacity/form input; B and double-binding refusals remain independent.',
    'F03': 'The same compatibility and atomic transfer resolver now supplies per-form resonance_max; invalid copy/slot inputs still cannot partially alter either owner.',
    'F04': 'Recipes, unlock and no-op semantics stay unchanged; every authored form now additionally requires a positive explicit capacity.',
    'F05': 'Dismantle retains exact-copy confirmation and grant rollback; a stale live skill attribution cannot recreate or select the removed copy.',
    'F06': 'Death clears attempt-local use/pulse while existing exact-copy recovery, unlocked forms and idempotent result ownership remain authoritative.',
    'F07': 'Public base/awakened trials retain original command/amount/trace oracles with explicit max25/start5 trial forms and the original23second wait; official form capacities remain100.',
    'F08': 'Selected-form capacity joins current composition without replacing exact source-copy/lease identities; old released deliveries keep their captured provenance.',
    'F09': 'Fresh-process current Continue is V11 with strict employee base authoring. Copy/form/binding/checksum validation remains atomic; live accepted use is deliberately excluded from durable state.',
    'F10': 'Historical originals are preserved. Explicit V7/V8 conversion strips only the exact old default0/100/1000 and produces checkpoint V12; custom/unknown inputs and all conversion outputs at normal V11 Continue are refused.',
    'F11': 'Readiness now additionally exposes actual use and gain-block reasons; copy/form/cost/disabled-reason queries remain read-only and use the same owner as commands.',
    'F12': 'New form/use/pulse/scan fields participate in clone, extent and work accounting. Existing cardinality/refusal/rollback and full inherited cost guards remain required.',
    'PE-CRSO-01': 'All38 configured defaults remain byte-independent numerically equal to the original defaults. Approved employee gameplay changes are assessed by U05; unchanged workload result/error oracles are still compared exactly.',
}
