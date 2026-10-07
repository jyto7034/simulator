from pathlib import Path
import json,hashlib,subprocess,datetime,difflib,re
from urllib.parse import unquote
root=Path('/mnt/f/work/simulator');p=root/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment';old=p/'baseline/close';new=p/'baseline/close_002'
def h(q):return hashlib.sha256(q.read_bytes()).hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=root).decode()
def save(q,x):q.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
assert not new.exists()
m=json.loads((old/'MANIFEST.json').read_text());ver=json.loads((p/'FINAL_VERIFICATION_001.json').read_text());assert ver['result']=='passed';assert h(old/'MANIFEST.json')==ver['preservation']['planning_close_manifest_sha256']
changed={'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md',str((p/'PLAN_HANDOFF.ko.md').relative_to(root)),str((p/'WORKING_POLICY.ko.md').relative_to(root))}
for r in m['files']:
 assert h(root/r['original'])==r['sha256']
 assert h(root/r['path'])==r['sha256'] or r['path'] in changed,r['path']
new.mkdir();(p/'authoring_scripts/r06_close_successor.py').write_bytes(Path('/tmp/r06_close_successor.py').read_bytes())
for name in ['MANIFEST.json','MANIFEST.sha256','GIT_STATE.json','GIT_STATUS.txt','DOCUMENT_CHANGES.patch']:(new/name).touch()
for name in ['FINAL_VERIFICATION_002.json','FINAL_VERIFICATION_002.sha256']:(p/name).touch()
by={r['path']:r for r in m['files']};paths={root/r['path'] for r in m['files']}
paths.update([old/'MANIFEST.json',old/'MANIFEST.sha256',p/'FINAL_VERIFICATION_001.json',p/'FINAL_VERIFICATION_001.sha256',p/'authoring_scripts/r06_close_successor.py'])
rows=[]
for q in sorted(paths):
 name=str(q.relative_to(root));hash=h(q);prior=by.get(name)
 if prior and hash==prior['sha256']:row=prior.copy();row['reused_prior_original']=True
 else:
  dest=new/'originals'/q.relative_to(root);dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(q.read_bytes());row={'path':name,'sha256':hash,'bytes':q.stat().st_size,'original':str(dest.relative_to(root)),'reused_prior_original':False}
 rows.append(row)
patch=''
for name in sorted(changed):
 prev=(root/by[name]['original']).read_text().splitlines(keepends=True);now=(root/name).read_text().splitlines(keepends=True)
 patch+=''.join(difflib.unified_diff(prev,now,fromfile='close001/'+name,tofile='close002/'+name))
(new/'DOCUMENT_CHANGES.patch').write_text(patch)
for q in [new/'GIT_STATE.json',new/'GIT_STATUS.txt',new/'DOCUMENT_CHANGES.patch']:
 dest=new/'originals'/q.relative_to(root);dest.parent.mkdir(parents=True,exist_ok=True);dest.touch()
