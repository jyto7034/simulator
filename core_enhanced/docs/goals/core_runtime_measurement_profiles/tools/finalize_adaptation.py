"""Bind untouched historical inputs, executed harness bytes and isolated R05."""
import hashlib
import sys
import tarfile
from evidence_common import ROOT, HERE, S, C, read, write, ref, selected, sha

selection = selected()
label = selection['labels']['public']
validation = C / ('input_validation_' + label) / 'SUMMARY.json'
result = read(validation)
assert result['passed'] and result['predecessor_inputs_unchanged']
original_rows = read(HERE/'evidence/gex00_001/HARNESS_INPUT_BEFORE.json')
with tarfile.open(HERE/'evidence/gex00_001/HARNESS_INPUT_BEFORE.tar.gz') as archive:
    for row in original_rows:
        assert sha(ROOT/row['path']) == row['sha256'], row['path']
        assert hashlib.sha256(archive.extractfile(row['path']).read()).hexdigest() == row['sha256']
intake = read(ROOT/'core_enhanced/docs/goals/core_independent_objects/execution/single_goal/compatibility/current_001/INTAKE_U07.json')
inputs = intake['unchanged_input_files']
assert len(inputs) == 448
for row in inputs:
    assert sha(ROOT/row['source']) == sha(ROOT/row['copy']) == row['sha256']
proofs = []
for name in ['verify_harness_002','r05_scripts_001','r05_dependency_harness_001']:
    proof_path = HERE/'evidence'/name/'ADAPTATION.json'
    proof = read(proof_path)
    assert proof.get('passed', proof.get('prepared', False))
    for row in proof['rows']:
        row_path = row.get('path',row.get('current'))
        expected = row.get('sha256',row.get('current_sha256'))
        assert sha(ROOT/row_path) == expected, row_path
        if row.get('original') is not None:
            assert sha(ROOT/row['original']) == row['original_sha256']
        elif 'original' in row:
            assert row['original_sha256'] is None
    proofs.append(ref(proof_path))
executed = []
for binding in selection['receipt_bindings']:
    value = read(ROOT/binding['path'])
    for item in value['command_inputs']:
        assert sha(HERE/item['archive']) == item['sha256']
        executed.append({'receipt':binding['path'],'script':item['path'],'executed_sha256':item['sha256'],
                         'executed_archive':ref(HERE/item['archive']),
                         'current_sha256':sha(__import__('pathlib').Path(item['path']))})
sys.path.insert(0,str(C))
from r05_harness import verify
r05_path, r05 = verify(selection['labels']['r05'],selection['source_sha256'])
dependencies = r05_path.parent/'EXECUTION_DEPENDENCIES.json'
assert read(dependencies)['passed'] and read(dependencies)['source_sha256'] == selection['source_sha256']
write(C/'ADAPTATION_FINAL.json',{'goal':'GOAL-CMRP-01','source_sha256':selection['source_sha256'],
      'input_validation':ref(validation),'immediate_unchanged_inputs':inputs,'immediate_input_count':len(inputs),
      'historical_harness_input_count':len(original_rows),'historical_preimages':ref(HERE/'evidence/gex00_001/HARNESS_INPUT_BEFORE.json'),
      'prepared_harness_proofs':proofs,'executed_harness_bindings':executed,'r05_harness':ref(r05_path),
      'r05_execution_dependencies':ref(dependencies),
      'claim':'Historical448 input bytes and all original oracle/cap/workload files remain exact. New-goal output namespace, optional measurement feature, and reference-only fixture paths are explicitly diffed. Every executed command input is archived by hash. R05 binds exact original64deb include and currentbbbae one-line None migration in a fresh cache; no baseline rewrite.'})
print('PASS448 unchanged inputs and actual executed harness adaptations')
