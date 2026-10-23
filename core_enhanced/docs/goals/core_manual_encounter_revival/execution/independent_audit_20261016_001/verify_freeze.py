"""Read-only archive/delta audit; never invoke historical finalizers."""
from pathlib import Path
import hashlib
import json
import tarfile

A = Path(__file__).resolve().parent
E = A.parent
R = next(p for p in E.parents if (p / 'game_server/Cargo.toml').is_file())
sha = lambda data: hashlib.sha256(data).hexdigest()
load = lambda path: json.loads(path.read_bytes())


def archive(path):
    with tarfile.open(path) as tar:
        members = tar.getmembers()
        assert all(m.isfile() for m in members), path
        assert len({m.name for m in members}) == len(members), path
        return {m.name: tar.extractfile(m).read() for m in members}


results = {}
for stage, target in [('single_goal', 'u17_current_001'), ('storage_refactor', 'u17_storage_001')]:
    root = E / stage / 'target' / target
    record = load(root / 'AUDIT_TARGET_RECORD.json')
    expected = record.get('hashes') or {
        'SOURCE.tar.gz': record['source_archive_sha256'],
        'BEFORE.tar.gz': record['before_archive_sha256'],
        'SOURCE_MANIFEST.json': record['source_manifest_sha256'],
        'EVIDENCE_MANIFEST.json': record['evidence_manifest_sha256'],
        'RESTORE.json': record['restore_sha256'],
        'SCOPED.diff': record['scoped_diff_sha256'],
        'DELTA.json': record['delta_sha256'],
    }
    for name, digest in expected.items():
        assert sha((root / name).read_bytes()) == digest, (target, name)
    before, after = archive(root / 'BEFORE.tar.gz'), archive(root / 'SOURCE.tar.gz')
    rows = load(root / 'SOURCE_MANIFEST.json')['files']
    assert set(after) == {r['path'] for r in rows}
    for row in rows:
        assert sha(after[row['path']]) == row['sha256'], row['path']
        assert len(after[row['path']]) == row['bytes'], row['path']
    ops = load(root / 'DELTA.json')
    if isinstance(ops, dict):
        ops = ops.get('operations', ops.get('files'))
    changed = {p for p in before.keys() | after.keys() if before.get(p) != after.get(p)}
    assert changed == {r['path'] for r in ops}, target
    for row in ops:
        for label, files in [('before', before), ('after', after)]:
            actual = None if row['path'] not in files else sha(files[row['path']])
            assert actual == row[label + '_sha256'], (target, row['path'], label)
    # Independently replay the guarded byte operations against the archive maps.
    # The original filesystem restore evidence remains separately identified.
    state = dict(before)
    for src, dst in [(before, after), (after, before)]:
        for row in ops:
            name = row['path']
            assert state.get(name) == src.get(name)
            if name in dst:
                state[name] = dst[name]
            else:
                del state[name]
        assert state == dst
    preimages = []
    for path in sorted((E / stage).glob('BASELINE*.json')):
        baseline = load(path)
        if not isinstance(baseline, dict) or 'files' not in baseline:
            continue
        baseline_rows = baseline['files']
        if isinstance(baseline_rows, dict):
            baseline_rows = [dict(path=p, sha256=h) for p,h in baseline_rows.items()]
        for row in baseline_rows:
            assert sha(before[row['path']]) == row['sha256'], (path, row['path'])
            preimages.append(row['path'])
    results[target] = dict(passed=True, record_sha256=sha((root/'AUDIT_TARGET_RECORD.json').read_bytes()),
                          archive_members=len(after), before_members=len(before), delta_members=len(ops),
                          actual_preimage_checks=len(preimages), archive_hashes=expected,
                          original_restore=load(root/'RESTORE.json'),
                          independent_method='archive digests/member bytes/delta preimages and forward/reverse map equality; no product writes')
out = A / 'FREEZE_VERIFICATION.json'
with out.open('x') as f:
    json.dump(results, f, ensure_ascii=False, indent=2)
    f.write('\n')
print(json.dumps({k: {x:v for x,v in r.items() if x in ['passed','archive_members','before_members','delta_members','actual_preimage_checks']} for k,r in results.items()}, indent=2))
