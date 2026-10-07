import json,hashlib,tarfile,datetime
from pathlib import Path
H=Path('core_enhanced/docs/goals/core_relic_numeric_variants');T=H/'execution/single_goal/target/u08_current_012';A=H/'post_audit/u08_current_012_pass001'
def sha(p):
 h=hashlib.sha256()
 with open(p,'rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()
m=json.loads((T/'MANIFEST.json').read_text());bad=[];seen={}
for r in m['source_files']+m['product_source']['files']+json.loads((T/'RECORDS.json').read_text())+json.loads((T/'REFERENCES.json').read_text()):
 p=r['path']
 if p in seen and seen[p]!=r['sha256']:bad.append(['conflicting_ref',p])
 seen[p]=r['sha256']
for p,s in seen.items():
 if not Path(p).is_file() or sha(p)!=s:bad.append(['current',p])
for r in m['archives_and_patches']:
 if sha(T/r['path'])!=r['sha256']:bad.append(['target_member',r['path']])
archives=[]
for filename,rows in [('SOURCE_BEFORE.tar.gz',m['before_files']),('SOURCE_AFTER.tar.gz',m['source_files']),('RECORDS.tar.gz',json.loads((T/'RECORDS.json').read_text())),('REFERENCES.tar.gz',json.loads((T/'REFERENCES.json').read_text()))]:
 expected={r['path']:r for r in rows};actual={}
 with tarfile.open(T/filename) as tf:
  for e in tf:
   if e.isfile():actual[e.name]={'sha256':hashlib.sha256(tf.extractfile(e).read()).hexdigest(),'bytes':e.size}
 if set(expected)!=set(actual):bad.append(['archive_membership',filename])
 for p,r in expected.items():
  if p not in actual or any(actual[p][k]!=r[k] for k in ['sha256','bytes']):bad.append(['archive_bytes',filename,p])
 archives.append({'path':filename,'count':len(actual)})
out={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'manifest_sha256':sha(T/'MANIFEST.json'),'config_sha256':sha(m['config']['path']),'source_declared':m['source_sha256'],'current_unique_files':len(seen),'archives':archives,'failures':bad}
(A/'ENTRY_IDENTITY.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
