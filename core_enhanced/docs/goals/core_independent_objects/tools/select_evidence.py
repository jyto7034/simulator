"""Select fresh successful receipts on the exact source; preserve older records."""
import argparse
import json
from run import ROOT, HERE, identity, sha

parser = argparse.ArgumentParser()
parser.add_argument('--integrated', default='007')
parser.add_argument('--public', default='002')
parser.add_argument('--starter', default='003')
parser.add_argument('--release', default='003')
parser.add_argument('--pc', default='004')
parser.add_argument('--prepare', default='009')
parser.add_argument('--r05', default='004')
parser.add_argument('--trace', default=None)
parser.add_argument('--finalization', default='001')
parser.add_argument('--focused', default='resume_u07_focused_005')
parser.add_argument('--native-focused', default='resume_native_focused_002')
parser.add_argument('--charge-scan', default='resume_charge_scan_002')
args = parser.parse_args()
args.trace = args.trace or args.public
assert all(getattr(args, key).isdigit() for key in ('integrated', 'public', 'starter', 'release', 'pc', 'prepare', 'r05', 'finalization'))
from finalization_paths import paths
_, names = paths(args.finalization)
S = HERE / 'execution/single_goal'
source = identity()
public_names = ['compat_prepared_public', 'compat_prepared_boundary', 'compat_input_validation',
                'compat_default_parity', 'compat_public_oracle', 'compat_public_trace',
                'compat_public_cli', 'compat_public_profile', 'compat_public_boundary', 'compat_public_api',
                'public_schema', 'public_independent_area', 'public_continuation', 'public_rewind', 'public_object_schema']
public_receipts = [f'{name}_{args.trace if name == "compat_public_trace" else args.public}' for name in public_names]
public_receipts += [f'public_starter_schema_{args.starter}', f'public_objects_{args.release}', f'public_object_combat_{args.release}']
public_receipts += [f'final_fault_{args.public}_{mode}' for mode in
                    ['build_on', 'none', 'detached', 'queue_full', 'on', 'websocket', 'build_off', 'off']]
