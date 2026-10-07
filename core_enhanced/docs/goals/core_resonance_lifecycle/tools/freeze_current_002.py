"""Freeze U05 after external CF-D09 document migration; original target is preserved."""
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
target_name = sys.argv[1] if len(sys.argv) > 1 else 'u05_002'
assert target_name == 'u05_002'
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
    coverage_name = 'COVERAGE.json'
    assert json.loads((S / coverage_name).read_text())['source_sha256'] == run.identity()['sha256']
    assert json.loads((S/'GEX_10_RESULT_002.json').read_text())['source_sha256'] == run.identity()['sha256']
    migration=json.loads((S/'corrections/cf_d09_001/DOCUMENT_PATH_MIGRATION.json').read_text())
    for row in migration['external_migration']['documents']:
        assert run.sha(R/row['after_path'])==row['after_sha256'],row['after_path']
    T.mkdir(parents=True, exist_ok=False)
    baseline = S / 'baseline'
    archives = ['SOURCE_BEFORE', 'AUTHORING_MODEL_BEFORE']
    old = {}
    for name in archives:
        data = json.loads((baseline / (name + '.json')).read_text())
        for row in data if isinstance(data, list) else data['files']:
            assert row['path'] not in old or old[row['path']] == row
            old[row['path']] = row
    before = [old[name] for name in sorted(old)]
    product = run.identity()
    names = sorted(set(old) | {row['path'] for row in product['files']} | {'core_enhanced/docs/archive/game_rulebook_legacy_2026-10-04.md'})
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
    prior = R / 'core_enhanced/docs/goals/core_result_persistence'
    prior_target = prior / 'execution/single_goal/target/u04_003'
    refs = {C / 'ROADMAP.ko.md', C / 'execution/CONTINUOUS_REQUEST_20261004.ko.md', C / 'execution/DECISIONS_20261004.ko.md'}
    refs.update(p for p in (C / 'execution/u04_closure_20261004_001').rglob('*') if p.is_file())
    refs.update(p for p in (C / 'execution/rulebook_authority_correction_20261004_001').rglob('*') if p.is_file())
    refs.update(p for p in (C/'execution/rulebook_path_migration_20261004_001').rglob('*') if p.is_file())
    refs.update(R/row['after_path'] for row in migration['external_migration']['documents'])
    refs.update(p for p in (H/'post_audit/u05_20261004_001').rglob('*') if p.is_file() and '__pycache__' not in p.parts)
    refs.update(p for p in (S/'target/u05_001').iterdir() if p.is_file() and p.suffix!='.gz')
    refs.update(p for p in (prior / 'post_audit/u04_20261004_002').rglob('*') if p.is_file() and '__pycache__' not in p.parts)
    refs.update(prior / name for name in ['execution/single_goal/COVERAGE_003.json', 'execution/single_goal/PREDECESSOR_IMPACT.json', 'execution/single_goal/SERVICE_REUSE.json', 'P2G_30_FREEZE_001.json'])
    refs.update(R / 'core_enhanced/docs' / name for name in ['policy_to_goal_pipeline_revised_v2.md', 'goal_execution_pipeline(1).md', 'game_rulebook.md', 'game_rules/combat.md', 'game_rules/equipment-and-fragments.md', 'game_rules/battle-controls.md', 'game_rules/results-and-save.md', 'game_rules/open-decisions.md'])
    refs.update(p for p in prior_target.iterdir() if p.is_file() and p.suffix != '.gz')
    # Large immutable predecessor archives stay external. Verify every actual
    # member against its original manifest, not only an archive path or hash.
    external_archives=[]
    for archive_target in [prior_target, S/'target/u05_001']:
        manifest=json.loads((archive_target/'MANIFEST.json').read_text())
        for archive_name, member_rows in [
            ('SOURCE_BEFORE.tar.gz', manifest['before_files']),
            ('SOURCE_AFTER.tar.gz', manifest['source_files']),
            ('RECORDS.tar.gz', json.loads((archive_target/'RECORDS.json').read_text())),
            ('REFERENCES.tar.gz', json.loads((archive_target/'REFERENCES.json').read_text())),
        ]:
            archive=archive_target/archive_name
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
            external_archives.append({'path':str(archive.relative_to(R)),'bytes':archive.stat().st_size,'sha256':run.sha(archive),'member_count':len(observed),'exact_member_bytes_verified':True,'basis_manifest_sha256':run.sha(archive_target/'MANIFEST.json')})
    service=json.loads((S/'SERVICE_REUSE.json').read_text())
    for key in ('reused_audit','reused_service_evidence','reused_master_source_review'):refs.add(R/service[key])
    for row in json.loads((S/'PREDECESSOR_IMPACT.json').read_text())['original_requirement_sources']:
        assert run.sha(R/row['path'])==row['sha256'],row['path']
        refs.add(R/row['path'])
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
    external_docs=migration['external_migration']['documents']
    attribution={'external_user_document_correction':[row for row in external_docs if row['after_path'] in names],
        'note':'CF-D09 migration follows CF-D08 authority correction. External before/after documents and exact mappings are preserved; this source/canonical delta is not U05 implementation. Product368 and all43 execution bindings remain unchanged.',
        'intake_sha256':run.sha(S/'corrections/cf_d09_001/DOCUMENT_PATH_MIGRATION.json')}
    write(T/'ATTRIBUTION.json',attribution)
    manifest = dict(schema=1, goal='GOAL-CRSL-01', phase='PHASE-CRSL-01', goal_set='GS-CRSL-001', handoff='HA-CRSL-001', state='GEX-10 self-verification complete; independent GEX-20 pending', repository=str(R), vcs_revision=revision, captured_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), product_source=product, before_files=before, source_files=after, before_sha256=digest(before), source_sha256=digest(after), changed=changed, added=sorted(set(names) - set(old)), deleted=[], records_manifest_sha256=run.sha(T / 'RECORDS.json'), references_manifest_sha256=run.sha(T / 'REFERENCES.json'), membership_manifest_sha256=run.sha(T / 'MEMBERSHIP.json'), external_actual='All public probes use isolated local config/content/store/endpoints. Exact commands, first fault, file hashes, websocket messages, observer selection cleanup, native parity and binary hashes are in final_fault receipts and execution/single_goal/compatibility/current_001. Numeric/performance predecessor comparisons retain raw failures and approved warning classification. No deployed Unity or remote service is asserted. Cargo.lock pins dependency revisions.', exclusions=['post_audit/** and COMPLETION_AUDIT.md audit output', 'target/** other than this manifest-defined snapshot', '__pycache__ and isolated build output', 'large predecessor archives: exact members and bytes verified, externally bound by EXTERNAL_ARCHIVES.json instead of recursive embedding', 'unrelated repository content outside explicit membership'], evidence_map='execution/single_goal/COVERAGE.json', predecessor_impact='execution/single_goal/PREDECESSOR_IMPACT.json', archives_and_patches=[dict(path=p.name, bytes=p.stat().st_size, sha256=run.sha(p)) for p in sorted(T.iterdir()) if p.is_file()])
    manifest['current_execution_record']='execution/single_goal/CURRENT_EXECUTION.json'
    manifest['current_gex10_record']='execution/single_goal/GEX_10_RESULT_002.json'
    manifest['document_migration']='execution/single_goal/corrections/cf_d09_001/DOCUMENT_PATH_MIGRATION.json'
    manifest['predecessor_target']={'path':'execution/single_goal/target/u05_001/MANIFEST.json','sha256':run.sha(S/'target/u05_001/MANIFEST.json')}
    manifest['predecessor_audit']={'path':'post_audit/u05_20261004_001/COMPLETION_AUDIT.md','sha256':run.sha(H/'post_audit/u05_20261004_001/COMPLETION_AUDIT.md'),'verdict':'Not Complete / High','finding':'F01 external document migration currentness'}
    write(T / 'MANIFEST.json', manifest)
    print(json.dumps(dict(target=str(T), manifest_sha256=run.sha(T / 'MANIFEST.json'), product_sha256=product['sha256'], source_sha256=manifest['source_sha256'], changed=len(changed))))


if __name__ == '__main__':
    main()
