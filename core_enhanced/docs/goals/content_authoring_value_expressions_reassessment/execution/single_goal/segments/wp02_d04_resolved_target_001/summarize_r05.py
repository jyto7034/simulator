"""Compare the D04 R05 allocation run to the unchanged previous checkpoint."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
PRIOR = HERE.parent / 'wp02_action_result_snapshot_001/evidence/r05_guard_box_raw.json'
CURRENT = HERE / 'evidence/r05_guard_raw.json'
CAP = 9_030_188


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
                'peak_delta_bytes': max(row['heap_allocation']['peak_delta_bytes']
                                        for row in data[key]['allocation_raw_samples']),
                'allocation_guard_pass': data[key]['allocation_retained_peak_guard_pass'],
                'deterministic_baseline_parity': data[key]['deterministic_baseline_parity'],
            }
            for key in ('event_heavy', 'concurrent_sessions')
        },
    }


prior, current = details(PRIOR), details(CURRENT)
assert prior['tool_source_sha256'] == current['tool_source_sha256']
assert prior['workload_input_sha256'] == current['workload_input_sha256']
assert current['allocation_gate_applicable'] and not current['latency_gate_applicable']
assert all(item['allocation_guard_pass'] and item['deterministic_baseline_parity']
           for item in current['workloads'].values())
peak = current['workloads']['concurrent_sessions']['peak_delta_bytes']
assert peak <= CAP
summary = {
    'original_concurrent_cap_bytes': CAP, 'prior': prior, 'current': current,
    'current_concurrent_headroom_bytes': CAP - peak,
}
with (HERE / 'evidence/R05_SUMMARY.json').open('x') as stream:
    json.dump(summary, stream, indent=2)
    stream.write('\n')
print(json.dumps(summary, indent=2))
