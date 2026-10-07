"""Read-only correction target verification. Emits JSON; never rewrites inputs."""
import hashlib
import json
import tarfile
from pathlib import Path
from datetime import datetime, timezone

ROOT = Path('/mnt/f/work/simulator')
BASE = ROOT / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/checkpoint_001_f01_f04'
TARGET = BASE / 'target/correction_001/MANIFEST.json'
EXPECTED = '87feab8cdfc20b43d7c49c2f6238bd1e684cb7e1daf4f993efe2f193f01dc45e'
sha = lambda data: hashlib.sha256(data).hexdigest()
target = json.loads(TARGET.read_text())
baseline = json.loads((ROOT / target['pre_fix_baseline']['manifest']).read_text())
errors, checked, archives = [], [], []

def check(path, expected):
    checked.append(str(path))
    actual = sha((ROOT / path).read_bytes())
    if actual != expected:
        errors.append({'path': str(path), 'expected': expected, 'actual': actual})

def scan(value):
    if isinstance(value, dict):
        for pathkey, hashkey in [('path', 'sha256'), ('manifest', 'manifest_sha256'), ('receipt', 'receipt_sha256')]:
            if pathkey in value and hashkey in value:
                check(value[pathkey], value[hashkey])
        for child in value.values():
            scan(child)
    elif isinstance(value, list):
        for child in value:
            scan(child)

check(TARGET, EXPECTED)
scan(target)
current_target_errors = list(errors)
for manifest in [TARGET, ROOT / target['pre_fix_baseline']['manifest'], ROOT / target['parent_target']['manifest']]:
    check(manifest, manifest.with_suffix('.sha256').read_text().split()[0])
for key, value in baseline.items():
    if key != 'source':
        scan(value)
for key, value in baseline['source'].items():
    if key != 'files':
        scan(value)
for label, source in [('before', baseline['source']), ('after', target['current_source'])]:
    with tarfile.open(ROOT / source['archive']['path'], 'r:gz') as archive:
        files = [m for m in archive.getmembers() if m.isfile()]
        names = {m.name for m in files}
        expected = {r['path']: r['sha256'] for r in source['files']}
        if names != set(expected) or len(names) != len(files):
            errors.append({'archive': label, 'error': 'member set/duplicate mismatch'})
        for member in files:
            if sha(archive.extractfile(member).read()) != expected.get(member.name):
                errors.append({'archive': label, 'error': 'content mismatch', 'path': member.name})
        archives.append({'name': label, 'verified_files': len(files)})
before = {r['path']: r['sha256'] for r in baseline['source']['files']}
after = {r['path']: r['sha256'] for r in target['current_source']['files']}
changes = {'added': sorted(after.keys()-before.keys()), 'deleted': sorted(before.keys()-after.keys()), 'modified': sorted(p for p in before.keys() & after.keys() if before[p] != after[p])}
declared = target['current_source']['changes_from_pre_fix']
for state, paths in changes.items():
    if declared[state] != len(paths) or set(paths) != {r['path'] for r in declared['files'] if r['state'] == state}:
        errors.append({'error': 'delta mismatch', 'state': state})
parent = json.loads((ROOT / target['parent_target']['manifest']).read_text())
if before != {r['path']: r['sha256'] for r in parent['current_source']['files']}:
    errors.append({'error': 'baseline differs from parent source'})
# Current additions inside source scope must not escape the file manifest.
scope_files = set()
for scoped in target['current_source']['scope']:
    path = ROOT / scoped
    if path.is_file():
        scope_files.add(scoped)
    elif path.is_dir():
        scope_files.update(str(f.relative_to(ROOT)) for f in path.rglob('*') if f.is_file())
if scope_files != set(after):
    errors.append({'error': 'current scope file set differs', 'extra': sorted(scope_files-set(after)), 'missing': sorted(set(after)-scope_files)})
print(json.dumps({'checked_at': datetime.now(timezone.utc).isoformat(), 'target': target['record'], 'manifest_sha256': sha(TARGET.read_bytes()), 'hash_checks': len(checked), 'source_count': len(after), 'record_count': len(target['execution_records']), 'evidence_count': len(target['evidence']), 'archives': archives, 'changes': changes, 'current_target_reference_hashes_passed': not current_target_errors, 'passed': not errors, 'errors': errors}, ensure_ascii=False, indent=2))
raise SystemExit(0 if not errors else 1)
