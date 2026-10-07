"""Read existing receipts; do not run workloads or modify frozen evidence."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
EXECUTION = HERE.parent.parent
ROOT = next(p for p in HERE.parents if (p / 'core_enhanced/Cargo.toml').is_file())


def meta(path):
    data = path.read_bytes()
    return {'path': str(path.relative_to(ROOT)), 'bytes': len(data),
            'sha256': hashlib.sha256(data).hexdigest()}


def analyze():
    paths = []
    for path in sorted(EXECUTION.rglob('*.json')):
        relative = path.relative_to(EXECUTION)
        if relative.parts[0] not in ('corrections', 'segments'):
            continue
        if 'target' in relative.parts or 'baseline' in relative.parts:
            continue
        data = json.loads(path.read_text())
        if isinstance(data, dict) and isinstance(data.get('concurrent_sessions'), dict):
            if data['concurrent_sessions'].get('allocation_raw_samples'):
                paths.append(path)
    paths.insert(0, ROOT / 'core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal/evidence/ORDINARY_GUARDS_005/kernel_alloc.json')
    rows = []
    for path in paths:
        data = json.loads(path.read_text())
        samples = data['concurrent_sessions']['allocation_raw_samples']
        observations = [s['heap_allocation'] for s in samples]
        peaks = [h['peak_delta_bytes'] for h in observations]
        rows.append({
            'input': meta(path),
            'tool_sha256': data['tool_source_sha256'],
            'workload_sha256': data['workload_input_sha256'],
            'sample_count': len(samples), 'peaks': peaks, 'max_peak': max(peaks),
            'above_original_peak_cap': sum(p > 9030188 for p in peaks),
            'allocation_counts': sorted({h['allocation_count'] for h in observations}),
            'reallocation_counts': sorted({h['reallocation_count'] for h in observations}),
            'retained_deltas': sorted({h['retained_delta_bytes'] for h in observations}),
            'event_hashes': sorted({s['combined_event_log_sha256'] for s in samples}),
            'guard_reported_pass': data['concurrent_sessions']['allocation_retained_peak_guard_pass'],
        })
    source_paths = [
        'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs',
        'core_enhanced/tools/runtime_architecture_preservation/src/main.rs',
        'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json',
        'core_enhanced/src/game/battle/core/runtime_facts.rs',
        'core_enhanced/src/game/battle/core/queue.rs',
        'core_enhanced/src/game/battle/core/value_expr_runtime.rs',
        'core_enhanced/src/game/battle/runtime_control.rs',
    ]
    manifest = EXECUTION / 'corrections/wp02_d04_f01_001/target/d04_f01_correction_001/MANIFEST.json'
    assert meta(manifest)['sha256'] == 'e4f0d77b1bd3066493d605271605d7a4b6e16317a6d540d303909b076b480104'
    return {'kind': 'existing_receipt_analysis_no_new_measurements',
            'target': meta(manifest), 'original_peak_cap_bytes': 9030188,
            'read_sources': [meta(ROOT / p) for p in source_paths], 'runs': rows}


if __name__ == '__main__':
    print(json.dumps(analyze(), ensure_ascii=False, indent=2))
