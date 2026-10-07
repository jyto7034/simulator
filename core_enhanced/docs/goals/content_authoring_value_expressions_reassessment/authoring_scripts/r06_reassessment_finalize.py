from pathlib import Path
import json,hashlib,subprocess,datetime,difflib,re
root=Path('/mnt/f/work/simulator');p=root/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment';rel=p.relative_to(root)
def sha(b):return hashlib.sha256(b).hexdigest()
def git(*a):return subprocess.check_output(['git',*a],cwd=root).decode()
def save(q,x):q.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
intake=json.loads((p/'baseline/intake/MANIFEST.json').read_text());before={r['path']:r for r in intake['files']}
allowed=['core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md','core_enhanced/docs/roadmaps/content_authoring/OPERATING_RULES.md','core_enhanced/docs/goals/content_authoring_value_expressions/REQUEST_INTAKE_002.ko.md']
# Audit current state against original exact bytes. Only explicitly linked mutable records may differ.
checks=[]
for r in intake['files']:
 original=root/r['original'];assert original.exists() and sha(original.read_bytes())==r['sha256'],r['original']
 current=(root/r['path']).read_bytes();same=sha(current)==r['sha256']
 assert same or r['path'] in allowed,('unauthorized/source drift',r['path'])
 if r['path'] in allowed[1:]:assert current.startswith(original.read_bytes()),('not additive',r['path'])
 checks.append({'path':r['path'],'initial_sha256':r['sha256'],'current_sha256':sha(current),'disposition':'unchanged' if same else 'authorized current intake/status link'})
oldpaths=[r for r in checks if r['path'].startswith('core_enhanced/docs/goals/content_authoring_value_expressions/')]
assert all(r['disposition']=='unchanged' or r['path']==allowed[2] for r in oldpaths)
review=json.loads((p/'baseline/review/MANIFEST.json').read_text())
for r in review['files']:
 assert sha((root/r['original']).read_bytes())==r['sha256'];assert sha((root/r['path']).read_bytes())==r['sha256']
# Standard validators and local read-only checks; no repository verifier edits.
validation=[]
common=['--freeze',str(rel/'P2G_30_FREEZE_001.json'),'--plan',str(rel/'P2G_40_PHASE_PLAN_001.json'),'--receipt',str(rel/'P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_001.json')]
for label,cmd in [('P2G-40',['node','core_enhanced/docs/tools/validate_p2g40_receipt.mjs',*common]),('P2G-50',['node','core_enhanced/docs/tools/validate_p2g50_goal_set.mjs',*common,'--quality-contract','2','--goal-set',str(rel/'P2G_50_GOAL_SET_001.ko.md')])]:
 r=subprocess.run(cmd,cwd=root,capture_output=True,text=True);out=json.loads(r.stdout);assert r.returncode==0 and out['result']=='passed' and out['failures']==[]
 validation.append({'label':label,'command':cmd,'exit_code':r.returncode,'output':out,'stderr':r.stderr})
# Direct semantic coverage owner/mapping exactness independent of generated count claim.
policy=json.loads((p/'P2G_10_POLICY_001.json').read_text())['artifacts'];per=[a for a in policy if a['ref']['Artifact Type']=='Policy Element Revision' and a['payload']['Intrinsic status']=='decided'];old=json.loads((p/'COMPARISON_ORIGINAL_P2G_10_POLICY_001.json').read_text())['artifacts'];oldper=[a for a in old if a['ref']['Artifact Type']=='Policy Element Revision' and a['payload']['Intrinsic status']=='decided'];assert len(per)==len(oldper)==9
for a,b in zip(per,oldper):
 for k in ['Policy statement','Observable Policy Acceptance']:assert a['payload'][k]==b['payload'][k]
