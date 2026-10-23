#!/usr/bin/env python3
"""Freeze U12 source/evidence and verify restoration in disposable trees only.

Reuse U11's exact archive/patch primitives, not its U11-specific main or verdict.
This is root-authored finalization tooling; it cannot award an independent audit.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
E = HERE.parent
GOAL = E.parent.parent
ROOT = next(p for p in HERE.parents if (p / 'core_enhanced/src/lib.rs').is_file())
PRIOR = ROOT / 'core_enhanced/docs/goals/core_mod_validation_apply/execution/single_goal/target/u11_current_001'
PRIOR_AUDIT = PRIOR.parents[3] / 'post_audit/u11_current_001_pass001'
HELPER = PRIOR.parents[1] / 'finalization001/freeze_target.py'
spec = importlib.util.spec_from_file_location('u11_restore_primitives', HELPER)
u11 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(u11)
read, ref, sha, digest = u11.read, u11.ref, u11.sha, u11.digest


def write(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')


def walk(directory, skip):
    if not directory.exists():
        return
    for current, directories, files in os.walk(directory):
        directories[:] = sorted(d for d in directories if d not in skip)
        for name in sorted(files):
            path = Path(current) / name
            assert not path.is_symlink(), path
            if path.is_file() and path.suffix not in {'.pyc', '.pyo'}:
                yield path


def inventories():
    baseline = read(E / 'baseline/BASELINE_001.json')
    scoped = read(E / 'baseline/SCOPED_PREIMAGES_001.json')
    prior_rows = read(u11.check_ref(baseline['inherited_u11']['source_manifest']))['files']
    # The inherited runtime input list excludes unrelated canonical design docs.
    # In particular run_relic_content_design_notes has a separate user's change.
    paths = {ROOT / row['path'] for row in prior_rows}
    for relative in scoped['roots']:
        paths.update(walk(ROOT / relative, set(scoped['skip_dirs'])))
    paths.update(ROOT / relative for relative in scoped['explicit'] + scoped['planned_new_paths'])
    for relative in ['test_parts/runtime001', 'test_parts/vault_cost001/inputs',
                     'test_parts/vault_cost001/probe', 'finalization001']:
        for path in walk(E / relative, {'target', '__pycache__', 'evidence'}):
            if relative == 'finalization001' and path.suffix != '.py':
                continue
            paths.add(path)
    paths.update(E / 'test_parts/vault_cost001' / name for name in ['collect.py', 'prepare_inputs.py'])
    paths.update(E / 'product_parts' / name for name in ['COLLECT_COMMAND_001.py', 'COLLECT_COMMAND_002.py', 'COLLECT_COMMAND_003.py'])
    after = [ref(path) for path in sorted(paths, key=lambda item: str(item.relative_to(ROOT))) if path.is_file()]
    archive_rows = read(E / 'baseline/REUSED_ARCHIVE_MEMBERS_001.json')['files']
    extra = read(E / 'baseline/EXTRA_PREIMAGE_MANIFEST_001.json')
    old = {row['path']: row for row in archive_rows + extra['files']}
    absent = {row['path'] for row in scoped['entries'] if row['preimage']['kind'] == 'absent'}
    scope = {str(path.relative_to(ROOT)) for path in paths} | {row['path'] for row in scoped['entries']}
    before = [old[name] for name in sorted(scope & old.keys() - absent)]
    for row in scoped['entries']:
        if row['exists']:
            assert old[row['path']] == row['current'], row['path']
    return baseline, scoped, after, before, scope


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--prepare-source', action='store_true')
    parser.add_argument('--target', default='u12_current_002')
    args = parser.parse_args()
    print('STAGE inventory', flush=True)
    baseline, scoped, after, before, scope = inventories()
    print(f'STAGE inventory verified: before={len(before)} after={len(after)}', flush=True)
    source = {'files': after, 'sha256': digest(after)}
    source_path = HERE / 'FINAL_SOURCE_MANIFEST_001.json'
    if args.prepare_source:
        write(source_path, source)
        write(HERE / 'RESTORATION_SCOPE_001.json', {
            'before': before, 'before_sha256': digest(before), 'scope': sorted(scope),
            'new_or_absent': sorted(scope - {row['path'] for row in before}),
            'unrelated_excluded': ['core_enhanced/docs/skills/run_relic_content_design_notes.ko.md'],
            'baseline_rule': 'Original scoped preimages override inherited archive membership; new U12 source paths are absent before creation. Existing unrelated dirty work is not reverse-applied.',
        })
        print(json.dumps({'source_files': len(after), 'before_files': len(before), 'source_sha256': source['sha256']}))
        return
    assert read(source_path) == source, 'Current source changed after final input preparation'
    target = E / 'target' / args.target
    assert target.parent == E / 'target' and not target.exists()
    coverage = read(E / 'COVERAGE.json')
    selection = read(HERE / 'EVIDENCE_SELECTION.json')
    impact = read(HERE / 'PREDECESSOR_IMPACT.json')
    review = read(HERE / 'ROOT_CURRENTNESS_REVIEW.json')
    assert all(d['source_sha256'] == source['sha256'] for d in [coverage, selection, impact, review])
    assert len(coverage['acceptances']) == 38 and len(coverage['integration_seams']) == 19
    assert len(coverage['required_dimensions']) == 10 and len(coverage['execution_nodes']) == 3
    assert review['active_writers'] == review['active_runners'] == 0
    assert review['unresolved_product_or_test_failures'] == []
    assert review['independent_audit'] == 'pending_nonparticipant_required'
    for row in selection['bound_artifacts']:
        u11.check_ref(row)
    closure_path = PRIOR_AUDIT / 'AUDIT_CLOSURE.json'
    closure = read(closure_path)
    assert closure['verdict'] == 'Complete' and closure['confidence'] == 'High'
    assert closure['open_findings'] == closure['blocking_findings'] == 0
    assert closure['auditor']['prior_goal_participation'] is False
    assert closure['target_manifest'] == ref(PRIOR / 'MANIFEST.json')
    print('STAGE predecessor archives', flush=True)
    base_archive = u11.check_ref(baseline['inherited_u11']['source_after_archive'])
    all_old = read(E / 'baseline/REUSED_ARCHIVE_MEMBERS_001.json')['files']
    u11.archive_exact(base_archive, all_old)
    extra = read(E / 'baseline/EXTRA_PREIMAGE_MANIFEST_001.json')
    extra_archive = u11.check_ref(extra['archive'])
    u11.archive_exact(extra_archive, extra['files'])
    print('STAGE record inventory', flush=True)
    records, external = [], []
    for path in walk(GOAL, {'target', 'post_audit', '__pycache__'}):
        row = ref(path)
        if path.name.endswith('.tar.gz') or 'artifacts' in path.relative_to(GOAL).parts:
            external.append(row)
        else:
            records.append(row)
    references = [ref(path) for path in sorted(PRIOR_AUDIT.iterdir()) if path.is_file()]
    references.extend([ref(PRIOR / 'MANIFEST.json'), ref(PRIOR / 'EXTERNAL_ARCHIVES.json'), ref(HELPER)])
    print(f'STAGE isolated restoration: records={len(records)} external={len(external)}', flush=True)
    target.mkdir(parents=True)
    expected_before = {row['path']: row for row in before}
    with tempfile.TemporaryDirectory(prefix='u12-isolated-restore-') as directory:
        temporary = Path(directory)
        b, a = temporary / 'before', temporary / 'after'
        b.mkdir(); a.mkdir()
        for archive_path in [base_archive, extra_archive]:
            with tarfile.open(archive_path) as archive:
                for member in archive:
                    if member.name not in expected_before:
                        continue
                    destination = b / member.name
                    assert destination.resolve().is_relative_to(b.resolve())
                    assert not destination.exists(), member.name
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    with archive.extractfile(member) as origin, destination.open('xb') as output:
                        shutil.copyfileobj(origin, output)
                    assert sha(destination) == expected_before[member.name]['sha256']
        assert u11.rows(b) == before
        for row in after:
            destination = a / row['path']
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / row['path'], destination)
        assert u11.rows(a) == after
        print('STAGE source archive packing', flush=True)
        u11.pack(target / 'SOURCE_BEFORE.tar.gz', b, before)
        u11.pack(target / 'SOURCE_AFTER.tar.gz', a, after)
        for label, left, right in [('FORWARD', 'before', 'after'), ('REVERSE', 'after', 'before')]:
            command = ['git', 'diff', '--no-index', '--binary', '--src-prefix=a/', '--dst-prefix=b/', '--', left, right]
            result = subprocess.run(command, cwd=temporary, capture_output=True)
            assert result.returncode in (0, 1), result.stderr
            (target / (label + '.patch')).write_bytes(result.stdout)
        for label, expected in [('FORWARD', after), ('REVERSE', before)]:
            result = subprocess.run(['git', 'apply', '--binary', '-p2', '--whitespace=nowarn', str(target / (label + '.patch'))], cwd=b, capture_output=True)
            (target / ('RESTORE_' + label + '.log')).write_bytes(result.stdout + result.stderr)
            assert result.returncode == 0, result.stderr
            assert u11.rows(b) == expected, label
    write(target / 'RESTORE_VERIFICATION.json', {'isolated_only': True, 'workspace_patch_applied': False,
        'forward_membership_and_bytes_match': True, 'reverse_membership_and_bytes_match': True,
        'before_sha256': digest(before), 'after_sha256': digest(after),
        'before_count': len(before), 'after_count': len(after),
        'baseline': ref(E / 'baseline/BASELINE_001.json'), 'scope': ref(HERE / 'RESTORATION_SCOPE_001.json')})
    print('STAGE forward/reverse verified; record archives', flush=True)
    u11.pack(target / 'RECORDS.tar.gz', ROOT, records)
    u11.pack(target / 'REFERENCES.tar.gz', ROOT, references)
    write(target / 'RECORDS.json', records)
    write(target / 'REFERENCES.json', references)
    write(target / 'EXTERNAL_ARTIFACTS.json', external + [ref(base_archive), ref(extra_archive)])
    git_env = dict(os.environ, GIT_OPTIONAL_LOCKS='0')
    tracked = set(subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT, env=git_env).decode().split('\0'))
    dirty = set(subprocess.check_output(['git', 'diff', '--name-only', '-z', 'HEAD'], cwd=ROOT, env=git_env).decode().split('\0'))
    membership = {}
    for group, rows in [('source', after), ('records', records), ('references', references), ('external', external)]:
        for row in rows:
            item = membership.setdefault(row['path'], dict(row, groups=[],
                kind='tracked_dirty' if row['path'] in dirty else 'tracked_clean' if row['path'] in tracked else 'untracked',
                generated=group in {'records', 'external'}))
            assert item['sha256'] == row['sha256']
            item['groups'].append(group)
    write(target / 'MEMBERSHIP.json', [membership[path] for path in sorted(membership)])
    (target / 'GIT_STATUS.txt').write_bytes(subprocess.check_output(['git', 'status', '--porcelain=v2', '--untracked-files=all'], cwd=ROOT, env=git_env))
    print('STAGE final source and artifact currentness', flush=True)
    assert inventories()[2] == after
    for row in after + records + references + external:
        u11.check_ref(row)
    before_by_path = {row['path']: row for row in before}
    after_by_path = {row['path']: row for row in after}
    write(target / 'MANIFEST.json', {
        'schema': 'u12.frozen-self-verification-target/v1', 'goal': 'GOAL-CEXL-01', 'goal_set': 'GS-CEXL-001',
        'state': 'Frozen self-verification target; independent nonparticipant audit pending',
        'source_manifest': ref(source_path), 'source_sha256': source['sha256'],
        'before_files': before, 'source_files': after, 'before_sha256': digest(before), 'restoration_source_sha256': digest(after),
        'changed': [name for name in sorted(before_by_path.keys() & after_by_path.keys()) if before_by_path[name] != after_by_path[name]],
        'added': sorted(after_by_path.keys() - before_by_path.keys()), 'deleted': sorted(before_by_path.keys() - after_by_path.keys()),
        'coverage': ref(E / 'COVERAGE.json'), 'selection': ref(HERE / 'EVIDENCE_SELECTION.json'),
        'predecessor_impact': ref(HERE / 'PREDECESSOR_IMPACT.json'), 'root_currentness_review': ref(HERE / 'ROOT_CURRENTNESS_REVIEW.json'),
        'predecessor_independent_closure': ref(closure_path),
        'vcs_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'archives_and_patches': [{'path': p.name, 'sha256': sha(p), 'bytes': p.stat().st_size} for p in sorted(target.iterdir()) if p.is_file()],
        'independent_completion': 'pending_nonparticipant_required; no self-awarded Complete/High/open0',
        'runtime_contract': {'continue_version': 12, 'savepoint_version': 13, 'active_store_version': 7, 'profile_store_version': 4, 'journal_version': 3,
            'environment': 'Local Linux x86_64 release host; production runtime config for functional normal path; explicit separate diagnostic full60 profile for costs.',
            'capture_and_commands': 'Exact RESULT_001.json argv/start/end/source/provenance and selected raw traces are bound in EVIDENCE_SELECTION and RECORDS.',
            'mutable_external': 'Private issuer directories and live /tmp stores are excluded; public calibration outputs, committed save samples and original-preservation byte comparisons are copied and bound. Recreate fresh host contexts through supported calibration/adoption; do not copy keys.',
            'excluded_consumers': 'Unity/UI playback, deployment migration, future U13 full map/U14 full deterrence/U15 material vault and unique final-boss E2E are outside U12.'},
        'scope': 'Inherited runtime source plus scoped U12 source, external current test/probe/fixture/finalization tools. All U12 records including failures are archived; binaries and old archives remain byte-bound external artifacts. Unrelated dirty design note excluded.',
    })
    print(json.dumps({'target': str(target.relative_to(ROOT)), 'restoration_verified': True, 'independent_audit': 'pending'}))


if __name__ == '__main__':
    main()
