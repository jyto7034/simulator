"""Bind this checkpoint's actual fresh validations and cumulative source delta."""
import hashlib
import json
from pathlib import Path
from run import identity
ROOT = Path('/mnt/f/work/simulator')
HERE = Path(__file__).resolve().parent
E = HERE / 'evidence'
sha = lambda data: hashlib.sha256(data).hexdigest()
source = identity()
names = ['f02_final_001', 'check_final_002', 'authoring_final_001', 'value_expr_final_001', 'all_targets_final_001',
         'clippy_final_001', 'format_final_001', 'pc_build_final_001', 'r05_build_final_001', 'pc_final_001']
results = {}
for name in names:
    receipt = json.loads((E / (name + '.json')).read_text())
    assert receipt['exit_code'] == 0
    assert receipt['source_before']['digest'] == receipt['source_after']['digest'] == source['digest']
    assert sha((E / (name + '.log')).read_bytes()) == receipt['log_sha256']
    if name in ['f02_final_001','authoring_final_001','value_expr_final_001','all_targets_final_001','pc_final_001']:
        assert receipt['test_passed'] > 0 and receipt['test_failed'] == 0
    results[name] = {k: receipt[k] for k in ['exit_code','test_passed','test_failed','elapsed_seconds','log_sha256']}
pc = json.loads((E / 'PC36_COMPARISON.json').read_text())
assert pc['same_gameplay_oracle_and_typed_refusal'] and pc['distinct_conditions'] == 36
r05 = json.loads((E / 'R05_SUMMARY.json').read_text())
assert not r05['other_hard_guard_failures'] and r05['source_digest'] == source['digest']
original_path = HERE.parents[1] / 'segments/wp02_d07_runtime_amount_001/target/d07_runtime_amount_001/MANIFEST.json'
original = json.loads(original_path.read_text())
old = {row['path']: row['after'] for row in original['source']['files'] if row['after'] is not None}
now = {name: {'bytes': size, 'sha256': digest} for name, size, digest in source['files']}
delta = {'original_manifest': str(original_path.relative_to(ROOT)), 'sha256': sha(original_path.read_bytes()),
         'modified': [name for name in old if name in now and old[name] != now[name]],
         'added': sorted(now.keys() - old.keys()), 'deleted': sorted(old.keys() - now.keys())}
assert len(delta['modified']) == 4 and delta['added'] == ['core_enhanced/tests/author_content_cli_arguments.rs'] and not delta['deleted']
summary = {'state':'F02_C02_self_validation_complete_stage_review_pending', 'source_file_count':source['file_count'],
           'source_digest':source['digest'], 'fresh_validations':names, 'results':results, 'r05_policy_classification':r05['classification'],
           'r05_raw_exit_code':r05['raw_exit_code'], 'initial_d07_b_cumulative_delta':delta,
           'f01_predecessor_review_pass':True, 'f02_stage_review_pass':False, 'f03_complete':False, 'd07_b_complete':False,
           'runtime_regression_reused_instead_of_rerun':False,
           'claims_limit':'C02 transaction retention only; dynamic query, terminal, lifetime, cancellation and other C03-C15 obligations remain open.'}
with (E / 'FINAL_VALIDATION.json').open('x') as stream:
    json.dump(summary, stream, indent=2);stream.write('\n')
print(json.dumps({k:summary[k] for k in ['state','source_file_count','source_digest','r05_policy_classification']},indent=2))
