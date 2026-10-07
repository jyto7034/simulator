#!/usr/bin/env python3
"""Immutable commands over the U08 Core, host and literal input basis."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def identity():
    paths = set()
    for folder in ('core_enhanced/src', 'core_enhanced/tests', 'game_server/src',
                   'game_server/tests', 'game_server/config', 'game_server/examples',
                   'config', 'game_resources/data', 'game_resources/live_data'):
        base = ROOT / folder
        if base.exists():
            paths.update(p for p in base.rglob('*') if p.is_file())
    for name in ('Cargo.toml', 'Cargo.lock', 'core_enhanced/Cargo.toml', 'game_server/Cargo.toml',
                 'core_enhanced/docs/audit/live_skill_catalog_manifest.ron',
                 'core_enhanced/docs/skills/live_content_index.md',
                 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json'):
        paths.add(ROOT / name)
    rows = [{'path': str(p.relative_to(ROOT)), 'sha256': sha(p), 'bytes': p.stat().st_size}
            for p in sorted(paths)]
    return {'files': rows, 'sha256': hashlib.sha256(json.dumps(rows, sort_keys=True).encode()).hexdigest()}


def main():
    name, separator, *command = sys.argv[1:]
    assert re.fullmatch(r'[a-z0-9_]+', name) and separator == '--' and command
    output = HERE / 'evidence'
    output.mkdir(exist_ok=True)
    log, receipt = output / f'{name}.log', output / f'{name}.json'
    assert not log.exists() and not receipt.exists()
    env = os.environ.copy()
    env['PATH'] = '/home/blast/.cargo/bin:' + env['PATH']
    env.setdefault('CARGO_HOME', '/tmp/wp02-cargo-home')
    env.setdefault('CARGO_TARGET_DIR', str(ROOT / '.cache/d07-runtime-amount-product-target'))
    env['PYTHONDONTWRITEBYTECODE'] = '1'
    toolchain = {key: subprocess.check_output(cmd, env=env, text=True).strip()
                 for key, cmd in (('rustc', ['rustc', '--version', '--verbose']), ('cargo', ['cargo', '--version']))}
    inputs = []
    for argument in command:
        path = Path(argument)
        path = path if path.is_absolute() else ROOT / path
        if path.is_file() and path.suffix in {'.py', '.rs', '.toml', '.json', '.ron', '.lock'}:
            digest = sha(path)
            archived = output / 'command_inputs' / (digest + path.suffix)
            archived.parent.mkdir(exist_ok=True)
            if not archived.exists():
                with archived.open('xb') as stream:
                    stream.write(path.read_bytes())
            inputs.append({'path': str(path), 'sha256': digest, 'archive': str(archived.relative_to(HERE))})
    runner_digest = sha(Path(__file__))
    archive = output / 'runner_versions' / (runner_digest + '.py')
    archive.parent.mkdir(exist_ok=True)
    if not archive.exists():
        with archive.open('xb') as stream:
            stream.write(Path(__file__).read_bytes())
    before = identity()
    versions = output / 'source_versions'
    versions.mkdir(exist_ok=True)
    source_archive = versions / (before['sha256'] + '.tar.gz')
    if not source_archive.exists():
        with tarfile.open(source_archive, 'x:gz') as archive:
            for row in before['files']:
                path = ROOT / row['path']
                assert sha(path) == row['sha256']
                archive.add(path, arcname=row['path'], recursive=False)
    started_at = datetime.datetime.now(datetime.timezone.utc).isoformat()
    started = time.monotonic()
    with log.open('xb') as stream:
        process = subprocess.run(command, cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT)
    after = identity()
    result = {'started_at_utc': started_at, 'toolchain': toolchain, 'command': command, 'cwd': str(ROOT),
              'source_before': before, 'source_after': after, 'source_unchanged': before == after,
              'exit_code': process.returncode, 'duration_seconds': time.monotonic() - started,
              'log_sha256': sha(log), 'runner_sha256': runner_digest, 'command_inputs': inputs,
              'source_archive': {'path': str(source_archive.relative_to(HERE)), 'sha256': sha(source_archive)},
              'environment': {key: env.get(key) for key in ('CARGO_HOME', 'CARGO_TARGET_DIR', 'RUST_TEST_THREADS')}}
    with receipt.open('x') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')
    print(json.dumps({k: v for k, v in result.items() if k not in ('source_before', 'source_after')}))
    print(log.read_text(errors='replace')[-5000:])
    return process.returncode if before == after else 99


if __name__ == '__main__':
    sys.exit(main())
