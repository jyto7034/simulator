"""Normal public independent-area/TTL lifecycle and distinct low-budget samples."""
import difflib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from run import ROOT, HERE, identity, sha

label, inputs_label, *reuse_labels = sys.argv[1:]
assert label.isdigit() and inputs_label.isdigit()
source = identity()
inputs = HERE / 'evidence' / ('playback_inputs_' + inputs_label)
intake = json.loads((inputs / 'INPUTS.json').read_text())
assert source['sha256'] == intake['source_sha256']
assert all(sha(ROOT / p) == h for p, h in intake['files'].items())
out = HERE / 'evidence' / ('lifecycle_samples_' + label)
out.mkdir()
prior = ROOT / 'core_enhanced/docs/goals/core_independent_objects/evidence'
target = Path(os.environ['CARGO_TARGET_DIR']) / 'debug'
binary = target / 'measure_runtime'
binary_sha = sha(binary)
reuse = None
if reuse_labels:
    assert len(reuse_labels) == 1 and reuse_labels[0].isdigit()
    prior_receipt = HERE / 'evidence' / ('lifecycle_actual' + reuse_labels[0] + '.json')
    receipt = json.loads(prior_receipt.read_text())
    assert receipt['source_unchanged'] and receipt['source_before'] == source
    reuse = HERE / 'evidence' / ('lifecycle_samples_' + reuse_labels[0])
rows = []
original_refs = {}

def copy_case(name, original):
    directory = out / name
    shutil.copytree(original, directory)
    original_refs.update({str(p.relative_to(ROOT)): sha(p) for p in original.rglob('*') if p.is_file()})
    scenario = json.loads((directory / 'scenario.json').read_text())
    return directory, scenario

def assertion(index, pointer, value):
    return {'kind': 'script', 'step': {'kind': 'assert', 'from': {'step': index, 'pointer': pointer},
                                      'assertion': {'test': 'equals', 'value': value}}}

def execute(name, pack, selection, session, steps, mode, repeats=1):
    if reuse is not None and name.startswith(('area_', 'ttl_')):
        manifest = reuse / (name + '.json')
        sample = reuse / (name + '.jsonl')
        value = json.loads(manifest.read_text())
        assert value['steps'] == steps and value['sessions'] == [session] and value['repeats'] == repeats and value['observation'] == mode
        assert all(sha(old) == sha(pack / old.relative_to(Path(value['pack_root']))) for old in Path(value['pack_root']).rglob('*') if old.is_file())
        records = [json.loads(line) for line in sample.read_text().splitlines()]
        report = json.loads((reuse / (name + '.stdout')).read_bytes().splitlines()[-1])
        assert report['summary_written'] and not report['summary_missing'] and records[-1]['driver_errors'] == 0
        assert records[0]['binary_sha256'] == binary_sha and records[0]['manifest_sha256'] == sha(manifest) and records[0]['source_sha256_supplied'] == source['sha256']
        row = {'name': name, 'command': [str(binary), str(manifest), str(sample)], 'exit_code': 0,
               'execution': 'exact current output revalidated', 'prior_receipt': str(prior_receipt), 'sample_path': str(sample),
               'manifest_sha256': sha(manifest), 'sample_sha256': sha(sample), 'summary': records[-1],
               'final_states': [r for r in records if r['record'] == 'final_state'],
               'steps': [r['outcome'] for r in records if r['record'] == 'step'],
               'native': [r['summary'] for r in records if r['record'] == 'native_operation']}
        rows.append(row)
        return row
    manifest = out / (name + '.json')
    value = {'schema_version': 1, 'case_id': name, 'source_sha256': source['sha256'],
             'pack_root': str(pack), 'selection_path': str(selection), 'expected_content_sha256': None,
             'observation': mode, 'repeats': repeats, 'sessions': [session], 'steps': steps,
             'max_samples': 4096, 'max_export_bytes': 16 * 1024 * 1024,
             'stop_on_rejection': name.startswith('low_')}
    with manifest.open('x') as f:
        json.dump(value, f, indent=2)
    sample = out / (name + '.jsonl')
    command = [str(binary), str(manifest), str(sample)]
    result = subprocess.run(command, cwd=ROOT, capture_output=True)
    for suffix, payload in [('stdout', result.stdout), ('stderr', result.stderr)]:
        with (out / (name + '.' + suffix)).open('xb') as f:
            f.write(payload)
    assert result.returncode == 0, (name, result.stderr[-2000:])
    records = [json.loads(line) for line in sample.read_text().splitlines()]
    report = json.loads(result.stdout.splitlines()[-1])
    footer = records[-1]
    assert report['summary_written'] and not report['summary_missing'] and footer['driver_errors'] == 0, (name, footer)
    assert records[0]['binary_sha256'] == binary_sha and records[0]['manifest_sha256'] == sha(manifest)
    row = {'name': name, 'command': command, 'exit_code': result.returncode, 'manifest_sha256': sha(manifest),
           'sample_sha256': sha(sample), 'summary': footer,
           'final_states': [r for r in records if r['record'] == 'final_state'],
           'steps': [r['outcome'] for r in records if r['record'] == 'step'],
           'native': [r['summary'] for r in records if r['record'] == 'native_operation']}
    rows.append(row)
    print(json.dumps({'name': name, 'accepted': footer['accepted_steps'], 'rejected': footer['rejected_steps'],
                      'raw': footer['raw_operations_seen'], 'window': footer['process_window']}), flush=True)
    return row

