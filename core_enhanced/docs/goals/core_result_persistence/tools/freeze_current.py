"""Freeze actual U04 bytes and prove exact forward/reverse restoration."""
import datetime
import hashlib
import json
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path
import run

R, H = run.ROOT, run.HERE
S = H / 'execution/single_goal'
target_name = sys.argv[1] if len(sys.argv) > 1 else 'u04_001'
assert target_name in ('u04_001', 'u04_002', 'u04_003')
T = S / 'target' / target_name


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
    coverage_name = 'COVERAGE.json' if target_name == 'u04_001' else 'COVERAGE_' + target_name.rsplit('_', 1)[1] + '.json'
    assert json.loads((S / coverage_name).read_text())['source_sha256'] == run.identity()['sha256']
    T.mkdir(parents=True, exist_ok=False)
    baseline = S / 'baseline'
    archives = ['SOURCE_BEFORE', 'RULEBOOK_BEFORE', 'STATUS_BEFORE']
    if target_name != 'u04_001':
        correction = json.loads((S / 'corrections/f01_host_contract_001/VALIDATION.json').read_text())
        assert correction['passed'] and correction['product_unchanged']
        assert run.sha(R / correction['after_document']['path']) == correction['after_document']['sha256']
        archives.append('HOST_PROTOCOL_BEFORE')
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
    prior = R / 'core_enhanced/docs/goals/core_runtime_fault_isolation'
    prior_target = prior / 'execution/single_goal/corrections/f02_origin_001/target/u03_003'
    refs = {C / 'ROADMAP.ko.md', C / 'execution/CONTINUOUS_REQUEST_20261004.ko.md', C / 'execution/DECISIONS_20261004.ko.md'}
    refs.update(p for p in (C / 'execution/u03_closure_20261004_001').rglob('*') if p.is_file())
    refs.update(p for p in (prior / 'post_audit/u03_20261004_003').rglob('*') if p.is_file() and '__pycache__' not in p.parts)
    refs.update(prior / name for name in ['execution/single_goal/corrections/f02_origin_001/COVERAGE.json', 'execution/single_goal/corrections/f02_origin_001/PREDECESSOR_IMPACT.json', 'execution/single_goal/SERVICE_REUSE.json', 'P2G_30_FREEZE_003.json'])
    refs.update(R / 'core_enhanced/docs' / name for name in ['policy_to_goal_pipeline_revised_v2.md', 'goal_execution_pipeline(1).md', 'game_rulebook_next.md', 'game_rules_next/results-and-save.md', 'game_rules_next/open-decisions.md'])
    refs.update(p for p in prior_target.iterdir() if p.is_file() and p.suffix != '.gz')
    if target_name != 'u04_001':
        first_audit = H / 'post_audit/u04_20261004_001'
        refs.update(p for p in first_audit.rglob('*') if p.is_file() and '__pycache__' not in p.parts)
        refs.update(p for p in (S / 'target/u04_001').iterdir() if p.is_file() and p.suffix != '.gz')
        if target_name == 'u04_003':
            intermediate = S / 'target/u04_002'
            erratum = json.loads((S / 'corrections/f01_host_contract_001/MEMBERSHIP_ERRATUM.json').read_text())
            assert run.sha(intermediate / 'MANIFEST.json') == erratum['intermediate_target_u04_002']['manifest_sha256']
            for row in json.loads((intermediate / 'MANIFEST.json').read_text())['archives_and_patches']:
                assert run.sha(intermediate / row['path']) == row['sha256'], row['path']
            refs.update(p for p in intermediate.iterdir() if p.is_file() and p.suffix != '.gz')
    # Large immutable predecessor archives stay external. Verify every actual
    # member against its original manifest, not only an archive path or hash.
    external_archives=[]
    manifest=json.loads((prior_target/'MANIFEST.json').read_text())
    for archive_name, member_rows in [
        ('SOURCE_BEFORE.tar.gz', manifest['before_files']),
        ('SOURCE_AFTER.tar.gz', manifest['source_files']),
        ('RECORDS.tar.gz', json.loads((prior_target/'RECORDS.json').read_text())),
        ('REFERENCES.tar.gz', json.loads((prior_target/'REFERENCES.json').read_text())),
    ]:
        archive=prior_target/archive_name
        expected={row['path']:row for row in member_rows};observed=set()
        with tarfile.open(archive) as stream:
            for member in stream:
                assert member.isfile() and member.name in expected and member.name not in observed, member.name
                record=expected[member.name];assert member.size==record['bytes']
                digest_member=hashlib.sha256()
                with stream.extractfile(member) as raw:
                    while chunk:=raw.read(1024*1024):digest_member.update(chunk)
                assert digest_member.hexdigest()==record['sha256'],member.name
                observed.add(member.name)
        assert observed==set(expected)
        external_archives.append({'path':str(archive.relative_to(R)),'bytes':archive.stat().st_size,'sha256':run.sha(archive),'member_count':len(observed),'exact_member_bytes_verified':True,'basis_manifest_sha256':run.sha(prior_target/'MANIFEST.json')})
    if target_name != 'u04_001':
        first = S / 'target/u04_001'
        first_manifest = json.loads((first / 'MANIFEST.json').read_text())
        for archive_name, member_rows in [
            ('SOURCE_BEFORE.tar.gz', first_manifest['before_files']),
            ('SOURCE_AFTER.tar.gz', first_manifest['source_files']),
            ('RECORDS.tar.gz', json.loads((first / 'RECORDS.json').read_text())),
            ('REFERENCES.tar.gz', json.loads((first / 'REFERENCES.json').read_text())),
        ]:
            archive = first / archive_name
            expected = {row['path']: row for row in member_rows}
            observed = set()
            with tarfile.open(archive) as stream:
                for member in stream:
                    assert member.isfile() and member.name in expected and member.name not in observed
                    row = expected[member.name]
                    assert member.size == row['bytes']
                    digest_member = hashlib.sha256()
                    with stream.extractfile(member) as raw:
                        while chunk := raw.read(1024 * 1024):
                            digest_member.update(chunk)
                    assert digest_member.hexdigest() == row['sha256'], member.name
                    observed.add(member.name)
            assert observed == set(expected)
            external_archives.append(dict(path=str(archive.relative_to(R)), bytes=archive.stat().st_size, sha256=run.sha(archive), member_count=len(observed), exact_member_bytes_verified=True, basis_manifest_sha256=run.sha(first / 'MANIFEST.json')))
    service=json.loads((S/'SERVICE_REUSE.json').read_text())
    for key in ('reused_audit','reused_service_evidence','reused_master_source_review'):refs.add(R/service[key])
    selected=json.loads((S/'EVIDENCE_SELECTION.json').read_text())
    external=json.loads((H/('evidence/'+selected['fault_prefix']+'SUMMARY.json')).read_text())['external_input_files']
    for path, expected in external.items():
        assert run.sha(R/path)==expected,path
        refs.add(R/path)
    validation_tool=R/'core_enhanced/tools/runtime_architecture_final_validation'
    refs.update(p for p in validation_tool.rglob('*') if p.is_file() and 'target' not in p.relative_to(validation_tool).parts)
    assert all(p.is_file() for p in refs)
    ref_rows=pack('REFERENCES',sorted(refs))
    write(T/'EXTERNAL_ARCHIVES.json',external_archives)
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=R, text=True).strip()
    tracked = set(subprocess.check_output(['git', 'ls-files', '-z'], cwd=R).decode().split('\0'))
    dirty = set(subprocess.check_output(['git', 'diff', '--name-only', '-z', 'HEAD'], cwd=R).decode().split('\0'))
    membership = {}
    for group, entries in [('source', after), ('records', record_rows), ('references', ref_rows), ('external_reference', external_archives)]:
        for row in entries:
            path = row['path']
            item = membership.setdefault(path, dict(row, groups=[], kind='tracked/dirty' if path in dirty else 'tracked/clean' if path in tracked else 'untracked', generated='/evidence/' in path or '/execution/' in path))
            item['groups'].append(group)
    write(T / 'MEMBERSHIP.json', [membership[name] for name in sorted(membership)])
    assert run.identity() == product
    assert rows(R, names) == after
    changed = [row['path'] for row in after if old.get(row['path']) != row]
    manifest = dict(schema=1, goal='GOAL-CRRP-01', phase='PHASE-CRRP-01', goal_set='GS-CRRP-001', handoff='HA-CRRP-001', state='GEX-10 self-verification complete; independent GEX-20 pending', repository=str(R), vcs_revision=revision, captured_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), product_source=product, before_files=before, source_files=after, before_sha256=digest(before), source_sha256=digest(after), changed=changed, added=sorted(set(names) - set(old)), deleted=[], records_manifest_sha256=run.sha(T / 'RECORDS.json'), references_manifest_sha256=run.sha(T / 'REFERENCES.json'), membership_manifest_sha256=run.sha(T / 'MEMBERSHIP.json'), external_actual='All public probes use isolated local config/content/store/endpoints. Exact commands, first fault, file hashes, websocket messages, observer selection cleanup, native parity and binary hashes are in final_fault receipts and execution/single_goal/compatibility/current_003. Numeric/performance predecessor comparisons retain raw failures and approved warning classification. No deployed Unity or remote service is asserted. Cargo.lock pins dependency revisions.', exclusions=['post_audit/** and COMPLETION_AUDIT.md audit output', 'target/** other than this manifest-defined snapshot', '__pycache__ and isolated build output', 'large predecessor archives: exact members and bytes verified, externally bound by EXTERNAL_ARCHIVES.json instead of recursive embedding', 'unrelated repository content outside explicit membership'], evidence_map='execution/single_goal/COVERAGE.json', predecessor_impact='execution/single_goal/PREDECESSOR_IMPACT.json', archives_and_patches=[dict(path=p.name, bytes=p.stat().st_size, sha256=run.sha(p)) for p in sorted(T.iterdir()) if p.is_file()])
    if target_name != 'u04_001':
        manifest['evidence_map'] = 'execution/single_goal/' + coverage_name
        manifest['correction'] = dict(
            finding='F01',
            owner='GOAL-CRRP-01 / PHASE-CRRP-01 correction GEX-10',
            predecessor_target='target/u04_001/MANIFEST.json',
            predecessor_manifest_sha256=run.sha(S / 'target/u04_001/MANIFEST.json'),
            predecessor_audit='post_audit/u04_20261004_001/COMPLETION_AUDIT.md',
            predecessor_audit_sha256=run.sha(H / 'post_audit/u04_20261004_001/COMPLETION_AUDIT.md'),
            product_unchanged=product == json.loads((S / 'target/u04_001/MANIFEST.json').read_text())['product_source'],
            canonical_surface='game_server/docs/unity_bridge_protocol.md',
            validation='execution/single_goal/corrections/f01_host_contract_001/VALIDATION.json',
            result='execution/single_goal/GEX_10_RESULT_' + target_name.rsplit('_', 1)[1] + '.json',
            independent_verdict='pending; original Not Complete remains unchanged',
        )
        assert manifest['correction']['product_unchanged']
        if target_name == 'u04_003':
            manifest['correction']['membership_erratum'] = 'execution/single_goal/corrections/f01_host_contract_001/MEMBERSHIP_ERRATUM.json'
            manifest['correction']['superseded_unaudited_target'] = dict(path='target/u04_002/MANIFEST.json', sha256=run.sha(S / 'target/u04_002/MANIFEST.json'), all_artifact_bytes_preserved=True)
    write(T / 'MANIFEST.json', manifest)
    print(json.dumps(dict(target=str(T), manifest_sha256=run.sha(T / 'MANIFEST.json'), product_sha256=product['sha256'], source_sha256=manifest['source_sha256'], changed=len(changed))))


if __name__ == '__main__':
    main()
