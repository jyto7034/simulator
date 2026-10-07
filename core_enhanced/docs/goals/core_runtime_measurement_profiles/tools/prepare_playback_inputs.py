"""Prepare finite playback inputs from immutable U07 normal public fixtures."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from run import ROOT, HERE, identity, sha

label = sys.argv[1]
assert label.isdigit()
out = HERE / 'evidence' / ('playback_inputs_' + label)
out.mkdir()
source = identity()
prior = ROOT / 'core_enhanced/docs/goals/core_independent_objects'
official = prior / 'execution/single_goal/compatibility/current_001/evidence/public_profile_003'
shutil.copytree(official / 'content', out / 'official_content')
shutil.copyfile(official / 'selection.json', out / 'official_selection.json')
original_refs = {str(p.relative_to(ROOT)): sha(p) for p in official.rglob('*') if p.is_file() and (p.is_relative_to(official / 'content') or p.name == 'selection.json')}
raw = (ROOT / 'config/runtime_safety.ron').read_text()
profiles = {'default': None}
for name, cap in [('dev16', 16 * 1024 * 1024), ('dev32', 32 * 1024 * 1024)]:
    path = out / (name + '.ron')
    with path.open('x') as stream:
        stream.write(raw.rstrip()[:-1] + f'development_measurement: Some((revision: 1, emergency_retained_bytes: {cap})),\n)\n')
    profiles[name] = path
binary = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / '.cache/d07-runtime-amount-product-target')) / 'debug/author_content'
validation = {}
commands = []
for name, profile in profiles.items():
    command = [str(binary), 'validate', str(out / 'official_content'), str(out / 'official_selection.json')]
    if profile:
        command += ['--runtime-safety', str(profile)]
    result = subprocess.run(command, cwd=ROOT, capture_output=True)
    for suffix, payload in [('stdout', result.stdout), ('stderr', result.stderr)]:
        with (out / (name + '_validate.' + suffix)).open('xb') as stream:
            stream.write(payload)
    commands.append({'command': command, 'exit_code': result.returncode, 'binary_sha256': sha(binary)})
    assert result.returncode == 0, (name, result.stderr[-1000:])
    validation[name] = json.loads(result.stdout.splitlines()[0])
assert validation['default']['runtime_safety_sha256'] == 'a7b932aa614c11c8adf2b0e43c35c89f841056d877b78800e935d4d0c61f4a49'
assert len({v['runtime_safety_sha256'] for v in validation.values()}) == 3
assert all(v['identity'] == validation['default']['identity'] for v in validation.values())

def session(profile, player='00000000-0000-0000-0000-000000000019', seed=23):
    return {'player': player, 'seed': seed, 'runtime_safety_path': str(profiles[profile]) if profiles[profile] else None,
            'expected_effective_profile_sha256': validation[profile]['runtime_safety_sha256']}

def manifest(name, sessions, steps, mode='normal', pack=None, selection=None, samples=4096, repeats=1):
    value = {'schema_version': 1, 'case_id': name, 'source_sha256': source['sha256'],
             'pack_root': str(pack or out / 'official_content'), 'selection_path': str(selection or out / 'official_selection.json'),
             'expected_content_sha256': validation['default']['identity']['semantic_sha256'] if not pack else None,
             'observation': mode, 'repeats': repeats, 'sessions': sessions, 'steps': steps,
             'max_samples': samples, 'max_export_bytes': 16 * 1024 * 1024, 'stop_on_rejection': False}
    path = out / (name + '.json')
    with path.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')
    return {'name': name, 'path': str(path), 'sha256': sha(path)}

normal_steps = [
    {'kind': 'prepare_discard', 'document': {'type': 'start_new_game', 'game_mode': 'Standard', 'replace_active_run': False}},
    {'kind': 'enter_ordinary_combat'},
    {'kind': 'command', 'document': {'type': 'pause_battle'}},
    {'kind': 'command', 'document': {'type': 'pause_battle'}, 'current_battle_authority': True},
    {'kind': 'reconnect'},
    {'kind': 'command', 'document': {'type': 'resume_battle'}, 'current_battle_authority': True},
    {'kind': 'elapsed', 'delta_ms': 10}, {'kind': 'host_tick', 'delta_ms': 10},
    {'kind': 'disconnect_pause'},
    {'kind': 'command', 'document': {'type': 'resume_battle'}, 'current_battle_authority': True},
    {'kind': 'checkpoint'},
    {'kind': 'command', 'document': {'type': 'retreat_battle'}, 'current_battle_authority': True},
    {'kind': 'capture_rollback'},
    {'kind': 'prepare_discard', 'document': {'type': 'start_new_game', 'game_mode': 'Standard', 'replace_active_run': True}},
    {'kind': 'restore_rollback'}, {'kind': 'restore_checkpoint'}, {'kind': 'reconnect'}
]
rows = []
for profile in profiles:
    for mode in ['none', 'normal', 'detached', 'full']:
        rows.append(manifest('normal_' + profile + '_' + mode, [session(profile)], normal_steps, mode))
queue_steps = [{'kind': 'admin', 'document': {'type': 'admin_set_enkephalin', 'amount': i}} for i in range(1100)]
for mode in ['none', 'normal', 'detached', 'full']:
    rows.append(manifest('queue_' + mode, [session('default')], queue_steps, mode))
rows.append(manifest('profile_isolation', [session('dev16'), session('dev32', player='00000000-0000-0000-0000-000000000020')], normal_steps))
for case in ['actor', 'small', 'ice']:
    original = prior / 'evidence/public_object_combat_inputs_001' / case
    target = out / ('objects_' + case)
    shutil.copytree(original, target)
    original_refs.update({str(p.relative_to(ROOT)): sha(p) for p in original.rglob('*') if p.is_file()})
    scenario = json.loads((target / 'scenario.json').read_text())
    steps = [{'kind': 'script', 'step': s} for s in scenario['preparation_steps'] + scenario['steps']]
    for profile in profiles:
        rows.append(manifest('objects_' + case + '_' + profile, [session(profile, scenario['initial']['player_id'], scenario['initial']['run_seed'])], steps,
                             pack=target / 'content', selection=target / 'selection.json'))
# Mismatch must refuse before creating output or opening the deliberately absent pack.
bad = json.loads(Path(rows[0]['path']).read_text())
bad['pack_root'] = str(out / 'absent-pack')
bad['sessions'][0]['expected_effective_profile_sha256'] = '0' * 64
with (out / 'mismatch.json').open('x') as stream:
    json.dump(bad, stream, indent=2)
assert source == identity()
assert all(sha(ROOT / path) == digest for path, digest in original_refs.items())
files = {str(p.relative_to(ROOT)): sha(p) for p in out.rglob('*') if p.is_file()}
with (out / 'INPUTS.json').open('x') as stream:
    json.dump({'source_sha256': source['sha256'], 'profiles': validation, 'rows': rows, 'validation_commands': commands,
               'files': files, 'original_refs': original_refs,
               'scope': 'Exact U07 external normal fixtures; actor custom authored and ice wall HP3/defense0 are inherited fixture choices. Trusted 16/32MiB settings live outside content. No numeric/default oracle changes.'}, stream, indent=2)
print(json.dumps({'inputs': str(out), 'cases': len(rows), 'source_sha256': source['sha256']}))