area, area_scenario = copy_case('area', prior / 'independent_attack_area_002')
ttl, ttl_scenario = copy_case('ttl', prior / 'public_object_combat_inputs_001/actor')
definition = ttl / 'content/official/data/summoned_units/base.ron'
before = definition.read_text()
old = 'family:"trial_actor",display_name:"Trial Actor",ttl_ms:30000'
new = 'family:"trial_actor",display_name:"Trial Actor",ttl_ms:1000'
assert before.count(old) == 1
definition.write_text(before.replace(old, new))
with (out / 'TTL_PROFILE.diff').open('x') as f:
    f.write(''.join(difflib.unified_diff(before.splitlines(True), definition.read_text().splitlines(True), fromfile='preserved_actor', tofile='ttl_1000_fixture')))

default_session = {'player': area_scenario['initial']['player_id'], 'seed': area_scenario['initial']['run_seed'],
                   'runtime_safety_path': None, 'expected_effective_profile_sha256': intake['profiles']['default']['runtime_safety_sha256']}
area_steps = [{'kind': 'script', 'step': s} for s in area_scenario['preparation_steps'] + area_scenario['steps']]
area_steps.append(assertion(27, '/payload/battle_update/checkpoint/deployment/deployed_units/0/skill_readiness/resonance_use', None))
ttl_steps = [{'kind': 'script', 'step': s} for s in (ttl_scenario['preparation_steps'] + ttl_scenario['steps'])[:28]]
ttl_steps.append({'kind': 'script', 'step': {'kind': 'assert', 'from': {'step': 27, 'pointer': '/payload/battle_update/checkpoint/independent_objects/active/0'},
                                          'assertion': {'test': 'exists', 'expected': True}}})
ttl_steps.append({'kind': 'elapsed', 'delta_ms': 500})
ttl_steps.append(assertion(29, '/payload/battle_update/checkpoint/independent_objects/active', []))
for case, directory, steps in [('area', area, area_steps), ('ttl', ttl, ttl_steps)]:
    baseline = None
    for mode in ['none', 'normal', 'detached', 'full']:
        row = execute(case + '_' + mode, directory / 'content', directory / 'selection.json', default_session, steps, mode, 3)
        assert row['summary']['rejected_steps'] == 0 and row['summary']['final_states_seen'] == 3
        if baseline is None:
            baseline = row
        else:
            assert row['steps'] == baseline['steps'] and row['final_states'] == baseline['final_states'], (case, mode)

