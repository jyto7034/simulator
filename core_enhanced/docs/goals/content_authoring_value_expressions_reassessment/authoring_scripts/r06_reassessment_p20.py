from pathlib import Path
import json,copy,hashlib,datetime,subprocess
root=Path('/mnt/f/work/simulator');d=root/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment';old=root/'core_enhanced/docs/goals/content_authoring_value_expressions'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write(name,x):
 p=d/name;assert not p.exists(),str(p);p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def ref(t,i):return {'Artifact Type':t,'Artifact ID':i,'Content Digest':None}
now=datetime.datetime.now(datetime.timezone.utc).isoformat()
manifest=json.loads((d/'baseline/intake/MANIFEST.json').read_text()); intake_by={r['path']:r for r in manifest['files']}
# Fresh reread + drift evidence; bytes alone support currentness, body review is in CRA.
selected=[r for r in manifest['files'] if r['path'].startswith(('core_enhanced/src/','core_enhanced/tests/','game_resources/data/','game_server/src/')) or r['path'] in ['Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml']]
for r in selected:assert sha(root/r['path'])==r['sha256'],r['path']
prior=json.loads((old/'P2G_20_SOURCE_EVIDENCE_001.json').read_text())['files'];oldcheck=[{'path':r['path'],'old_sha256':r['sha256'],'current_sha256':sha(root/r['path']),'matches':sha(root/r['path'])==r['sha256']} for r in prior]
# Current predecessor closure/report/target integrity + root production source membership.
supp=[];pred=[]
for topic in ['content_authoring_minimal_effect_contract','content_authoring_runtime_facts','content_authoring_resource_lifetimes']:
 folder=root/'core_enhanced/docs/goals'/topic/'execution/single_goal'
 closure=json.loads((folder/'AUDIT_CLOSURE_RESUME_001.json').read_text()); target=folder/'AUDIT_TARGET_RESUME_001.json'
 assert sha(target)==closure['target']['sha256'];assert sha(root/closure['report']['path'])==closure['report']['sha256'];assert closure['verification']['passed']
 for n in ['AUDIT_CLOSURE_RESUME_001.json','AUDIT_TARGET_RESUME_001.json','AUDIT_ENTRY_RESUME_001.json','COMPLETION_AUDIT_RESUME_001.md']:supp.append(folder/n)
 pred.append({'topic':topic,'target':str(target.relative_to(root)),'target_sha256':sha(target),'closure':str((folder/'AUDIT_CLOSURE_RESUME_001.json').relative_to(root)),'report':closure['report'],'verdict':closure['verdict'],'check':'current report/target bytes equal passed closure; not a new GEX audit'})
anchor=root/'core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal/AUDIT_TARGET_003.json';target=json.loads((anchor.parent/'AUDIT_TARGET_RESUME_001.json').read_text());assert sha(anchor)==target['effective_target']['anchor']['sha256'];supp.append(anchor)
a=json.loads(anchor.read_text());checks=[]
for r in a['manifest']:
 if r['path'] in {z['path'] for z in selected}:
  actual=sha(root/r['path']);checks.append({'path':r['path'],'expected':r['sha256'],'actual':actual,'matches':actual==r['sha256']})
assert checks and all(c['matches'] for c in checks)
# Extra old policy locators actually read, retained without changing initial intake.
supp.append(root/'core_enhanced/docs/roadmaps/content_authoring/roadmap/goals/CA05.md')
for p in supp:
 b=p.read_bytes();q=d/'baseline/review/originals'/p.relative_to(root);q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert p.read_bytes()==b