base = 'execution/single_goal/compatibility/current_001'
summaries = {
    'prepared_host': f'{base}/public_probe_{args.public}/RESULT.json',
    'prepared_boundary': f'{base}/boundary_{args.public}/RESULT.json',
    'input_validation': f'{base}/input_validation_{args.public}/SUMMARY.json',
    'defaults': f'{base}/evidence/DEFAULT_NUMBERS_{args.public}.json',
    'oracle': f'{base}/evidence/public_after_{args.public}/SUMMARY.json',
    'trace': f'{base}/evidence/public_after_{args.public}_trace/SUMMARY.json',
    'cli': f'{base}/evidence/cli_current_{args.public}/CLI_FINAL_SUMMARY_002.json',
    'profile': f'{base}/evidence/public_profile_{args.public}/SUMMARY.json',
    'boundary': f'{base}/evidence/public_boundary_{args.public}/SUMMARY.json',
    'api': f'{base}/evidence/public_api_{args.public}/RESULT.json',
    'starter': f'evidence/public_starter_schema_{args.starter}/SUMMARY.json',
    'schema': f'evidence/public_schema_{args.public}/SUMMARY.json',
    'area': f'evidence/public_independent_area_{args.public}/SUMMARY.json',
    'continuation': f'evidence/public_continuation_{args.public}/SUMMARY.json',
    'rewind': f'evidence/public_rewind_{args.public}/SUMMARY.json',
    'object_schema': f'evidence/public_object_schema_{args.public}/SUMMARY.json',
    'objects': f'evidence/public_objects_{args.release}/SUMMARY.json',
    'object_combat': f'evidence/public_object_combat_{args.release}/SUMMARY.json',
    'fault': f'evidence/final_fault_{args.public}_SUMMARY.json',
    'fault_websocket': f'evidence/public_fault_final_{args.public}/SUMMARY.json',
    'public_cost_guards': f'evidence/public_guards{args.public}_raw.json',
    'public_allocator_guards': f'evidence/public_guards_allocator{args.public}_raw.json',
}
selection = {
    'source_sha256': source['sha256'], 'labels': vars(args),
    'finalization': names, 'target': 'u07_' + args.finalization,
    'suites': [f'integrated_{name}{args.integrated}' for name in
               ['core_default', 'core_fixtures', 'core_observation', 'host_default', 'host_observation']],
    'checks': [f'integrated_{name}{args.integrated}' for name in
               ['fmt', 'check', 'event_log', 'clippy_core', 'clippy_host', 'heap']]
              + [f'compat_pc_prepare_{args.prepare}', f'compat_pc_{args.pc}', f'compat_pc_gameplay_{args.pc}',
                 f'compat_pc_cost_{args.pc}', f'compat_r05_prepare_{args.r05}', f'compat_r05_summary_{args.r05}',
                 *([f'compat_r05_dependencies_{args.r05}'] if args.finalization != '001' else []),
                 f'public_guards{args.public}', f'public_guards_allocator{args.public}', f'release_build_{args.release}',
                 args.focused, args.native_focused, args.charge_scan],
    'public_receipts': public_receipts, 'public_summaries': summaries,
    'pc_raw': f'pc_raw_current_{args.pc}.json', 'pc_summary': f'PC_SUMMARY_CURRENT_{args.pc}.json',
    'pc_cost': f'PC_IMMEDIATE_ACCOUNTED_U07_{args.pc}.json',
    'r05_harness': f'r05_harness_{args.r05}/PROOF.json',
    'r05_execution': f'r05_harness_{args.r05}/EXECUTION_DEPENDENCIES.json' if args.finalization != '001' else None,
    'r05_summary': f'R05_SUMMARY_CURRENT_{args.r05}.json', 'r05_receipt': f'compat_r05_{args.r05}',
    'object_receipt': f'public_objects_{args.release}',
    'adaptation': f'compatibility/current_001/ADAPTATION_FINAL_{args.public}.json',
    'performance_environment': [f'compatibility/current_001/evidence/PERFORMANCE_ENVIRONMENT_{args.pc}_pc.json',
                                f'compatibility/current_001/evidence/PERFORMANCE_ENVIRONMENT_{args.r05}_r05.json'],
    'supersession': 'All selected executions are fresh on this exact source. Historical intermediate successes, failures, stopped PC003 and their bytes remain original; no retrospective relabeling or independent verdict.',
}
if args.finalization != '001':
    correction = HERE / 'evidence/f01_correction_001/RECORD.json'
    record = json.loads(correction.read_text())
    assert record['source_after'] == source
    selection['audit_correction'] = {'path':str(correction.relative_to(ROOT)), 'sha256':sha(correction), 'finding':'U07-F01', 'closed_audit_sha256':record['closed_audit_sha256']}
    investigation = HERE / 'evidence/f01_correction_002/RECORD.json'
    investigation_record = json.loads(investigation.read_text())
    assert investigation_record['source_after'] == source and investigation_record['production_change'] is False
    selection['withdrawn_investigation'] = {'path':str(investigation.relative_to(ROOT)), 'sha256':sha(investigation), 'claim':'Normal HP0 evolution premise refuted; own test changes selectively restored to the exact passed source; no additional product guard.'}
objects = [f'public_objects_{args.release}', f'public_object_combat_{args.release}']
validation = {f'V{i:02d}': [] for i in range(1, 21)}
for key in ['V01', 'V04', 'V06', 'V08', 'V09', 'V12', 'V15']:
    validation[key] = objects.copy()
for key in ['V07', 'V10', 'V11', 'V14', 'V17']:
    validation[key] = [selection['object_receipt']]
validation['V05'] = [f'public_independent_area_{args.public}']
validation['V18'] = [selection['object_receipt'], f'public_object_schema_{args.public}', f'public_schema_{args.public}', f'public_starter_schema_{args.starter}']
validation['V17'] += [f'public_continuation_{args.public}', f'public_rewind_{args.public}', f'final_fault_{args.public}_websocket']
validation['V19'] = selection['checks'] + public_receipts + [selection['r05_receipt']]
selection['acceptance_public_receipts'] = validation
for name in selection['suites'] + selection['checks'] + public_receipts + [selection['r05_receipt']]:
    path = HERE / 'evidence' / f'{name}.json'
    receipt = json.loads(path.read_text())
    assert receipt['exit_code'] in ((0, 101) if name == selection['r05_receipt'] else (0,)), name
    assert receipt['source_unchanged'] and receipt['source_before'] == source == receipt['source_after'], name
    assert receipt['log_sha256'] == sha(path.with_suffix('.log')), name
for key, path in summaries.items():
    value = json.loads((HERE / path).read_text())
    assert value.get('passed', True), key
with (S / names['selection']).open('x') as stream:
    json.dump(selection, stream, indent=2)
    stream.write('\n')
print('Selected only exact current-source successful receipts')
