import hashlib, json, subprocess, tarfile, tempfile
from pathlib import Path

root=Path('/mnt/f/work/simulator')
here=Path(__file__).resolve().parent
target=root/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/segments/wp02_effect_execution_prefix_001/target/effect_execution_prefix_001'
def digest(b): return hashlib.sha256(b).hexdigest()
def check(p, row):
    b=p.read_bytes()
    assert len(b)==row['bytes'] and digest(b)==row['sha256'],str(p)
def archive(p):
    with tarfile.open(p) as a:
        result={}
        for m in a:
            assert m.isfile() and not Path(m.name).is_absolute() and '..' not in Path(m.name).parts
            assert m.name not in result
            result[m.name]=a.extractfile(m).read()
        return result
raw=(target/'MANIFEST.json').read_bytes()
assert digest(raw)=='f8d854b8ea257a2ea19045caf61434d0592ee2333cfe827edfb5eaac7530b52f'
m=json.loads(raw)
for row in m['artifacts']+[m['predecessor'],m['implementation_baseline'],m['handoff']]:check(root/row['path'],row)
before=archive(target/'SOURCE_BEFORE_EFFECT_EXECUTION_001.tar.gz')
after=archive(target/'SOURCE_AFTER_EFFECT_EXECUTION_001.tar.gz')
assert set(before)==set(after)=={f['path'] for f in m['source']['files']}
for f in m['source']['files']:
    for name,entries in [('before',before),('after',after)]:
        assert digest(entries[f['path']])==f[name]['sha256'] and len(entries[f['path']])==f[name]['bytes']
    check(root/f['path'],f['after'])
evidence=archive(target/'EVIDENCE_AND_HANDOFF_001.tar.gz')
for f in m['evidence']['files']:
    check(root/f['path'],f)
    assert digest(evidence[f['path']])==f['sha256']
restoration={}
for name,entries,expected,patch in [('forward',before,after,'SOURCE_DELTA.patch'),('reverse',after,before,'SOURCE_RESTORE.patch')]:
    destination=Path(tempfile.mkdtemp(prefix='r06-prefix-'+name+'-'))
    archive_path=target/('SOURCE_BEFORE_EFFECT_EXECUTION_001.tar.gz' if name=='forward' else 'SOURCE_AFTER_EFFECT_EXECUTION_001.tar.gz')
    subprocess.run(['tar','-xzf',str(archive_path),'-C',str(destination)],check=True)
    result=subprocess.run(['git','apply','--unsafe-paths',str(target/patch)],cwd=destination,capture_output=True,text=True)
    assert result.returncode==0,result.stderr
    for p,b in expected.items():assert (destination/p).read_bytes()==b,p
    restoration[name]={'files':len(expected),'path':str(destination),'passed':True}
result={'target_sha256':digest(raw),'current_source_files':len(after),'evidence_files_checked':len(m['evidence']['files']),'restoration':restoration}
(here/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
