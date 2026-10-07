"""Verify preparation artifacts and the sole existing-document edit, not gameplay."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import re
import subprocess
import tarfile
import tempfile

ROOT = Path('/mnt/f/work/simulator')
HERE = Path(__file__).resolve().parent
ROADMAP = 'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md'
sha = lambda b: hashlib.sha256(b).hexdigest()
before = json.loads((HERE / 'BEFORE_MANIFEST.json').read_text())
assert sha((HERE / 'SOURCE_BEFORE.tar.gz').read_bytes()) == before['archive_sha256']
changes = [r['path'] for r in before['files'] if not (ROOT / r['path']).is_file() or sha((ROOT / r['path']).read_bytes()) != r['sha256']]
assert changes == [ROADMAP], changes

target_path = ROOT / 'core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal/target/r06_r07_integrated_001/MANIFEST.json'
target = json.loads(target_path.read_text())
assert sha(target_path.read_bytes()) == '22b61c4eb0518fcc11209eaf00e415a4f8b6f283c73037d9a04e8e5c168df59a'
for row in target['source_files']:
    assert sha((ROOT / row['path']).read_bytes()) == row['sha256'], row['path']

# Recompute dynamic membership at close without running the predecessor tool.
base = json.loads((target_path.parents[2] / 'baseline/MANIFEST.json').read_text())['files']
membership = {r['path'] for r in base if (ROOT / r['path']).is_file()}
for folder in ('core_enhanced/src', 'core_enhanced/tests', 'game_resources/data', 'core_enhanced/tools/runtime_architecture_final_validation', 'core_enhanced/tools/runtime_architecture_preservation'):
    for parent, dirs, files in os.walk(ROOT / folder):
        dirs[:] = [d for d in dirs if d not in {'target', '__pycache__'}]
        for name in files:
            p = Path(parent) / name
            if p.is_file():
                membership.add(str(p.relative_to(ROOT)))
membership.add('core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json')
assert membership == {r['path'] for r in target['source_files']}

code_path = ROOT / 'core_enhanced/docs/roadmaps/content_authoring/handoffs/R08_R12_CODE_EVIDENCE_20260930.json'
code = json.loads(code_path.read_text())
for row in code['code_entries'] + code['document_entries']:
    assert sha((ROOT / row['path']).read_bytes()) == row['sha256'], row['path']

coverage = json.loads((HERE / 'COVERAGE.json').read_text())
expected = {f'A{group:02}-{n:02}' for group,count in ((8,7),(9,10),(10,9),(11,9),(12,10)) for n in range(1,count+1)} | {f'X{n:02}' for n in range(1,11)}
assert len(coverage['conditions']) == 55
assert {r['id'] for r in coverage['conditions']} == expected
assert all(r['status'] == 'not_run' and r['actual_test_selectors'] == [] and r['actual_receipts'] == [] for r in coverage['conditions'])

links = []
for p in HERE.glob('*.md'):
    for label, relative in re.findall(r'\[([^\]]+)\]\(([^)]+)\)', p.read_text()):
        if '://' in relative or relative.startswith('#'):
            continue
        destination = (p.parent / relative.split('#')[0]).resolve()
        assert destination.exists(), (str(p), relative)
        links.append({'from': p.name, 'to': relative})

with tempfile.TemporaryDirectory(prefix='r08-r12-preparation-') as td:
    isolated = Path(td)
    with tarfile.open(HERE / 'SOURCE_BEFORE.tar.gz') as archive:
        archive.extractall(isolated, filter='data')
    for row in before['files']:
        assert sha((isolated / row['path']).read_bytes()) == row['sha256']
    before_bytes = (isolated / ROADMAP).read_bytes()
    diff = subprocess.run(['diff', '-u', '--label', 'a/' + ROADMAP, '--label', 'b/' + ROADMAP, str(isolated / ROADMAP), str(ROOT / ROADMAP)], capture_output=True)
    assert diff.returncode == 1
    with (HERE / 'DOCUMENT_CHANGE.patch').open('xb') as f:
        f.write(diff.stdout)
    for reverse in (False, True):
        command = ['patch', '-p1', '--fuzz=0', '--batch'] + (['-R'] if reverse else []) + ['--input', str(HERE / 'DOCUMENT_CHANGE.patch')]
        result = subprocess.run(command, cwd=isolated, capture_output=True)
        with (HERE / ('DOCUMENT_RESTORE_REVERSE.log' if reverse else 'DOCUMENT_RESTORE_FORWARD.log')).open('xb') as f:
            f.write(result.stdout + result.stderr)
        assert result.returncode == 0
        assert (isolated / ROADMAP).read_bytes() == (before_bytes if reverse else (ROOT / ROADMAP).read_bytes())
    for row in before['files']:
        assert sha((isolated / row['path']).read_bytes()) == row['sha256']

status = subprocess.check_output(['git', 'status', '--porcelain=v1', '-z', '--untracked-files=normal'], cwd=ROOT)
(HERE / 'GIT_STATUS_AFTER.z').write_bytes(status)
report = {'timestamp_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'command': 'python3 -B ' + str(Path(__file__).relative_to(ROOT)), 'scope': 'preparation artifacts only; not implementation validation or independent audit', 'result': 'passed', 'r07_source_current_count': len(target['source_files']), 'r07_membership_equal': True, 'instruction_code_current_count': len(code['code_entries']), 'instruction_document_current_count': len(code['document_entries']), 'before_archive_restored_files': len(before['files']), 'changed_existing_snapshot_files': changes, 'document_forward_and_reverse_isolated_restore': True, 'acceptance_count': 55, 'acceptances_not_run': 55, 'local_links_checked': links, 'product_tests_run': False, 'formal_p2g_validation_run': False, 'independent_audit_run': False, 'r08_r12_final_target_created': False}
with (HERE / 'FINAL_PREPARATION_CHECK.json').open('x') as f:
    json.dump(report, f, ensure_ascii=False, indent=2)
    f.write('\n')
print(json.dumps({k:v for k,v in report.items() if k != 'local_links_checked'}, ensure_ascii=False, indent=2))
