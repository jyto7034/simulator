"""Read-only prerequisite identity check; writes only this preparation directory.

Does not execute audits, product tests, or frozen predecessor runners.
Run with python3 -B; each output is exclusive to preserve failed attempts.
"""
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
from datetime import datetime, timezone

ROOT = Path('/mnt/f/work/simulator')
HERE = Path(__file__).resolve().parent
DOCS = ROOT / 'core_enhanced/docs'
R07 = 'core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal'
R06 = 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/continuations/r06_completion_001'
CODE = 'core_enhanced/docs/roadmaps/content_authoring/handoffs/R08_R12_CODE_EVIDENCE_20260930.json'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(name, value):
    with (HERE / name).open('x') as stream:
        json.dump(value, stream, indent=2, ensure_ascii=False)
        stream.write('\n')


def walk(folder):
    for parent, dirs, files in os.walk(ROOT / folder):
        dirs[:] = [d for d in dirs if d not in {'target', '__pycache__', '.git', '.cache', 'node_modules'}]
        for name in files:
            p = Path(parent) / name
            if p.is_file():
                yield str(p.relative_to(ROOT))


def compare(entries):
    changed, missing = [], []
    for row in entries:
        p = ROOT / row['path']
        if not p.is_file():
            missing.append(row['path'])
        elif sha(p.read_bytes()) != row['sha256']:
            changed.append({'path': row['path'], 'before': row['sha256'], 'current': sha(p.read_bytes())})
    return {'count': len(entries), 'matched': len(entries) - len(changed) - len(missing), 'changed': changed, 'missing': missing}


started = datetime.now(timezone.utc).isoformat()
status = subprocess.run(['git', 'status', '--porcelain=v1', '-z', '--untracked-files=normal'], cwd=ROOT, capture_output=True, check=True)
(HERE / 'GIT_STATUS_BEFORE.z').write_bytes(status.stdout)
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
index_delta = subprocess.run(['git', 'diff', '--cached', '--binary'], cwd=ROOT, capture_output=True, check=True).stdout
(HERE / 'INDEX_BEFORE.patch').write_bytes(index_delta)

m7path = R07 + '/target/r06_r07_integrated_001/MANIFEST.json'
m6path = R06 + '/target/r06_boundary_001/MANIFEST.json'
m7 = json.loads((ROOT / m7path).read_text())
m6 = json.loads((ROOT / m6path).read_text())
code = json.loads((ROOT / CODE).read_text())

# Same dynamic source membership as the R07 runner, without importing/executing it.
base = json.loads((ROOT / R07 / 'baseline/MANIFEST.json').read_text())['files']
membership = {row['path'] for row in base if (ROOT / row['path']).is_file()}
for folder in ('core_enhanced/src', 'core_enhanced/tests', 'game_resources/data', 'core_enhanced/tools/runtime_architecture_final_validation', 'core_enhanced/tools/runtime_architecture_preservation'):
    membership.update(walk(folder))
membership.add('core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json')
expected = {r['path'] for r in m7['source_files']}

code_results = {}
extra = set()
for key, value in code.items():
    if isinstance(value, list) and value and isinstance(value[0], dict) and 'path' in value[0] and 'sha256' in value[0]:
        code_results[key] = compare(value)
        extra.update(r['path'] for r in value)

# Preserve actual dirty/untracked bytes in the investigation/mutation scope.
# Unrelated caches, archives and other tasks remain untouched, not attributed here.
extra.update({m7path, m6path, CODE, R07 + '/REVIEW_HANDOFF.md', R06 + '/REVIEW_HANDOFF.md', 'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md'})
for name in walk('core_enhanced/docs/roadmaps/content_authoring/handoffs'):
    if 'R08_R12' in name:
        extra.add(name)
