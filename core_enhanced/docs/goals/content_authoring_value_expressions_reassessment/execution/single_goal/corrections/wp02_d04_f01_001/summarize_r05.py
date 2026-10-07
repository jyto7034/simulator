"""Preserve every D04-F01 R05 run and compare the exact-source control."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
PRIOR = HERE.parent.parent / 'segments/wp02_d04_resolved_target_001/evidence/r05_guard_raw.json'
EVIDENCE = HERE / 'evidence'
CAP = 9_030_188
RUNS = {
    'frozen_d04': PRIOR,
    'correction_first': EVIDENCE / 'r05_guard_raw.json',
    'correction_repeat': EVIDENCE / 'r05_guard_repeat_raw.json',
    'predecessor_control': EVIDENCE / 'r05_predecessor_control_raw_002.json',
    'correction_after_control': EVIDENCE / 'r05_guard_after_control_raw.json',
}


def details(path):
    data = json.loads(path.read_text())
    assert data['allocation_observer_enabled']
    return {
        'path': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
        'tool_source_sha256': data['tool_source_sha256'],
        'workload_input_sha256': data['workload_input_sha256'],
        'latency_gate_applicable': data['latency_gate_applicable'],
        'allocation_gate_applicable': data['allocation_gate_applicable'],
        'workloads': {
            key: {
                'peak_delta_samples_bytes': [row['heap_allocation']['peak_delta_bytes']
                                             for row in data[key]['allocation_raw_samples']],
                'max_peak_delta_bytes': max(row['heap_allocation']['peak_delta_bytes']
                                            for row in data[key]['allocation_raw_samples']),
                'allocation_operations': [row['heap_allocation']['allocation_count']
                                          + row['heap_allocation']['reallocation_count']
                                          for row in data[key]['allocation_raw_samples']],
                'retained_delta_bytes': [row['heap_allocation']['retained_delta_bytes']
                                         for row in data[key]['allocation_raw_samples']],
                'allocation_guard_pass': data[key]['allocation_retained_peak_guard_pass'],
                'deterministic_baseline_parity': data[key]['deterministic_baseline_parity'],
            }
            for key in ('event_heavy', 'concurrent_sessions')
        },
    }


runs = {name: details(path) for name, path in RUNS.items()}
assert len({run['tool_source_sha256'] for run in runs.values()}) == 1
assert len({run['workload_input_sha256'] for run in runs.values()}) == 1
assert all(run['allocation_gate_applicable'] and not run['latency_gate_applicable']
           for run in runs.values())
assert all(item['deterministic_baseline_parity']
           for run in runs.values() for item in run['workloads'].values())
assert all(run['workloads']['event_heavy']['allocation_guard_pass']
           for run in runs.values())
assert runs['frozen_d04']['workloads']['concurrent_sessions']['allocation_guard_pass']
assert not runs['correction_first']['workloads']['concurrent_sessions']['allocation_guard_pass']
assert not runs['correction_repeat']['workloads']['concurrent_sessions']['allocation_guard_pass']
assert not runs['predecessor_control']['workloads']['concurrent_sessions']['allocation_guard_pass']
assert runs['correction_after_control']['workloads']['concurrent_sessions']['allocation_guard_pass']
current_peak = runs['correction_after_control']['workloads']['concurrent_sessions']['max_peak_delta_bytes']
assert current_peak <= CAP
assert runs['predecessor_control']['workloads']['concurrent_sessions']['max_peak_delta_bytes'] > CAP
for name, expected in [
    ('predecessor_control', '4c63ca018cd39d50e4a896711debaaf7970dc2636841a77eb74457f83313582f'),
    ('correction_first', '34ceedbebc84b9396d42f92fa31033fe0b831a90e5325c8c4463e0c88f067145'),
    ('correction_repeat', '34ceedbebc84b9396d42f92fa31033fe0b831a90e5325c8c4463e0c88f067145'),
    ('correction_after_control', '34ceedbebc84b9396d42f92fa31033fe0b831a90e5325c8c4463e0c88f067145'),
]:
    receipt_name = {
        'predecessor_control': 'r05_predecessor_control_002.json',
        'correction_first': 'r05_guard_final_001.json',
        'correction_repeat': 'r05_guard_repeat_001.json',
        'correction_after_control': 'r05_guard_after_control_001.json',
    }[name]
    receipt = json.loads((EVIDENCE / receipt_name).read_text())
    assert receipt['source_before']['digest'] == receipt['source_after']['digest'] == expected

summary = {
    'original_concurrent_cap_bytes': CAP,
    'runs': runs,
    'correction_after_control_headroom_bytes': CAP - current_peak,
    'interpretation': ('The final correction passes one direct guard run; two preceding runs '
                       'and an exact D04 predecessor control exceed the same narrow peak cap. '
                       'All runs retain deterministic parity and unchanged allocation counts '
                       'and retained deltas. The peak guard varies with concurrent scheduling.'),
}
with (EVIDENCE / 'R05_SUMMARY.json').open('x') as stream:
    json.dump(summary, stream, indent=2)
    stream.write('\n')
print(json.dumps({'latest_peak': current_peak, 'headroom': CAP - current_peak,
                  'predecessor_control_peak': runs['predecessor_control']['workloads']
                  ['concurrent_sessions']['max_peak_delta_bytes']}, indent=2))
