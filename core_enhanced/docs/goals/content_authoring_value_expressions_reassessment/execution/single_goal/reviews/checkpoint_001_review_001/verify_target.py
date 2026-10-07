"""Read-only currentness and archive verification for checkpoint 001; emits JSON."""
import hashlib
import json
from pathlib import Path
import tarfile
from datetime import datetime, timezone

ROOT = Path('/mnt/f/work/simulator')
TARGET = ROOT / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/checkpoints/checkpoint_001/MANIFEST.json'
digest = lambda data: hashlib.sha256(data).hexdigest()
m = json.loads(TARGET.read_text())
errors = []
checked = []

def scan(value):
    if not isinstance(value, (dict, list)):
        return
    if isinstance(value, dict) and 'path' in value and 'sha256' in value:
        path = ROOT / value['path']
        try:
            actual = digest(path.read_bytes())
        except OSError as error:
            actual = str(error)
        checked.append(value['path'])
        if actual != value['sha256']:
            errors.append({'path': value['path'], 'expected': value['sha256'], 'actual': actual})
    for child in value.values() if isinstance(value, dict) else value:
        scan(child)

scan(m)
manifest_hash = digest(TARGET.read_bytes())
if TARGET.with_suffix('.sha256').read_text().split()[0] != manifest_hash:
    errors.append({'path': str(TARGET), 'error': 'detached target hash mismatch'})
initial_path = ROOT / m['initial_baseline']['manifest']
initial_hash = digest(initial_path.read_bytes())
if initial_hash != m['initial_baseline']['manifest_sha256']:
    errors.append({'path': str(initial_path), 'error': 'initial manifest hash mismatch'})
if initial_path.with_suffix('.sha256').read_text().split()[0] != initial_hash:
    errors.append({'path': str(initial_path), 'error': 'initial detached hash mismatch'})
initial = json.loads(initial_path.read_text())
archives = []
for name, path, rows in [
    ('before', m['initial_baseline']['source_archive']['path'], initial['files']),
    ('after', m['current_source']['archive']['path'], m['current_source']['files']),
]:
    with tarfile.open(ROOT / path, 'r:gz') as archive:
        files = [item for item in archive.getmembers() if item.isfile()]
        members = {item.name: item for item in files}
        wanted = {row['path'] for row in rows}
        if len(members) != len(files) or set(members) != wanted:
            errors.append({'archive': name, 'error': 'archive member set mismatch/duplicates'})
        for row in rows:
            member = members.get(row['path'])
            if member is None or digest(archive.extractfile(member).read()) != row['sha256']:
                errors.append({'archive': name, 'path': row['path'], 'error': 'archive content mismatch'})
        archives.append({'name': name, 'file_count': len(files), 'manifest_count': len(rows)})
before = {row['path']: row['sha256'] for row in initial['files']}
after = {row['path']: row['sha256'] for row in m['current_source']['files']}
changes = {'added': sorted(after.keys() - before.keys()), 'deleted': sorted(before.keys() - after.keys()),
           'modified': sorted(p for p in before.keys() & after.keys() if before[p] != after[p])}
declared = m['current_source']['changes_from_initial']
for kind, paths in changes.items():
    if declared[kind] != len(paths) or set(paths) != {r['path'] for r in declared['files'] if r['state'] == kind}:
        errors.append({'error': 'change set mismatch', 'kind': kind})
print(json.dumps({'checked_at': datetime.now(timezone.utc).isoformat(), 'target': m['record'],
    'manifest_sha256': manifest_hash, 'reference_hash_checks': len(checked),
    'source_count': len(m['current_source']['files']), 'execution_record_count': len(m['execution_records']),
    'evidence_count': len(m['evidence']), 'archives': archives, 'changes': changes,
    'passed': not errors, 'errors': errors}, ensure_ascii=False, indent=2))
raise SystemExit(0 if not errors else 1)
