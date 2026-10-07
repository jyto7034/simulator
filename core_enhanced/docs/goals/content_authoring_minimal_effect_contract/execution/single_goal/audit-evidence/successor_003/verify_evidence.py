from pathlib import Path
import datetime
import difflib
import hashlib
import json
import re
import subprocess

R01 = Path('core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal')
R02 = Path('core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal')
OUT = R01 / 'audit-evidence/successor_003'
digest = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
read = lambda p: json.loads(Path(p).read_text())
issues = []

def check(path, expected):
    p = Path(path)
    actual = digest(p) if p.is_file() else None
    if actual != expected:
        issues.append({'path': str(p), 'expected': expected, 'actual': actual})

target = read(R01 / 'AUDIT_TARGET_003.json')
old = read(R01 / 'AUDIT_TARGET_002.json')
old_manifest = {m['path']: m['sha256'] for m in old['manifest']}
current_manifest = {m['path']: m['sha256'] for m in target['manifest']}
old_changed = [p for p in old_manifest if old_manifest[p] != current_manifest.get(p)]
vi = read(R02 / 'VALIDATION_INPUT_027.json')
current = {m['path']: m['sha256'] for m in vi['inputs']}
for p, h in current.items():
    check(p, h)
prior = {m['path']: m['sha256'] for m in read(R02 / 'VALIDATION_INPUT_023.json')['inputs']}
changed = [p for p in prior if prior[p] != current.get(p)]
before = Path('/tmp/capg-r02-correction-003-before')
for p, h in prior.items():
    check(before / p, h)
patch = ''.join(''.join(difflib.unified_diff(
    (before / p).read_text().splitlines(keepends=True),
    Path(p).read_text().splitlines(keepends=True),
    fromfile='before003/' + p, tofile='after003/' + p,
)) for p in sorted(changed))
stored_patch = R02 / 'evidence/CORRECTION_DIFF_003_FINAL.patch'
if patch != stored_patch.read_text():
    issues.append({'patch': str(stored_patch), 'issue': 'reconstructed patch differs'})

enumerated = subprocess.check_output(['rg', '--files', 'core_enhanced/src', 'core_enhanced/tests', 'game_resources/data'], text=True).splitlines()
missing_inputs = [p for p in enumerated if (p.endswith('.rs') or p.endswith('.ron')) and p not in current]
if missing_inputs:
    issues.append({'omitted_rust_ron_inputs': missing_inputs})

# The old default-release author's three executions are historical. Only its
# unchanged source/query/scalar/parser paths can be reused, with current loader
# and generated-failure coverage supplied by actual VI027 owner executions.
probe = read(R01 / 'audit-evidence/successor_002/external_authoring_probe_result.json')
check(R01 / 'audit-evidence/successor_002/external_authoring_probe.rs', probe['source_sha256'])
check(R01 / 'audit-evidence/successor_002/external_authoring_probe.log', probe['output_sha256'])
check('core_enhanced/tests/fixtures/content_authoring/literal_effects.ron', probe['fixture_ron_sha256'])
assert '3 passed; 0 failed; 0 ignored' in (R01 / 'audit-evidence/successor_002/external_authoring_probe.log').read_text()
stable_r01_surfaces = [p for p in old_manifest if any(p.startswith(prefix) for prefix in [
    'core_enhanced/src/game/ability/', 'core_enhanced/src/game/battle/',
    'core_enhanced/src/game/world', 'core_enhanced/tests/', 'game_resources/data/'
])]
assert all(old_manifest[p] == current_manifest[p] for p in stable_r01_surfaces)

# Verify original baseline bytes and current attribution independently of status.
baseline = read(R02 / 'BASELINE_001.json')
preservation = read(R02 / 'evidence/PRESERVATION_003.json')
baseline_changed = []
for p, h in baseline['scope_hashes'].items():
    check(Path(baseline['backup_path']) / p, h)
    actual = digest(p) if Path(p).is_file() else None
    if actual != h:
        baseline_changed.append(p)
expected_changed = {m['path'] for m in preservation['changed_existing']}
if set(baseline_changed) != expected_changed:
    issues.append({'baseline_attribution_difference': sorted(set(baseline_changed) ^ expected_changed)})

# Verify exact upstream identities as well as the target's general hash binding.
handoff = read(R01.parent.parent / 'P2G_60_HANDOFF_001.json')
for source in handoff['source_locations'] + handoff['external_context_locations']:
    check(source['path'], source['sha256'])
pvr = read(R01.parent.parent / 'P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_002.json')
for key in ['evaluated_phase_plan', 'bound_final_policy_freeze']:
    check(pvr[key]['path'], pvr[key]['sha256'])
assert pvr['overall_result'] == 'passed'

records = []
for rel in target['validation_results']:
    p = Path(rel)
    record = read(p)
    for key in ['input', 'log', 'receipt']:
        if key in record and key + '_sha256' in record:
            check(R02 / record[key], record[key + '_sha256'])
    records.append({'path': str(p), 'sha256': digest(p), 'command': record.get('command'),
                    'input': record.get('input'), 'terminal_exit_code': record.get('terminal_exit_code'),
                    'count': record.get('total_passed', record.get('passed', record.get('commands')))})

