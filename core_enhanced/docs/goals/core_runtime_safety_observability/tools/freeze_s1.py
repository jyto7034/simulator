"""Freeze actual S1 bytes and prove exact forward/reverse restoration."""
import datetime
import hashlib
import json
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path
import run

R, H = run.ROOT, run.HERE
S = H / 'execution/s1'
T = S / 'target/s1_001'


def write(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')


def rows(base, names):
    return [dict(path=name, bytes=(base / name).stat().st_size, sha256=run.sha(base / name)) for name in sorted(names)]


def digest(entries):
    return hashlib.sha256(json.dumps(entries, sort_keys=True).encode()).hexdigest()


def unpack(archive, destination):
    with tarfile.open(archive) as stream:
        stream.extractall(destination, filter='data')


def pack(name, paths):
    entries = rows(R, [str(p.relative_to(R)) for p in paths])
    with tarfile.open(T / (name + '.tar.gz'), 'x:gz') as stream:
        for row in entries:
            stream.add(R / row['path'], arcname=row['path'], recursive=False)
    write(T / (name + '.json'), entries)
    with tempfile.TemporaryDirectory(prefix='s1-record-restore-') as temp:
        unpack(T / (name + '.tar.gz'), temp)
        names = sorted(str(p.relative_to(temp)) for p in Path(temp).rglob('*') if p.is_file())
        assert rows(Path(temp), names) == entries
    return entries


def main():
    assert json.loads((S / 'COVERAGE.json').read_text())['source_sha256'] == run.identity()['sha256']
    T.mkdir(parents=True, exist_ok=False)
    baseline = S / 'baseline'
    archives = ['SOURCE_BEFORE', 'HOST_CONFIG_BEFORE', 'HOST_EXAMPLES_BEFORE', 'STATUS_BEFORE', 'HOST_PROTOCOL_BEFORE', 'COMPILE_INPUTS_BEFORE']
    old = {}
    for name in archives:
        data = json.loads((baseline / (name + '.json')).read_text())
        for row in data if isinstance(data, list) else data['files']:
            assert row['path'] not in old or old[row['path']] == row
            old[row['path']] = row
    before = [old[name] for name in sorted(old)]
    product = run.identity()
    names = sorted(set(old) | {row['path'] for row in product['files']})
    after = rows(R, names)
    with tempfile.TemporaryDirectory(prefix='s1-source-restore-') as temp:
        b, a = Path(temp) / 'before', Path(temp) / 'after'
        b.mkdir(); a.mkdir()
        for name in archives:
            unpack(baseline / (name + '.tar.gz'), b)
        assert rows(b, old) == before
        with tarfile.open(T / 'SOURCE_BEFORE.tar.gz', 'x:gz') as stream:
            for row in before:
                stream.add(b / row['path'], arcname=row['path'], recursive=False)
        with tarfile.open(T / 'SOURCE_AFTER.tar.gz', 'x:gz') as stream:
            for row in after:
                stream.add(R / row['path'], arcname=row['path'], recursive=False)
        unpack(T / 'SOURCE_AFTER.tar.gz', a)
        assert rows(a, names) == after
        patches = []
        for name in names:
            command = ['diff', '-u', '--label', 'a/' + name if name in old else '/dev/null', '--label', 'b/' + name, str(b / name) if name in old else '/dev/null', str(a / name)]
            result = subprocess.run(command, capture_output=True)
            assert result.returncode in (0, 1)
            patches.append(result.stdout)
        (T / 'FORWARD.patch').write_bytes(b''.join(patches))
        for reverse, expected in [(False, after), (True, before)]:
            result = subprocess.run(['patch', *(['-R'] if reverse else []), '-p1', '--fuzz=0', '--batch', '--input', str(T / 'FORWARD.patch')], cwd=b, capture_output=True)
            (T / ('RESTORE_REVERSE.log' if reverse else 'RESTORE_FORWARD.log')).write_bytes(result.stdout + result.stderr)
            assert result.returncode == 0
            actual_names = sorted(str(p.relative_to(b)) for p in b.rglob('*') if p.is_file())
            assert actual_names == [row['path'] for row in expected]
            assert rows(b, actual_names) == expected
    write(T / 'RESTORE_VERIFICATION.json', dict(before_sha256=digest(before), after_sha256=digest(after), forward_membership_and_bytes_match=True, reverse_membership_and_bytes_match=True, patch_fuzz=0))
    records = sorted(p for p in H.rglob('*') if p.is_file() and not {'target', 'post_audit', '__pycache__'} & set(p.relative_to(H).parts))
    record_rows = pack('RECORDS', records)
    C = R / 'core_enhanced/docs/roadmaps/core_first'
    P = R / 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime'
    F = P / 'execution/single_goal/corrections/f01_action_flow_20261004_001'
    refs = {C / 'ROADMAP.ko.md', C / 'execution/CONTINUOUS_REQUEST_20261004.ko.md'}
    refs.update(p for p in (C / 'execution').rglob('*') if p.is_file())
    refs.update(P / name for name in ['post_audit/20261004_f01_001/COMPLETION_AUDIT.md', 'reviews/r08_r12_20261002_001/CONDITION_REVIEW.json', 'reviews/r08_r12_20261002_001/R06_R07_IMPACT_REVIEW.json'])
    refs.update(F / name for name in ['COVERAGE_CURRENT.json', 'target/f01_action_flow_001/MANIFEST.json'])
    refs.update(p for p in P.iterdir() if p.is_file())
    refs.update(R / 'core_enhanced/docs' / name for name in ['policy_to_goal_pipeline_revised_v2.md', 'goal_execution_pipeline(1).md', 'game_rulebook_next.md', 'game_rules_next/results-and-save.md', 'game_rules_next/open-decisions.md'])
    assert all(p.is_file() for p in refs), sorted(str(p) for p in refs if not p.is_file())
    ref_rows = pack('REFERENCES', sorted(refs))
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=R, text=True).strip()
    tracked = set(subprocess.check_output(['git', 'ls-files', '-z'], cwd=R).decode().split('\0'))
    dirty = set(subprocess.check_output(['git', 'diff', '--name-only', '-z', 'HEAD'], cwd=R).decode().split('\0'))
    membership = {}
    for group, entries in [('source', after), ('records', record_rows), ('references', ref_rows)]:
        for row in entries:
            path = row['path']
            item = membership.setdefault(path, dict(row, groups=[], kind='tracked/dirty' if path in dirty else 'tracked/clean' if path in tracked else 'untracked', generated='/evidence/' in path or '/execution/' in path))
            item['groups'].append(group)
    write(T / 'MEMBERSHIP.json', [membership[name] for name in sorted(membership)])
    assert run.identity() == product
    assert rows(R, names) == after
    changed = [row['path'] for row in after if old.get(row['path']) != row]
    manifest = dict(schema=1, goal='GOAL-CRSO-S1', phase='PHASE-CRSO-S1', goal_set='GS-CRSO-001', handoff='HA-CRSO-001', state='GEX-10 self-verification complete; independent GEX-20 pending', repository=str(R), vcs_revision=revision, captured_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), product_source=product, before_files=before, source_files=after, before_sha256=digest(before), source_sha256=digest(after), changed=changed, added=sorted(set(names) - set(old)), deleted=[], records_manifest_sha256=run.sha(T / 'RECORDS.json'), references_manifest_sha256=run.sha(T / 'REFERENCES.json'), membership_manifest_sha256=run.sha(T / 'MEMBERSHIP.json'), external_actual='All runtime probes use local temporary config/content/store/endpoints; exact argv, endpoints, captured outputs, content and binary hashes are in evidence/public_profile_003, public_api_cli_001, public_boundary_002, cli_current and raw runner receipts. No deployed Unity or remote service is asserted. Cargo.lock pins dependency revisions.', exclusions=['post_audit/** and COMPLETION_AUDIT.md audit output', 'target/** other than this manifest-defined snapshot', '__pycache__ and isolated build output', 'unrelated repository content outside explicit membership', 'S2 observation/export/Grafana, not yet implemented'], evidence_map='execution/s1/COVERAGE.json', predecessor_impact='execution/s1/PREDECESSOR_IMPACT.json', archives_and_patches=[dict(path=p.name, bytes=p.stat().st_size, sha256=run.sha(p)) for p in sorted(T.iterdir()) if p.is_file()])
    write(T / 'MANIFEST.json', manifest)
    print(json.dumps(dict(target=str(T), manifest_sha256=run.sha(T / 'MANIFEST.json'), product_sha256=product['sha256'], source_sha256=manifest['source_sha256'], changed=len(changed))))


if __name__ == '__main__':
    main()
