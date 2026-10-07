"""Freeze exact U07-M membership and fuzz0 forward/reverse restoration."""
import datetime
import hashlib
import json
import subprocess
import tarfile
import tempfile
from pathlib import Path
from evidence_common import ROOT, HERE, S, C, read, write, ref, selected, identity, sha

selection = selected()
product = identity()
for name in ['COVERAGE.json','PREDECESSOR_IMPACT.json','GEX10_RESULT.json']:
    assert read(S/name)['source_sha256'] == product['sha256']
for directory in Path('/proc').iterdir():
    if not directory.name.isdigit():
        continue
    try:
        comm = (directory/'comm').read_text().strip()
        command = (directory/'cmdline').read_bytes()
        assert not (comm in ('cargo','rustc') and str(ROOT).encode() in command), command
    except (FileNotFoundError,PermissionError,ProcessLookupError):
        pass
target = S/'target'/selection['target']
target.mkdir(parents=True,exist_ok=False)
baseline = HERE/'evidence/gex00_001'


def rows(base, names):
    return [{'path':name,'bytes':(base/name).stat().st_size,'sha256':sha(base/name)} for name in sorted(names)]


def digest(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True).encode()).hexdigest()


def unpack(path, destination):
    with tarfile.open(path) as archive:
        archive.extractall(destination,filter='data')


def verify_archive(path, entries):
    expected = {row['path']:row for row in entries}
    seen = set()
    with tarfile.open(path) as archive:
        for member in archive:
            assert member.isfile() and member.name in expected and member.name not in seen
            row = expected[member.name]
            assert member.size == row['bytes']
            hash_value = hashlib.sha256()
            with archive.extractfile(member) as stream:
                while chunk := stream.read(1024*1024):
                    hash_value.update(chunk)
            assert hash_value.hexdigest() == row['sha256'], member.name
            seen.add(member.name)
    assert seen == set(expected)


def pack(name, paths):
    entries = rows(ROOT,[str(path.relative_to(ROOT)) for path in paths])
    with tarfile.open(target/(name+'.tar.gz'),'x:gz') as archive:
        for row in entries:
            archive.add(ROOT/row['path'],arcname=row['path'],recursive=False)
    write(target/(name+'.json'),entries)
    verify_archive(target/(name+'.tar.gz'),entries)
    return entries


before_map = {row['path']:row for row in read(baseline/'PRODUCT_BEFORE.json')['files']}
docs = read(baseline/'DOCS_BEFORE.json')['files']
for row in docs:
    assert row['path'] not in before_map or before_map[row['path']] == row
    before_map[row['path']] = row
before = [before_map[name] for name in sorted(before_map)]
names = set(before_map) | {row['path'] for row in product['files']} | {str((HERE/'README.md').relative_to(ROOT))}
after = rows(ROOT,[name for name in names if (ROOT/name).is_file()])
after_names = {row['path'] for row in after}
with tempfile.TemporaryDirectory(prefix='u07m-source-restore-') as temp:
    old, new = Path(temp)/'before',Path(temp)/'after'
    old.mkdir();new.mkdir()
    unpack(baseline/'SOURCE_BEFORE.tar.gz',old)
    unpack(baseline/'DOCS_BEFORE.tar.gz',old)
    assert rows(old,[str(path.relative_to(old)) for path in old.rglob('*') if path.is_file()]) == before
    with tarfile.open(target/'SOURCE_BEFORE.tar.gz','x:gz') as archive:
        for row in before:
            archive.add(old/row['path'],arcname=row['path'],recursive=False)
    with tarfile.open(target/'SOURCE_AFTER.tar.gz','x:gz') as archive:
        for row in after:
            archive.add(ROOT/row['path'],arcname=row['path'],recursive=False)
    unpack(target/'SOURCE_AFTER.tar.gz',new)
    assert rows(new,after_names) == after
    changes = []
    for name in sorted(names):
        result = subprocess.run(['diff','-u','--label','a/'+name if name in before_map else '/dev/null',
                                 '--label','b/'+name if name in after_names else '/dev/null',
                                 str(old/name) if name in before_map else '/dev/null',
                                 str(new/name) if name in after_names else '/dev/null'],capture_output=True)
        assert result.returncode in (0,1)
        changes.append(result.stdout)
    (target/'FORWARD.patch').write_bytes(b''.join(changes))
    for reverse, expected in [(False,after),(True,before)]:
        result = subprocess.run(['patch',*(['-R'] if reverse else []),'-p1','--fuzz=0','--batch','--input',str(target/'FORWARD.patch')],cwd=old,capture_output=True)
        (target/('RESTORE_REVERSE.log' if reverse else 'RESTORE_FORWARD.log')).write_bytes(result.stdout+result.stderr)
        assert result.returncode == 0
        actual_names = [str(path.relative_to(old)) for path in old.rglob('*') if path.is_file()]
        assert rows(old,actual_names) == expected
