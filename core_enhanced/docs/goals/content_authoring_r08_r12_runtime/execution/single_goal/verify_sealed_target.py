"""Read back all frozen bytes; create only the external final-target pointer."""
import hashlib,json,tarfile
from pathlib import Path
import run
H=Path(__file__).resolve().parent;ROOT=run.ROOT;T=H/'target/r08_r12_integrated_001'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
M=json.loads((T/'MANIFEST.json').read_text());current=run.identity()
assert current['digest']==M['source_digest']
for group,base in [('source_files',ROOT),('records',ROOT),('archives_and_patches',T)]:
 for r in M[group]:
  p=base/r['path'];assert p.stat().st_size==r['bytes'] and sha(p)==r['sha256'],(group,r['path'])
refs=json.loads((T/'REFERENCES.json').read_text())
for r in refs:
 p=ROOT/r['path'];assert p.stat().st_size==r['bytes'] and sha(p)==r['sha256'],r['path']
for name,rows in [('SOURCE_BEFORE.tar.gz',M['before_files']),('SOURCE_AFTER.tar.gz',M['source_files']),('RECORDS.tar.gz',M['records']),('REFERENCES.tar.gz',refs)]:
 expected={r['path']:r for r in rows}
 with tarfile.open(T/name) as archive:
  members=archive.getmembers();assert len(members)==len(expected)
  for member in members:
   assert member.isfile() and member.name in expected
   data=archive.extractfile(member).read();r=expected[member.name]
   assert len(data)==r['bytes'] and hashlib.sha256(data).hexdigest()==r['sha256'],(name,member.name)
restore=json.loads((T/'RESTORE_VERIFICATION.json').read_text())
assert restore['after_digest']==current['digest'] and restore['forward_membership_and_bytes_match'] and restore['reverse_membership_and_bytes_match']
assert M['source_files']==[{'path':a,'bytes':b,'sha256':c} for a,b,c in current['files']]
result={'status':'implementation_and_self_validation_frozen_with_one_environment_limitation_independent_audit_pending','manifest_path':str((T/'MANIFEST.json').relative_to(ROOT)),'manifest_sha256':sha(T/'MANIFEST.json'),'source_digest':M['source_digest'],'source_file_count':current['file_count'],'self_verified_acceptances':54,'environment_limited_acceptances':1,'independent_audit':False,'sealed_verification':{'source_membership_and_bytes_current':True,'records_current':len(M['records']),'references_current':len(refs),'all_archives_and_patches_hashes_match':True,'all_four_archive_membership_and_contents_match':True,'isolated_forward_reverse_fuzz0':True},'pointer_excluded_from_manifest_self_hash':True}
with (H/'FINAL_TARGET.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps(result,indent=2))