status=git('status','--porcelain=v1','--untracked-files=all');head=git('rev-parse','HEAD').strip();branch=git('branch','--show-current').strip();(new/'GIT_STATUS.txt').write_text(status)
state=json.loads((old/'GIT_STATE.json').read_text());state.update({'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'branch':branch,'head':head,'status_file':str((new/'GIT_STATUS.txt').relative_to(root)),'status_sha256':h(new/'GIT_STATUS.txt'),'status_rows':len(status.splitlines()),'predecessor':str((old/'GIT_STATE.json').relative_to(root))});save(new/'GIT_STATE.json',state)
for q in [new/'GIT_STATE.json',new/'GIT_STATUS.txt',new/'DOCUMENT_CHANGES.patch']:
 dest=new/'originals'/q.relative_to(root);dest.write_bytes(q.read_bytes());rows.append({'path':str(q.relative_to(root)),'sha256':h(q),'bytes':q.stat().st_size,'original':str(dest.relative_to(root)),'reused_prior_original':False})
newm={**m,'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'branch':branch,'head':head,'files':rows,'predecessor_manifest':{'path':str((old/'MANIFEST.json').relative_to(root)),'sha256':h(old/'MANIFEST.json')},'change':'Three mutable document pointers/status only; all formal artifacts and production/source data preserved.','self_hash':'baseline/close_002/MANIFEST.sha256','excluded_self_and_post_close_evidence':['baseline/close_002/MANIFEST.json','baseline/close_002/MANIFEST.sha256','FINAL_VERIFICATION_002.json','FINAL_VERIFICATION_002.sha256']}
save(new/'MANIFEST.json',newm);(new/'MANIFEST.sha256').write_text(h(new/'MANIFEST.json')+'  MANIFEST.json\n')
for row in rows:
 assert h(root/row['path'])==row['sha256'];assert h(root/row['original'])==row['sha256']
# Revalidate complete current selection metadata and every goal requirement projection remains exact.
assert '- Current Working Revision: `WR-CAVER-P2G20-003`' in (p/'WORKING_POLICY.ko.md').read_text()
assert '**현재 R06 P2G 재검토·계획 인계 완료 / 구현 요청 대기.**' in (root/'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md').read_text()
assert 'FINAL_VERIFICATION_002.json' in (p/'PLAN_HANDOFF.ko.md').read_text()
assert 'baseline/close_002/MANIFEST.json' in (p/'PLAN_HANDOFF.ko.md').read_text()
count=0;missing=[]
for q in [*p.glob('*.md'),root/'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md',root/'core_enhanced/docs/roadmaps/content_authoring/OPERATING_RULES.md',root/'core_enhanced/docs/goals/content_authoring_value_expressions/REQUEST_INTAKE_002.ko.md']:
 for match in re.finditer(r'\[[^\]]*\]\(([^)]+)\)',q.read_text()):
  target=unquote(match.group(1).split('#')[0].strip('<>'))
  if not target or re.match(r'^[a-z]+://',target):continue
  count+=1
  if not (q.parent/target).exists():missing.append([str(q.relative_to(root)),target])
assert not missing,missing
assert git('status','--porcelain=v1','--untracked-files=all')==status
assert git('rev-parse','HEAD').strip()==head
# Reuse standard results only after verifying all prior validator input closures are byte-identical.
receipt=json.loads((p/'P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_001.json').read_text())
for name in ['P2G_30_FREEZE_001.json','P2G_40_PHASE_PLAN_001.json','P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_001.json','P2G_50_GOAL_SET_001.ko.md','P2G_60_HANDOFF_001.json','P2G_20_REVIEW_003.json','P2G_10_POLICY_001.json','P2G_00_INITIAL_001.json']:
 r=by[str((p/name).relative_to(root))];assert h(p/name)==r['sha256']
assert all(x['exit_code']==0 and x['output']['result']=='passed' and not x['output']['failures'] for x in ver['standard_validation_rerun'])
result={**ver,'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'predecessor_verification':{'path':str((p/'FINAL_VERIFICATION_001.json').relative_to(root)),'sha256':h(p/'FINAL_VERIFICATION_001.json')},'post_interruption_audit':{'previous_turn':'progress; source/plan/Handoff and final verifier process completed with exit0. Same session44258 resumed, no task restarted.','changed_documents':sorted(changed),'formal_artifact_and_source_bytes':'unchanged, checked all predecessor461 manifest members and actual original bytes','standard_validator_reuse':'all exact validator input/root artifact bytes unchanged; prior real40/50 passed outputs reused','current_summary':'ROADMAP top/current card/Working Policy/PLAN_HANDOFF now consistently select completed plan and implementation request wait'},'preservation':{**ver['preservation'],'current_source_and_original_hashes_checked':len(rows),'planning_close_manifest_sha256':h(new/'MANIFEST.json'),'planning_close_manifest_path':str((new/'MANIFEST.json').relative_to(root))},'links':{'checked':count,'new_missing':missing,'preexisting_missing_preserved':[]},'git':{'branch':branch,'head':head,'status_stable':True},'result':'passed'}
save(p/'FINAL_VERIFICATION_002.json',result);(p/'FINAL_VERIFICATION_002.sha256').write_text(h(p/'FINAL_VERIFICATION_002.json')+'  FINAL_VERIFICATION_002.json\n')
print(json.dumps({'result':'passed','members':len(rows),'links':count,'manifest_sha256':h(new/'MANIFEST.json'),'current_handoff':'HA-CAVER-P2G60-001','GEX_started':False}))