verify_archive(target/'SOURCE_BEFORE.tar.gz',before)
verify_archive(target/'SOURCE_AFTER.tar.gz',after)
write(target/'RESTORE_VERIFICATION.json',{'before_sha256':digest(before),'after_sha256':digest(after),
      'forward_membership_and_bytes_match':True,'reverse_membership_and_bytes_match':True,'patch_fuzz':0,
      'product_archive_scope':'Exact initial389 plus separately archived6 canonical documents. Current source additionally contains the new measurement guide and all product additions.'})
records = sorted(path for path in HERE.rglob('*') if path.is_file() and
                 not {'target','post_audit','__pycache__'} & set(path.relative_to(HERE).parts))
record_rows = pack('RECORDS',records)
refs = set()
for name in ['core_enhanced/docs/policy_to_goal_pipeline_revised_v2.md','core_enhanced/docs/goal_execution_pipeline(1).md',
             'core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md','core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md',
             'core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md']:
    refs.add(ROOT/name)
for folder in ['core_enhanced/docs/roadmaps/core_first/execution/u07_memory_budget_followup_20261005_001',
               'core_enhanced/docs/roadmaps/core_first/execution/u07_closure_20261005_001',
               'core_enhanced/docs/roadmaps/core_first/execution/u07m_start_20261005_001',
               'core_enhanced/docs/goals/core_independent_objects/post_audit/u07_20261005_002']:
    refs.update(path for path in (ROOT/folder).rglob('*') if path.is_file())
for row in read(baseline/'HARNESS_INPUT_BEFORE.json'):
    assert sha(ROOT/row['path']) == row['sha256']
    refs.add(ROOT/row['path'])
# Every externally consumed legacy fixture is bound without copying prior targets.
for binding in selection['public_summaries'].values():
    value = read(ROOT/binding['path'])
    for key in ['external_input_files','original_refs']:
        for name, expected in value.get(key,{}).items():
            assert sha(ROOT/name) == expected
            refs.add(ROOT/name)
for row in read(C/'ADAPTATION_FINAL.json')['immediate_unchanged_inputs']:
    refs.update([ROOT/row['source'],ROOT/row['copy']])
prior = ROOT/'core_enhanced/docs/goals/core_independent_objects'
prior_target = prior/'execution/single_goal/target/u07_002'
manifest = read(prior_target/'MANIFEST.json')
external = []
for name, entries in [('SOURCE_BEFORE.tar.gz',manifest['before_files']),('SOURCE_AFTER.tar.gz',manifest['source_files']),
                      ('RECORDS.tar.gz',read(prior_target/'RECORDS.json')),('REFERENCES.tar.gz',read(prior_target/'REFERENCES.json'))]:
    path = prior_target/name
    verify_archive(path,entries)
    external.append(dict(ref(path),member_count=len(entries),exact_member_bytes_verified=True))
refs.update(path for path in prior_target.iterdir() if path.is_file() and path.suffix != '.gz')
for name in ['COVERAGE_002.json','PREDECESSOR_IMPACT_002.json','EVIDENCE_SELECTION_002.json','GEX_10_RESULT_002.json']:
    refs.add(prior/'execution/single_goal'/name)