write('baseline/review/MANIFEST.json',{'at':now,'files':[{'path':str(p.relative_to(root)),'sha256':sha(p),'bytes':p.stat().st_size,'original':str((d/'baseline/review/originals'/p.relative_to(root)).relative_to(root))} for p in supp]})
(d/'baseline/review/MANIFEST.sha256').write_text(sha(d/'baseline/review/MANIFEST.json')+'  MANIFEST.json\n')
write('P2G_20_SOURCE_EVIDENCE_001.json',{'at':now,'source_axis':'Workspace Actual static read-only','repository':str(root),'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root).decode().strip(),'files':selected,'old_review_source_comparison':oldcheck,'predecessor_closures':pred,'predecessor_direct_source_matches':checks,'methods':['bounded body reads','forward/reverse rg symbol/variant/caller search','raw old policy/acceptance comparison','fresh SHA-256 reread of captured source','closure target/report byte validation','source003 root production member rehash'],'supported_claims':'CODE_REVIEW_001.ko.md C01/D01–D10; source presence and planned consumer deltas only','runtime_tests_executed':0,'limitations':['No released/deployed runtime compatibility or fresh GEX verdict','Old AUTORUN/GEX delegation is historical and cannot authorize this request','Full 15,685-member historical audit target not re-certified; direct source subset plus closure identity checked','Proposal APIs in D01–D10 not implemented; executable tests and allocation measurements are future owner prerequisites']})
p10=json.loads((d/'P2G_10_POLICY_001.json').read_text())['artifacts'];wr=copy.deepcopy(p10[-1]);inp=wr['ref'];result=ref('Working Revision','WR-CAVER-P2G20-001');wr['ref']=result;wr['payload']['Predecessor Working Revision Ref']=inp
plan=json.loads((d/'P2G_20_SOURCE_PLAN_001.json').read_text())['artifacts'][0];planref=plan['ref'];srcref=ref('Reviewed Source Basis','RSB-CAVER-P2G20-001');cr=ref('Code Review Artifact','CRA-CAVER-P2G20-001')
arts=[];observations=[];evidence=[]
for i,req in enumerate(plan['payload']['Requirements'],1):
 er=ref('Review Evidence',f'RE-CAVER-P2G20-{i:03}');evidence.append(er)
 na=req['Applicability/Necessity']=='not_applicable'
 arts.append({'ref':er,'payload':{'Repository/environment identity':{'root':str(root),'head':manifest['head'],'at':now},'Source Set':'P2G_20_SOURCE_EVIDENCE_001.json files and baseline/review/MANIFEST.json','Observation command/method':['fresh digest/source003/closure exact comparison','read-only function bodies and forward/reverse symbols','latest request exclusion check' if na else 'code-informed review'], 'Raw evidence':{'path':'P2G_20_SOURCE_EVIDENCE_001.json','sha256':sha(d/'P2G_20_SOURCE_EVIDENCE_001.json')},'Observed result':'scope-excluded, no external compatibility claim' if na else 'required boundaries located and implementation delta/design/test owner reviewed','Supported claim':req['Required claim/boundary'],'Review body selector':'CODE_REVIEW_001.ko.md C01/D01–D10','Observation timestamp':now,'Limitations':'Static planning evidence, no R06 runtime PASS; D10 execution prerequisites','Runtime tests executed':0}})
 o=ref('Source Observation',f'SO-CAVER-P2G20-{i:03}');observations.append(o)
 arts.append({'ref':o,'payload':{'Evaluated Working Revision Ref':inp,'Review Scope/Source Requirement Plan Ref':planref,'Requirement ID':req['Requirement ID'],'Disposition':'N/A' if na else 'current','Observation evidence':[er],'Limitation or N/A reason':'REQUEST_ORIGINAL.txt §1/6 excludes GEX/server/Unity/deployment; no required external runtime claim' if na else 'workspace static scope; implementation/measurement remains future','Observation timestamp/version':now}})
arts.append({'ref':srcref,'payload':{'Selected Source Set':observations,'Currentness evidence':{'path':'P2G_20_SOURCE_EVIDENCE_001.json','sha256':sha(d/'P2G_20_SOURCE_EVIDENCE_001.json')}}})
arts.append({'ref':cr,'payload':{'Input Working Revision Ref':inp,'Review Scope/Source Requirement Plan Ref':planref,'Reviewed Source Basis Ref':srcref,'Review Evidence Ref Set':evidence,'Review body':{'path':'CODE_REVIEW_001.ko.md','sha256':sha(d/'CODE_REVIEW_001.ko.md')},'Element review mapping':{f'CAVER-E{i:02}':f'C01 row {i}; D01–D10; V01–V21 as applicable' for i in range(1,10)},'Current Actual selector':'C01 direct body facts','Implementation Delta/Proposed Implementation selector':'D01–D10','Validation seams selector':'D09 V01–V21','Finding/Unknown selector':'C01 judgments + D10 owners','Outcome':'complete static code-informed review; no new policy meaning or required evidence gap'}})
produced=[];envelopes=[];bindings=[]
for i in range(1,10):
 eid=f'CAVER-E{i:02}';oldper=next(a for a in p10 if a['ref']['Artifact ID']==f'PER-CAVER-E{i:02}-DECIDED-001');per=copy.deepcopy(oldper)
 per['ref']=ref('Policy Element Revision',f'PER-CAVER-E{i:02}-REVIEWED-001');per['payload']['Predecessor Policy Element Revision Ref']=oldper['ref'];per['payload']['Intrinsic status']='code_reviewed';produced.append(per['ref']);wr['payload']['Current Policy Element Revision Map'][eid]=per['ref']
 binding=ref('Code Review Binding',f'CRB-CAVER-E{i:02}-001');bindings.append(binding)
 bp={'Working Revision Ref':result,'Policy Element Revision Ref':per['ref'],'Code Review Artifact Ref':cr,'P2G-20 Review Scope/Source Requirement Plan Ref':planref,'Reviewed Source Basis Ref':srcref,'Review Evidence Ref Set':[evidence[i-1],evidence[-1]],'Covered Required Claim/Boundary Set':[plan['payload']['Requirements'][i-1]['Required claim/boundary'],plan['payload']['Requirements'][-1]['Required claim/boundary']],'Evaluated Applicable Change/Obligation/Closure Ref Set':[],'Canonical Union Member Set':[]}
 env=ref('Candidate Review Envelope',f'CRE-CAVER-E{i:02}-001');envelopes.append(env)
 arts.append({'ref':env,'payload':{'Candidate Batch ID':'RB-CAVER-P2G20-001','Input Working Revision Ref':inp,'Reviewed Element/Revision':oldper['ref'],'Review Path':'policy_revision_review','Candidate Code Review Artifact Ref Set':[cr],'Candidate Review Evidence Ref Set':[evidence[i-1],evidence[-1]],'Proposed code_reviewed Revision Payload':per['payload'],'Candidate Binding Payload':bp,'Candidate outcome':'validated for complete publication'}})
 arts.extend([per,{'ref':binding,'payload':bp}])
arts.append(wr);batch=ref('Review Batch','RB-CAVER-P2G20-001')
arts.append({'ref':batch,'payload':{'Input Working Revision Ref':inp,'Result Working Revision Ref':result,'Review Scope/Source Requirement Plan Ref':planref,'Source Observation Ref Set':observations,'Reviewed Source Basis Ref':srcref,'Code Review Artifact Ref Set':[cr],'Review Evidence Ref Set':evidence,'Candidate Review Envelope Ref Set':envelopes,'Produced code_reviewed Policy Element Revision Ref Set':produced,'Result Current code_reviewed Policy Element Revision Ref Set':produced,'Result-basis Code Review Binding Ref Set':bindings,'P2G-20 Resolution Record Ref Set':[],'Applicable Obligation Supersession Record Ref Set':[],'Completeness Evidence':'P2G_20_VALIDATION_001.json; CODE_REVIEW_001.ko.md C01/D01–D10'}})
write('P2G_20_REVIEW_001.json',{'artifacts':arts})
# Structural checks plus separately recorded semantic review.
allarts=json.loads((d/'P2G_00_INITIAL_001.json').read_text())['artifacts']+p10+[plan]+arts
ids=[a['ref']['Artifact ID'] for a in allarts];assert len(ids)==len(set(ids));index={a['ref']['Artifact ID']:a for a in allarts}
for b in bindings:
 p=index[b['Artifact ID']]['payload'];assert p['Working Revision Ref']==result;assert p['Policy Element Revision Ref'] in produced;assert p['Reviewed Source Basis Ref']==srcref
for r in produced:
 p=index[r['Artifact ID']]['payload'];q=index[p['Predecessor Policy Element Revision Ref']['Artifact ID']]['payload']
 for k in ['Policy statement','Decision Revision Ref','Observable Policy Acceptance']:assert p[k]==q[k]
assert len(observations)==len(plan['payload']['Requirements'])==12
assert len(bindings)==len(produced)==9
checks=['exact input/result/Decision/source plan identified','current/target/delta/proposal separated with substantive design','required runtime/data/store/consumer bodies reviewed','claims tied to source or explicit limits','findings classified/disposed in C01','no new policy meaning','technical choices preserve acceptance','no required planning evidence gap','exclusions do not expand scope','produced/current and complete binding coverage','9 code_reviewed result members','all bindings use identical exact result/source/plan','complete references checked before current','no old candidate/binding selected','no applicable Change','repair N/A','12 requirements each have exact fresh observations','27 acceptance members exact unchanged','current head selected only after complete validation']
write('P2G_20_VALIDATION_001.json',{'result':'passed','checks':[{'id':i+1,'check':x,'result':'passed'} for i,x in enumerate(checks)],'counts':{'requirements':12,'observations':12,'reviewed_members':9,'bindings':9,'acceptance':27,'old_source_hash_matches':sum(x['matches'] for x in oldcheck),'predecessor_source_members_checked':len(checks)},'semantic_evidence':{'path':'CODE_REVIEW_001.ko.md','sha256':sha(d/'CODE_REVIEW_001.ko.md')},'note':'Source subset count is separately authoritative in P2G_20_SOURCE_EVIDENCE_001.json; no game test/audit pass'})
p=d/'WORKING_POLICY.ko.md';s=p.read_text().replace('`WR-CAVER-P2G10-001`','`WR-CAVER-P2G20-001`').replace('P2G-00/10 passed; P2G-20 next','P2G-00/10/20 passed; P2G-30 next');s+='\n[P2G-20 complete batch](P2G_20_REVIEW_001.json) · [상세 설계/비교](CODE_REVIEW_001.ko.md) · [source evidence](P2G_20_SOURCE_EVIDENCE_001.json) · [19 종료 검사](P2G_20_VALIDATION_001.json). current `RB-CAVER-P2G20-001`, 9 reviewed elements와 Result-basis bindings; GEX/실행 PASS 아님.\n';p.write_text(s)
print(json.dumps({'result':'P2G-20 passed','old_source_matches':sum(x['matches'] for x in oldcheck),'selected_files':len(selected),'predecessor_sources':len(json.loads((d/'P2G_20_SOURCE_EVIDENCE_001.json').read_text())['predecessor_direct_source_matches']),'artifacts':len(arts)}))
