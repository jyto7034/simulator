from pathlib import Path
import json,hashlib,tarfile,tempfile,subprocess,datetime
root=Path('/mnt/f/work/simulator')
r=root/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_eep_f01_f03_correction_001_review_001'
c=r.parent.parent/'corrections/wp02_eep_f01_f03_001'
m=json.loads((c/'target/correction_001/MANIFEST.json').read_text())
b=json.loads((c/'baseline/entry_001/MANIFEST.json').read_text())
sha=lambda v:hashlib.sha256(v).hexdigest()
for f in b['source']['files']:
 assert sha((root/f['path']).read_bytes())==next(x['after']['sha256'] for x in m['source']['files'] if x['path']==f['path'])
with tarfile.open(root/b['source']['archive']['path']) as t:
 for f in b['source']['files']:
  data=t.extractfile(f['path']).read()
  assert sha(data)==f['sha256'],f
with tarfile.open(c/'target/correction_001/EVIDENCE_AND_HANDOFF.tar.gz') as t:
 assert len(t.getmembers())==len(m['records'])
 for f in m['records']:
  data=t.extractfile(f['path']).read()
  assert len(data)==f['bytes'] and sha(data)==f['sha256']
iso=Path(tempfile.mkdtemp(prefix='r06-eep-independent-'))
with tarfile.open(root/m['after_archive']['path']) as t:
 for f in m['source']['files']:
  data=t.extractfile(f['path']).read()
  assert sha(data)==f['after']['sha256'] and len(data)==f['after']['bytes']
  p=iso/f['path'];p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data)
config=dict(root=str(root),review_dir=str(r),correction_dir=str(c),isolate=str(iso),target_dir=str(iso/'build'),source=m['source']['files'])
(r/'isolate.json').write_text(json.dumps(config,indent=2)+'\n')
(r/'independent_archive_verification.json').write_text(json.dumps(dict(baseline_files_verified=len(b['source']['files']),target_files_verified=len(m['source']['files']),evidence_archive_members_verified=len(m['records']),time=datetime.datetime.now(datetime.timezone.utc).isoformat()),indent=2)+'\n')
print(iso)