for row in read(S/'PREDECESSOR_IMPACT.json')['original_requirement_sources']:
    assert sha(ROOT/row['path']) == row['sha256']
    refs.add(ROOT/row['path'])
for folder in ['core_enhanced/tools/runtime_architecture_final_validation','core_enhanced/tools/runtime_architecture_preservation']:
    base = ROOT/folder
    refs.update(path for path in base.rglob('*') if path.is_file() and 'target' not in path.relative_to(base).parts)
for name in ['core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/baseline/SOURCE_BEFORE.tar.gz',
             'core_enhanced/docs/goals/core_runtime_safety_observability/execution/s1/baseline/SOURCE_BEFORE.tar.gz',
             'core_enhanced/docs/goals/core_runtime_safety_observability/execution/s1/baseline/SOURCE_BEFORE.json',
             'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/evidence/P01_LOG_V32_BASELINE_001.json',
             'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/evidence/LOG_VERSION_MIGRATION_001.json',
             'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json',
             'core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal/evidence/r05_raw_integrated_003.json']:
    refs.add(ROOT/name)
assert all(path.is_file() for path in refs), [str(path) for path in refs if not path.is_file()]
ref_rows = pack('REFERENCES',sorted(refs))
write(target/'EXTERNAL_ARCHIVES.json',external)
tracked = set(subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode().split('\0'))
dirty = set(subprocess.check_output(['git','diff','--name-only','-z','HEAD'],cwd=ROOT).decode().split('\0'))
members = {}
for group, entries in [('source',after),('records',record_rows),('references',ref_rows),('external_reference',external)]:
    for row in entries:
        name = row['path']
        item = members.setdefault(name,dict(row,groups=[],kind='tracked/dirty' if name in dirty else 'tracked/clean' if name in tracked else 'untracked'))
        assert item['sha256'] == row['sha256']
        item['groups'].append(group)
write(target/'MEMBERSHIP.json',[members[name] for name in sorted(members)])
assert identity() == product and rows(ROOT,after_names) == after
write(target/'MANIFEST.json',{'schema':1,'goal':'GOAL-CMRP-01','goal_set':'GS-CMRP-001','phase':'PHASE-CMRP-01',
      'state':'GEX10 self-verification; independent GEX20 pending','repository':str(ROOT),
      'captured_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
      'vcs_revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
      'product_source':product,'before_files':before,'source_files':after,'before_sha256':digest(before),'source_sha256':digest(after),
      'changed':[row['path'] for row in after if row['path'] in before_map and row != before_map[row['path']]],
      'added':sorted(after_names-set(before_map)),'deleted':sorted(set(before_map)-after_names),
      'records_manifest_sha256':sha(target/'RECORDS.json'),'references_manifest_sha256':sha(target/'REFERENCES.json'),
      'membership_manifest_sha256':sha(target/'MEMBERSHIP.json'),
      'evidence_map':'execution/single_goal/COVERAGE.json','predecessor_impact':'execution/single_goal/PREDECESSOR_IMPACT.json',
      'state_inventory':'execution/single_goal/STATE_INVENTORY.md','gex10':'execution/single_goal/GEX10_RESULT.json',
      'external_actual':'Local Linux normal public CoreApplication and real game_server WebSocket/process/file sampling; no Unity/deployed/all-content maximum assertion.',
      'exclusions':['post_audit/**','other target/**','__pycache__ and build artifacts','unrelated repository members','predecessor target archives are externally verified, not recursively embedded'],
      'archives_and_patches':[{'path':path.name,'bytes':path.stat().st_size,'sha256':sha(path)} for path in sorted(target.iterdir()) if path.is_file()]})
print(json.dumps({'target':str(target),'manifest_sha256':sha(target/'MANIFEST.json'),'product_sha256':product['sha256'],
                  'source_members':len(after),'records_members':len(record_rows),'reference_members':len(ref_rows),'all_writers_stop_after_this_result':True}))
