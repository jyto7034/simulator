"""Compare every non-timing field after explicit U05 state accounting.

This does not assert unchanged cost, or replace numeric performance guards.
All raw costs remain in the inputs and the per-row delta report. The existing
PC gameplay comparator and R05/public numeric ceilings are separate gates.
"""
import copy
import hashlib
import json
import re
from pathlib import Path

ROOT = Path('/mnt/f/work/simulator')
C = Path(__file__).resolve().parent
BEFORE = ROOT / 'core_enhanced/docs/goals/core_runtime_safety_observability/execution/s2/compatibility/current_001/evidence/pc_raw_current_002.json'
AFTER = C / 'evidence/pc_raw_current_003.json'
OUTPUT = C / 'evidence/PC_IMMEDIATE_ACCOUNTED_U05.json'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def without_time(value):
    value = copy.deepcopy(value)
    for row in value['rows']:
        for key in ('measurement', 'setup'):
            if key in row:
                del row[key]['elapsed_ns']
    return value


before, after = [without_time(json.loads(p.read_text())) for p in (BEFORE, AFTER)]
assert before['schema'] == after['schema'] == 1
assert before['seed'] == after['seed'] == 17
assert len(before['rows']) == len(after['rows']) == 576
normalized = copy.deepcopy(after)
costs = []
for index, (old, raw, new) in enumerate(zip(before['rows'], after['rows'], normalized['rows'])):
    family = old['family']
    assert family == new['family']
    changes = []

    def subtract(container, key, delta, label):
        actual = container[key]
        assert isinstance(actual, int) and actual >= delta, (index, label)
        container[key] -= delta
        changes.append({'field': label, 'before': actual - delta, 'after': actual, 'delta': delta})

    if family in ('literal_battle_public_step', 'damage_burst_private_transaction'):
        # Two new absent Option fields: ResonanceUse 48 + timestamp 16 bytes
        # on this measured x86_64 toolchain; fact_slot_extent accounts both.
        units = old['enemies'] + old['allies']
        assert units == old['units'] and old['allies'] == 12
        extent = 64 * units
        # RuntimeFactSource adds one primitive clone-shape visit. At each
        # 100ms step: two persistent source visits, plus two visits for each
        # of twelve pending attacks on alternate (release) steps.
        if family == 'literal_battle_public_step':
            assert old['chunk_ms'] in (100, 1000)
            slots = [2, 26] * 5 if old['chunk_ms'] == 100 else [140]
        else:
            assert old['chunk_ms'] == 0 and old['burst_sources'] in (1, 12)
            slots = [1]
        assert len(new['native_usage']) == len(slots)
        for step, (usage, slot_delta) in enumerate(zip(new['native_usage'], slots)):
            assert len(usage) == 3
            for field, delta, name in ((0, 2 * slot_delta, 'work'), (1, slot_delta, 'clone_slots'), (2, extent, 'retained_admission')):
                subtract(usage, field, delta, f'native_usage[{step}].{name}')
        subtract(new, 'retained_extent_at_end', extent, 'retained_extent_at_end')
        # HashMap grows through power-of-two buckets at 7/8 load. Setup
        # allocates 4+8+...+capacity buckets; each transaction clones capacity.
        capacity = 4
        while capacity * 7 // 8 < units:
            capacity *= 2
        for stage, delta in (('setup', 64 * (2 * capacity - 4)), ('measurement', 64 * capacity * len(slots))):
            if new[stage]['allocation_observer']:
                subtract(new[stage], 'requested_bytes_sum', delta, stage + '.requested_bytes_sum')
            else:
                assert new[stage]['requested_bytes_sum'] is None
    elif family == 'live_adapter_repetition_synthetic':
        # The adapter uses the same two-unit core_with_units fixture.
        assert len(new['native_usage']) == 3
        subtract(new['native_usage'], 2, 128, 'native_usage.retained_admission')
        if new['error'] is not None:
            pattern = r'requested: Some\((\d+)\)'
            found = re.findall(pattern, new['error'])
            assert len(found) == 1
            current = int(found[0])
            new['error'] = re.sub(pattern, f'requested: Some({current - 128})', new['error'])
            changes.append({'field': 'error.requested', 'before': current - 128, 'after': current, 'delta': 128})
    else:
        assert family == 'kernel_resolved_aggregate_synthetic'
    # Compare every field, including request counts, full error text, hashes,
    # expression usage, limits, facts, queue, HP and all measurement metadata.
    assert old == new, (index, family, {k: (old.get(k), new.get(k)) for k in old.keys() | new.keys() if old.get(k) != new.get(k)})
    costs.append({'row': index, 'family': family, 'warmup': old['warmup'], 'sample': old['sample'], 'allocation_observer': old['measurement']['allocation_observer'], 'changes': changes})
assert before == normalized
sources = ['core_enhanced/src/game/battle/core/types.rs', 'core_enhanced/src/game/battle/runtime_facts.rs', 'core_enhanced/src/authoring/clone_shape.rs']
result = {
    'passed': True, 'rows': 576, 'before': {'path': str(BEFORE.relative_to(ROOT)), 'sha256': sha(BEFORE)},
    'after': {'path': str(AFTER.relative_to(ROOT)), 'sha256': sha(AFTER)},
    'claim': 'All non-timing fields equal after ONLY the explicit per-row state-size/source-traversal adjustments below; actual costs are not unchanged.',
    'excluded': ['setup.elapsed_ns', 'measurement.elapsed_ns'],
    'unchanged_gates': 'Original compare_pc_v32.py gameplay/error-code oracle, public performance/allocator guards and original R05 numeric ceilings are separately required. No cap or exception changed.',
    'source_basis': {p: sha(ROOT / p) for p in sources},
    'per_row_cost_changes': costs,
}
with OUTPUT.open('x') as stream:
    json.dump(result, stream, indent=2)
    stream.write('\n')
print('PASS 576 rows: exact non-timing equality after declared U05 cost deltas; unchanged-cost parity is not claimed')