full_log = (R02 / 'evidence/full_core_004.log').read_text()
counts = [tuple(map(int, m)) for m in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out', full_log)]
assert [r[0] for r in counts] == [1165, 4, 4, 25, 15, 4]
assert all(not any(r[1:]) for r in counts)
assert '16 passed; 0 failed; 0 ignored; 0 measured; 1149 filtered out' in (R02 / 'evidence/correction_generated_003.log').read_text()

execution_sets = []
for group in ['release_ab_005', 'correction_release_diagnostic_004', 'api_batch_002']:
    directory = R02 / 'evidence' / group
    receipt = read(directory / 'receipt.json')
    rows = []
    for command in receipt['commands']:
        if 'log_sha256' in command:
            log = directory / Path(command['log']).name
            check(log, command['log_sha256'])
            for diagnostic in command.get('required_diagnostics', []):
                if diagnostic not in log.read_text():
                    issues.append({'log': str(log), 'missing_diagnostic': diagnostic})
        for stream in ['stdout', 'stderr']:
            if stream + '_sha256' in command:
                check(directory / (command['name'] + '.' + stream), command[stream + '_sha256'])
        rows.append({'name': command['name'], 'exit_code': command['exit_code']})
    execution_sets.append({'group': group, 'commands': rows})

# API VI025->VI027 change is one appended test and error-only contributor code.
v25 = {m['path']: m['sha256'] for m in read(R02 / 'VALIDATION_INPUT_025.json')['inputs']}
api_changed = [p for p in v25 if v25[p] != current[p]]
reconstructed = Path('/tmp/capg-r02-vi025-reconstructed')
for p in changed:
    check(reconstructed / p, v25[p])
vp = 'core_enhanced/src/game/data/validation.rs'
old_text, new_text = (reconstructed / vp).read_text(), Path(vp).read_text()
start = 'impl GeneratedValidationContext'
end = '\nfn baseline_tuning_path'
api_outside_error_equal = old_text.split(start)[0] == new_text.split(start)[0] and old_text.split(end)[1] == new_text.split(end)[1]
assert api_outside_error_equal
testp = 'core_enhanced/src/game/data/content_pack/tests.rs'
assert Path(testp).read_text().startswith((reconstructed / testp).read_text())

v22 = {m['path']: m['sha256'] for m in read(R02 / 'VALIDATION_INPUT_022.json')['inputs']}
observer_unchanged = []
for p in current:
    if p.startswith('core_enhanced/src/game/world') or p.startswith('core_enhanced/src/game/application') or 'runtime_architecture_final_validation/' in p:
        if p in v22:
            if v22[p] != current[p]: issues.append({'observer_dependency_changed': p})
            else: observer_unchanged.append(p)

result = {
    'reviewer': '/root/r01_correction_fresh_audit',
    'captured_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'new_gameplay_or_test_executions_this_audit': 0,
    'validation_input_count': len(current), 'vi027_sha256': digest(R02 / 'VALIDATION_INPUT_027.json'),
    'vi023_prior_copy_checks': len(prior), 'vi023_to_vi027_changed': changed,
    'vi023_to_vi027_unchanged': len(prior) - len(changed),
    'correction_patch_reconstructed_exactly': patch == stored_patch.read_text(),
    'correction_patch_sha256': digest(stored_patch),
    'independent_rust_ron_enumeration_count': len(enumerated), 'missing_inputs': missing_inputs,
    'prior_r01_target_changed_members': old_changed,
    'prior_r01_target_added_members': len(current_manifest.keys() - old_manifest.keys()),
    'prior_r01_runtime_ability_world_tests_data_unchanged': stable_r01_surfaces,
    'baseline_originals_verified': len(baseline['scope_hashes']),
    'baseline_changed_exactly_attributed': baseline_changed,
    'handoff_exact_hash_checks': 6, 'pvr_overall_result': pvr['overall_result'],
    'validation_records': records, 'full_core_passed_by_target': [r[0] for r in counts],
    'full_core_total_passed': sum(r[0] for r in counts), 'generated_focused_passed': 16,
    'execution_sets': execution_sets,
    'api_vi025_to_vi027_changed': api_changed, 'api_error_helper_outside_bytes_unchanged': api_outside_error_equal,
    'api_limitation': 'Five actual positive consumers and six actual compiler negatives at VI025; reused only for unchanged public/success paths. Current VI027 generated failures use full/focused/current release evidence.',
    'observer_v22_unchanged_paths': observer_unchanged,
    'observer_limitation': 'Historical fixture-capability timing/allocation evidence for unchanged measured snapshot/export/ContextIrrelevant staging. No new execution, loader measurement, or product SLA.',
    'r01_author_probe_limitation': 'Historical successor_002 three-test result remains original target execution; reused only for unchanged authoring semantic/parser source paths, with new actual VI027 full/focused/release coverage for changed publication/generation.',
    'issues': issues,
}
(OUT / 'evidence_review.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: result[k] for k in ['validation_input_count', 'vi023_to_vi027_unchanged', 'correction_patch_reconstructed_exactly', 'baseline_originals_verified', 'full_core_total_passed', 'generated_focused_passed', 'issues']}, indent=2))
