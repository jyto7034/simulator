"""Seal new audit evidence, while proving product and predecessor records stayed unchanged."""
from pathlib import Path
import json,subprocess,datetime,re
import run
H=run.HERE;T=run.OLD/'target/r08_r12_integrated_001'
def put(name,data):
 with (H/name).open('x') as f:json.dump(data,f,ensure_ascii=False,indent=2);f.write('\n')
target=json.loads((H/'AUDIT_TARGET.json').read_text());now=run.identity();assert now==target['current']
prior=json.loads((T/'MANIFEST.json').read_text())
for group,base in [('records',run.ROOT),('archives_and_patches',T)]:
 for r in prior[group]:assert run.sha(base/r['path'])==r['sha256'] and (base/r['path']).stat().st_size==r['bytes']
assert run.sha(T/'MANIFEST.json')==target['manifest_sha256']
for p in H.glob('*.md'):
 for link in re.findall(r'\]\(([^)]+)\)',p.read_text()):
  if '://' not in link:assert (p.parent/link.split('#')[0]).exists(),(p,link)
for name,args in [('GIT_STATUS_AT_AUDIT_CLOSE.txt',['status','--short']),('INDEX_DIFF_AT_AUDIT_CLOSE.patch',['diff','--cached','--binary'])]:
 with (H/name).open('xb') as f:subprocess.run(['git',*args],cwd=run.ROOT,stdout=f,check=True)
put('AUDIT_CURRENTNESS_AT_CLOSE.json',dict(current_digest=now['digest'],files=now['file_count'],unchanged_from_entry=True,previous_records_verified=len(prior['records']),previous_archives_patches_verified=len(prior['archives_and_patches']),old_manifest_unchanged=True,product_mutation=False))
files=[]
for p in sorted(H.rglob('*')):
 if p.is_file() and '__pycache__' not in p.parts and p.name not in ('AUDIT_MANIFEST.json','AUDIT_CLOSURE.json'):
  files.append(dict(path=str(p.relative_to(run.ROOT)),bytes=p.stat().st_size,sha256=run.sha(p)))
put('AUDIT_MANIFEST.json',dict(schema=1,owner='independent GEX-20 R08-R12 audit evidence',files=files,source_digest=now['digest'],source_target_sha256=run.sha(H/'AUDIT_TARGET.json'),exclusions=['AUDIT_MANIFEST.json (self hash)','AUDIT_CLOSURE.json (external pointer)','__pycache__ (interpreter cache)']))
for r in files:assert run.sha(run.ROOT/r['path'])==r['sha256']
put('AUDIT_CLOSURE.json',dict(schema=1,closed_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),goal='GS-CARX-001 / GOAL-CARX-01 / HA-CARX-001',stage='GEX-20',verdict='Not Complete',blocking_findings=['F01'],reviewer_participated_in_product_implementation=False,frozen_source_digest=target['frozen_digest'],current_source_digest=now['digest'],currentness='current / proven non-impacting ROADMAP_STATUS-only delta',condition_count=55,default_tests_passed=1677,feature_tests_passed=1677,x09_environment_limitation_resolved=True,cancellation_oracle_cases=5,cancellation_oracle_passed=2,cancellation_oracle_failed=3,product_modified=False,required_next_owner='GOAL-CARX-01 R08/R10 correction GEX-10 and fresh independent GEX-20',r13_r16_implementation_started=False,r13_r16_final_target_frozen=False,r13_r16_independent_audit_performed=False,manifest=dict(path=str((H/'AUDIT_MANIFEST.json').relative_to(run.ROOT)),sha256=run.sha(H/'AUDIT_MANIFEST.json'),file_count=len(files))))
print(json.dumps(dict(verdict='Not Complete',finding='F01',source=now['digest'],files_sealed=len(files),manifest_sha256=run.sha(H/'AUDIT_MANIFEST.json'))))
