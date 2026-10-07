"""Freeze one reviewable U07 target with exact forward/reverse source restoration."""
import datetime,hashlib,json,subprocess,sys,tarfile,tempfile
from pathlib import Path
from run import ROOT,HERE,identity,sha
from finalization_paths import paths
_, final_names = paths(sys.argv[2] if len(sys.argv)>2 else '001')
S=HERE/'execution/single_goal';label=sys.argv[1] if len(sys.argv)>1 else 'u07_001';T=S/'target'/label
assert label.startswith('u07_')
selection=json.loads((S/final_names['selection']).read_text())
assert selection['finalization']==final_names and selection['target']==label
def write(path,value):
 with path.open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
def rows(root,names):return [{'path':n,'bytes':(root/n).stat().st_size,'sha256':sha(root/n)} for n in sorted(names)]
def digest(value):return hashlib.sha256(json.dumps(value,sort_keys=True).encode()).hexdigest()
def unpack(path,root):
 with tarfile.open(path) as archive:archive.extractall(root,filter='data')
def pack(name,paths):
 entries=rows(ROOT,[str(p.relative_to(ROOT)) for p in paths])
 with tarfile.open(T/(name+'.tar.gz'),'x:gz') as archive:
  for row in entries:archive.add(ROOT/row['path'],arcname=row['path'],recursive=False)
 write(T/(name+'.json'),entries)
 with tempfile.TemporaryDirectory(prefix='u06-record-restore-') as temp:
  unpack(T/(name+'.tar.gz'),temp);base=Path(temp)
  assert rows(base,[str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()])==entries
 return entries
product=identity()
for name in [final_names[key] for key in ('coverage','impact','gex')]:
 assert json.loads((S/name).read_text())['source_sha256']==product['sha256'],name
