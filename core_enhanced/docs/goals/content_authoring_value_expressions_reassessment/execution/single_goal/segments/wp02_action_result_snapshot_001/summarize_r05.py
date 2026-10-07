"""Preserve the unchanged R05 guard and the observed diagnostic sequence."""

import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
CAP = 9030188
paths = {
    'prior_correction': ROOT / 'corrections/wp02_eep_f01_f03_001/evidence/r05_guard_raw.json',
    'current_inline_parallel': HERE / 'evidence/r05_guard_raw.json',
    'current_inline_serial': HERE / 'evidence/r05_guard_serial_raw.json',
    'entry_control': HERE / 'evidence/r05_control_raw_002.json',
    'current_boxed': HERE / 'evidence/r05_guard_box_raw.json',
}


def details(path):
    data = json.loads(path.read_text())
    result = {'path': str(path), 'tool_source_sha256': data['tool_source_sha256'],
              'workload_input_sha256': data['workload_input_sha256'],
              'allocation_observer_enabled': data['allocation_observer_enabled']}
    for name in ('event_heavy', 'concurrent_sessions'):
        item = data[name]
        peak = max(sample['heap_allocation']['peak_delta_bytes']
                   for sample in item['allocation_raw_samples'])
        result[name] = {'peak_delta_bytes': peak,
                        'allocation_guard_pass': item['allocation_retained_peak_guard_pass'],
                        'latency_cap_pass': item['absolute_cap_pass'],
                        'deterministic_baseline_parity': item['deterministic_baseline_parity'],
                        'repeatable': item['repeatable']}
    return result


observations = {name: details(path) for name, path in paths.items()}
assert len({value['tool_source_sha256'] for value in observations.values()}) == 1
assert len({value['workload_input_sha256'] for value in observations.values()}) == 1
assert all(value['allocation_observer_enabled'] for value in observations.values())
assert observations['current_boxed']['concurrent_sessions']['allocation_guard_pass']
summary = {'original_concurrent_cap_bytes': CAP, 'observations': observations,
           'current_boxed_headroom_bytes': CAP - observations['current_boxed']
           ['concurrent_sessions']['peak_delta_bytes']}
with (HERE / 'evidence/R05_SUMMARY.json').open('x') as stream:
    json.dump(summary, stream, indent=2)
    stream.write('\n')
print(json.dumps({'current_boxed_headroom_bytes': summary['current_boxed_headroom_bytes']},
                 indent=2))
