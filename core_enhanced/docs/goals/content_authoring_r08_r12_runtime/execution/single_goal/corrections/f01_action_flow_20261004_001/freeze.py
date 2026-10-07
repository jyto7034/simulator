"""Freeze this correction, preserving the predecessor and proving exact restoration."""
from pathlib import Path
import hashlib
import json
import subprocess
import tarfile
import tempfile
import run

H = Path(__file__).resolve().parent
R = run.ROOT
T = H / 'target' / 'f01_action_flow_001'
T.mkdir(parents=True, exist_ok=False)
sha = run.sha

def write(path, data):
    with path.open('x') as stream:
        json.dump(data, stream, ensure_ascii=False, indent=2)
        stream.write('\n')

def rows(base, names):
    return [[name, (base / name).stat().st_size, sha(base / name)] for name in names]

before = json.loads((H / 'baseline/SOURCE_BEFORE.json').read_text())
after = run.identity()
bnames = [row[0] for row in before['files']]
anames = [row[0] for row in after['files']]
with tarfile.open(T / 'SOURCE_AFTER.tar.gz', 'x:gz') as archive:
    for name in anames:
        archive.add(R / name, arcname=name, recursive=False)
with tempfile.TemporaryDirectory(prefix='u01-exact-restore-') as directory:
    b = Path(directory) / 'before'
    a = Path(directory) / 'after'
    b.mkdir(); a.mkdir()
    with tarfile.open(H / 'baseline/SOURCE_BEFORE.tar.gz') as archive:
        archive.extractall(b, filter='data')
    with tarfile.open(T / 'SOURCE_AFTER.tar.gz') as archive:
        archive.extractall(a, filter='data')
    assert rows(b, bnames) == before['files']
    assert rows(a, anames) == after['files']
    patch = []
    for name in sorted(set(bnames) | set(anames)):
        old = b / name if name in bnames else Path('/dev/null')
        new = a / name if name in anames else Path('/dev/null')
        result = subprocess.run(['diff', '-u', '--label', 'a/' + name if name in bnames else '/dev/null', '--label', 'b/' + name if name in anames else '/dev/null', str(old), str(new)], capture_output=True)
        assert result.returncode in (0, 1)
        patch.append(result.stdout)
    (T / 'FORWARD.patch').write_bytes(b''.join(patch))
    for reverse, expected, names in [(False, after, anames), (True, before, bnames)]:
        command = ['patch', *(['-R'] if reverse else []), '-p1', '--fuzz=0', '--batch', '--input', str(T / 'FORWARD.patch')]
        result = subprocess.run(command, cwd=b, capture_output=True)
        (T / ('RESTORE_REVERSE.log' if reverse else 'RESTORE_FORWARD.log')).write_bytes(result.stdout + result.stderr)
        assert result.returncode == 0
        assert sorted(str(p.relative_to(b)) for p in b.rglob('*') if p.is_file()) == names
        assert rows(b, names) == expected['files']
assert run.identity() == after
changed = [name for name in sorted(set(bnames) | set(anames)) if next((r for r in before['files'] if r[0] == name), None) != next((r for r in after['files'] if r[0] == name), None)]
write(T / 'RESTORE_VERIFICATION.json', dict(current_membership_and_bytes_match=True, forward_membership_and_bytes_match=True, reverse_membership_and_bytes_match=True, patch_fuzz=0, before_digest=before['digest'], after_digest=after['digest'], added=sorted(set(anames) - set(bnames)), deleted=sorted(set(bnames) - set(anames)), changed=changed))
records = sorted(p for p in H.rglob('*') if p.is_file() and not {'target', '__pycache__'} & set(p.relative_to(H).parts))
G = H.parents[3]
C = R / 'core_enhanced/docs/roadmaps/core_first'
references = set(p for p in G.iterdir() if p.is_file())
references.update([G / 'post_audit/20261003_001/COMPLETION_AUDIT.md', G / 'reviews/r08_r12_20261002_001/CONDITION_REVIEW.json', G / 'reviews/r08_r12_20261002_001/R06_R07_IMPACT_REVIEW.json', G / 'execution/single_goal/COVERAGE.json', G / 'execution/single_goal/target/r08_r12_integrated_001/MANIFEST.json'])
references.update(p for p in (C / 'execution').rglob('*') if p.is_file() and p.suffix in ('.md', '.json', '.txt', '.gz'))
references.add(C / 'ROADMAP.ko.md')
references.update(R / 'core_enhanced/docs' / name for name in ['policy_to_goal_pipeline_revised_v2.md', 'goal_execution_pipeline(1).md', 'game_rulebook_next.md', 'game_rules_next/results-and-save.md', 'game_rules_next/open-decisions.md', 'game_rules_next/relics.md'])
references = sorted(references)
assert all(p.is_file() for p in references)
for name, paths in [('RECORDS', records), ('REFERENCES', references)]:
    with tarfile.open(T / (name + '.tar.gz'), 'x:gz') as archive:
        for path in paths:
            archive.add(path, arcname=str(path.relative_to(R)), recursive=False)
    write(T / (name + '.json'), [{'path': str(p.relative_to(R)), 'bytes': p.stat().st_size, 'sha256': sha(p)} for p in paths])
write(T / 'MANIFEST.json', dict(schema=1, goal='GS-CARX-001 / GOAL-CARX-01 / HA-CARX-001', owner='U01 F01 GEX-10 correction; independent GEX-20 pending', before_digest=before['digest'], source_digest=after['digest'], before_files=before['files'], source_files=after['files'], changed=changed, records_manifest_sha256=sha(T / 'RECORDS.json'), references_manifest_sha256=sha(T / 'REFERENCES.json'), archives_and_patches=[{'path': p.name, 'bytes': p.stat().st_size, 'sha256': sha(p)} for p in sorted(T.iterdir()) if p.is_file()]))
print(json.dumps({'target': str(T), 'manifest_sha256': sha(T / 'MANIFEST.json'), 'source_digest': after['digest'], 'changed': changed}))
