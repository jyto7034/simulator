"""Audit-only normal-host probe on disposable copies; never edits target inputs."""
import datetime
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time

A = Path(__file__).resolve().parent
ROOT = next(p for p in A.parents if (p / 'core_enhanced/src/lib.rs').is_file())
E = A.parent.parent / 'execution/single_goal'
P = E / 'product_parts'
OUT = A / 'original_protection_probe'
OUT.mkdir()
BINARY = P / 'release_compile014/artifacts/normal_host_lifecycle'
CONFIG = P / 'normal_contexts005/standalone.json'
SEED = P / 'normal_return_cycle002/evidence/second_departed/save'
FILES = ['player_profile.json', 'player_profile.backup.json', 'active_run.json', 'active_run.backup.json']

def save(path, value):
    with path.open('x') as f:
        json.dump(value, f, ensure_ascii=False, indent=2)
        f.write('\n')

def digest(path):
    b = path.read_bytes()
    return {'path': str(path.relative_to(ROOT)), 'bytes': len(b), 'sha256': hashlib.sha256(b).hexdigest()}

def compact(value):
    return json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode()

save(OUT / 'INPUTS.json', {'binary': digest(BINARY), 'config': digest(CONFIG),
    'seed': [digest(SEED / n) for n in FILES],
    'authority': 'Existing actual normal admission; no issuer keys read or copied; no approval bypass.',
    'scope': 'Read-only audit with commands applied only to disposable copied save roots.'})
rows = []
for case, bad_file in [('positive', None), ('unknown_profile_current', 'player_profile.json'),
                       ('unknown_profile_backup', 'player_profile.backup.json')]:
    directory = OUT / case
    directory.mkdir()
    root = directory / 'save'
    root.mkdir()
    for name in FILES:
        shutil.copyfile(SEED / name, root / name)
    if bad_file:
        path = root / bad_file
        document = json.loads(path.read_text())
        unsigned = {k: v for k, v in document.items() if k != 'checksum_sha256'}
        assert hashlib.sha256(compact(unsigned)).hexdigest() == document['checksum_sha256'], 'positive checksum reconstruction'
        document['profile']['standard_tutorial_completed'] = True
        unsigned = {k: v for k, v in document.items() if k != 'checksum_sha256'}
        document['checksum_sha256'] = hashlib.sha256(compact(unsigned)).hexdigest()
        path.write_bytes(compact(document))
        shutil.copyfile(path, directory / 'unsupported_original.json')
    initial = [digest(root / n) for n in FILES]
    steps = []
    for operation in (['continue'] if case == 'positive' else ['continue', 'command']):
        command = [str(BINARY), operation, str(CONFIG), str(root), str(root), str(directory / operation)]
        if operation == 'command':
            command_file = directory / 'abandon.json'
            save(command_file, {'type': 'abandon_run'})
            command.append(str(command_file))
        started = time.monotonic()
        with (directory / (operation + '.stdout')).open('xb') as stdout, (directory / (operation + '.stderr')).open('xb') as stderr:
            result = subprocess.run(command, cwd=ROOT, stdout=stdout, stderr=stderr, timeout=60)
        receipt = {'argv': command, 'exit_code': result.returncode, 'duration_seconds': time.monotonic() - started,
                   'child_reaped': True, 'current_files': [digest(root / n) for n in FILES]}
        save(directory / (operation + '.receipt.json'), receipt)
        steps.append(receipt)
        if result.returncode != 0:
            break
    rows.append({'case': case, 'unsupported_file': bad_file, 'initial': initial, 'steps': steps,
                 'unsupported_original_preserved': bad_file is None or (root / bad_file).read_bytes() == (directory / 'unsupported_original.json').read_bytes()})
save(OUT / 'RESULT.json', {'timestamp': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'cases': rows})
print(json.dumps([{'case': r['case'], 'exits': [s['exit_code'] for s in r['steps']],
                  'original_preserved': r['unsupported_original_preserved']} for r in rows]))
