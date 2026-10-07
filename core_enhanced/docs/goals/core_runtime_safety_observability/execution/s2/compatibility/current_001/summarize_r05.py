"""Apply only the approved concurrent-peak warning exception to raw evidence."""
import hashlib
import json
from pathlib import Path

ROOT = Path('/mnt/f/work/simulator')
HERE = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/corrections/f01_action_flow_20261004_001')
E = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_safety_observability/execution/s2/compatibility/current_001') / 'evidence'
raw_path = E / 'r05_raw_current_002.json'
raw = json.loads(raw_path.read_text())
receipt = json.loads((Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_safety_observability/evidence') / 's2compat_r05_001.json').read_text())
baseline_path = HERE.parents[1] / 'evidence/P01_LOG_V32_BASELINE_001.json'
baseline = json.loads(baseline_path.read_text())
prior = json.loads((ROOT / 'core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal/evidence/r05_raw_integrated_003.json').read_text())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
original_baseline = json.loads((ROOT / 'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json').read_text())
import copy
original_projection = copy.deepcopy(original_baseline)
new_projection = copy.deepcopy(baseline)
for family, key in [('event_heavy', 'event_log_sha256'), ('concurrent_sessions', 'combined_event_log_sha256')]:
    for a, b in zip(original_projection[family]['raw_samples'], new_projection[family]['raw_samples']):
        a.pop(key); b.pop(key)
assert original_projection == new_projection, 'Only migrated log hashes may differ; preserve every original cap and input.'
migration = json.loads((HERE.parents[1] / 'evidence/LOG_VERSION_MIGRATION_001.json').read_text())
assert migration['migrated_baseline_sha256'] == sha(baseline_path)
assert migration['original_baseline_sha256'] == sha(ROOT / 'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json')

assert raw['schema_version'] == 1 and baseline['schema_version'] == 2
assert raw['allocation_observer_enabled'] and raw['allocation_gate_applicable'] and not raw['latency_gate_applicable']
assert raw['baseline']['receipt_sha256'] == sha(baseline_path)
assert raw['tool_source_sha256'] == sha(ROOT / 'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs')
assert raw['workload_input_sha256'] == prior['workload_input_sha256']
assert raw['tool_source_sha256'] == prior['tool_source_sha256']
assert receipt['source_before']['sha256'] == receipt['source_after']['sha256']
failures = []
summaries = {}
for key in ['event_heavy', 'concurrent_sessions']:
    work = raw[key]
    samples = work['allocation_raw_samples']
    old = baseline[key]['raw_samples']
    assert len(samples) == len(old) == 5
    assert len(work['latency_raw_samples']) == 21
    caps = {
        'operations': max(s['heap_allocation']['allocation_count'] + s['heap_allocation']['reallocation_count'] for s in old),
        'retained': max(s['heap_allocation']['retained_delta_bytes'] for s in old),
        'peak': max(s['heap_allocation']['peak_delta_bytes'] for s in old),
    }
    ops = [s['heap_allocation']['allocation_count'] + s['heap_allocation']['reallocation_count'] for s in samples]
    retained = [s['heap_allocation']['retained_delta_bytes'] for s in samples]
    peaks = [s['heap_allocation']['peak_delta_bytes'] for s in samples]
    assert work['allocation_retained_peak_guard_pass'] == (max(ops) <= caps['operations'] and max(retained) <= caps['retained'] and max(peaks) <= caps['peak'])
    if not work['repeatable']: failures.append(key + ':repeatability')
    if not work['deterministic_baseline_parity']: failures.append(key + ':parity')
    if max(ops) > caps['operations']: failures.append(key + ':allocation_operations')
    if max(retained) > caps['retained']: failures.append(key + ':retained')
    if key == 'event_heavy' and max(peaks) > caps['peak']: failures.append(key + ':peak')
    summaries[key] = {'caps': caps, 'peak_samples': peaks, 'max_peak_bytes': max(peaks), 'operations': ops,
                      'retained_samples': retained, 'original_allocation_guard_pass': work['allocation_retained_peak_guard_pass'],
                      'repeatable': work['repeatable'], 'parity': work['deterministic_baseline_parity'],
                      'latency_gate_applicable': False, 'p95_ns': work['nearest_rank_p95_ns'],
                      'latency_absolute_cap_pass_observation': work['absolute_cap_pass'],
                      'latency_regression_observation': work['statistically_supported_regression']}
concurrent = summaries['concurrent_sessions']
assert concurrent['caps']['peak'] == 9_030_188
excess = max(0, concurrent['max_peak_bytes'] - concurrent['caps']['peak'])
log = (Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_safety_observability/evidence') / 's2compat_r05_001.log').read_text()
if receipt['exit_code'] == 101:
    assert excess > 0 and not failures
    assert log.count("thread 'main'") == 1
    assert 'assertion failed: receipt.concurrent_sessions.allocation_retained_peak_guard_pass' in log
else:
    assert receipt['exit_code'] == 0 and not excess and not failures
summary = {'source_digest': receipt['source_after']['sha256'], 'raw_sha256': sha(raw_path),
           'baseline_sha256': sha(baseline_path), 'raw_exit_code': receipt['exit_code'],
           'classification': 'warning_concurrent_peak_only' if excess else 'pass_original_guards',
           'concurrent_peak_excess_bytes': excess, 'other_hard_guard_failures': failures, **summaries}
with (E / 'R05_SUMMARY_CURRENT_002.json').open('x') as stream:
    json.dump(summary, stream, indent=2); stream.write('\n')
print(json.dumps(summary, indent=2))
