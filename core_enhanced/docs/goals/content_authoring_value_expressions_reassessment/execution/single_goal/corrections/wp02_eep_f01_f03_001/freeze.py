"""Freeze or independently verify only this correction's actual-byte artifacts."""
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
TARGET = HERE / 'target/correction_001'
BASE = HERE / 'baseline/entry_001/MANIFEST.json'

def sha(data):
    return hashlib.sha256(data).hexdigest()

def meta(p):
    data = p.read_bytes()
    return dict(path=str(p.relative_to(ROOT)), bytes=len(data), sha256=sha(data))

def save(p, value):
    with p.open('x') as f:
        json.dump(value, f, indent=2, ensure_ascii=False)
        f.write('\n')

def check(entry):
    assert meta(ROOT / entry['path']) == entry, entry['path']

baseline = json.loads(BASE.read_text())
check(baseline['predecessor'])
check(baseline['source']['archive'])
files = [f['path'] for f in baseline['source']['files']]

def archived(p):
    with tarfile.open(p) as archive:
        assert {m.name for m in archive.getmembers()} == set(files)
        assert all(m.isfile() and not Path(m.name).is_absolute() and '..' not in Path(m.name).parts for m in archive.getmembers())
        return {name: archive.extractfile(name).read() for name in files}

def verify(manifest):
    check(manifest['baseline'])
    for entry in manifest['artifacts'] + manifest['records']:
        check(entry)
    before = archived(ROOT / baseline['source']['archive']['path'])
    after = archived(ROOT / manifest['after_archive']['path'])
    for f in manifest['source']['files']:
        assert sha(before[f['path']]) == f['before']['sha256']
        assert sha(after[f['path']]) == f['after']['sha256']
        assert (ROOT / f['path']).read_bytes() == after[f['path']]
    for origin, destination, patch_name in [(before, after, 'SOURCE_DELTA.patch'), (after, before, 'SOURCE_RESTORE.patch')]:
        with tempfile.TemporaryDirectory(prefix='r06-eep-restore-') as tmp:
            for name, data in origin.items():
                p = Path(tmp) / name
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_bytes(data)
            subprocess.run(['patch', '--batch', '--fuzz=0', '-p1', '-i', str(TARGET / patch_name)], cwd=tmp, check=True, capture_output=True)
            for name in files:
                assert (Path(tmp) / name).read_bytes() == destination[name], name
    return dict(state='CURRENTNESS_AND_BIDIRECTIONAL_RESTORATION_VERIFIED', source_count=len(files), changed=manifest['source']['changed_paths'], record_count=len(manifest['records']))

if sys.argv[1] == 'freeze':
    TARGET.mkdir(parents=True, exist_ok=False)
    before = archived(ROOT / baseline['source']['archive']['path'])
    after = {name: (ROOT / name).read_bytes() for name in files}
    changed = [name for name in files if before[name] != after[name]]
    assert set(changed) == {'core_enhanced/src/game/battle/core/sim.rs', 'core_enhanced/src/game/battle/core/value_expr_runtime.rs'}, changed
    for reverse, filename in [(False, 'SOURCE_DELTA.patch'), (True, 'SOURCE_RESTORE.patch')]:
        source, result = (after, before) if reverse else (before, after)
        text = ''.join(''.join(difflib.unified_diff(source[name].decode().splitlines(keepends=True), result[name].decode().splitlines(keepends=True), fromfile='a/' + name, tofile='b/' + name)) for name in changed)
        (TARGET / filename).write_text(text)
    with tarfile.open(TARGET / 'SOURCE_AFTER.tar.gz', 'w:gz') as archive:
        for name in files:
            archive.add(ROOT / name, arcname=name, recursive=False)
    records = [meta(p) for p in sorted(HERE.rglob('*')) if p.is_file() and TARGET not in p.parents]
    review = HERE.parent.parent / 'reviews/wp02_effect_execution_prefix_001_review_001'
    records.extend(meta(review / name) for name in ['REVIEW.md', 'probes.rs', 'probes_expanded.log', 'probes_expanded.json'])
    with tarfile.open(TARGET / 'EVIDENCE_AND_HANDOFF.tar.gz', 'w:gz') as archive:
        for entry in records:
            archive.add(ROOT / entry['path'], arcname=entry['path'], recursive=False)
    manifest = dict(
        record='R06_WP02_EEP_F01_F03_CORRECTION_001',
        state='correction_self_validation_complete_independent_review_pending',
        created_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        baseline=meta(BASE), predecessor=baseline['predecessor'],
        source=dict(file_count=len(files), changed_paths=changed, files=[dict(path=name, before=dict(bytes=len(before[name]), sha256=sha(before[name])), after=dict(bytes=len(after[name]), sha256=sha(after[name]))) for name in files]),
        after_archive=meta(TARGET / 'SOURCE_AFTER.tar.gz'),
        artifacts=[meta(p) for p in sorted(TARGET.iterdir())], records=records,
        claims=dict(independent_review_pass=False, wp02_complete=False, r06_complete=False, publication_unblocked=False, limits_raised=False))
    verification = verify(manifest)
    save(TARGET / 'RESTORATION_VERIFICATION.json', verification)
    manifest['artifacts'].append(meta(TARGET / 'RESTORATION_VERIFICATION.json'))
    save(TARGET / 'MANIFEST.json', manifest)
    print(json.dumps(dict(manifest=meta(TARGET / 'MANIFEST.json'), verification=verification), indent=2))
elif sys.argv[1] == 'verify':
    manifest = json.loads((TARGET / 'MANIFEST.json').read_text())
    print(json.dumps(dict(manifest=meta(TARGET / 'MANIFEST.json'), verification=verify(manifest)), indent=2))
else:
    raise SystemExit('use freeze or verify')