rows = []
with tarfile.open(HERE / 'SOURCE_BEFORE.tar.gz', 'x:gz') as archive:
    for name in sorted(membership | extra):
        data = (ROOT / name).read_bytes()
        info = tarfile.TarInfo(name)
        info.size = len(data)
        info.mode = (ROOT / name).stat().st_mode & 0o777
        archive.addfile(info, io.BytesIO(data))
        rows.append({'path': name, 'bytes': len(data), 'sha256': sha(data)})
with tarfile.open(HERE / 'SOURCE_BEFORE.tar.gz') as archive:
    restored = [{'path': m.name, 'bytes': m.size, 'sha256': sha(archive.extractfile(m).read())} for m in archive.getmembers()]
assert restored == rows
assert compare(rows)['matched'] == len(rows)
write('BEFORE_MANIFEST.json', {'purpose': 'preparation snapshot, not final implementation target', 'git_head': head, 'scope': 'R07 dynamic source + R08-R12 investigated inputs + original mutable roadmap; unrelated workspace untouched', 'files': rows, 'archive_sha256': sha((HERE / 'SOURCE_BEFORE.tar.gz').read_bytes()), 'archive_bytes_verified': True})

current_rows = [[n, (ROOT / n).stat().st_size, sha((ROOT / n).read_bytes())] for n in sorted(membership)]
result = {'started_utc': started, 'git_head': head, 'product_tests_run': False, 'independent_audit_run': False, 'r07_manifest': {'path': m7path, 'sha256': sha((ROOT / m7path).read_bytes()), **compare(m7['source_files']), 'added_members': sorted(membership - expected), 'removed_members': sorted(expected - membership), 'expected_source_digest': m7['source_digest'], 'current_source_digest': sha(json.dumps(current_rows, separators=(',', ':')).encode())}, 'r06_boundary': {'path': m6path, 'sha256': sha((ROOT / m6path).read_bytes()), **compare(m6['source_files'])}, 'instruction_evidence': code_results}
write('CURRENTNESS.json', result)

# Search current documentation, preserving paths/lines; archives/before copies excluded.
searched, references, candidates = [], [], []
for parent, dirs, files in os.walk(DOCS):
    dirs[:] = [d for d in dirs if d not in {'target', 'baseline', 'originals', '__pycache__', 'evidence', 'preparation'} and not d.startswith('baseline_')]
    for filename in files:
        if not filename.endswith('.md'):
            continue
        p = Path(parent) / filename
        name = str(p.relative_to(ROOT))
        content = p.read_text(errors='replace')
        searched.append(name)
        for number, line in enumerate(content.splitlines(), 1):
            if 'r06_r07_integrated_001' in line or '22b61c4eb0518fcc11209eaf00e415a4f8b6f283c73037d9a04e8e5c168df59a' in line:
                references.append({'path': name, 'line': number, 'text': line})
        if ('AUDIT' in filename.upper() or 'REVIEW' in filename.upper()) and ('content_authoring_fragment_copies' in name or 'r06_completion_001' in name):
            candidates.append({'path': name, 'sha256': sha(p.read_bytes()), 'lines': content.splitlines()[:12]})
write('AUDIT_DISCOVERY.json', {'searched_markdown_count': len(searched), 'searched_paths': searched, 'excluded': ['target', 'baseline*', 'originals', 'evidence', 'preparation', '__pycache__'], 'exact_integrated_target_mentions': references, 'r06_r07_audit_or_handoff_candidates': candidates, 'limitation': 'Repository evidence discovery only; no external conversation or semantic independent audit. Partial historical R06 reviews are not full R06/R07 completion.'})
print(json.dumps({'r07': result['r07_manifest'], 'r06': {k:v for k,v in result['r06_boundary'].items() if k != 'changed'}, 'r06_changed_count': len(result['r06_boundary']['changed']), 'instruction_evidence': code_results, 'snapshot_files': len(rows), 'audit_candidates': [r['path'] for r in candidates]}, ensure_ascii=False, indent=2))
