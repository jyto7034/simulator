"""Bind untouched predecessor inputs and explicit output-label harness deltas."""
import json
import sys
import tarfile
from run import ROOT, HERE, identity, sha
C = HERE / 'execution/single_goal/compatibility/current_001'
label = sys.argv[1] if len(sys.argv) > 1 else '002'
assert label.isdigit() and len(label)==3
from finalization_paths import paths
_, final_names = paths(sys.argv[2] if len(sys.argv)>2 else '001')
intake = json.loads((C / 'INTAKE_U07.json').read_text())
validation = C / f'input_validation_{label}/SUMMARY.json'
v = json.loads(validation.read_text())
assert v['passed'] and v['predecessor_inputs_unchanged']
delta_label = sys.argv[3] if len(sys.argv)>3 else label
assert delta_label.isdigit()
delta = C / f'evidence/harness_label_delta_{delta_label}'
current_harnesses = {r['path']: r for r in json.loads((delta / 'AFTER.json').read_text())['files']}
allowed = {'default_parity.py', 'public_trace_probe.py', 'public_cli.py', 'summarize_r05_current.py', 'public_probe.py'}
files = []
for row in intake['unchanged_input_files']:
    assert sha(ROOT / row['source']) == row['sha256'] == sha(ROOT / row['copy']), row['copy']
    files.append(dict(row, source_sha256=row['sha256'], copy_sha256=row['sha256'], adaptation='none; exact immediate U06 bytes'))
with tarfile.open(delta / 'BEFORE.tar.gz') as archive:
    import hashlib
    for row in intake['scripts']:
        assert sha(ROOT / row['source']) == row['source_sha256'], row['source']
        original = archive.extractfile(row['copy']).read()
        assert hashlib.sha256(original).hexdigest() == row['copy_sha256'], row['copy']
        bound = current_harnesses[row['copy']]
        assert bound['before_sha256'] == row['copy_sha256'] and bound['after_sha256'] == sha(ROOT / row['copy'])
        assert not bound['changed'] or (ROOT / row['copy']).name in allowed
        if (ROOT / row['copy']).name == 'public_probe.py' and bound['changed']:
            anchor = b"assert label in ('before', 'after', 'after_002')"
            assert original.count(anchor) == 1
            expected = original.replace(b'import sys\n', b'import re\nimport sys\n', 1).replace(anchor, b"assert re.fullmatch(r'(?:before|after|after_[0-9]{3})', label)")
            assert expected == (ROOT / row['copy']).read_bytes(), 'Only validated three-digit output-label suffixes are allowed; oracle bytes remain exact'
        files.append(dict(row, current_copy_sha256=bound['after_sha256'], adaptation='U06 owner-path adaptation remains; fresh output-label parameterization and matching trace input root only' if bound['changed'] else 'U06 goal owner path to U07 goal owner path only; exact HARNESS_OWNER.diff preserved'))
prior = ROOT / intake['historical_migrations']
result = {'source_sha256': identity()['sha256'], 'supersedes': f'ADAPTATION_FINAL_{int(label)-1:03d}.json' if int(label) > 2 and (C / f'ADAPTATION_FINAL_{int(label)-1:03d}.json').exists() else 'ADAPTATION_FINAL_002.json' if label != '002' else 'ADAPTATION_FINAL.json', 'files': files,
          'immediate_input_count': len(intake['unchanged_input_files']), 'harness_count': len(intake['scripts']),
          'input_validation': str(validation.relative_to(ROOT)), 'input_validation_sha256': sha(validation),
          'predecessor_adaptation': {'path': str(prior.relative_to(ROOT)), 'sha256': sha(prior), 'claim': 'Earlier documented U04/U05/U06 fixture migrations retained exactly; none added for U07.'},
          'harness_owner_diff_sha256': sha(C / 'HARNESS_OWNER.diff'),
          'harness_label_delta_revision': delta_label,
          'harness_label_delta': {p.name: sha(p) for p in sorted(delta.iterdir()) if p.is_file()},
          'claim': 'All 448 immediate predecessor input bytes and numeric/gameplay expectations are exact. The fourteen harnesses are bound to original and actual current hashes; explicit changed harnesses have fresh-output labels/provenance proof with matching trace input selection. No response, numeric cap or expected gameplay normalization. U07 object packs and strict negative cases are separate.'}
execution_binding_path = delta / 'EXECUTED_BINDINGS.json'
if execution_binding_path.exists():
    execution_bindings = json.loads(execution_binding_path.read_text())
    for binding in execution_bindings['rows']:
        assert sha(ROOT / binding['script']) == binding['current_sha256']
        assert sha(ROOT / binding['executed_receipt']) == binding['receipt_sha256']
        assert sha(ROOT / binding['executed_script_archive']) == binding['executed_sha256']
    result['executed_harness_bindings'] = {'path':str(execution_binding_path.relative_to(ROOT)), 'sha256':sha(execution_binding_path), 'claim':execution_bindings['claim']}
result['changed_harnesses'] = sorted(p for p,r in current_harnesses.items() if r['changed'])
selection = json.loads((HERE / 'execution/single_goal' / final_names['selection']).read_text())
sys.path.insert(0, str(C))
from r05_harness import verify
proof_path, proof = verify(selection['labels']['r05'], result['source_sha256'])
result['r05_harness'] = {'path': str(proof_path.relative_to(ROOT)), 'sha256': sha(proof_path), 'claim': proof['adaptation']}
result['claim'] += ' R05 separately binds the current execution tool to its original workload and guards with one-line None fixture migration and an exact historical include provenance sentinel; the original baseline/tool inputs remain archived.'
with (C / f'ADAPTATION_FINAL_{label}.json').open('x') as stream:
    json.dump(result, stream, indent=2)
    stream.write('\n')
print('PASS unchanged predecessor inputs and explicit current harness-label bindings')
