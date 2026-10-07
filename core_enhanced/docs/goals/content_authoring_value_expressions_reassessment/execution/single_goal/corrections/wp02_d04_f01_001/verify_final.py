"""Bind the selected D04-F01 receipts to the final product and controls."""
import json
from pathlib import Path

from run import identity

HERE = Path(__file__).resolve().parent
EVIDENCE = HERE / 'evidence'
SOURCE = '34ceedbebc84b9396d42f92fa31033fe0b831a90e5325c8c4463e0c88f067145'
selected = {
    'check_all_targets_final_002': 0,
    'focused_d04_f01_final_002': 9,
    'focused_value_expr_final_002': 74,
    'target_locked_success_final_002': 1,
    'area_live_success_final_002': 1,
    'all_targets_native_final_002': 1509,
    'clippy_all_targets_final_002': 0,
    'changed_region_format_final_002': 0,
}
assert identity()['digest'] == SOURCE
for name, passed in selected.items():
    receipt = json.loads((EVIDENCE / f'{name}.json').read_text())
    assert receipt['source_before']['digest'] == SOURCE
    assert receipt['source_after']['digest'] == SOURCE
    assert receipt['exit_code'] == receipt['test_failed'] == 0
    assert receipt['test_passed'] == passed
pc_isolate = json.loads((EVIDENCE / 'pc_isolate_002.json').read_text())
pc_run = json.loads((EVIDENCE / 'pc_36_final_002.json').read_text())
pc_summary = json.loads((EVIDENCE / 'PC_SUMMARY.json').read_text())
assert pc_isolate['current_source']['digest'] == SOURCE
assert pc_isolate['built_source'] == pc_run['source_before'] == pc_run['source_after']
assert pc_run['exit_code'] == 0 and pc_run['test_passed'] == 1
assert pc_summary['distinct_conditions'] == 36
assert pc_summary['successful_conditions'] == 31
assert pc_summary['typed_refusal_conditions'] == 5
assert pc_summary['same_gameplay_oracle_and_typed_refusal']
r05 = json.loads((EVIDENCE / 'R05_SUMMARY.json').read_text())
assert r05['runs']['correction_after_control']['workloads']['concurrent_sessions']['allocation_guard_pass']
assert not r05['runs']['correction_first']['workloads']['concurrent_sessions']['allocation_guard_pass']
assert not r05['runs']['correction_repeat']['workloads']['concurrent_sessions']['allocation_guard_pass']
assert not r05['runs']['predecessor_control']['workloads']['concurrent_sessions']['allocation_guard_pass']
prior = json.loads((EVIDENCE / 'PRIOR_EVIDENCE.json').read_text())
assert prior['review_failure_cases'] == 4
assert prior['predecessor_scalar_control_passed'] == 2
result = {'final_source_digest': SOURCE, 'selected_receipts': selected,
          'pc_conditions': {'successful': 31, 'typed_refusal': 5},
          'r05_original_cap_bytes': r05['original_concurrent_cap_bytes'],
          'r05_last_peak_bytes': r05['runs']['correction_after_control']['workloads']
                                 ['concurrent_sessions']['max_peak_delta_bytes'],
          'r05_failures_preserved': 3,
          'predecessor_reproduction_failures': 4}
with (EVIDENCE / 'FINAL_VALIDATION.json').open('x') as stream:
    json.dump(result, stream, indent=2)
    stream.write('\n')
print(json.dumps(result, indent=2))
