"""Read-only WP01 exact currentness and before/after archive verification."""
import json, hashlib, tarfile, importlib.util, io, contextlib
from pathlib import Path
from datetime import datetime, timezone
ROOT=Path('/mnt/f/work/simulator')
TOPIC=ROOT/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment'
TARGET=TOPIC/'execution/single_goal/segments/wp01_content_input_compile_001/target/target_001/MANIFEST.json'
sha=lambda data:hashlib.sha256(data).hexdigest()
assert sha(TARGET.read_bytes())=='6f55cfa0fae0d88971eba6cb3c47e39ab5d12a2e99b2656a40eb2b51e7cf76de'
m=json.loads(TARGET.read_text())
b=json.loads((ROOT/m['baseline']['manifest']).read_text())
spec=importlib.util.spec_from_file_location('v',TOPIC/'authoring_scripts/r06_wp01_verify_target_001.py')
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
        for value in x.values():scan(value)
    elif isinstance(x,list):
        for value in x:scan(value)
scan(m)
archives=[]
for label,row,rows in [('before',b['source']['archive'],b['source']['files']),('after',m['current_source']['archive'],m['current_source']['files']),('records',m['records_archive'],m['records']),('evidence',m['evidence_archive'],m['evidence'])]:
    assert sha((ROOT/row['path']).read_bytes())==row['sha256']
    expected={r['path']:r for r in rows}
    with tarfile.open(ROOT/row['path'],'r:gz') as a:
        members=[x for x in a.getmembers() if x.isfile()]
        assert len(members)==len(expected)==len({x.name for x in members})
        assert set(expected)=={x.name for x in members}
        for f in members:
            data=a.extractfile(f).read()
            assert sha(data)==expected[f.name]['sha256'] and len(data)==expected[f.name]['bytes'],(label,f.name)
    archives.append({'label':label,'files':len(rows)})
old={r['path']:r['sha256'] for r in b['source']['files']}
new={r['path']:r['sha256'] for r in m['current_source']['files']}
changes={'added':sorted(new.keys()-old.keys()),'deleted':sorted(old.keys()-new.keys()),'modified':sorted(p for p in old.keys()&new.keys() if old[p]!=new[p])}
for kind,paths in changes.items():
    assert set(paths)=={r['path'] for r in m['current_source']['changes'] if r['state']==kind}
scope=set()
for path in m['current_source']['scope']:
    f=ROOT/path
    if f.is_file():scope.add(path)
    elif f.is_dir():scope.update(str(x.relative_to(ROOT)) for x in f.rglob('*') if x.is_file())
assert scope==set(new),{'extra':sorted(scope-set(new)),'missing':sorted(set(new)-scope)}
print(json.dumps({'checked_at':datetime.now(timezone.utc).isoformat(),'target':m['record'],'manifest_sha256':sha(TARGET.read_bytes()),'hash_checks':len(checks),'archives':archives,'changes':changes,'implementer_verifier':json.loads(out.getvalue()),'passed':True},ensure_ascii=False,indent=2))
