#!/usr/bin/env python3
"""Read-only rustfmt invocation collection; no cfg parser or source rewriting."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

ROOT = Path('/mnt/f/work/simulator')
B = Path(__file__).resolve().parent
OUT = B / 'u09_product_bundle_b_format_check002'


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def ref(path):
    data = path.read_bytes()
    return {'path': str(path.relative_to(ROOT)), 'bytes': len(data),
            'sha256': hashlib.sha256(data).hexdigest()}


def write(name, value):
    with (OUT / name).open('x') as stream:
        stream.write(json.dumps(value, indent=2) + '\n')


assert not OUT.exists()
handoff = json.loads((B / 'IMPLEMENTATION_HANDOFF_001.json').read_text())
check = json.loads((B / 'u09_product_bundle_b_check012.json').read_text())
cfg_path = B.parent.parent / 'test_parts/bundle_cfg001/CLOSED.json'
cfg = json.loads(cfg_path.read_text())
assert ref(cfg_path)['sha256'] == '2050b31c541e5066c52b872fcd4c5d11dc40ab899c88c34322aad9a3190fef75'
expected = check['source_after']
assert cfg['frozen_source_sha256'] == expected['sha256'] == handoff['source']['sha256']
paths = handoff['delta']['paths']
assert len(paths) == 40


def current_source():
    files = [ref(ROOT / row['path']) for row in expected['files']]
    return {'files': files,
            'sha256': hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest()}


before = current_source()
assert before == expected
directories = {ROOT, *ROOT.parents}
for name in paths:
    directories.add((ROOT / name).parent)
    directories.update((ROOT / name).parent.parents)
configs = sorted(directory / name for directory in directories
                 for name in ('rustfmt.toml', '.rustfmt.toml')
                 if (directory / name).is_file())
assert not configs, 'Unexpected discovered config; inspect before running'
env = os.environ.copy()
env['PATH'] = '/home/blast/.cargo/bin:' + env['PATH']
env['PYTHONDONTWRITEBYTECODE'] = '1'
rustfmt = '/home/blast/.cargo/bin/rustfmt'
OUT.mkdir()
started = utc()
write('BEFORE.json', {'actual_before_utc': started, 'source': before,
                     'source_writers': 0, 'sole_format_runner': True,
                     'cfg_closed': ref(cfg_path), 'check012': ref(B / 'u09_product_bundle_b_check012.json'),
                     'collector': ref(Path(__file__).resolve()),
                     'edition_basis': [ref(ROOT / 'Cargo.toml'), ref(ROOT / 'core_enhanced/Cargo.toml')],
                     'discovered_config_files': [], 'paths': paths})
version = subprocess.check_output([rustfmt, '--version'], env=env, text=True).strip()
config_command = [rustfmt, '--check', '--edition', '2021', '--config',
                  'skip_children=true', '--print-config', 'current', str(ROOT / paths[0])]
with (OUT / 'current_config.stdout').open('xb') as stdout, (OUT / 'current_config.stderr').open('xb') as stderr:
    config_run = subprocess.run(config_command, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
assert config_run.returncode == 0
config_text = (OUT / 'current_config.stdout').read_text()
assert 'skip_children = true' in config_text and 'edition = "2021"' in config_text
assert current_source() == before
rows = []
for index, name in enumerate(paths):
    command = [rustfmt, '--check', '--edition', '2021', '--config',
               'skip_children=true', '--color', 'never', str(ROOT / name)]
    tick = time.monotonic()
    actual_started = utc()
    stdout_path, stderr_path = OUT / f'{index:02d}.stdout', OUT / f'{index:02d}.stderr'
    with stdout_path.open('xb') as stdout, stderr_path.open('xb') as stderr:
        run = subprocess.run(command, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
    rows.append({'path': name, 'command': command, 'actual_started_at_utc': actual_started,
                 'actual_closed_at_utc': utc(), 'duration_seconds': time.monotonic() - tick,
                 'exit_code': run.returncode, 'stdout': ref(stdout_path), 'stderr': ref(stderr_path)})
after = current_source()
result = {'label': OUT.name, 'kind': 'read_only_rustfmt_check',
          'actual_started_at_utc': started, 'actual_closed_at_utc': utc(),
          'cwd': str(ROOT), 'rustfmt_version': version,
          'edition': '2021', 'config': {'skip_children': True, 'discovered_files': [],
                                     'effective_config_command': config_command,
                                     'stdout': ref(OUT / 'current_config.stdout'),
                                     'stderr': ref(OUT / 'current_config.stderr')},
          'source_before': before, 'source_after': after, 'source_unchanged': before == after,
          'checked_file_count': len(rows), 'rows': rows,
          'all_exit_zero': all(row['exit_code'] == 0 for row in rows),
          'diff_paths': [row['path'] for row in rows if row['exit_code'] == 1],
          'error_paths': [row['path'] for row in rows if row['exit_code'] not in (0, 1)],
          'cfg_closed': ref(cfg_path), 'check012': ref(B / 'u09_product_bundle_b_check012.json'),
          'collector': ref(Path(__file__).resolve()), 'source_writers': 0,
          'product_format_writes': 0, 'other_runners': 0,
          'visibility': 'Visible PID namespace / explicit parent sole-runner handoff; no whole shared-host assertion'}
write('RESULT.json', result)
print(json.dumps({'result': ref(OUT / 'RESULT.json'), 'files': len(rows),
                  'source_unchanged': before == after, 'all_exit_zero': result['all_exit_zero'],
                  'diff_paths': result['diff_paths'], 'error_paths': result['error_paths']}, indent=2))
raise SystemExit(0 if before == after and result['all_exit_zero'] else 1)
