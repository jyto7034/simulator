"""Read-only target comparison; writes only the requested new audit receipt."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from datetime import datetime, timezone

A = Path(__file__).resolve().parent
E = A.parent.parent
R = next(p for p in E.parents if (p / 'game_server/Cargo.toml').is_file())
T = E / 'target/u16_2_current_001'
def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda: f.read(1024 * 1024), b''):
            h.update(b)
    return h.hexdigest()

record = json.loads((T / 'AUDIT_TARGET_RECORD.json').read_text())
result = {'target': record['target_id'], 'at_utc': datetime.now(timezone.utc).isoformat(), 'mismatches': []}
for filename, key in [('SOURCE_MANIFEST.json', 'source_manifest_sha256'), ('EVIDENCE_MANIFEST.json', 'evidence_manifest_sha256'), ('SOURCE.tar.gz', 'source_archive_sha256'), ('RESTORE.json', 'restore_sha256')]:
    if digest(T / filename) != record[key]:
        result['mismatches'].append(filename)
members = set()
for kind in ['SOURCE', 'EVIDENCE']:
    rows = json.loads((T / (kind + '_MANIFEST.json')).read_text())['files']
    result[kind.lower() + '_members'] = len(rows)
    for row in rows:
        p = R / row['path']
        if not p.is_file() or p.stat().st_size != row['bytes'] or digest(p) != row['sha256']:
            result['mismatches'].append(row['path'])
        if kind == 'SOURCE':
            members.add(row['path'])
result['production_root_omissions'] = [str(p.relative_to(R)) for folder in ['core_enhanced/src', 'game_server/src', 'game_resources/data'] for p in (R / folder).rglob('*') if p.is_file() and str(p.relative_to(R)) not in members]
result['head'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=R, text=True).strip()
result['passed'] = not result['mismatches'] and not result['production_root_omissions'] and result['head'] == record['HEAD']
result['currentness'] = 'current / unchanged' if result['passed'] else 'requires delta review'
with Path(sys.argv[1]).open('x') as f:
    json.dump(result, f, indent=2)
    f.write('\n')
print(json.dumps(result, indent=2))
