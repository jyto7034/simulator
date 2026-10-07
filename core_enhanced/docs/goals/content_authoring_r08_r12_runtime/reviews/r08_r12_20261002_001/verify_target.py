"""Read-only independent identity, archive, evidence and isolated restoration audit."""
from pathlib import Path
import json,hashlib,tarfile,tempfile,subprocess,difflib
import run
ROOT=run.ROOT; HERE=run.HERE; OLD=run.OLD; T=OLD/'target/r08_r12_integrated_001'; E=HERE/'evidence'
sha=run.sha
m=json.loads((T/'MANIFEST.json').read_text()); current=run.identity()
frozen={r['path']:r for r in m['source_files']}; actual={n:dict(path=n,bytes=b,sha256=h) for n,b,h in current['files']}
refs=json.loads((T/'REFERENCES.json').read_text())
changed=[n for n in frozen.keys() & actual.keys() if frozen[n]!=actual[n]]
refchanged=[r['path'] for r in refs if not (ROOT/r['path']).is_file() or sha(ROOT/r['path'])!=r['sha256']]
checks={}
for group,base in [('records',ROOT),('archives_and_patches',T)]:
 checks[group]=dict(count=len(m[group]),mismatches=[r['path'] for r in m[group] if not (base/r['path']).is_file() or sha(base/r['path'])!=r['sha256'] or (base/r['path']).stat().st_size!=r['bytes']])
for name,rows in [('SOURCE_BEFORE.tar.gz',m['before_files']),('SOURCE_AFTER.tar.gz',m['source_files']),('RECORDS.tar.gz',m['records']),('REFERENCES.tar.gz',refs)]:
 expected={r['path']:r for r in rows}
 with tarfile.open(T/name) as archive:
  members=archive.getmembers(); assert len(members)==len(expected)
  for member in members:
   assert member.name in expected and (member.isfile() or member.islnk())
   if member.islnk(): assert member.linkname in expected
   data=archive.extractfile(member).read(); r=expected[member.name]
   assert len(data)==r['bytes'] and hashlib.sha256(data).hexdigest()==r['sha256'],(name,member.name)
 checks[name]=dict(count=len(rows),membership_and_bytes_match=True)
 if name=='REFERENCES.tar.gz':
  with tarfile.open(T/name) as archive:
   diffs=[]
   for n in refchanged:
    old=archive.extractfile(n).read().decode().splitlines(True);new=(ROOT/n).read_text().splitlines(True)
    diffs.extend(difflib.unified_diff(old,new,fromfile='frozen/'+n,tofile='current/'+n))
   (E/'reference_delta.patch').write_text(''.join(diffs))
with tempfile.TemporaryDirectory(prefix='r08-r12-independent-restore-') as temp:
 b=Path(temp)
 with tarfile.open(T/'SOURCE_BEFORE.tar.gz') as archive:archive.extractall(b,filter='data')
 def verify(rows):
  ns=sorted(str(p.relative_to(b)) for p in b.rglob('*') if p.is_file());assert ns==sorted(r['path'] for r in rows)
  for r in rows:assert sha(b/r['path'])==r['sha256'] and (b/r['path']).stat().st_size==r['bytes']
 verify(m['before_files'])
 for name,args,rows in [('forward',[],m['source_files']),('reverse',['-R'],m['before_files'])]:
  p=subprocess.run(['patch',*args,'-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=b,capture_output=True)
  (E/f'restore_{name}.log').write_bytes(p.stdout+p.stderr);assert p.returncode==0;verify(rows)
  checks[name]=dict(exit_code=p.returncode,membership_and_bytes_match=True)
pointer=json.loads((OLD/'FINAL_TARGET.json').read_text())
result=dict(frozen_digest=m['source_digest'],current=current,manifest_sha256=sha(T/'MANIFEST.json'),pointer_hash_matches=sha(T/'MANIFEST.json')==pointer['manifest_sha256'],changed=sorted(changed),added=sorted(actual.keys()-frozen.keys()),missing=sorted(frozen.keys()-actual.keys()),reference_delta=refchanged,checks=checks,source_unchanged_during_verification=current==run.identity())
with (HERE/'AUDIT_TARGET.json').open('x') as f:json.dump(result,f,ensure_ascii=False,indent=2)
print(json.dumps({k:v for k,v in result.items() if k!='current'},indent=2))
