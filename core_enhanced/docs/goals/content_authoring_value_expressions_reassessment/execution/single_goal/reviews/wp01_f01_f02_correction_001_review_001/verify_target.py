"""Independent exact-source delta and archive checks; writes only isolated restored bytes."""
import hashlib, json, tarfile, tempfile, subprocess, importlib.util, io, contextlib
from pathlib import Path
from datetime import datetime, timezone
ROOT=Path('/mnt/f/work/simulator')
TOPIC=ROOT/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment'
TARGET=TOPIC/'execution/single_goal/corrections/wp01_target_001_f01_f02/target/correction_001/MANIFEST.json'
sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha(TARGET.read_bytes())=='679336ad025f9dfaa6b44345ca3e494fd694e5903d49786a91d0ff270dbd642f'
m=json.loads(TARGET.read_text())
b=json.loads((ROOT/m['pre_fix_baseline']['manifest']).read_text())
p=json.loads((ROOT/m['predecessor_target']['manifest']).read_text())
spec=importlib.util.spec_from_file_location('v',TOPIC/'authoring_scripts/r06_wp01_verify_correction_target_001.py')
v=importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)
out=io.StringIO()
with contextlib.redirect_stdout(out):v.main()
checks=[]
def scan(x):
    if isinstance(x,dict):
        for pk,hk in [('path','sha256'),('manifest','manifest_sha256')]:
            if pk in x and hk in x:
                assert sha((ROOT/x[pk]).read_bytes())==x[hk],x[pk]
                checks.append(x[pk])
        for val in x.values():scan(val)
    elif isinstance(x,list):
        for val in x:scan(val)
scan(m)
old={r['path']:r['sha256'] for r in b['source']['files']}
new={r['path']:r['sha256'] for r in m['current_source']['files']}
assert old=={r['path']:r['sha256'] for r in p['current_source']['files']}
assert old.keys()==new.keys()
changed=sorted(path for path in old if old[path]!=new[path])
assert changed==sorted(r['path'] for r in m['current_source']['changes_from_pre_fix']['files'])
assert len(changed)==4
archives=[]
for label,section in [('before',b['source']),('after',m['current_source']),('before_documents',b['existing_execution_documents']),('review_inputs',b['review_inputs']),('predecessor_records',b['predecessor_target_records']),('records',m['execution_records']),('evidence',m['evidence'])]:
    expected={r['path']:r for r in section['files']}
    with tarfile.open(ROOT/section['archive']['path'],'r:gz') as a:
        members=a.getmembers()
        assert all(x.isfile() for x in members)
        assert len(members)==len(expected)==len({x.name for x in members})
        assert set(expected)=={x.name for x in members}
        for item in members:
            data=a.extractfile(item).read()
            assert sha(data)==expected[item.name]['sha256']
            assert len(data)==expected[item.name]['bytes']
    archives.append({'label':label,'count':len(expected)})
# Restore only in a fresh temporary directory, never in the working tree.
scratch=Path(tempfile.mkdtemp(prefix='r06-wp01-delta-review-'))
with tarfile.open(ROOT/b['source']['archive']['path'],'r:gz') as a:
    for member in a.getmembers():
        assert member.isfile() and not Path(member.name).is_absolute() and '..' not in Path(member.name).parts
    a.extractall(scratch,filter='data')
patch=ROOT/m['current_source']['recoverable_delta']['path']
for args in [['git','apply','--check','-p4',str(patch)],['git','apply','-p4',str(patch)]]:
    r=subprocess.run(args,cwd=scratch,capture_output=True,text=True)
    assert r.returncode==0,{'command':args,'stdout':r.stdout,'stderr':r.stderr}
restored={str(f.relative_to(scratch)):sha(f.read_bytes()) for f in scratch.rglob('*') if f.is_file()}
assert restored==new
print(json.dumps({'checked_at':datetime.now(timezone.utc).isoformat(),'manifest_sha256':sha(TARGET.read_bytes()),'implementer_verifier':json.loads(out.getvalue()),'hash_checks':len(checks),'archives':archives,'changed_files':changed,'baseline_matches_predecessor':True,'delta_applied_and_restored_exactly':True,'restored_files':len(restored),'isolated_restore_path':str(scratch),'passed':True},ensure_ascii=False,indent=2))