raw = (ROOT / 'config/runtime_safety.ron').read_text()
assert raw.count('max_retained_bytes: 8388608') == 1
nominal = raw.replace('max_retained_bytes: 8388608', 'max_retained_bytes: 262144')
budget_rows = {}
for name, text in [('low_strict', nominal), ('low_dev16', nominal.rstrip()[:-1] + 'development_measurement: Some((revision: 1, emergency_retained_bytes: 16777216)),\n)\n')]:
    profile = out / (name + '.ron')
    with profile.open('x') as f:
        f.write(text)
    budget_fixture = inputs / 'objects_actor'
    validation_command = [str(target / 'author_content'), 'validate', str(budget_fixture / 'content'), str(budget_fixture / 'selection.json'), '--runtime-safety', str(profile)]
    validated = subprocess.run(validation_command, cwd=ROOT, capture_output=True)
    with (out / (name + '_validate.stdout')).open('xb') as f:
        f.write(validated.stdout)
    with (out / (name + '_validate.stderr')).open('xb') as f:
        f.write(validated.stderr)
    assert validated.returncode == 0, validated.stderr[-2000:]
    validation = json.loads(validated.stdout.splitlines()[0])
    budget_scenario = json.loads((budget_fixture / 'scenario.json').read_text())
    session = {'player': budget_scenario['initial']['player_id'], 'seed': budget_scenario['initial']['run_seed'], 'runtime_safety_path': str(profile),
               'expected_effective_profile_sha256': validation['runtime_safety_sha256']}
    budget_steps = [{'kind': 'script', 'step': s} for s in (budget_scenario['preparation_steps'] + budget_scenario['steps'])[:12]]
    row = execute(name, budget_fixture / 'content', budget_fixture / 'selection.json', session, budget_steps, 'normal')
    row['validation'] = {'command': validation_command, 'exit_code': validated.returncode,
                         'binary_sha256': sha(target / 'author_content'), 'profile_file_sha256': sha(profile), 'result': validation}
    budget_rows[name] = row
strict = budget_rows['low_strict']
soft = budget_rows['low_dev16']
assert strict['summary']['rejected_steps'] == 1 and soft['summary']['rejected_steps'] == 0
hard = [r['retained'] for r in strict['native'] if r['retained'] and r['retained']['first_hard_exceed']]
warning = [r['retained'] for r in soft['native'] if r['retained'] and r['retained']['first_soft_exceed']]
assert hard and warning, (strict['native'], soft['native'])
assert all(r['owner_hard_bytes'] == 262144 and r['first_hard_exceed']['requested_bytes'] > 262144 for r in hard)
assert all(r['profile_warning_bytes'] == 262144 and r['owner_hard_bytes'] == 16777216 for r in warning)
assert all(r['retained'] is None or r['retained']['first_hard_exceed'] is None for r in soft['native'])
assert source == identity() and sha(binary) == binary_sha
assert all(sha(ROOT / p) == h for p, h in intake['files'].items())
assert all(sha(ROOT / p) == h for p, h in original_refs.items())
with (out / 'SUMMARY.json').open('x') as f:
    json.dump({'passed': True, 'source_sha256': source['sha256'], 'binary_sha256': binary_sha, 'original_refs': original_refs,
               'rows': rows, 'hard_requests': hard, 'soft_requests': warning,
               'scope': 'Exact independent-area fixture plus actor TTL1000ms in a separate lifecycle input; three fresh public generations per mode. Prior256KiB ordinary entry did not exceed(seed8451/max12935, failure001);8KiB refused generated-content preflight(failure002). Current256KiB is above preflight and below measured634913 cumulative request of the preserved actor fixture first12public steps. Same inputs for strict/dev16MiB, stopping on typed refusal. This is a separate low engineering profile, not deployment8MiB or an8MiB exceed measurement. Native requests/accepted/current/site and allocator/RSS remain separate.'}, f, indent=2)
print('PASS actual repeated public area/TTL release and separate low-profile hard/soft samples')