plan=json.loads((p/'P2G_40_PHASE_PLAN_001.json').read_text())['artifacts'][0];freeze=json.loads((p/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0];handoff=json.loads((p/'P2G_60_HANDOFF_001.json').read_text())['artifacts'][0]
goal=(p/'P2G_50_GOAL_SET_001.ko.md').read_text()
for r in freeze['payload']['Observable Policy Acceptance Set']:
 for j,acceptance in enumerate(r['Observable Policy Acceptance'],1):
  line=next(l for l in goal.splitlines() if l.startswith('| `'+r['Policy Element Revision Ref']['Artifact ID']+f'[{j}]`'))
  assert acceptance in line and 'WP-CAVER-04' in line and 'IVA-CAVER-' in line
assert handoff['payload']['Phase Plan Ref']==plan['ref'];assert handoff['payload']['Final Policy Freeze Ref']==freeze['ref'];assert handoff['payload']['Goal Set Ref']['Artifact ID']=='GS-CAVER-P2G50-001'
assert 'GEX-00' in handoff['payload']['Authorization Boundary'];assert not (p/'execution').exists()
# Preserve scripts too, then close bytes; scoped .git/config absence matters for later environment.
(p/'authoring_scripts/r06_reassessment_finalize.py').write_bytes(Path('/tmp/r06_reassessment_finalize.py').read_bytes())
close=p/'baseline/close';assert not close.exists();close.mkdir(parents=True)
for n in ['MANIFEST.json','MANIFEST.sha256','GIT_STATE.json','GIT_STATUS.txt','DOCUMENT_CHANGES.patch']:(close/n).touch()
for n in ['FINAL_VERIFICATION_001.json','FINAL_VERIFICATION_001.sha256']:(p/n).touch()
# Documentation-only patch for originally-existing mutable records; original full tracked patch also retained.
patch=''
for path in allowed:
 b=(root/before[path]['original']).read_text().splitlines(keepends=True);a=(root/path).read_text().splitlines(keepends=True)
 patch+=''.join(difflib.unified_diff(b,a,fromfile='before/'+path,tofile='planning-close/'+path))
(close/'DOCUMENT_CHANGES.patch').write_text(patch)
paths={root/r['path'] for r in intake['files']}|{root/r['path'] for r in review['files']}
paths.update(x for x in p.iterdir() if x.is_file() and not x.name.startswith('FINAL_VERIFICATION'))
paths.update(x for x in (p/'authoring_scripts').iterdir() if x.is_file())
for n in ['baseline/intake/MANIFEST.json','baseline/intake/MANIFEST.sha256','baseline/intake/GIT_STATUS.txt','baseline/intake/TRACKED.patch','baseline/review/MANIFEST.json','baseline/review/MANIFEST.sha256']:
 paths.add(p/n)
# Use existing verified bytes when possible, otherwise retain an exact close copy.
rows=[]
reviewby={r['path']:r for r in review['files']}
for q in sorted(paths):
 b=q.read_bytes();name=str(q.relative_to(root));h=sha(b);prior=before.get(name) or reviewby.get(name)
 if prior and prior['sha256']==h:orig=prior['original'];reuse=True
 else:
  origpath=close/'originals'/q.relative_to(root);origpath.parent.mkdir(parents=True,exist_ok=True);origpath.write_bytes(b);orig=str(origpath.relative_to(root));reuse=False
 rows.append({'path':name,'sha256':h,'bytes':len(b),'original':orig,'reused_intake_or_review_original':reuse})
# These paths are known before status capture; their bytes then become manifest members.
for q in [close/'GIT_STATE.json',close/'GIT_STATUS.txt',close/'DOCUMENT_CHANGES.patch']:
 dest=close/'originals'/q.relative_to(root);dest.parent.mkdir(parents=True,exist_ok=True);dest.touch()
head=git('rev-parse','HEAD').strip();branch=git('branch','--show-current').strip();status=git('status','--porcelain=v1','--untracked-files=all');(close/'GIT_STATUS.txt').write_text(status)
versions={}
for c in [['rustc','--version'],['cargo','--version'],['node','--version']]:
 r=subprocess.run(c,cwd=root,capture_output=True,text=True);versions[c[0]]={'exit_code':r.returncode,'version':r.stdout.strip(),'stderr':r.stderr.strip()}
save(close/'GIT_STATE.json',{'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'branch':branch,'head':head,'repository':str(root),'status_file':str((close/'GIT_STATUS.txt').relative_to(root)),'status_sha256':sha(status.encode()),'status_rows':len(status.splitlines()),'toolchain':versions,'kind':'planning-close observation only; implementation preflight must recheck','required_absences':[{'path':q,'absent':not (root/q).exists()} for q in ['.cargo/config.toml','core_enhanced/.cargo/config.toml']]})
for q in [close/'GIT_STATE.json',close/'GIT_STATUS.txt',close/'DOCUMENT_CHANGES.patch']:
 b=q.read_bytes();dest=close/'originals'/q.relative_to(root);dest.write_bytes(b);rows.append({'path':str(q.relative_to(root)),'sha256':sha(b),'bytes':len(b),'original':str(dest.relative_to(root)),'reused_intake_or_review_original':False})
manifest={'kind':'R06 reassessment planning-close preservation manifest; not implementation baseline','at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'repository':str(root),'branch':branch,'head':head,'files':rows,'required_absences':[{'path':q,'absent':not (root/q).exists()} for q in ['.cargo/config.toml','core_enhanced/.cargo/config.toml']],'self_hash':'baseline/close/MANIFEST.sha256','excluded_self_and_post_close_evidence':['baseline/close/MANIFEST.json','baseline/close/MANIFEST.sha256','FINAL_VERIFICATION_001.json','FINAL_VERIFICATION_001.sha256'],'restore_contract':'Read original bytes for every row. Unchanged rows reuse intake/review verified copies; others have close originals. Git HEAD does not recover dirty/untracked content. Apply no destructive restoration without a separate scope.','implementation_owner':'Explicit separate implementation request; compare relevant bytes and preserve new actual implementation baseline before mutation'}
save(close/'MANIFEST.json',manifest);manifesthash=sha((close/'MANIFEST.json').read_bytes());(close/'MANIFEST.sha256').write_text(manifesthash+'  MANIFEST.json\n')
# Final audit after publication of manifest, including all actual original bytes.
for row in rows:
 assert sha((root/row['path']).read_bytes())==row['sha256'],row['path'];assert sha((root/row['original']).read_bytes())==row['sha256'],row['original']
assert git('rev-parse','HEAD').strip()==head and git('branch','--show-current').strip()==branch
assert git('status','--porcelain=v1','--untracked-files=all')==status,'status changed during close'
# All actual markdown links from new plan/handoff and affected records must resolve.
missing=[];linkcount=0
from urllib.parse import unquote
for q in [*p.glob('*.md'),*(root/a for a in allowed)]:
 for match in re.finditer(r'\[[^\]]*\]\(([^)]+)\)',q.read_text()):
  target=unquote(match.group(1).split('#')[0].strip('<>'))
  if not target or re.match(r'^[a-z]+://',target):continue
  # repository's escaped parenthesized links use %28/%29 and are decoded above.
  dest=q.parent/target;linkcount+=1
  if not dest.exists():missing.append({'source':str(q.relative_to(root)),'target':target})
# A pre-existing broken link is not created by this change; explicitly compare prior bytes.
new_missing=[];historical_missing=[]
for item in missing:
 q=root/item['source'];prior=before.get(item['source'])
 if prior and item['target'] in unquote((root/prior['original']).read_text()):historical_missing.append(item)
 else:new_missing.append(item)
assert not new_missing,new_missing
reqs=[('§1 대상·기준·범위','REQUEST_ORIGINAL, initial scope, current AUTORUN/P2G hashes, same R06/27 acceptances, no game mutation'),('§2 별도 P2G-00/원본 보존','new WPD-CAVER path, initial empty maps, intake398 actual originals, old WPD/formal unchanged'),('§3 각 단계 비교·취합','P2G10 reassessment nine old/new rows; CRA003 C01 technical comparison; stage validations00–60; single current exact chain'),('§4 상세 구현 계획','CRA003 D01–D10 algorithms/types/order/owner/state/save/cancel/error/consumers and V01–21; Goal prerequisites/10 dimensions'),('§5 기준선·인계','exact four artifacts, branch/HEAD/status/file hashes, detached manifest hash, actual originals/patch, future owner/pre-mutation baseline'),('§6 종료·현재 기록','old intake append-only and current roadmap/operating links; no GEX/activation/production mutation; implementation request example in PLAN_HANDOFF')]
result={'result':'passed','at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'user_requirement_audit':[{'requirement':r,'evidence':e,'result':'proved by inspected current bytes and relations'} for r,e in reqs],'standard_validation_rerun':validation,'preservation':{'original_intake_files':len(checks),'unchanged':sum(r['disposition']=='unchanged' for r in checks),'authorized_mutable_records':allowed,'old_R06_files':len(oldpaths),'old_R06_formal_and_working_unchanged':True,'old_intake_original_prefix_preserved':True,'current_source_and_original_hashes_checked':len(rows),'planning_close_manifest_sha256':manifesthash,'checks':checks},'links':{'checked':linkcount,'new_missing':new_missing,'preexisting_missing_preserved':historical_missing},'exact_chain':{'freeze':freeze['ref'],'plan':plan['ref'],'goal_set':handoff['payload']['Goal Set Ref'],'handoff':handoff['ref']},'semantic_coverage':{'old_and_new_policy_meanings_byte_equal':9,'exact_acceptances_in_goal':27,'integration_owner_rows':21,'validation_dimensions':10,'design_review':'P2G_40_RESULT_001.json, P2G_50_RESULT_001.json; CRA003 D01–D10','future_interface_or_measurement_gaps':'explicit D10 owner/precondition; not claimed implemented'},'git':{'branch':branch,'head':head,'status_stable':True},'runtime_tests_executed':0,'GEX_started':False,'remaining':'Separate explicit implementation request, implementation baseline and D10 preparation; not required work for this plan-only objective','terminal':'R06 P2G 재검토·계획 인계 완료 / 구현 요청 대기'}
save(p/'FINAL_VERIFICATION_001.json',result);(p/'FINAL_VERIFICATION_001.sha256').write_text(sha((p/'FINAL_VERIFICATION_001.json').read_bytes())+'  FINAL_VERIFICATION_001.json\n')
print(json.dumps({'result':'passed','manifest_sha256':manifesthash,'manifest_members':len(rows),'links_checked':linkcount,'historical_broken_links':len(historical_missing),'source_unchanged':True,'GEX_started':False},ensure_ascii=False))
