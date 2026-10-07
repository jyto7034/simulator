"""Freeze this WP02 checkpoint and verify its current bytes and reversible delta."""

import datetime
import difflib
import hashlib
import json
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

ROOT = Path('/mnt/f/work/simulator')
HERE = Path(__file__).resolve().parent
TARGET = HERE / 'target/d04_f01_correction_001'
BASE = HERE / 'baseline/entry_001/MANIFEST.json'
EXPECTED = {
    'core_enhanced/src/game/battle/core/mod.rs',
    'core_enhanced/src/game/battle/core/sim.rs',
    'core_enhanced/src/game/battle/core/value_expr_runtime.rs',
    'core_enhanced/src/game/data/mod.rs',
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def meta(path):
    data = path.read_bytes()
    return {'path': str(path.relative_to(ROOT)), 'bytes': len(data), 'sha256': sha(data)}


def save(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, ensure_ascii=False)
        stream.write('\n')


def check(entry):
    assert meta(ROOT / entry['path']) == entry, entry['path']


baseline = json.loads(BASE.read_text())
check(baseline['predecessor'])
check(baseline['archive'])
files = [entry['path'] for entry in baseline['source']['files']]
assert len(files) == len(set(files)) == 333


def archived(path):
    with tarfile.open(path) as archive:
        members = archive.getmembers()
        assert {member.name for member in members} == set(files)
        assert all(member.isfile() and not Path(member.name).is_absolute()
                   and '..' not in Path(member.name).parts for member in members)
        return {name: archive.extractfile(name).read() for name in files}


def identity(contents):
    rows = [(name, len(contents[name]), sha(contents[name])) for name in files]
    return sha(json.dumps(rows, separators=(',', ':')).encode())


def verify(manifest):
    check(manifest['baseline'])
    check(manifest['predecessor'])
    check(manifest['after_archive'])
    for entry in manifest['artifacts'] + manifest['records']:
        check(entry)
    record_by_path = {entry['path']: entry for entry in manifest['records']}
    assert len(record_by_path) == len(manifest['records'])
    with tarfile.open(TARGET / 'EVIDENCE_AND_HANDOFF.tar.gz') as evidence:
        members = evidence.getmembers()
        assert {member.name for member in members} == set(record_by_path)
        for member in members:
            assert member.isfile() and not Path(member.name).is_absolute()
            assert '..' not in Path(member.name).parts
            data = evidence.extractfile(member).read()
            assert {'path': member.name, 'bytes': len(data), 'sha256': sha(data)} == record_by_path[member.name]
    before = archived(ROOT / baseline['archive']['path'])
    after = archived(ROOT / manifest['after_archive']['path'])
    assert identity(before) == manifest['source']['before_digest']
    assert identity(after) == manifest['source']['after_digest']
    assert [entry['path'] for entry in manifest['source']['files']] == files
    assert [entry['before'] for entry in manifest['source']['files']] == [
        {'bytes': item['bytes'], 'sha256': item['sha256']}
        for item in baseline['source']['files']]
    changed = []
    for entry in manifest['source']['files']:
        name = entry['path']
        assert {'bytes': len(before[name]), 'sha256': sha(before[name])} == entry['before']
        assert {'bytes': len(after[name]), 'sha256': sha(after[name])} == entry['after']
        assert (ROOT / name).read_bytes() == after[name], name
        if before[name] != after[name]:
            changed.append(name)
    assert set(changed) == EXPECTED
    assert changed == manifest['source']['changed_paths']
    for origin, destination, patch_name in [
        (before, after, 'SOURCE_DELTA.patch'),
        (after, before, 'SOURCE_RESTORE.patch'),
    ]:
        with tempfile.TemporaryDirectory(prefix='wp02-checkpoint-restore-') as tmp:
            for name, data in origin.items():
                path = Path(tmp) / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
            subprocess.run(['patch', '--batch', '--fuzz=0', '-p1', '-i',
                            str(TARGET / patch_name)], cwd=tmp, check=True,
                           capture_output=True)
            for name in files:
                assert (Path(tmp) / name).read_bytes() == destination[name], name
    return {'state': 'CURRENTNESS_AND_BIDIRECTIONAL_RESTORATION_VERIFIED',
            'source_count': len(files), 'changed_paths': changed,
            'record_count': len(manifest['records'])}


if __name__ == '__main__':
    if len(sys.argv) != 2 or sys.argv[1] not in ('freeze', 'verify'):
        raise SystemExit('use freeze or verify')
    if sys.argv[1] == 'verify':
        manifest = json.loads((TARGET / 'MANIFEST.json').read_text())
        print(json.dumps({'manifest': meta(TARGET / 'MANIFEST.json'),
                          'verification': verify(manifest)}, indent=2))
        raise SystemExit(0)

    assert not TARGET.exists(), TARGET
    before = archived(ROOT / baseline['archive']['path'])
    after = {name: (ROOT / name).read_bytes() for name in files}
    changed = [name for name in files if before[name] != after[name]]
    assert set(changed) == EXPECTED, changed
    TARGET.mkdir(parents=True)
    for reverse, filename in [(False, 'SOURCE_DELTA.patch'),
                              (True, 'SOURCE_RESTORE.patch')]:
        origin, result = (after, before) if reverse else (before, after)
        delta = ''.join(''.join(difflib.unified_diff(
            origin[name].decode().splitlines(keepends=True),
            result[name].decode().splitlines(keepends=True),
            fromfile='a/' + name, tofile='b/' + name)) for name in changed)
        (TARGET / filename).write_text(delta)
    with tarfile.open(TARGET / 'SOURCE_AFTER.tar.gz', 'w:gz') as archive:
        for name in files:
            archive.add(ROOT / name, arcname=name, recursive=False)
    records = [meta(path) for path in sorted(HERE.rglob('*'))
               if path.is_file() and TARGET not in path.parents]
    with tarfile.open(TARGET / 'EVIDENCE_AND_HANDOFF.tar.gz', 'w:gz') as archive:
        for entry in records:
            archive.add(ROOT / entry['path'], arcname=entry['path'], recursive=False)
    manifest = {
        'record': 'R06_WP02_D04_F01_CORRECTION_001',
        'state': 'partial_implementation_self_validation_complete_review_pending',
        'created_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'baseline': meta(BASE),
        'predecessor': baseline['predecessor'],
        'source': {'file_count': len(files), 'before_digest': identity(before),
                   'after_digest': identity(after), 'changed_paths': changed,
                   'added_paths': [], 'deleted_paths': [],
                   'files': [{'path': name,
                              'before': {'bytes': len(before[name]), 'sha256': sha(before[name])},
                              'after': {'bytes': len(after[name]), 'sha256': sha(after[name])}}
                             for name in files]},
        'after_archive': meta(TARGET / 'SOURCE_AFTER.tar.gz'),
        'artifacts': [meta(path) for path in sorted(TARGET.iterdir())],
        'records': records,
        'claims': {'independent_review_pass': False, 'wp02_complete': False,
                   'r06_complete': False, 'publication_unblocked': False,
                   'limits_raised': False},
    }
    verification = verify(manifest)
    save(TARGET / 'RESTORATION_VERIFICATION.json', verification)
    manifest['artifacts'].append(meta(TARGET / 'RESTORATION_VERIFICATION.json'))
    save(TARGET / 'MANIFEST.json', manifest)
    print(json.dumps({'manifest': meta(TARGET / 'MANIFEST.json'),
                      'verification': verification}, indent=2))
