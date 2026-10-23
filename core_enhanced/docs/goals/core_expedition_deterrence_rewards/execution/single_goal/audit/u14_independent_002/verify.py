"""Read-only target/evidence checks for the independent successor audit."""
from pathlib import Path
import datetime, hashlib, json, subprocess, sys, tarfile

R = Path('/mnt/f/work/simulator')
A = Path(__file__).resolve().parent
E = A.parent.parent
T = E / 'target/u14_current_002'
P = E / 'target/u14_current_001'
sha = lambda b: hashlib.sha256(b).hexdigest()

def write(name, value):
    with (A / name).open('x') as f:
        json.dump(value, f, indent=2, ensure_ascii=False)
        f.write('\n')

def manifest(path):
    rows = json.loads(path.read_text())['files']
    assert len({r['path'] for r in rows}) == len(rows)
    bad = []
    for row in rows:
        p = R / row['path']
        if not p.is_file():
            bad.append({'path': row['path'], 'reason': 'missing'})
        else:
            b = p.read_bytes()
            if len(b) != row['bytes'] or sha(b) != row['sha256']:
                bad.append({'path': row['path'], 'reason': 'bytes/hash mismatch'})
    return {'members': len(rows), 'mismatches': bad}

phase = sys.argv[1]
assert phase in ('ENTRY', 'EXIT')
record = json.loads((T / 'AUDIT_TARGET_RECORD.json').read_text())
checks = {name: sha((T / name).read_bytes()) == record[key] for name, key in [
    ('SOURCE.tar.gz', 'source_archive_sha256'),
    ('SOURCE_MANIFEST.json', 'source_manifest_sha256'),
    ('EVIDENCE_MANIFEST.json', 'evidence_manifest_sha256'),
    ('RESTORE.json', 'restore_sha256')]}
em = json.loads((T / 'EVIDENCE_MANIFEST.json').read_text())
checks['predecessor_evidence_manifest'] = sha((P / 'EVIDENCE_MANIFEST.json').read_bytes()) == em['predecessor_evidence_manifest_sha256']
checks['HEAD'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=R, text=True).strip() == record['HEAD']
manifests = {str(p.relative_to(E)): manifest(p) for p in [T / 'SOURCE_MANIFEST.json', T / 'EVIDENCE_MANIFEST.json', P / 'EVIDENCE_MANIFEST.json']}
paths = {row['path'] for row in json.loads((T / 'SOURCE_MANIFEST.json').read_text())['files']}
extra = []
for root in ['core_enhanced/src', 'game_server/src', 'game_resources']:
    extra += [str(p.relative_to(R)) for p in (R / root).rglob('*') if p.is_file() and str(p.relative_to(R)) not in paths]
passed = all(checks.values()) and not extra and all(not m['mismatches'] for m in manifests.values())
value = {'time': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'target': record['target_id'], 'record_checks': checks, 'manifests': manifests, 'unlisted_production_files': extra, 'passed': passed}
write(phase + '_CURRENTNESS.json', value)
assert passed, value
if phase == 'ENTRY':
    def readtar(path):
        with tarfile.open(path) as t:
            items = [m for m in t if m.isfile()]
            assert len({m.name for m in items}) == len(items)
            return {m.name: t.extractfile(m).read() for m in items}
    before, after = readtar(P / 'SOURCE.tar.gz'), readtar(T / 'SOURCE.tar.gz')
    for target, archived in [(P, before), (T, after)]:
        rows = json.loads((target / 'SOURCE_MANIFEST.json').read_text())['files']
        assert set(archived) == {r['path'] for r in rows}
        assert all(sha(archived[r['path']]) == r['sha256'] and len(archived[r['path']]) == r['bytes'] for r in rows)
    delta = json.loads((T / 'DELTA.json').read_text())
    assert {r['path'] for r in delta} == {p for p in before.keys() | after.keys() if before.get(p) != after.get(p)}
    for src, dst in [(before, after), (after, before)]:
        actual = dict(src)
        for op in delta:
            p = op['path']
            assert (sha(before[p]) if p in before else None) == op['before_sha256']
            assert (sha(after[p]) if p in after else None) == op['after_sha256']
            if p in dst:
                actual[p] = dst[p]
            else:
                del actual[p]
        assert actual == dst
    write('RESTORE_RECHECK.json', {'passed': True, 'before_members': len(before), 'after_members': len(after), 'delta_members': len(delta), 'method': 'Independent archive membership/hash and exact forward/reverse bytes verification in memory; existing isolated filesystem restoration reused', 'working_tree_mutations': 0})
print(json.dumps(value))