T.mkdir(parents=True,exist_ok=False)
before=json.loads((S/'baseline/SOURCE_BEFORE.json').read_text())['files']
supplement=S/'baseline_supplement_001';preimage=json.loads((supplement/'PREIMAGE_RECORD.json').read_text());prepath=supplement/preimage['path']
assert preimage['roundtrip_equal'] and sha(prepath)==preimage['preimage_sha256']
assert preimage['path'] not in {r['path'] for r in before}
before.append({'path':preimage['path'],'bytes':preimage['preimage_bytes'],'sha256':preimage['preimage_sha256']})
before=sorted(before,key=lambda r:r['path']);old={r['path']:r for r in before}
names={r['path'] for r in before}|{r['path'] for r in product['files']};after=rows(ROOT,[n for n in names if (ROOT/n).is_file()]);after_names={r['path'] for r in after}
with tempfile.TemporaryDirectory(prefix='u06-source-restore-') as temp:
 b,a=Path(temp)/'before',Path(temp)/'after';b.mkdir();a.mkdir()
 unpack(S/'baseline/SOURCE_BEFORE.tar.gz',b)
 destination=b/preimage['path'];destination.parent.mkdir(parents=True,exist_ok=True);destination.write_bytes(prepath.read_bytes())
 assert rows(b,old)==before
 with tarfile.open(T/'SOURCE_BEFORE.tar.gz','x:gz') as archive:
  for row in before:archive.add(b/row['path'],arcname=row['path'],recursive=False)
 with tarfile.open(T/'SOURCE_AFTER.tar.gz','x:gz') as archive:
  for row in after:archive.add(ROOT/row['path'],arcname=row['path'],recursive=False)
 unpack(T/'SOURCE_AFTER.tar.gz',a);assert rows(a,after_names)==after
 patches=[]
 for n in sorted(names):
  result=subprocess.run(['diff','-u','--label','a/'+n if n in old else '/dev/null','--label','b/'+n if n in after_names else '/dev/null',str(b/n) if n in old else '/dev/null',str(a/n) if n in after_names else '/dev/null'],capture_output=True)
  assert result.returncode in (0,1);patches.append(result.stdout)
 (T/'FORWARD.patch').write_bytes(b''.join(patches))
 for reverse,expected in [(False,after),(True,before)]:
  result=subprocess.run(['patch',*(['-R'] if reverse else []),'-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=b,capture_output=True)
  (T/('RESTORE_REVERSE.log' if reverse else 'RESTORE_FORWARD.log')).write_bytes(result.stdout+result.stderr)
  assert result.returncode==0
  actual=sorted(str(p.relative_to(b)) for p in b.rglob('*') if p.is_file());assert actual==[r['path'] for r in expected];assert rows(b,actual)==expected
write(T/'RESTORE_VERIFICATION.json',{'before_sha256':digest(before),'after_sha256':digest(after),'forward_membership_and_bytes_match':True,'reverse_membership_and_bytes_match':True,'patch_fuzz':0,'supplemented_preimage':preimage,'initial_archive_unchanged':True})
records=sorted(p for p in HERE.rglob('*') if p.is_file() and not {'target','post_audit','__pycache__'}&set(p.relative_to(HERE).parts))
record_rows=pack('RECORDS',records)
prior=ROOT/'core_enhanced/docs/goals/core_battle_starter_tactics';prior_target=prior/'execution/single_goal/target/u06_002'
refs=set()
for relative in ['core_enhanced/docs/policy_to_goal_pipeline_revised_v2.md','core_enhanced/docs/goal_execution_pipeline(1).md','core_enhanced/docs/game_rulebook.md','core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md','core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md','core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md','core_enhanced/docs/roadmaps/core_first/execution/HANDOFF_20261005.ko.md','core_enhanced/docs/roadmaps/core_first/execution/HANDOFF_20261005_002.ko.md','core_enhanced/docs/roadmaps/core_first/execution/u07_pause_20261005_001/STATE.json','core_enhanced/docs/roadmaps/core_first/execution/u07_pause_20261005_001/PRODUCT_CURRENT.json','core_enhanced/docs/roadmaps/core_first/execution/u07_pause_20261005_001/UNVERIFIED_CORRECTION.patch','core_enhanced/docs/roadmaps/core_first/execution/u07_pause_20261005_001/PUBLICATION_RECORD.json','core_enhanced/docs/roadmaps/core_first/execution/u07_memory_budget_followup_20261005_001/RECORD.ko.md','core_enhanced/docs/roadmaps/core_first/execution/u07_memory_budget_followup_20261005_001/FOLLOWUP.ko.md','core_enhanced/docs/roadmaps/core_first/execution/u07_memory_budget_followup_20261005_001/PUBLICATION_RECORD.json']:
 refs.add(ROOT/relative)
for folder in [ROOT/'core_enhanced/docs/game_rules',ROOT/'core_enhanced/docs/roadmaps/core_first/execution/workflow_efficiency_update_20261004_001',ROOT/'core_enhanced/docs/roadmaps/core_first/execution/u06_resume_20261005_001',prior/'post_audit/u06_20261005_002']:
 refs.update(p for p in folder.rglob('*') if p.is_file() and '__pycache__' not in p.parts)
for n in ['P2G_30_FREEZE_001.json','execution/single_goal/COVERAGE.json','execution/single_goal/PREDECESSOR_IMPACT.json','execution/single_goal/SERVICE_REUSE.json']:refs.add(prior/n)
refs.update(p for p in prior_target.iterdir() if p.is_file() and p.suffix!='.gz')
external=[];manifest=json.loads((prior_target/'MANIFEST.json').read_text())
for name,entries in [('SOURCE_BEFORE.tar.gz',manifest['before_files']),('SOURCE_AFTER.tar.gz',manifest['source_files']),('RECORDS.tar.gz',json.loads((prior_target/'RECORDS.json').read_text())),('REFERENCES.tar.gz',json.loads((prior_target/'REFERENCES.json').read_text()))]:
 path=prior_target/name;expected={r['path']:r for r in entries};seen=set()
 with tarfile.open(path) as archive:
  for member in archive:
   assert member.isfile() and member.name in expected and member.name not in seen
   row=expected[member.name];assert member.size==row['bytes'];h=hashlib.sha256()
   with archive.extractfile(member) as stream:
    while chunk:=stream.read(1024*1024):h.update(chunk)
   assert h.hexdigest()==row['sha256'];seen.add(member.name)
 assert seen==set(expected)
 external.append({'path':str(path.relative_to(ROOT)),'sha256':sha(path),'bytes':path.stat().st_size,'member_count':len(seen),'exact_member_bytes_verified':True})
# A correction target retains the previous U07 target and closed audit verbatim.
for previous in sorted(T.parent.glob('u07_*')):
 if previous==T or not (previous/'MANIFEST.json').is_file():continue
 prior_manifest=json.loads((previous/'MANIFEST.json').read_text())
 refs.update(p for p in previous.iterdir() if p.is_file() and p.suffix!='.gz')
 for name,entries in [('SOURCE_BEFORE.tar.gz',prior_manifest['before_files']),('SOURCE_AFTER.tar.gz',prior_manifest['source_files']),('RECORDS.tar.gz',json.loads((previous/'RECORDS.json').read_text())),('REFERENCES.tar.gz',json.loads((previous/'REFERENCES.json').read_text()))]:
  path=previous/name;expected={r['path']:r for r in entries};seen=set()
  with tarfile.open(path) as archive:
   for member in archive:
    assert member.isfile() and member.name in expected and member.name not in seen
    row=expected[member.name];assert member.size==row['bytes'];h=hashlib.sha256()
    with archive.extractfile(member) as stream:
     while chunk:=stream.read(1024*1024):h.update(chunk)
    assert h.hexdigest()==row['sha256'];seen.add(member.name)
  assert seen==set(expected)
  external.append({'path':str(path.relative_to(ROOT)),'sha256':sha(path),'bytes':path.stat().st_size,'member_count':len(seen),'exact_member_bytes_verified':True})
for audit in (HERE/'post_audit').glob('*'):
 if (audit/'AUDIT_CLOSURE.json').is_file():refs.update(p for p in audit.rglob('*') if p.is_file() and '__pycache__' not in p.parts)
write(T/'EXTERNAL_ARCHIVES.json',external)
impact=json.loads((S/final_names['impact']).read_text())
for row in impact['original_requirement_sources']:
 assert sha(ROOT/row['path'])==row['sha256'];refs.add(ROOT/row['path'])
service=json.loads((S/final_names['service']).read_text())
for key in ['reused_audit','reused_service_evidence','reused_master_source_review']:refs.add(ROOT/service[key])
tool=ROOT/'core_enhanced/tools/runtime_architecture_final_validation';refs.update(p for p in tool.rglob('*') if p.is_file() and 'target' not in p.relative_to(tool).parts)
for relative in ['core_enhanced/tools/runtime_architecture_preservation/src/main.rs', 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/baseline/SOURCE_BEFORE.tar.gz', 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/evidence/P01_LOG_V32_BASELINE_001.json', 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/evidence/LOG_VERSION_MIGRATION_001.json', 'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json', 'core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal/evidence/r05_raw_integrated_003.json']:
 refs.add(ROOT/relative)
assert all(p.is_file() for p in refs),[str(p) for p in refs if not p.is_file()]
ref_rows=pack('REFERENCES',sorted(refs))
tracked=set(subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode().split('\0'));dirty=set(subprocess.check_output(['git','diff','--name-only','-z','HEAD'],cwd=ROOT).decode().split('\0'))
membership={}
for group,entries in [('source',after),('records',record_rows),('references',ref_rows),('external_reference',external)]:
 for row in entries:
  p=row['path'];item=membership.setdefault(p,dict(row,groups=[],kind='tracked/dirty' if p in dirty else 'tracked/clean' if p in tracked else 'untracked'));item['groups'].append(group)
write(T/'MEMBERSHIP.json',[membership[p] for p in sorted(membership)])
assert identity()==product and rows(ROOT,after_names)==after
result={'schema':1,'goal':'GOAL-CIOB-01','goal_set':'GS-CIOB-001','phase':'PHASE-CIOB-01','state':'GEX-10 self-verification; independent GEX-20 pending','repository':str(ROOT),'vcs_revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'captured_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'product_source':product,'before_files':before,'source_files':after,'before_sha256':digest(before),'source_sha256':digest(after),'changed':[r['path'] for r in after if old.get(r['path'])!=r],'added':sorted(after_names-set(old)),'deleted':sorted(set(old)-after_names),'records_manifest_sha256':sha(T/'RECORDS.json'),'references_manifest_sha256':sha(T/'REFERENCES.json'),'membership_manifest_sha256':sha(T/'MEMBERSHIP.json'),'evidence_map':'execution/single_goal/'+final_names['coverage'],'predecessor_impact':'execution/single_goal/'+final_names['impact'],'state_inventory':'execution/single_goal/STATE_INVENTORY.md','external_actual':'Local Linux CoreApplication, real actor and stock WebSocket/process/file probes; no Unity or deployed service assertion. Original limits and only inherited exceptions apply.','exclusions':['post_audit/** audit output','target/** outside this explicit snapshot','__pycache__ and build artifacts','Unrelated repository content outside explicit membership','Large predecessor archives verified member-by-member and referenced, not recursively embedded'],'archives_and_patches':[{'path':p.name,'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(T.iterdir()) if p.is_file()]}
write(T/'MANIFEST.json',result);print(json.dumps({'target':str(T),'source_files':len(after),'records':len(record_rows),'references':len(ref_rows),'product':product['sha256']}))
